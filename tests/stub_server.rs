use blockvisord::grpc::pb;
use std::{pin::Pin, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};

type ResponseResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<pb::Command, Status>> + Send>>;

pub struct StubServer {
    pub commands: Arc<Mutex<Vec<pb::Command>>>,
    pub updates_tx: mpsc::Sender<pb::InfoUpdate>,
    pub shutdown_tx: mpsc::Sender<()>,
}

#[tonic::async_trait]
impl pb::command_flow_server::CommandFlow for StubServer {
    type CommandsStream = ResponseStream;

    async fn commands(
        &self,
        req: Request<Streaming<pb::InfoUpdate>>,
    ) -> ResponseResult<Self::CommandsStream> {
        let mut in_stream = req.into_inner();
        let (tx, rx) = mpsc::channel(128);

        for command in self.commands.lock().await.iter() {
            tx.send(Ok(command.clone())).await.unwrap();
        }
        self.commands.lock().await.clear();

        let shutdown_tx = self.shutdown_tx.clone();
        let updates_tx = self.updates_tx.clone();
        tokio::spawn(async move {
            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(v) => {
                        println!("server: get update {:?}", &v);
                        updates_tx.send(v).await.unwrap();
                    }
                    Err(err) => {
                        eprintln!("server: error {:?}", err);
                        break;
                    }
                }
            }
            println!("server: stream ended");
            shutdown_tx.send(()).await.unwrap();
        });

        let out_stream = ReceiverStream::new(rx);

        Ok(Response::new(Box::pin(out_stream) as Self::CommandsStream))
    }
}
