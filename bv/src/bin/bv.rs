use anyhow::{bail, Result};
use blockvisord::{
    cli::{App, ChainCommand, Command, HostCommand, NodeCommand},
    config::Config,
    grpc::pb,
    hosts::{get_host_info, get_ip_address},
    nodes::{CommonData, Nodes},
    pretty_table::{PrettyTable, PrettyTableRow},
    server::{
        bv_pb, bv_pb::blockvisor_client::BlockvisorClient, BlockvisorServer, BLOCKVISOR_SERVICE_URL,
    },
    utils::run_cmd,
};
use clap::Parser;
use cli_table::print_stdout;
use petname::Petnames;
use tokio::time::{sleep, Duration};
use tonic::transport::Channel;
use uuid::Uuid;

// TODO: use proper wait mechanism
const BLOCKVISOR_START_TIMEOUT: Duration = Duration::from_secs(5);
const BLOCKVISOR_STOP_TIMEOUT: Duration = Duration::from_secs(5);

#[tokio::main]
async fn main() -> Result<()> {
    let args = App::parse();

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
                ip_range_to: None,
                ip_range_from: None,
                ip_gateway: None,
            };
            let create = pb::ProvisionHostRequest {
                request_id: Some(Uuid::new_v4().to_string()),
                otp: cmd_args.otp,
                info: Some(info),
                status: pb::ConnectionStatus::Online.into(),
            };
            println!("{:?}", create);

            let mut client =
                pb::hosts_client::HostsClient::connect(cmd_args.blockjoy_api_url.clone()).await?;

            let host = client.provision(create).await?.into_inner();

            Config {
                id: host.host_id,
                token: host.token,
                blockjoy_api_url: cmd_args.blockjoy_api_url,
            }
            .save()
            .await?;

            if !Nodes::exists() {
                let nodes_data = CommonData { machine_index: 0 };
                Nodes::new(nodes_data).save().await?;
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
                    request_id: Some(Uuid::new_v4().to_string()),
                    host_id: host_id.clone(),
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

            if BlockvisorServer::is_running().await {
                println!("Service already running");
                return Ok(());
            }

            run_cmd("systemctl", &["start", "blockvisor.service"]).await?;
            sleep(BLOCKVISOR_START_TIMEOUT).await;
            println!("blockvisor service started successfully");
        }
        Command::Stop(_) => {
            run_cmd("systemctl", &["stop", "blockvisor.service"]).await?;
            sleep(BLOCKVISOR_STOP_TIMEOUT).await;
            println!("blockvisor service stopped successfully");
        }
        Command::Status(_) => {
            if !Config::exists() {
                bail!("Host is not registered, please run `init` first");
            }

            if BlockvisorServer::is_running().await {
                println!("Service running");
            } else {
                println!("Service stopped");
            }
        }
        Command::Host { command } => process_host_command(command).await?,
        Command::Chain { command } => process_chain_command(command).await?,
        Command::Node { command } => process_node_command(command).await?,
    }

    Ok(())
}

async fn process_host_command(command: HostCommand) -> Result<()> {
    match command {
        HostCommand::Info => {
            let info = get_host_info();
            println!("{:?}", info);
        }
    }

    Ok(())
}

#[allow(unreachable_code)]
async fn process_chain_command(command: ChainCommand) -> Result<()> {
    match command {
        ChainCommand::List => todo!(),
        ChainCommand::Status { id: _ } => todo!(),
        ChainCommand::Sync { id: _ } => todo!(),
    }

    Ok(())
}

