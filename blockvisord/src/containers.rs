use anyhow::{Ok, Result};
use async_trait::async_trait;
use firec::Machine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;
use uuid::Uuid;

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

#[derive(Deserialize, Serialize, PartialEq, Clone, Copy, Debug)]
pub enum ContainerStatus {
    Created,
    Started,
    Stopped,
    Deleted,
}

#[async_trait]
pub trait NodeContainer {
    /// Creates a new container with `id`.
    /// TODO: machine_index is a hack. Remove after demo.
    async fn create(id: &str, machine_index: usize) -> Result<Self>
    where
        Self: Sized;

    /// Returns the container's `id`.
    fn id(&self) -> &str;

    /// Starts the container.
    async fn start(&mut self) -> Result<()>;

    /// Returns the state of the container.
    async fn state(&self) -> Result<ContainerStatus>;

    /// Kills the running container.
    async fn kill(&mut self) -> Result<()>;

    /// Deletes the container.
    async fn delete(&mut self) -> Result<()>;
}

pub trait NodeRegistry {
    fn contains(id: &str) -> bool;
    fn get(id: &str) -> Result<Box<dyn NodeContainer>>;
}

pub struct LinuxNode {
    id: String,
    machine: Machine<'static>,
}

// FIXME: Hardcoding everything for now.
const KERNEL_PATH: &str = "/var/demo/debian-vmlinux";
const ROOT_FS: &str = "/var/demo/debian.ext4";
const CHROOT_PATH: &str = "/var/demo/helium";
const FC_BIN_PATH: &str = "/usr/bin/firecracker";
const FC_SOCKET_PATH: &str = "/firecracker.socket";

#[async_trait]
impl NodeContainer for LinuxNode {
    async fn create(id: &str, machine_index: usize) -> Result<Self> {
        let jailer = firec::config::Jailer {
            chroot_base_dir: Path::new(CHROOT_PATH).into(),
            exec_file: Path::new(FC_BIN_PATH).into(),
            ..Default::default()
        };

        let root_drive = firec::config::Drive {
            drive_id: "root".into(),
            path_on_host: Path::new(ROOT_FS).into(),
            is_root_device: true,
            ..Default::default()
        };

        let kernel_args = Some(
            format!(
                "console=ttyS0 reboot=k panic=1 pci=off random.trust_cpu=on \
            ip=74.50.82.8{}::74.50.82.81:255.255.255.240::eth0:on",
                machine_index + 3,
            )
            .into(),
        );

        let cni = firec::config::network::Cni {
            network_name: "eth0".into(),
            if_name: Some(format!("bv{}", machine_index).into()),
            ..Default::default()
        };
        let iface = firec::config::network::Interface::Cni(cni);

        let machine_cfg = firec::config::Machine {
            vcpu_count: 1,
            mem_size_mib: 8192,
            ..Default::default()
        };

        let config = firec::config::Config {
            vm_id: Some(Uuid::parse_str(id)?),
            jailer_cfg: Some(jailer),
            kernel_image_path: Path::new(KERNEL_PATH).into(),
            kernel_args,
            machine_cfg,
            drives: vec![root_drive],
            network_interfaces: vec![iface],
            socket_path: Path::new(FC_SOCKET_PATH).into(),
            ..Default::default()
        };
        let machine = firec::Machine::new(config).await?;

        Ok(Self {
            id: id.to_string(),
            machine,
        })
    }

    fn id(&self) -> &str {
        &self.id
    }

    async fn start(&mut self) -> Result<()> {
        self.machine.start().await.map_err(Into::into)
    }

    async fn state(&self) -> Result<ContainerStatus> {
        unimplemented!()
    }

    async fn kill(&mut self) -> Result<()> {
        self.machine.shutdown().await.map_err(Into::into)
    }

    async fn delete(&mut self) -> Result<()> {
        unimplemented!()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DummyNode {
    pub id: String,
    pub state: ContainerStatus,
}

#[async_trait]
impl NodeContainer for DummyNode {
    async fn create(id: &str, _machine_index: usize) -> Result<Self> {
        info!("Creating node: {}", id);
        let node = Self {
            id: id.to_owned(),
            state: ContainerStatus::Created,
        };
        let contents = toml::to_string(&node)?;
        fs::write(format!("/tmp/{}.txt", id), &contents)?;
        Ok(node)
    }

    fn id(&self) -> &str {
        &self.id
    }

    async fn start(&mut self) -> Result<()> {
        info!("Starting node: {}", self.id());
        self.state = ContainerStatus::Started;
        let contents = toml::to_string(&self)?;
        fs::write(format!("/tmp/{}.txt", self.id), &contents)?;
        Ok(())
    }

    async fn state(&self) -> Result<ContainerStatus> {
        Ok(self.state)
    }

    async fn kill(&mut self) -> Result<()> {
        info!("Killing node: {}", self.id());
        self.state = ContainerStatus::Stopped;
        let contents = toml::to_string(&self)?;
        fs::write(format!("/tmp/{}.txt", self.id), &contents)?;
        Ok(())
    }

    async fn delete(&mut self) -> Result<()> {
        info!("Deleting node: {}", self.id());
        self.kill().await?;
        fs::remove_file(format!("/tmp/{}.txt", self.id))?;
        Ok(())
    }
}

pub struct DummyNodeRegistry {}

impl NodeRegistry for DummyNodeRegistry {
    fn contains(id: &str) -> bool {
        Path::new(&format!("/tmp/{}.txt", id)).exists()
    }

    fn get(id: &str) -> Result<Box<dyn NodeContainer>> {
        let node = fs::read_to_string(format!("/tmp/{}.txt", id))?;
        let node: DummyNode = toml::from_str(&node)?;

        Ok(Box::new(DummyNode {
            id: id.to_string(),
            state: node.state,
        }))
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Containers {
    pub containers: HashMap<String, ContainerData>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ContainerData {
    pub id: String,
    pub chain: String,
    pub status: ContainerStatus,
}

impl Containers {
    pub fn load() -> Result<Containers> {
        info!(
            "Reading containers config: {}",
            REGISTRY_CONFIG_FILE.display()
        );
        let config = fs::read_to_string(&*REGISTRY_CONFIG_FILE)?;
        Ok(toml::from_str(&config)?)
    }

    pub fn save(&self) -> Result<()> {
        info!(
            "Writing containers config: {}",
            REGISTRY_CONFIG_FILE.display()
        );
        let config = toml::Value::try_from(self)?;
        let config = toml::to_string(&config)?;
        fs::write(&*REGISTRY_CONFIG_FILE, &*config)?;
        Ok(())
    }

    pub fn exists() -> bool {
        Path::new(&*REGISTRY_CONFIG_FILE).exists()
    }
}
