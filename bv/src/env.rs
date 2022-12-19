use std::env;
use std::path::PathBuf;

const ENV_BV_ROOT_KEY: &str = "BV_ROOT";

lazy_static::lazy_static! {
    pub static ref ROOT_DIR: PathBuf = PathBuf::from(env::var(ENV_BV_ROOT_KEY).unwrap_or_else(|_| "/".to_string()));
    pub static ref VARS_DIR: PathBuf = ROOT_DIR.join("var").join("lib").join("blockvisor");

    pub static ref HOST_CONFIG_FILE: PathBuf = ROOT_DIR
        .join("etc")
        .join("blockvisor.toml");

    pub static ref REGISTRY_CONFIG_DIR: PathBuf = VARS_DIR.join("nodes");
    pub static ref REGISTRY_CONFIG_FILE: PathBuf = REGISTRY_CONFIG_DIR.join("nodes.toml");

    pub static ref IMAGE_CACHE_DIR: PathBuf = VARS_DIR.join("images");

    pub static ref DATA_PATH: PathBuf = VARS_DIR.join("data.img");
    pub static ref CHROOT_PATH: PathBuf = VARS_DIR.clone();
}
