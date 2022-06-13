use anyhow::{bail, Ok, Result};
use firec::config::JailerMode;
use firec::Machine;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use sysinfo::{PidExt, ProcessExt, ProcessRefreshKind, RefreshKind, System, SystemExt};
use tokio::fs;
use tokio::time::sleep;
use tracing::{debug, info, instrument, trace};
use uuid::Uuid;
use zbus::{dbus_interface, fdo, zvariant::Type};

const CONTAINERS_CONFIG_FILENAME: &str = "containers.toml";

lazy_static::lazy_static! {
    static ref REGISTRY_CONFIG_FILE: PathBuf = home::home_dir()
        .map(|p| p.join(".cache"))
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("blockvisor")
        .join(CONTAINERS_CONFIG_FILENAME);
}

#[derive(Clone, Debug)]
pub enum ServiceStatus {
    Enabled,
    Disabled,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Copy, Debug, Type)]
pub enum ContainerState {
    Created,
    Started,
    Stopped,
    Deleted,
}

pub struct LinuxNode {
    id: Uuid,
    machine: Machine<'static>,
}

// FIXME: Hardcoding everything for now.
const KERNEL_PATH: &str = "/var/demo/debian-vmlinux";
const ROOT_FS: &str = "/var/demo/debian.ext4";
const CHROOT_PATH: &str = "/var/demo/helium";
const FC_BIN_PATH: &str = "/usr/bin/firecracker";
const FC_BIN_NAME: &str = "firecracker";
const FC_SOCKET_PATH: &str = "/firecracker.socket";

impl LinuxNode {
    /// Creates a new container with `id`.
    /// TODO: machine_index is a hack. Remove after demo.
    #[instrument]
    pub async fn create(id: Uuid, network_interface: &NetworkInterface) -> Result<Self> {
        let config = LinuxNode::create_config(id, network_interface)?;
        let machine = firec::Machine::create(config).await?;

        Ok(Self { id, machine })
    }

    /// Checks if container exists on this host.
    pub async fn exists(id: Uuid) -> bool {
        let cmd = id.to_string();
        get_process_pid(FC_BIN_NAME, &cmd).is_ok()
    }

    /// Returns container previously created on this host.
    #[instrument]
    pub async fn connect(id: Uuid, network_interface: &NetworkInterface) -> Result<Self> {
        let config = LinuxNode::create_config(id, network_interface)?;
        let cmd = id.to_string();
        let pid = get_process_pid(FC_BIN_NAME, &cmd)?;
        let machine = firec::Machine::connect(config, pid).await;

        Ok(Self { id, machine })
    }

    /// Returns the container's `id`.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Starts the container.
    #[instrument(skip(self))]
    pub async fn start(&mut self) -> Result<()> {
        self.machine.start().await.map_err(Into::into)
    }

    /// Returns the state of the container.
    pub async fn state(&self) -> Result<ContainerState> {
        unimplemented!()
    }

    /// Kills the running container.
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

        Ok(())
    }

    /// Deletes the container.
    #[instrument(skip(self))]
    pub async fn delete(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn create_config(
        id: Uuid,
        network_interface: &NetworkInterface,
    ) -> Result<firec::config::Config<'static>> {
        let jailer = firec::config::Jailer::builder()
            .chroot_base_dir(Path::new(CHROOT_PATH))
            .exec_file(Path::new(FC_BIN_PATH))
            .mode(JailerMode::Daemon)
            .build();

        let root_drive = firec::config::Drive::builder("root", Path::new(ROOT_FS))
            .is_root_device(true)
            .build();
        let kernel_args = Some(format!(
            "console=ttyS0 reboot=k panic=1 pci=off random.trust_cpu=on \
            ip={}::74.50.82.81:255.255.255.240::eth0:on",
            network_interface.ip,
        ));

        let iface = firec::config::network::Interface::new("eth0", network_interface.name.clone());

        let machine_cfg = firec::config::Machine::builder()
            .vcpu_count(1)
            .mem_size_mib(8192)
            .build();

        let config = firec::config::Config::builder(Path::new(KERNEL_PATH))
            .vm_id(id)
            .jailer_cfg(Some(jailer))
            .kernel_args(kernel_args)
            .machine_cfg(machine_cfg)
            .add_drive(root_drive)
            .add_network_interface(iface)
            .socket_path(Path::new(FC_SOCKET_PATH))
            .build()?;

        Ok(config)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Containers {
    pub containers: HashMap<Uuid, ContainerData>,
    machine_index: Arc<Mutex<u32>>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Type)]
pub struct ContainerData {
    pub id: Uuid,
    pub chain: String,
    pub state: ContainerState,
}

