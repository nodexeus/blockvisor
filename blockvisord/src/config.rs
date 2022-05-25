use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::info;

const HOST_CONFIG_FILENAME: &str = "blockvisor.toml";

lazy_static::lazy_static! {
    static ref HOST_CONFIG_FILE: PathBuf = home::home_dir()
        .map(|p| p.join(".config"))
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(HOST_CONFIG_FILENAME);
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub id: String,
    pub data_dir: String,
    pub pool_dir: String,
    pub token: String,
    pub blockjoy_api_url: String,
}

impl Config {
    pub fn load() -> Result<Config> {
        info!("Reading host config: {}", HOST_CONFIG_FILE.display());
        let config = fs::read_to_string(&*HOST_CONFIG_FILE)?;
        Ok(toml::from_str(&config)?)
    }

    pub fn save(&self) -> Result<()> {
        info!("Writing host config: {}", HOST_CONFIG_FILE.display());
        let config = toml::Value::try_from(self)?;
        let config = toml::to_string(&config)?;
        fs::write(&*HOST_CONFIG_FILE, &*config)?;
        Ok(())
    }

    pub fn exists() -> bool {
        Path::new(&*HOST_CONFIG_FILE).exists()
    }
}
