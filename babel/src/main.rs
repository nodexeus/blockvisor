use std::path::Path;
use tokio::signal::unix;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{self, EnvFilter, FmtSubscriber};

// TODO: What are we going to use as backup when vsock is disabled?
#[cfg(feature = "vsock")]
mod client;
mod config;
mod error;
#[cfg(feature = "vsock")]
mod vsock;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .init();

    let cfg_path = Path::new("/etc/babel.conf");
    tracing::info!("Loading babel configuration at {}", cfg_path.display());
    let cfg = config::load(cfg_path).await?;
    tracing::debug!("Loaded babel configuration: {:?}", cfg);

    tokio::spawn(async move {
        let mut signals = unix::signal(unix::SignalKind::interrupt()).unwrap();
        signals.recv().await;
        println!("Received sigint");
    });

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