#[dbus_interface(interface = "com.BlockJoy.blockvisor.Node")]
impl Containers {
    #[instrument(skip(self))]
    async fn create(&mut self, chain: String) -> fdo::Result<Uuid> {
        let id = Uuid::new_v4();
        let container = ContainerData {
            id,
            chain,
            state: ContainerState::Created,
        };

        let network_interface = self.next_network_interface();
        LinuxNode::create(id, &network_interface)
            .await
            .map_err(|e| fdo::Error::IOError(e.to_string()))?;

        self.containers.insert(id, container);
        self.save()
            .await
            .map_err(|e| fdo::Error::IOError(e.to_string()))?;
        debug!("Container with id `{}` created", id);

        fdo::Result::Ok(id)
    }

    #[instrument(skip(self))]
    async fn delete(&mut self, id: Uuid) -> fdo::Result<()> {
        self.containers.remove(&id).ok_or_else(|| {
            let msg = format!("Container with id {} not found", id);
            fdo::Error::FileNotFound(msg)
        })?;
        let mut node = self
            .get_node(&id)
            .await
            .map_err(|e| fdo::Error::IOError(e.to_string()))?;
        node.delete()
            .await
            .map_err(|e| fdo::Error::IOError(e.to_string()))?;
        self.save()
            .await
            .map_err(|e| fdo::Error::IOError(e.to_string()))?;
        debug!("deleted");

        fdo::Result::Ok(())
    }

    #[instrument(skip(self))]
    async fn start(&self, id: Uuid) -> fdo::Result<()> {
        self.containers.get(&id).ok_or_else(|| {
            let msg = format!("Container with id {} not found", id);
            fdo::Error::FileNotFound(msg)
        })?;
        debug!("found container");
        let mut node = self
            .get_node(&id)
            .await
            .map_err(|e| fdo::Error::IOError(e.to_string()))?;
        node.start()
            .await
            .map_err(|e| fdo::Error::IOError(e.to_string()))?;
        debug!("started");

        fdo::Result::Ok(())
    }

    #[instrument(skip(self))]
    async fn stop(&self, id: Uuid) -> fdo::Result<()> {
        self.containers.get(&id).ok_or_else(|| {
            let msg = format!("Container with id {} not found", id);
            fdo::Error::FileNotFound(msg)
        })?;
        debug!("found container");
        let mut node = self
            .get_node(&id)
            .await
            .map_err(|e| fdo::Error::IOError(e.to_string()))?;
        node.kill()
            .await
            .map_err(|e| fdo::Error::IOError(e.to_string()))?;
        debug!("stopped");

        fdo::Result::Ok(())
    }

    #[instrument(skip(self))]
    async fn list(&self) -> fdo::Result<HashMap<Uuid, ContainerData>> {
        debug!("listing {} containers", self.containers.len());
        fdo::Result::Ok(self.containers.clone())
    }

    // TODO: Rest of the NodeCommand variants.
}

impl Containers {
    pub async fn load() -> Result<Containers> {
        info!(
            "Reading containers config: {}",
            REGISTRY_CONFIG_FILE.display()
        );
        let config = fs::read_to_string(&*REGISTRY_CONFIG_FILE).await?;
        Ok(toml::from_str(&config)?)
    }

    pub async fn save(&self) -> Result<()> {
        info!(
            "Writing containers config: {}",
            REGISTRY_CONFIG_FILE.display()
        );
        let config = toml::Value::try_from(self)?;
        let config = toml::to_string(&config)?;
        let parent = REGISTRY_CONFIG_FILE
            .parent()
            .expect("config file has no parent");
        fs::create_dir_all(parent).await?;
        fs::write(&*REGISTRY_CONFIG_FILE, &*config).await?;
        Ok(())
    }

    pub fn exists() -> bool {
        Path::new(&*REGISTRY_CONFIG_FILE).exists()
    }

    /// Get the next machine index and increment it.
    pub fn next_network_interface(&self) -> NetworkInterface {
        let mut machine_index = self.machine_index.lock().expect("lock poisoned");

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

    async fn get_node(&self, id: &Uuid) -> Result<LinuxNode> {
        // FIXME: This is wrong and bad to create the interface just to delete the VMM but until we keep the Nodes in the
        // memory and don't save the machine config, we need to do this.
        let network_interface = self.next_network_interface();
        LinuxNode::connect(*id, &network_interface).await
    }
}

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub ip: IpAddr,
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
        let containers = super::Containers::default();
        let iface = containers.next_network_interface();
        assert_eq!(iface.name, "bv0");
        assert_eq!(
            iface.ip,
            super::IpAddr::V4(super::Ipv4Addr::new(74, 50, 82, 83))
        );

        let iface = containers.next_network_interface();
        assert_eq!(iface.name, "bv1");
        assert_eq!(
            iface.ip,
            super::IpAddr::V4(super::Ipv4Addr::new(74, 50, 82, 84))
        );

        // Let's take the machine_index beyond u8 boundry.
        *containers.machine_index.lock().expect("lock poisoned") = u8::MAX as u32 + 1;
        let iface = containers.next_network_interface();
        assert_eq!(iface.name, format!("bv{}", u8::MAX as u32 + 1));
        assert_eq!(
            iface.ip,
            super::IpAddr::V4(super::Ipv4Addr::new(74, 50, 83, 83))
        );
    }
}