async fn process_node_command(command: NodeCommand) -> Result<()> {
    let mut service_client = BlockvisorClient::connect(BLOCKVISOR_SERVICE_URL).await?;

    match command {
        NodeCommand::List { running, image } => {
            let nodes = service_client
                .get_nodes(bv_pb::GetNodesRequest {})
                .await?
                .into_inner()
                .nodes;
            let mut nodes = nodes
                .iter()
                .filter(|n| {
                    image
                        .as_ref()
                        .map(|image| n.image.contains(image))
                        .unwrap_or(true)
                        && (!running || (n.status == bv_pb::NodeStatus::Running as i32))
                })
                .peekable();
            if nodes.peek().is_some() {
                let mut table = vec![];
                for node in nodes.cloned() {
                    table.push(PrettyTableRow {
                        id: node.id,
                        name: node.name,
                        image: node.image,
                        status: bv_pb::NodeStatus::from_i32(node.status).unwrap(),
                        ip: node.ip,
                    })
                }
                print_stdout(table.to_pretty_table())?;
            } else {
                println!("No nodes found.");
            }
        }
        NodeCommand::Create { image, ip, gateway } => {
            let id = Uuid::new_v4();
            let name = Petnames::default().generate_one(3, "_");
            // TODO: this configurations is useful for testing on CI machine
            let gateway = gateway.unwrap_or_else(|| "216.18.214.193".to_string());
            let ip = ip.unwrap_or_else(|| "216.18.214.195".to_string());
            service_client
                .create_node(bv_pb::CreateNodeRequest {
                    id: id.to_string(),
                    name: name.clone(),
                    image: image.to_string(),
                    ip,
                    gateway,
                })
                .await?;
            println!(
                "Created new node from `{}` image with ID `{}` and name `{}`",
                image, id, &name
            );
        }
        NodeCommand::Start { id_or_names } => {
            for id_or_name in id_or_names {
                let id = resolve_id_or_name(&mut service_client, &id_or_name)
                    .await?
                    .to_string();
                service_client
                    .start_node(bv_pb::StartNodeRequest { id })
                    .await?;
                println!("Started node `{}`", id_or_name);
            }
        }
        NodeCommand::Stop { id_or_names } => {
            for id_or_name in id_or_names {
                let id = resolve_id_or_name(&mut service_client, &id_or_name)
                    .await?
                    .to_string();
                service_client
                    .stop_node(bv_pb::StopNodeRequest { id })
                    .await?;
                println!("Stopped node `{}`", id_or_name);
            }
        }
        NodeCommand::Delete { id_or_names } => {
            for id_or_name in id_or_names {
                let id = resolve_id_or_name(&mut service_client, &id_or_name)
                    .await?
                    .to_string();
                service_client
                    .delete_node(bv_pb::DeleteNodeRequest { id })
                    .await?;
                println!("Deleted node `{id_or_name}`");
            }
        }
        NodeCommand::Restart { id_or_names: _ } => todo!(),
        NodeCommand::Console { id_or_name: _ } => todo!(),
        NodeCommand::Logs { id_or_name: _ } => todo!(),
        NodeCommand::Status { id_or_names } => {
            for id_or_name in id_or_names {
                let id = resolve_id_or_name(&mut service_client, &id_or_name)
                    .await?
                    .to_string();
                let status = service_client
                    .get_node_status(bv_pb::GetNodeStatusRequest { id })
                    .await?;
                let status = status.into_inner().status;
                match bv_pb::NodeStatus::from_i32(status) {
                    Some(status) => println!("{status}"),
                    None => eprintln!("Invalid status {status}"),
                }
            }
        }
        NodeCommand::Capabilities { id_or_name } => {
            let node_id = resolve_id_or_name(&mut service_client, &id_or_name).await?;
            let caps = list_capabilities(&mut service_client, node_id).await?;
            print!("{caps}");
        }
        NodeCommand::Run {
            id_or_name,
            method,
            payload,
        } => {
            let node_id = resolve_id_or_name(&mut service_client, &id_or_name).await?;
            let req = bv_pb::BlockchainRequest {
                method,
                node_id: node_id.to_string(),
                payload,
            };
            match service_client.blockchain(req).await {
                Ok(result) => println!("{}", result.into_inner().value),
                Err(e) => {
                    if e.message().contains("not found") {
                        let msg = "Method not found. Options are:";
                        let caps = list_capabilities(&mut service_client, node_id).await?;
                        anyhow::bail!("{msg}\n{caps}");
                    }
                    return Err(anyhow::Error::from(e));
                }
            }
        }
    }

    Ok(())
}

async fn list_capabilities(
    client: &mut BlockvisorClient<Channel>,
    node_id: uuid::Uuid,
) -> Result<String> {
    let req = bv_pb::ListCapabilitiesRequest {
        node_id: node_id.to_string(),
    };
    let caps = client
        .list_capabilities(req)
        .await?
        .into_inner()
        .capabilities
        .into_iter()
        .reduce(|msg, cap| msg + "\n" + &cap)
        .unwrap_or_default();
    Ok(caps)
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
