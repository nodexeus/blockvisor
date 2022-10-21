use crate::{
    config::{self, Method, MethodResponseFormat},
    error,
};
use eyre::Context;
use serde_json::json;
use std::time::Duration;

pub struct Client {
    inner: reqwest::Client,
    cfg: config::Babel,
}

impl std::ops::Deref for Client {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Client {
    pub fn new(cfg: config::Babel, timeout: Duration) -> eyre::Result<Self> {
        let client = reqwest::Client::builder().timeout(timeout).build()?;
        Ok(Self { inner: client, cfg })
    }
}

/// Each request that comes over the VSock to babel must be a piece of JSON that can be
/// deserialized into this struct.
#[derive(Debug, serde::Deserialize)]
pub enum BabelRequest {
    /// List the endpoints that are available for the current blockchain. These are extracted from
    /// the config, and just sent back as strings for now.
    ListCapabilities,
    /// Send a request to the current blockchain. We can identify the way to do this from the
    /// config and forward the provided parameters.
    BlockchainCommand(BlockchainCommand),
}

impl BabelRequest {
    pub async fn handle(self, client: &Client) -> eyre::Result<BabelResponse> {
        use BabelResponse::*;
        let resp = match self {
            Self::ListCapabilities => ListCapabilities(Self::handle_list_caps(client)),
            Self::BlockchainCommand(cmd) => BlockchainResponse(cmd.handle(client).await?),
        };
        Ok(resp)
    }

