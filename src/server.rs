pub mod bv_pb {
    // https://github.com/tokio-rs/prost/issues/661
    #![allow(clippy::derive_partial_eq_without_eq)]
    tonic::include_proto!("blockjoy.blockvisor.v1");
}

use crate::{node_data::NodeStatus, nodes::Nodes};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use uuid::Uuid;
pub struct BlockvisorServer {
    pub nodes: Arc<Mutex<Nodes>>,
}

#[tonic::async_trait]
impl bv_pb::blockvisor_server::Blockvisor for BlockvisorServer {
    async fn create_node(
        &self,
        request: Request<bv_pb::CreateNodeRequest>,
    ) -> Result<Response<bv_pb::CreateNodeResponse>, Status> {
        let request = request.into_inner();
        let id = Uuid::parse_str(&request.id.unwrap().value).unwrap();
        let name = request.name;
        let chain = request.chain;

        self.nodes
            .lock()
            .await
            .create2(id, name, chain)
            .await
            .unwrap();

        let reply = bv_pb::CreateNodeResponse {};

        Ok(Response::new(reply))
    }

    async fn delete_node(
        &self,
        request: Request<bv_pb::DeleteNodeRequest>,
    ) -> Result<Response<bv_pb::DeleteNodeResponse>, Status> {
        let request = request.into_inner();
        let id = Uuid::parse_str(&request.id.unwrap().value).unwrap();

        self.nodes.lock().await.delete2(id).await.unwrap();

        let reply = bv_pb::DeleteNodeResponse {};

        Ok(Response::new(reply))
    }

    async fn start_node(
        &self,
        request: Request<bv_pb::StartNodeRequest>,
    ) -> Result<Response<bv_pb::StartNodeResponse>, Status> {
        let request = request.into_inner();
        let id = Uuid::parse_str(&request.id.unwrap().value).unwrap();

        self.nodes.lock().await.start2(id).await.unwrap();

        let reply = bv_pb::StartNodeResponse {};

        Ok(Response::new(reply))
    }

    async fn stop_node(
        &self,
        request: Request<bv_pb::StopNodeRequest>,
    ) -> Result<Response<bv_pb::StopNodeResponse>, Status> {
        let request = request.into_inner();
        let id = Uuid::parse_str(&request.id.unwrap().value).unwrap();

        self.nodes.lock().await.stop2(id).await.unwrap();

        let reply = bv_pb::StopNodeResponse {};

        Ok(Response::new(reply))
    }

    async fn get_nodes(
        &self,
        _request: Request<bv_pb::GetNodesRequest>,
    ) -> Result<Response<bv_pb::GetNodesResponse>, Status> {
        let list = self.nodes.lock().await.list2().await;
        let mut nodes = vec![];
        for node in list {
            let status = match node.status {
                NodeStatus::Running => bv_pb::NodeStatus::Running,
                NodeStatus::Stopped => bv_pb::NodeStatus::Stopped,
            };
            let n = bv_pb::Node {
                id: Some(bv_pb::Uuid {
                    value: node.id.to_string(),
                }),
                name: node.name,
                chain: node.chain,
                status: status.into(),
                ip: node.network_interface.ip.to_string(),
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
        let id = Uuid::parse_str(&request.id.unwrap().value).unwrap();

        let status = match self.nodes.lock().await.status2(id).await.unwrap() {
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
            .node_id_for_name2(&name)
            .await
            .unwrap();

        let reply = bv_pb::GetNodeIdForNameResponse {
            id: Some(bv_pb::Uuid {
                value: id.to_string(),
            }),
        };

        Ok(Response::new(reply))
    }
}
