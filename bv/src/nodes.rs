use anyhow::{anyhow, bail, Result};
use futures_util::TryFutureExt;
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::{collections::HashMap, sync::Arc};
use tokio::fs::{self, read_dir};
use tokio::sync::broadcast::{self, Sender};
use tokio::sync::OnceCell;
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
        .join("blockvisor");

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

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct CommonData {
    pub machine_index: Arc<AtomicU32>,
    pub ip_range_from: IpAddr,
    pub ip_range_to: IpAddr,
    pub ip_gateway: IpAddr,
}

impl Nodes {
    #[instrument(skip(self))]
    pub async fn create(&mut self, id: Uuid, name: String, chain: String) -> Result<()> {
        if self.nodes.contains_key(&id) {
            bail!(format!("Node with id `{}` exists", &id));
        }

        if self.node_ids.contains_key(&name) {
            bail!(format!("Node with name `{}` exists", &name));
        }

        if let Err(error) = self.send_node_status(&id, pb::node_info::ContainerStatus::Creating) {
            error!("Cannot send node status: {error:?}");
        };

        self.data.machine_index.fetch_add(1, Ordering::SeqCst);
        let network_interface = self.create_network_interface().await?;
        let node = NodeData {
            id,
            name: name.clone(),
            chain,
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
        dbg!(backtrace::Backtrace::new());
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

        node.status().await
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
                    let babel_conn = Node::conn(data.id).await?;
                    Node::connect(data, babel_conn).await
                })
                .await
            {
                Ok(node) => {
                    this.node_ids.insert(node.data.name.clone(), *node.id());
                    this.nodes.insert(node.data.id, node);
                }
                Err(e) => warn!("Failed to read node file `{}`: {}", path.display(), e),
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
    pub async fn create_network_interface(&self) -> Result<NetworkInterface> {
        let machine_index = self.data.machine_index.load(Ordering::SeqCst);
        let idx_bytes = machine_index.to_be_bytes();
        let octets = match self.data.ip_range_from {
            IpAddr::V4(v4) => v4.octets(),
            IpAddr::V6(_) => unimplemented!(),
        };
        let ip = IpAddr::V4(Ipv4Addr::new(
            idx_bytes[0] + octets[0],
            idx_bytes[1] + octets[1],
            idx_bytes[2] + octets[2],
            idx_bytes[3] + octets[3],
        ));

        let iface =
            NetworkInterface::create(format!("bv{}", machine_index), ip, self.data.ip_gateway)
                .await?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn network_interface_gen() {
        let gw = IpAddr::V4(Ipv4Addr::new(216, 18, 214, 193));
        let nodes_data = CommonData {
            machine_index: Arc::new(AtomicU32::new(0)),
            ip_range_from: IpAddr::V4(Ipv4Addr::new(216, 18, 214, 195)),
            ip_range_to: IpAddr::V4(Ipv4Addr::new(216, 18, 214, 206)),
            ip_gateway: gw.clone(),
        };
        let nodes = Nodes::new(nodes_data);
        clean_test_iface(
            &nodes,
            "bv1",
            &IpAddr::V4(Ipv4Addr::new(216, 18, 214, 196)),
            &gw,
        )
        .await;
        clean_test_iface(
            &nodes,
            "bv2",
            &IpAddr::V4(Ipv4Addr::new(216, 18, 214, 197)),
            &gw,
        )
        .await;
        // Let's take the machine_index beyond u8 boundry.
        nodes
            .data
            .machine_index
            .store(u8::MAX as u32, Ordering::SeqCst);
        let iface_name = format!("bv{}", u8::MAX as u32 + 1);
        clean_test_iface(
            &nodes,
            &iface_name,
            &IpAddr::V4(Ipv4Addr::new(216, 18, 215, 195)),
            &gw,
        )
        .await;
    }

    async fn clean_test_iface(nodes: &Nodes, name: &str, ip: &IpAddr, gw: &IpAddr) {
        // Make sure the interface doesn't exist already.
        let _ = crate::network_interface::NetworkInterface {
            name: name.to_owned(),
            ip: ip.to_owned(),
            gw: gw.to_owned(),
        }
        .delete()
        .await;

        nodes.data.machine_index.fetch_add(1, Ordering::SeqCst);
        let iface = nodes.create_network_interface().await.unwrap();
        let next_name = iface.name.clone();
        let next_ip = iface.ip.clone();

        // Clean up
        let _ = iface.delete().await;

        assert_eq!(&next_name, name);
        assert_eq!(&next_ip, ip);
    }
}
