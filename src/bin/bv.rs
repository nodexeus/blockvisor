use anyhow::{bail, Result};
use blockvisord::{
    cli::{App, ChainCommand, Command, HostCommand, NodeCommand},
    config::Config,
    grpc::pb,
    hosts::{get_host_info, get_ip_address},
    nodes::Nodes,
    pretty_table::{PrettyTable, PrettyTableRow},
    server::{bv_pb, bv_pb::blockvisor_client::BlockvisorClient, BLOCKVISOR_SERVICE_URL},
    systemd::{ManagerProxy, UnitStartMode, UnitStopMode},
};
use clap::Parser;
use cli_table::print_stdout;
use petname::Petnames;
use std::str::FromStr;
use tokio::time::{sleep, Duration};
use tonic::transport::{Channel, Endpoint};
use uuid::Uuid;
use zbus::Connection;

const BLOCKVISOR_START_TIMEOUT: Duration = Duration::from_secs(5);

#[tokio::main]
async fn main() -> Result<()> {
    let args = App::parse();

    let conn = Connection::system().await?;
    let systemd_manager_proxy = ManagerProxy::new(&conn).await?;

    match args.command {
        Command::Init(cmd_args) => {
            println!("Configuring blockvisor");

            let ip = get_ip_address(&cmd_args.ifa);
            let host_info = get_host_info();

            let info = pb::HostInfo {
                id: None,
                name: host_info.name,
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                location: None,
                cpu_count: host_info.cpu_count,
                mem_size: host_info.mem_size,
                disk_size: host_info.disk_size,
                os: host_info.os,
                os_version: host_info.os_version,
                ip: Some(ip),
            };
            let create = pb::ProvisionHostRequest {
                request_id: Some(pb::Uuid {
                    value: Uuid::new_v4().to_string(),
                }),
                otp: cmd_args.otp,
                org_id: None,
                info: Some(info),
                validator_ips: vec![],
                status: pb::ConnectionStatus::Online.into(),
            };
            println!("{:?}", create);

            let mut client =
                pb::hosts_client::HostsClient::connect(cmd_args.blockjoy_api_url.clone()).await?;

            let host = client.provision(create).await?.into_inner();

            Config {
                id: host.host_id.unwrap().value.to_string(),
                token: host.token,
                blockjoy_api_url: cmd_args.blockjoy_api_url,
            }
            .save()
            .await?;

            if !Nodes::exists() {
                Nodes::default().save().await?;
            }
        }
        Command::Reset(cmd_args) => {
            let confirm = if cmd_args.yes {
                true
            } else {
                let mut input = String::new();
                println!(
                    "Are you sure you want to delete all nodes and remove the host from API? [y/N]:"
                );
                std::io::stdin().read_line(&mut input)?;
                input.trim().to_lowercase() == "y"
            };

            let mut service_client = BlockvisorClient::connect(BLOCKVISOR_SERVICE_URL).await?;

            if confirm {
                let nodes = service_client
                    .get_nodes(bv_pb::GetNodesRequest {})
                    .await?
                    .into_inner()
                    .nodes;
                for node in nodes {
                    let id = node.id;
                    println!("Deleting node with ID `{}`", &id);
                    service_client
                        .delete_node(bv_pb::DeleteNodeRequest { id })
                        .await?;
                }

                let config = Config::load().await?;
                let url = config.blockjoy_api_url;
                let host_id = config.id;

                let delete = pb::DeleteHostRequest {
                    request_id: Some(pb::Uuid {
                        value: Uuid::new_v4().to_string(),
                    }),
                    host_id: Some(pb::Uuid {
                        value: host_id.clone(),
                    }),
                };
                let mut client = pb::hosts_client::HostsClient::connect(url.clone()).await?;
                println!("Deleting host `{host_id}` from API `{url}`");
                client.delete(delete).await?;

                Config::remove().await?;
            }
        }
        Command::Start(_) => {
            if !Config::exists() {
                bail!("Host is not registered, please run `init` first");
            }

            if Endpoint::connect(&Endpoint::from_str(BLOCKVISOR_SERVICE_URL)?)
                .await
                .is_ok()
            {
                println!("Service already running");
                return Ok(());
            }

            // Enable the blockvisor service and babel socket to start on host bootup and start it.
            println!("Enabling blockvisor service to start on host boot.");
            systemd_manager_proxy
                .enable_unit_files(&["blockvisor.service", "babel-bus.socket"], false, false)
                .await?;

            println!("Starting babel socket unit");
            systemd_manager_proxy
                .start_unit("babel-bus.socket", UnitStartMode::Fail)
                .await?;
            println!("babel socket setup");

            println!("Starting blockvisor service");
            systemd_manager_proxy
                .start_unit("blockvisor.service", UnitStartMode::Fail)
                .await?;

            sleep(BLOCKVISOR_START_TIMEOUT).await;

            println!("blockvisor service started successfully");
        }
        Command::Stop(_) => {
            println!("Stopping blockvisor service");
            systemd_manager_proxy
                .stop_unit("blockvisor.service", UnitStopMode::Fail)
                .await?;
            println!("blockvisor service stopped successfully");

            println!("Shutting down babel socket unit");
            systemd_manager_proxy
                .stop_unit("babel-bus.socket", UnitStopMode::Fail)
                .await?;
            println!("babel socket terminated");
        }
        Command::Status(_) => {
            todo!()
        }
        Command::Host { command } => process_host_command(&command).await?,
        Command::Chain { command } => process_chain_command(&command).await?,
        Command::Node { command } => process_node_command(&command).await?,
    }

    Ok(())
}

