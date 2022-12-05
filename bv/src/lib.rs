pub mod babel_connection;
pub mod cli;
pub mod config;
pub mod env;
pub mod grpc;
pub mod hosts;
pub mod installer;
pub mod key_service;
pub mod logging;
pub mod network_interface;
pub mod node;
pub mod node_data;
pub mod node_metrics;
pub mod nodes;
pub mod pretty_table;
pub mod server;
pub mod utils;

use tokio::sync::RwLock;

lazy_static::lazy_static! {
    pub static ref BV_STATUS: RwLock<server::bv_pb::ServiceStatus> = RwLock::new(server::bv_pb::ServiceStatus::UndefinedServiceStatus);
}
