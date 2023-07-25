use crate::node_data::{NodeDisplayInfo, NodeStatus};
use crate::nodes::Nodes;
use crate::pal::{NetInterface, Pal};
use crate::{get_bv_status, set_bv_status, ServiceStatus};
use chrono::Utc;
use std::fmt::Debug;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::instrument;
use uuid::Uuid;

#[tonic_rpc::tonic_rpc(bincode)]
trait Service {
    fn health() -> ServiceStatus;
    fn start_update() -> ServiceStatus;
    fn get_node_status(id: Uuid) -> NodeStatus;
    fn get_nodes() -> Vec<NodeDisplayInfo>;
}

pub struct State<P: Pal + Debug> {
    pub nodes: Arc<Nodes<P>>,
}

async fn status_check() -> Result<(), Status> {
    match get_bv_status().await {
        ServiceStatus::Undefined => Err(Status::unavailable("service not ready, try again later")),
        ServiceStatus::Updating => Err(Status::unavailable("pending update, try again later")),
        ServiceStatus::Broken => Err(Status::internal("service is broken, call support")),
        ServiceStatus::Ok => Ok(()),
    }
}

#[tonic::async_trait]
impl<P> service_server::Service for State<P>
where
    P: Pal + Debug + Send + Sync + 'static,
    P::NetInterface: Send + Sync + 'static,
    P::NodeConnection: Send + Sync + 'static,
    P::VirtualMachine: Send + Sync + 'static,
{
    #[instrument(skip(self), ret(Debug))]
    async fn health(&self, _request: Request<()>) -> Result<Response<ServiceStatus>, Status> {
        Ok(Response::new(get_bv_status().await))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn start_update(&self, _request: Request<()>) -> Result<Response<ServiceStatus>, Status> {
        set_bv_status(ServiceStatus::Updating).await;
        Ok(Response::new(ServiceStatus::Updating))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn get_node_status(
        &self,
        request: Request<Uuid>,
    ) -> Result<Response<NodeStatus>, Status> {
        status_check().await?;
        let id = request.into_inner();
        let status = self
            .nodes
            .status(id)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;
        Ok(Response::new(status))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn get_nodes(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Vec<NodeDisplayInfo>>, Status> {
        status_check().await?;
        let nodes_lock = self.nodes.nodes.read().await;
        let mut nodes = vec![];
        for (id, node_lock) in nodes_lock.iter() {
            let n = if let Ok(node) = node_lock.try_read() {
                let status = node.status();
                NodeDisplayInfo {
                    id: node.data.id,
                    name: node.data.name.clone(),
                    image: node.data.image.clone(),
                    status,
                    ip: node.data.network_interface.ip().to_string(),
                    gateway: node.data.network_interface.gateway().to_string(),
                    uptime: node
                        .data
                        .started_at
                        .map(|dt| Utc::now().signed_duration_since(dt).num_seconds()),
                }
            } else {
                let cache = self
                    .nodes
                    .node_data_cache(*id)
                    .await
                    .map_err(|e| Status::unknown(format!("{e:#}")))?;
                NodeDisplayInfo {
                    id: *id,
                    name: cache.name,
                    image: cache.image,
                    status: NodeStatus::Busy,
                    ip: cache.ip,
                    gateway: cache.gateway,
                    uptime: cache
                        .started_at
                        .map(|dt| Utc::now().signed_duration_since(dt).num_seconds()),
                }
            };
            nodes.push(n);
        }
        Ok(Response::new(nodes))
    }
}
