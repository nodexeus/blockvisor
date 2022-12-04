use eyre::Result;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{self, EnvFilter, FmtSubscriber};

pub fn setup_logging() -> Result<()> {
    FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .init();

    Ok(())
}
