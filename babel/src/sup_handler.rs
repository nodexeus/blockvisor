use crate::error::Error;
#[cfg(target_os = "linux")]
use crate::vsock::Handler;
use crate::{babel_binary, error, Result};
#[cfg(target_os = "linux")]
use async_trait::async_trait;
use babel_api::*;
use std::path::PathBuf;
use tokio::sync::{broadcast, mpsc, Mutex};

pub struct SupHandler {
    logs_rx: Mutex<broadcast::Receiver<String>>,
    babel_restart_tx: mpsc::Sender<()>,
    babel_bin_path: PathBuf,
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
    pub fn new(
        logs_rx: broadcast::Receiver<String>,
        babel_restart_tx: mpsc::Sender<()>,
        babel_bin_path: PathBuf,
    ) -> Result<Self> {
        Ok(Self {
            logs_rx: Mutex::new(logs_rx),
            babel_restart_tx,
            babel_bin_path,
        })
    }

    pub async fn handle(&self, req: SupervisorRequest) -> Result<SupervisorResponse> {
        match req {
            SupervisorRequest::StartBabel(babel_bin, checksum) => {
                self.handle_start_babel(babel_bin, checksum).await
            }
            SupervisorRequest::CheckBabelChecksum(checksum) => self.handle_checksum(checksum),
            SupervisorRequest::Logs => Ok(SupervisorResponse::Logs(self.handle_logs().await)),
        }
    }

    async fn handle_start_babel(
        &self,
        babel_bin: Vec<u8>,
        expected_checksum: u32,
    ) -> Result<SupervisorResponse, Error> {
        babel_binary::save(babel_bin, &self.babel_bin_path).map_err(|err| {
            error::Error::InternalError {
                description: format!("failed to save babel binary: {err}"),
            }
        })?;
        let checksum = babel_binary::checksum(&self.babel_bin_path).map_err(|err| {
            error::Error::InternalError {
                description: format!("failed to calculate babel binary checksum: {err}"),
            }
        })?;
        if expected_checksum == checksum {
            self.babel_restart_tx
                .send(())
                .await
                .map_err(|err| error::Error::InternalError {
                    description: format!("failed to send babel restart signal: {err}"),
                })?;
            Ok(SupervisorResponse::Ok)
        } else {
            Err(error::Error::InternalError {
                description: "babel binary checksum doesn't match".to_string(),
            })
        }
    }

    fn handle_checksum(&self, expected_checksum: u32) -> Result<SupervisorResponse> {
        let checksum = babel_binary::checksum(&self.babel_bin_path).map_err(|err| {
            error::Error::InternalError {
                description: format!("failed to calculate babel binary checksum: {err}"),
            }
        })?;

        if expected_checksum == checksum {
            Ok(SupervisorResponse::Ok)
        } else {
            Err(error::Error::InvalidBabel)
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
    use assert_fs::TempDir;
    use std::fs;
    use std::io::Write;

    fn create_dummy_babel(path: &PathBuf) -> Result<()> {
        // create dummy babel bin
        let _ = fs::create_dir_all(path.parent().unwrap());
        let mut babel = fs::OpenOptions::new().create(true).write(true).open(path)?;
        writeln!(babel, "dummy babel")?;
        Ok(())
    }

    #[tokio::test]
    async fn test_start_babel() -> Result<()> {
        let tmp_dir = TempDir::new().unwrap();
        let babel_bin_path = tmp_dir.join("babel");
        let babel_bin = vec![1, 2, 3, 4, 6, 7, 8, 9, 10, 11];
        let (_, logs_rx) = broadcast::channel(16);
        let (babel_restart_tx, mut babel_restart_rx) = mpsc::channel(16);
        let sup_handler =
            SupHandler::new(logs_rx, babel_restart_tx, babel_bin_path.clone()).unwrap();

        let resp = sup_handler
            .handle(SupervisorRequest::StartBabel(babel_bin.clone(), 0))
            .await;
        if !matches!(resp, Err(error::Error::InternalError { .. })) {
            panic!("invalid StartBabel response {resp:?}")
        }
        if let Ok(()) = babel_restart_rx.try_recv() {
            panic!("unexpected babel restart signal")
        }

        let resp = sup_handler
            .handle(SupervisorRequest::StartBabel(babel_bin.clone(), 2368545531))
            .await?;
        if !matches!(resp, SupervisorResponse::Ok) {
            panic!("invalid StartBabel response {resp:?}")
        }
        babel_restart_rx
            .try_recv()
            .expect("babel restart signal not sent");

        Ok(())
    }

    #[tokio::test]
    async fn test_checksum() -> Result<()> {
        let tmp_dir = TempDir::new().unwrap();
        let babel_bin_path = tmp_dir.join("babel");
        let (_, logs_rx) = broadcast::channel(16);
        let (babel_restart_tx, _) = mpsc::channel(16);
        let sup_handler =
            SupHandler::new(logs_rx, babel_restart_tx, babel_bin_path.clone()).unwrap();

        let resp = sup_handler
            .handle(SupervisorRequest::CheckBabelChecksum(0))
            .await;
        if !matches!(resp, Err(error::Error::InternalError { .. })) {
            panic!("invalid CheckBabelChecksum response {resp:?}")
        }
        create_dummy_babel(&babel_bin_path)?;
        let resp = sup_handler
            .handle(SupervisorRequest::CheckBabelChecksum(0))
            .await;
        if !matches!(resp, Err(error::Error::InvalidBabel)) {
            panic!("invalid CheckBabelChecksum response {resp:?}")
        }
        let resp = sup_handler
            .handle(SupervisorRequest::CheckBabelChecksum(2395331324))
            .await?;
        if !matches!(resp, SupervisorResponse::Ok) {
            panic!("invalid CheckBabelChecksum response {resp:?}")
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_logs() {
        let (tx, logs_rx) = broadcast::channel(16);
        let (babel_restart_tx, _) = mpsc::channel(16);
        let sup_handler = SupHandler::new(logs_rx, babel_restart_tx, Default::default()).unwrap();
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
