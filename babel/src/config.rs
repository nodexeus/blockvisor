use serde::Deserialize;
use std::{collections::BTreeMap, path::Path};
use tokio::fs;

#[derive(Debug, Deserialize)]
pub struct Babel {
    pub urn: String,
    pub export: Option<Vec<String>>,
    pub env: Option<Env>,
    pub config: Config,
    pub monitor: Option<Monitor>,
    #[serde(deserialize_with = "deserialize_methods")]
    pub methods: BTreeMap<String, Method>,
}

pub fn deserialize_methods<'de, D>(deserializer: D) -> Result<BTreeMap<String, Method>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let methods = Vec::<Method>::deserialize(deserializer)?;
    let map = methods
        .into_iter()
        .map(|m| (m.name().to_string(), m))
        .collect();

    Ok(map)
}

#[derive(Debug, Deserialize)]
pub struct Env {
    pub path_append: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub babel_version: String,
    pub node_version: String,
    pub node_type: String,
    pub description: Option<String>,
    pub api_host: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Monitor {
    pub pid_file: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "transport", rename_all = "kebab-case")]
pub enum Method {
    Jrpc {
        name: String,
        method: String,
        response: JrpcResponse,
    },
    Rest {
        name: String,
        method: String,
        response: RestResponse,
    },
    Sh {
        name: String,
        body: String,
        response: ShResponse,
    },
}

impl Method {
    pub fn name(&self) -> &str {
        match self {
            Method::Jrpc { name, .. } => name,
            Method::Rest { name, .. } => name,
            Method::Sh { name, .. } => name,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct JrpcResponse {
    pub code: u32,
    pub field: String,
}

#[derive(Debug, Deserialize)]
pub struct RestResponse {
    pub status: u32,
    pub field: String,
    pub format: MethodResponseFormat,
}

#[derive(Debug, Deserialize)]
pub struct ShResponse {
    pub status: i32,
    pub format: MethodResponseFormat,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MethodResponseFormat {
    Raw,
    Json,
}

pub async fn load(path: &str) -> eyre::Result<Babel> {
    let path = Path::new(&path);
    let toml_str = fs::read_to_string(path).await?;

    let cfg = toml::from_str(&toml_str)?;

    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load() {
        let _ = load("examples/pokt.toml").await.unwrap();
        let _ = load("examples/solana.toml").await.unwrap();
        let _ = load("examples/avax.toml").await.unwrap();
        let _ = load("examples/cosmos.toml").await.unwrap();
        let _ = load("examples/eth.toml").await.unwrap();
        let _ = load("examples/near.toml").await.unwrap();
        let _ = load("examples/helium.toml").await.unwrap();
        let _ = load("examples/pedge.toml").await.unwrap();
        let _ = load("examples/cardano.toml").await.unwrap();
        let _ = load("examples/casper.toml").await.unwrap();
        let _ = load("examples/kava.toml").await.unwrap();
        let _ = load("examples/moonbeam.toml").await.unwrap();
        let _ = load("examples/olabs.toml").await.unwrap();
        let _ = load("examples/osmosis.toml").await.unwrap();
        let _ = load("examples/polkadot.toml").await.unwrap();
        let _ = load("examples/sentinel.toml").await.unwrap();
        let _ = load("examples/tezos.toml").await.unwrap();
        let _ = load("examples/algo.toml").await.unwrap(); // new blockchains from here
        let _ = load("examples/band.toml").await.unwrap();
        let _ = load("examples/chainlink.toml").await.unwrap();
        let _ = load("examples/edgeware.toml").await.unwrap();
        let _ = load("examples/iris.toml").await.unwrap();
        let _ = load("examples/juno.toml").await.unwrap();
        let _ = load("examples/flow.toml").await.unwrap();
    }
}
