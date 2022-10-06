use anyhow::{anyhow, bail, Context, Result};
use futures_util::TryFutureExt;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::{collections::HashMap, sync::Arc};
use tokio::fs::{self, read_dir};
use tokio::sync::broadcast::{self, Sender};
use tokio::sync::OnceCell;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;
use zbus::{Connection, ConnectionBuilder};

use crate::{
    grpc::pb,
    network_interface::NetworkInterface,
    node::Node,
    node_data::{NodeData, NodeStatus},
};

const NODES_CONFIG_FILENAME: &str = "nodes.toml";
const BABEL_BUS_ADDRESS: &str = "unix:path=/var/lib/blockvisor/vsock.socket_42";

lazy_static::lazy_static! {
    pub static ref REGISTRY_CONFIG_DIR: PathBuf = home::home_dir()
        .map(|p| p.join(".cache"))
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("blockvisor");
}
lazy_static::lazy_static! {
    static ref REGISTRY_CONFIG_FILE: PathBuf = REGISTRY_CONFIG_DIR.join(NODES_CONFIG_FILENAME);
}

#[derive(Clone, Debug)]
pub enum ServiceStatus {
    Enabled,
    Disabled,
}

#[derive(Debug, Default)]
pub struct Nodes {
    pub nodes: HashMap<Uuid, Node>,
    pub node_ids: HashMap<String, Uuid>,
    babel_conn: OnceCell<Connection>,
    data: CommonData,
    tx: OnceCell<Sender<pb::InfoUpdate>>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
struct CommonData {
    machine_index: Arc<AtomicU32>,
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

        let network_interface = self.next_network_interface().await?;
        let node = NodeData {
            id,
            name: name.clone(),
            chain,
            network_interface,
        };

        let babel_conn = self.babel_conn().await?;
        let node = Node::create(node, babel_conn).await?;
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
        let mut this = Nodes {
            data: nodes_data,
            ..Default::default()
        };
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
                    let babel_conn = this.babel_conn().await?;
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

    /// Get the next machine index and increment it.
    pub async fn next_network_interface(&self) -> Result<NetworkInterface> {
        let machine_index = self.data.machine_index.fetch_add(1, Ordering::SeqCst);

        let idx_bytes = machine_index.to_be_bytes();
        let iface = NetworkInterface::create(
            format!("bv{}", machine_index),
            // FIXME: Hardcoding address for now.
            IpAddr::V4(Ipv4Addr::new(
                idx_bytes[0] + 74,
                idx_bytes[1] + 50,
                idx_bytes[2] + 82,
                idx_bytes[3] + 83,
            )),
        )
        .await?;
        self.save().await?;

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

    async fn babel_conn(&self) -> Result<&Connection> {
        self.babel_conn
            .get_or_try_init(|| async {
                ConnectionBuilder::address(BABEL_BUS_ADDRESS)?.build().await
            })
            .await
            .context("Failed to connect to babel bus")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn network_interface_gen() {
        let nodes = Nodes::default();
        clean_test_iface(&nodes, "bv0", &IpAddr::V4(Ipv4Addr::new(74, 50, 82, 83))).await;
        clean_test_iface(&nodes, "bv1", &IpAddr::V4(Ipv4Addr::new(74, 50, 82, 84))).await;
        // Let's take the machine_index beyond u8 boundry.
        nodes
            .data
            .machine_index
            .store(u8::MAX as u32 + 1, Ordering::SeqCst);
        let iface_name = format!("bv{}", u8::MAX as u32 + 1);
        clean_test_iface(
            &nodes,
            &iface_name,
            &IpAddr::V4(Ipv4Addr::new(74, 50, 83, 83)),
        )
        .await;
    }

    async fn clean_test_iface(nodes: &Nodes, name: &str, ip: &IpAddr) {
        // Make sure the interface doesn't exist already.
        let _ = crate::network_interface::NetworkInterface {
            name: name.to_owned(),
            ip: ip.to_owned(),
        }
        .delete()
        .await;

        let iface = nodes.next_network_interface().await.unwrap();
        let next_name = iface.name.clone();
        let next_ip = iface.ip.clone();

        // Clean up
        let _ = iface.delete().await;

        assert_eq!(&next_name, name);
        assert_eq!(&next_ip, ip);
    }
}
