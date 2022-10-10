use std::future::ready;

use eyre::ContextCompat;
use futures::TryStreamExt;
use reqwest::Client;
use serde_json::json;
use tokio::fs;
use tracing::trace;
use zbus::{fdo, Address, ConnectionBuilder, MessageStream, MessageType, VsockAddress};

use crate::config::{self, Method, MethodResponseFormat};

const VSOCK_HOST_CID: u32 = 2;
const VSOCK_PORT: u32 = 42;

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

    let client = Client::new();

    let mut stream = MessageStream::from(&conn).try_filter(|msg| {
        trace!("got message: {:?}", msg);
        ready(
            msg.header().is_ok()
                && msg.member().is_some()
                && msg.message_type() == MessageType::MethodCall
                && msg.interface().as_ref().map(|i| i.as_str()) == Some("com.BlockJoy.Babel")
                && msg.path().as_ref().map(|i| i.as_str()) == Some("/com/BlockJoy/Babel"),
        )
    });
    while let Some(msg) = stream.try_next().await? {
        trace!("received Babel method message: {:?}", msg);
        // SAFETY: The filter we setup above already ensures header and member are set.
        let header = msg.header().unwrap();
        let method_name = msg.member().unwrap();
        if let Err(e) = match handle_method_call(method_name.as_str(), &client, &cfg).await {
            Ok(response) => conn.reply(&msg, &response).await,
            Err(e) => conn.reply_dbus_error(&header, e).await,
        } {
            tracing::error!("failed to reply to method call `{method_name}`: {e}");
        }
    }

    Ok(())
}

async fn handle_method_call(
    method_name: &str,
    client: &Client,
    cfg: &config::Babel,
) -> fdo::Result<String> {
    let method = match cfg.methods.get(method_name) {
        Some(method) => method,
        None => {
            return Err(fdo::Error::UnknownMethod(format!(
                "{} not found",
                method_name
            )))
        }
    };

    match method {
        Method::Jrpc {
            name: _,
            method: jrpc_method,
            response: _,
        } => {
            let url = cfg.config.api_host.clone().ok_or_else(|| {
                fdo::Error::InvalidFileContent(format!("`{jrpc_method}` specified as a JSON-RPC method in the config but no host specified"))
            })?;
            client
                .post(&url)
                .json(&json!({ "jsonrpc": "2.0", "id": "id", "method": jrpc_method }))
                .send()
                .await
                .map_err(|e| fdo::Error::IOError(format!("failed to call {url}: {e}")))?
                .json()
                .await
                .map_err(|e| fdo::Error::IOError(format!("failed to call {url}: {e}")))
        }
        Method::Rest {
            name: _,
            method: rest_method,
            response,
        } => {
            let url = cfg.config.api_host.as_ref().map(|host| {
                format!("{}/{}", host.trim_end_matches('/'), rest_method.trim_start_matches('/'))
            }).ok_or_else(|| {
                fdo::Error::InvalidFileContent(format!(
                    "`{rest_method}` specified as a REST method in the config but no host specified",
                ))
            })?;
            let res = client
                .post(&url)
                .send()
                .await
                .map_err(|e| fdo::Error::IOError(format!("failed to call {url}: {e}")))?;

            match response.format {
                MethodResponseFormat::Json => res.json().await,
                MethodResponseFormat::Raw => res.text().await,
            }
            .map_err(|e| fdo::Error::IOError(format!("failed to call {url}: {e}")))
        }
        Method::Sh {
            name: _,
            body: command,
            response,
        } => {
            let args = command.split_whitespace();
            let output = tokio::process::Command::new("sh")
                .args(args)
                .output()
                .await
                .map_err(|e| fdo::Error::IOError(format!("failed to run {command}: {e}")))?;

            if !output.status.success() {
                return Err(fdo::Error::IOError(format!(
                    "failed to run {command}: {}",
                    output.status,
                )));
            }

            match response.format {
                MethodResponseFormat::Json => serde_json::from_slice(&output.stdout),
                MethodResponseFormat::Raw => {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                }
            }
            .map_err(|e| fdo::Error::IOError(format!("failed to run {command}: {e}")))
        }
    }
}
