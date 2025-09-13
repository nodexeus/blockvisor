/// R3.4: Integration module for multi-client upload/download functionality with RHAI plugins
/// This module provides utilities to bridge RHAI plugin configuration with the per-client
/// upload and download systems.

use crate::{
    download_job::{ClientDownloadConfig, MultiClientDownloader},
    upload_job::{ClientArchiveConfig, MultiClientUploader},
    job_runner::TransferConfig,
    pal::BabelEngineConnector,
};
use babel_api::plugin_config::PluginConfig;
use eyre::{bail, Context, Result};
use nu_glob::Pattern;
use std::path::PathBuf;
use bv_utils::run_flag::RunFlag;
use tracing::{info, warn};

/// R2.1: Extract archivable clients from plugin configuration
pub fn get_archivable_clients(
    plugin_config: &PluginConfig,
    protocol_data_path: &PathBuf,
    exclude_patterns: &[String],
) -> Result<Vec<ClientArchiveConfig>> {
    let exclude = exclude_patterns
        .iter()
        .map(|pattern_str| Pattern::new(pattern_str))
        .collect::<Result<Vec<Pattern>, _>>()
        .context("Failed to parse exclude patterns")?;
    
    plugin_config.services.iter()
        .filter(|service| service.archive)                      // R2.1: Archive by default unless explicitly false
        .filter(|service| service.store_key.is_some())          // Only services with store_key
        .map(|service| {
            let data_dir_name = service.data_dir.as_ref()
                .unwrap_or(&service.name);                       // R1.1: Default to service name
            let data_dir = protocol_data_path.join(data_dir_name);
            Ok(ClientArchiveConfig {
                client_name: service.name.clone(),
                data_directory: data_dir,
                store_key: service.store_key.clone().unwrap(),   // R2.2: Individual store key
                exclude_patterns: exclude.clone(),
            })
        })
        .collect()
}

/// R3.1: Extract downloadable clients from plugin configuration
pub fn get_downloadable_clients(
    plugin_config: &PluginConfig,
    protocol_data_path: &PathBuf,
) -> Result<Vec<ClientDownloadConfig>> {
    plugin_config.services.iter()
        .filter(|service| service.archive)                      // Only archived services can be downloaded
        .filter(|service| service.store_key.is_some())          // Only services with store_key
        .map(|service| {
            let data_dir_name = service.data_dir.as_ref()
                .unwrap_or(&service.name);                       // R1.1: Default to service name
            let data_dir = protocol_data_path.join(data_dir_name);
            Ok(ClientDownloadConfig {
                client_name: service.name.clone(),
                data_directory: data_dir,
                store_key: service.store_key.clone().unwrap(),   // R2.2: Individual store key
            })
        })
        .collect()
}

/// R2.4 & R3.1: Execute multi-client upload with plugin configuration
pub async fn execute_multi_client_upload<C: BabelEngineConnector + Clone + Send + Sync + 'static>(
    plugin_config: &PluginConfig,
    connector: C,
    config: TransferConfig,
    url_expires_secs: Option<u32>,
    data_version: Option<u64>,
    exclude_patterns: &[String],
    run: RunFlag,
) -> Result<()> {
    let clients = get_archivable_clients(plugin_config, &config.data_mount_point, exclude_patterns)?;
    
    if clients.is_empty() {
        warn!("No archivable clients found in plugin configuration");
        return Ok(());
    }
    
    info!("Starting multi-client upload for {} clients: {:?}", 
          clients.len(), 
          clients.iter().map(|c| &c.client_name).collect::<Vec<_>>());
    
    let mut multi_uploader = MultiClientUploader::new(
        connector,
        url_expires_secs,
        data_version,
        config,
    );
    
    multi_uploader.upload_all_clients(clients, run).await
}

/// R3.1: Execute multi-client download with plugin configuration
pub async fn execute_multi_client_download<C: BabelEngineConnector + Clone + Send + Sync + 'static>(
    plugin_config: &PluginConfig,
    connector: C,
    config: TransferConfig,
    run: RunFlag,
) -> Result<()> {
    let clients = get_downloadable_clients(plugin_config, &config.data_mount_point)?;
    
    if clients.is_empty() {
        warn!("No downloadable clients found in plugin configuration");
        return Ok(());
    }
    
    info!("Starting multi-client download for {} clients: {:?}", 
          clients.len(), 
          clients.iter().map(|c| &c.client_name).collect::<Vec<_>>());
    
    let mut multi_downloader = MultiClientDownloader::new(connector, config);
    
    multi_downloader.download_all_clients(clients, run).await
}

