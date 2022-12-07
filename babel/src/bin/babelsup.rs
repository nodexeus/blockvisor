use async_trait::async_trait;
#[cfg(target_os = "linux")]
use babel::vsock;
use babel::{config, logging, run_flag::RunFlag, supervisor};
use babel_api::config::Entrypoint;
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

const VSOCK_HOST_CID: u32 = 3;
const VSOCK_SUPERVISOR_PORT: u32 = 41;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    logging::setup_logging()?;

    let cfg_path = Path::new(config::CONFIG_PATH);
    let cfg = config::load(cfg_path).await?;

    let run = RunFlag::run_until_ctrlc();
    let mut supervisor_cfg = cfg.supervisor.clone();
    supervisor_cfg.entry_point.insert(
        0,
        Entrypoint {
            command: config::BABEL_BIN_PATH.to_string(),
            args: vec![],
        },
    );

    let supervisor = supervisor::Supervisor::<SysTimer>::new(run.clone(), supervisor_cfg);
    let rx = supervisor.get_logs_rx();
    let (supervisor, sup_server) = tokio::join!(
        supervisor.run(),
        serve(run, VSOCK_HOST_CID, VSOCK_SUPERVISOR_PORT, rx,),
    );
    supervisor?;
    sup_server
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
    cid: u32,
    port: u32,
    logs_rx: broadcast::Receiver<String>,
) -> eyre::Result<()> {
    use std::sync::Arc;

    let sup_handler = babel::sup_handler::SupHandler::new(logs_rx)?;
    vsock::serve(run, cid, port, Arc::new(sup_handler)).await
}

#[cfg(not(target_os = "linux"))]
async fn serve(
    _run: RunFlag,
    _cid: u32,
    _port: u32,
    _logs_rx: broadcast::Receiver<String>,
) -> eyre::Result<()> {
    unimplemented!()
}
