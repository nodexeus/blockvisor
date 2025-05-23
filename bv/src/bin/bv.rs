use blockvisord::{
    bv,
    bv_cli::{App, Command},
    bv_config::{Config, SharedConfig, CONFIG_PATH},
    internal_server,
    linux_platform::bv_root,
};
use bv_utils::cmd::run_cmd;
use clap::Parser;
use eyre::{bail, Result};
use tokio::time::{sleep, Duration};
use tonic::transport::Endpoint;

const BLOCKVISOR_STATUS_CHECK_INTERVAL: Duration = Duration::from_millis(500);
const BLOCKVISOR_START_TIMEOUT: Duration = Duration::from_secs(5);
const BLOCKVISOR_STOP_TIMEOUT: Duration = Duration::from_secs(5);
const BV_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const BV_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() -> Result<()> {
    let args = App::parse();

    let bv_root = bv_root();
    if !bv_root.join(CONFIG_PATH).exists() {
        bail!("Host is not registered, please run `bvup` first");
    }
    let config = SharedConfig::new(Config::load(&bv_root).await?, bv_root);
    let port = config.read().await.blockvisor_port;
    let bv_url = format!("http://127.0.0.1:{port}");

    match args.command {
        Command::Start(_) => {
            if let Ok(info) = service_info(bv_url.clone()).await {
                println!("blockvisor service started successfully: {info}");
                return Ok(());
            }

            run_cmd("systemctl", ["start", "blockvisor.service"]).await?;

            let start = std::time::Instant::now();
            loop {
                match service_info(bv_url.clone()).await {
                    Ok(info) => {
                        println!("blockvisor service started successfully: {info}");
                        break;
                    }
                    Err(err) => {
                        if start.elapsed() < BLOCKVISOR_START_TIMEOUT {
                            sleep(BLOCKVISOR_STATUS_CHECK_INTERVAL).await;
                        } else {
                            bail!("blockvisor service did not start: {err:#}")
                        }
                    }
                }
            }
        }
        Command::Stop(_) => {
            run_cmd("systemctl", ["stop", "blockvisor.service"]).await?;

            let start = std::time::Instant::now();
            while let Ok(info) = service_info(bv_url.clone()).await {
                if start.elapsed() < BLOCKVISOR_STOP_TIMEOUT {
                    sleep(BLOCKVISOR_STATUS_CHECK_INTERVAL).await;
                } else {
                    bail!("blockvisor service did not stop: {info}");
                }
            }
            println!("blockvisor service stopped successfully");
        }
        Command::Status(_) => {
            if let Ok(info) = service_info(bv_url).await {
                println!("Service running: {info}");
            } else {
                println!("Service stopped");
            }
        }
        Command::Host { command } => bv::process_host_command(bv_url, config, command).await?,
        Command::Protocol { command } => bv::process_protocol_command(config, command).await?,
        Command::Node { command } => bv::process_node_command(bv_url, command).await?,
        Command::Cluster { command } => bv::process_cluster_command(bv_url, command).await?,
    }

    Ok(())
}

async fn service_info(url: String) -> Result<String> {
    let mut client = internal_server::service_client::ServiceClient::connect(
        Endpoint::from_shared(url)?
            .connect_timeout(BV_CONNECT_TIMEOUT)
            .timeout(BV_REQUEST_TIMEOUT),
    )
    .await?;
    Ok(client.info(()).await?.into_inner())
}
