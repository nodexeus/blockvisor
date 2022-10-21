use crate::config;
use eyre::{Context, ContextCompat};
use futures::TryStreamExt;
use serde_json::json;
use std::future::ready;
use tokio::fs;
use tracing::trace;
use zbus::{Address, ConnectionBuilder, MessageStream, MessageType, VsockAddress};

const VSOCK_HOST_CID: u32 = 2;
const VSOCK_PORT: u32 = 42;

/// This message will be sent to blockvisor on startup once we get rid of dbus.
#[derive(serde::Serialize)]
struct Start {
    start_msg: String,
}

impl Start {
    fn _new() -> Self {
        Self {
            start_msg: "Oh lawd we be startin'".to_string(),
        }
    }
}

#[derive(serde::Serialize)]
struct Stop {
    stop_msg: String,
}

impl Stop {
    fn _new() -> Self {
        Self {
            stop_msg: "Oh lawd we be stoppin'".to_string(),
        }
    }
}

pub async fn serve(cfg: config::Babel) -> eyre::Result<()> {
    let id = fs::read_to_string("/proc/cmdline")
        .await?
        .split(' ')
        .find_map(|x| x.strip_prefix("blockvisor.node=").map(|id| id.to_string()))
        .with_context(|| "Node UUID not passed through kernel cmdline".to_string())?;
    let name = format!("com.BlockJoy.Babel.Node{}", id);

    let addr = Address::Vsock(VsockAddress::new(VSOCK_HOST_CID, VSOCK_PORT));
    trace!("creating DBus connection..");
    let conn = ConnectionBuilder::address(addr)?
        .name(name)?
        .build()
        .await?;
    trace!("D-bus connection created: {:?}", conn);
    let client = reqwest::Client::new();

    let mut stream = MessageStream::from(&conn).try_filter(|msg| {
        trace!("got message: {:?}", msg);
        ready(
            msg.header().is_ok()
                && msg.member().is_some()
                && msg.body::<String>().is_ok()
                && msg.message_type() == MessageType::MethodCall
                && msg.interface().as_ref().map(|i| i.as_str()) == Some("com.BlockJoy.Babel")
                && msg.path().as_ref().map(|i| i.as_str()) == Some("/com/BlockJoy/Babel"),
        )
    });
    while let Some(msg) = stream.try_next().await? {
        trace!("received Babel method message: {:?}", msg);
        // SAFETY: The filter we setup above already ensures header and member are set.
        let method_name = msg.member().unwrap();
        let body: String = msg.body().unwrap();

        let req: BabelRequest = serde_json::from_str(&body).unwrap(); // TODO
        let resp = req.handle(&client, &cfg).await.unwrap_or_else(|e| {
            tracing::error!("Failed to handle request: {e}");
            BabelResponse::Error(e.to_string())
        });
        if let Err(e) = conn
            .reply(&msg, &serde_json::to_string(&resp).unwrap())
            .await
        {
            tracing::error!("failed to reply to method call `{method_name}`: {e}");
        }
    }

    Ok(())
}

/// Each request that comes over the VSock to babel must be a piece of JSON that can be
/// deserialized into this struct.
#[derive(Debug, serde::Deserialize)]
enum BabelRequest {
    /// List the endpoints that are available for the current blockchain. These are extracted from
    /// the config, and just sent back as strings for now.
    ListCapabilities,
    /// Send a request to the current blockchain. We can identify the way to do this from the
    /// config and forward the provided parameters.
    BlockchainCommand(BlockchainCommand),
}

impl BabelRequest {
    async fn handle(
        self,
        client: &reqwest::Client,
        cfg: &config::Babel,
    ) -> eyre::Result<BabelResponse> {
        use BabelResponse::*;
        let resp = match self {
            Self::ListCapabilities => ListCapabilities(Self::handle_list_caps(cfg)),
            Self::BlockchainCommand(cmd) => BlockchainResponse(cmd.handle(client, cfg).await?),
        };
        Ok(resp)
    }

    /// List the capabilities that the current blockchain node supports.
    fn handle_list_caps(cfg: &config::Babel) -> Vec<String> {
        cfg.methods
            .keys()
            .map(|method| method.to_string())
            .collect()
    }
}

