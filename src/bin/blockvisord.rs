use anyhow::Result;
use blockvisord::{
    config::Config,
    dbus::NodeProxy,
    grpc::{self, pb::node_command::Command},
    logging::setup_logging,
    nodes::Nodes,
};
use std::str::FromStr;
use tokio::{
    sync::broadcast::Sender,
    time::{sleep, Duration},
};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tonic::transport::{Channel, Endpoint};
use tracing::{error, info};
use uuid::Uuid;
use zbus::{Connection, ConnectionBuilder, ProxyDefault};

const STATUS_OK: i32 = 0;
const STATUS_ERROR: i32 = 1;

#[allow(unreachable_code)]
#[tokio::main]
async fn main() -> Result<()> {
    setup_logging()?;
    info!("Starting...");

    let config = Config::load().await?;
    let nodes = Nodes::load().await?;
    let updates_tx = nodes.get_updates_sender().await?.clone();

    let _conn = ConnectionBuilder::system()?
        .name(NodeProxy::DESTINATION)?
        .serve_at(NodeProxy::PATH, nodes)?
        .build()
        .await?;

    let token = grpc::AuthToken(config.token.to_owned());
    let endpoint = Endpoint::from_str(&config.blockjoy_api_url)?;
    let channel = wait_for_channel(&endpoint).await;

    info!("Creating gRPC client...");
    let mut client = grpc::Client::with_auth(channel, token);

    loop {
        if let Err(e) = process_commands_stream(&mut client, updates_tx.clone()).await {
            error!("Error processing pending commands: {:?}", e);
        }
        sleep(Duration::from_secs(5)).await;
    }

    info!("Stopping...");
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
    updates_tx: Sender<grpc::pb::InfoUpdate>,
) -> Result<()> {
    info!("Processing pending commands");
    let updates_rx = updates_tx.subscribe();

    let updates_stream = BroadcastStream::new(updates_rx).filter_map(|item| item.ok());

    let response = client.commands(updates_stream).await?;
    let mut commands_stream = response.into_inner();

    let conn = Connection::system().await?;
    let node_proxy = NodeProxy::new(&conn).await?;

    info!("Getting pending commands from stream...");
    while let Some(received) = commands_stream.next().await {
        info!("received: {received:?}");
        let received = received?;

        let update = match received.r#type {
            Some(grpc::pb::command::Type::Node(node_command)) => {
                let command_id = node_command.meta.clone().unwrap().api_command_id.unwrap();
                match process_node_command(&node_proxy, node_command).await {
                    Err(error) => {
                        error!("Error processing command: {error}");
                        create_info_update(command_id, STATUS_ERROR, Some(error.to_string()))
                    }
                    Ok(()) => create_info_update(command_id, STATUS_OK, None),
                }
            }
            Some(grpc::pb::command::Type::Host(host_command)) => {
                let msg = "Command type `Host` not supported".to_string();
                error!("Error processing command: {msg}");
                let command_id = host_command.meta.clone().unwrap().api_command_id.unwrap();
                create_info_update(command_id, STATUS_ERROR, Some(msg))
            }
            None => unreachable!(),
        };

        updates_tx.send(update)?;
    }

    Ok(())
}

fn create_info_update(
    command_id: grpc::pb::Uuid,
    code: i32,
    msg: Option<String>,
) -> grpc::pb::InfoUpdate {
    grpc::pb::InfoUpdate {
        info: Some(grpc::pb::info_update::Info::Command(
            grpc::pb::CommandInfo {
                id: Some(command_id),
                response: msg,
                exit_code: Some(code),
            },
        )),
    }
}

async fn process_node_command(
    node_proxy: &NodeProxy<'_>,
    node_command: grpc::pb::NodeCommand,
) -> Result<()> {
    let node_id = node_command.id.unwrap().value;
    let node_id = Uuid::from_str(&node_id)?;
    match node_command.command {
        Some(cmd) => match cmd {
            Command::Create(args) => {
                node_proxy
                    .create(&node_id, &args.name, &args.image.unwrap().url)
                    .await?;
            }
            Command::Delete(_) => {
                node_proxy.delete(&node_id).await?;
            }
            Command::Start(_) => {
                node_proxy.start(&node_id).await?;
            }
            Command::Stop(_) => {
                node_proxy.stop(&node_id).await?;
            }
            Command::Restart(_) => {
                node_proxy.stop(&node_id).await?;
                node_proxy.start(&node_id).await?;
            }
            Command::Upgrade(_) => unimplemented!(),
            Command::Update(_) => unimplemented!(),
            Command::InfoGet(_) => unimplemented!(),
            Command::Generic(_) => unimplemented!(),
        },
        None => unreachable!(),
    };

    Ok(())
}
