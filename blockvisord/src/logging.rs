use anyhow::Result;
use tracing::Level;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{self, FmtSubscriber};

pub fn setup_logging(level: Level) -> Result<()> {
    FmtSubscriber::builder()
        .json()
        .with_max_level(level)
        .with_ansi(false)
        .finish()
        .init();

    Ok(())
}