#[derive(Debug, serde::Deserialize)]
struct BlockchainCommand {
    name: String,
}

impl BlockchainCommand {
    async fn handle(
        self,
        client: &reqwest::Client,
        cfg: &config::Babel,
    ) -> eyre::Result<BlockchainResponse> {
        use config::Method::*;

        let method = cfg
            .methods
            .get(&self.name)
            .ok_or_else(|| eyre::eyre!("No method named {}", self.name))?;
        match method {
            Jrpc { method, .. } => Self::handle_jrpc(method, client, cfg).await,
            Rest {
                method, response, ..
            } => Self::handle_rest(method, response, client, cfg).await,
            Sh { body, response, .. } => Self::handle_sh(body, response).await,
        }
    }

    async fn handle_jrpc(
        method: &str,
        client: &reqwest::Client,
        cfg: &config::Babel,
    ) -> eyre::Result<BlockchainResponse> {
        let url = cfg.config.api_host.as_deref().ok_or_else(|| {
            eyre::eyre!(
                "`{method}` specified as a JSON-RPC method in the config but no host specified"
            )
        })?;
        let value = client
            .post(url)
            .json(&json!({ "jsonrpc": "2.0", "id": "id", "method": method }))
            .send()
            .await
            .wrap_err(format!("failed to call {url}"))?
            .text()
            .await
            .wrap_err(format!("failed to call {url}"))?;
        let resp = BlockchainResponse { value };
        Ok(resp)
    }

    async fn handle_rest(
        method: &str,
        response_config: &config::RestResponse,
        client: &reqwest::Client,
        cfg: &config::Babel,
    ) -> eyre::Result<BlockchainResponse> {
        use config::MethodResponseFormat::*;

        let host = cfg.config.api_host.as_ref().ok_or_else(|| {
            eyre::eyre!("`{method}` specified as a REST method in the config but no host specified")
        })?;
        let url = format!(
            "{}/{}",
            host.trim_end_matches('/'),
            method.trim_start_matches('/')
        );

        let res = client
            .post(&url)
            .send()
            .await
            .wrap_err(format!("failed to call {url}"))?;

        match response_config.format {
            Json => {
                let content: serde_json::Value = res
                    .json()
                    .await
                    .wrap_err(format!("Failed to receive valid json from {url}"))?;
                Ok(content.into())
            }
            Raw => {
                let content = res
                    .text()
                    .await
                    .wrap_err(format!("failed to receive text from {url}"))?;
                Ok(content.into())
            }
        }
    }

    async fn handle_sh(
        command: &str,
        response_config: &config::ShResponse,
    ) -> eyre::Result<BlockchainResponse> {
        use config::MethodResponseFormat::*;

        let args = command.split_whitespace();
        let output = tokio::process::Command::new("sh")
            .args(args)
            .output()
            .await
            .wrap_err(format!("failed to run {command}"))?;

        if !output.status.success() {
            eyre::bail!("failed to run {command}: {}", output.status);
        }

        match response_config.format {
            Json => {
                let content: serde_json::Value = serde_json::from_slice(&output.stdout)
                    .wrap_err(format!("failed to run {command}"))?;
                Ok(content.into())
            }
            Raw => {
                let content = String::from_utf8_lossy(&output.stdout).to_string();
                Ok(content.into())
            }
        }
    }
}

#[derive(Debug, serde::Serialize)]
enum BabelResponse {
    ListCapabilities(Vec<String>),
    BlockchainResponse(BlockchainResponse),
    Error(String),
}

#[derive(Debug, serde::Serialize)]
struct BlockchainResponse {
    value: String,
}

impl From<serde_json::Value> for BlockchainResponse {
    fn from(content: serde_json::Value) -> Self {
        Self {
            value: content
                .get("todo we gotta get this from the config")
                .and_then(|val| val.as_str())
                .unwrap_or_default()
                .to_string(),
        }
    }
}

impl From<String> for BlockchainResponse {
    fn from(value: String) -> Self {
        Self { value }
    }
}
