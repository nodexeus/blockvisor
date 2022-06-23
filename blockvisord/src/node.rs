use anyhow::{bail, Context, Result};
use cli_table::{
    format::Justify,
    CellStruct,
    Color::{Blue, Cyan, Green, Red, Yellow},
    Style, Table,
};
use firec::config::JailerMode;
use firec::Machine;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use sysinfo::{PidExt, ProcessExt, ProcessRefreshKind, RefreshKind, System, SystemExt};
use tokio::fs;
use tokio::time::sleep;
use tracing::{info, instrument, trace};
use uuid::Uuid;
use zbus::zvariant::Type;

use crate::nodes::{NetworkInterface, REGISTRY_CONFIG_DIR};

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Copy, Debug, Type)]
pub enum NodeState {
    Running,
    Stopped,
}

fn style_node_state(cell: CellStruct, value: &NodeState) -> CellStruct {
    match value {
        NodeState::Running => cell.foreground_color(Some(Green)),
        NodeState::Stopped => cell.foreground_color(Some(Red)),
    }
}

impl fmt::Display for NodeState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct Node {
    pub data: NodeData,
    machine: Machine<'static>,
}

// FIXME: Hardcoding everything for now.
const KERNEL_PATH: &str = "/var/demo/debian-vmlinux";
const ROOT_FS: &str = "/var/demo/debian.ext4";
const CHROOT_PATH: &str = "/var/demo/helium";
const FC_BIN_PATH: &str = "/usr/bin/firecracker";
const FC_BIN_NAME: &str = "firecracker";
const FC_SOCKET_PATH: &str = "/firecracker.socket";

impl Node {
    /// Creates a new node with `id`.
    /// TODO: machine_index is a hack. Remove after demo.
    #[instrument]
    pub async fn create(data: NodeData) -> Result<Self> {
        let config = Node::create_config(data.id, &data.network_interface)?;
        let machine = firec::Machine::create(config).await?;
        data.save().await?;

        Ok(Self { data, machine })
    }

    /// Returns node previously created on this host.
    #[instrument]
    pub async fn connect(data: NodeData) -> Result<Self> {
        let config = Node::create_config(data.id, &data.network_interface)?;
        let cmd = data.id.to_string();
        let state = match get_process_pid(FC_BIN_NAME, &cmd) {
            Ok(pid) => firec::MachineState::RUNNING { pid },
            Err(_) => firec::MachineState::SHUTOFF,
        };
        let machine = firec::Machine::connect(config, state).await;

        Ok(Self { data, machine })
    }

    /// Returns the node's `id`.
    pub fn id(&self) -> &Uuid {
        &self.data.id
    }

    /// Starts the node.
    #[instrument(skip(self))]
    pub async fn start(&mut self) -> Result<()> {
        self.machine.start().await?;
        self.data.state = NodeState::Running;
        self.data.save().await
    }

    /// Returns the state of the node.
    pub async fn state(&self) -> Result<NodeState> {
        unimplemented!()
    }

    /// Kills the running node.
    #[instrument(skip(self))]
    pub async fn kill(&mut self) -> Result<()> {
        match self.machine.state() {
            firec::MachineState::SHUTOFF => {}
            firec::MachineState::RUNNING { .. } => {
                if let Err(err) = self.machine.shutdown().await {
                    trace!("Shutdown error: {err}");
                } else {
                    sleep(Duration::from_secs(10)).await;
                }

                if let Err(err) = self.machine.force_shutdown().await {
                    trace!("Forced shutdown error: {err}");
                }
            }
        }
        self.data.state = NodeState::Stopped;
        self.data.save().await?;

        Ok(())
    }

    /// Deletes the node.
    #[instrument(skip(self))]
    pub async fn delete(self) -> Result<()> {
        self.machine.delete().await?;
        self.data.delete().await
    }

    fn create_config(
        id: Uuid,
        network_interface: &NetworkInterface,
    ) -> Result<firec::config::Config<'static>> {
        let kernel_args = format!(
            "console=ttyS0 reboot=k panic=1 pci=off random.trust_cpu=on \
            ip={}::74.50.82.81:255.255.255.240::eth0:on",
            network_interface.ip,
        );
        let iface = firec::config::network::Interface::new(network_interface.name.clone(), "eth0");

        let config = firec::config::Config::builder(Some(id), Path::new(KERNEL_PATH))
            // Jailer configuration.
            .jailer_cfg()
            .chroot_base_dir(Path::new(CHROOT_PATH))
            .exec_file(Path::new(FC_BIN_PATH))
            .mode(JailerMode::Daemon)
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
            .build();

        Ok(config)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Type, Table)]
pub struct NodeData {
    #[table(title = "VM ID", justify = "Justify::Right", color = "Cyan")]
    pub id: Uuid,
    #[table(title = "Chain", color = "Blue")]
    pub chain: String,
    #[table(title = "State", customize_fn = "style_node_state")]
    pub state: NodeState,
    #[table(title = "IP Address", color = "Yellow")]
    pub network_interface: NetworkInterface,
}

impl NodeData {
    pub async fn load(path: &Path) -> Result<Self> {
        info!("Reading nodes config file: {}", path.display());
        fs::read_to_string(&path)
            .await
            .and_then(|s| toml::from_str::<Self>(&s).map_err(Into::into))
            .with_context(|| format!("Failed to read node file `{}`", path.display()))
    }

    pub async fn save(&self) -> Result<()> {
        let path = self.file_path();
        info!("Writing node config: {}", path.display());
        let config = toml::to_string(self)?;
        fs::write(&path, &*config).await?;

        Ok(())
    }

    pub async fn delete(self) -> Result<()> {
        let path = self.file_path();
        info!("Deleting node config: {}", path.display());
        fs::remove_file(&*path)
            .await
            .with_context(|| format!("Failed to delete node file `{}`", path.display()))
    }

    fn file_path(&self) -> PathBuf {
        let filename = format!("{}.toml", self.id);
        REGISTRY_CONFIG_DIR.join(filename)
    }
}

/// Get the pid of the running VM process knowing its process name and part of command line.
fn get_process_pid(process_name: &str, cmd: &str) -> Result<i32> {
    let mut sys = System::new();
    // TODO: would be great to save the System and not do a full refresh each time
    sys.refresh_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::everything()));
    let processes: Vec<_> = sys
        .processes_by_name(process_name)
        .filter(|&process| process.cmd().contains(&cmd.to_string()))
        .collect();

    match processes.len() {
        0 => bail!("No {process_name} processes running for id: {cmd}"),
        1 => processes[0].pid().as_u32().try_into().map_err(Into::into),
        _ => bail!("More then 1 {process_name} process running for id: {cmd}"),
    }
}
