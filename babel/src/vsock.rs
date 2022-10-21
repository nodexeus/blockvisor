use crate::{client, config};
use eyre::ContextCompat;
use futures::TryStreamExt;
use std::{future::ready, time};
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
    let client = client::Client::new(cfg, time::Duration::from_secs(10))?;

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

        let req: client::BabelRequest = serde_json::from_str(&body).unwrap(); // TODO: remove unwrap
        let resp = req.handle(&client).await.unwrap_or_else(|e| {
            tracing::error!("Failed to handle request: {e}");
            client::BabelResponse::Error(e.to_string())
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
