use async_trait::async_trait;
#[cfg(target_os = "linux")]
use babel::vsock;
use babel::{config, logging, run_flag::RunFlag, supervisor};
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::fs::DirBuilder;
use tokio::sync::broadcast;

const CONFIG_PATH: &str = "/etc/babel.conf";
const DATA_DRIVE_PATH: &str = "/dev/vdb";
const VSOCK_HOST_CID: u32 = 3;
const VSOCK_PORT: u32 = 42;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    logging::setup_logging()?;

    let cfg_path = Path::new(CONFIG_PATH);
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
    let (supervisor, server) = tokio::join!(
        supervisor.run(),
        serve(run, cfg, VSOCK_HOST_CID, VSOCK_PORT, logs_rx)
    );
    supervisor?;
    server
}

struct SysTimer;

#[async_trait]
impl supervisor::Timer for SysTimer {
    fn now() -> Instant {
        Instant::now()
    }

    async fn sleep(duration: Duration) {
        tokio::time::sleep(duration).await
    }
}

#[cfg(target_os = "linux")]
async fn serve(
    run: RunFlag,
    cfg: babel_api::config::Babel,
    cid: u32,
    port: u32,
    logs_rx: broadcast::Receiver<String>,
) -> eyre::Result<()> {
    use babel::msg_handler;
    use std::sync::Arc;
    use std::time;

    let msg_handler = msg_handler::MsgHandler::new(cfg, time::Duration::from_secs(10), logs_rx)?;
    vsock::serve(run, cid, port, Arc::new(msg_handler)).await
}

#[cfg(not(target_os = "linux"))]
async fn serve(
    _run: RunFlag,
    _cfg: babel_api::config::Babel,
    _cid: u32,
    _port: u32,
    _logs_rx: broadcast::Receiver<String>,
) -> eyre::Result<()> {
    unimplemented!()
}
