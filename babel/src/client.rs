use crate::{
    config::{self, Method, MethodResponseFormat},
    error,
};
use serde_json::json;
use std::time::Duration;

#[allow(dead_code)]
pub struct Client {
    inner: reqwest::Client,
    cfg: config::Babel,
}

#[allow(dead_code)]
impl Client {
    pub fn new(cfg: config::Babel, timeout: Duration) -> eyre::Result<Self> {
        let client = reqwest::Client::builder().timeout(timeout).build()?;
        Ok(Self { inner: client, cfg })
    }

    pub async fn handle_method_call(&self, method_name: &str) -> Result<String, error::Error> {
        let method = match self.cfg.methods.get(method_name) {
            Some(method) => method,
            None => {
                return Err(error::Error::UnknownMethod {
                    method: method_name.to_string(),
                })
            }
        };

        match method {
            Method::Jrpc {
                name: _,
                method: jrpc_method,
                response,
            } => {
                let url = self.cfg.config.api_host.clone().ok_or_else(|| {
                    error::Error::NoHostSpecified {
                        method: method_name.to_string(),
                    }
                })?;
                let text = self
                    .inner
                    .post(&url)
                    .json(&json!({ "jsonrpc": "2.0", "id": 0, "method": jrpc_method }))
                    .send()
                    .await?
                    .text()
                    .await?;
                if let Some(field) = &response.field {
                    Ok(gjson::get(&text, field).to_string())
                } else {
                    Ok(text)
                }
            }
            Method::Rest {
                name: _,
                method: rest_method,
                response,
            } => {
                let url = self
                    .cfg
                    .config
                    .api_host
                    .as_ref()
                    .map(|host| {
                        format!(
                            "{}/{}",
                            host.trim_end_matches('/'),
                            rest_method.trim_start_matches('/')
                        )
                    })
                    .ok_or_else(|| error::Error::NoHostSpecified {
                        method: method_name.to_string(),
                    })?;
                let text = self.inner.post(&url).send().await?.text().await?;
                if let Some(field) = &response.field {
                    Ok(gjson::get(&text, field).to_string())
                } else {
                    Ok(text)
                }
            }
            Method::Sh {
                name: _,
                body: command,
                response,
            } => {
                let output = tokio::process::Command::new("sh")
                    .args(&["-c", &command.to_string()])
                    .output()
                    .await?;

                if !output.status.success() {
                    return Err(error::Error::Command {
                        args: command.to_string(),
                        output: format!("{output:?}"),
                    });
                }

                match response.format {
                    MethodResponseFormat::Json => {
                        println!("{:?}", &output);
                        Ok(serde_json::from_slice(&output.stdout)?)
                    }
                    MethodResponseFormat::Raw => {
                        Ok(String::from_utf8_lossy(&output.stdout).to_string())
                    }
                }
            }
        }
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
