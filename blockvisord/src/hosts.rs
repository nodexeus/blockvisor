use crate::containers::NodeContainer;
use std::path::Path;

/// Host.
pub struct Host {
    containers: Vec<NodeContainer>,
    data_dir: Path,
    pool_dir: Path,
}
