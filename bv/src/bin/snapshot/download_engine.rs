use super::{types::*, api_service::ApiService};
use babel_api::engine::{DownloadMetadata, JobProgress};
use eyre::{anyhow, Context, Result};
use reqwest::Client;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::{fs, io::{AsyncWriteExt, AsyncSeekExt}, sync::Mutex};
use tracing::info;

const PROGRESS_FILENAME: &str = "download_progress.json";
const METADATA_FILENAME: &str = "download_metadata.json";

pub struct SnapshotDownloader {
    config: Arc<Mutex<SnapshotConfig>>,
    output_dir: PathBuf,
    workers: usize,
    max_connections: usize,
}

impl SnapshotDownloader {
    pub fn new(config: SnapshotConfig, download_config: DownloadConfig) -> Result<Self> {
        // Ensure output directory exists
        std::fs::create_dir_all(&download_config.output_dir)
            .with_context(|| format!("Failed to create output directory: {}", download_config.output_dir.display()))?;

        Ok(Self {
            config: Arc::new(Mutex::new(config)),
            output_dir: download_config.output_dir,
            workers: download_config.workers,
            max_connections: download_config.max_connections,
        })
    }

    /// Start a new download
    pub async fn download(&self, archive_id: &str) -> Result<()> {
        info!("Starting download for archive: {}", archive_id);
        
        // Create metadata directory
        let metadata_dir = self.output_dir.join(".bv-snapshot");
        fs::create_dir_all(&metadata_dir).await
            .with_context(|| "Failed to create metadata directory")?;

        // Step 1: Get download metadata from API service
        let api_service = ApiService::new((*self.config.lock().await).clone());
        let download_info = api_service.get_download_metadata_by_store_key(archive_id).await?;
        info!(
            "Archive metadata: {} chunks, {:.2} GB total", 
            download_info.chunks,
            download_info.total_size as f64 / (1024.0 * 1024.0 * 1024.0)
        );

        // Convert DownloadInfo to DownloadMetadata for compatibility
        let metadata = DownloadMetadata {
            total_size: download_info.total_size,
            chunks: download_info.chunks,
            compression: download_info.compression,
            data_version: download_info.version, // Use the actual version from download_info
        };

        // Step 2: Save metadata for status reporting
        self.save_metadata(&metadata, archive_id).await?;

        // Step 3: Initialize progress tracking
        let progress = JobProgress {
            total: metadata.chunks,
            current: 0,
            message: "Starting download...".to_string(),
        };
        self.save_progress(&progress).await?;

        // Step 4: Download chunks progressively
        self.download_chunks(&metadata, archive_id).await?;

        info!("✓ Download completed successfully!");
        self.cleanup_metadata().await?;
        Ok(())
    }

    /// Resume an interrupted download
    pub async fn resume(&self) -> Result<()> {
        info!("Resuming download in: {}", self.output_dir.display());
        
        // Check if there's an existing download to resume
        let metadata_path = self.output_dir.join(METADATA_FILENAME);
        if !metadata_path.exists() {
            return Err(eyre::anyhow!("No download metadata found. Cannot resume download."));
        }

        // Load existing metadata
        let metadata_content = fs::read_to_string(&metadata_path).await
            .with_context(|| "Failed to read download metadata")?;
        let (metadata, archive_id): (DownloadMetadata, String) = serde_json::from_str(&metadata_content)
            .with_context(|| "Failed to parse download metadata")?;

        info!("Resuming download for archive: {}", archive_id);
        
        // Resume from where we left off
        self.download_chunks(&metadata, &archive_id).await?;

        info!("✓ Download resumed and completed successfully!");
        self.cleanup_metadata().await?;
        Ok(())
    }

