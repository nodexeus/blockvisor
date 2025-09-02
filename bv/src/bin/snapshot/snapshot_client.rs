use super::api_service::ApiService;
use super::download_engine::SnapshotDownloader;
use super::protocol_resolver::ProtocolResolver;
use super::types::*;
use blockvisord::services::api::pb;
use eyre::{anyhow, bail, Context, Result};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, info};

const CONFIG_FILENAME: &str = ".bv-snapshot.json";

impl SnapshotConfig {
    pub async fn load() -> Result<Self> {
        let path = homedir::my_home()?
            .ok_or_else(|| anyhow!("can't get home directory"))?
            .join(CONFIG_FILENAME);

        if !path.exists() {
            bail!("You are not logged-in yet, please run `bv-snapshot auth login` first.");
        }

        debug!("Reading snapshot config: {}", path.display());
        let config_content = fs::read_to_string(&path)
            .await
            .with_context(|| format!("failed to read snapshot config: {}", path.display()))?;

        let config: Self = serde_json::from_str(&config_content)
            .with_context(|| format!("failed to parse snapshot config: {}", path.display()))?;

        Ok(config)
    }

    pub async fn save(&self) -> Result<()> {
        let path = homedir::my_home()?
            .ok_or_else(|| anyhow!("can't get home directory"))?
            .join(CONFIG_FILENAME);

        debug!("Writing snapshot config: {}", path.display());
        let config_content = serde_json::to_string_pretty(self)?;

        fs::write(&path, config_content)
            .await
            .with_context(|| format!("failed to save snapshot config: {}", path.display()))?;

        Ok(())
    }

    pub async fn exists() -> bool {
        if let Ok(Some(home)) = homedir::my_home() {
            home.join(CONFIG_FILENAME).exists()
        } else {
            false
        }
    }

    pub async fn remove() -> Result<()> {
        let path = homedir::my_home()?
            .ok_or_else(|| anyhow!("can't get home directory"))?
            .join(CONFIG_FILENAME);

        if path.exists() {
            fs::remove_file(&path)
                .await
                .with_context(|| format!("failed to remove config: {}", path.display()))?;
        }

        Ok(())
    }
}

pub struct SnapshotClient {
    config: Option<SnapshotConfig>,
    protocol_resolver: ProtocolResolver,
}

impl SnapshotClient {
    pub async fn new() -> Result<Self> {
        let config = if SnapshotConfig::exists().await {
            Some(SnapshotConfig::load().await?)
        } else {
            None
        };

        Ok(Self {
            config,
            protocol_resolver: ProtocolResolver::new(),
        })
    }

    /// Handle login authentication - similar to NIB's approach
    pub async fn login(&mut self, api_url: String) -> Result<()> {
        // Prompt for email and password
        print!("Email: ");
        std::io::stdout().flush()?;
        let mut email = String::new();
        std::io::stdin().lock().read_line(&mut email)?;
        let password = rpassword::prompt_password("Password: ")?;

        info!("Authenticating with {}...", api_url);

        // Step 1: Login to get auth token
        let mut auth_client =
            pb::auth_service_client::AuthServiceClient::connect(api_url.clone()).await?;
        let login_response = auth_client
            .login(pb::AuthServiceLoginRequest {
                email: email.trim().to_string(),
                password,
            })
            .await?
            .into_inner();

        // Step 2: Save both JWT token and refresh token
        let config = SnapshotConfig {
            token: login_response.token,
            refresh_token: login_response.refresh,
            api_url,
        };

        config.save().await?;
        self.config = Some(config);

        println!("✓ Authentication successful! Config saved to ~/.bv-snapshot.json");

        Ok(())
    }

    pub async fn auth_status(&self) -> Result<String> {
        if let Some(config) = &self.config {
            // TODO: Could make a test API call to verify token is still valid
            Ok(format!("Authenticated with {}", config.api_url))
        } else {
            Ok("Not authenticated. Run `bv-snapshot auth login` first.".to_string())
        }
    }

    pub async fn logout(&mut self) -> Result<()> {
        SnapshotConfig::remove().await?;
        self.config = None;
        Ok(())
    }

