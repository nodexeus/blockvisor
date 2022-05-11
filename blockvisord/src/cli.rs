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
    #[clap(long = "url")]
    pub blockjoy_api_url: String,
}

#[derive(Debug, Args)]
pub struct StartArgs {
    /// Should the app run as daemon
    #[clap(long, short)]
    pub daemonize: bool,

    /// Path to config file
    #[clap(long, short, default_value = "config.toml")]
    pub config_path: Utf8PathBuf,
}

#[derive(Debug, Args)]
pub struct StopArgs {
    /// Path to config file
    #[clap(long, short, default_value = "config.toml")]
    pub config_path: Utf8PathBuf,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Configure blockvisor to run on this host
    Configure(ConfigureArgs),

    /// Start blockvisor service
    Start(StartArgs),

    /// Stop blockvisor service
    Stop(StopArgs),

    /// Manage nodes on this host
    Node {
        #[clap(subcommand)]
        command: NodeCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum NodeCommand {
    /// Create node
    Create {
        /// Node type
        #[clap(long, arg_enum)]
        r#type: NodeType,

        /// Node id
        #[clap(long)]
        id: String,
    },

    /// List created nodes
    List,

    /// Delete node
    Delete {
        /// Node id
        #[clap(long)]
        id: String,
    },
}

#[derive(Clone, Debug, ArgEnum)]
pub enum NodeType {
    HeliumLatest,
}

#[derive(Debug, Args)]
pub struct GlobalOpts {
    /// Verbosity level (can be specified multiple times)
    #[clap(long, short, global = true, parse(from_occurrences))]
    verbose: usize,
}
