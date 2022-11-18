use serde::{Deserialize, Serialize};

/// Each request that comes over the VSock to babel must be a piece of JSON that can be
/// deserialized into this struct.
#[derive(Debug, Serialize, Deserialize)]
pub enum BabelRequest {
    /// List the endpoints that are available for the current blockchain. These are extracted from
    /// the config, and just sent back as strings for now.
    ListCapabilities,
    /// List of logs from blockchain entry_points.
    Logs,
    /// Returns `Pong`. Useful to check for the liveness of the node.
    Ping,
    /// Send a request to the current blockchain. We can identify the way to do this from the
    /// config and forward the provided parameters.
    BlockchainCommand(BlockchainCommand),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainCommand {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BabelResponse {
    ListCapabilities(Vec<String>),
    Pong,
    Logs(Vec<String>),
    BlockchainResponse(BlockchainResponse),
    Error(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainResponse {
    pub value: String,
}

impl From<String> for BlockchainResponse {
    fn from(value: String) -> Self {
        Self { value }
    }
}
