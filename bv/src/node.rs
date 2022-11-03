use anyhow::{anyhow, Context, Result};
use firec::config::JailerMode;
use firec::Machine;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};
use tokio::{
    fs::DirBuilder,
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    time::{sleep, timeout},
};
use tracing::{debug, error, instrument, trace, warn};
use uuid::Uuid;

use crate::{
    node_data::{NodeData, NodeStatus},
    utils::{get_process_pid, run_cmd},
};

#[derive(Debug)]
pub struct Node {
    pub data: NodeData,
    machine: Machine<'static>,
    babel_conn: Connection,
}

#[derive(Debug)]
enum Connection {
    Closed,
    Open { babel_conn: UnixStream },
}

impl Connection {
    /// Returns the open babel connection, if there is one.
    fn conn_mut(&mut self) -> Option<&mut UnixStream> {
        use Connection::*;
        match self {
            Closed => None,
            Open { babel_conn } => Some(babel_conn),
        }
    }

    /// Tries to return the babel connection, and if there isn't one, returns an error message.
    fn try_conn_mut(&mut self) -> Result<&mut UnixStream> {
        self.conn_mut()
            .ok_or_else(|| anyhow::anyhow!("Tried to get babel connection while there isn't one"))
    }
}

// FIXME: Hardcoding everything for now.
pub const FC_BIN_NAME: &str = "firecracker";
const KERNEL_PATH: &str = "/var/lib/blockvisor/debian-vmlinux";
const DATA_PATH: &str = "/var/lib/blockvisor/data.img";
const CHROOT_PATH: &str = "/var/lib/blockvisor";
const FC_BIN_PATH: &str = "/usr/bin/firecracker";
const FC_SOCKET_PATH: &str = "/firecracker.socket";
const VSOCK_PATH: &str = "/vsock.socket";
const VSOCK_GUEST_CID: u32 = 3;
const BABEL_VSOCK_PORT: u32 = 42;

const BABEL_START_TIMEOUT: Duration = Duration::from_secs(30);
const BABEL_STOP_TIMEOUT: Duration = Duration::from_secs(15);
const SOCKET_TIMEOUT: Duration = Duration::from_secs(5);

lazy_static::lazy_static! {
    static ref IMAGE_CACHE_DIR: PathBuf = home::home_dir()
        .map(|p| p.join(".cache"))
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("blockvisor")
        .join("images");
}

impl Node {
    /// Creates a new node according to specs.
    #[instrument]
    pub async fn create(data: NodeData) -> Result<Self> {
        let config = Node::create_config(&data)?;
        Node::copy_data_image(&data).await?;
        let machine = firec::Machine::create(config).await?;

        data.save().await?;

        Ok(Self {
            data,
            machine,
            babel_conn: Connection::Closed,
        })
    }

    /// Returns node previously created on this host.
    #[instrument]
    pub async fn connect(data: NodeData, babel_conn: Option<UnixStream>) -> Result<Self> {
        let config = Node::create_config(&data)?;
        let cmd = data.id.to_string();
        let (state, babel_conn) = match get_process_pid(FC_BIN_NAME, &cmd) {
            Ok(pid) => {
                let c = babel_conn.ok_or_else(|| anyhow!("Node running, need babel_conn"))?;
                (
                    firec::MachineState::RUNNING { pid },
                    Connection::Open { babel_conn: c },
                )
            }
            Err(_) => (firec::MachineState::SHUTOFF, Connection::Closed),
        };
        let machine = firec::Machine::connect(config, state).await;

        Ok(Self {
            data,
            machine,
            babel_conn,
        })
    }

    /// Returns the node's `id`.
    pub fn id(&self) -> Uuid {
        self.data.id
    }

    /// Starts the node.
    #[instrument(skip(self))]
    pub async fn start(&mut self) -> Result<()> {
        self.machine.start().await?;
        let babel_conn = match Self::conn(self.id()).await {
            Ok(conn) => Ok(conn),
            Err(_) => {
                // Extremely scientific retrying mechanism
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                Self::conn(self.id()).await
            }
        }?;
        self.babel_conn = Connection::Open { babel_conn };
        let resp = self.send(BabelRequest::Ping).await;
        if !matches!(resp, Ok(BabelResponse::Pong)) {
            tracing::warn!("Ping request did not respond with `Pong`, but `{resp:?}`");
        }
        self.data.expected_status = NodeStatus::Running;
        self.data.save().await
    }