    /// List the capabilities that the current blockchain node supports.
    fn handle_list_caps(client: &Client) -> Vec<String> {
        client
            .cfg
            .methods
            .keys()
            .map(|method| method.to_string())
            .collect()
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct BlockchainCommand {
    name: String,
}

impl BlockchainCommand {
    async fn handle(self, client: &Client) -> eyre::Result<BlockchainResponse> {
        use config::Method::*;

        let method = client
            .cfg
            .methods
            .get(&self.name)
            .ok_or_else(|| error::Error::UnknownMethod { method: self.name })?;
        match method {
            Jrpc { method, .. } => Self::handle_jrpc(&method, client).await,
            Rest {
                method, response, ..
            } => Self::handle_rest(method, response, client).await,
            Sh { body, response, .. } => Self::handle_sh(body, response).await,
        }
    }

    async fn handle_jrpc(method: &str, client: &Client) -> eyre::Result<BlockchainResponse> {
        let url =
            client
                .cfg
                .config
                .api_host
                .as_deref()
                .ok_or_else(|| error::Error::NoHostSpecified {
                    method: method.to_string(),
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
        client: &Client,
    ) -> eyre::Result<BlockchainResponse> {
        use config::MethodResponseFormat::*;

        let host =
            client
                .cfg
                .config
                .api_host
                .as_ref()
                .ok_or_else(|| error::Error::NoHostSpecified {
                    method: method.to_string(),
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
            return Err(error::Error::Command {
                args: command.to_string(),
                output: format!("{output:?}"),
            }
            .into());
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
pub enum BabelResponse {
    ListCapabilities(Vec<String>),
    BlockchainResponse(BlockchainResponse),
    Error(String),
}

#[derive(Debug, serde::Serialize)]
pub struct BlockchainResponse {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Babel, Config, JrpcResponse, Method, RestResponse, ShResponse};
    use httpmock::prelude::*;
    use std::collections::BTreeMap;

    #[tokio::test]
    async fn test_sh() {
        let cfg = Babel {
            urn: "".to_string(),
            export: None,
            env: None,
            config: Config {
                babel_version: "".to_string(),
                node_version: "".to_string(),
                node_type: "".to_string(),
                description: None,
                api_host: None,
            },
            monitor: None,
            methods: BTreeMap::from([
                (
                    "raw".to_string(),
                    Method::Sh {
                        name: "raw".to_string(),
                        body: "echo make a toast".to_string(),
                        response: ShResponse {
                            status: 101,
                            format: MethodResponseFormat::Raw,
                        },
                    },
                ),
                (
                    "json".to_string(),
                    Method::Sh {
                        name: "json".to_string(),
                        body: "echo \\\"make a toast\\\"".to_string(),
                        response: ShResponse {
                            status: 102,
                            format: MethodResponseFormat::Json,
                        },
                    },
                ),
            ]),
        };
        let client = Client::new(cfg, Duration::from_secs(10)).unwrap();

        let output = client.handle_method_call("raw").await.unwrap();
        assert_eq!(output, "make a toast\n");

        let output = client.handle_method_call("json").await.unwrap();
        assert_eq!(output, "make a toast");

        let output = client.handle_method_call("unknown").await;
        assert_eq!(
            output.err().unwrap().to_string(),
            "Method `unknown` not found"
        );
    }

    #[tokio::test]
    async fn test_rest_json_ok() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/items");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({"result": [1, 2, 3]}));
        });

        let cfg = Babel {
            urn: "".to_string(),
            export: None,
            env: None,
            config: Config {
                babel_version: "".to_string(),
                node_version: "".to_string(),
                node_type: "".to_string(),
                description: None,
                api_host: Some(format!("http://{}", server.address())),
            },
            monitor: None,
            methods: BTreeMap::from([(
                "json items".to_string(),
                Method::Rest {
                    name: "json items".to_string(),
                    method: "items".to_string(),
                    response: RestResponse {
                        status: 101,
                        field: Some("result".to_string()),
                        format: MethodResponseFormat::Json,
                    },
                },
            )]),
        };

        let client = Client::new(cfg, Duration::from_secs(1)).unwrap();
        let output = client.handle_method_call("json items").await.unwrap();

        mock.assert();
        assert_eq!(output, "[1,2,3]");
    }

    #[tokio::test]
    async fn test_rest_json_full_response_ok() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/items");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({"result": [1, 2, 3]}));
        });

        let cfg = Babel {
            urn: "".to_string(),
            export: None,
            env: None,
            config: Config {
                babel_version: "".to_string(),
                node_version: "".to_string(),
                node_type: "".to_string(),
                description: None,
                api_host: Some(format!("http://{}", server.address())),
            },
            monitor: None,
            methods: BTreeMap::from([(
                "json items".to_string(),
                Method::Rest {
                    name: "json items".to_string(),
                    method: "items".to_string(),
                    response: RestResponse {
                        status: 101,
                        field: None,
                        format: MethodResponseFormat::Json,
                    },
                },
            )]),
        };

        let client = Client::new(cfg, Duration::from_secs(1)).unwrap();
        let output = client.handle_method_call("json items").await.unwrap();

        mock.assert();
        assert_eq!(output, "{\"result\":[1,2,3]}");
    }

    #[tokio::test]
    async fn test_jrpc_json_ok() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .header("Content-Type", "application/json")
                .json_body(json!({
                    "id": 0,
                    "jsonrpc": "2.0",
                    "method": "info_get",
                }));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!({
                        "id": 0,
                        "jsonrpc": "2.0",
                        "result": {"info": {"height": 123, "address": "abc"}},
                }));
        });

        let cfg = Babel {
            urn: "".to_string(),
            export: None,
            env: None,
            config: Config {
                babel_version: "".to_string(),
                node_version: "".to_string(),
                node_type: "".to_string(),
                description: None,
                api_host: Some(format!("http://{}", server.address())),
            },
            monitor: None,
            methods: BTreeMap::from([(
                "get height".to_string(),
                Method::Jrpc {
                    name: "get height".to_string(),
                    method: "info_get".to_string(),
                    response: JrpcResponse {
                        code: 101,
                        field: Some("result.info.height".to_string()),
                    },
                },
            )]),
        };

        let client = Client::new(cfg, Duration::from_secs(1)).unwrap();
        let output = client.handle_method_call("get height").await.unwrap();

        mock.assert();
        assert_eq!(output, "123");
    }
}
