use crate::config;
use eyre::Context;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const VSOCK_HOST_CID: u32 = 2;
const VSOCK_PORT: u32 = 42;

#[derive(serde::Serialize)]
struct Start {
    start_msg: String,
}

impl Start {
    fn new() -> Self {
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

/// This function tries to read messages from the vsocket and keeps responding to those messages.
pub async fn serve(cfg: config::Babel) -> eyre::Result<()> {
    let client = reqwest::Client::new();

    tracing::debug!("creating VSock connection..");
    let mut stream = tokio_vsock::VsockStream::connect(VSOCK_HOST_CID, VSOCK_PORT).await?;
    tracing::debug!("connected");
    write_json(&mut stream, Start::new()).await.unwrap(); // just testing
    let mut buf = String::new();
    loop {
        if let Err(e) = handle_message(&mut buf, &mut stream, &client, &cfg).await {
            tracing::debug!("Failed to handle message: {e}");
            let resp = BabelResponse::Error(e.to_string());
            let _ = write_json(&mut stream, resp).await;
        }
    }
    // write_json(&mut stream, Stop::new()).await.unwrap();
    // Ok(())
}

async fn handle_message(
    buf: &mut String,
    stream: &mut tokio_vsock::VsockStream,
    client: &reqwest::Client,
    cfg: &config::Babel,
) -> eyre::Result<()> {
    let read = stream.read_to_string(buf).await?;
    if read == 0 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        return Ok(());
    }
    tracing::debug!("Received message: {buf:?}");
    let request: BabelRequest =
        serde_json::from_str(buf).wrap_err("Could not parse request as json")?;
    let response = request.handle(client, cfg).await?;
    tracing::debug!("Sending response: {response:?}");
    Ok(())
}

async fn write_json<W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
    message: impl serde::Serialize,
) -> eyre::Result<()> {
    let msg = serde_json::to_vec(&message)?;
    writer.write_all(&msg).await?;
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
