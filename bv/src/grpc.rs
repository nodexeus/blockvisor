use crate::nodes::Nodes;
use anyhow::{anyhow, bail, Result};
use pb::command_flow_client::CommandFlowClient;
use pb::metrics_service_client::MetricsServiceClient;
use pb::node_command::Command;
use std::{str::FromStr, sync::Arc};
use tokio::sync::{broadcast::Sender, Mutex};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tonic::codegen::InterceptedService;
use tonic::service::Interceptor;
use tonic::{Request, Status};
use tracing::{error, info};
use uuid::Uuid;

pub mod pb {
    // https://github.com/tokio-rs/prost/issues/661
    #![allow(clippy::derive_partial_eq_without_eq)]
    tonic::include_proto!("blockjoy.api.v1");
}

const STATUS_OK: i32 = 0;
const STATUS_ERROR: i32 = 1;

#[derive(Clone)]
pub struct AuthToken(pub String);

pub type CommandsClient =
    CommandFlowClient<InterceptedService<tonic::transport::Channel, AuthToken>>;
pub type MetricsClient =
    MetricsServiceClient<InterceptedService<tonic::transport::Channel, AuthToken>>;

impl Interceptor for AuthToken {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        let mut request = request;
        let val = format!("Bearer {}", base64::encode(self.0.clone()));
        request
            .metadata_mut()
            .insert("authorization", val.parse().unwrap());
        Ok(request)
    }
}

impl CommandsClient {
    pub fn with_auth(channel: tonic::transport::Channel, token: AuthToken) -> Self {
        CommandFlowClient::with_interceptor(channel, token)
    }
}

impl MetricsClient {
    pub fn with_auth(channel: tonic::transport::Channel, token: AuthToken) -> Self {
        MetricsServiceClient::with_interceptor(channel, token)
    }
}

pub async fn process_commands_stream(
    client: &mut CommandsClient,
    nodes: Arc<Mutex<Nodes>>,
    updates_tx: Sender<pb::InfoUpdate>,
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
            Some(pb::command::Type::Node(node_command)) => {
                let command_id = node_command.api_command_id.clone();
                match process_node_command(nodes.clone(), node_command).await {
                    Err(error) => {
                        error!("Error processing command: {error}");
                        create_info_update(command_id, STATUS_ERROR, Some(error.to_string()))
                    }
                    Ok(()) => create_info_update(command_id, STATUS_OK, None),
                }
            }
            Some(pb::command::Type::Host(host_command)) => {
                let msg = "Command type `Host` not supported".to_string();
                error!("Error processing command: {msg}");
                let command_id = host_command.api_command_id;
                create_info_update(command_id, STATUS_ERROR, Some(msg))
            }
            None => {
                let msg = "Command type is `None`".to_string();
                error!("Error processing command: {msg}");
                None
            }
        };

        if let Some(update) = update {
            updates_tx.send(update)?;
        }
    }

    Ok(())
}

fn create_info_update(
    command_id: String,
    code: i32,
    msg: Option<String>,
) -> Option<pb::InfoUpdate> {
    Some(pb::InfoUpdate {
        info: Some(pb::info_update::Info::Command(pb::CommandInfo {
            id: command_id,
            response: msg,
            exit_code: Some(code),
        })),
    })
}

async fn process_node_command(
    nodes: Arc<Mutex<Nodes>>,
    node_command: pb::NodeCommand,
) -> Result<()> {
    let node_id = Uuid::from_str(&node_command.id)?;
    match node_command.command {
        Some(cmd) => match cmd {
            Command::Create(args) => {
                let image = args
                    .image
                    .ok_or_else(|| anyhow!("Image not provided"))?
                    .url();
                nodes
                    .lock()
                    .await
                    .create(node_id, args.name, image, args.ip, args.gateway)
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
            Command::Upgrade(args) => {
                let image = args
                    .image
                    .ok_or_else(|| anyhow!("Image not provided"))?
                    .url();
                nodes.lock().await.upgrade(node_id, image).await?;
            }
            Command::Update(_) => unimplemented!(),
            Command::InfoGet(_) => unimplemented!(),
            Command::Generic(_) => unimplemented!(),
        },
        None => bail!("Node command is `None`"),
    };

    Ok(())
}

pub fn with_auth<T>(inner: T, auth_token: &str) -> Request<T> {
    let mut request = Request::new(inner);
    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {}", base64::encode(auth_token))
            .parse()
            .unwrap(),
    );
    request
}

impl pb::ContainerImage {
    pub fn url(&self) -> String {
        format!(
            "{}/{}/{}/os.img",
            self.protocol, self.node_type, self.node_version
        )
    }
}
