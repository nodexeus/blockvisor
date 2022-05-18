use std::path::Path;

use anyhow::{Result, Ok};
use async_trait::async_trait;
use firec::Machine;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub enum ServiceStatus {
    Enabled,
    Disabled,
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Copy, Debug)]
pub enum ContainerStatus {
    Created,
    Started,
    Killed,
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
        let mut jailer = firec::config::Jailer::default();
        jailer.chroot_base_dir = Path::new(CHROOT_PATH).into();
        jailer.exec_file = Path::new(FC_BIN_PATH).into();

        let mut root_drive = firec::config::Drive::default();
        root_drive.drive_id = "root".into();
        root_drive.path_on_host = Path::new(ROOT_FS).into();
        root_drive.is_root_device = true;

        let kernel_args = Some(format!(
            "console=ttyS0 reboot=k panic=1 pci=off random.trust_cpu=on \
            ip=74.50.82.8{}::74.50.82.82:255.255.255.240::eth0:on",
            machine_index + 3,
        ).into());

        let mut cni = firec::config::network::Cni::default();
        cni.network_name = "eth0".into();
        cni.if_name = Some(format!("bv{}", machine_index).into());
        let iface = firec::config::network::Interface::Cni(cni);

        let mut machine_cfg = firec::config::Machine::default();
        machine_cfg.vcpu_count = 1;
        machine_cfg.mem_size_mib = 8192;

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

pub struct DummyNode {
    pub id: String,
    pub state: ContainerStatus,
}

#[async_trait]
impl NodeContainer for DummyNode {
    async fn create(id: &str, _machine_index: usize) -> Result<Self> {
        println!("Creating node: {}", id);
        Ok(Self {
            id: id.to_owned(),
            state: ContainerStatus::Created,
        })
    }

    fn id(&self) -> &str {
        &self.id
    }

    async fn start(&mut self) -> Result<()> {
        println!("Starting node: {}", self.id());
        self.state = ContainerStatus::Started;
        Ok(())
    }

    async fn state(&self) -> Result<ContainerStatus> {
        Ok(self.state)
    }

    async fn kill(&mut self) -> Result<()> {
        println!("Killing node: {}", self.id());
        self.state = ContainerStatus::Killed;
        Ok(())
    }

    async fn delete(&mut self) -> Result<()> {
        println!("Deleting node: {}", self.id());
        self.kill().await?;
        Ok(())
    }
}
