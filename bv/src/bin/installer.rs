use anyhow::Result;
use async_trait::async_trait;
use blockvisord::installer;
use blockvisord::installer::Installer;
use blockvisord::linux_platform::bv_root;
use bv_utils::{cmd::run_cmd, timer::SysTimer};
use tracing::error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, FmtSubscriber};

struct SystemCtl;

#[async_trait]
impl installer::BvService for SystemCtl {
    async fn reload(&self) -> Result<()> {
        run_cmd("systemctl", ["daemon-reload"]).await
    }

    async fn stop(&self) -> Result<()> {
        run_cmd("systemctl", ["stop", "blockvisor.service"]).await
    }

    async fn start(&self) -> Result<()> {
        run_cmd("systemctl", ["start", "blockvisor.service"]).await
    }

    async fn enable(&self) -> Result<()> {
        run_cmd("systemctl", ["enable", "blockvisor.service"]).await
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .with(tracing_journald::layer()?)
        .init();

    let installer = Installer::new(SysTimer, SystemCtl, &bv_root()).await?;
    if let Err(err) = installer.run().await {
        error!("{err}");
        Err(err)
    } else {
        Ok(())
    }
}
