pub mod bv_pb {
    tonic::include_proto!("blockjoy.blockvisor.v1");
}

use crate::nodes::BabelError;
use crate::pal::Pal;
use crate::{get_bv_status, node_data::NodeImage, nodes, nodes::Nodes, ServiceStatus};
use babel_api::engine::JobStatus;
use std::fmt::Debug;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::instrument;

async fn status_check() -> Result<(), Status> {
    match get_bv_status().await {
        ServiceStatus::Undefined => Err(Status::unavailable("service not ready, try again later")),
        ServiceStatus::Updating => Err(Status::unavailable("pending update, try again later")),
        ServiceStatus::Broken => Err(Status::internal("service is broken, call support")),
        ServiceStatus::Ok => Ok(()),
    }
}

pub struct BlockvisorServer<P: Pal + Debug> {
    pub nodes: Arc<Nodes<P>>,
}

#[tonic::async_trait]
impl<P> bv_pb::blockvisor_server::Blockvisor for BlockvisorServer<P>
where
    P: Pal + Debug + Send + Sync + 'static,
    P::NetInterface: Send + Sync + 'static,
    P::NodeConnection: Send + Sync + 'static,
    P::VirtualMachine: Send + Sync + 'static,
{
    #[instrument(skip(self), ret(Debug))]
    async fn create_node(
        &self,
        request: Request<bv_pb::CreateNodeRequest>,
    ) -> Result<Response<bv_pb::CreateNodeResponse>, Status> {
        status_check().await?;
        let request = request.into_inner();
        let id = helpers::parse_uuid(request.id)?;
        let image: NodeImage = request
            .image
            .ok_or_else(|| Status::invalid_argument("Image not provided"))?
            .into();
        let properties = request
            .properties
            .into_iter()
            .map(|p| (p.name, p.value))
            .chain([("network".to_string(), request.network.clone())])
            .collect();

        self.nodes
            .create(
                id,
                nodes::NodeConfig {
                    name: request.name,
                    image,
                    ip: request.ip,
                    gateway: request.gateway,
                    rules: vec![],
                    properties,
                    network: request.network,
                },
            )
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;

        let reply = bv_pb::CreateNodeResponse {};

        Ok(Response::new(reply))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn upgrade_node(
        &self,
        request: Request<bv_pb::UpgradeNodeRequest>,
    ) -> Result<Response<bv_pb::UpgradeNodeResponse>, Status> {
        status_check().await?;
        let request = request.into_inner();
        let id = helpers::parse_uuid(request.id)?;
        let image: NodeImage = request
            .image
            .ok_or_else(|| Status::invalid_argument("Image not provided"))?
            .into();

        self.nodes
            .upgrade(id, image)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;

        let reply = bv_pb::UpgradeNodeResponse {};

        Ok(Response::new(reply))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn delete_node(
        &self,
        request: Request<bv_pb::DeleteNodeRequest>,
    ) -> Result<Response<bv_pb::DeleteNodeResponse>, Status> {
        status_check().await?;
        let request = request.into_inner();
        let id = helpers::parse_uuid(request.id)?;

        self.nodes
            .delete(id)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;

        let reply = bv_pb::DeleteNodeResponse {};
        Ok(Response::new(reply))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn start_node(
        &self,
        request: Request<bv_pb::StartNodeRequest>,
    ) -> Result<Response<bv_pb::StartNodeResponse>, Status> {
        status_check().await?;
        let request = request.into_inner();
        let id = helpers::parse_uuid(request.id)?;

        self.nodes
            .start(id)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;

        let reply = bv_pb::StartNodeResponse {};

        Ok(Response::new(reply))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn stop_node(
        &self,
        request: Request<bv_pb::StopNodeRequest>,
    ) -> Result<Response<bv_pb::StopNodeResponse>, Status> {
        status_check().await?;
        let request = request.into_inner();
        let id = helpers::parse_uuid(request.id)?;

        self.nodes
            .stop(id)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;

        let reply = bv_pb::StopNodeResponse {};

        Ok(Response::new(reply))
    }

    #[instrument(skip(self))]
    async fn get_node_jobs(
        &self,
        request: Request<bv_pb::GetNodeJobsRequest>,
    ) -> Result<Response<bv_pb::GetNodeJobsResponse>, Status> {
        status_check().await?;
        let request = request.into_inner();
        let id = helpers::parse_uuid(request.id)?;

        let mut jobs = vec![];
        for (name, status) in self
            .nodes
            .jobs(id)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?
        {
            let (job_status, exit_code, message) = match status {
                JobStatus::Pending => (bv_pb::JobStatus::JobPending, None, None),
                JobStatus::Running => (bv_pb::JobStatus::JobRunning, None, None),
                JobStatus::Finished { exit_code, message } => (
                    bv_pb::JobStatus::JobFinished,
                    exit_code.map(|c| c as u64),
                    Some(message),
                ),
                JobStatus::Stopped => (bv_pb::JobStatus::JobStopped, None, None),
            };
            let job = bv_pb::NodeJob {
                name,
                status: job_status.into(),
                exit_code,
                message,
            };
            jobs.push(job);
        }

        Ok(Response::new(bv_pb::GetNodeJobsResponse { jobs }))
    }

    #[instrument(skip(self))]
    async fn get_node_logs(
        &self,
        request: Request<bv_pb::GetNodeLogsRequest>,
    ) -> Result<Response<bv_pb::GetNodeLogsResponse>, Status> {
        status_check().await?;
        let request = request.into_inner();
        let id = helpers::parse_uuid(request.id)?;

        let logs = self
            .nodes
            .logs(id)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;

        Ok(Response::new(bv_pb::GetNodeLogsResponse { logs }))
    }

    #[instrument(skip(self))]
    async fn get_babel_logs(
        &self,
        request: Request<bv_pb::GetBabelLogsRequest>,
    ) -> Result<Response<bv_pb::GetBabelLogsResponse>, Status> {
        status_check().await?;
        let request = request.into_inner();
        let id = helpers::parse_uuid(request.id)?;

        let logs = self
            .nodes
            .babel_logs(id, request.max_lines)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;

        Ok(Response::new(bv_pb::GetBabelLogsResponse { logs }))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn get_node_keys(
        &self,
        request: Request<bv_pb::GetNodeKeysRequest>,
    ) -> Result<Response<bv_pb::GetNodeKeysResponse>, Status> {
        status_check().await?;
        let request = request.into_inner();
        let id = helpers::parse_uuid(request.id)?;

        let keys = self
            .nodes
            .keys(id)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;

        let names = keys.into_iter().map(|k| k.name).collect();
        Ok(Response::new(bv_pb::GetNodeKeysResponse { names }))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn get_node_id_for_name(
        &self,
        request: Request<bv_pb::GetNodeIdForNameRequest>,
    ) -> Result<Response<bv_pb::GetNodeIdForNameResponse>, Status> {
        status_check().await?;
        let request = request.into_inner();
        let name = request.name;

        let id = self
            .nodes
            .node_id_for_name(&name)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;

        let reply = bv_pb::GetNodeIdForNameResponse { id: id.to_string() };

        Ok(Response::new(reply))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn list_capabilities(
        &self,
        request: Request<bv_pb::ListCapabilitiesRequest>,
    ) -> Result<Response<bv_pb::ListCapabilitiesResponse>, Status> {
        status_check().await?;
        let request = request.into_inner();
        let id = helpers::parse_uuid(request.node_id)?;

        let capabilities = self
            .nodes
            .capabilities(id)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;

        Ok(Response::new(bv_pb::ListCapabilitiesResponse {
            capabilities,
        }))
    }

    /// Calls an arbitrary method on a the blockchain node running inside the VM.
    #[instrument(skip(self), ret(Debug))]
    async fn blockchain(
        &self,
        request: Request<bv_pb::BlockchainRequest>,
    ) -> Result<Response<bv_pb::BlockchainResponse>, Status> {
        status_check().await?;
        let request = request.into_inner();
        let id = helpers::parse_uuid(request.node_id)?;

        let value = self
            .nodes
            .call_method(id, &request.method, &request.param, true)
            .await
            .map_err(|e| match e {
                BabelError::MethodNotFound => Status::not_found("blockchain method not found"),
                BabelError::Internal { err } => Status::internal(format!("{err:#}")),
                BabelError::Plugin { err } => Status::unknown(format!("{err:#}")),
            })?;

        Ok(Response::new(bv_pb::BlockchainResponse { value }))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn get_node_metrics(
        &self,
        request: Request<bv_pb::GetNodeMetricsRequest>,
    ) -> Result<Response<bv_pb::GetNodeMetricsResponse>, Status> {
        status_check().await?;
        let request = request.into_inner();
        let id = helpers::parse_uuid(request.node_id)?;

        let metrics = self
            .nodes
            .metrics(id)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;

        Ok(Response::new(bv_pb::GetNodeMetricsResponse {
            height: metrics.height,
            block_age: metrics.block_age,
            staking_status: metrics.staking_status.map(|value| format!("{value:?}")),
            consensus: metrics.consensus,
            application_status: metrics.application_status.map(|value| format!("{value:?}")),
            sync_status: metrics.sync_status.map(|value| format!("{value:?}")),
        }))
    }
}

impl From<NodeImage> for bv_pb::NodeImage {
    fn from(image: NodeImage) -> Self {
        Self {
            protocol: image.protocol,
            node_type: image.node_type,
            node_version: image.node_version,
        }
    }
}

mod helpers {
    pub(super) fn parse_uuid(uuid: impl AsRef<str>) -> Result<uuid::Uuid, tonic::Status> {
        uuid.as_ref()
            .parse()
            .map_err(|_| tonic::Status::invalid_argument("Unparseable node id"))
    }
}

impl From<bv_pb::NodeImage> for NodeImage {
    fn from(image: bv_pb::NodeImage) -> Self {
        Self {
            protocol: image.protocol,
            node_type: image.node_type,
            node_version: image.node_version,
        }
    }
}
