use crate::grpc::with_auth;
use anyhow::{Context, Result};
use tonic::transport::Channel;

pub mod cb_pb {
    // https://github.com/tokio-rs/prost/issues/661
    #![allow(clippy::derive_partial_eq_without_eq)]
    tonic::include_proto!("blockjoy.api.v1.babel");
}
pub struct CookbookService {
    token: String,
    client: cb_pb::cook_book_service_client::CookBookServiceClient<Channel>,
}

impl CookbookService {
    pub async fn connect(url: &str, token: &str) -> Result<Self> {
        let client =
            cb_pb::cook_book_service_client::CookBookServiceClient::connect(url.to_string())
                .await
                .context("Failed to connect to cookbook service")?;
        Ok(Self {
            token: token.to_string(),
            client,
        })
    }

    pub async fn list_versions(&mut self, protocol: &str, node_type: &str) -> Result<Vec<String>> {
        let req = cb_pb::BabelVersionsRequest {
            protocol: protocol.to_string(),
            node_type: node_type.to_string(),
            status: cb_pb::StatusName::Development.into(),
        };

        let resp = self
            .client
            .list_babel_versions(with_auth(req, &self.token))
            .await?
            .into_inner();

        let mut versions: Vec<String> = resp
            .identifiers
            .into_iter()
            .map(|id| id.node_version)
            .collect();
        // sort desc
        versions.sort_by(|a, b| b.cmp(a));

        Ok(versions)
    }
}
