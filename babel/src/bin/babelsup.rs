use async_trait::async_trait;
use babel::{babelsup_service, utils};
use babel::{config, logging, run_flag::RunFlag, supervisor};
use eyre::Context;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, watch};
use tonic::transport::Server;

const VSOCK_HOST_CID: u32 = 3;
const VSOCK_SUPERVISOR_PORT: u32 = 41;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    logging::setup_logging()?;

    let cfg = config::load(&babel::env::BABEL_CONFIG_PATH).await?;

    let run = RunFlag::run_until_ctrlc();

    let (babel_change_tx, babel_change_rx) =
        watch::channel(utils::file_checksum(&babel::env::BABEL_BIN_PATH).await.ok());

    let supervisor = supervisor::Supervisor::<SysTimer>::new(
        run.clone(),
        cfg.supervisor,
        babel::env::BABEL_BIN_PATH.clone(),
        babel_change_rx,
    );
    let logs_rx = supervisor.get_logs_rx();
    let (supervisor, sup_server) =
        tokio::join!(supervisor.run(), serve(run, logs_rx, babel_change_tx),);
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
    mut run: RunFlag,
    logs_rx: broadcast::Receiver<String>,
    babel_change_tx: watch::Sender<Option<u32>>,
) -> eyre::Result<()> {
    let babelsup_service = babel::babelsup_service::BabelSupService::new(
        logs_rx,
        babel_change_tx,
        babel::env::BABEL_BIN_PATH.clone(),
    );
    let listener = tokio_vsock::VsockListener::bind(VSOCK_HOST_CID, VSOCK_SUPERVISOR_PORT)
        .with_context(|| "failed to bind to vsock")?;

    Server::builder()
        .max_concurrent_streams(2)
        .add_service(babelsup_service::pb::babel_sup_server::BabelSupServer::new(
            babelsup_service,
        ))
        .serve_with_incoming_shutdown(listener.incoming(), run.wait())
        .await?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
async fn serve(
    _run: RunFlag,
    _logs_rx: broadcast::Receiver<String>,
    _babel_change_tx: watch::Sender<Option<u32>>,
) -> eyre::Result<()> {
    unimplemented!()
}
