#[cfg(target_os = "linux")]
use babel::vsock;
use babel::{config, logging, run_flag::RunFlag};
use tokio::fs::DirBuilder;

const DATA_DRIVE_PATH: &str = "/dev/vdb";
const VSOCK_HOST_CID: u32 = 3;
const VSOCK_BABEL_PORT: u32 = 42;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    logging::setup_logging()?;

    let cfg = config::load(&babel::env::BABEL_CONFIG_PATH).await?;

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
