use crate::engine::NodeEnv;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

const PROTOCOL_DATA_LOCK_FILENAME: &str = ".protocol_data.lock";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BinaryStatus {
    Ok,
    ChecksumMismatch,
    Missing,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Binary {
    Destination(PathBuf),
    Bin(Vec<u8>),
    Checksum(u32),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct BabelConfig {
    pub node_env: NodeEnv,
    /// RAM disks configuration.
    pub ramdisks: Vec<RamdiskConfiguration>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RamdiskConfiguration {
    /// Path to mount RAM disk to.
    pub ram_disk_mount_point: String,
    /// RAM disk size, in MB.
    pub ram_disk_size_mb: u64,
}

pub fn is_protocol_data_locked(data_mount_point: &Path) -> bool {
    data_mount_point.join(PROTOCOL_DATA_LOCK_FILENAME).exists()
}

pub fn lock_protocol_data(data_mount_point: &Path) -> eyre::Result<()> {
    let data_lock = data_mount_point.join(PROTOCOL_DATA_LOCK_FILENAME);
    if !data_lock.exists() {
        fs::File::create(data_lock)?;
    }
    Ok(())
}
