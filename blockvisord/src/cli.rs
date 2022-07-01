use clap::{ArgEnum, Args, Parser, Subcommand};
use uuid::Uuid;

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
        /// Should we display all nodes including stopped
        #[clap(long, short)]
        all: bool,

        /// Display nodes of particular chain
        #[clap(long, short)]
        chain: Option<String>,
    },

    /// Create node
    #[clap(alias = "c")]
    Create {
        /// Chain identifier
        chain: String,
    },

    /// Start node
    Start {
        /// Node id
        id: Uuid,
    },

    /// Stop node
    Stop {
        /// Node id
        id: Uuid,
    },

    /// Restart node
    Restart {
        /// Node id
        id: Uuid,
    },

    /// Delete node and clean up resources
    #[clap(alias = "d")]
    Delete {
        /// Node id
        id: Uuid,
    },

    /// Attach to node console
    #[clap(alias = "c")]
    Console {
        /// Node id
        id: Uuid,
    },

    /// Display node logs
    #[clap(alias = "l")]
    Logs {
        /// Node id
        id: Uuid,
    },
}

#[derive(Debug, Subcommand)]
pub enum HostCommand {
    /// Collect host system information
    #[clap(alias = "i")]
    Info,

    /// Manage host network configuration
    #[clap(alias = "n")]
    Network {
        #[clap(subcommand)]
        command: HostNetworkCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum HostNetworkCommand {
    #[clap(alias = "i")]
    Info,

    #[clap(alias = "b")]
    Bridge {
        #[clap(subcommand)]
        command: HostNetworkBridgeCommand,
    },

    Ip {
        #[clap(subcommand)]
        command: HostNetworkIpCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum HostNetworkBridgeCommand {
    #[clap(alias = "ls")]
    List,

    #[clap(alias = "c")]
    Create {
        #[clap(long)]
        name: String,
    },

    #[clap(alias = "d")]
    Delete {
        #[clap(long)]
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum HostNetworkIpCommand {
    #[clap(alias = "ls")]
    List,

    #[clap(alias = "a")]
    Add {
        #[clap(long)]
        net: String,
    },

    #[clap(alias = "d")]
    Delete {
        #[clap(long)]
        net: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ChainCommand {
    /// Show chains list
    #[clap(alias = "ls")]
    List,

    /// Display chain status
    Status {
        /// Chain identifier
        #[clap(long)]
        id: String,
    },

    /// Run chain synchronization process
    Sync {
        /// Chain identifier
        #[clap(long)]
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
