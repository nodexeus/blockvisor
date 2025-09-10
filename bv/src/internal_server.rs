use crate::{
    apptainer_platform::ApptainerPlatform,
    bv_config,
    bv_config::SharedConfig,
    cluster::ClusterData,
    hosts,
    node_state::{NodeProperties, NodeState, ProtocolImageKey, VmStatus},
    nodes_manager::{self, MaybeNode, NodesManager},
    pal::Pal,
    services::{
        self,
        api::{self, common, pb},
    },
    {get_bv_status, set_bv_status, ServiceStatus}, {node_metrics, BV_VAR_PATH},
};
use babel_api::engine::JobsInfo;
use eyre::{anyhow, bail, Context};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fmt::Debug, sync::Arc};
use tonic::{Request, Response, Status};
use tracing::instrument;
use uuid::Uuid;

// Data that we display in cli
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NodeDisplayInfo {
    pub status: VmStatus,
    pub state: NodeState,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateNodeRequest {
    pub protocol_image_key: ProtocolImageKey,
    pub image_version: Option<String>,
    pub build_version: Option<u64>,
    pub properties: NodeProperties,
    pub tags: Vec<String>,
}

#[tonic_rpc::tonic_rpc(bincode)]
trait Service {
    fn info() -> String;
    fn health() -> ServiceStatus;
    fn start_update() -> ServiceStatus;
    fn get_host_metrics() -> hosts::HostMetrics;
    fn get_node(id: Uuid) -> NodeDisplayInfo;
    fn get_nodes(local: bool) -> Vec<NodeDisplayInfo>;
    fn create_node(req: CreateNodeRequest) -> NodeDisplayInfo;
    fn create_dev_node(req: NodeState) -> NodeDisplayInfo;
    fn start_node(id: Uuid);
    fn stop_node(id: Uuid, force: bool);
    fn upgrade_node(id: Uuid, version: Option<String>, build: Option<u64>);
    fn upgrade_dev_node(req: NodeState) -> NodeDisplayInfo;
    fn delete_node(id: Uuid);
    fn get_node_jobs(id: Uuid) -> JobsInfo;
    fn get_node_job_info(id: Uuid, job_name: String) -> babel_api::engine::JobInfo;
    fn start_node_job(id: Uuid, job_name: String);
    fn stop_node_job(id: Uuid, job_name: String);
    fn skip_node_job(id: Uuid, job_name: String);
    fn cleanup_node_job(id: Uuid, job_name: String);
    fn get_node_id_for_name(name: String) -> String;
    fn list_capabilities(id: Uuid) -> Vec<String>;
    fn run(id: Uuid, method: String, param: String) -> String;
    fn reload_plugin(id: Uuid);
    fn get_node_metrics(id: Uuid) -> node_metrics::Metric;
    fn get_cluster_status() -> String; // TODO: update with proper struct
}

pub struct State<P: Pal + Debug> {
    pub config: SharedConfig,
    pub nodes_manager: Arc<NodesManager<P>>,
    pub cluster: Arc<Option<ClusterData>>,
    pub dev_mode: bool,
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
    P::NodeConnection: Send + Sync + 'static,
    P::ApiServiceConnector: Send + Sync + 'static,
    P::VirtualMachine: Send + Sync + 'static,
    P::RecoveryBackoff: Send + Sync + 'static,
{
    #[instrument(skip(self), ret(Debug))]
    async fn info(&self, _request: Request<()>) -> Result<Response<String>, Status> {
        let pal = ApptainerPlatform::default()
            .await
            .map_err(|e| Status::internal(format!("{e:#}")))?;
        let mut config = bv_config::Config::load(pal.bv_root())
            .await
            .map_err(|e| Status::internal(format!("{e:#}")))?;
        config.api_config.token = "***".to_string();
        config.api_config.refresh_token = "***".to_string();
        let service_name = if self.dev_mode {
            format!("{}-dev", env!("CARGO_PKG_NAME"))
        } else {
            env!("CARGO_PKG_NAME").to_owned()
        };
        Ok(Response::new(format!(
            "{} {} - {:?}\n BV_PATH: {}\n BABEL_PATH: {}\n JOB_RUNNER_PATH: {}\n CONFIG: {:#?}",
            service_name,
            env!("CARGO_PKG_VERSION"),
            get_bv_status().await,
            pal.bv_root().join(BV_VAR_PATH).to_string_lossy(),
            pal.babel_path().to_string_lossy(),
            pal.job_runner_path().to_string_lossy(),
            config,
        )))
    }

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
    async fn get_host_metrics(
        &self,
        _request: Request<()>,
    ) -> Result<Response<hosts::HostMetrics>, Status> {
        Ok(Response::new(
            hosts::HostMetrics::collect(
                self.nodes_manager.nodes_data_cache().await,
                self.nodes_manager.pal(),
            )
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?,
        ))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn get_node(&self, request: Request<Uuid>) -> Result<Response<NodeDisplayInfo>, Status> {
        status_check().await?;
        let id = request.into_inner();
        let nodes_lock = self.nodes_manager.nodes_list().await;
        if let Some(node_lock) = nodes_lock.get(&id) {
            Ok(Response::new(
                self.get_node_display_info(id, node_lock)
                    .await
                    .map_err(|e| Status::unknown(format!("{e:#}")))?,
            ))
        } else {
            self.get_remote_nodes(Some(id))
                .await?
                .pop()
                .map(Response::new)
                .ok_or(Status::not_found(format!("Node {id} not found")))
        }
    }

    #[instrument(skip(self), ret(Debug))]
    async fn get_nodes(
        &self,
        request: Request<bool>,
    ) -> Result<Response<Vec<NodeDisplayInfo>>, Status> {
        let local = request.into_inner();
        status_check().await?;
        let mut nodes = if local {
            Default::default()
        } else {
            self.get_remote_nodes(None).await?
        };
        let nodes_lock = self.nodes_manager.nodes_list().await;
        for (id, node_lock) in nodes_lock.iter() {
            nodes.retain(|node| node.state.id != *id);
            nodes.push(
                self.get_node_display_info(*id, node_lock)
                    .await
                    .map_err(|e| Status::unknown(format!("{e:#}")))?,
            );
        }
        Ok(Response::new(nodes))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn create_node(
        &self,
        request: Request<CreateNodeRequest>,
    ) -> Result<Response<NodeDisplayInfo>, Status> {
        status_check().await?;
        Ok(Response::new(
            self.create_node(request.into_inner())
                .await
                .map_err(|err| Status::unknown(format!("{err:#}")))?,
        ))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn create_dev_node(
        &self,
        request: Request<NodeState>,
    ) -> Result<Response<NodeDisplayInfo>, Status> {
        status_check().await?;
        Ok(Response::new(
            self.create_dev_node(request.into_inner())
                .await
                .map_err(|err| Status::unknown(format!("{err:#}")))?,
        ))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn start_node(&self, request: Request<Uuid>) -> Result<Response<()>, Status> {
        status_check().await?;
        let id = request.into_inner();
        if self.is_dev_node(id).await {
            self.nodes_manager
                .start(id, true)
                .await
                .map_err(|e| Status::unknown(format!("{e:#}")))?;
        } else {
            self.connect_to_node_service()
                .await?
                .start(pb::NodeServiceStartRequest {
                    node_id: id.to_string(),
                })
                .await
                .map_err(|e| Status::unknown(format!("{e:#}")))?;
        }
        Ok(Response::new(()))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn stop_node(&self, request: Request<(Uuid, bool)>) -> Result<Response<()>, Status> {
        status_check().await?;
        let (id, force) = request.into_inner();
        if self.is_dev_node(id).await {
            self.nodes_manager
                .stop(id, force)
                .await
                .map_err(|e| Status::unknown(format!("{e:#}")))?;
        } else {
            self.connect_to_node_service()
                .await?
                .stop(pb::NodeServiceStopRequest {
                    node_id: id.to_string(),
                })
                .await
                .map_err(|e| Status::unknown(format!("{e:#}")))?;
        }
        Ok(Response::new(()))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn upgrade_node(
        &self,
        request: Request<(Uuid, Option<String>, Option<u64>)>,
    ) -> Result<Response<()>, Status> {
        status_check().await?;
        let (id, version, build) = request.into_inner();
        if self.is_dev_node(id).await {
            Err(Status::unimplemented("dev node upgrade is not supported"))
        } else {
            let node = self
                .nodes_manager
                .node_state_cache(id)
                .await
                .map_err(|e| Status::unknown(format!("{e:#}")))?;
            let image = services::connect_to_api_service(
                &self.config,
                pb::image_service_client::ImageServiceClient::with_interceptor,
            )
            .await
            .map_err(|e| Status::unknown(format!("Error connecting to api: {e:#}")))?
            .get_image(pb::ImageServiceGetImageRequest {
                version_key: Some(node.image_key.clone().into()),
                org_id: None,
                semantic_version: version,
                build_version: build,
            })
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?
            .into_inner()
            .image
            .ok_or(Status::not_found(format!(
                "image for {}/{} not found",
                node.image_key.protocol_key, node.image_key.variant_key
            )))?;

            self.connect_to_node_service()
                .await?
                .upgrade_image(pb::NodeServiceUpgradeImageRequest {
                    node_ids: vec![id.to_string()],
                    image_id: image.image_id,
                    org_id: None,
                })
                .await
                .map_err(|e| Status::unknown(format!("{e:#}")))?;
            Ok(Response::new(()))
        }
    }

    #[instrument(skip(self), ret(Debug))]
    async fn upgrade_dev_node(
        &self,
        request: Request<NodeState>,
    ) -> Result<Response<NodeDisplayInfo>, Status> {
        status_check().await?;
        let desired_node_state = request.into_inner();
        if !self.is_dev_node(desired_node_state.id).await {
            Err(Status::unimplemented(
                "can't upgrade non dev node with dev image",
            ))
        } else {
            Ok(Response::new(
                self.upgrade_dev_node(desired_node_state)
                    .await
                    .map_err(|err| Status::unknown(format!("{err:#}")))?,
            ))
        }
    }

    #[instrument(skip(self), ret(Debug))]
    async fn delete_node(&self, request: Request<Uuid>) -> Result<Response<()>, Status> {
        status_check().await?;
        let id = request.into_inner();
        if self.is_dev_node(id).await {
            self.nodes_manager
                .delete(id)
                .await
                .map_err(|e| Status::unknown(format!("{e:#}")))?;
        } else {
            self.connect_to_node_service()
                .await?
                .delete(pb::NodeServiceDeleteRequest {
                    node_id: id.to_string(),
                })
                .await
                .map_err(|e| Status::unknown(format!("{e:#}")))?;
        }
        Ok(Response::new(()))
    }

    #[instrument(skip(self))]
    async fn get_node_jobs(&self, request: Request<Uuid>) -> Result<Response<JobsInfo>, Status> {
        status_check().await?;
        let id = request.into_inner();
        let jobs = self
            .nodes_manager
            .jobs(id)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;
        Ok(Response::new(jobs))
    }

    #[instrument(skip(self))]
    async fn get_node_job_info(
        &self,
        request: Request<(Uuid, String)>,
    ) -> Result<Response<babel_api::engine::JobInfo>, Status> {
        status_check().await?;
        let (id, job_name) = request.into_inner();
        let info = self
            .nodes_manager
            .job_info(id, &job_name)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;
        Ok(Response::new(info))
    }

    #[instrument(skip(self))]
    async fn start_node_job(
        &self,
        request: Request<(Uuid, String)>,
    ) -> Result<Response<()>, Status> {
        status_check().await?;
        let (id, job_name) = request.into_inner();
        self.nodes_manager
            .start_job(id, &job_name)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;
        Ok(Response::new(()))
    }

    #[instrument(skip(self))]
    async fn skip_node_job(
        &self,
        request: Request<(Uuid, String)>,
    ) -> Result<Response<()>, Status> {
        status_check().await?;
        let (id, job_name) = request.into_inner();
        self.nodes_manager
            .skip_job(id, &job_name)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;
        Ok(Response::new(()))
    }

    #[instrument(skip(self))]
    async fn stop_node_job(
        &self,
        request: Request<(Uuid, String)>,
    ) -> Result<Response<()>, Status> {
        status_check().await?;
        let (id, job_name) = request.into_inner();
        self.nodes_manager
            .stop_job(id, &job_name)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;
        Ok(Response::new(()))
    }

    #[instrument(skip(self))]
    async fn cleanup_node_job(
        &self,
        request: Request<(Uuid, String)>,
    ) -> Result<Response<()>, Status> {
        status_check().await?;
        let (id, job_name) = request.into_inner();
        self.nodes_manager
            .cleanup_job(id, &job_name)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;
        Ok(Response::new(()))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn get_node_id_for_name(
        &self,
        request: Request<String>,
    ) -> Result<Response<String>, Status> {
        status_check().await?;
        let name = request.into_inner();
        match self.nodes_manager.node_id_for_name(&name).await {
            Ok(id) => Ok(Response::new(id.to_string())),
            Err(err) => {
                let remote_nodes = self
                    .get_remote_nodes(None)
                    .await
                    .map_err(|e| Status::unknown(format!("{e:#}")))?;
                remote_nodes
                    .into_iter()
                    .find_map(|node| {
                        if node.state.name == name {
                            Some(Response::new(node.state.id.to_string()))
                        } else {
                            None
                        }
                    })
                    .ok_or(Status::not_found(format!("{err:#}")))
            }
        }
    }

    #[instrument(skip(self), ret(Debug))]
    async fn list_capabilities(
        &self,
        request: Request<Uuid>,
    ) -> Result<Response<Vec<String>>, Status> {
        status_check().await?;
        let id = request.into_inner();
        let capabilities = self
            .nodes_manager
            .capabilities(id)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;
        Ok(Response::new(capabilities))
    }

    /// Calls an arbitrary method on the node running inside the VM.
    #[instrument(skip(self), ret(Debug))]
    async fn run(
        &self,
        request: Request<(Uuid, String, String)>,
    ) -> Result<Response<String>, Status> {
        status_check().await?;
        let (id, method, param) = request.into_inner();
        let value = self
            .nodes_manager
            .call_method(id, &method, &param, self.is_dev_node(id).await)
            .await
            .map_err(|e| match e {
                nodes_manager::BabelError::MethodNotFound => {
                    Status::not_found("protocol method not found")
                }
                nodes_manager::BabelError::Internal { err } => Status::internal(format!("{err:#}")),
                nodes_manager::BabelError::Plugin { err } => Status::unknown(format!("{err:#}")),
            })?;
        Ok(Response::new(value))
    }

    /// Reload babel plugin for given node.
    #[instrument(skip(self), ret(Debug))]
    async fn reload_plugin(&self, request: Request<Uuid>) -> Result<Response<()>, Status> {
        status_check().await?;
        let id = request.into_inner();
        self.nodes_manager
            .reload_plugin(id)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;
        Ok(Response::new(()))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn get_node_metrics(
        &self,
        request: Request<Uuid>,
    ) -> Result<Response<node_metrics::Metric>, Status> {
        status_check().await?;
        let id = request.into_inner();
        let metrics = self
            .nodes_manager
            .metrics(id)
            .await
            .map_err(|e| Status::unknown(format!("{e:#}")))?;
        Ok(Response::new(metrics))
    }

    #[instrument(skip(self), ret(Debug))]
    async fn get_cluster_status(&self, _request: Request<()>) -> Result<Response<String>, Status> {
        status_check().await?;
        let status = if let Some(ref cluster) = *self.cluster {
            let chitchat = cluster.chitchat.lock().await;
            json!({"cluster_id": chitchat.cluster_id().to_string(),
                "cluster_state": chitchat.state_snapshot(),
                "live_hosts": chitchat.live_nodes().cloned().collect::<Vec<_>>(),
                "dead_hosts": chitchat.dead_nodes().cloned().collect::<Vec<_>>(),
            })
            .to_string()
        } else {
            "None".to_string()
        };
        Ok(Response::new(status))
    }
}

impl<P> State<P>
where
    P: Pal + Send + Sync + Debug + 'static,
    P::NodeConnection: Send + Sync,
    P::ApiServiceConnector: Send + Sync,
    P::VirtualMachine: Send + Sync,
    P::RecoveryBackoff: Send + Sync + 'static,
{
    async fn is_dev_node(&self, id: Uuid) -> bool {
        self.dev_mode
            || match self.nodes_manager.nodes_list().await.get(&id) {
                Some(MaybeNode::Node(node)) => node.read().await.state.dev_mode,
                Some(MaybeNode::BrokenNode(state)) => state.dev_mode,
                None => false,
            }
    }

    async fn connect_to_node_service(&self) -> Result<api::NodesServiceClient, Status> {
        services::connect_to_api_service(
            &self.config,
            pb::node_service_client::NodeServiceClient::with_interceptor,
        )
        .await
        .map_err(|e| Status::unknown(format!("Error connecting to api: {e:#}")))
    }

    async fn get_node_display_info(
        &self,
        id: Uuid,
        maybe_node: &MaybeNode<P>,
    ) -> eyre::Result<NodeDisplayInfo> {
        Ok(match maybe_node {
            MaybeNode::Node(node_lock) => {
                if let Ok(node) = node_lock.try_read() {
                    let status = node.status().await;
                    NodeDisplayInfo {
                        state: node.state.clone(),
                        status,
                    }
                } else {
                    let cache = self
                        .nodes_manager
                        .node_state_cache(id)
                        .await
                        .map_err(|e| Status::unknown(format!("{e:#}")))?;
                    NodeDisplayInfo {
                        state: cache,
                        status: VmStatus::Busy,
                    }
                }
            }
            MaybeNode::BrokenNode(state) => NodeDisplayInfo {
                state: state.clone(),
                status: VmStatus::Failed,
            },
        })
    }

    async fn get_remote_nodes(
        &self,
        node_id: Option<Uuid>,
    ) -> Result<Vec<NodeDisplayInfo>, Status> {
        if self.dev_mode {
            Ok(Default::default())
        } else {
            let bv_config = self.config.config.read().await.clone();
            if let Some(org_id) = bv_config.private_org_id.or(bv_config.org_id) {
                let mut node_client = self.connect_to_node_service().await?;
                let nodes = node_client
                    .list(pb::NodeServiceListRequest {
                        org_ids: vec![org_id],
                        offset: 0,
                        host_ids: vec![bv_config.id],
                        limit: if node_id.is_some() {
                            1
                        } else {
                            bv_config.net_conf.available_ips.len() as u64
                        },
                        search: node_id.map(|id| pb::NodeSearch {
                            operator: common::SearchOperator::And.into(),
                            node_id: Some(id.to_string()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    })
                    .await?
                    .into_inner()
                    .nodes
                    .into_iter();
                nodes
                    .filter_map(|node| {
                        if let Some(status) = &node.node_status {
                            if status.state() == common::NodeState::Deleted {
                                return None;
                            }
                        }
                        let status = node
                            .node_status
                            .as_ref()
                            .map(|status| match status.state() {
                                common::NodeState::Running => VmStatus::Running,
                                common::NodeState::Stopped => VmStatus::Stopped,
                                common::NodeState::Failed => VmStatus::Failed,
                                _ => VmStatus::Busy,
                            })
                            .unwrap_or(VmStatus::Busy);
                        Some(
                            node.try_into()
                                .map(|state| NodeDisplayInfo { status, state })
                                .map_err(|e| Status::unknown(format!("{e:#}"))),
                        )
                    })
                    .collect::<Result<Vec<NodeDisplayInfo>, Status>>()
            } else {
                Ok(Default::default())
            }
        }
    }

    async fn create_node(&self, req: CreateNodeRequest) -> eyre::Result<NodeDisplayInfo> {
        if self.dev_mode {
            bail!("blockvisord-dev support only dev nodes")
        }
        // map properties into api format
        let properties = req
            .properties
            .clone()
            .into_iter()
            .map(|(key, value)| pb::NewImagePropertyValue { key, value })
            .collect::<Vec<_>>();
        let config = self.config.read().await;
        let image_id = self
            .get_image_id(
                &req.protocol_image_key.protocol_key,
                &req.protocol_image_key.variant_key,
                req.image_version,
                req.build_version,
            )
            .await?;

        let mut node_client = self.connect_to_node_service().await?;
        let mut created_nodes = node_client
            .create(pb::NodeServiceCreateRequest {
                old_node_id: None,
                org_id: config.private_org_id.or(config.org_id).ok_or(anyhow!(
                    "create node is not supported on public hosts without org assigned - use web frontend"
                ))?,
                new_values: properties,
                image_id,
                add_rules: vec![],
                tags: if req.tags.is_empty() {
                    None
                } else {
                    Some(common::Tags {
                        tags: req
                            .tags
                            .into_iter()
                            .map(|tag| common::Tag { name: tag })
                            .collect(),
                    })
                },
                launcher: Some(common::NodeLauncher {
                    launch: Some(common::node_launcher::Launch::ByHost(common::ByHost {
                        host_counts: vec![common::HostCount {
                            host_id: config.id,
                            node_count: 1,
                        }],
                    })),
                }),
            })
            .await
            .with_context(|| "create node via API failed")?
            .into_inner()
            .nodes;

        let node = match created_nodes.len() {
            0 => Err(anyhow!("empty node create response from API")),
            1 => Ok(created_nodes.pop().expect("one created node")),
            _ => Err(anyhow!("unexpected multiple node creation response")),
        }?;

        Ok(NodeDisplayInfo {
            state: node.try_into()?,
            status: VmStatus::Stopped,
        })
    }

    async fn create_dev_node(
        &self,
        mut new_node_state: NodeState,
    ) -> eyre::Result<NodeDisplayInfo> {
        new_node_state.dev_mode = true;
        Ok(NodeDisplayInfo {
            state: self.nodes_manager.create(new_node_state).await?,
            status: VmStatus::Stopped,
        })
    }

    async fn upgrade_dev_node(
        &self,
        desired_node_state: NodeState,
    ) -> eyre::Result<NodeDisplayInfo> {
        let (state, status) = self.nodes_manager.upgrade(desired_node_state).await?;
        Ok(NodeDisplayInfo { state, status })
    }

    /// Find image id by protocol name and version.
    async fn get_image_id(
        &self,
        protocol: &str,
        variant: &str,
        version: Option<String>,
        build_version: Option<u64>,
    ) -> eyre::Result<String> {
        Ok(
            services::protocol::ProtocolService::new(services::DefaultConnector {
                config: self.config.clone(),
            })
            .await?
            .get_image(
                ProtocolImageKey {
                    protocol_key: protocol.to_string(),
                    variant_key: variant.to_string(),
                },
                version,
                build_version,
            )
            .await?
            .ok_or(anyhow!("image not found"))?
            .image_id,
        )
    }
}
