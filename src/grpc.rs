pub mod pb {
    // https://github.com/tokio-rs/prost/issues/661
    #![allow(clippy::derive_partial_eq_without_eq)]
    tonic::include_proto!("blockjoy.api.v1");
}

use pb::command_flow_client::CommandFlowClient;
use tonic::codegen::InterceptedService;
use tonic::service::Interceptor;
use tonic::{Request, Status};

pub struct AuthToken(pub String);

pub type Client = CommandFlowClient<InterceptedService<tonic::transport::Channel, AuthToken>>;

impl Interceptor for AuthToken {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        let mut request = request;
        let val = format!("Bearer {}", base64::encode(self.0.clone()));
        request
            .metadata_mut()
            .insert("authorization", val.parse().unwrap());
        Ok(request)
    }
}

impl Client {
    pub fn with_auth(channel: tonic::transport::Channel, token: AuthToken) -> Self {
        CommandFlowClient::with_interceptor(channel, token)
    }
}
