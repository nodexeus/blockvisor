use anyhow::{Result, Context};
use firec::config::JailerMode;
use firec::Machine;
use std::{path::Path, time::Duration};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    time::{sleep, timeout},
};
use tracing::{instrument, trace};
use uuid::Uuid;

use crate::{
    node_data::{NodeData, NodeStatus},
    utils::get_process_pid,
};

#[derive(Debug)]
pub struct Node {
    pub data: NodeData,
    machine: Machine<'static>,
    babel_conn: UnixStream,
}

// FIXME: Hardcoding everything for now.
pub const FC_BIN_NAME: &str = "firecracker";
const KERNEL_PATH: &str = "/var/lib/blockvisor/debian-vmlinux";
const ROOT_FS: &str = "/var/lib/blockvisor/debian.ext4";
pub const CHROOT_PATH: &str = "/var/lib/blockvisor";
const FC_BIN_PATH: &str = "/usr/bin/firecracker";
const FC_SOCKET_PATH: &str = "/firecracker.socket";
const VSOCK_PATH: &str = "/vsock.socket";
const VSOCK_GUEST_CID: u32 = 3;
const BABEL_VSOCK_PORT: u32 = 42;
const BABEL_VSOCK_PATH: &str = "/var/lib/blockvisor/vsock.socket_42";

const BABEL_START_TIMEOUT: Duration = Duration::from_secs(30);
const BABEL_STOP_TIMEOUT: Duration = Duration::from_secs(15);

impl Node {
    /// Creates a new node with `id`.
    #[instrument]
    pub async fn create(data: NodeData) -> Result<Self> {
        let id = data.id;
        let config = Node::create_config(&data)?;
        let machine = firec::Machine::create(config).await?;
        let workspace_dir = machine.config().jailer_cfg().expect("").workspace_dir();
        let babel_socket_link = workspace_dir.join("vsock.socket_42");
        fs::hard_link(BABEL_VSOCK_PATH, babel_socket_link).await?;
        data.save().await?;

        Ok(Self {
            data,
            machine,
            babel_conn: Self::conn(id).await?,
        })
    }

    /// Returns node previously created on this host.
    #[instrument]
    pub async fn connect(data: NodeData, babel_conn: UnixStream) -> Result<Self> {
        // let babel_proxy = create_babel_proxy(babel_conn, data.id).await?;
        let config = Node::create_config(&data)?;
        let cmd = data.id.to_string();
        let state = match get_process_pid(FC_BIN_NAME, &cmd) {
            Ok(pid) => firec::MachineState::RUNNING { pid },
            Err(_) => firec::MachineState::SHUTOFF,
        };
        let machine = firec::Machine::connect(config, state).await;

        Ok(Self {
            data,
            machine,
            babel_conn,
        })
    }

    /// Returns the node's `id`.
    pub fn id(&self) -> &Uuid {
        &self.data.id
    }

    /// Starts the node.
    #[instrument(skip(self))]
    pub async fn start(&mut self) -> Result<()> {
        self.machine.start().await?;
        let mut buf = String::new();
        timeout(
            BABEL_START_TIMEOUT,
            self.babel_conn.read_to_string(&mut buf),
        )
        .await??;
        let json: serde_json::Value = buf.parse()?;
        if json.get("start_msg").is_none() {
            tracing::error!("Node did not start!");
            return Err(anyhow::anyhow!("Node did not start!"));
        }

        self.data.save().await
    }

