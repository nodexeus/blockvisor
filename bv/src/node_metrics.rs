//! Here we have the code related to the metrics for nodes. We

use crate::grpc::pb;
use crate::node;
use std::collections::HashMap;

/// The max duration we will wait for a node to return a metric.
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

/// Type alias for a uuid that is the id of a node.
type NodeId = uuid::Uuid;

/// The metrics for a group of nodes.
#[derive(serde::Serialize)]
pub struct Metrics(HashMap<NodeId, Metric>);

/// The metrics for a single node.
#[derive(serde::Serialize)]
struct Metric {
    height: Option<u64>,
    block_age: Option<u64>,
    staking_status: Option<i32>,
    consensus: Option<bool>,
}

/// Given a list of nodes, returns for each node their metric. It does this concurrently for each
/// node, but queries the different metrics sequentially for a given node. Normally this would not
/// be efficient, but since we are dealing with a virtual socket the latency is very low, in the
/// hundres of nanoseconds. Furthermore, we require unique access to the node to query a metric, so
/// sequentially is easier to program.
pub async fn collect_metrics(nodes: impl Iterator<Item = &mut node::Node>) -> Metrics {
    let metrics_fut: Vec<_> = nodes.map(collect_metric).collect();
    let metrics: Vec<_> = futures_util::future::join_all(metrics_fut).await;
    Metrics(metrics.into_iter().collect())
}

/// Returns the metric for a single node.
async fn collect_metric(node: &mut node::Node) -> (NodeId, Metric) {
    let metric = Metric {
        height: timeout(node.height()).await.ok(),
        block_age: timeout(node.block_age()).await.ok(),
        staking_status: timeout(node.stake_status()).await.ok(),
        consensus: timeout(node.consensus()).await.ok(),
    };
    (node.id(), metric)
}

async fn timeout<F, T>(fut: F) -> anyhow::Result<T>
where
    F: std::future::Future<Output = anyhow::Result<T>>,
{
    tokio::time::timeout(TIMEOUT, fut).await?
}

impl From<Metrics> for pb::NodeMetricsRequest {
    fn from(metrics: Metrics) -> Self {
        let metrics = metrics
            .0
            .into_iter()
            .map(|(k, v)| {
                (
                    k.to_string(),
                    pb::Metrics {
                        height: v.height,
                        block_age: v.block_age,
                        staking_status: v.staking_status,
                        consensus: v.consensus,
                    },
                )
            })
            .collect();
        Self { metrics }
    }
}