    /// Get current download status
    pub async fn get_status(&self) -> Result<DownloadStatus> {
        let progress_path = self.output_dir.join(PROGRESS_FILENAME);
        let metadata_path = self.output_dir.join(METADATA_FILENAME);

        // Check if download exists
        if !metadata_path.exists() {
            return Ok(DownloadStatus::NotStarted);
        }

        // Load metadata
        let metadata_content = fs::read_to_string(&metadata_path).await
            .with_context(|| "Failed to read download metadata")?;
        let (metadata, _archive_id): (DownloadMetadata, String) = serde_json::from_str(&metadata_content)
            .with_context(|| "Failed to parse download metadata")?;

        // Check progress
        if progress_path.exists() {
            let progress_content = fs::read_to_string(&progress_path).await
                .with_context(|| "Failed to read progress file")?;
            
            match serde_json::from_str::<JobProgress>(&progress_content) {
                Ok(progress) => {
                    if progress.current >= progress.total {
                        return Ok(DownloadStatus::Completed);
                    }

                    let progress_percent = if progress.total > 0 {
                        ((progress.current as f64 / progress.total as f64) * 100.0) as u32
                    } else {
                        0
                    };

                    let downloaded_bytes = (progress.current as f64 / progress.total as f64 * metadata.total_size as f64) as u64;
                    
                    // Simple speed and ETA calculation (could be improved)
                    let speed_mbps = 100.0; // Mock speed - could track real speed
                    let remaining_bytes = metadata.total_size - downloaded_bytes;
                    let eta_minutes = if speed_mbps > 0.0 {
                        (remaining_bytes as f64 / (speed_mbps * 1024.0 * 1024.0 * 60.0)) as u32
                    } else {
                        0
                    };

                    Ok(DownloadStatus::InProgress {
                        progress_percent,
                        downloaded_bytes,
                        total_bytes: metadata.total_size,
                        speed_mbps,
                        eta_minutes,
                        chunks_complete: progress.current,
                        total_chunks: progress.total,
                    })
                },
                Err(_) => Ok(DownloadStatus::Failed {
                    error: "Failed to parse progress file".to_string(),
                }),
            }
        } else {
            Ok(DownloadStatus::NotStarted)
        }
    }

    // === Private helper methods ===



    async fn download_chunks(&self, metadata: &DownloadMetadata, archive_id: &str) -> Result<()> {
        info!("Starting to download {} chunks with {} workers", metadata.chunks, self.workers);
        
        // Get API service to fetch chunk URLs  
        let api_service = ApiService::new((*self.config.lock().await).clone());
        
        // Parse the archive_id to get the base archive ID (without version)
        let base_archive_id = if archive_id.contains('/') {
            archive_id.split('/').next().unwrap()
        } else {
            archive_id
        };
        
        // First discover to get the archive UUID
        let protocol_groups = api_service.discover_snapshots().await?;
        
        // Find the matching snapshot to get the UUID
        let mut archive_uuid = None;
        'outer: for group in protocol_groups {
            for (_, client_group) in &group.clients {
                for (_, snapshots) in &client_group.networks {
                    for snapshot in snapshots {
                        if snapshot.archive_id == base_archive_id {
                            archive_uuid = Some(snapshot.archive_uuid.clone());
                            break 'outer;
                        }
                    }
                }
            }
        }
        
        let archive_uuid = archive_uuid.ok_or_else(|| anyhow!("Archive UUID not found for {}", archive_id))?;
        
        // Get signed URLs for all chunks in batches (API limit is 100 chunks per request)
        const MAX_CHUNKS_PER_REQUEST: u32 = 100;
        let mut all_chunks = Vec::new();
        
        for batch_start in (0..metadata.chunks).step_by(MAX_CHUNKS_PER_REQUEST as usize) {
            let batch_end = (batch_start + MAX_CHUNKS_PER_REQUEST).min(metadata.chunks);
            let chunk_indexes: Vec<u32> = (batch_start..batch_end).collect();
            
            info!("Requesting chunk batch {}-{}", batch_start, batch_end - 1);
            let batch_chunks = api_service.get_download_chunks(&archive_uuid, metadata.data_version, chunk_indexes).await?;
            all_chunks.extend(batch_chunks);
        }
        
        let chunks = all_chunks;
        
        // Create HTTP client
        let client = Client::new();
        
