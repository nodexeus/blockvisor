use crate::{bv_config::ApptainerConfig, firewall, utils};
use babel_api::utils::RamdiskConfiguration;
use chrono::serde::ts_seconds_option;
use chrono::{DateTime, Utc};
use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, fmt::Debug, mem, net::IpAddr, path::Path, time::SystemTime};
use tokio::fs;
use tracing::{error, info};
use uuid::Uuid;

pub const NODE_STATE_FILENAME: &str = "state.json";
pub type NodeProperties = HashMap<String, String>;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub enum VmStatus {
    Running,
    Stopped,
    Busy,
    Failed,
}

impl fmt::Display for VmStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct NodeImage {
    pub id: String,
    pub version: String,
    pub config_id: String,
    pub archive_id: String,
    pub store_key: String,
    pub uri: String,
    pub min_babel_version: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct NodeInfo {
    pub id: Uuid,
    pub name: String,
    pub status: VmStatus,

    pub uptime: Option<i64>,
    pub requirements: Option<VmConfig>,
    pub properties: NodeProperties,
    pub assigned_cpus: Vec<usize>,
    pub dev_mode: bool,
}

/// Node state data that we store in state file
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct NodeState {
    // static properties
    pub id: Uuid,
    pub name: String,
    pub protocol_id: String,
    pub image_key: ProtocolImageKey,
    pub dev_mode: bool,
    // potentially configurable
    pub ip: IpAddr,
    pub gateway: IpAddr,

    // dynamic
    pub properties: NodeProperties,
    pub firewall: firewall::Config,

    // dynamic-description
    pub display_name: String,
    pub org_id: String,
    pub org_name: String,
    pub protocol_name: String,
    pub dns_name: String,
    #[serde(default)]
    pub tags: Vec<String>,

    // upgradeable
    pub vm_config: VmConfig,
    pub image: NodeImage,

    // internal state
    pub assigned_cpus: Vec<usize>,
    pub expected_status: VmStatus,
    #[serde(default, with = "ts_seconds_option")]
    /// Time when node was started, None if node should not be running now
    pub started_at: Option<DateTime<Utc>>,
    pub initialized: bool,
    pub restarting: bool,
    pub upgrade_state: UpgradeState,
    pub apptainer_config: Option<ApptainerConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct UpgradeState {
    pub active: bool,
    pub state_backup: Option<StateBackup>,
    pub need_rollback: Option<String>,
    pub steps: Vec<UpgradeStep>,
    pub data_stamp: Option<SystemTime>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StateBackup {
    pub properties: NodeProperties,
    pub firewall: firewall::Config,
    pub display_name: String,
    pub org_id: String,
    pub org_name: String,
    pub protocol_name: String,
    pub dns_name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub vm_config: VmConfig,
    pub image: NodeImage,
    pub initialized: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum UpgradeStep {
    Stop,
    CpuAssignment(CpuAssignmentUpdate),
    Vm,
    Plugin,
    Firewall,
    Restart,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum CpuAssignmentUpdate {
    #[default]
    None,
    AcquiredCpus(usize),
    ReleasedCpus(Vec<usize>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProtocolImageKey {
    /// The key identifier to a protocol.
    pub protocol_key: String,
    /// A unique identifier to the implementation variant (e.g. reth or geth).
    pub variant_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct VmConfig {
    /// Virtual cores to share with VM.
    pub vcpu_count: usize,
    /// RAM allocated to VM in MB.
    pub mem_size_mb: u64,
    /// Size of data drive for storing protocol data (not to be confused with OS drive).
    pub disk_size_gb: u64,
    /// RAM disks configuration.
    pub ramdisks: Vec<RamdiskConfiguration>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct ConfigUpdate {
    pub config_id: String,
    pub new_org_id: Option<String>,
    pub new_org_name: Option<String>,
    pub new_display_name: Option<String>,
    pub new_values: NodeProperties,
    pub new_firewall: Option<firewall::Config>,
}

impl NodeState {
    pub async fn load(path: &Path) -> Result<Self> {
        info!("Reading node state file: {}", path.display());
        let state = fs::read_to_string(&path)
            .await
            .and_then(|content| match serde_json::from_str::<Self>(&content) {
                Ok(state) => Ok(state),
                Err(err) => {
                    error!("{err:#}");
                    Err(err.into())
                }
            })
            .with_context(|| format!("Failed to read node state file `{}`", path.display()))?;
        Ok(state)
    }

    pub async fn save(&self, nodes_dir: &Path) -> Result<()> {
        let path = nodes_dir
            .join(self.id.to_string())
            .join(NODE_STATE_FILENAME);
        info!("Writing node state: {}", path.display());
        utils::careful_save(&path, serde_json::to_string(self)?.as_bytes()).await?;
        Ok(())
    }
}

impl StateBackup {
    pub fn swap_state(&mut self, node_state: &mut NodeState) {
        mem::swap(&mut self.image, &mut node_state.image);
        mem::swap(&mut self.vm_config, &mut node_state.vm_config);
        mem::swap(&mut self.properties, &mut node_state.properties);
        mem::swap(&mut self.firewall, &mut node_state.firewall);
        mem::swap(&mut self.protocol_name, &mut node_state.protocol_name);
        mem::swap(&mut self.org_name, &mut node_state.org_name);
        mem::swap(&mut self.org_id, &mut node_state.org_id);
        mem::swap(&mut self.display_name, &mut node_state.display_name);
        mem::swap(&mut self.tags, &mut node_state.tags);
        mem::swap(&mut self.dns_name, &mut node_state.dns_name);
        mem::swap(&mut self.initialized, &mut node_state.initialized);
    }
}

impl UpgradeState {
    pub fn insert_step(&mut self, step: UpgradeStep) -> bool {
        if self.steps.contains(&step) {
            false
        } else {
            self.steps.push(step);
            true
        }
    }
}

impl From<NodeState> for StateBackup {
    fn from(value: NodeState) -> Self {
        Self {
            properties: value.properties,
            firewall: value.firewall,
            display_name: value.display_name,
            org_id: value.org_id,
            org_name: value.org_name,
            protocol_name: value.protocol_name,
            dns_name: value.dns_name,
            tags: value.tags,
            vm_config: value.vm_config,
            image: value.image,
            initialized: false,
        }
    }
}
