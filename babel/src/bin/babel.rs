#[cfg(target_os = "linux")]
use babel::vsock;
use babel::{config, logging, run_flag::RunFlag};

const VSOCK_HOST_CID: u32 = 3;
const VSOCK_BABEL_PORT: u32 = 42;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    logging::setup_logging()?;

    let cfg = config::load(&babel::env::BABEL_CONFIG_PATH).await?;

    let run = RunFlag::run_until_ctrlc();
    serve(run, cfg, VSOCK_HOST_CID, VSOCK_BABEL_PORT).await?;

    Ok(())
}

#[cfg(target_os = "linux")]
async fn serve(
    run: RunFlag,
    cfg: babel_api::config::Babel,
    cid: u32,
    port: u32,
) -> eyre::Result<()> {
    use std::time::Duration;

    let msg_handler = babel::msg_handler::MsgHandler::new(cfg, Duration::from_secs(10))?;
    vsock::serve(run, cid, port, msg_handler).await
}

#[cfg(not(target_os = "linux"))]
async fn serve(
    _run: RunFlag,
    _cfg: babel_api::config::Babel,
    _cid: u32,
    _port: u32,
) -> eyre::Result<()> {
    unimplemented!()
}
