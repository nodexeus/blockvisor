use babel_api::engine::Compression;
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    time::SystemTime,
};

/// S3 configuration for accessing snapshots
#[derive(Debug, Clone)]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
    pub endpoint_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub protocol: String,        // "arbitrum-one", "ethereum" 
    pub client: String,          // "nitro", "geth", "reth"
    pub network: String,         // "mainnet", "sepolia"
    pub node_type: String,       // "full", "archive"
    pub version: u64,            // 1, 2, 3, etc.
    pub total_size: u64,         // Size in bytes
    pub chunks: u32,             // Number of chunks
    pub compression: Option<Compression>,
    pub created_at: SystemTime,
    pub archive_uuid: String,    // UUID from API (e.g. "550e8400-e29b-41d4-a716-446655440000") 
    pub archive_id: String,      // "arbitrum-one-nitro-mainnet-full-v1"
    pub full_path: String,       // "arbitrum-one-nitro-mainnet-full-v1/2" (relative path without bucket)
}

#[derive(Debug, Clone)]
pub struct ProtocolGroup {
    pub protocol: String,
    pub clients: HashMap<String, ClientGroup>,
}

#[derive(Debug, Clone)]  
pub struct ClientGroup {
    pub client: String,
    pub networks: HashMap<String, Vec<SnapshotMetadata>>,
}

#[derive(Debug)]
pub struct ListFilter {
    pub protocol: Option<String>,
    pub client: Option<String>,
    pub network: Option<String>,
}

/// Configuration for snapshot downloads
#[derive(Debug)]
pub struct DownloadConfig {
    /// Number of concurrent worker tasks for processing chunks
    /// Each worker handles the complete processing of a chunk (download, decompress, write)
    pub workers: usize,
    
    /// Maximum number of concurrent HTTP connections
    /// This limits the number of simultaneous HTTP GET requests to prevent overwhelming the server
    /// Should typically be equal to or greater than workers for optimal performance
    pub max_connections: usize,
    
    /// Directory where downloaded files will be stored
    pub output_dir: PathBuf,
}

impl DownloadConfig {
    /// Validate the download configuration parameters
    pub fn validate(&self) -> Result<()> {
        if self.workers == 0 {
            return Err(eyre::anyhow!("Workers must be greater than 0"));
        }
        
        if self.max_connections == 0 {
            return Err(eyre::anyhow!("Max connections must be greater than 0"));
        }
        
        // Reasonable upper limits to prevent resource exhaustion
        const MAX_WORKERS: usize = 100;
        const MAX_CONNECTIONS: usize = 1000;
        
        if self.workers > MAX_WORKERS {
            return Err(eyre::anyhow!(
                "Workers ({}) exceeds maximum allowed ({})", 
                self.workers, MAX_WORKERS
            ));
        }
        
        if self.max_connections > MAX_CONNECTIONS {
            return Err(eyre::anyhow!(
                "Max connections ({}) exceeds maximum allowed ({})", 
                self.max_connections, MAX_CONNECTIONS
            ));
        }
        
        // Warn if max_connections is much lower than workers
        if self.max_connections < self.workers / 2 {
            eprintln!(
                "Warning: max_connections ({}) is significantly lower than workers ({}). \
                This may limit download performance.",
                self.max_connections, self.workers
            );
        }
        
        Ok(())
    }
    
    /// Create a new DownloadConfig with validation
    pub fn new(workers: usize, max_connections: usize, output_dir: PathBuf) -> Result<Self> {
        let config = Self {
            workers,
            max_connections,
            output_dir,
        };
        config.validate()?;
        Ok(config)
    }
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
pub struct SnapshotConfig {
    /// Client auth token (JWT)
    pub token: String,
    /// Refresh token for renewing expired tokens
    pub refresh_token: String,
    /// API endpoint URL
    pub api_url: String,
}

#[derive(Debug)]
pub struct DownloadInfo {
    pub protocol: String,
    pub client: String,
    pub network: String,
    pub node_type: String,
    pub version: u64,
    pub archive_id: String,
    pub full_path: String,
    pub total_size: u64,
    pub chunks: u32,
    pub compression: Option<Compression>,
}

#[derive(Debug)]
pub enum DownloadStatus {
    NotStarted,
    InProgress {
        progress_percent: u32,
        downloaded_bytes: u64,
        total_bytes: u64,
        speed_mbps: f64,
        eta_minutes: u32,
        chunks_complete: u32,
        total_chunks: u32,
    },
    Completed,
    Failed {
        error: String,
    },
}

impl SnapshotMetadata {
    /// Build archive ID from user-friendly components (latest version)
    pub fn build_archive_id(protocol: &str, client: &str, network: &str, node_type: &str) -> String {
        format!("{}-{}-{}-{}-v1", protocol, client, network, node_type)
    }
    
