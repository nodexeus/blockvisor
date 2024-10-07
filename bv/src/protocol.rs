use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Protocol {
    /// Globally unique protocol key.
    pub key: String,
    /// Display name visible in frontend - can be modified.
    pub name: String,
    /// Uuid of organization where which protocol belongs to, or null if public.
    pub org_id: Option<String>,
    pub ticker: Option<String>,
    /// Brief protocol description.
    pub description: Option<String>,
    /// Protocols visibility.
    pub visibility: Visibility,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ImageKey {
    pub protocol_key: String,
    pub variant_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Image {
    /// Set by image provider, shall follow semver.
    pub version: String,
    pub container_uri: String,
    pub key: ImageKey,
    pub sku_code: String,
    pub org_id: Option<String>,
    pub description: Option<String>,
    pub visibility: Visibility,
    pub properties: Vec<ImageProperty>,
    pub firewall_config: FirewallConfig,
    pub min_cpu: u64,
    pub min_memory_bytes: u64,
    pub min_disk_bytes: u64,
    pub ramdisks: Vec<RamdiskConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RamdiskConfig {
    pub mount: String,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FirewallConfig {
    pub default_in: Action,
    pub default_out: Action,
    pub rules: Vec<FirewallRule>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FirewallRule {
    pub key: String,
    pub description: Option<String>,
    pub protocol: NetProtocol,
    pub direction: Direction,
    pub action: Action,
    pub ips: Vec<IpName>,
    pub ports: Vec<PortName>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Allow,
    Deny,
    Reject,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Out,
    In,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NetProtocol {
    Tcp,
    Udp,
    Both,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Private,
    Public,
    Development,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct IpName {
    pub ip: String,
    pub name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PortName {
    pub port: u16,
    pub name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ImageProperty {
    pub key: String,
    pub description: Option<String>,
    pub dynamic_value: bool,
    pub default_value: String,
    pub ui_type: UiType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ImageImpact {
    pub new_archive: bool,
    pub add_cpu: Option<i64>,
    pub add_memory_bytes: Option<i64>,
    pub add_disk_bytes: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UiType {
    Switch { on: EnumVariant, off: EnumVariant },
    Text(Option<ImageImpact>),
    Password(Option<ImageImpact>),
    Enum(Vec<EnumVariant>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EnumVariant {
    pub value: String,
    pub impact: Option<ImageImpact>,
}
