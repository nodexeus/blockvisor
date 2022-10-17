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
    pub field: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RestResponse {
    pub status: u32,
    pub field: Option<String>,
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

pub async fn load(path: &Path) -> eyre::Result<Babel> {
    let toml_str = fs::read_to_string(path).await?;

    let cfg = toml::from_str(&toml_str)?;

    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load() {
        let mut dir = fs::read_dir("examples").await.unwrap();
        while let Some(entry) = dir.next_entry().await.unwrap() {
            println!("loading: {entry:?}");
            load(&entry.path()).await.unwrap();
        }
    }
}
