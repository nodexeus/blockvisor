pub mod pb {
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
        request
            .metadata_mut()
            .insert("authorization", self.0.parse().unwrap());
        Ok(request)
    }
}

impl Client {
    pub fn with_auth(channel: tonic::transport::Channel, token: AuthToken) -> Self {
        CommandFlowClient::with_interceptor(channel, token)
    }
}
