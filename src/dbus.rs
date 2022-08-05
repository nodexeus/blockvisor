use uuid::Uuid;
use zbus::{dbus_proxy, Result};

use crate::node_data::NodeData;

#[dbus_proxy(
    interface = "com.BlockJoy.blockvisor.Node",
    default_path = "/com/BlockJoy/blockvisor/Node",
    default_service = "com.BlockJoy.blockvisor"
)]
trait Node {
    async fn create(&self, id: &Uuid, name: &str, chain: &str) -> Result<()>;
    async fn delete(&self, id: &Uuid) -> Result<()>;
    async fn start(&self, id: &Uuid) -> Result<()>;
    async fn stop(&self, id: &Uuid) -> Result<()>;
    async fn list(&self) -> Result<Vec<NodeData>>;
    // TODO: Rest of the NodeCommand variants.
    async fn node_id_for_name(&self, name: &str) -> Result<Uuid>;
}