    /// Returns the actual status of the node.
    pub fn status(&self) -> NodeStatus {
        self.data.status()
    }

    /// Returns the expected status of the node.
    pub fn expected_status(&self) -> NodeStatus {
        self.data.expected_status
    }

    /// Stops the running node.
    #[instrument(skip(self))]
    pub async fn stop(&mut self) -> Result<()> {
        match self.machine.state() {
            firec::MachineState::SHUTOFF => {}
            firec::MachineState::RUNNING { .. } => {
                let mut shutdown_success = true;
                if let Err(err) = self.machine.shutdown().await {
                    trace!("Graceful shutdown failed: {err}");

                    // FIXME: Perhaps we should be just bailing out on this one?
                    if let Err(err) = self.machine.force_shutdown().await {
                        trace!("Forced shutdown failed: {err}");
                        shutdown_success = false;
                    }
                }

                if shutdown_success {
                    if let Some(babel_conn) = self.babel_conn.conn_mut() {
                        // We can verify successful shutdown success by checking whether we can read
                        // into a buffer of nonzero length. If the stream is closed, the number of
                        // bytes read should be zero.
                        let read = timeout(BABEL_STOP_TIMEOUT, babel_conn.read(&mut [0])).await;
                        match read {
                            // Successful shutdown in this case
                            Ok(Ok(0)) => debug!("Node {} gracefully shut down", self.id()),
                            // The babel stream has more to say...
                            Ok(Ok(_)) => warn!("Babel stream returned data instead of closing"),
                            // The read timed out. It is still live so the node did not shut down.
                            Err(timeout_err) => warn!("Babel shutdown timeout: {timeout_err}"),
                            // Reading returned _before_ the timeout, but was otherwise unsuccessful.
                            // Could happpen I guess? Lets log the error.
                            Ok(Err(io_err)) => error!("Babel stream broke on closing: {io_err}"),
                        }
                    } else {
                        tracing::warn!("Terminating node has no babel conn!");
                    }
                }
            }
        }
        self.data.expected_status = NodeStatus::Stopped;
        self.data.save().await?;
        self.babel_conn = Connection::Closed;

        // FIXME: for some reason firecracker socket is not created by
        // consequent start command if we do not wait a bit here
        sleep(Duration::from_secs(10)).await;

        Ok(())
    }

    /// Deletes the node.
    #[instrument(skip(self))]
    pub async fn delete(self) -> Result<()> {
        self.machine.delete().await?;
        self.data.delete().await
    }

    async fn copy_data_image(data: &NodeData) -> Result<()> {
        // Workaround: using system `cp` because unlike std one it works well with sparse files
        // https://github.com/rust-lang/rust/issues/58635
        // firec will not overwrite the file if it's already present
        //
        // TODO: we need to create a new data image according to spec
        // At the time of writing we use the same 10 Gb empty image for every node
        let data_dir = Path::new(CHROOT_PATH)
            .join(FC_BIN_NAME)
            .join(data.id.to_string())
            .join("root");
        DirBuilder::new().recursive(true).create(&data_dir).await?;
        run_cmd("cp", &[DATA_PATH, &data_dir.to_string_lossy()]).await?;

        Ok(())
    }

    fn create_config(data: &NodeData) -> Result<firec::config::Config<'static>> {
        let kernel_args = format!(
            "console=ttyS0 reboot=k panic=1 pci=off random.trust_cpu=on \
            ip={}::{}:255.255.255.240::eth0:on blockvisor.node={}",
            data.network_interface.ip, data.network_interface.gateway, data.id,
        );
        let iface =
            firec::config::network::Interface::new(data.network_interface.name.clone(), "eth0");

        let root_fs_path = IMAGE_CACHE_DIR.join(data.image.clone());
        let root_fs_path = if root_fs_path.exists() {
            root_fs_path
        } else {
            // TODO: download from remote images repository into cache dir
            // return error if not present in remote
            // for now we just return a default one
            IMAGE_CACHE_DIR.join("debian.ext4")
        };

