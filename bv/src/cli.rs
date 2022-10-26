use clap::{ArgEnum, Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "bv", author, version, about)]
pub struct App {
    #[clap(flatten)]
    pub global_opts: GlobalOpts,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// One-time password
    pub otp: String,

    /// BlockJoy API url
    #[clap(long = "url", default_value = "https://api.blockvisor.dev")]
    pub blockjoy_api_url: String,

    /// Network interface name
    #[clap(long = "ifa", default_value = "bvbr0")]
    pub ifa: String,
}

#[derive(Debug, Args)]
pub struct ResetArgs {
    /// Assume "yes" as answer to all prompts
    #[clap(short, long)]
    pub yes: bool,
}

#[derive(Debug, Args)]
pub struct StartArgs {}

#[derive(Debug, Args)]
pub struct StopArgs {}

#[derive(Debug, Args)]
pub struct StatusArgs {}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialise blockvisor to run on this host
    #[clap(alias = "i")]
    Init(InitArgs),

    /// Completelly remove all nodes, configs and unregister the host from the API
    Reset(ResetArgs),

    /// Start blockvisor service
    Start(StartArgs),

    /// Stop blockvisor service
    Stop(StopArgs),

    /// Return blockvisor status
    Status(StatusArgs),

    /// Manage nodes on this host
    #[clap(alias = "n")]
    Node {
        #[clap(subcommand)]
        command: NodeCommand,
    },

    /// Manage host configuration and collect host info
    #[clap(alias = "h")]
    Host {
        #[clap(subcommand)]
        command: HostCommand,
    },

    /// Get information about chains
    #[clap(alias = "c")]
    Chain {
        #[clap(subcommand)]
        command: ChainCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum NodeCommand {
    /// Show nodes list
    #[clap(alias = "ls")]
    List {
        /// Display only running nodes
        #[clap(long, short)]
        running: bool,

        /// Display nodes of particular chain
        #[clap(long, short)]
        chain: Option<String>,
    },

    /// Create node
    #[clap(alias = "c")]
    Create {
        /// Chain identifier
        chain: String,

        /// Node IP Address
        #[clap(long)]
        ip: Option<String>,

        /// Gateway IP Address
        #[clap(long)]
        gateway: Option<String>,
    },

    /// Start node
    Start {
        /// Node id or name
        id_or_name: String,
    },

    /// Stop node
    Stop {
        /// Node id or name
        id_or_name: String,
    },

    /// Restart node
    Restart {
        /// Node id or name
        id_or_name: String,
    },

    /// Delete node and clean up resources
    #[clap(alias = "d")]
    Delete {
        /// Node id or name
        id_or_name: String,
    },

    /// Attach to node console
    #[clap(alias = "c")]
    Console {
        /// Node id or name
        id_or_name: String,
    },

    /// Display node logs
    #[clap(alias = "l")]
    Logs {
        /// Node id or name
        id_or_name: String,
    },

    /// Get node status
    Status {
        /// Node id or name
        id_or_name: String,
    },

    /// Return the block height of the blockchain the node is running
    #[clap(alias = "caps")]
    Capabilities {
        /// Node id or name
        id_or_name: String,
    },

    /// Return the block height of the blockchain the node is running
    Height {
        /// Node id or name
        id_or_name: String,
    },

    /// Return the block height of the blockchain the node is running
    BlockAge {
        /// Node id or name
        id_or_name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum HostCommand {
    /// Collect host system information
    #[clap(alias = "i")]
    Info,
}

#[derive(Debug, Subcommand)]
pub enum ChainCommand {
    /// Show chains list
    #[clap(alias = "ls")]
    List,

    /// Display chain status
    Status {
        /// Chain identifier
        id: String,
    },

    /// Run chain synchronization process
    Sync {
        /// Chain identifier
        id: String,
    },
}

#[derive(ArgEnum, PartialEq, Eq, Debug, Clone)]
pub enum FormatArg {
    Text,
    Json,
}

#[derive(Debug, Args)]
pub struct GlobalOpts {
    /// Verbosity level (can be specified multiple times)
    #[clap(long, short, global = true, parse(from_occurrences))]
    verbose: usize,

    /// Output format
    #[clap(long, short, global = true, arg_enum, default_value = "text")]
    pub format: FormatArg,
}
