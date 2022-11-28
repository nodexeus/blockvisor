use async_trait::async_trait;
#[cfg(target_os = "linux")]
use babel::vsock;
use babel::{config, run_flag::RunFlag, supervisor};
use std::path::Path;
use std::time::{Duration, SystemTime};
use tokio::fs::DirBuilder;
use tokio::sync::broadcast;
use tracing_subscriber::util::SubscriberInitExt;

const DATA_DRIVE_PATH: &str = "/dev/vdb";

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish()
        .init();

    let cfg_path = Path::new("/etc/babel.conf");
    tracing::info!("Loading babel configuration at {}", cfg_path.display());
    let cfg = config::load(cfg_path).await?;
    tracing::debug!("Loaded babel configuration: {:?}", &cfg);

    let data_dir = &cfg.config.data_directory_mount_point;
    tracing::info!("Recursively creating data directory at {data_dir}");
    DirBuilder::new().recursive(true).create(&data_dir).await?;
    tracing::info!("Mounting data directory at {data_dir}");
    // We assume that root drive will become /dev/vda, and data drive will become /dev/vdb inside VM
    // However, this can be a wrong assumption ¯\_(ツ)_/¯:
    // https://github.com/firecracker-microvm/firecracker-containerd/blob/main/docs/design-approaches.md#block-devices
    let output = tokio::process::Command::new("mount")
        .args([DATA_DRIVE_PATH, data_dir])
        .output()
        .await?;
    tracing::debug!("Mounted data directory: {output:?}");

    let run = RunFlag::run_until_ctrlc();
    let supervisor_cfg = cfg.supervisor.clone();

    let supervisor = supervisor::Supervisor::<SysTimer>::new(run.clone(), supervisor_cfg);
    let logs_rx = supervisor.get_logs_rx();
    let (supervisor, server) = tokio::join!(supervisor.run(), serve(run, cfg, logs_rx));
    supervisor?;
    server
}

struct SysTimer;

#[async_trait]
impl supervisor::Timer for SysTimer {
    fn now() -> SystemTime {
        SystemTime::now()
    }

    async fn sleep(duration: Duration) {
        tokio::time::sleep(duration).await
    }
}

#[cfg(target_os = "linux")]
async fn serve(
    run: RunFlag,
    cfg: config::Babel,
    logs_rx: broadcast::Receiver<String>,
) -> eyre::Result<()> {
    vsock::serve(run, cfg, logs_rx).await
}

#[cfg(not(target_os = "linux"))]
async fn serve(
    _run: RunFlag,
    _cfg: config::Babel,
    _logs_rx: broadcast::Receiver<String>,
) -> eyre::Result<()> {
    unimplemented!()
}