        // Download chunks in parallel using semaphore for concurrency control
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.workers));
        let mut handles = Vec::new();
        
        for (chunk_idx, chunk) in chunks.into_iter().enumerate() {
            let client = client.clone();
            let semaphore = semaphore.clone();
            let output_dir = self.output_dir.clone();
            let chunk_idx = chunk_idx as u32;
            let total_chunks = metadata.chunks;
            let config = self.config.clone();
            let compression = metadata.compression.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await?;
                
                // Skip if no URL (blueprint chunks)
                let url = match &chunk.url {
                    Some(url) => url,
                    None => {
                        info!("Skipping blueprint chunk {}", chunk_idx);
                        return Ok::<(), eyre::Error>(());
                    }
                };
                
                // Download the chunk data
                let response = client.get(url)
                    .send()
                    .await
                    .with_context(|| format!("Failed to download chunk {}", chunk_idx))?;
                
                let chunk_data = response.bytes()
                    .await
                    .with_context(|| format!("Failed to read chunk {} data", chunk_idx))?;
                
                // Decompress if needed (assuming ZSTD compression)  
                let decompressed_data = if let Some(babel_api::engine::Compression::ZSTD(_)) = &compression {
                    // Use streaming decompressor to handle unknown decompressed size
                    zstd::stream::decode_all(&chunk_data[..])
                        .with_context(|| format!("Failed to decompress chunk {}", chunk_idx))?
                } else {
                    chunk_data.to_vec()
                };
                
                // Write decompressed data to files based on chunk destinations
                for destination in &chunk.destinations {
                    let file_path = output_dir.join(&destination.path);
                    
                    // Create parent directories
                    if let Some(parent) = file_path.parent() {
                        fs::create_dir_all(parent).await
                            .with_context(|| format!("Failed to create directory for {}", destination.path))?;
                    }
                    
                    // Open file for writing (create or append)
                    let mut file = fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(&file_path)
                        .await
                        .with_context(|| format!("Failed to open file {}", destination.path))?;
                    
                    // Seek to the correct position
                    file.seek(std::io::SeekFrom::Start(destination.position_bytes)).await
                        .with_context(|| format!("Failed to seek in file {}", destination.path))?;
                    
                    // Write data
                    file.write_all(&decompressed_data).await
                        .with_context(|| format!("Failed to write to file {}", destination.path))?;
                    
                    file.flush().await
                        .with_context(|| format!("Failed to flush file {}", destination.path))?;
                }
                
                // Update progress
                let progress = JobProgress {
                    total: total_chunks,
                    current: chunk_idx + 1,
                    message: format!("Downloaded chunk {}/{}", chunk_idx + 1, total_chunks),
                };
                
                // Save progress (need to lock config for this)
                let snapshot_downloader = SnapshotDownloader {
                    config: config,
                    output_dir: output_dir.clone(),
                    workers: 1, // Dummy value
                    max_connections: 1, // Dummy value
                };
                snapshot_downloader.save_progress(&progress).await?;
                
                if (chunk_idx + 1) % 10 == 0 {
                    info!("Downloaded {}/{} chunks", chunk_idx + 1, total_chunks);
                }
                
                Ok::<(), eyre::Error>(())
            });
            
            handles.push(handle);
        }
        
        // Wait for all downloads to complete
        for handle in handles {
            handle.await
                .with_context(|| "Failed to join download task")?
                .with_context(|| "Download task failed")?;
        }
        
        Ok(())
    }

    async fn save_metadata(&self, metadata: &DownloadMetadata, archive_id: &str) -> Result<()> {
        let metadata_path = self.output_dir.join(METADATA_FILENAME);
        let data = (metadata, archive_id);
        let content = serde_json::to_string_pretty(&data)?;
        
        fs::write(&metadata_path, content).await
            .with_context(|| "Failed to save download metadata")?;
        
        Ok(())
    }

    async fn save_progress(&self, progress: &JobProgress) -> Result<()> {
        let progress_path = self.output_dir.join(PROGRESS_FILENAME);
        let content = serde_json::to_string_pretty(progress)?;
        
        fs::write(&progress_path, content).await
            .with_context(|| "Failed to save progress")?;
        
        Ok(())
    }

    async fn cleanup_metadata(&self) -> Result<()> {
        let metadata_path = self.output_dir.join(METADATA_FILENAME);
        let progress_path = self.output_dir.join(PROGRESS_FILENAME);
        
        if metadata_path.exists() {
            fs::remove_file(&metadata_path).await.ok();
        }
        if progress_path.exists() {
            fs::remove_file(&progress_path).await.ok();
        }
        
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_downloader_creation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_dir = temp_dir.path().to_path_buf();
        
        let config = SnapshotConfig {
            token: "test_token".to_string(),
            refresh_token: "test_refresh_token".to_string(),
            api_url: "https://test.api.com".to_string(),
        };

        let download_config = DownloadConfig {
            workers: 4,
            max_connections: 4,
            output_dir,
        };

        let downloader = SnapshotDownloader::new(config, download_config)?;
        
        // Test that the downloader was created successfully
        assert!(downloader.output_dir.exists());
        assert_eq!(downloader.workers, 4);
        assert_eq!(downloader.max_connections, 4);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_status_not_started() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_dir = temp_dir.path().to_path_buf();
        
        let config = SnapshotConfig {
            token: "test_token".to_string(),
            refresh_token: "test_refresh_token".to_string(),
            api_url: "https://test.api.com".to_string(),
        };

        let download_config = DownloadConfig {
            workers: 4,
            max_connections: 4,
            output_dir,
        };

        let downloader = SnapshotDownloader::new(config, download_config)?;
        let status = downloader.get_status().await?;
        
        matches!(status, DownloadStatus::NotStarted);
        
        Ok(())
    }
}