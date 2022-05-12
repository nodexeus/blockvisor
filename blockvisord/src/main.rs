use anyhow::{bail, Result};
use clap::Parser;
use cli::{App, Command};
use daemonize::Daemonize;
use hosts::Host;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::path::Path;
use sysinfo::{DiskExt, System, SystemExt};
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use crate::client::{APIClient, CommandStatusUpdate, HostCreateRequest};
use crate::containers::{DummyNode, NodeContainer};
use crate::hosts::HostConfig;

mod cli;
mod client;
mod containers;
mod hosts;

const CONFIG_FILE: &str = "/tmp/config.toml";

const PID_FILE: &str = "/tmp/blockvisor.pid";
const OUT_FILE: &str = "/tmp/blockvisor.out";
const ERR_FILE: &str = "/tmp/blockvisor.err";

fn main() -> Result<()> {
    let args = App::parse();
    println!("{:?}", args);
    let timeout = Duration::from_secs(10);

    match args.command {
        Command::Configure(cmd_args) => {
            println!("Configuring blockvisor");

            let network_interfaces = local_ip_address::list_afinet_netifas().unwrap();
            let (_, ip) = local_ip_address::find_ifa(network_interfaces, &cmd_args.ifa).unwrap();

            let sys = System::new_all();

            let create = HostCreateRequest {
                org_id: None,
                name: sys.host_name().unwrap(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                location: None,
                cpu_count: sys.physical_core_count().map(|x| x as i64),
                mem_size: Some(sys.total_memory() as i64),
                disk_size: Some(sys.disks()[0].total_space() as i64),
                os: sys.name(),
                os_version: sys.os_version(),
                ip_addr: ip.to_string(),
                val_ip_addrs: None,
            };
            println!("{:?}", create);

            let client = APIClient::new(&cmd_args.blockjoy_api_url, timeout)?;
            let rt = tokio::runtime::Runtime::new()?;
            let host = rt.block_on(client.register_host(&cmd_args.otp, &create))?;

            let config = HostConfig {
                data_dir: ".".to_string(),
                pool_dir: ".".to_string(),
                id: host.id.to_string(),
                token: host.token,
                blockjoy_api_url: cmd_args.blockjoy_api_url,
            };
            let config = toml::to_string(&config)?;
            fs::write(CONFIG_FILE, config)?;
        }
        Command::Start(cmd_args) => {
            if !Path::new(CONFIG_FILE).exists() {
                bail!("Error: not configured, please run `configure` first");
            }

            if cmd_args.daemonize {
                let stdout = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(OUT_FILE)?;
                let stderr = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(ERR_FILE)?;

                let daemonize = Daemonize::new()
                    .pid_file(PID_FILE)
                    .stdout(stdout)
                    .stderr(stderr);

                match daemonize.start() {
                    Ok(_) => println!("Starting blockvisor in background"),
                    Err(e) => {
                        bail!(e);
                    }
                }
            }

            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(work(cmd_args.daemonize))?;
        }
        Command::Stop(_) => {
            if Path::new(PID_FILE).exists() {
                fs::remove_file(PID_FILE)?
            }
        }
        _ => {}
    }

    Ok(())
}

async fn work(daemonized: bool) -> Result<()> {
    let config = fs::read_to_string(CONFIG_FILE)?;
    let config: HostConfig = toml::from_str(&config)?;
    let mut host = Host {
        containers: HashMap::new(),
        config,
    };

    loop {
        if !daemonized || Path::new(PID_FILE).exists() {
            println!("Reading config: {}", CONFIG_FILE);
            let config = host.config.clone();
            let timeout = Duration::from_secs(10);
            let client = APIClient::new(&config.blockjoy_api_url, timeout)?;

            println!("Getting pending commands for host: {}", &config.id);
            for command in client
                .get_pending_commands(&config.token, &config.id)
                .await?
            {
                let mut response = "Done".to_string();
                let mut exit_status = 0;

                println!("Processing command: {}", &command.cmd);
                match command.cmd.as_str() {
                    "create_node" => {
                        let id = Uuid::new_v4().to_string();
                        let node = DummyNode::create(&id).await?;
                        host.containers.insert(id, Box::new(node));
                    }
                    "kill_node" => {
                        if let Some(id) = command.sub_cmd {
                            if let Some(node) = host.containers.get_mut(&id) {
                                node.kill().await?;
                                host.containers.remove(&id);
                            } else {
                                println!("Cannot kill node: {} not present", id);
                                response = "Error".to_string();
                                exit_status = 1;
                            };
                        } else {
                            println!("Cannot kill node: id not provided");
                            response = "Error".to_string();
                            exit_status = 2;
                        }
                    }
                    _ => {}
                };

                let update = CommandStatusUpdate {
                    response,
                    exit_status,
                };

                client
                    .update_command_status(&config.token, &command.id, &update)
                    .await?;
            }

            sleep(Duration::from_secs(5)).await;
        } else {
            println!("Stopping blockvisor");
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::client::{APIClient, CommandStatusUpdate, HostCreateRequest};
    use chrono::{TimeZone, Utc};
    use httpmock::prelude::*;
    use serde_json::json;
    use std::time::Duration;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_register_host() {
        let server = MockServer::start();
        let org_id = Uuid::new_v4();

        let m = server.mock(|when, then| {
            when.method(POST)
                .path("/host_provisions/OTP/hosts")
                .header("Content-Type", "application/json")
                .json_body(json!({
                    "org_id": org_id,
                    "name": "some-host",
                    "version": "1.0",
                    "location": null,
                    "cpu_count": 4_i64,
                    "mem_size": 8_i64,
                    "disk_size": 100_i64,
                    "os": "ubuntu",
                    "os_version": "4.14.12",
                    "ip_addr": "192.168.0.1",
                    "val_ip_addrs": null,
                }));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!({
                    "id": "eb4e20fc-2b4a-4d0c-811f-48abcf12b89b",
                    "token": "secret_token",
                    "org_id": org_id,
                    "name": "some-host",
                    "version": "1.0",
                    "location": null,
                    "cpu_count": 4_i64,
                    "mem_size": 8_i64,
                    "disk_size": 100_i64,
                    "os": "ubuntu",
                    "os_version": "4.14.12",
                    "ip_addr": "192.168.0.1",
                    "val_ip_addrs": null,
                    "created_at": "2019-08-24T14:15:22Z",
                    "status": "online",
                    "validators": [],
                }));
        });

        let client = APIClient::new(&server.base_url(), Duration::from_secs(10)).unwrap();
        let otp = "OTP";
        let info = HostCreateRequest {
            org_id: Some(org_id),
            name: "some-host".to_string(),
            version: Some("1.0".to_string()),
            location: None,
            cpu_count: Some(4),
            mem_size: Some(8),
            disk_size: Some(100),
            os: Some("ubuntu".to_string()),
            os_version: Some("4.14.12".to_string()),
            ip_addr: "192.168.0.1".to_string(),
            val_ip_addrs: None,
        };
        let resp = client.register_host(otp, &info).await.unwrap();

        assert_eq!(resp.id.to_string(), "eb4e20fc-2b4a-4d0c-811f-48abcf12b89b");
        assert_eq!(resp.token, "secret_token");

        m.assert();
    }

    #[tokio::test]
    async fn test_get_pending_commands() {
        let server = MockServer::start();

        let token = "TOKEN";
        let host_id = "eb4e20fc-2b4a-4d0c-811f-48abcf12b89b";

        let m = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/hosts/{}/commands/pending", host_id))
                .header("Content-Type", "application/json")
                .header("authorization", format!("Bearer {}", token));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([
                  {
                    "id": "497f6eca-6276-4993-bfeb-53cbbbba6f08",
                    "host_id": host_id,
                    "cmd": "restart_miner",
                    "created_at": "2019-08-24T14:15:22Z",
                  }
                ]));
        });

        let client = APIClient::new(&server.base_url(), Duration::from_secs(10)).unwrap();
        let resp = client.get_pending_commands(token, host_id).await.unwrap();

        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].id, "497f6eca-6276-4993-bfeb-53cbbbba6f08");
        assert_eq!(resp[0].host_id, host_id);
        assert_eq!(resp[0].cmd, "restart_miner");
        assert_eq!(resp[0].sub_cmd, None);
        assert_eq!(resp[0].response, None);
        assert_eq!(resp[0].exit_status, None);
        assert_eq!(resp[0].created_at, Utc.ymd(2019, 8, 24).and_hms(14, 15, 22));
        assert_eq!(resp[0].completed_at, None);

        m.assert();
    }

    #[tokio::test]
    async fn test_update_command_status() {
        let server = MockServer::start();

        let token = "TOKEN";
        let command_id = "497f6eca-6276-4993-bfeb-53cbbbba6f08";

        let m = server.mock(|when, then| {
            when.method(PUT)
                .path(format!("/commands/{}/response", command_id))
                .header("Content-Type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .json_body(json!({
                    "response": "restarted",
                    "exit_status": 0_i32,
                }));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!(
                  {
                    "id": command_id,
                    "host_id": "eb4e20fc-2b4a-4d0c-811f-48abcf12b89b",
                    "cmd": "restart_miner",
                    "response": "restarted",
                    "exit_status": 0_i32,
                    "created_at": "2019-08-24T14:15:22Z",
                    "completed_at": "2020-08-24T14:15:22Z",
                  }
                ));
        });

        let client = APIClient::new(&server.base_url(), Duration::from_secs(10)).unwrap();
        let update = CommandStatusUpdate {
            response: "restarted".to_string(),
            exit_status: 0,
        };
        let resp = client
            .update_command_status(token, command_id, &update)
            .await
            .unwrap();

        assert_eq!(resp.id, command_id);
        assert_eq!(resp.host_id, "eb4e20fc-2b4a-4d0c-811f-48abcf12b89b");
        assert_eq!(resp.cmd, "restart_miner");
        assert_eq!(resp.sub_cmd, None);
        assert_eq!(resp.response, Some("restarted".to_string()));
        assert_eq!(resp.exit_status, Some(0_i32));
        assert_eq!(resp.created_at, Utc.ymd(2019, 8, 24).and_hms(14, 15, 22));
        assert_eq!(
            resp.completed_at,
            Some(Utc.ymd(2020, 8, 24).and_hms(14, 15, 22))
        );

        m.assert();
    }
}
