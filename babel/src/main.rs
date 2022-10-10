use tracing::debug;

mod api;
mod config;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let cfg = config::load("/etc/babel.conf").await?;
    debug!("Loaded babel configuration: {:?}", cfg);

    api::serve(cfg).await
}
