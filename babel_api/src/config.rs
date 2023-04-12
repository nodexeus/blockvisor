use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, collections::HashMap};

pub type KeysConfig = HashMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Babel {
    pub nets: HashMap<String, NetConfiguration>,
    pub export: Option<Vec<String>>,
    pub env: Option<Env>,
    pub config: Config,
    pub requirements: Requirements,
    ///Commands to start blockchain node
    pub supervisor: SupervisorConfig,
    /// Firewall configuration that is applied on node start.
    pub firewall: Option<firewall::Config>,
    pub keys: Option<KeysConfig>,
    #[serde(
        deserialize_with = "deserialize_methods",
        serialize_with = "serialize_methods"
    )]
    pub methods: BTreeMap<String, Method>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SupervisorConfig {
    /// Path to mount data drive to
    pub data_directory_mount_point: String,
    ///  if entry_point stay alive given amount of time (in miliseconds) backoff is reset
    pub backoff_timeout_ms: u64,
    /// base time (in miliseconds) for backof, multiplied by consecutive power of 2 each time
    pub backoff_base_ms: u64,
    /// capacity of log buffer (in lines)
    pub log_buffer_capacity_ln: usize,
    pub entry_point: Vec<Entrypoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BabelConfig {
    /// Path to mount data drive to
    pub data_directory_mount_point: String,
    /// capacity of log buffer (in lines)
    pub log_buffer_capacity_ln: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entrypoint {
    pub name: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobConfig {
    pub body: String,
    pub restart: RestartPolicy,
    pub needs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RestartConfig {
    /// if entry_point stay alive given amount of time (in miliseconds) backoff is reset
    pub backoff_timeout_ms: u64,
    /// base time (in miliseconds) for backof, multiplied by consecutive power of 2 each time
    pub backoff_base_ms: u64,
    /// maximum number of retries, or `None` if there is no such limit
    pub max_retries: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RestartPolicy {
    /// Indicates that this job will never be restarted, whether succeeded or not - appropriate for jobs
    /// that can't be simply restarted on failure (e.g. need some manual actions).
    Never,
    /// Job is always restarted - equivalent to entrypoint.
    Always(RestartConfig),
    /// Job is restarted only if `exit_code != 0`.
    OnFailure(RestartConfig),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    /// The current job was requested to start, but the process has not been launched yet.
    /// It was not picked by the JobRunner yet, or needs another job to be finished first.
    /// Every job starts with this state.
    Pending,
    /// The JobRunner actually picked that job.
    Running,
    /// Job finished - successfully or not. It means that the JobRunner won't try to restart that job anymore.
    Finished {
        /// Job `sh` script exit code, if any. `None` always means some error, usually before the process
        /// was even started (e.g. needed job failed).  
        exit_code: Option<i32>,
        /// Error description or empty if successful.
        message: String,
    },
    /// Job was explicitly stopped.
    Stopped,
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            data_directory_mount_point: "/blockjoy/miner/data".to_string(),
            backoff_timeout_ms: 60_000,
            backoff_base_ms: 100,
            log_buffer_capacity_ln: 1_000,
            entry_point: vec![],
        }
    }
}

pub fn deserialize_methods<'de, D>(deserializer: D) -> Result<BTreeMap<String, Method>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let methods = Vec::<Method>::deserialize(deserializer)?;
    let map = methods
        .into_iter()
        .map(|m| (m.name().to_string(), m))
        .collect();

    Ok(map)
}

fn serialize_methods<S>(value: &BTreeMap<String, Method>, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let vectorized: Vec<&Method> = value.iter().map(|(_, v)| v).collect();
    vectorized.serialize(s)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Env {
    pub path_append: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// A semver version of the babel program, indicating the minimum version of the babel
    /// program that a config file is compatible with.
    pub min_babel_version: String,
    /// A semver version of the blockchain node program.
    pub node_version: String,
    /// Name of the blockchain protocol
    pub protocol: String,
    /// Type of the node (validator, beacon, etc)
    pub node_type: String,
    /// Some description of the node
    pub description: Option<String>,
    /// The url where the node exposes its endpoints. Since the blockchain node is running on the
    /// same OS as babel, this will be a local url (usually). Example: `http://localhost:4467/`.
    pub api_host: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirements {
    // Virtual cores to share with VM
    pub vcpu_count: usize,
    // RAM allocated to VM
    pub mem_size_mb: usize,
    // Size of data drive for storing blockchain data (not to be confused with OS drive)
    pub disk_size_gb: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NetType {
    Dev,
    Test,
    Main,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetConfiguration {
    pub url: String,
    pub net_type: NetType,
    #[serde(flatten)]
    pub meta: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "transport", rename_all = "kebab-case")]
pub enum Method {
    Jrpc {
        /// This field is ignored.
        name: String,
        /// The name of the jRPC method that we are going to call into.
        method: String,
        /// This field is ignored.
        response: JrpcResponse,
    },
    Rest {
        /// This field is ignored.
        name: String,
        /// This is the relative url of the rest endpoint. So if the host is `"https://api.com/"`,
        /// and the method is `"/v1/users"`, then the url that called is
        /// `"https://api.com/v1/users"`.
        method: String,
        /// These are the configuration options for parsing the response.
        response: RestResponse,
    },
    Sh {
        /// This field is ignored.
        name: String,
        /// These are the arguments to the sh command that is executed for this `Method`.
        body: String,
        /// These are the configuration options for parsing the response.
        response: ShResponse,
    },
}

impl Method {
    pub fn name(&self) -> &str {
        match self {
            Method::Jrpc { name, .. } => name,
            Method::Rest { name, .. } => name,
            Method::Sh { name, .. } => name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JrpcResponse {
    pub code: u32,
    pub field: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestResponse {
    pub status: u32,
    pub field: Option<String>,
    pub format: MethodResponseFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShResponse {
    pub status: i32,
    pub format: MethodResponseFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MethodResponseFormat {
    Raw,
    Json,
}

pub mod firewall {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Rule {
        pub name: String,
        pub action: Action,
        pub direction: Direction,
        pub protocol: Option<Protocol>,
        pub ips: Option<String>,
        pub ports: Vec<u16>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Config {
        pub enabled: bool,
        pub default_in: Action,
        pub default_out: Action,
        pub rules: Vec<Rule>,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                enabled: true,
                default_in: firewall::Action::Deny,
                default_out: firewall::Action::Allow,
                rules: vec![],
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum Action {
        Allow,
        Deny,
        Reject,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum Direction {
        Out,
        In,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum Protocol {
        Tcp,
        Udp,
        Both,
    }
}
