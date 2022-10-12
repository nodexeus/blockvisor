use crate::{
    client::Client,
    config::{self, Method, MethodResponseFormat},
    error,
};
use eyre::ContextCompat;
use futures::TryStreamExt;
use serde_json::json;
use std::{future::ready, time::Duration};
use tokio::fs;
use tracing::trace;
use zbus::{fdo, Address, ConnectionBuilder, MessageStream, MessageType, VsockAddress};

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

    let client = Client::new(cfg, Duration::from_secs(10))?;

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

        if let Err(e) = match client
            .handle_method_call(method_name.as_str())
            .await
            .map_err(|e| match e {
                error::Error::UnknownMethod { method } => {
                    fdo::Error::UnknownMethod(format!("{method} not found"))
                }
                error::Error::NoHostSpecified { method } => fdo::Error::InvalidFileContent(
                    format!("`{method}` specified in the config but no host specified"),
                ),
                error::Error::Command { args, output } => {
                    fdo::Error::IOError(format!("failed to run command `{args}`: {output:?}"))
                }
                err => fdo::Error::IOError(err.to_string()),
            }) {
            Ok(response) => conn.reply(&msg, &response).await,
            Err(e) => conn.reply_dbus_error(&header, e).await,
        } {
            tracing::error!("failed to reply to method call `{method_name}`: {e}");
        }
    }

    Ok(())
}
