use anyhow::{anyhow, bail, Context, Result};
use firec::config::JailerMode;
use firec::Machine;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};
use tokio::{fs::DirBuilder, net::UnixStream, time::sleep};
use tracing::{debug, instrument, trace, warn};
use uuid::Uuid;

use crate::{
    babel_connection::BabelConnection,
    env::*,
    node_data::{NodeData, NodeImage, NodeStatus},
    utils::{get_process_pid, run_cmd},
};

#[derive(Debug)]
pub struct Node {
    pub data: NodeData,
    machine: Machine<'static>,
    babel_conn: BabelConnection,
}

// FIXME: Hardcoding everything for now.
pub const FC_BIN_NAME: &str = "firecracker";
const FC_BIN_PATH: &str = "/usr/bin/firecracker";
const FC_SOCKET_PATH: &str = "/firecracker.socket";
pub const ROOT_FS_FILE: &str = "os.img";
pub const VSOCK_PATH: &str = "/vsock.socket";
const VSOCK_GUEST_CID: u32 = 3;

impl Node {
    /// Creates a new node according to specs.
    #[instrument]
    pub async fn create(data: NodeData) -> Result<Self> {
        let config = Node::create_config(&data)?;
        Node::copy_data_image(&data.id).await?;
        let machine = firec::Machine::create(config).await?;

        data.save().await?;

        Ok(Self {
            data,
            machine,
            babel_conn: BabelConnection::Closed,
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
                    BabelConnection::Open { babel_conn: c },
                )
            }
            Err(_) => (firec::MachineState::SHUTOFF, BabelConnection::Closed),
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

    /// Updates OS image for VM.
    #[instrument(skip(self))]
    pub async fn upgrade(&mut self, image: &NodeImage) -> Result<()> {
        if self.status() != NodeStatus::Stopped {
            bail!("Node should be stopped before running upgrade");
        }
        if image.protocol != self.data.image.protocol {
            bail!("Cannot upgrade protocol to `{}`", image.protocol);
        }
        if image.node_type != self.data.image.node_type {
            bail!("Cannot upgrade node type to `{}`", image.node_type);
        }

        Node::copy_os_image(&self.data.id, image).await?;
        self.data.image = image.clone();
        self.data.save().await
    }

    /// Starts the node.
    #[instrument(skip(self))]
    pub async fn start(&mut self) -> Result<()> {
        if self.status() == NodeStatus::Running
            || (self.status() == NodeStatus::Failed
                && self.expected_status() == NodeStatus::Stopped)
        {
            return Ok(());
        }

        self.machine.start().await?;
        let node_id = self.id();
        let babel_conn = match BabelConnection::connect(&node_id).await {
            Ok(conn) => Ok(conn),
            Err(_) => {
                // Extremely scientific retrying mechanism
                sleep(Duration::from_secs(10)).await;
                BabelConnection::connect(&node_id).await
            }
        }?;
        self.babel_conn = BabelConnection::Open { babel_conn };
        let resp = self.send(babel_api::BabelRequest::Ping).await;
        if !matches!(resp, Ok(babel_api::BabelResponse::Pong)) {
            warn!("Ping request did not respond with `Pong`, but `{resp:?}`");
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
                if let Err(err) = self.machine.shutdown().await {
                    trace!("Graceful shutdown failed: {err}");

                    if let Err(err) = self.machine.force_shutdown().await {
                        bail!("Forced shutdown failed: {err}");
                    }
                }
                self.babel_conn.wait_for_disconnect(&self.id()).await;
            }
        }
        self.data.expected_status = NodeStatus::Stopped;
        self.data.save().await?;
        self.babel_conn = BabelConnection::Closed;

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

    pub async fn set_self_update(&mut self, self_update: bool) -> Result<()> {
        self.data.self_update = self_update;
        self.data.save().await
    }

    /// Copy OS drive into chroot location.
    async fn copy_os_image(id: &Uuid, image: &NodeImage) -> Result<()> {
        let root_fs_path = Self::get_normalized_root_fs_path(image)?;

        let data_dir = CHROOT_PATH
            .join(FC_BIN_NAME)
            .join(id.to_string())
            .join("root");
        DirBuilder::new().recursive(true).create(&data_dir).await?;

        run_cmd(
            "cp",
            &[&root_fs_path.to_string_lossy(), &data_dir.to_string_lossy()],
        )
        .await?;

        Ok(())
    }

    /// Check root if fs source path is correct and exists
    ///
    /// Also resolve symlinks into canonical path
    fn get_normalized_root_fs_path(image: &NodeImage) -> Result<PathBuf> {
        let root_fs_path = IMAGE_CACHE_DIR.join(image.url()).canonicalize()?;
        if root_fs_path.file_name() != Some(OsStr::new(ROOT_FS_FILE)) {
            bail!(
                "Bad root image file name: `{:?}` in `{}`",
                root_fs_path.file_name(),
                root_fs_path.display()
            )
        }
        if !root_fs_path.try_exists()? {
            // TODO: download from remote images repository into cache dir
            // return error if not present in remote
            bail!("Root image file not found: `{}`", root_fs_path.display())
        }
        Ok(root_fs_path)
    }

    /// Copy data drive into chroot location.
    ///
    /// NOTE: this is a workaround using system `cp` instead of `std::fs::copy`
    /// to be able to preserve sparsity of the file. Firec will not overwrite
    /// the file in chroot if it's already present.
    ///
    /// See discussion here for more details: https://github.com/rust-lang/rust/issues/58635
    async fn copy_data_image(id: &Uuid) -> Result<()> {
        // TODO: we need to create a new data image according to spec
        // At the time of writing we use the same 10 Gb empty image for every node
        let data_dir = CHROOT_PATH
            .join(FC_BIN_NAME)
            .join(id.to_string())
            .join("root");
        DirBuilder::new().recursive(true).create(&data_dir).await?;
        run_cmd(
            "cp",
            &[&DATA_PATH.to_string_lossy(), &data_dir.to_string_lossy()],
        )
        .await?;

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
        let root_fs_path = Self::get_normalized_root_fs_path(&data.image)?;

        let config = firec::config::Config::builder(Some(data.id), &*KERNEL_PATH)
            // Jailer configuration.
            .jailer_cfg()
            .chroot_base_dir(&*CHROOT_PATH)
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
            .add_drive("data", &*DATA_PATH)
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

    /// TODO: Wait for Sean to tell us how to do this.
    pub async fn stake_status(&mut self) -> Result<i32> {
        Ok(0)
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
        let request = babel_api::BabelRequest::BlockchainCommand(babel_api::BlockchainCommand {
            name: method.to_string(),
        });
        debug!("Calling method: {method}");
        let resp: babel_api::BabelResponse = self.send(request).await?;
        let inner = match resp {
            babel_api::BabelResponse::BlockchainResponse(babel_api::BlockchainResponse {
                value,
            }) => value.parse().context(format!("Could not parse {method}"))?,
            e => bail!("Unexpected BabelResponse for `{method}`: `{e:?}`"),
        };
        Ok(inner)
    }

    /// Returns the methods that are supported by this blockchain. Calling any method on this
    /// blockchain that is not listed here will result in an error being returned.
    pub async fn capabilities(&mut self) -> Result<Vec<String>> {
        let request = babel_api::BabelRequest::ListCapabilities;
        let resp: babel_api::BabelResponse = self.send(request).await?;
        let capabilities = match resp {
            babel_api::BabelResponse::ListCapabilities(caps) => caps,
            e => bail!("Unexpected BabelResponse for `capabilities`: `{e:?}`"),
        };
        Ok(capabilities)
    }

    /// Checks if node has some particular capability
    pub async fn has_capability(&mut self, method: &str) -> Result<bool> {
        let caps = self.capabilities().await?;
        Ok(caps.contains(&method.to_owned()))
    }

    /// Returns the list of logs from blockchain entry_points.
    pub async fn logs(&mut self) -> Result<Vec<String>> {
        let request = babel_api::BabelRequest::Logs;
        let resp: babel_api::BabelResponse = self.send(request).await?;
        let logs = match resp {
            babel_api::BabelResponse::Logs(logs) => logs,
            e => bail!("Unexpected BabelResponse for `logs`: `{e:?}`"),
        };
        Ok(logs)
    }

    /// Returns blockchain node keys.
    pub async fn download_keys(&mut self) -> Result<Vec<babel_api::BlockchainKey>> {
        let request = babel_api::BabelRequest::DownloadKeys;
        let resp: babel_api::BabelResponse = self.send(request).await?;
        let keys = match resp {
            babel_api::BabelResponse::Keys(keys) => keys,
            e => bail!("Unexpected BabelResponse for `download_keys`: `{e:?}`"),
        };
        Ok(keys)
    }

    /// Sets blockchain node keys.
    pub async fn upload_keys(&mut self, keys: Vec<babel_api::BlockchainKey>) -> Result<()> {
        let request = babel_api::BabelRequest::UploadKeys(keys);
        let resp: babel_api::BabelResponse = self.send(request).await?;
        match resp {
            babel_api::BabelResponse::BlockchainResponse(babel_api::BlockchainResponse {
                value,
            }) => debug!("Upload keys: {value}"),
            e => bail!("Unexpected BabelResponse for `upload_keys`: `{e:?}`"),
        };
        Ok(())
    }

    /// Generates keys on node
    pub async fn generate_keys(&mut self) -> Result<String> {
        self.call_method(&babel_api::BabelMethod::GenerateKeys.to_string())
            .await
    }

    /// This function combines the capabilities from `write_data` and `read_data` to allow you to
    /// send some request and then obtain a response back.
    async fn send<S: serde::ser::Serialize, D: serde::de::DeserializeOwned>(
        &mut self,
        data: S,
    ) -> Result<D> {
        self.babel_conn.write_data(data).await?;
        self.babel_conn.read_data().await
    }
}
