use anyhow::{anyhow, bail, Result};
use futures_util::TryFutureExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use tokio::fs::{self, read_dir};
use tokio::sync::broadcast::{self, Sender};
use tokio::sync::OnceCell;
use tokio::time::timeout;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::{
    grpc::pb,
    network_interface::NetworkInterface,
    node::Node,
    node_data::{NodeData, NodeStatus},
};

const NODES_CONFIG_FILENAME: &str = "nodes.toml";

lazy_static::lazy_static! {
    pub static ref REGISTRY_CONFIG_DIR: PathBuf = home::home_dir()
        .map(|p| p.join(".cache"))
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("blockvisor")
        .join("nodes");

    static ref REGISTRY_CONFIG_FILE: PathBuf = REGISTRY_CONFIG_DIR.join(NODES_CONFIG_FILENAME);
}

#[derive(Clone, Debug)]
pub enum ServiceStatus {
    Enabled,
    Disabled,
}

#[derive(Debug)]
pub struct Nodes {
    pub nodes: HashMap<Uuid, Node>,
    pub node_ids: HashMap<String, Uuid>,
    data: CommonData,
    tx: OnceCell<Sender<pb::InfoUpdate>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CommonData {
    pub machine_index: u32,
}

impl Nodes {
    #[instrument(skip(self))]
    pub async fn create(
        &mut self,
        id: Uuid,
        name: String,
        image: String,
        ip: String,
        gateway: String,
    ) -> Result<()> {
        if self.nodes.contains_key(&id) {
            bail!(format!("Node with id `{}` exists", &id));
        }

        if self.node_ids.contains_key(&name) {
            bail!(format!("Node with name `{}` exists", &name));
        }

        if let Err(error) = self.send_node_status(&id, pb::node_info::ContainerStatus::Creating) {
            error!("Cannot send node status: {error:?}");
        };

        self.data.machine_index += 1;
        let ip = ip.parse()?;
        let gateway = gateway.parse()?;
        let network_interface = self.create_network_interface(ip, gateway).await?;
        let node = NodeData {
            id,
            name: name.clone(),
            image,
            expected_status: NodeStatus::Stopped,
            network_interface,
        };
        self.save().await?;

        let node = Node::create(node).await?;
        self.nodes.insert(id, node);
        self.node_ids.insert(name, id);
        debug!("Node with id `{}` created", id);

        if let Err(error) = self.send_node_status(&id, pb::node_info::ContainerStatus::Stopped) {
            error!("Cannot send node status: {error:?}");
        };

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn delete(&mut self, id: Uuid) -> Result<()> {
        let node = self
            .nodes
            .remove(&id)
            .ok_or_else(|| anyhow!("Node with id `{}` not found", &id))?;
        self.node_ids.remove(&node.data.name);

        if let Err(error) = self.send_node_status(&id, pb::node_info::ContainerStatus::Deleting) {
            error!("Cannot send node status: {error:?}");
        };

        node.delete().await?;
        debug!("deleted");

        if let Err(error) = self.send_node_status(&id, pb::node_info::ContainerStatus::Deleted) {
            error!("Cannot send node status: {error:?}");
        };

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn start(&mut self, id: Uuid) -> Result<()> {
        if let Err(error) = self.send_node_status(&id, pb::node_info::ContainerStatus::Starting) {
            error!("Cannot send node status: {error:?}");
        };

        let node = self
            .nodes
            .get_mut(&id)
            .ok_or_else(|| anyhow!("Node with id `{}` not found", &id))?;
        debug!("found node");

        node.start().await?;
        debug!("started");

        if let Err(error) = self.send_node_status(&id, pb::node_info::ContainerStatus::Running) {
            error!("Cannot send node status: {error:?}");
        };

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn stop(&mut self, id: Uuid) -> Result<()> {
        if let Err(error) = self.send_node_status(&id, pb::node_info::ContainerStatus::Stopping) {
            error!("Cannot send node status: {error:?}");
        };

        let node = self
            .nodes
            .get_mut(&id)
            .ok_or_else(|| anyhow!("Node with id `{}` not found", &id))?;
        debug!("found node");

        node.stop().await?;
        debug!("stopped");

        if let Err(error) = self.send_node_status(&id, pb::node_info::ContainerStatus::Stopped) {
            error!("Cannot send node status: {error:?}");
        };

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn list(&self) -> Vec<NodeData> {
        debug!("listing {} nodes", self.nodes.len());

        self.nodes.values().map(|n| n.data.clone()).collect()
    }

    #[instrument(skip(self))]
    pub async fn status(&self, id: Uuid) -> Result<NodeStatus> {
        let node = self
            .nodes
            .get(&id)
            .ok_or_else(|| anyhow!("Node with id `{}` not found", &id))?;

        Ok(node.status())
    }

    // TODO: Rest of the NodeCommand variants.

    pub async fn node_id_for_name(&self, name: &str) -> Result<Uuid> {
        let uuid = self
            .node_ids
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow!("Node with name `{}` not found", name))?;

        Ok(uuid)
    }
}

impl Nodes {
    pub fn new(nodes_data: CommonData) -> Self {
        Self {
            data: nodes_data,
            nodes: HashMap::new(),
            node_ids: HashMap::new(),
            tx: OnceCell::new(),
        }
    }

    pub async fn load() -> Result<Nodes> {
        // First load the common data file.
        info!(
            "Reading nodes common config file: {}",
            REGISTRY_CONFIG_FILE.display()
        );
        let config = fs::read_to_string(&*REGISTRY_CONFIG_FILE).await?;
        let nodes_data = toml::from_str(&config)?;
        // Now the individual node data files.
        info!(
            "Reading nodes config dir: {}",
            REGISTRY_CONFIG_DIR.display()
        );
        let mut this = Nodes::new(nodes_data);
        let mut dir = read_dir(&*REGISTRY_CONFIG_DIR).await?;
        while let Some(entry) = dir.next_entry().await? {
            // blockvisord should not bail on problems with individual node files.
            // It should log warnings though.
            let path = entry.path();
            if path == *REGISTRY_CONFIG_FILE {
                // Skip the common data file.
                continue;
            }
            match NodeData::load(&path)
                .and_then(|data| async {
                    // Since this is the startup phase it doesn't make sense to wait a long time
                    // for the nodes to come online. For that reason we restrict the allowed delay
                    // further down to one second.
                    let max_delay = std::time::Duration::from_secs(1);
                    let babel_conn = timeout(max_delay, Node::conn(data.id))
                        .await
                        .ok()
                        .and_then(Result::ok);
                    tracing::debug!("Established babel connection");
                    Node::connect(data, babel_conn).await
                })
                .await
            {
                Ok(node) => {
                    this.node_ids.insert(node.data.name.clone(), node.id());
                    this.nodes.insert(node.data.id, node);
                }
                Err(e) => warn!(
                    "Failed to connect to node from file `{}`: {}",
                    path.display(),
                    e
                ),
            }
        }

        Ok(this)
    }

    pub async fn save(&self) -> Result<()> {
        // We only save the common data file. The individual node data files save themselves.
        info!(
            "Writing nodes common config file: {}",
            REGISTRY_CONFIG_FILE.display()
        );
        let config = toml::Value::try_from(&self.data)?;
        let config = toml::to_string(&config)?;
        fs::create_dir_all(REGISTRY_CONFIG_DIR.as_path()).await?;
        fs::write(&*REGISTRY_CONFIG_FILE, &*config).await?;

        Ok(())
    }

    pub fn exists() -> bool {
        Path::new(&*REGISTRY_CONFIG_FILE).exists()
    }

    pub fn send_node_status(
        &self,
        id: &Uuid,
        status: pb::node_info::ContainerStatus,
    ) -> Result<()> {
        if !self.tx.initialized() {
            bail!("Updates channel not initialized")
        }

        let node_id = id.to_string();
        self.tx.get().unwrap().send(pb::InfoUpdate {
            info: Some(pb::info_update::Info::Node(pb::NodeInfo {
                id: node_id,
                container_status: Some(status.into()),
                ..Default::default()
            })),
        })?;

        Ok(())
    }

    /// Create and return the next network interface using machine index
    pub async fn create_network_interface(
        &mut self,
        ip: IpAddr,
        gateway: IpAddr,
    ) -> Result<NetworkInterface> {
        self.data.machine_index += 1;

        let iface =
            NetworkInterface::create(format!("bv{}", self.data.machine_index), ip, gateway).await?;

        Ok(iface)
    }

    // Get or init updates sender
    pub async fn get_updates_sender(&self) -> Result<&Sender<pb::InfoUpdate>> {
        self.tx
            .get_or_try_init(|| async {
                let (tx, _rx) = broadcast::channel(128);
                Ok(tx)
            })
            .await
    }
}
