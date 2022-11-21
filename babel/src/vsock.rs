use std::{sync::Arc, time};

use crate::run_flag::RunFlag;
use crate::{config, msg_handler};
use eyre::Context;
use futures::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast;
use tokio_vsock::VsockStream;

const VSOCK_HOST_CID: u32 = 3;
const VSOCK_PORT: u32 = 42;

/// This function tries to read messages from the vsocket and keeps responding to those messages.
/// Each opened connection gets handled separately by a tokio task and then the listener starts
/// listening for new messages. This means that we do not need to care if blockvisor shuts down or
/// restarts.
pub async fn serve(
    mut run: RunFlag,
    cfg: config::Babel,
    logs_rx: broadcast::Receiver<String>,
) -> eyre::Result<()> {
    let msf_handler = msg_handler::MsgHandler::new(cfg, time::Duration::from_secs(10), logs_rx)?;
    let client = Arc::new(msf_handler);

    tracing::debug!("Binding to virtual socket...");
    let listener = tokio_vsock::VsockListener::bind(VSOCK_HOST_CID, VSOCK_PORT)?;
    tracing::debug!("Bound");
    let mut incoming = listener.incoming();
    tracing::debug!("Receiving incoming messages");
    while run.load() {
        tokio::select!(
            res = incoming.next() => {
                if let Some(res) = res {
                    match res {
                        Ok(stream) => {
                            tracing::debug!("Stream opened, delegating to handler.");
                            tokio::spawn(serve_stream(stream, Arc::clone(&client)));
                        }
                        Err(_) => {
                            tracing::debug!("Receiving streams failed. Aborting babel.");
                            run.stop();
                        }
                    }
                }
            },
            _ = run.wait() => {},
        )
    }
    Ok(())
}

async fn serve_stream(mut stream: VsockStream, client: Arc<msg_handler::MsgHandler>) {
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
            let resp = babel_api::BabelResponse::Error(e.to_string());
            let _ = write_json(&mut stream, resp).await;
        }
    }
}

async fn handle_message(
    msg: &str,
    stream: &mut tokio_vsock::VsockStream,
    msg_handler: &msg_handler::MsgHandler,
) -> eyre::Result<()> {
    tracing::debug!("Received message: `{msg}`");
    let request: babel_api::BabelRequest =
        serde_json::from_str(msg).wrap_err(format!("Could not parse request as json '{msg}'"))?;
    let response = msg_handler.handle(request).await?;
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