/// R3.4: Validate plugin configuration for multi-client functionality
pub fn validate_multi_client_config(plugin_config: &PluginConfig) -> Result<()> {
    let mut errors = Vec::new();
    
    for service in &plugin_config.services {
        if service.archive {
            // R2.1: Archived services must have store_key
            if service.store_key.is_none() {
                errors.push(format!(
                    "Service '{}' has archive=true but no store_key specified", 
                    service.name
                ));
            }
            
            // R1.1: Validate data_dir if specified
            if let Some(data_dir) = &service.data_dir {
                if data_dir.is_empty() || data_dir.starts_with('/') || data_dir.contains("..") {
                    errors.push(format!(
                        "Service '{}' data_dir '{}' must be a relative path without '..' components",
                        service.name,
                        data_dir
                    ));
                }
            }
        }
    }
    
    if !errors.is_empty() {
        bail!("Multi-client configuration validation failed:\n{}", errors.join("\n"));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use babel_api::plugin_config::Service;
    use std::path::Path;

    fn create_test_service(name: &str, archive: bool, store_key: Option<String>, data_dir: Option<String>) -> Service {
        Service {
            name: name.to_string(),
            run_sh: format!("/usr/bin/{} start", name),
            restart_config: None,
            shutdown_timeout_secs: None,
            shutdown_signal: None,
            run_as: None,
            use_protocol_data: true,
            log_buffer_capacity_mb: None,
            log_timestamp: None,
            data_dir,
            store_key,
            archive,
        }
    }

    #[test]
    fn test_get_archivable_clients() {
        let plugin_config = PluginConfig {
            config_files: None,
            aux_services: None,
            init: None,
            download: None,
            alternative_download: None,
            post_download: None,
            cold_init: None,
            services: vec![
                create_test_service("reth", true, Some("ethereum-reth-mainnet-v1".to_string()), None),
                create_test_service("lighthouse", true, Some("ethereum-lighthouse-mainnet-v1".to_string()), Some("beacon".to_string())),
                create_test_service("monitoring", false, Some("monitoring-v1".to_string()), None),
                create_test_service("helper", true, None, None), // No store_key
            ],
            pre_upload: None,
            upload: None,
            post_upload: None,
            scheduled: None,
        };

        let protocol_data_path = PathBuf::from("/blockjoy/protocol_data");
        let clients = get_archivable_clients(&plugin_config, &protocol_data_path, &[]).unwrap();
        
        assert_eq!(clients.len(), 2);
        assert_eq!(clients[0].client_name, "reth");
        assert_eq!(clients[0].data_directory, Path::new("/blockjoy/protocol_data/reth"));
        assert_eq!(clients[0].store_key, "ethereum-reth-mainnet-v1");
        
        assert_eq!(clients[1].client_name, "lighthouse");
        assert_eq!(clients[1].data_directory, Path::new("/blockjoy/protocol_data/beacon"));
        assert_eq!(clients[1].store_key, "ethereum-lighthouse-mainnet-v1");
    }
    
    #[test]
    fn test_validate_multi_client_config() {
        let mut plugin_config = PluginConfig {
            config_files: None,
            aux_services: None,
            init: None,
            download: None,
            alternative_download: None,
            post_download: None,
            cold_init: None,
            services: vec![
                create_test_service("reth", true, Some("ethereum-reth-mainnet-v1".to_string()), None),
                create_test_service("lighthouse", true, None, None), // Missing store_key
            ],
            pre_upload: None,
            upload: None,
            post_upload: None,
            scheduled: None,
        };
        
        // Should fail due to missing store_key
        assert!(validate_multi_client_config(&plugin_config).is_err());
        
        // Fix the configuration
        plugin_config.services[1].store_key = Some("ethereum-lighthouse-mainnet-v1".to_string());
        assert!(validate_multi_client_config(&plugin_config).is_ok());
    }

    #[test]
    fn test_get_downloadable_clients() {
        let plugin_config = PluginConfig {
            config_files: None,
            aux_services: None,
            init: None,
            download: None,
            alternative_download: None,
            post_download: None,
            cold_init: None,
            services: vec![
                create_test_service("reth", true, Some("ethereum-reth-mainnet-v1".to_string()), None),
                create_test_service("lighthouse", true, Some("ethereum-lighthouse-mainnet-v1".to_string()), Some("beacon".to_string())),
                create_test_service("monitoring", false, Some("monitoring-v1".to_string()), None), // Not archived
                create_test_service("helper", true, None, None), // No store_key
            ],
            pre_upload: None,
            upload: None,
            post_upload: None,
            scheduled: None,
        };

        let protocol_data_path = PathBuf::from("/blockjoy/protocol_data");
        let clients = get_downloadable_clients(&plugin_config, &protocol_data_path).unwrap();
        
        // Should only include archived services with store_key
        assert_eq!(clients.len(), 2);
        assert_eq!(clients[0].client_name, "reth");
        assert_eq!(clients[0].data_directory, Path::new("/blockjoy/protocol_data/reth"));
        assert_eq!(clients[0].store_key, "ethereum-reth-mainnet-v1");
        
        assert_eq!(clients[1].client_name, "lighthouse");
        assert_eq!(clients[1].data_directory, Path::new("/blockjoy/protocol_data/beacon"));
        assert_eq!(clients[1].store_key, "ethereum-lighthouse-mainnet-v1");
    }

    #[test]
    fn test_get_archivable_clients_with_exclude_patterns() {
        let plugin_config = PluginConfig {
            config_files: None,
            aux_services: None,
            init: None,
            download: None,
            alternative_download: None,
            post_download: None,
            cold_init: None,
            services: vec![
                create_test_service("reth", true, Some("ethereum-reth-mainnet-v1".to_string()), None),
                create_test_service("lighthouse", true, Some("ethereum-lighthouse-mainnet-v1".to_string()), Some("beacon".to_string())),
            ],
            pre_upload: None,
            upload: None,
            post_upload: None,
            scheduled: None,
        };

        let protocol_data_path = PathBuf::from("/blockjoy/protocol_data");
        let exclude_patterns = vec!["*.log".to_string(), "temp/*".to_string()];
        let clients = get_archivable_clients(&plugin_config, &protocol_data_path, &exclude_patterns).unwrap();
        
        assert_eq!(clients.len(), 2);
        // Check that exclude patterns are applied to both clients
        for client in clients {
            assert_eq!(client.exclude_patterns.len(), 2);
        }
    }

    #[test]
    fn test_invalid_exclude_patterns() {
        let plugin_config = PluginConfig {
            config_files: None,
            aux_services: None,
            init: None,
            download: None,
            alternative_download: None,
            post_download: None,
            cold_init: None,
            services: vec![
                create_test_service("reth", true, Some("ethereum-reth-mainnet-v1".to_string()), None),
            ],
            pre_upload: None,
            upload: None,
            post_upload: None,
            scheduled: None,
        };

        let protocol_data_path = PathBuf::from("/blockjoy/protocol_data");
        let invalid_exclude_patterns = vec!["[".to_string()]; // Invalid glob pattern
        let result = get_archivable_clients(&plugin_config, &protocol_data_path, &invalid_exclude_patterns);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse exclude patterns"));
    }

    #[test]
    fn test_validate_multi_client_config_invalid_data_dir() {
        let plugin_config = PluginConfig {
            config_files: None,
            aux_services: None,
            init: None,
            download: None,
            alternative_download: None,
            post_download: None,
            cold_init: None,
            services: vec![
                // Absolute path in data_dir (invalid)
                create_test_service("reth", true, Some("ethereum-reth-mainnet-v1".to_string()), Some("/absolute/path".to_string())),
                // Path with ".." (invalid)
                create_test_service("lighthouse", true, Some("ethereum-lighthouse-mainnet-v1".to_string()), Some("../parent/path".to_string())),
                // Empty data_dir (invalid)
                create_test_service("empty", true, Some("empty-v1".to_string()), Some("".to_string())),
            ],
            pre_upload: None,
            upload: None,
            post_upload: None,
            scheduled: None,
        };
        
        let result = validate_multi_client_config(&plugin_config);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("must be a relative path"));
    }

    #[test]
    fn test_validate_multi_client_config_valid_cases() {
        let valid_plugin_config = PluginConfig {
            config_files: None,
            aux_services: None,
            init: None,
            download: None,
            alternative_download: None,
            post_download: None,
            cold_init: None,
            services: vec![
                // Valid archived service with store_key and no data_dir (defaults to name)
                create_test_service("reth", true, Some("ethereum-reth-mainnet-v1".to_string()), None),
                // Valid archived service with custom data_dir
                create_test_service("lighthouse", true, Some("ethereum-lighthouse-mainnet-v1".to_string()), Some("beacon".to_string())),
                // Valid non-archived service (no store_key required)
                create_test_service("monitoring", false, None, None),
            ],
            pre_upload: None,
            upload: None,
            post_upload: None,
            scheduled: None,
        };
        
        assert!(validate_multi_client_config(&valid_plugin_config).is_ok());
    }

    #[test]
    fn test_empty_services_list() {
        let plugin_config = PluginConfig {
            config_files: None,
            aux_services: None,
            init: None,
            download: None,
            alternative_download: None,
            post_download: None,
            cold_init: None,
            services: vec![], // Empty services
            pre_upload: None,
            upload: None,
            post_upload: None,
            scheduled: None,
        };

        let protocol_data_path = PathBuf::from("/blockjoy/protocol_data");
        
        // Should return empty lists but not error
        let archivable = get_archivable_clients(&plugin_config, &protocol_data_path, &[]).unwrap();
        let downloadable = get_downloadable_clients(&plugin_config, &protocol_data_path).unwrap();
        
        assert_eq!(archivable.len(), 0);
        assert_eq!(downloadable.len(), 0);
        
        // Validation should pass for empty services
        assert!(validate_multi_client_config(&plugin_config).is_ok());
    }
}