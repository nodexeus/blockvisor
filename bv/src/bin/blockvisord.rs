use blockvisord::blockvisord::BlockvisorD;
use blockvisord::linux_platform::LinuxPlatform;
use bv_utils::logging::setup_logging;
use bv_utils::run_flag::RunFlag;
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    setup_logging();
    let run = RunFlag::run_until_ctrlc();
    let pal = LinuxPlatform::new()?;
    BlockvisorD::new(pal).await?.run(run).await?;
    Ok(())
}
