use crate::{get_bv_status, set_bv_status, ServiceStatus};
use tonic::{Request, Response, Status};
use tracing::instrument;

#[tonic_rpc::tonic_rpc(bincode)]
trait Service {
    fn health() -> ServiceStatus;
    fn start_update() -> ServiceStatus;
}

pub struct State {}

#[tonic::async_trait]
impl service_server::Service for State {
    #[instrument(skip(self), ret(Debug))]
    async fn health(&self, _request: Request<()>) -> Result<Response<ServiceStatus>, Status> {
        Ok(Response::new(get_bv_status().await))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn start_update(&self, _request: Request<()>) -> Result<Response<ServiceStatus>, Status> {
        set_bv_status(ServiceStatus::Updating).await;
        Ok(Response::new(ServiceStatus::Updating))
    }
}
