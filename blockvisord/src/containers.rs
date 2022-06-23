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
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use sysinfo::{PidExt, ProcessExt, ProcessRefreshKind, RefreshKind, System, SystemExt};
use tokio::fs::{self, read_dir};
use tokio::time::sleep;
use tracing::{debug, info, instrument, trace, warn};
use uuid::Uuid;
use zbus::export::futures_util::TryFutureExt;
use zbus::{dbus_interface, fdo, zvariant::Type};

const NODES_CONFIG_FILENAME: &str = "nodes.toml";

lazy_static::lazy_static! {
    static ref REGISTRY_CONFIG_DIR: PathBuf = home::home_dir()
        .map(|p| p.join(".cache"))
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("blockvisor");
}
lazy_static::lazy_static! {
    static ref REGISTRY_CONFIG_FILE: PathBuf = REGISTRY_CONFIG_DIR.join(NODES_CONFIG_FILENAME);
}

#[derive(Clone, Debug)]
pub enum ServiceStatus {
    Enabled,
    Disabled,
}

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
    data: NodeData,
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

#[derive(Debug, Default)]
pub struct Nodes {
    pub nodes: HashMap<Uuid, Node>,
    data: CommonData,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct CommonData {
    machine_index: Arc<Mutex<u32>>,
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
    async fn load(path: &Path) -> Result<Self> {
        info!("Reading nodes config file: {}", path.display());
        fs::read_to_string(&path)
            .await
            .and_then(|s| toml::from_str::<Self>(&s).map_err(Into::into))
            .with_context(|| format!("Failed to read node file `{}`", path.display()))
    }

    async fn save(&self) -> Result<()> {
        let path = self.file_path();
        info!("Writing node config: {}", path.display());
        let config = toml::to_string(self)?;
        fs::write(&path, &*config).await?;

        Ok(())
    }

    async fn delete(self) -> Result<()> {
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

#[dbus_interface(interface = "com.BlockJoy.blockvisor.Node")]
impl Nodes {
    #[instrument(skip(self))]
    async fn create(&mut self, id: Uuid, chain: String) -> fdo::Result<()> {
        let network_interface = self.next_network_interface();
        let node = NodeData {
            id,
            chain,
            state: NodeState::Stopped,
            network_interface,
        };

        let node = Node::create(node)
            .await
            .map_err(|e| fdo::Error::IOError(e.to_string()))?;
        self.nodes.insert(id, node);
        debug!("Container with id `{}` created", id);

        fdo::Result::Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&mut self, id: Uuid) -> fdo::Result<()> {
        let node = self.nodes.remove(&id).ok_or_else(|| {
            let msg = format!("Container with id {} not found", id);
            fdo::Error::FileNotFound(msg)
        })?;
        node.delete()
            .await
            .map_err(|e| fdo::Error::IOError(e.to_string()))?;
        debug!("deleted");

        fdo::Result::Ok(())
    }

    #[instrument(skip(self))]
    async fn start(&mut self, id: Uuid) -> fdo::Result<()> {
        let node = self.nodes.get_mut(&id).ok_or_else(|| {
            let msg = format!("Container with id {} not found", id);
            fdo::Error::FileNotFound(msg)
        })?;
        debug!("found node");
        node.start()
            .await
            .map_err(|e| fdo::Error::IOError(e.to_string()))?;
        debug!("started");

        fdo::Result::Ok(())
    }

    #[instrument(skip(self))]
    async fn stop(&mut self, id: Uuid) -> fdo::Result<()> {
        let node = self.nodes.get_mut(&id).ok_or_else(|| {
            let msg = format!("Container with id {} not found", id);
            fdo::Error::FileNotFound(msg)
        })?;
        debug!("found node");
        node.kill()
            .await
            .map_err(|e| fdo::Error::IOError(e.to_string()))?;
        debug!("stopped");

        fdo::Result::Ok(())
    }

    #[instrument(skip(self))]
    async fn list(&self) -> Vec<NodeData> {
        debug!("listing {} nodes", self.nodes.len());

        self.nodes.values().map(|n| n.data.clone()).collect()
    }

    // TODO: Rest of the NodeCommand variants.
}

impl Nodes {
    pub async fn load() -> Result<Nodes> {
        // First load the common data file.
        info!(
            "Reading nodes common config file: {}",
            REGISTRY_CONFIG_FILE.display()
        );
        let config = fs::read_to_string(&*REGISTRY_CONFIG_FILE).await?;
        let nodes_data = toml::from_str(&config)?;

        // Now the individual node data files.
        info!(
            "Reading nodes config dir: {}",
            REGISTRY_CONFIG_DIR.display()
        );
        let mut this = Nodes {
            nodes: HashMap::new(),
            data: nodes_data,
        };
        let mut dir = read_dir(&*REGISTRY_CONFIG_DIR).await?;
        while let Some(entry) = dir.next_entry().await? {
            // blockvisord should not bail on problems with individual node files.
            // It should log warnings though.
            let path = entry.path();
            if path == *REGISTRY_CONFIG_FILE {
                // Skip the common data file.
                continue;
            }
            match NodeData::load(&*path).and_then(Node::connect).await {
                Ok(node) => {
                    this.nodes.insert(node.data.id, node);
                }
                Err(e) => warn!("Failed to read node file `{}`: {}", path.display(), e),
            }
        }

        Ok(this)
    }

    pub async fn save(&self) -> Result<()> {
        // We only save the common data file. The individual node data files save themselves.
        info!(
            "Writing nodes common config file: {}",
            REGISTRY_CONFIG_FILE.display()
        );
        let config = toml::Value::try_from(&self.data)?;
        let config = toml::to_string(&config)?;
        fs::create_dir_all(REGISTRY_CONFIG_DIR.as_path()).await?;
        fs::write(&*REGISTRY_CONFIG_FILE, &*config).await?;

        Ok(())
    }

    pub fn exists() -> bool {
        Path::new(&*REGISTRY_CONFIG_FILE).exists()
    }

    /// Get the next machine index and increment it.
    pub fn next_network_interface(&self) -> NetworkInterface {
        let mut machine_index = self.data.machine_index.lock().expect("lock poisoned");

        let idx_bytes = machine_index.to_be_bytes();
        let iface = NetworkInterface {
            name: format!("bv{}", *machine_index),
            // FIXME: Hardcoding address for now.
            ip: IpAddr::V4(Ipv4Addr::new(
                idx_bytes[0] + 74,
                idx_bytes[1] + 50,
                idx_bytes[2] + 82,
                idx_bytes[3] + 83,
            )),
        };
        *machine_index += 1;

        iface
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Type)]
pub struct NetworkInterface {
    pub name: String,
    pub ip: IpAddr,
}

impl fmt::Display for NetworkInterface {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.ip)
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

#[cfg(test)]
mod tests {
    #[test]
    fn network_interface_gen() {
        let nodes = super::Nodes::default();
        let iface = nodes.next_network_interface();
        assert_eq!(iface.name, "bv0");
        assert_eq!(
            iface.ip,
            super::IpAddr::V4(super::Ipv4Addr::new(74, 50, 82, 83))
        );

        let iface = nodes.next_network_interface();
        assert_eq!(iface.name, "bv1");
        assert_eq!(
            iface.ip,
            super::IpAddr::V4(super::Ipv4Addr::new(74, 50, 82, 84))
        );

        // Let's take the machine_index beyond u8 boundry.
        *nodes.data.machine_index.lock().expect("lock poisoned") = u8::MAX as u32 + 1;
        let iface = nodes.next_network_interface();
        assert_eq!(iface.name, format!("bv{}", u8::MAX as u32 + 1));
        assert_eq!(
            iface.ip,
            super::IpAddr::V4(super::Ipv4Addr::new(74, 50, 83, 83))
        );
    }
}
