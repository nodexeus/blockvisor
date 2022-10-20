pub mod bv_pb {
    // https://github.com/tokio-rs/prost/issues/661
    #![allow(clippy::derive_partial_eq_without_eq)]
    tonic::include_proto!("blockjoy.blockvisor.v1");
}

use crate::{node_data::NodeStatus, nodes::Nodes};
use std::sync::Arc;
use std::{fmt, str::FromStr};
use tokio::sync::Mutex;
use tonic::{transport::Endpoint, Request, Response, Status};
use uuid::Uuid;

pub const BLOCKVISOR_SERVICE_PORT: usize = 9001;
pub const BLOCKVISOR_SERVICE_URL: &str = "http://localhost:9001";

lazy_static::lazy_static! {
    pub static ref BLOCKVISOR_SERVICE_ENDPOINT: Endpoint = Endpoint::from_str(BLOCKVISOR_SERVICE_URL).expect("valid url");
}

pub struct BlockvisorServer {
    pub nodes: Arc<Mutex<Nodes>>,
}

impl BlockvisorServer {
    pub async fn is_running() -> bool {
        Endpoint::connect(&BLOCKVISOR_SERVICE_ENDPOINT)
            .await
            .is_ok()
    }
}

#[tonic::async_trait]
impl bv_pb::blockvisor_server::Blockvisor for BlockvisorServer {
    async fn health(
        &self,
        _request: Request<bv_pb::HealthRequest>,
    ) -> Result<Response<bv_pb::HealthResponse>, Status> {
        let reply = bv_pb::HealthResponse {
            status: bv_pb::ServiceStatus::Ok.into(),
        };
        Ok(Response::new(reply))
    }

    async fn create_node(
        &self,
        request: Request<bv_pb::CreateNodeRequest>,
    ) -> Result<Response<bv_pb::CreateNodeResponse>, Status> {
        let request = request.into_inner();
        let id =
            Uuid::parse_str(&request.id).map_err(|e| Status::invalid_argument(e.to_string()))?;

        self.nodes
            .lock()
            .await
            .create(id, request.name, request.chain, request.ip, request.gateway)
            .await
            .map_err(|e| Status::unknown(e.to_string()))?;

        let reply = bv_pb::CreateNodeResponse {};

        Ok(Response::new(reply))
    }

    async fn delete_node(
        &self,
        request: Request<bv_pb::DeleteNodeRequest>,
    ) -> Result<Response<bv_pb::DeleteNodeResponse>, Status> {
        let request = request.into_inner();
        let id =
            Uuid::parse_str(&request.id).map_err(|e| Status::invalid_argument(e.to_string()))?;

        self.nodes
            .lock()
            .await
            .delete(id)
            .await
            .map_err(|e| Status::unknown(e.to_string()))?;

        let reply = bv_pb::DeleteNodeResponse {};

        Ok(Response::new(reply))
    }

    async fn start_node(
        &self,
        request: Request<bv_pb::StartNodeRequest>,
    ) -> Result<Response<bv_pb::StartNodeResponse>, Status> {
        let request = request.into_inner();
        let id =
            Uuid::parse_str(&request.id).map_err(|e| Status::invalid_argument(e.to_string()))?;

        self.nodes
            .lock()
            .await
            .start(id)
            .await
            .map_err(|e| Status::unknown(e.to_string()))?;

        let reply = bv_pb::StartNodeResponse {};

        Ok(Response::new(reply))
    }

    async fn stop_node(
        &self,
        request: Request<bv_pb::StopNodeRequest>,
    ) -> Result<Response<bv_pb::StopNodeResponse>, Status> {
        let request = request.into_inner();
        let id =
            Uuid::parse_str(&request.id).map_err(|e| Status::invalid_argument(e.to_string()))?;

        self.nodes
            .lock()
            .await
            .stop(id)
            .await
            .map_err(|e| Status::unknown(e.to_string()))?;

        let reply = bv_pb::StopNodeResponse {};

        Ok(Response::new(reply))
    }

    async fn get_nodes(
        &self,
        _request: Request<bv_pb::GetNodesRequest>,
    ) -> Result<Response<bv_pb::GetNodesResponse>, Status> {
        let list = self.nodes.lock().await.list().await;
        let mut nodes = vec![];
        for node in list {
            let status = match node.status() {
                NodeStatus::Running => bv_pb::NodeStatus::Running,
                NodeStatus::Stopped => bv_pb::NodeStatus::Stopped,
            };
            let n = bv_pb::Node {
                id: node.id.to_string(),
                name: node.name,
                chain: node.chain,
                status: status.into(),
                ip: node.network_interface.ip.to_string(),
                gateway: node.network_interface.gateway.to_string(),
            };
            nodes.push(n);
        }

        let reply = bv_pb::GetNodesResponse { nodes };

        Ok(Response::new(reply))
    }

    async fn get_node_status(
        &self,
        request: Request<bv_pb::GetNodeStatusRequest>,
    ) -> Result<Response<bv_pb::GetNodeStatusResponse>, Status> {
        let request = request.into_inner();
        let id =
            Uuid::parse_str(&request.id).map_err(|e| Status::invalid_argument(e.to_string()))?;

        let status = self
            .nodes
            .lock()
            .await
            .status(id)
            .await
            .map_err(|e| Status::unknown(e.to_string()))?;
        let status = match status {
            NodeStatus::Running => bv_pb::NodeStatus::Running,
            NodeStatus::Stopped => bv_pb::NodeStatus::Stopped,
        };

        let reply = bv_pb::GetNodeStatusResponse {
            status: status.into(),
        };

        Ok(Response::new(reply))
    }

    async fn get_node_id_for_name(
        &self,
        request: Request<bv_pb::GetNodeIdForNameRequest>,
    ) -> Result<Response<bv_pb::GetNodeIdForNameResponse>, Status> {
        let request = request.into_inner();
        let name = request.name;

        let id = self
            .nodes
            .lock()
            .await
            .node_id_for_name(&name)
            .await
            .map_err(|e| Status::unknown(e.to_string()))?;

        let reply = bv_pb::GetNodeIdForNameResponse { id: id.to_string() };

        Ok(Response::new(reply))
    }
}

impl fmt::Display for bv_pb::NodeStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