    /// List available snapshots with filtering
    pub async fn list_snapshots(&self, filter: ListFilter) -> Result<Vec<ProtocolGroup>> {
        let config = self.require_auth()?;

        // Create API service and discover snapshots
        let api_service = ApiService::new(config.clone());
        let mut groups = api_service.discover_snapshots().await?;

        // Apply filters
        if let Some(protocol_filter) = &filter.protocol {
            let normalized = self.protocol_resolver.normalize_protocol(protocol_filter)?;
            groups.retain(|g| g.protocol == normalized);
        }

        if let Some(client_filter) = &filter.client {
            for group in &mut groups {
                group.clients.retain(|name, _| name == client_filter);
            }
            groups.retain(|g| !g.clients.is_empty());
        }

        if let Some(network_filter) = &filter.network {
            for group in &mut groups {
                for client in group.clients.values_mut() {
                    client.networks.retain(|name, _| name == network_filter);
                }
                group
                    .clients
                    .retain(|_, client| !client.networks.is_empty());
            }
            groups.retain(|g| !g.clients.is_empty());
        }

        Ok(groups)
    }

    /// Get download information without actually downloading
    pub async fn get_download_info(
        &self,
        protocol: &str,
        client: &str,
        network: &str,
        node_type: &str,
        version: Option<u64>,
    ) -> Result<DownloadInfo> {
        let config = self.require_auth()?;

        let normalized_protocol = self.protocol_resolver.normalize_protocol(protocol)?;
        let archive_id =
            SnapshotMetadata::build_archive_id(&normalized_protocol, client, network, node_type);

        // Use API service to get real metadata
        let api_service = ApiService::new(config.clone());
        
        // If no version specified, get metadata to determine the latest version
        let (target_version, full_path) = if let Some(v) = version {
            // Use the specified version
            (v, format!("{}/{}", archive_id, v))
        } else {
            // Get metadata without version to fetch the latest
            let latest_info = api_service.get_download_metadata_by_store_key(&archive_id).await?;
            (latest_info.version, format!("{}/{}", archive_id, latest_info.version))
        };
        
        let mut download_info = api_service.get_download_metadata_by_store_key(&full_path).await?;

        // Override version info
        download_info.version = target_version;
        download_info.full_path = full_path;

        Ok(download_info)
    }

    /// Download a snapshot
    pub async fn download_snapshot(
        &self,
        protocol: &str,
        client: &str,
        network: &str,
        node_type: &str,
        version: Option<u64>,
        config: DownloadConfig,
    ) -> Result<()> {
        let auth_config = self.require_auth()?;

        let normalized_protocol = self.protocol_resolver.normalize_protocol(protocol)?;
        let archive_id =
            SnapshotMetadata::build_archive_id(&normalized_protocol, client, network, node_type);

        // Get the latest version if not specified
        let api_service = ApiService::new(auth_config.clone());
        let (target_version, full_path) = if let Some(v) = version {
            // Use the specified version
            (v, format!("{}/{}", archive_id, v))
        } else {
            // Get metadata without version to fetch the latest
            let latest_info = api_service.get_download_metadata_by_store_key(&archive_id).await?;
            (latest_info.version, format!("{}/{}", archive_id, latest_info.version))
        };

        println!("Starting download: {} (version {})", archive_id, target_version);
        println!("Output directory: {}", config.output_dir.display());
        println!(
            "Workers: {}, Max connections: {}",
            config.workers, config.max_connections
        );

        // Create and use the download engine
        let downloader = SnapshotDownloader::new(auth_config.clone(), config)?;
        downloader.download(&full_path).await?;

        Ok(())
    }

    /// Resume an interrupted download
    pub async fn resume_download(&self, output_dir: PathBuf) -> Result<()> {
        let auth_config = self.require_auth()?;

        println!("Resuming download in: {}", output_dir.display());

        // Create download config for resuming
        let download_config = DownloadConfig {
            workers: 4, // Default values for resume
            max_connections: 4,
            output_dir,
        };

        let downloader = SnapshotDownloader::new(auth_config.clone(), download_config)?;
        downloader.resume().await?;

        Ok(())
    }