        let config = firec::config::Config::builder(Some(data.id), Path::new(KERNEL_PATH))
            // Jailer configuration.
            .jailer_cfg()
            .chroot_base_dir(Path::new(CHROOT_PATH))
            .exec_file(Path::new(FC_BIN_PATH))
            .mode(JailerMode::Tmux(Some(data.name.clone().into())))
            .build()
            // Machine configuration.
            .machine_cfg()
            .vcpu_count(1)
            .mem_size_mib(8192)
            .build()
            // Add root drive.
            .add_drive("root", root_fs_path)
            .is_root_device(true)
            .build()
            // Add data drive.
            .add_drive("data", Path::new(DATA_PATH))
            .build()
            // Network configuration.
            .add_network_interface(iface)
            // Rest of the configuration.
            .socket_path(Path::new(FC_SOCKET_PATH))
            .kernel_args(kernel_args)
            .vsock_cfg(VSOCK_GUEST_CID, Path::new(VSOCK_PATH))
            .build();

        Ok(config)
    }

    /// Returns the height of the blockchain (in blocks).
    pub async fn height(&mut self) -> Result<u64> {
        self.call_method("height").await
    }

    /// Returns the block age of the blockchain (in seconds).
    pub async fn block_age(&mut self) -> Result<u64> {
        self.call_method("block_age").await
    }

    /// Returns the name of the node. This is usually some random generated name that you may use
    /// to recognise the node, but the purpose may vary per blockchain.
    /// ### Example
    /// `chilly-peach-kangaroo`
    pub async fn name(&mut self) -> Result<String> {
        self.call_method("name").await
    }

    /// The address of the node. The meaning of this varies from blockchain to blockchain.
    /// ### Example
    /// `/p2p/11Uxv9YpMpXvLf8ZyvGWBdbgq3BXv8z1pra1LBqkRS5wmTEHNW3`
    pub async fn address(&mut self) -> Result<String> {
        self.call_method("address").await
    }

    /// Returns whether this node is in consensus or not.
    pub async fn consensus(&mut self) -> Result<bool> {
        self.call_method("consensus").await
    }

    /// This function calls babel by sending a blockchain command using the specified method name.
    pub async fn call_method<T>(&mut self, method: &str) -> Result<T>
    where
        T: FromStr,
        <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
    {
        let request = BabelRequest::BlockchainCommand { name: method };
        let resp: BabelResponse = self.send(request).await?;
        let inner = match resp {
            BabelResponse::BlockchainResponse { value } => {
                value.parse().context(format!("Could not parse {method}"))?
            }
            e => anyhow::bail!("Unexpected BabelResponse for `{method}`: `{e:?}`"),
        };
        Ok(inner)
    }

    /// Returns the methods that are supported by this blockchain. Calling any method on this
    /// blockchain that is not listed here will result in an error being returned.
    pub async fn capabilities(&mut self) -> Result<Vec<String>> {
        let request = BabelRequest::ListCapabilities;
        let resp: BabelResponse = self.send(request).await?;
        let height = match resp {
            BabelResponse::ListCapabilities(caps) => caps,
            e => anyhow::bail!("Unexpected BabelResponse for `height`: `{e:?}`"),
        };
        Ok(height)
    }

    /// This function combines the capabilities from `write_data` and `read_data` to allow you to
    /// send some request and then obtain a response back.
    async fn send<S: serde::ser::Serialize, D: serde::de::DeserializeOwned>(
        &mut self,
        data: S,
    ) -> Result<D> {
        self.write_data(data).await?;
        self.read_data().await
    }

    /// Waits for the socket to become readable, then writes the data as json to the socket. The max
    /// time that is allowed to elapse  is `SOCKET_TIMEOUT`.
    async fn write_data(&mut self, data: impl serde::Serialize) -> Result<()> {
        use std::io::ErrorKind::WouldBlock;

        let data = serde_json::to_string(&data)?;
        let babel_conn = self.babel_conn.try_conn_mut()?;
        let write_data = async {
            loop {
                // Wait for the socket to become ready to write to.
                babel_conn.writable().await?;
                // Try to write data, this may still fail with `WouldBlock` if the readiness event
                // is a false positive.
                match babel_conn.try_write(data.as_bytes()) {
                    Ok(_) => break,
                    Err(e) if e.kind() == WouldBlock => continue,
                    Err(e) => anyhow::bail!("Writing socket failed with `{e}`"),
                }
            }
            Ok(())
        };
        timeout(SOCKET_TIMEOUT, write_data).await?
    }

    /// Waits for the socket to become readable, then reads data from it. The max time that is
    /// allowed to elapse (per read) is `SOCKET_TIMEOUT`. When data is sent over the vsock, this
    /// data is parsed as json and returned as the requested type.
    async fn read_data<D: serde::de::DeserializeOwned>(&mut self) -> Result<D> {
        use std::io::ErrorKind::WouldBlock;

        let babel_conn = self.babel_conn.try_conn_mut()?;
        let read_data = async {
            loop {
                // Wait for the socket to become ready to read from.
                babel_conn.readable().await?;
                let mut data = vec![0; 2048];
                // Try to read data, this may still fail with `WouldBlock`
                // if the readiness event is a false positive.
                match babel_conn.try_read(&mut data) {
                    Ok(n) => {
                        data.resize(n, 0);
                        let s = std::str::from_utf8(&data)?;
                        return Ok(serde_json::from_str(s)?);
                    }
                    Err(e) if e.kind() == WouldBlock => continue,
                    Err(e) => anyhow::bail!("Writing socket failed with `{e}`"),
                }
            }
        };
        timeout(SOCKET_TIMEOUT, read_data).await?
    }

    /// Establishes a new connection to the VM. Note that this fails if the VM hasn't started yet.
    /// It also initializes that connection by sending the opening message. Therefore, if this
    /// function succeeds the connection is guaranteed to be writeable at the moment of returning.
    pub async fn conn(node_id: uuid::Uuid) -> Result<UnixStream> {
        use std::io::ErrorKind::WouldBlock;

        // We are going to connect to the central socket for this VM. Later we will specify which
        // port we want to talk to.
        let socket = format!("{CHROOT_PATH}/firecracker/{node_id}/root{VSOCK_PATH}");
        tracing::debug!("Connecting to node at `{socket}`");

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
                    tracing::debug!("No socket file yet, retrying in 5 seconds: {e}");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
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
        tracing::debug!("Sending open message : `{open_message:?}`.");
        timeout(SOCKET_TIMEOUT, conn.write(open_message.as_bytes())).await??;
        tracing::debug!("Sent open message.");
        let mut sock_opened_buf = [0; 20];
        let resp = async {
            loop {
                conn.readable().await?;
                match conn.try_read(&mut sock_opened_buf) {
                    Ok(0) => {
                        tracing::error!("Socket responded to open message with empty message :(");
                        anyhow::bail!("Socket responded to open message with empty message :(");
                    }
                    Ok(n) => {
                        let sock_opened_msg = std::str::from_utf8(&sock_opened_buf[..n]).unwrap();
                        let msg_valid = sock_opened_msg.starts_with("OK ");
                        anyhow::ensure!(msg_valid, "Invalid opening message for new socket");
                        break;
                    }
                    // Ignore false-positive readable events
                    Err(e) if e.kind() == WouldBlock => continue,
                    Err(e) => anyhow::bail!("Establishing socket failed with `{e}`"),
                }
            }
            Ok(conn)
        };
        timeout(SOCKET_TIMEOUT, resp).await?
    }
}

/// Each request that comes over the VSock to babel must be a piece of JSON that can be
/// deserialized into this struct.
#[derive(Debug, serde::Serialize)]
pub enum BabelRequest<'a> {
    /// List the endpoints that are available for the current blockchain. These are extracted from
    /// the config, and just sent back as strings for now.
    ListCapabilities,
    /// Returns `Pong`. Useful to check for the liveness of the node.
    Ping,
    /// Send a request to the current blockchain. We can identify the way to do this from the
    /// config and forward the provided parameters.
    BlockchainCommand { name: &'a str },
}

#[derive(Debug, serde::Deserialize)]
enum BabelResponse {
    ListCapabilities(Vec<String>),
    Pong,
    BlockchainResponse { value: String },
    Error(String),
}
