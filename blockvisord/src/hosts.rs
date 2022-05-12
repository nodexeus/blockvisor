use crate::containers::NodeContainer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct Host {
    pub containers: HashMap<String, Box<dyn NodeContainer>>,
    pub config: HostConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HostConfig {
    pub id: String,
    pub data_dir: String,
    pub pool_dir: String,
    pub token: String,
    pub blockjoy_api_url: String,
}
