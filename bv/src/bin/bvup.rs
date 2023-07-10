use anyhow::{anyhow, Context, Result};
use blockvisord::config::SharedConfig;
use blockvisord::{
    config, config::Config, hosts::HostInfo, linux_platform::bv_root, self_updater,
    services::api::pb,
};
use bv_utils::cmd::ask_confirm;
use clap::{crate_version, ArgGroup, Parser};

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
#[clap(group(ArgGroup::new("input").required(true).args(&["provision_token", "skip_init"])))]
#[clap(group(ArgGroup::new("skip").args(&["skip_download", "skip_init"])))]
pub struct CmdArgs {
    /// Provision token
    pub provision_token: Option<String>,

    /// BlockJoy API url
    #[clap(long = "api", default_value = "https://api.prod.blockjoy.com")]
    pub blockjoy_api_url: String,

    /// BlockJoy MQTT url
    #[clap(long = "mqtt")]
    pub blockjoy_mqtt_url: Option<String>,

    /// Network interface name
    #[clap(long = "ifa", default_value = "bvbr0")]
    pub ifa: String,

    /// Network gateway IPv4 address
    #[clap(long = "ip-gateway")]
    pub ip_gateway: String,

    /// Network IP range from IPv4 value
    #[clap(long = "ip-range-from")]
    pub ip_range_from: String,

    /// Network IP range to IPv4 value
    #[clap(long = "ip-range-to")]
    pub ip_range_to: String,

    #[clap(long = "port")]
    pub blockvisor_port: Option<u16>,

    /// Skip provisioning and init phase
    #[clap(long = "skip-init")]
    pub skip_init: bool,

    /// Skip download and install phase
    #[clap(long = "skip-download")]
    pub skip_download: bool,

    /// Skip all [y/N] prompts.
    #[clap(short, long)]
    yes: bool,
}

pub fn get_ip_address(ifa_name: &str) -> Result<String> {
    let ifas = local_ip_address::list_afinet_netifas()?;
    let (_, ip) = ifas
        .into_iter()
        .find(|(name, ipaddr)| name == ifa_name && ipaddr.is_ipv4())
        .ok_or_else(|| anyhow!("interface {ifa_name} not found"))?;
    Ok(ip.to_string())
}

/// Simple host init tool. It provision host with PROVISION_TOKEN then download and install latest bv bundle.
#[tokio::main]
async fn main() -> Result<()> {
    let bv_root = bv_root();
    let cmd_args = CmdArgs::parse();
    let api_config = if !cmd_args.skip_init {
        println!("Provision and init blockvisor configuration");

        let ip = get_ip_address(&cmd_args.ifa).with_context(|| "failed to get ip address")?;
        let host_info = HostInfo::collect()?;
        let cpu_count = host_info
            .cpu_count
            .try_into()
            .with_context(|| "Cannot convert cpu count from usize to u64")?;
        let to_gb = |n| n as f64 / 1_000_000_000.0;

        println!("Hostname:            {:>16}", &host_info.name);
        println!("CPU count:           {:>16}", cpu_count);
        println!(
            "Total mem:           {:>16.3} GB",
            to_gb(host_info.memory_bytes)
        );
        println!(
            "Total disk:          {:>16.3} GB",
            to_gb(host_info.disk_space_bytes)
        );
        println!("OS:                  {:>16}", &host_info.os);
        println!("OS version:          {:>16}", &host_info.os_version);
        println!("API url:             {:>16}", &cmd_args.blockjoy_api_url);
        println!(
            "MQTT url:            {:>16}",
            cmd_args
                .blockjoy_mqtt_url
                .as_ref()
                .unwrap_or(&"(auto)".to_string())
        );
        println!("IP address:          {:>16}", &ip);
        println!("Gateway IP address:  {:>16}", &cmd_args.ip_gateway);
        println!("VM IP range from:    {:>16}", &cmd_args.ip_range_from);
        println!("VM IP range to:      {:>16}", &cmd_args.ip_range_to);

        let confirm = ask_confirm("Register the host with this configuration?", cmd_args.yes)?;
        if !confirm {
            return Ok(());
        }

        let create = pb::HostServiceCreateRequest {
            provision_token: cmd_args.provision_token.unwrap(),
            name: host_info.name,
            version: crate_version!().to_string(),
            cpu_count,
            mem_size_bytes: host_info.memory_bytes,
            disk_size_bytes: host_info.disk_space_bytes,
            os: host_info.os,
            os_version: host_info.os_version,
            ip_addr: ip,
            ip_gateway: cmd_args.ip_gateway,
            ip_range_from: cmd_args.ip_range_from,
            ip_range_to: cmd_args.ip_range_to,
            org_id: None,
        };

        let mut client =
            pb::host_service_client::HostServiceClient::connect(cmd_args.blockjoy_api_url.clone())
                .await?;

        let host = client.create(create).await?.into_inner();

        let api_config = Config {
            id: host
                .host
                .ok_or_else(|| anyhow!("No `host` in response"))?
                .id,
            token: host.token,
            refresh_token: host.refresh,
            blockjoy_api_url: cmd_args.blockjoy_api_url.clone(),
            blockjoy_mqtt_url: cmd_args.blockjoy_mqtt_url,
            update_check_interval_secs: None,
            blockvisor_port: cmd_args
                .blockvisor_port
                .unwrap_or_else(config::default_blockvisor_port),
        };
        api_config.save(&bv_root).await?;
        Some(api_config)
    } else {
        None
    };
    if !cmd_args.skip_download {
        println!("Download and install bv bundle");
        let config = match api_config {
            None => Config::load(&bv_root)
                .await
                .with_context(|| "Failed to load host configuration - need to init first")?,
            Some(value) => value,
        };
        let api_config = SharedConfig::new(config, bv_root.clone());

        let mut updater =
            self_updater::new(bv_utils::timer::SysTimer, &bv_root, &api_config).await?;
        let bundle_id = updater
            .get_latest()
            .await?
            .ok_or_else(|| anyhow!("No bv bundle found"))?;
        updater.download_and_install(bundle_id).await?;
    }
    Ok(())
}
