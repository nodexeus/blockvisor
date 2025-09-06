use super::{protocol_resolver::ProtocolResolver, types::*};
use base64::Engine;
use blockvisord::services::api::{common, pb};
use blockvisord::services::{ApiInterceptor, AuthToken, DEFAULT_API_REQUEST_TIMEOUT};
use bv_utils::rpc::DefaultTimeout;
use eyre::{anyhow, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;
use tonic::transport::Endpoint;
use tracing::{debug, info, warn};

#[derive(Debug, Deserialize)]
struct JwtClaims {
    org_id: Option<String>,
    // Add other fields as needed, but we only need org_id for now
}

pub struct ApiService {
    config: Arc<Mutex<SnapshotConfig>>,
    protocol_resolver: ProtocolResolver,
}

impl ApiService {
    pub fn new(config: SnapshotConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            protocol_resolver: ProtocolResolver::new(),
        }
    }

    /// List all available snapshots by discovering protocols, variants, and archives
    pub async fn discover_snapshots(&self) -> Result<Vec<ProtocolGroup>> {
        info!("Discovering available snapshots...");
        
        // Step 1: List all protocols
        let protocols = self.list_protocols().await?;
        debug!("Found {} protocols", protocols.len());
        
        let mut protocol_groups = Vec::new();
        
        for protocol in protocols {
            // Step 2: For each protocol, list variants
            let variants = self.list_variants(&protocol.protocol_id).await?;
            debug!("Protocol {} has {} variants", protocol.key, variants.len());
            
            let mut clients = HashMap::new();
            
            for variant in variants {
                // Step 3: For each variant, get the image and archives
                let version_key = common::ProtocolVersionKey {
                    protocol_key: protocol.key.clone(),
                    variant_key: variant.clone(),
                };
                
                // Get the latest image for this protocol/variant
                if let Ok(image) = self.get_image(&version_key).await {
                    // Step 4: List archives for this image
                    if let Ok(archives) = self.list_archives(&image.image_id).await {
                        debug!("Image {} has {} archives", image.image_id, archives.len());
                        
                        for archive in archives {
                            debug!("Archive object: archive_id={}, store_key={}", archive.archive_id, archive.store_key);
                            // Parse the archive store_key to extract network and node_type
                            if let Ok((parsed_protocol, parsed_client, parsed_network, parsed_node_type)) = 
                                self.parse_store_key(&archive.store_key) {
                                
                                // Group by client (variant)
                                let client_group = clients
                                    .entry(parsed_client.clone())
                                    .or_insert_with(|| ClientGroup {
                                        client: parsed_client.clone(),
                                        networks: HashMap::new(),
                                    });
                                
                                                // Try to fetch multiple versions (try versions 1-5 to see what exists)
                                let mut snapshots = Vec::new();
                                
                                // Try to fetch versions 1 through 5 (most archives won't have more than this)
                                for version in 1..=5 {
                                    match self.fetch_download_metadata_with_version(&archive.archive_id, Some(version)).await {
                                        Ok(metadata) => {
                                            let full_path = format!("{}/{}", archive.store_key, version);
                                            
                                            let snapshot = SnapshotMetadata {
                                                protocol: parsed_protocol.clone(),
                                                client: parsed_client.clone(),
                                                network: parsed_network.clone(),
                                                node_type: parsed_node_type.clone(),
                                                version,
                                                total_size: metadata.total_size,
                                                chunks: metadata.chunks,
                                                compression: metadata.compression,
                                                created_at: SystemTime::UNIX_EPOCH, // Placeholder - actual metadata comes from download
                                                archive_uuid: archive.archive_id.clone(),
                                                archive_id: archive.store_key.clone(),
                                                full_path,
                                            };
                                            snapshots.push(snapshot);
                                        },
                                        Err(e) => {
                                            debug!("Version {} not found for archive {}: {}", version, archive.archive_id, e);
                                        }
                                    }
                                }
                                
                                if !snapshots.is_empty() {
                                    client_group.networks.insert(parsed_network, snapshots);
                                }
                            }
                        }
                    }
                }
            }
            
            if !clients.is_empty() {
                protocol_groups.push(ProtocolGroup {
                    protocol: protocol.key,
                    clients,
                });
            }
        }
        
        Ok(protocol_groups)
    }

    /// Get download metadata for a snapshot (using its UUID)
    pub async fn get_download_metadata(&self, snapshot: &SnapshotMetadata) -> Result<DownloadInfo> {
        info!("Getting download metadata for archive: {} version {}", snapshot.archive_id, snapshot.version);
        
        let metadata = self.fetch_download_metadata_with_version(&snapshot.archive_uuid, Some(snapshot.version)).await?;
        
        Ok(DownloadInfo {
            protocol: snapshot.protocol.clone(),
            client: snapshot.client.clone(),
            network: snapshot.network.clone(),
            node_type: snapshot.node_type.clone(),
            version: snapshot.version,
            archive_id: snapshot.archive_id.clone(),
            full_path: snapshot.full_path.clone(),
            total_size: metadata.total_size,
            chunks: metadata.chunks,
            compression: metadata.compression,
        })
    }

    /// Find and get download metadata by store_key (requires discovery first)
    pub async fn get_download_metadata_by_store_key(&self, store_key: &str) -> Result<DownloadInfo> {
        info!("Looking up download metadata for store key: {}", store_key);
        
        // Parse the store_key to separate base archive_id from version
        let (base_archive_id, requested_version) = if store_key.contains('/') {
            let parts: Vec<&str> = store_key.splitn(2, '/').collect();
            let base_id = parts[0];
            let version = parts[1].parse::<u64>().map_err(|_| anyhow!("Invalid version in store key: {}", store_key))?;
            (base_id, Some(version))
        } else {
            (store_key, None)
        };
        
        // First discover all snapshots to find the UUID for this store_key
        let protocol_groups = self.discover_snapshots().await?;
        
        // Search through all snapshots to find the matching base archive_id
        for group in protocol_groups {
            debug!("Searching protocol group: {}", group.protocol);
            for (_, client_group) in group.clients {
                debug!("  Searching client group: {}", client_group.client);
                for (network, snapshots) in &client_group.networks {
                    debug!("    Searching network: {}", network);
                    
                    // Check if any snapshot in this network matches our base_archive_id
                    let matching_snapshots: Vec<&SnapshotMetadata> = snapshots.iter()
                        .filter(|s| s.archive_id == base_archive_id)
                        .collect();
                    
                    if !matching_snapshots.is_empty() {
                        debug!("      MATCH! Base archive_id found: {}", base_archive_id);
                        
                        // If a specific version was requested, find that version
                        if let Some(version) = requested_version {
                            // Find the specific version in the snapshots
                            for snap in &matching_snapshots {
                                if snap.version == version {
                                    debug!("      Found requested version {}: {}", version, snap.archive_uuid);
                                    return self.get_download_metadata(snap).await;
                                }
                            }
                            return Err(anyhow!("Version {} not found for archive '{}'", version, base_archive_id));
                        } else {
                            // No specific version requested, find the latest version
                            let latest_snapshot = matching_snapshots.iter()
                                .max_by_key(|s| s.version)
                                .ok_or_else(|| anyhow!("No versions found for archive '{}'", base_archive_id))?;
                            
                            debug!("      Using latest version {}: {}", latest_snapshot.version, latest_snapshot.archive_uuid);
                            return self.get_download_metadata(latest_snapshot).await;
                        }
                    }
                }
            }
        }
        
        Err(anyhow!("Archive with store key '{}' not found", store_key))
    }

    /// Get download chunk URLs for an archive
    pub async fn get_download_chunks(&self, archive_uuid: &str, data_version: u64, chunk_indexes: Vec<u32>) -> Result<Vec<pb::ArchiveChunk>> {
        info!("Getting download chunks for archive UUID: {}", archive_uuid);

        let config = self.config.lock().await;
        let api_url = config.api_url.clone();
        drop(config);

        let archive_uuid = archive_uuid.to_string();

        let response = self.with_token_refresh(|token| {
            let api_url = api_url.clone();
            let archive_uuid = archive_uuid.clone();
            let chunk_indexes = chunk_indexes.clone();
            Box::pin(async move {
                let endpoint = Endpoint::from_shared(api_url)
                    .map_err(|e| tonic::Status::invalid_argument(format!("Invalid API URL: {}", e)))?;
                let channel = endpoint.connect().await
                    .map_err(|e| tonic::Status::unavailable(format!("Failed to connect: {}", e)))?;
                
                let mut client = pb::archive_service_client::ArchiveServiceClient::with_interceptor(
                    channel,
                    ApiInterceptor(
                        AuthToken(token),
                        DefaultTimeout(DEFAULT_API_REQUEST_TIMEOUT),
                    ),
                );
                
                let response = client
                    .get_download_chunks(pb::ArchiveServiceGetDownloadChunksRequest {
                        archive_id: archive_uuid,
                        org_id: None,
                        data_version,
                        chunk_indexes,
                    })
                    .await?
                    .into_inner();
                
                Ok(response)
            })
        }).await?;

        // Return the protobuf chunks directly
        Ok(response.chunks)
    }

    /// Check if an archive exists
    pub async fn archive_exists(&self, archive_id: &str) -> Result<bool> {
        match self.fetch_download_metadata(archive_id).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }


    // === Private helper methods ===

    /// Extract org_id from JWT token without verification (for client use only)
    async fn get_org_id_from_token(&self) -> Option<String> {
        let config = self.config.lock().await;
        let token = config.token.clone();
        drop(config);

        // JWT tokens have 3 parts separated by dots: header.payload.signature
        // We only need the payload (middle part) to extract org_id
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            warn!("Invalid JWT token format");
            return None;
        }

        // Decode the payload (base64url encoded)
        match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[1]) {
            Ok(decoded) => {
                match serde_json::from_slice::<JwtClaims>(&decoded) {
                    Ok(claims) => {
                        debug!("Extracted org_id from token: {:?}", claims.org_id);
                        claims.org_id
                    }
                    Err(e) => {
                        warn!("Failed to parse JWT claims: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                warn!("Failed to decode JWT payload: {}", e);
                None
            }
        }
    }

    /// Execute an API call with automatic token refresh on authentication failure
    async fn with_token_refresh<F, R>(&self, mut api_call: F) -> Result<R>
    where
        F: FnMut(String) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, tonic::Status>> + Send>>,
    {
        let config = self.config.lock().await;
        let token = config.token.clone();
        let api_url = config.api_url.clone();
        let refresh_token = config.refresh_token.clone();
        drop(config);

        // First attempt with current token
        match api_call(token.clone()).await {
            Ok(response) => Ok(response),
            Err(status) if self.is_auth_error(&status) => {
                warn!("Authentication failed, attempting to refresh token...");
                
                // Refresh the token
                self.refresh_token(api_url, token, refresh_token).await?;
                
                // Retry with new token
                let config = self.config.lock().await;
                let new_token = config.token.clone();
                drop(config);
                
                api_call(new_token).await.map_err(|e| anyhow!("API call failed after token refresh: {}", e))
            }
            Err(status) => Err(anyhow!("API call failed: {}", status)),
        }
    }

    /// Check if a tonic::Status indicates an authentication error
    fn is_auth_error(&self, status: &tonic::Status) -> bool {
        matches!(status.code(), tonic::Code::Unauthenticated | tonic::Code::PermissionDenied)
    }

    /// Refresh the JWT token using the refresh token
    async fn refresh_token(&self, api_url: String, token: String, refresh_token: String) -> Result<()> {
        info!("Refreshing expired token...");
        
        let endpoint = Endpoint::from_shared(api_url.clone())?;
        let channel = endpoint.connect().await?;
        
        let mut auth_client = pb::auth_service_client::AuthServiceClient::new(channel);
        let refresh_response = auth_client
            .refresh(pb::AuthServiceRefreshRequest {
                token,
                refresh: Some(refresh_token),
            })
            .await?
            .into_inner();

        // Update config with new tokens
        let mut config = self.config.lock().await;
        config.token = refresh_response.token;
        config.refresh_token = refresh_response.refresh;
        
        // Save updated config to disk
        config.save().await?;
        
        info!("✓ Token refreshed successfully");
        Ok(())
    }

    async fn list_protocols(&self) -> Result<Vec<pb::Protocol>> {
        let config = self.config.lock().await;
        let api_url = config.api_url.clone();
        drop(config);
        
        self.with_token_refresh(|token| {
            let api_url = api_url.clone();
            Box::pin(async move {
                let endpoint = Endpoint::from_shared(api_url)
                    .map_err(|e| tonic::Status::invalid_argument(format!("Invalid API URL: {}", e)))?;
                let channel = endpoint.connect().await
                    .map_err(|e| tonic::Status::unavailable(format!("Failed to connect: {}", e)))?;
                
                let mut client = pb::protocol_service_client::ProtocolServiceClient::with_interceptor(
                    channel,
                    ApiInterceptor(
                        AuthToken(token),
                        DefaultTimeout(DEFAULT_API_REQUEST_TIMEOUT),
                    ),
                );
                
                let response = client
                    .list_protocols(pb::ProtocolServiceListProtocolsRequest {
                        org_ids: vec![], // Public protocols only for now
                        offset: 0,
                        limit: 1000, // Get up to 1000 protocols
                        search: None,
                        sort: vec![],
                    })
                    .await?
                    .into_inner();
                
                Ok(response.protocols)
            })
        }).await
    }

    async fn list_variants(&self, protocol_id: &str) -> Result<Vec<String>> {
        let config = self.config.lock().await;
        let api_url = config.api_url.clone();
        drop(config);
        
        let protocol_id = protocol_id.to_string();
        
        self.with_token_refresh(|token| {
            let api_url = api_url.clone();
            let protocol_id = protocol_id.clone();
            Box::pin(async move {
                let endpoint = Endpoint::from_shared(api_url)
                    .map_err(|e| tonic::Status::invalid_argument(format!("Invalid API URL: {}", e)))?;
                let channel = endpoint.connect().await
                    .map_err(|e| tonic::Status::unavailable(format!("Failed to connect: {}", e)))?;
                
                let mut client = pb::protocol_service_client::ProtocolServiceClient::with_interceptor(
                    channel,
                    ApiInterceptor(
                        AuthToken(token),
                        DefaultTimeout(DEFAULT_API_REQUEST_TIMEOUT),
                    ),
                );
                
                let response = client
                    .list_variants(pb::ProtocolServiceListVariantsRequest {
                        protocol_id,
                        org_id: None,
                    })
                    .await?
                    .into_inner();
                
                Ok(response.variant_keys)
            })
        }).await
    }

    async fn get_image(&self, version_key: &common::ProtocolVersionKey) -> Result<pb::Image> {
        let config = self.config.lock().await;
        let api_url = config.api_url.clone();
        drop(config);
        
        let version_key = version_key.clone();
        
        self.with_token_refresh(|token| {
            let api_url = api_url.clone();
            let version_key = version_key.clone();
            Box::pin(async move {
                let endpoint = Endpoint::from_shared(api_url)
                    .map_err(|e| tonic::Status::invalid_argument(format!("Invalid API URL: {}", e)))?;
                let channel = endpoint.connect().await
                    .map_err(|e| tonic::Status::unavailable(format!("Failed to connect: {}", e)))?;
                
                let mut client = pb::image_service_client::ImageServiceClient::with_interceptor(
                    channel,
                    ApiInterceptor(
                        AuthToken(token),
                        DefaultTimeout(DEFAULT_API_REQUEST_TIMEOUT),
                    ),
                );
                
                let response = client
                    .get_image(pb::ImageServiceGetImageRequest {
                        version_key: Some(version_key),
                        org_id: None,
                        semantic_version: None,
                        build_version: None,
                    })
                    .await?
                    .into_inner();
                
                Ok(response.image.ok_or_else(|| tonic::Status::not_found("No image found"))?)
            })
        }).await
    }

    async fn list_archives(&self, image_id: &str) -> Result<Vec<pb::Archive>> {
        let config = self.config.lock().await;
        let api_url = config.api_url.clone();
        drop(config);
        
        let image_id = image_id.to_string();
        
        self.with_token_refresh(|token| {
            let api_url = api_url.clone();
            let image_id = image_id.clone();
            Box::pin(async move {
                let endpoint = Endpoint::from_shared(api_url)
                    .map_err(|e| tonic::Status::invalid_argument(format!("Invalid API URL: {}", e)))?;
                let channel = endpoint.connect().await
                    .map_err(|e| tonic::Status::unavailable(format!("Failed to connect: {}", e)))?;
                
                let mut client = pb::image_service_client::ImageServiceClient::with_interceptor(
                    channel,
                    ApiInterceptor(
                        AuthToken(token),
                        DefaultTimeout(DEFAULT_API_REQUEST_TIMEOUT),
                    ),
                );
                
                let response = client
                    .list_archives(pb::ImageServiceListArchivesRequest {
                        image_id,
                        org_id: None,
                    })
                    .await?
                    .into_inner();
                
                Ok(response.archives)
            })
        }).await
    }

    pub async fn fetch_download_metadata(&self, archive_id: &str) -> Result<babel_api::engine::DownloadMetadata> {
        self.fetch_download_metadata_with_version(archive_id, None).await
    }
    
    pub async fn fetch_download_metadata_with_version(&self, archive_id: &str, version: Option<u64>) -> Result<babel_api::engine::DownloadMetadata> {
        let config = self.config.lock().await;
        let api_url = config.api_url.clone();
        drop(config);
        
        let archive_id = archive_id.to_string();
        // Try without org_id first (for public archives)
        let org_id: Option<String> = None;
        
        let response = self.with_token_refresh(|token| {
            let api_url = api_url.clone();
            let archive_id = archive_id.clone();
            let org_id = org_id.clone();
            let data_version = version;
            Box::pin(async move {
                let endpoint = Endpoint::from_shared(api_url)
                    .map_err(|e| tonic::Status::invalid_argument(format!("Invalid API URL: {}", e)))?;
                let channel = endpoint.connect().await
                    .map_err(|e| tonic::Status::unavailable(format!("Failed to connect: {}", e)))?;
                
                let mut client = pb::archive_service_client::ArchiveServiceClient::with_interceptor(
                    channel,
                    ApiInterceptor(
                        AuthToken(token),
                        DefaultTimeout(DEFAULT_API_REQUEST_TIMEOUT),
                    ),
                );
                
                let response = client
                    .get_download_metadata(pb::ArchiveServiceGetDownloadMetadataRequest {
                        archive_id,
                        org_id,
                        data_version,
                    })
                    .await?
                    .into_inner();
                
                Ok(response)
            })
        }).await?;
        
        response.try_into()
            .map_err(|e| anyhow!("Failed to convert response: {}", e))
    }

    /// Parse a store_key like "arbitrum-one-nitro-mainnet-full-v1" into components
    fn parse_store_key(&self, store_key: &str) -> Result<(String, String, String, String)> {
        // Use the existing parsing logic from SnapshotMetadata
        SnapshotMetadata::parse_archive_id(store_key)
    }
}