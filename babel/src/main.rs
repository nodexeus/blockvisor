use std::path::Path;
use tokio::fs::DirBuilder;
use tracing_subscriber::util::SubscriberInitExt;

// TODO: What are we going to use as backup when vsock is disabled?
#[cfg(feature = "vsock")]
mod client;
mod config;
mod error;
#[cfg(feature = "vsock")]
mod vsock;

const DATA_DRIVE_PATH: &str = "/dev/vdb";

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish()
        .init();

    let cfg_path = Path::new("/etc/babel.conf");
    tracing::info!("Loading babel configuration at {}", cfg_path.display());
    let cfg = config::load(cfg_path).await?;
    tracing::debug!("Loaded babel configuration: {:?}", &cfg);

    let data_dir = &cfg.config.data_directory_mount_point;
    tracing::info!("Recursively creating data directory at {data_dir}");
    DirBuilder::new().recursive(true).create(&data_dir).await?;
    tracing::info!("Mounting data directory at {data_dir}");
    // We assume that root drive will become /dev/vda, and data drive will become /dev/vdb inside VM
    // However, this can be a wrong assumption ¯\_(ツ)_/¯:
    // https://github.com/firecracker-microvm/firecracker-containerd/blob/main/docs/design-approaches.md#block-devices
    let output = tokio::process::Command::new("mount")
        .args([DATA_DRIVE_PATH, data_dir])
        .output()
        .await?;
    tracing::debug!("Mounted data directory: {output:?}");

    serve(cfg).await
}

#[cfg(feature = "vsock")]
async fn serve(cfg: config::Babel) -> eyre::Result<()> {
    vsock::serve(cfg).await
}

#[cfg(not(feature = "vsock"))]
async fn serve(_cfg: config::Babel) -> eyre::Result<()> {
    unimplemented!()
}
