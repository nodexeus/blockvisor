use camino::Utf8PathBuf;
use clap::{ArgEnum, Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "bvs", author, version, about)]
pub struct App {
    #[clap(flatten)]
    pub global_opts: GlobalOpts,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Args)]
pub struct ConfigureArgs {
    /// One-time password
    #[clap(long)]
    pub otp: String,

    /// BlockJoy API url
    #[clap(long = "url", default_value = "https://api.stakejoy.com")]
    pub blockjoy_api_url: String,

    /// Network interface name
    #[clap(long = "ifa", default_value = "bvbr0")]
    pub ifa: String,
}

#[derive(Debug, Args)]
pub struct StartArgs {
    /// Should the app run as daemon
    #[clap(long, short)]
    pub daemonize: bool,
}

#[derive(Debug, Args)]
pub struct StopArgs {}

#[derive(Debug, Args)]
pub struct StatusArgs {}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Configure blockvisor to run on this host
    Configure(ConfigureArgs),

    /// Start blockvisor service
    Start(StartArgs),

    /// Stop blockvisor service
    Stop(StopArgs),

    /// Return blockvisor status
    Status(StatusArgs),

    /// Manage nodes on this host
    Node {
        #[clap(subcommand)]
        command: NodeCommand,
    },

    /// Manage host configuration and collect host info
    Host {
        #[clap(subcommand)]
        command: HostCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum NodeCommand {
    /// Show nodes list
    List {
        /// Should we display all nodes including stopped
        #[clap(long)]
        all: bool,

        /// Display nodes of particular chain
        #[clap(long)]
        chain: String,
    },

    /// Create node
    Create {
        /// Chain identifier
        #[clap(long)]
        chain: String,
    },

    /// Start node
    Start {
        /// Node id
        #[clap(long)]
        id: String,
    },

    /// Stop node
    Stop {
        /// Node id
        #[clap(long)]
        id: String,
    },

    /// Restart node
    Restart {
        /// Node id
        #[clap(long)]
        id: String,
    },

    /// Delete node and clean up resources
    Delete {
        /// Node id
        #[clap(long)]
        id: String,
    },

    /// Attach to node console
    Console {
        /// Node id
        #[clap(long)]
        id: String,
    },

    /// Display node logs
    Logs {
        /// Node id
        #[clap(long)]
        id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum HostCommand {
    /// Collect host system information
    Info,

    /// Manage host network configuration
    Network {
        #[clap(subcommand)]
        command: HostNetworkCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum HostNetworkCommand {
    Info,

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
    List,

    Create {
        #[clap(long)]
        name: String,
    },

    Delete {
        #[clap(long)]
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum HostNetworkIpCommand {
    List,

    Add {
        #[clap(long)]
        net: String,
    },

    Delete {
        #[clap(long)]
        net: String,
    },
}

#[derive(ArgEnum, PartialEq, Debug, Clone)]
pub enum FormatArg {
    Text,
    Json,
}

#[derive(Debug, Args)]
pub struct GlobalOpts {
    /// Verbosity level (can be specified multiple times)
    #[clap(long, short, global = true, parse(from_occurrences))]
    verbose: usize,

    /// Path to config file
    #[clap(long, short, global = true, default_value = "/tmp/blockvisor.toml")]
    pub config_path: Utf8PathBuf,

    /// Output format
    #[clap(long, short, global = true, arg_enum, default_value = "text")]
    pub format: FormatArg,
}
