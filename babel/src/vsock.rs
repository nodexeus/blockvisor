use std::time;

use crate::{client, config};
use eyre::Context;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const VSOCK_HOST_CID: u32 = 2;
const VSOCK_PORT: u32 = 42;

/// This message will be sent to blockvisor on startup once we get rid of dbus.
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

pub async fn serve(cfg: config::Babel) -> eyre::Result<()> {
    let client = client::Client::new(cfg, time::Duration::from_secs(10))?;

    tracing::debug!("creating VSock connection..");
    let mut stream = tokio_vsock::VsockStream::connect(VSOCK_HOST_CID, VSOCK_PORT).await?;
    tracing::debug!("connected");
    write_json(&mut stream, Start::new()).await.unwrap(); // just testing
    let mut buf = String::new();
    loop {
        if let Err(e) = handle_message(&mut buf, &mut stream, &client).await {
            tracing::debug!("Failed to handle message: {e}");
            let resp = client::BabelResponse::Error(e.to_string());
            let _ = write_json(&mut stream, resp).await;
        }
    }
    // write_json(&mut stream, Stop::new()).await.unwrap();
    // Ok(())
}

async fn handle_message(
    buf: &mut String,
    stream: &mut tokio_vsock::VsockStream,
    client: &client::Client,
) -> eyre::Result<()> {
    let read = stream.read_to_string(buf).await?;
    if read == 0 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        return Ok(());
    }
    tracing::debug!("Received message: {buf:?}");
    let request: client::BabelRequest =
        serde_json::from_str(buf).wrap_err("Could not parse request as json")?;
    let response = client.handle(request).await;
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
