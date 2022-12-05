use anyhow::{anyhow, bail, ensure, Context, Result};
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    time::{sleep, timeout},
};
use tracing::{debug, error, warn};

use crate::{env::*, node::VSOCK_PATH};

const BABEL_START_TIMEOUT: Duration = Duration::from_secs(30);
const BABEL_STOP_TIMEOUT: Duration = Duration::from_secs(15);
const SOCKET_TIMEOUT: Duration = Duration::from_secs(5);
const BABEL_VSOCK_PORT: u32 = 42;

#[derive(Debug)]
pub enum BabelConnection {
    Closed,
    Open { babel_conn: UnixStream },
}

impl BabelConnection {
    /// Establishes a new connection to the VM. Note that this fails if the VM hasn't started yet.
    /// It also initializes that connection by sending the opening message. Therefore, if this
    /// function succeeds the connection is guaranteed to be writeable at the moment of returning.
    pub async fn connect(node_id: &uuid::Uuid) -> Result<UnixStream> {
        use std::io::ErrorKind::WouldBlock;

        // We are going to connect to the central socket for this VM. Later we will specify which
        // port we want to talk to.
        let socket = format!(
            "{}/firecracker/{node_id}/root{VSOCK_PATH}",
            CHROOT_PATH.to_string_lossy()
        );
        debug!("Connecting to node at `{socket}`");

        // We need to implement retrying when reading from the socket, as it may take a little bit
        // of time for the socket file to get created on disk, because this is done asynchronously
        // by Firecracker.
        let start = std::time::Instant::now();
        let elapsed = || std::time::Instant::now() - start;
        let mut conn = loop {
            let maybe_conn = UnixStream::connect(&socket).await;
            match maybe_conn {
                Ok(conn) => break Ok(conn),
                Err(e) if elapsed() < BABEL_START_TIMEOUT => {
                    debug!("No socket file yet, retrying in 5 seconds: {e}");
                    sleep(std::time::Duration::from_secs(5)).await;
                }
                Err(e) => break Err(e),
            };
        }
        .context("Failed to connect to babel bus")?;
        // For host initiated connections we have to send a message to the guess socket that
        // contains the port number. As a response we expect a message of the format `Ok <num>\n`.
        // We check this by asserting that the first message received starts with `OK`. Popping
        // this message here prevents us from having to check for the opening message elsewhere
        // were we expect the response to be valid json.
        let open_message = format!("CONNECT {BABEL_VSOCK_PORT}\n");
        debug!("Sending open message : `{open_message:?}`.");
        timeout(SOCKET_TIMEOUT, conn.write(open_message.as_bytes())).await??;
        debug!("Sent open message.");
        let mut sock_opened_buf = [0; 20];
        let resp = async {
            loop {
                conn.readable().await?;
                match conn.try_read(&mut sock_opened_buf) {
                    Ok(0) => {
                        error!("Socket responded to open message with empty message :(");
                        bail!("Socket responded to open message with empty message :(");
                    }
                    Ok(n) => {
                        let sock_opened_msg = std::str::from_utf8(&sock_opened_buf[..n]).unwrap();
                        let msg_valid = sock_opened_msg.starts_with("OK ");
                        ensure!(msg_valid, "Invalid opening message for new socket");
                        break;
                    }
                    // Ignore false-positive readable events
                    Err(e) if e.kind() == WouldBlock => continue,
                    Err(e) => bail!("Establishing socket failed with `{e}`"),
                }
            }
            Ok(conn)
        };
        timeout(SOCKET_TIMEOUT, resp).await?
    }

    /// Returns the open babel connection, if there is one.
    pub fn conn_mut(&mut self) -> Option<&mut UnixStream> {
        use BabelConnection::*;
        match self {
            Closed => None,
            Open { babel_conn } => Some(babel_conn),
        }
    }

    /// Tries to return the babel connection, and if there isn't one, returns an error message.
    pub fn try_conn_mut(&mut self) -> Result<&mut UnixStream> {
        self.conn_mut()
            .ok_or_else(|| anyhow!("Tried to get babel connection while there isn't one"))
    }

    /// Waits for the socket to become readable, then writes the data as json to the socket. The max
    /// time that is allowed to elapse  is `SOCKET_TIMEOUT`.
    pub async fn write_data(&mut self, data: impl serde::Serialize) -> Result<()> {
        use std::io::ErrorKind::WouldBlock;

        let data = serde_json::to_string(&data)?;
        let unix_stream = self.try_conn_mut()?;
        let write_data = async {
            loop {
                // Wait for the socket to become ready to write to.
                unix_stream.writable().await?;
                // Try to write data, this may still fail with `WouldBlock` if the readiness event
                // is a false positive.
                match unix_stream.try_write(data.as_bytes()) {
                    Ok(_) => break,
                    Err(e) if e.kind() == WouldBlock => continue,
                    Err(e) => bail!("Writing socket failed with `{e}`"),
                }
            }
            Ok(())
        };
        timeout(SOCKET_TIMEOUT, write_data).await?
    }

    /// Waits for the socket to become readable, then reads data from it. The max time that is
    /// allowed to elapse (per read) is `SOCKET_TIMEOUT`. When data is sent over the vsock, this
    /// data is parsed as json and returned as the requested type.
    pub async fn read_data<D: serde::de::DeserializeOwned>(&mut self) -> Result<D> {
        use std::io::ErrorKind::WouldBlock;

        let unix_stream = self.try_conn_mut()?;
        let read_data = async {
            loop {
                // Wait for the socket to become ready to read from.
                unix_stream.readable().await?;
                let mut data = vec![0; 4194304];
                // Try to read data, this may still fail with `WouldBlock`
                // if the readiness event is a false positive.
                match unix_stream.try_read(&mut data) {
                    Ok(n) => {
                        data.resize(n, 0);
                        let s = std::str::from_utf8(&data)?;
                        return Ok(serde_json::from_str(s)?);
                    }
                    Err(e) if e.kind() == WouldBlock => continue,
                    Err(e) => bail!("Writing socket failed with `{e}`"),
                }
            }
        };
        timeout(SOCKET_TIMEOUT, read_data).await?
    }

    pub async fn wait_for_disconnect(&mut self, node_id: &uuid::Uuid) {
        if let Some(babel_conn) = self.conn_mut() {
            // We can verify successful shutdown success by checking whether we can read
            // into a buffer of nonzero length. If the stream is closed, the number of
            // bytes read should be zero.
            let read = timeout(BABEL_STOP_TIMEOUT, babel_conn.read(&mut [0])).await;
            match read {
                // Successful shutdown in this case
                Ok(Ok(0)) => debug!("Node {} gracefully shut down", node_id),
                // The babel stream has more to say...
                Ok(Ok(_)) => warn!("Babel stream returned data instead of closing"),
                // The read timed out. It is still live so the node did not shut down.
                Err(timeout_err) => warn!("Babel shutdown timeout: {timeout_err}"),
                // Reading returned _before_ the timeout, but was otherwise unsuccessful.
                // Could happpen I guess? Lets log the error.
                Ok(Err(io_err)) => error!("Babel stream broke on closing: {io_err}"),
            }
        } else {
            warn!("Terminating node has no babel conn!");
        }
    }
}
