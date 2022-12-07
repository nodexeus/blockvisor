#[cfg(target_os = "linux")]
use crate::vsock::Handler;
use crate::Result;
#[cfg(target_os = "linux")]
use async_trait::async_trait;
use babel_api::*;
use tokio::sync::{broadcast, Mutex};

pub struct SupHandler {
    logs_rx: Mutex<broadcast::Receiver<String>>,
}

#[cfg(target_os = "linux")]
#[async_trait]
impl Handler for SupHandler {
    async fn handle(&self, message: &str) -> eyre::Result<Vec<u8>> {
        use eyre::Context;

        let request: babel_api::SupervisorRequest = serde_json::from_str(message).wrap_err(
            format!("Could not parse supervisor request as json: '{message}'"),
        )?;

        let resp = match self.handle(request).await {
            Ok(resp) => resp,
            Err(e) => {
                tracing::debug!("Failed to handle message in supervisor: {e}");
                babel_api::SupervisorResponse::Error(e.to_string())
            }
        };

        Ok(serde_json::to_vec(&resp)?)
    }
}

impl SupHandler {
    pub fn new(logs_rx: broadcast::Receiver<String>) -> Result<Self> {
        Ok(Self {
            logs_rx: Mutex::new(logs_rx),
        })
    }

    pub async fn handle(&self, req: SupervisorRequest) -> Result<SupervisorResponse> {
        use SupervisorResponse::*;

        match req {
            SupervisorRequest::Ping => Ok(Pong),
            SupervisorRequest::Logs => Ok(Logs(self.handle_logs().await)),
        }
    }

    /// List logs from blockchain entry_points.
    async fn handle_logs(&self) -> Vec<String> {
        let mut logs = Vec::default();
        let mut rx = self.logs_rx.lock().await;
        loop {
            match rx.try_recv() {
                Ok(log) => logs.push(log),
                Err(broadcast::error::TryRecvError::Lagged(_)) => {}
                Err(_) => break,
            }
        }
        logs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_logs() {
        let (tx, logs_rx) = broadcast::channel(16);
        let sup_handler = SupHandler::new(logs_rx).unwrap();
        tx.send("log1".to_string()).expect("failed to send log");
        tx.send("log2".to_string()).expect("failed to send log");
        tx.send("log3".to_string()).expect("failed to send log");

        if let SupervisorResponse::Logs(logs) = sup_handler
            .handle(SupervisorRequest::Logs)
            .await
            .expect("failed to handle logs request")
        {
            assert_eq!(vec!["log1", "log2", "log3"], logs)
        } else {
            panic!("invalid logs response")
        }
    }
}