    /// Build archive path with version
    pub fn _build_full_path(archive_id: &str, version: u64) -> String {
        format!("{}/{}", archive_id, version)
    }
    
    /// Parse archive ID back to components  
    pub fn parse_archive_id(archive_id: &str) -> Result<(String, String, String, String)> {
        // Parse: "arbitrum-one-nitro-mainnet-full-v1"
        // Into: ("arbitrum-one", "nitro", "mainnet", "full")
        
        if !archive_id.ends_with("-v1") {
            return Err(eyre::anyhow!("Archive ID must end with -v1"));
        }
        
        let without_version = archive_id.trim_end_matches("-v1");
        let parts: Vec<&str> = without_version.split('-').collect();
        
        if parts.len() < 4 {
            return Err(eyre::anyhow!("Invalid archive ID format, need at least protocol-client-network-nodetype"));
        }
        
        // Work backwards from the known structure
        let node_type = parts[parts.len() - 1];
        let network = parts[parts.len() - 2]; 
        let client = parts[parts.len() - 3];
        let protocol = parts[0..parts.len() - 3].join("-");
        
        Ok((protocol, client.to_string(), network.to_string(), node_type.to_string()))
    }
    
    /// Parse archive path (without bucket)
    pub fn _parse_full_path(full_path: &str) -> Result<(String, u64)> {
        // Parse: "arbitrum-one-nitro-mainnet-full-v1/2"
        // Into: ("arbitrum-one-nitro-mainnet-full-v1", 2)
        
        let parts: Vec<&str> = full_path.split('/').collect();
        if parts.len() != 2 {
            return Err(eyre::anyhow!("Expected format: archive-id/version"));
        }
        
        let archive_id = parts[0].to_string();
        let version = parts[1].parse::<u64>()
            .map_err(|_| eyre::anyhow!("Invalid version number: {}", parts[1]))?;
            
        Ok((archive_id, version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_archive_id() {
        let archive_id = SnapshotMetadata::build_archive_id("arbitrum-one", "nitro", "mainnet", "full");
        assert_eq!(archive_id, "arbitrum-one-nitro-mainnet-full-v1");
    }

    #[test]
    fn test_parse_archive_id() {
        let (protocol, client, network, node_type) = 
            SnapshotMetadata::parse_archive_id("arbitrum-one-nitro-mainnet-full-v1").unwrap();
        
        assert_eq!(protocol, "arbitrum-one");
        assert_eq!(client, "nitro");
        assert_eq!(network, "mainnet");
        assert_eq!(node_type, "full");
    }

    #[test]
    fn test_parse_single_word_protocol() {
        let (protocol, client, network, node_type) = 
            SnapshotMetadata::parse_archive_id("ethereum-geth-mainnet-archive-v1").unwrap();
        
        assert_eq!(protocol, "ethereum");
        assert_eq!(client, "geth");
        assert_eq!(network, "mainnet");
        assert_eq!(node_type, "archive");
    }

    #[test]
    fn test_build_full_path() {
        let full_path = SnapshotMetadata::_build_full_path("arbitrum-one-nitro-mainnet-full-v1", 2);
        assert_eq!(full_path, "arbitrum-one-nitro-mainnet-full-v1/2");
    }

    #[test]
    fn test_parse_full_path() {
        let (archive_id, version) = 
            SnapshotMetadata::_parse_full_path("arbitrum-one-nitro-mainnet-full-v1/2").unwrap();
        
        assert_eq!(archive_id, "arbitrum-one-nitro-mainnet-full-v1");
        assert_eq!(version, 2);
    }

    #[test]
    fn test_download_config_validation() {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().to_path_buf();
        
        // Valid configuration should pass
        let valid_config = DownloadConfig::new(4, 8, output_dir.clone()).unwrap();
        assert_eq!(valid_config.workers, 4);
        assert_eq!(valid_config.max_connections, 8);
        
        // Zero workers should fail
        assert!(DownloadConfig::new(0, 4, output_dir.clone()).is_err());
        
        // Zero max_connections should fail
        assert!(DownloadConfig::new(4, 0, output_dir.clone()).is_err());
        
        // Excessive workers should fail
        assert!(DownloadConfig::new(200, 4, output_dir.clone()).is_err());
        
        // Excessive max_connections should fail
        assert!(DownloadConfig::new(4, 2000, output_dir.clone()).is_err());
        
        // Valid edge cases should pass
        assert!(DownloadConfig::new(1, 1, output_dir.clone()).is_ok());
        assert!(DownloadConfig::new(100, 1000, output_dir.clone()).is_ok());
    }
}