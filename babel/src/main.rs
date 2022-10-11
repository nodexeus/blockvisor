use std::path::Path;
use tracing::debug;

#[cfg(feature = "vsock")]
mod api;

mod config;

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
    api::serve(cfg).await
}

#[cfg(not(feature = "vsock"))]
async fn serve(_cfg: config::Babel) -> eyre::Result<()> {
    unimplemented!()
}
