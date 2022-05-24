use crate::containers::ContainerStatus;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use tracing::info;

const CONFIG_FILENAME: &str = "blockvisor.toml";

lazy_static::lazy_static! {
    static ref CONFIG_FILE: PathBuf = home::home_dir()
        .map(|p| p.join(".config"))
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(CONFIG_FILENAME);
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
pub struct ContainerConfig {
    pub id: String,
    pub chain: String,
    pub status: ContainerStatus,
}

// TODO: probably should get into config type
pub fn read_config() -> Result<HostConfig> {
    info!("Reading config: {}", CONFIG_FILE.display());
    let config = fs::read_to_string(&*CONFIG_FILE)?;
    Ok(toml::from_str(&config)?)
}

pub fn write_config(config: HostConfig) -> Result<()> {
    info!("Writing config: {}", CONFIG_FILE.display());
    let config = toml::Value::try_from(&config)?;
    let config = toml::to_string(&config)?;
    fs::write(&*CONFIG_FILE, &*config)?;
    Ok(())
}

pub fn config_exists() -> bool {
    Path::new(&*CONFIG_FILE).exists()
}
