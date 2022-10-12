use std::path::Path;
use tracing::debug;

// TODO: What are we going to use as backup when vsock is disabled?
#[cfg(feature = "vsock")]
mod vsock;

mod client;
mod config;
mod error;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let cfg = config::load(Path::new("/etc/babel.conf")).await?;
    debug!("Loaded babel configuration: {:?}", cfg);

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
