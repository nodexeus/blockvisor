use std::path::Path;
use tracing_subscriber::util::SubscriberInitExt;

// TODO: What are we going to use as backup when vsock is disabled?
#[cfg(feature = "vsock")]
mod client;
mod config;
mod error;
#[cfg(feature = "vsock")]
mod vsock;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish()
        .init();

    let cfg_path = Path::new("/etc/babel.conf");
    tracing::info!("Loading babel configuration at {}", cfg_path.display());
    let cfg = config::load(cfg_path).await?;
    tracing::debug!("Loaded babel configuration: {:?}", cfg);

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
