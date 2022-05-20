use crate::containers::{
    ContainerStatus, DummyNode, DummyNodeRegistry, NodeContainer, NodeRegistry,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};
use sysinfo::{DiskExt, System, SystemExt};

const CONFIG_FILE: &str = "/tmp/config.toml";

pub struct Host {
    pub containers: HashMap<String, Box<dyn NodeContainer>>,
    pub config: HostConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ContainerConfig {
    pub id: String,
    pub chain: String,
    pub status: ContainerStatus,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HostConfig {
    pub id: String,
    pub containers: HashMap<String, ContainerConfig>,
    pub data_dir: String,
    pub pool_dir: String,
    pub token: String,
    pub blockjoy_api_url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HostInfo {
    pub name: Option<String>,
    pub cpu_count: Option<i64>, // because postgres does not have unsigned
    pub mem_size: Option<i64>,
    pub disk_size: Option<i64>,
    pub os: Option<String>,
    pub os_version: Option<String>,
}

pub fn get_host_info() -> HostInfo {
    let sys = System::new_all();

    HostInfo {
        name: sys.name(),
        cpu_count: sys.physical_core_count().map(|x| x as i64),
        mem_size: Some(sys.total_memory() as i64 * 1024),
        disk_size: Some(sys.disks()[0].total_space() as i64),
        os: sys.name(),
        os_version: sys.os_version(),
    }
}

pub fn get_ip_address(ifa_name: &str) -> String {
    let ifas = local_ip_address::list_afinet_netifas().unwrap();
    let (_, ip) = local_ip_address::find_ifa(ifas, ifa_name).unwrap();
    ip.to_string()
}

// TODO: probably should get into config type
pub fn read_config() -> Result<HostConfig> {
    println!("Reading config: {}", CONFIG_FILE);
    let config = fs::read_to_string(CONFIG_FILE)?;
    Ok(toml::from_str(&config)?)
}

pub fn write_config(config: HostConfig) -> Result<()> {
    println!("Writing config: {}", CONFIG_FILE);
    let config = toml::Value::try_from(&config)?;
    let config = toml::to_string(&config)?;
    fs::write(CONFIG_FILE, config)?;
    Ok(())
}

pub fn config_exists() -> bool {
    Path::new(CONFIG_FILE).exists()
}

// used for testing purposes
pub async fn dummy_apply_config(config: &HostConfig, machine_index: &mut usize) -> Result<()> {
    for (id, container_config) in &config.containers {
        // remove deleted nodes
        if container_config.status == ContainerStatus::Deleted {
            if DummyNodeRegistry::contains(id) {
                let mut node = DummyNodeRegistry::get(id)?;
                node.delete().await?;
            }
        } else {
            // create non existing nodes
            if !DummyNodeRegistry::contains(id) {
                DummyNode::create(id, *machine_index).await?;
                *machine_index += 1;
            }

            // fix nodes status
            let mut node = DummyNodeRegistry::get(id)?;
            let state = node.state().await?;
            if state != container_config.status {
                println!(
                    "Changing state from {:?} to {:?}: {}",
                    state, container_config.status, id
                );
                match container_config.status {
                    ContainerStatus::Started => node.start().await?,
                    ContainerStatus::Stopped => node.kill().await?,
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
