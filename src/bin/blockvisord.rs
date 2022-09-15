use anyhow::{bail, Result};
use blockvisord::{
    config::Config,
    grpc::{self, pb::node_command::Command},
    logging::setup_logging,
    nodes::Nodes,
    server::{bv_pb, BlockvisorServer, BLOCKVISOR_SERVICE_PORT},
};
use std::{net::ToSocketAddrs, str::FromStr, sync::Arc};
use tokio::{
    sync::{broadcast::Sender, Mutex},
    time::{sleep, Duration},
};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tonic::transport::{Channel, Endpoint, Server};
use tracing::{error, info};
use uuid::Uuid;

const STATUS_OK: i32 = 0;
const STATUS_ERROR: i32 = 1;

#[tokio::main]
async fn main() -> Result<()> {
    setup_logging()?;
    info!("Starting...");

    let config = Config::load().await?;
    let nodes = Nodes::load().await?;
    let updates_tx = nodes.get_updates_sender().await?.clone();
    let nodes = Arc::new(Mutex::new(nodes));

    let url = format!("0.0.0.0:{BLOCKVISOR_SERVICE_PORT}");
    let server = BlockvisorServer {
        nodes: nodes.clone(),
    };
    let internal_api_server_future = create_server(url, server);

    let token = grpc::AuthToken(config.token.to_owned());
    let endpoint = Endpoint::from_str(&config.blockjoy_api_url)?;
    let external_api_client_future = async {
        let channel = wait_for_channel(&endpoint).await;

        info!("Creating gRPC client...");
        let mut client = grpc::Client::with_auth(channel, token);

        loop {
            if let Err(e) =
                process_commands_stream(&mut client, nodes.clone(), updates_tx.clone()).await
            {
                error!("Error processing pending commands: {:?}", e);
            }
            sleep(Duration::from_secs(5)).await;
        }
    };

    tokio::select! {
        _ = internal_api_server_future => {},
        _ = external_api_client_future => {}
    }

    info!("Stopping...");
    Ok(())
}

async fn create_server(url: String, server: BlockvisorServer) -> Result<()> {
    Server::builder()
        .max_concurrent_streams(1)
        .add_service(bv_pb::blockvisor_server::BlockvisorServer::new(server))
        .serve(url.to_socket_addrs()?.next().unwrap())
        .await?;

    Ok(())
}

async fn wait_for_channel(endpoint: &Endpoint) -> Channel {
    loop {
        match Endpoint::connect(endpoint).await {
            Ok(channel) => return channel,
            Err(e) => {
                error!("Error connecting to endpoint: {:?}", e);
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn process_commands_stream(
    client: &mut grpc::Client,
    nodes: Arc<Mutex<Nodes>>,
    updates_tx: Sender<grpc::pb::InfoUpdate>,
) -> Result<()> {
    info!("Processing pending commands");
    let updates_rx = updates_tx.subscribe();

    let updates_stream = BroadcastStream::new(updates_rx).filter_map(|item| item.ok());

    let response = client.commands(updates_stream).await?;
    let mut commands_stream = response.into_inner();

    info!("Getting pending commands from stream...");
    while let Some(received) = commands_stream.next().await {
        info!("received: {received:?}");
        let received = received?;

        let update = match received.r#type {
            Some(grpc::pb::command::Type::Node(node_command)) => {
                let command_id = node_command.meta.clone().unwrap().api_command_id.unwrap();
                match process_node_command(nodes.clone(), node_command).await {
                    Err(error) => {
                        error!("Error processing command: {error}");
                        create_info_update(Some(command_id), STATUS_ERROR, Some(error.to_string()))
                    }
                    Ok(()) => create_info_update(Some(command_id), STATUS_OK, None),
                }
            }
            Some(grpc::pb::command::Type::Host(host_command)) => {
                let msg = "Command type `Host` not supported".to_string();
                error!("Error processing command: {msg}");
                let command_id = host_command.meta.clone().unwrap().api_command_id.unwrap();
                create_info_update(Some(command_id), STATUS_ERROR, Some(msg))
            }
            None => {
                let msg = "Command type is `None`".to_string();
                error!("Error processing command: {msg}");
                create_info_update(None, STATUS_ERROR, Some(msg))
            }
        };

        updates_tx.send(update)?;
    }

    Ok(())
}

fn create_info_update(
    command_id: Option<grpc::pb::Uuid>,
    code: i32,
    msg: Option<String>,
) -> grpc::pb::InfoUpdate {
    grpc::pb::InfoUpdate {
        info: Some(grpc::pb::info_update::Info::Command(
            grpc::pb::CommandInfo {
                id: command_id,
                response: msg,
                exit_code: Some(code),
            },
        )),
    }
}

async fn process_node_command(
    nodes: Arc<Mutex<Nodes>>,
    node_command: grpc::pb::NodeCommand,
) -> Result<()> {
    let node_id = node_command.id.unwrap().value;
    let node_id = Uuid::from_str(&node_id)?;
    match node_command.command {
        Some(cmd) => match cmd {
            Command::Create(args) => {
                nodes
                    .lock()
                    .await
                    .create(node_id, args.name, args.image.unwrap().url)
                    .await?;
            }
            Command::Delete(_) => {
                nodes.lock().await.delete(node_id).await?;
            }
            Command::Start(_) => {
                nodes.lock().await.start(node_id).await?;
            }
            Command::Stop(_) => {
                nodes.lock().await.stop(node_id).await?;
            }
            Command::Restart(_) => {
                nodes.lock().await.stop(node_id).await?;
                nodes.lock().await.start(node_id).await?;
            }
            Command::Upgrade(_) => unimplemented!(),
            Command::Update(_) => unimplemented!(),
            Command::InfoGet(_) => unimplemented!(),
            Command::Generic(_) => unimplemented!(),
        },
        None => bail!("Node command is `None`"),
    };

    Ok(())
}
