use std::{sync::Arc, time};

use crate::{client, config};
use eyre::Context;
use futures::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_vsock::VsockStream;

const VSOCK_HOST_CID: u32 = 3;
const VSOCK_PORT: u32 = 42;

/// This function tries to read messages from the vsocket and keeps responding to those messages.
/// Each opened connection gets handled separately by a tokio task and then the listener starts
/// listening for new messages. This means that we do not need to care if blockvisor shuts down or
/// restarts.
pub async fn serve(cfg: config::Babel) -> eyre::Result<()> {
    let client = client::Client::new(cfg, time::Duration::from_secs(10))?;
    let client = Arc::new(client);

    tracing::debug!("Binding to virtual socket...");
    let listener = tokio_vsock::VsockListener::bind(VSOCK_HOST_CID, VSOCK_PORT)?;
    tracing::debug!("Bound");
    let mut incoming = listener.incoming();
    tracing::debug!("Receiving incoming messages");
    while let Some(res) = incoming.next().await {
        match res {
            Ok(stream) => {
                tracing::debug!("Stream opened, delegating to handler.");
                tokio::spawn(serve_stream(stream, Arc::clone(&client)));
            }
            Err(_) => {
                tracing::debug!("Receiving streams failed. Aborting babel.");
                break;
            }
        }
    }
    Ok(())
}

async fn serve_stream(mut stream: VsockStream, client: Arc<client::Client>) {
    loop {
        let mut buf = vec![0u8; 5000];
        let len = stream.read(&mut buf).await.unwrap();
        if len == 0 {
            tracing::info!("Vsock stream closed. Shutting down connection handler.");
            break;
        }
        buf.resize(len, 0);
        let msg = std::str::from_utf8(&buf).unwrap();
        if let Err(e) = handle_message(msg, &mut stream, &client).await {
            tracing::debug!("Failed to handle message: {e}");
            let resp = client::BabelResponse::Error(e.to_string());
            let _ = write_json(&mut stream, resp).await;
        }
    }
}

async fn handle_message(
    msg: &str,
    stream: &mut tokio_vsock::VsockStream,
    client: &client::Client,
) -> eyre::Result<()> {
    tracing::debug!("Received message: `{msg}`");
    let request: client::BabelRequest =
        serde_json::from_str(msg).wrap_err("Could not parse request as json")?;
    let response = client.handle(request).await?;
    tracing::debug!("Sending response: {response:?}");
    write_json(stream, response).await?;
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
