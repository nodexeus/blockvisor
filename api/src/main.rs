use anyhow::Result;
use blockvisor_api::{config::AppConfig, server, telemetry};
use tracing::debug;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::new()?;
    telemetry::init(&config)?;
    debug!(?config);
    server::start(&config).await?;
    Ok(())
}