    /// Returns the status of the node.
    pub async fn status(&self) -> Result<NodeStatus> {
        Ok(self.data.status())
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
                    let mut buf = String::new();
                    timeout(BABEL_STOP_TIMEOUT, self.babel_conn.read_to_string(&mut buf)).await??;
                    let json: serde_json::Value = buf.parse()?;
                    if json.get("start_msg").is_none() {
                        tracing::error!("Node did not stop!!");
                    }
                }
            }
        }
        self.data.save().await?;

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

    fn create_config(data: &NodeData) -> Result<firec::config::Config<'static>> {
        let kernel_args = format!(
            "console=ttyS0 reboot=k panic=1 pci=off random.trust_cpu=on \
            ip={}::{}:255.255.255.240::eth0:on blockvisor.node={}",
            data.network_interface.ip, data.network_interface.gateway, data.id,
        );
        let iface =
            firec::config::network::Interface::new(data.network_interface.name.clone(), "eth0");

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
        .add_drive("root", Path::new(ROOT_FS))
        .is_root_device(true)
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

    pub async fn height(&mut self) -> Result<u64> {
        let request = serde_json::json!({
            "BlockchainCommand": {
                "name": "height",
            },
        });

        let resp: serde_json::Value = self.rw(request).await?;
        let height = dbg!(dbg!(resp)
            .get("BlockchainResponse"))
            .unwrap()
            .as_object()
            .unwrap()
            .get("value")
            .ok_or_else(|| anyhow::anyhow!("No height returned from babel"))?
            .as_str()
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| anyhow::anyhow!("Height is not parseable as a number"))?;
        Ok(height)
    }

    // ///
    // async fn read_data<D: serde::de::DeserializeOwned>(&mut self) -> Result<D> {
    //     let mut buf = String::new();
        
    //     tracing::debug!("Registring read intent");
    //     self.babel_conn.readable().await?;
    //     tracing::debug!("Readble!");
    //     dbg!(self.babel_conn.read_to_string(&mut buf).await)?;
    //     let res = serde_json::from_str(&buf)?;
    //     Ok(res)
    // }

    // async fn write_data(&mut self, data: impl serde::Serialize) -> Result<()> {
    //     tracing::debug!("Registring write intent");
    //     self.babel_conn.writable().await?;
    //     tracing::debug!("Writable!");
    //     let data = serde_json::to_string(&data)?;
    //     dbg!(self.babel_conn.write_all(data.as_bytes()).await)?;
    //     Ok(())
    // }

    async fn rw<S: serde::ser::Serialize, D: serde::de::DeserializeOwned>(&mut self, data: S) -> Result<D> {
        let mut should_write = true;
        loop {
            let ready = self.babel_conn.ready(tokio::io::Interest::READABLE | tokio::io::Interest::WRITABLE).await.unwrap();
    
            if ready.is_readable() {
                let mut data = vec![0; 1024];
                // Try to read data, this may still fail with `WouldBlock`
                // if the readiness event is a false positive.
                match self.babel_conn.try_read(&mut data) {
                    Ok(n) => {
                        println!("read {} bytes", n);
                        data.resize(n, 0);
                        println!("Received message: `{data:?}`");
                        let s = std::str::from_utf8(&data).unwrap();
                        println!("As string: `{s}`");
                        if s.starts_with("OK ") {
                            // This is the init message, skip
                            continue;
                        }
                        return Ok(serde_json::from_str(s).unwrap());
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        println!("Readable sad times {e}");
                        panic!();
                    }
                }
    
            }
    
            if should_write && ready.is_writable() {
                should_write = false;
                // Try to write data, this may still fail with `WouldBlock`
                // if the readiness event is a false positive.
                let data = serde_json::to_string(&data)?;
                match self.babel_conn.try_write(data.as_bytes()) {
                    Ok(n) => {
                        println!("wrote {n} bytes");
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        println!("Writable sad times {e}");
                        panic!();
                    }
                }
            }
        }
    }

    /// Establishes a new connection to the VM. Note that this fails if the VM hasn't started yet.
    /// It also initializes that connection by sending the opening message. Therefore, if this
    /// function succeeds the connection is guaranteed to be writeable at the moment of returning.
    pub async fn conn(node_id: uuid::Uuid) -> Result<UnixStream> {
        // We are going to connect to the central socket for this VM. Later we will specify which
        // port we want to talk to.
        let socket = dbg!(format!("{CHROOT_PATH}/firecracker/{node_id}/root/vsock.socket"));
        tracing::debug!("Connecting to node at `{socket}`");
        let mut conn = UnixStream::connect(&socket)
            .await
            .context("Failed to connect to babel bus")?;
        // For host initiated connections we have to send a message to the guess socket that
        // contains the port number.
        let open_message = format!("CONNECT {BABEL_VSOCK_PORT}\n");
        conn.write(open_message.as_bytes()).await?;
        Ok(conn)
    }
}