    /// Get download status
    pub async fn get_status(&self, output_dir: Option<PathBuf>) -> Result<DownloadStatus> {
        let auth_config = self.require_auth()?;

        if let Some(dir) = output_dir {
            // Create a temporary download config for status checking
            let download_config = DownloadConfig {
                workers: 4,
                max_connections: 4,
                output_dir: dir,
            };

            let downloader = SnapshotDownloader::new(auth_config.clone(), download_config)?;
            downloader.get_status().await
        } else {
            Ok(DownloadStatus::NotStarted)
        }
    }

    fn require_auth(&self) -> Result<&SnapshotConfig> {
        self.config
            .as_ref()
            .ok_or_else(|| anyhow!("Not authenticated. Please run `bv-snapshot auth login` first."))
    }

    /// Refresh the JWT token using the refresh token
    async fn refresh_token(&mut self) -> Result<()> {
        let config = self.require_auth()?.clone();

        info!("Refreshing expired token...");

        let mut auth_client =
            pb::auth_service_client::AuthServiceClient::connect(config.api_url.clone()).await?;
        let refresh_response = auth_client
            .refresh(pb::AuthServiceRefreshRequest {
                token: config.token,
                refresh: Some(config.refresh_token),
            })
            .await?
            .into_inner();

        // Update config with new tokens
        let updated_config = SnapshotConfig {
            token: refresh_response.token,
            refresh_token: refresh_response.refresh,
            api_url: config.api_url,
        };

        updated_config.save().await?;
        self.config = Some(updated_config);

        info!("✓ Token refreshed successfully");
        Ok(())
    }

    // Mock data for development - replace with actual API calls
    fn create_mock_data(&self) -> Result<Vec<ProtocolGroup>> {
        use std::time::SystemTime;

        let mut groups = Vec::new();

        // Ethereum group
        let mut ethereum_clients = HashMap::new();

        let mut geth_networks = HashMap::new();
        geth_networks.insert(
            "mainnet".to_string(),
            vec![SnapshotMetadata {
                protocol: "ethereum".to_string(),
                client: "geth".to_string(),
                network: "mainnet".to_string(),
                node_type: "archive".to_string(),
                version: 15,
                total_size: 1_200_000_000_000,
                chunks: 2400,
                compression: Some(babel_api::engine::Compression::ZSTD(5)),
                created_at: SystemTime::now(),
                archive_uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
                archive_id: "ethereum-geth-mainnet-archive-v1".to_string(),
                full_path: "ethereum-geth-mainnet-archive-v1/15".to_string(),
            }],
        );

        ethereum_clients.insert(
            "geth".to_string(),
            ClientGroup {
                client: "geth".to_string(),
                networks: geth_networks,
            },
        );

        groups.push(ProtocolGroup {
            protocol: "ethereum".to_string(),
            clients: ethereum_clients,
        });

        // Arbitrum One group
        let mut arbitrum_clients = HashMap::new();

        let mut nitro_networks = HashMap::new();
        nitro_networks.insert(
            "mainnet".to_string(),
            vec![SnapshotMetadata {
                protocol: "arbitrum-one".to_string(),
                client: "nitro".to_string(),
                network: "mainnet".to_string(),
                node_type: "full".to_string(),
                version: 3,
                total_size: 800_000_000_000,
                chunks: 1600,
                compression: Some(babel_api::engine::Compression::ZSTD(5)),
                created_at: SystemTime::now(),
                archive_uuid: "660e8400-e29b-41d4-a716-446655440001".to_string(),
                archive_id: "arbitrum-one-nitro-mainnet-full-v1".to_string(),
                full_path: "arbitrum-one-nitro-mainnet-full-v1/3".to_string(),
            }],
        );

        arbitrum_clients.insert(
            "nitro".to_string(),
            ClientGroup {
                client: "nitro".to_string(),
                networks: nitro_networks,
            },
        );

        groups.push(ProtocolGroup {
            protocol: "arbitrum-one".to_string(),
            clients: arbitrum_clients,
        });

        Ok(groups)
    }
}
