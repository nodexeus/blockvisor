use crate::{babel_engine::NodeInfo, config::SharedConfig, services};
use async_trait::async_trait;
use babel_api::engine::{Chunk, DownloadManifest, DownloadMetadata, UploadSlots};
use std::path::PathBuf;
use tokio::fs;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::{
    transport::Server,
    {Request, Response, Status},
};
use tracing::{debug, error, warn};

struct BabelEngineService {
    node_info: NodeInfo,
    config: SharedConfig,
}

#[async_trait]
impl babel_api::babel::babel_engine_server::BabelEngine for BabelEngineService {
    async fn put_download_manifest(
        &self,
        request: Request<DownloadManifest>,
    ) -> eyre::Result<Response<()>, Status> {
        debug!("putting DownloadManifest to API...");
        let manifest = request.into_inner();
        if let Err(err) = services::blockchain_archive::put_download_manifest(
            &self.config,
            self.node_info.image.clone(),
            self.node_info.network.clone(),
            manifest,
        )
        .await
        {
            warn!("{err:#}");
            Err(Status::internal(err.to_string()))
        } else {
            Ok(Response::new(()))
        }
    }

    async fn get_download_metadata(
        &self,
        _request: Request<()>,
    ) -> eyre::Result<Response<DownloadMetadata>, Status> {
        debug!(
            "getting DownloadMetadata from API for {}/{}",
            self.node_info.image, self.node_info.network
        );
        match services::blockchain_archive::get_download_metadata(
            &self.config,
            self.node_info.image.clone(),
            self.node_info.network.clone(),
        )
        .await
        {
            Err(err) => {
                warn!("{err:#}");
                Err(Status::internal(err.to_string()))
            }
            Ok(metadata) => Ok(Response::new(metadata)),
        }
    }
    async fn get_download_chunks(
        &self,
        request: Request<(u64, Vec<u32>)>,
    ) -> eyre::Result<Response<Vec<Chunk>>, Status> {
        let (data_version, chunk_indexes) = request.into_inner();
        debug!(
            "getting DownloadChunks from API for {}/{}/{}",
            self.node_info.image, self.node_info.network, data_version
        );
        match services::blockchain_archive::get_download_chunks(
            &self.config,
            self.node_info.image.clone(),
            self.node_info.network.clone(),
            data_version,
            chunk_indexes,
        )
        .await
        {
            Err(err) => {
                warn!("{err:#}");
                Err(Status::internal(err.to_string()))
            }
            Ok(chunks) => Ok(Response::new(chunks)),
        }
    }

    async fn get_upload_slots(
        &self,
        request: Request<(Option<u64>, Vec<u32>, u32)>,
    ) -> eyre::Result<Response<UploadSlots>, Status> {
        let (data_version, slots, url_expires_secs) = request.into_inner();
        debug!(
            "getting UploadSlots from API for {}/{}/{:?} with {} slots",
            self.node_info.image,
            self.node_info.network,
            data_version,
            slots.len()
        );
        match services::blockchain_archive::get_upload_slots(
            &self.config,
            self.node_info.image.clone(),
            self.node_info.network.clone(),
            data_version,
            slots,
            url_expires_secs,
        )
        .await
        {
            Err(err) => {
                warn!("{err:#}");
                Err(Status::internal(err.to_string()))
            }
            Ok(slots) => Ok(Response::new(slots)),
        }
    }

    async fn upgrade_blocking_jobs_finished(
        &self,
        _request: Request<()>,
    ) -> eyre::Result<Response<()>, Status> {
        Ok(Response::new(()))
    }

    async fn bv_error(&self, request: Request<String>) -> eyre::Result<Response<()>, Status> {
        let message = request.into_inner();
        error!("Babel: {message}");
        Ok(Response::new(()))
    }
}

#[derive(Debug)]
pub struct BabelEngineServer {
    handle: tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
    tx: tokio::sync::oneshot::Sender<()>,
}

pub async fn start_server(
    engine_socket_path: PathBuf,
    node_info: NodeInfo,
    config: SharedConfig,
) -> eyre::Result<BabelEngineServer> {
    let engine_service = BabelEngineService { node_info, config };
    let _ = fs::remove_file(&engine_socket_path).await;
    let uds_stream = UnixListenerStream::new(tokio::net::UnixListener::bind(engine_socket_path)?);
    let (tx, rx) = tokio::sync::oneshot::channel();
    Ok(BabelEngineServer {
        handle: tokio::spawn(
            Server::builder()
                .max_concurrent_streams(1)
                .add_service(
                    babel_api::babel::babel_engine_server::BabelEngineServer::new(engine_service),
                )
                .serve_with_incoming_shutdown(uds_stream, async {
                    rx.await.ok();
                }),
        ),
        tx,
    })
}

impl BabelEngineServer {
    pub async fn stop(self) -> eyre::Result<()> {
        let _ = self.tx.send(());
        Ok(self.handle.await??)
    }
}
