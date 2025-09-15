use babel::{babel::BABEL_CONFIG_PATH, chroot_platform::UdsConnector, jobs};
use bv_utils::{logging::setup_logging, run_flag::RunFlag};
use eyre::{anyhow, bail};
use libc::setsid;
use std::env;
use tracing::{error, info, warn};

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> eyre::Result<()> {
    setup_logging();
    info!(
        "Starting {} {} ...",
        env!("CARGO_BIN_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    // use `setsid()` to make sure job runner won't be killed when babel is stopped with SIGINT
    unsafe {
        let _ = setsid();
    }
    match get_job_name() {
        Ok(job_name) => {
            // Create RunFlag with both SIGINT and SIGTERM handlers
            let mut run_flag = RunFlag::default();
            setup_signal_handlers(&mut run_flag);

            let res = babel::job_runner::run_job(
                run_flag,
                job_name,
                &jobs::JOBS_DIR,
                babel::load_config(&BABEL_CONFIG_PATH).await?,
                UdsConnector,
            )
            .await;
            if let Err(err) = &res {
                warn!("JobRunner failed with: {err:#}");
            }
            res
        }
        Err(err) => {
            error!("{err:#}");
            Err(err)
        }
    }
}

fn setup_signal_handlers(run_flag: &mut RunFlag) {
    // Handle SIGINT (Ctrl+C)
    let mut ctrlc_run = run_flag.clone();
    ctrlc::set_handler(move || {
        info!("Received SIGINT, shutting down...");
        ctrlc_run.stop();
    })
    .expect("Error setting Ctrl-C handler");

    // Handle SIGTERM
    let mut sigterm_run = run_flag.clone();
    tokio::spawn(async move {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::terminate()).expect("Error setting SIGTERM handler");
        sigterm.recv().await;
        info!("Received SIGTERM, shutting down...");
        sigterm_run.stop();
    });
}

fn get_job_name() -> eyre::Result<String> {
    let mut args = env::args();
    let job_name = args
        .nth(1)
        .ok_or_else(|| anyhow!("Missing argument! Expected unique job name."))?;
    if args.count() != 0 {
        bail!("Invalid number of arguments! Expected only one argument: unique job name.");
    }
    Ok(job_name)
}
