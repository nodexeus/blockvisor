use async_trait::async_trait;
#[cfg(target_os = "linux")]
use babel::vsock;
use babel::{config, logging, run_flag::RunFlag, supervisor};
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc};

const VSOCK_HOST_CID: u32 = 3;
const VSOCK_SUPERVISOR_PORT: u32 = 41;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    logging::setup_logging()?;

    let cfg = config::load(&babel::env::BABEL_CONFIG_PATH).await?;

    let run = RunFlag::run_until_ctrlc();

    let supervisor = supervisor::Supervisor::<SysTimer>::new(
        run.clone(),
        cfg.supervisor,
        babel::env::BABEL_BIN_PATH.clone(),
    );
    let rx = supervisor.get_logs_rx();
    let tx = supervisor.get_babel_restart_tx();
    let (supervisor, sup_server) = tokio::join!(
        supervisor.run(),
        serve(run, VSOCK_HOST_CID, VSOCK_SUPERVISOR_PORT, rx, tx),
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
    babel_restart_tx: mpsc::Sender<()>,
) -> eyre::Result<()> {
    let sup_handler = babel::sup_handler::SupHandler::new(
        logs_rx,
        babel_restart_tx,
        babel::env::BABEL_BIN_PATH.clone(),
    )?;
    vsock::serve(run, cid, port, sup_handler).await
}

#[cfg(not(target_os = "linux"))]
async fn serve(
    _run: RunFlag,
    _cid: u32,
    _port: u32,
    _logs_rx: broadcast::Receiver<String>,
    _babel_restart_tx: mpsc::Sender<()>,
) -> eyre::Result<()> {
    unimplemented!()
}
