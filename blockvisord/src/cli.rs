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
}

#[derive(Debug, Subcommand)]
pub enum NodeCommand {
    /// Create node
    Create,

    /// Delete node and clean up resources
    Kill {
        /// Node id
        #[clap(long)]
        id: String,
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