async fn process_host_command(command: &HostCommand) -> Result<()> {
    match command {
        HostCommand::Info => {
            let info = get_host_info();
            println!("{:?}", info);
        }
    }

    Ok(())
}

#[allow(unreachable_code)]
async fn process_chain_command(command: &ChainCommand) -> Result<()> {
    match command {
        ChainCommand::List => todo!(),
        ChainCommand::Status { id: _ } => todo!(),
        ChainCommand::Sync { id: _ } => todo!(),
    }

    Ok(())
}

async fn process_node_command(command: &NodeCommand) -> Result<()> {
    let mut service_client = BlockvisorClient::connect(BLOCKVISOR_SERVICE_URL).await?;

    match command {
        NodeCommand::List { all, chain } => {
            let nodes = service_client
                .get_nodes(bv_pb::GetNodesRequest {})
                .await?
                .into_inner()
                .nodes;
            let mut nodes = nodes
                .iter()
                .filter(|c| {
                    chain
                        .as_ref()
                        .map(|chain| c.chain.contains(chain))
                        .unwrap_or(true)
                        && (*all || c.status == bv_pb::NodeStatus::Running as i32)
                })
                .peekable();
            if nodes.peek().is_some() {
                let mut table = vec![];
                for node in nodes.cloned() {
                    table.push(PrettyTableRow {
                        id: node.id,
                        name: node.name,
                        chain: node.chain,
                        status: bv_pb::NodeStatus::from_i32(node.status).unwrap(),
                        ip: node.ip,
                    })
                }
                print_stdout(table.to_pretty_table())?;
            } else {
                println!("No nodes found.");
            }
        }
        NodeCommand::Create { chain } => {
            let id = Uuid::new_v4();
            let name = Petnames::default().generate_one(3, "-");
            service_client
                .create_node(bv_pb::CreateNodeRequest {
                    id: id.to_string(),
                    name: name.clone(),
                    chain: chain.to_string(),
                })
                .await?;
            println!(
                "Created new node for `{}` chain with ID `{}` and name `{}`",
                chain, &id, &name
            );
        }
        NodeCommand::Start { id_or_name } => {
            let id = resolve_id_or_name(&mut service_client, id_or_name).await?;
            service_client
                .start_node(bv_pb::StartNodeRequest { id: id.to_string() })
                .await?;
            println!("Started node `{}`", id_or_name);
        }
        NodeCommand::Stop { id_or_name } => {
            let id = resolve_id_or_name(&mut service_client, id_or_name).await?;
            service_client
                .stop_node(bv_pb::StopNodeRequest { id: id.to_string() })
                .await?;
            println!("Stopped node `{}`", id_or_name);
        }
        NodeCommand::Delete { id_or_name } => {
            let id = resolve_id_or_name(&mut service_client, id_or_name).await?;
            service_client
                .delete_node(bv_pb::DeleteNodeRequest { id: id.to_string() })
                .await?;
            println!("Deleted node `{}`", id_or_name);
        }
        NodeCommand::Restart { id_or_name: _ } => todo!(),
        NodeCommand::Console { id_or_name: _ } => todo!(),
        NodeCommand::Logs { id_or_name: _ } => todo!(),
        NodeCommand::Status { id_or_name } => {
            let id = resolve_id_or_name(&mut service_client, id_or_name).await?;
            let status = service_client
                .get_node_status(bv_pb::GetNodeStatusRequest { id: id.to_string() })
                .await?
                .into_inner()
                .status;
            println!("{}", bv_pb::NodeStatus::from_i32(status).unwrap());
        }
    }
    Ok(())
}

async fn resolve_id_or_name(
    service_client: &mut BlockvisorClient<Channel>,
    id_or_name: &str,
) -> Result<Uuid> {
    let uuid = match Uuid::parse_str(id_or_name) {
        Ok(v) => v,
        Err(_) => {
            let uuid = service_client
                .get_node_id_for_name(bv_pb::GetNodeIdForNameRequest {
                    name: id_or_name.to_string(),
                })
                .await?
                .into_inner()
                .id;
            Uuid::parse_str(&uuid)?
        }
    };

    Ok(uuid)
}
