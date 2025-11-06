use super::{api_service::ApiService, types::*};
use babel_api::engine::{DownloadMetadata, JobProgress};
use blockvisord::services::api::pb;
use eyre::{anyhow, Context, Result};
use reqwest::Client;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::{
    fs,
    io::{AsyncSeekExt, AsyncWriteExt},
    sync::Mutex,
};
use tracing::{info, warn, debug, error};

const PROGRESS_FILENAME: &str = "download_progress.json";
const METADATA_FILENAME: &str = "download_metadata.json";

/// Validation errors for chunk processing
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Incomplete chunk consumption: {remaining} bytes remaining")]
    IncompleteConsumption { remaining: usize },
    #[error("Size mismatch: expected {expected} bytes, got {actual} bytes")]
    SizeMismatch { expected: u64, actual: u64 },
    #[error("Destination overlap detected in chunk {chunk_idx}: destinations {dest1} and {dest2} overlap")]
    DestinationOverlap { chunk_idx: u32, dest1: usize, dest2: usize },
    #[error("Invalid destination size: destination {dest_idx} has zero size_bytes")]
    InvalidDestinationSize { dest_idx: usize },
    #[error("Checksum verification failed for chunk {chunk_idx}: expected {expected:?}, got {actual:?}")]
    ChecksumMismatch { chunk_idx: u32, expected: String, actual: String },
    #[error("Unsupported checksum type for chunk {chunk_idx}")]
    UnsupportedChecksum { chunk_idx: u32 },
    #[error("Invalid destination path '{path}': {reason}")]
    InvalidDestinationPath { path: String, reason: String },
    #[error("Destination file size exceeds reasonable limits: {size} bytes for '{path}'")]
    UnreasonableFileSize { path: String, size: u64 },
    #[error("Potential directory traversal attempt in path: '{path}'")]
    DirectoryTraversalAttempt { path: String },
}

/// Information about a destination write operation
#[derive(Debug, Clone)]
pub struct DestinationWrite {
    pub file_path: String,
    pub position_bytes: u64,
    pub size_bytes: u64,
    pub offset_in_chunk: usize,
}

/// Logger for tracking chunk processing operations
#[derive(Debug)]
pub struct ChunkWriteLogger {
    pub chunk_idx: u32,
    pub total_decompressed_size: usize,
    pub destinations_written: Vec<DestinationWrite>,
}

/// Progress tracker for cumulative download statistics
#[derive(Debug, Clone)]
pub struct DownloadProgressTracker {
    pub total_chunks: u32,
    pub chunks_processed: u32,
    pub total_bytes_written: u64,
    pub expected_total_size: u64,
}

impl DownloadProgressTracker {
    pub fn new(total_chunks: u32, expected_total_size: u64) -> Self {
        Self {
            total_chunks,
            chunks_processed: 0,
            total_bytes_written: 0,
            expected_total_size,
        }
    }
    
    pub fn update_chunk_processed(&mut self, bytes_written: u64) {
        self.chunks_processed += 1;
        self.total_bytes_written += bytes_written;
        
        let progress_percent = if self.total_chunks > 0 {
            (self.chunks_processed as f64 / self.total_chunks as f64) * 100.0
        } else {
            0.0
        };
        
        let size_ratio = if self.expected_total_size > 0 {
            self.total_bytes_written as f64 / self.expected_total_size as f64
        } else {
            0.0
        };
        
        info!(
            "Download progress: {}/{} chunks ({:.1}%), {} bytes written (ratio: {:.2}x expected)",
            self.chunks_processed, self.total_chunks, progress_percent, 
            self.total_bytes_written, size_ratio
        );
    }
    
    pub fn get_inflation_ratio(&self) -> f64 {
        if self.expected_total_size > 0 {
            self.total_bytes_written as f64 / self.expected_total_size as f64
        } else {
            0.0
        }
    }
}

/// Real-time size monitor for tracking disk usage during downloads
#[derive(Debug)]
pub struct SizeMonitor {
    output_dir: PathBuf,
    expected_total: u64,
    bytes_written_correctly: u64,
    last_warning_ratio: f64,
}

impl SizeMonitor {
    pub fn new(output_dir: PathBuf, expected_total: u64) -> Self {
        Self {
            output_dir,
            expected_total,
            bytes_written_correctly: 0,
            last_warning_ratio: 0.0,
        }
    }
    
    pub fn track_correct_write(&mut self, bytes: u64) {
        self.bytes_written_correctly += bytes;
    }
    
    pub async fn get_current_disk_usage(&self) -> Result<u64> {
        let mut total_size = 0u64;
        
        // Walk through all files in the output directory
        let mut entries = fs::read_dir(&self.output_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                if let Ok(metadata) = fs::metadata(&path).await {
                    total_size += metadata.len();
                }
            } else if path.is_dir() && !path.file_name().unwrap_or_default().to_string_lossy().starts_with('.') {
                // Recursively calculate directory size (excluding hidden dirs like .snapper)
                total_size += self.calculate_directory_size(&path).await?;
            }
        }
        
        Ok(total_size)
    }
    
    async fn calculate_directory_size(&self, dir_path: &PathBuf) -> Result<u64> {
        use std::collections::VecDeque;
        
        let mut total_size = 0u64;
        let mut dirs_to_process = VecDeque::new();
        dirs_to_process.push_back(dir_path.clone());
        
        // Use iterative approach instead of recursion to avoid boxing issues
        while let Some(current_dir) = dirs_to_process.pop_front() {
            let mut entries = fs::read_dir(&current_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = fs::metadata(&path).await {
                        total_size += metadata.len();
                    }
                } else if path.is_dir() {
                    dirs_to_process.push_back(path);
                }
            }
        }
        
        Ok(total_size)
    }
    
    pub fn calculate_inflation_ratio(&self) -> f64 {
        if self.expected_total > 0 {
            self.bytes_written_correctly as f64 / self.expected_total as f64
        } else {
            0.0
        }
    }
    
    pub async fn should_warn_user(&mut self) -> Result<bool> {
        let current_disk_usage = self.get_current_disk_usage().await?;
        let current_ratio = if self.expected_total > 0 {
            current_disk_usage as f64 / self.expected_total as f64
        } else {
            0.0
        };
        
        // Warn if disk usage exceeds 150% of expected and we haven't warned at this level
        const WARNING_THRESHOLD: f64 = 1.5;
        const WARNING_INTERVAL: f64 = 0.5; // Warn every 50% increase
        
        if current_ratio > WARNING_THRESHOLD && 
           (current_ratio - self.last_warning_ratio) >= WARNING_INTERVAL {
            
            warn!(
                "⚠️  Disk usage warning: Current usage {:.2} GB ({:.1}x expected size of {:.2} GB)",
                current_disk_usage as f64 / (1024.0 * 1024.0 * 1024.0),
                current_ratio,
                self.expected_total as f64 / (1024.0 * 1024.0 * 1024.0)
            );
            
            self.last_warning_ratio = current_ratio;
            return Ok(true);
        }
        
        Ok(false)
    }
    
    pub async fn log_size_status(&self) -> Result<()> {
        let current_disk_usage = self.get_current_disk_usage().await?;
        let inflation_ratio = self.calculate_inflation_ratio();
        let disk_ratio = if self.expected_total > 0 {
            current_disk_usage as f64 / self.expected_total as f64
        } else {
            0.0
        };
        
        info!(
            "Size monitoring: Expected {:.2} GB, Written correctly {:.2} GB (ratio: {:.2}x), Disk usage {:.2} GB (ratio: {:.2}x)",
            self.expected_total as f64 / (1024.0 * 1024.0 * 1024.0),
            self.bytes_written_correctly as f64 / (1024.0 * 1024.0 * 1024.0),
            inflation_ratio,
            current_disk_usage as f64 / (1024.0 * 1024.0 * 1024.0),
            disk_ratio
        );
        
        Ok(())
    }
}

impl ChunkWriteLogger {
    pub fn new(chunk_idx: u32, total_decompressed_size: usize) -> Self {
        Self {
            chunk_idx,
            total_decompressed_size,
            destinations_written: Vec::new(),
        }
    }

    pub fn log_destination_write(&mut self, path: &str, position_bytes: u64, size_bytes: u64, offset_in_chunk: usize) {
        let write = DestinationWrite {
            file_path: path.to_string(),
            position_bytes,
            size_bytes,
            offset_in_chunk,
        };
        
        debug!(
            "Chunk {} destination write: file='{}', position={}, size={}, chunk_offset={}",
            self.chunk_idx, path, position_bytes, size_bytes, offset_in_chunk
        );
        
        self.destinations_written.push(write);
    }

    pub fn validate_chunk_consumption(&self) -> Result<(), ValidationError> {
        let total_written: u64 = self.destinations_written.iter()
            .map(|d| d.size_bytes)
            .sum();
        
        if total_written as usize != self.total_decompressed_size {
            return Err(ValidationError::SizeMismatch {
                expected: self.total_decompressed_size as u64,
                actual: total_written,
            });
        }
        
        Ok(())
    }
}

/// Checksum verifier for chunk data integrity
pub struct ChecksumVerifier;

impl ChecksumVerifier {
    /// Verify chunk integrity after decompression using the provided checksum
    pub fn verify_chunk_integrity(
        chunk_idx: u32,
        decompressed_data: &[u8],
        expected_checksum: &Option<pb::Checksum>,
    ) -> Result<(), ValidationError> {
        let checksum = match expected_checksum {
            Some(checksum) => checksum,
            None => {
                debug!("No checksum provided for chunk {}, skipping verification", chunk_idx);
                return Ok(());
            }
        };

        let checksum_inner = match &checksum.checksum {
            Some(checksum_inner) => checksum_inner,
            None => {
                debug!("Empty checksum for chunk {}, skipping verification", chunk_idx);
                return Ok(());
            }
        };

        let calculated_checksum = match checksum_inner {
            pb::checksum::Checksum::Sha1(_expected) => {
                use sha1::{Sha1, Digest};
                let mut hasher = Sha1::new();
                hasher.update(decompressed_data);
                let result = hasher.finalize();
                format!("sha1:{}", hex::encode(result))
            }
            pb::checksum::Checksum::Sha256(_expected) => {
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(decompressed_data);
                let result = hasher.finalize();
                format!("sha256:{}", hex::encode(result))
            }
            pb::checksum::Checksum::Blake3(_expected) => {
                let mut hasher = blake3::Hasher::new();
                hasher.update(decompressed_data);
                let result = hasher.finalize();
                format!("blake3:{}", hex::encode(result.as_bytes()))
            }
        };

        let expected_checksum_str = match checksum_inner {
            pb::checksum::Checksum::Sha1(bytes) => format!("sha1:{}", hex::encode(bytes)),
            pb::checksum::Checksum::Sha256(bytes) => format!("sha256:{}", hex::encode(bytes)),
            pb::checksum::Checksum::Blake3(bytes) => format!("blake3:{}", hex::encode(bytes)),
        };

        if calculated_checksum != expected_checksum_str {
            error!(
                "Checksum verification failed for chunk {}: expected {}, got {}",
                chunk_idx, expected_checksum_str, calculated_checksum
            );
            return Err(ValidationError::ChecksumMismatch {
                chunk_idx,
                expected: expected_checksum_str,
                actual: calculated_checksum,
            });
        }

        debug!(
            "Checksum verification passed for chunk {}: {}",
            chunk_idx, calculated_checksum
        );
        Ok(())
    }
}

/// Validator for chunk destinations
pub struct DestinationValidator;

impl DestinationValidator {
    /// Maximum reasonable file size (10 GB per destination)
    const MAX_REASONABLE_FILE_SIZE: u64 = 10 * 1024 * 1024 * 1024;

    /// Validate that sum of destination sizes equals decompressed chunk size
    pub fn validate_destination_sizes(
        _chunk_idx: u32,
        destinations: &[pb::ChunkTarget],
        decompressed_size: usize,
    ) -> Result<(), ValidationError> {
        let total_size: u64 = destinations.iter()
            .map(|dest| dest.size_bytes)
            .sum();
        
        if total_size as usize != decompressed_size {
            return Err(ValidationError::SizeMismatch {
                expected: decompressed_size as u64,
                actual: total_size,
            });
        }
        
        // Check for zero-sized destinations
        for (idx, dest) in destinations.iter().enumerate() {
            if dest.size_bytes == 0 {
                return Err(ValidationError::InvalidDestinationSize { dest_idx: idx });
            }
        }
        
        Ok(())
    }

    /// Comprehensive destination validation including security checks
    pub fn validate_destinations_comprehensive(
        chunk_idx: u32,
        destinations: &[pb::ChunkTarget],
        decompressed_size: usize,
    ) -> Result<(), ValidationError> {
        // First run basic size validation
        Self::validate_destination_sizes(chunk_idx, destinations, decompressed_size)?;
        
        // Validate each destination for security and reasonableness
        for dest in destinations {
            Self::validate_destination_path(&dest.path)?;
            Self::validate_destination_size(&dest.path, dest.size_bytes)?;
        }
        
        // Check for overlaps
        Self::validate_no_overlaps(chunk_idx, destinations)?;
        
        Ok(())
    }

    /// Validate destination paths for security (prevent directory traversal)
    pub fn validate_destination_path(path: &str) -> Result<(), ValidationError> {
        // Check for directory traversal attempts
        if path.contains("..") {
            return Err(ValidationError::DirectoryTraversalAttempt {
                path: path.to_string(),
            });
        }

        // Check for absolute paths (should be relative)
        if path.starts_with('/') || (cfg!(windows) && path.len() > 1 && path.chars().nth(1) == Some(':')) {
            return Err(ValidationError::InvalidDestinationPath {
                path: path.to_string(),
                reason: "Path must be relative".to_string(),
            });
        }

        // Check for empty or invalid paths
        if path.is_empty() {
            return Err(ValidationError::InvalidDestinationPath {
                path: path.to_string(),
                reason: "Path cannot be empty".to_string(),
            });
        }

        // Check for paths with null bytes
        if path.contains('\0') {
            return Err(ValidationError::InvalidDestinationPath {
                path: path.to_string(),
                reason: "Path contains null bytes".to_string(),
            });
        }

        Ok(())
    }

    /// Validate destination file sizes are reasonable
    pub fn validate_destination_size(path: &str, size_bytes: u64) -> Result<(), ValidationError> {
        if size_bytes > Self::MAX_REASONABLE_FILE_SIZE {
            return Err(ValidationError::UnreasonableFileSize {
                path: path.to_string(),
                size: size_bytes,
            });
        }
        Ok(())
    }
    
    /// Check for overlapping destinations within a single chunk
    pub fn validate_no_overlaps(
        chunk_idx: u32,
        destinations: &[pb::ChunkTarget],
    ) -> Result<(), ValidationError> {
        // Group destinations by file path
        let mut file_destinations: std::collections::HashMap<&str, Vec<(usize, &pb::ChunkTarget)>> = 
            std::collections::HashMap::new();
        
        for (idx, dest) in destinations.iter().enumerate() {
            file_destinations.entry(&dest.path)
                .or_insert_with(Vec::new)
                .push((idx, dest));
        }
        
        // Check for overlaps within each file
        for (file_path, file_dests) in file_destinations {
            if file_dests.len() > 1 {
                // Sort by position for overlap detection
                let mut sorted_dests = file_dests;
                sorted_dests.sort_by_key(|(_, dest)| dest.position_bytes);
                
                for i in 0..sorted_dests.len() - 1 {
                    let (idx1, dest1) = sorted_dests[i];
                    let (idx2, dest2) = sorted_dests[i + 1];
                    
                    let end1 = dest1.position_bytes + dest1.size_bytes;
                    let start2 = dest2.position_bytes;
                    
                    if end1 > start2 {
                        warn!(
                            "Overlap detected in chunk {} file '{}': dest {} ({}..{}) overlaps with dest {} ({}..{})",
                            chunk_idx, file_path, idx1, dest1.position_bytes, end1, idx2, start2, dest2.position_bytes + dest2.size_bytes
                        );
                        return Err(ValidationError::DestinationOverlap {
                            chunk_idx,
                            dest1: idx1,
                            dest2: idx2,
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Enhanced error reporting for chunk processing failures
#[derive(Debug, Clone)]
pub struct ChunkProcessingError {
    pub chunk_idx: u32,
    pub error_type: ChunkErrorType,
    pub error_message: String,
    pub recovery_suggestions: Vec<String>,
    pub context: ChunkErrorContext,
}

#[derive(Debug, Clone)]
pub enum ChunkErrorType {
    DownloadFailure,
    DecompressionFailure,
    ChecksumMismatch,
    ValidationFailure,
    FileWriteFailure,
    SecurityViolation,
}

#[derive(Debug, Clone)]
pub struct ChunkErrorContext {
    pub chunk_size: Option<u64>,
    pub destinations_count: usize,
    pub decompressed_size: Option<usize>,
    pub file_paths: Vec<String>,
}

impl ChunkProcessingError {
    pub fn new(
        chunk_idx: u32,
        error_type: ChunkErrorType,
        error_message: String,
        context: ChunkErrorContext,
    ) -> Self {
        let recovery_suggestions = Self::generate_recovery_suggestions(&error_type, &context);
        
        Self {
            chunk_idx,
            error_type,
            error_message,
            recovery_suggestions,
            context,
        }
    }

    fn generate_recovery_suggestions(
        error_type: &ChunkErrorType,
        context: &ChunkErrorContext,
    ) -> Vec<String> {
        match error_type {
            ChunkErrorType::DownloadFailure => vec![
                "Check your internet connection".to_string(),
                "Verify the download URL is still valid".to_string(),
                "Try resuming the download after a short delay".to_string(),
                "Check if the server is experiencing issues".to_string(),
            ],
            ChunkErrorType::DecompressionFailure => vec![
                "The chunk data may be corrupted during download".to_string(),
                "Try re-downloading this specific chunk".to_string(),
                "Verify the compression format is supported".to_string(),
                "Check available disk space for decompression".to_string(),
            ],
            ChunkErrorType::ChecksumMismatch => vec![
                "The chunk data was corrupted during download or decompression".to_string(),
                "Re-download the chunk to get a fresh copy".to_string(),
                "Check network stability during download".to_string(),
                "Verify the source data integrity on the server".to_string(),
            ],
            ChunkErrorType::ValidationFailure => vec![
                "The chunk metadata may be malformed".to_string(),
                "Check if the dataset version is compatible".to_string(),
                "Try downloading a different version of the dataset".to_string(),
                "Report this issue to the dataset provider".to_string(),
            ],
            ChunkErrorType::FileWriteFailure => vec![
                format!("Check write permissions for files: {}", context.file_paths.join(", ")),
                "Verify sufficient disk space is available".to_string(),
                "Ensure the output directory is writable".to_string(),
                "Check if any files are locked by other processes".to_string(),
            ],
            ChunkErrorType::SecurityViolation => vec![
                "The chunk contains potentially unsafe file paths".to_string(),
                "This may indicate a malicious or corrupted dataset".to_string(),
                "Do not proceed with this download".to_string(),
                "Report this security issue to the dataset provider".to_string(),
            ],
        }
    }

    pub fn to_detailed_string(&self) -> String {
        let mut result = format!(
            "Chunk {} processing failed ({}): {}\n",
            self.chunk_idx,
            self.error_type_string(),
            self.error_message
        );

        result.push_str(&format!(
            "Context: {} destinations, chunk size: {}, decompressed size: {}\n",
            self.context.destinations_count,
            self.context.chunk_size.map_or("unknown".to_string(), |s| s.to_string()),
            self.context.decompressed_size.map_or("unknown".to_string(), |s| s.to_string())
        ));

        if !self.context.file_paths.is_empty() {
            result.push_str(&format!("Affected files: {}\n", self.context.file_paths.join(", ")));
        }

        result.push_str("\nRecovery suggestions:\n");
        for (i, suggestion) in self.recovery_suggestions.iter().enumerate() {
            result.push_str(&format!("  {}. {}\n", i + 1, suggestion));
        }

        result
    }

    fn error_type_string(&self) -> &'static str {
        match self.error_type {
            ChunkErrorType::DownloadFailure => "Download Failure",
            ChunkErrorType::DecompressionFailure => "Decompression Failure",
            ChunkErrorType::ChecksumMismatch => "Checksum Mismatch",
            ChunkErrorType::ValidationFailure => "Validation Failure",
            ChunkErrorType::FileWriteFailure => "File Write Failure",
            ChunkErrorType::SecurityViolation => "Security Violation",
        }
    }
}

pub struct SnapshotDownloader {
    config: Arc<Mutex<SnapshotConfig>>,
    output_dir: PathBuf,
    workers: usize,
    max_connections: usize,
}

impl SnapshotDownloader {
    pub fn new(config: SnapshotConfig, download_config: DownloadConfig) -> Result<Self> {
        // Validate download configuration
        download_config.validate().with_context(|| "Invalid download configuration")?;
        
        // Ensure output directory exists
        std::fs::create_dir_all(&download_config.output_dir).with_context(|| {
            format!(
                "Failed to create output directory: {}",
                download_config.output_dir.display()
            )
        })?;

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
        let metadata_dir = self.output_dir.join(".snapper");
        fs::create_dir_all(&metadata_dir)
            .await
            .with_context(|| "Failed to create metadata directory")?;

        // Step 1: Get download metadata from API service
        let api_service = ApiService::new((*self.config.lock().await).clone());
        let download_info = api_service
            .get_download_metadata_by_store_key(archive_id)
            .await?;
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
            return Err(eyre::anyhow!(
                "No download metadata found. Cannot resume download."
            ));
        }

        // Load existing metadata
        let metadata_content = fs::read_to_string(&metadata_path)
            .await
            .with_context(|| "Failed to read download metadata")?;
        let (metadata, archive_id): (DownloadMetadata, String) =
            serde_json::from_str(&metadata_content)
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
        let metadata_content = fs::read_to_string(&metadata_path)
            .await
            .with_context(|| "Failed to read download metadata")?;
        let (metadata, _archive_id): (DownloadMetadata, String) =
            serde_json::from_str(&metadata_content)
                .with_context(|| "Failed to parse download metadata")?;

        // Check progress
        if progress_path.exists() {
            let progress_content = fs::read_to_string(&progress_path)
                .await
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

                    let downloaded_bytes = (progress.current as f64 / progress.total as f64
                        * metadata.total_size as f64)
                        as u64;

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
                }
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
        info!(
            "Starting to download {} chunks with {} workers and {} max HTTP connections",
            metadata.chunks, self.workers, self.max_connections
        );

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

        let archive_uuid =
            archive_uuid.ok_or_else(|| anyhow!("Archive UUID not found for {}", archive_id))?;

        // Get signed URLs for all chunks in batches (API limit is 100 chunks per request)
        const MAX_CHUNKS_PER_REQUEST: u32 = 100;
        let mut all_chunks = Vec::new();

        for batch_start in (0..metadata.chunks).step_by(MAX_CHUNKS_PER_REQUEST as usize) {
            let batch_end = (batch_start + MAX_CHUNKS_PER_REQUEST).min(metadata.chunks);
            let chunk_indexes: Vec<u32> = (batch_start..batch_end).collect();

            info!("Requesting chunk batch {}-{}", batch_start, batch_end - 1);
            let batch_chunks = api_service
                .get_download_chunks(&archive_uuid, metadata.data_version, chunk_indexes)
                .await?;
            all_chunks.extend(batch_chunks);
        }

        let chunks = all_chunks;

        // Create HTTP client
        let client = Client::new();
        
        // Initialize progress tracker
        let progress_tracker = Arc::new(Mutex::new(DownloadProgressTracker::new(
            metadata.chunks,
            metadata.total_size,
        )));
        
        // Initialize size monitor
        let size_monitor = Arc::new(Mutex::new(SizeMonitor::new(
            self.output_dir.clone(),
            metadata.total_size,
        )));

        // Download chunks in parallel using semaphore for concurrency control
        let worker_semaphore = Arc::new(tokio::sync::Semaphore::new(self.workers));
        // Create separate semaphore for HTTP connections
        let http_semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_connections));
        let mut handles = Vec::new();

        for (chunk_idx, chunk) in chunks.into_iter().enumerate() {
            let client = client.clone();
            let worker_semaphore = worker_semaphore.clone();
            let http_semaphore = http_semaphore.clone();
            let output_dir = self.output_dir.clone();
            let chunk_idx = chunk_idx as u32;
            let total_chunks = metadata.chunks;
            let config = self.config.clone();
            let compression = metadata.compression.clone();
            let progress_tracker = progress_tracker.clone();
            let size_monitor = size_monitor.clone();

            let handle = tokio::spawn(async move {
                let _worker_permit = worker_semaphore.acquire().await?;

                // Skip if no URL (blueprint chunks)
                let url = match &chunk.url {
                    Some(url) => url,
                    None => {
                        info!("Skipping blueprint chunk {}", chunk_idx);
                        return Ok::<(), eyre::Error>(());
                    }
                };

                // Download the chunk data with HTTP connection limiting
                let chunk_data = {
                    // Acquire HTTP connection permit before making request
                    let _http_permit = http_semaphore.acquire().await?;
                    
                    let response = client
                        .get(url)
                        .send()
                        .await
                        .with_context(|| format!("Failed to download chunk {}", chunk_idx))?;

                    let chunk_data = response
                        .bytes()
                        .await
                        .with_context(|| format!("Failed to read chunk {} data", chunk_idx))?;
                    
                    // HTTP permit is automatically released when _http_permit goes out of scope
                    chunk_data
                };

                // Decompress if needed (assuming ZSTD compression)
                let decompressed_data =
                    if let Some(babel_api::engine::Compression::ZSTD(_)) = &compression {
                        // Use streaming decompressor to handle unknown decompressed size
                        zstd::stream::decode_all(&chunk_data[..])
                            .with_context(|| format!("Failed to decompress chunk {}", chunk_idx))?
                    } else {
                        chunk_data.to_vec()
                    };

                // Enhanced validation with comprehensive checks
                let error_context = ChunkErrorContext {
                    chunk_size: Some(chunk.size),
                    destinations_count: chunk.destinations.len(),
                    decompressed_size: Some(decompressed_data.len()),
                    file_paths: chunk.destinations.iter().map(|d| d.path.clone()).collect(),
                };

                // Comprehensive destination validation (includes security checks)
                if let Err(validation_error) = DestinationValidator::validate_destinations_comprehensive(chunk_idx, &chunk.destinations, decompressed_data.len()) {
                    let processing_error = ChunkProcessingError::new(
                        chunk_idx,
                        match validation_error {
                            ValidationError::DirectoryTraversalAttempt { .. } => ChunkErrorType::SecurityViolation,
                            ValidationError::InvalidDestinationPath { .. } => ChunkErrorType::SecurityViolation,
                            ValidationError::UnreasonableFileSize { .. } => ChunkErrorType::ValidationFailure,
                            _ => ChunkErrorType::ValidationFailure,
                        },
                        validation_error.to_string(),
                        error_context.clone(),
                    );
                    error!("{}", processing_error.to_detailed_string());
                    return Err(validation_error.into());
                }

                // Optional checksum verification after decompression
                if let Err(checksum_error) = ChecksumVerifier::verify_chunk_integrity(chunk_idx, &decompressed_data, &chunk.checksum) {
                    let processing_error = ChunkProcessingError::new(
                        chunk_idx,
                        ChunkErrorType::ChecksumMismatch,
                        checksum_error.to_string(),
                        error_context.clone(),
                    );
                    error!("{}", processing_error.to_detailed_string());
                    return Err(checksum_error.into());
                }
                
                // Initialize chunk write logger
                let mut logger = ChunkWriteLogger::new(chunk_idx, decompressed_data.len());
                
                info!(
                    "Processing chunk {}: {} bytes decompressed, {} destinations",
                    chunk_idx, decompressed_data.len(), chunk.destinations.len()
                );

                // Write decompressed data to files based on chunk destinations
                // Track offset within decompressed data for proper slicing
                let mut offset: usize = 0;
                
                for destination in &chunk.destinations {
                    let file_path = output_dir.join(&destination.path);

                    // Handle malformed chunks: check for missing or zero size_bytes
                    if destination.size_bytes == 0 {
                        tracing::warn!(
                            "Chunk {} destination '{}' has zero size_bytes, skipping",
                            chunk_idx, destination.path
                        );
                        continue;
                    }

                    // Create parent directories
                    if let Some(parent) = file_path.parent() {
                        fs::create_dir_all(parent).await.with_context(|| {
                            format!("Failed to create directory for {}", destination.path)
                        })?;
                    }

                    // Open file for writing (create or append)
                    let mut file = fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(&file_path)
                        .await
                        .with_context(|| format!("Failed to open file {}", destination.path))?;

                    // Seek to the correct position
                    file.seek(std::io::SeekFrom::Start(destination.position_bytes))
                        .await
                        .with_context(|| format!("Failed to seek in file {}", destination.path))?;

                    // Calculate slice bounds for this destination
                    let len = destination.size_bytes as usize;
                    
                    // Enhanced bounds check with clear error messages
                    if offset + len > decompressed_data.len() {
                        return Err(anyhow!(
                            "Chunk {} destination '{}' bounds check failed: \
                             requesting bytes {}..{} but decompressed data only has {} bytes. \
                             This indicates malformed chunk metadata or corruption.",
                            chunk_idx, destination.path, offset, offset + len, decompressed_data.len()
                        ));
                    }

                    // Write only the slice intended for this destination
                    let slice = &decompressed_data[offset..offset + len];
                    file.write_all(slice)
                        .await
                        .with_context(|| format!(
                            "Failed to write {} bytes to file '{}' at position {}",
                            len, destination.path, destination.position_bytes
                        ))?;

                    file.flush()
                        .await
                        .with_context(|| format!("Failed to flush file {}", destination.path))?;
                    
                    // Log the destination write
                    logger.log_destination_write(&destination.path, destination.position_bytes, destination.size_bytes, offset);
                    
                    // Increment offset for next destination
                    offset += len;
                }

                // Validate that all chunk data was consumed
                if offset != decompressed_data.len() {
                    let warning_msg = format!(
                        "Chunk {} data not fully consumed: processed {} bytes, total {} bytes",
                        chunk_idx, offset, decompressed_data.len()
                    );
                    tracing::warn!("{}", warning_msg);
                    
                    // Return error for incomplete consumption as this indicates a data integrity issue
                    return Err(anyhow!(
                        "Chunk {} incomplete consumption: {} bytes remaining",
                        chunk_idx, decompressed_data.len() - offset
                    ));
                }
                
                // Validate chunk consumption using logger
                logger.validate_chunk_consumption()
                    .with_context(|| format!("Chunk {} validation failed", chunk_idx))?;
                
                debug!(
                    "Chunk {} processed successfully: {} destinations, {} bytes total",
                    chunk_idx, logger.destinations_written.len(), decompressed_data.len()
                );
                
                // Update cumulative progress tracker
                {
                    let mut tracker = progress_tracker.lock().await;
                    tracker.update_chunk_processed(decompressed_data.len() as u64);
                }
                
                // Update size monitor and check for warnings
                {
                    let mut monitor = size_monitor.lock().await;
                    monitor.track_correct_write(decompressed_data.len() as u64);
                    
                    // Check for size warnings every 10 chunks or on the last chunk
                    if (chunk_idx + 1) % 10 == 0 || chunk_idx + 1 == total_chunks {
                        if monitor.should_warn_user().await.unwrap_or(false) {
                            warn!("Consider monitoring disk space - download may be consuming more space than expected");
                        }
                        
                        // Log size status periodically
                        if (chunk_idx + 1) % 50 == 0 || chunk_idx + 1 == total_chunks {
                            monitor.log_size_status().await.unwrap_or_else(|e| {
                                warn!("Failed to log size status: {}", e);
                            });
                        }
                    }
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
                    workers: 1,         // Dummy value
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
            handle
                .await
                .with_context(|| "Failed to join download task")?
                .with_context(|| "Download task failed")?;
        }

        Ok(())
    }

    async fn save_metadata(&self, metadata: &DownloadMetadata, archive_id: &str) -> Result<()> {
        let metadata_path = self.output_dir.join(METADATA_FILENAME);
        let data = (metadata, archive_id);
        let content = serde_json::to_string_pretty(&data)?;

        fs::write(&metadata_path, content)
            .await
            .with_context(|| "Failed to save download metadata")?;

        Ok(())
    }

    async fn save_progress(&self, progress: &JobProgress) -> Result<()> {
        let progress_path = self.output_dir.join(PROGRESS_FILENAME);
        let content = serde_json::to_string_pretty(progress)?;

        fs::write(&progress_path, content)
            .await
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

    #[test]
    fn test_destination_validation() {
        // Test destination size validation
        let destinations = vec![
            pb::ChunkTarget {
                path: "file1.dat".to_string(),
                position_bytes: 0,
                size_bytes: 100,
            },
            pb::ChunkTarget {
                path: "file2.dat".to_string(),
                position_bytes: 0,
                size_bytes: 50,
            },
        ];

        // Should pass when sizes match
        assert!(DestinationValidator::validate_destination_sizes(0, &destinations, 150).is_ok());

        // Should fail when sizes don't match
        assert!(DestinationValidator::validate_destination_sizes(0, &destinations, 200).is_err());

        // Test overlap validation
        let overlapping_destinations = vec![
            pb::ChunkTarget {
                path: "file1.dat".to_string(),
                position_bytes: 0,
                size_bytes: 100,
            },
            pb::ChunkTarget {
                path: "file1.dat".to_string(),
                position_bytes: 50,  // Overlaps with first destination
                size_bytes: 100,
            },
        ];

        assert!(DestinationValidator::validate_no_overlaps(0, &overlapping_destinations).is_err());

        // Non-overlapping should pass
        let non_overlapping_destinations = vec![
            pb::ChunkTarget {
                path: "file1.dat".to_string(),
                position_bytes: 0,
                size_bytes: 100,
            },
            pb::ChunkTarget {
                path: "file1.dat".to_string(),
                position_bytes: 100,  // No overlap
                size_bytes: 50,
            },
        ];

        assert!(DestinationValidator::validate_no_overlaps(0, &non_overlapping_destinations).is_ok());
    }

    #[test]
    fn test_chunk_write_logger() {
        let mut logger = ChunkWriteLogger::new(0, 150);
        
        logger.log_destination_write("file1.dat", 0, 100, 0);
        logger.log_destination_write("file2.dat", 0, 50, 100);
        
        // Should validate successfully when all data is consumed
        assert!(logger.validate_chunk_consumption().is_ok());
        
        // Test with incomplete consumption
        let mut incomplete_logger = ChunkWriteLogger::new(1, 200);
        incomplete_logger.log_destination_write("file1.dat", 0, 100, 0);
        
        // Should fail validation when not all data is consumed
        assert!(incomplete_logger.validate_chunk_consumption().is_err());
    }

    #[tokio::test]
    async fn test_size_monitor() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_dir = temp_dir.path().to_path_buf();
        
        let mut monitor = SizeMonitor::new(output_dir.clone(), 1000);
        
        // Test tracking correct writes
        monitor.track_correct_write(500);
        assert_eq!(monitor.calculate_inflation_ratio(), 0.5);
        
        // Test disk usage calculation with empty directory
        let disk_usage = monitor.get_current_disk_usage().await?;
        assert_eq!(disk_usage, 0);
        
        Ok(())
    }

    #[test]
    fn test_checksum_verification() {
        let test_data = b"Hello, World!";
        let chunk_idx = 0;

        // Test SHA256 checksum verification
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(test_data);
        let expected_hash = hasher.finalize().to_vec();

        let checksum = pb::Checksum {
            checksum: Some(pb::checksum::Checksum::Sha256(expected_hash)),
        };

        // Should pass with correct checksum
        assert!(ChecksumVerifier::verify_chunk_integrity(chunk_idx, test_data, &Some(checksum.clone())).is_ok());

        // Should fail with incorrect checksum
        let wrong_checksum = pb::Checksum {
            checksum: Some(pb::checksum::Checksum::Sha256(vec![0u8; 32])),
        };
        assert!(ChecksumVerifier::verify_chunk_integrity(chunk_idx, test_data, &Some(wrong_checksum)).is_err());

        // Should pass with no checksum (optional verification)
        assert!(ChecksumVerifier::verify_chunk_integrity(chunk_idx, test_data, &None).is_ok());
    }

    #[test]
    fn test_comprehensive_destination_validation() {
        // Test valid destinations
        let valid_destinations = vec![
            pb::ChunkTarget {
                path: "data/file1.dat".to_string(),
                position_bytes: 0,
                size_bytes: 100,
            },
            pb::ChunkTarget {
                path: "data/file2.dat".to_string(),
                position_bytes: 0,
                size_bytes: 50,
            },
        ];

        assert!(DestinationValidator::validate_destinations_comprehensive(0, &valid_destinations, 150).is_ok());

        // Test directory traversal attempt
        let malicious_destinations = vec![
            pb::ChunkTarget {
                path: "../../../etc/passwd".to_string(),
                position_bytes: 0,
                size_bytes: 100,
            },
        ];

        assert!(DestinationValidator::validate_destinations_comprehensive(0, &malicious_destinations, 100).is_err());

        // Test absolute path
        let absolute_path_destinations = vec![
            pb::ChunkTarget {
                path: "/etc/passwd".to_string(),
                position_bytes: 0,
                size_bytes: 100,
            },
        ];

        assert!(DestinationValidator::validate_destinations_comprehensive(0, &absolute_path_destinations, 100).is_err());

        // Test unreasonable file size
        let large_file_destinations = vec![
            pb::ChunkTarget {
                path: "huge_file.dat".to_string(),
                position_bytes: 0,
                size_bytes: 20 * 1024 * 1024 * 1024, // 20 GB
            },
        ];

        assert!(DestinationValidator::validate_destinations_comprehensive(0, &large_file_destinations, 20 * 1024 * 1024 * 1024).is_err());
    }

    #[test]
    fn test_chunk_processing_error_reporting() {
        let context = ChunkErrorContext {
            chunk_size: Some(1024),
            destinations_count: 2,
            decompressed_size: Some(1024),
            file_paths: vec!["file1.dat".to_string(), "file2.dat".to_string()],
        };

        let error = ChunkProcessingError::new(
            42,
            ChunkErrorType::ChecksumMismatch,
            "Checksum verification failed".to_string(),
            context,
        );

        let detailed_string = error.to_detailed_string();
        assert!(detailed_string.contains("Chunk 42"));
        assert!(detailed_string.contains("Checksum Mismatch"));
        assert!(detailed_string.contains("Recovery suggestions"));
        assert!(detailed_string.contains("file1.dat, file2.dat"));
    }

    #[test]
    fn test_destination_path_validation() {
        // Valid paths
        assert!(DestinationValidator::validate_destination_path("data/file.dat").is_ok());
        assert!(DestinationValidator::validate_destination_path("subdir/file.txt").is_ok());

        // Invalid paths
        assert!(DestinationValidator::validate_destination_path("../file.dat").is_err());
        assert!(DestinationValidator::validate_destination_path("/absolute/path").is_err());
        assert!(DestinationValidator::validate_destination_path("").is_err());
        assert!(DestinationValidator::validate_destination_path("file\0with\0nulls").is_err());
    }

    #[test]
    fn test_destination_size_validation() {
        // Valid sizes
        assert!(DestinationValidator::validate_destination_size("file.dat", 1024).is_ok());
        assert!(DestinationValidator::validate_destination_size("file.dat", 1024 * 1024 * 1024).is_ok()); // 1 GB

        // Invalid sizes
        assert!(DestinationValidator::validate_destination_size("huge.dat", 20 * 1024 * 1024 * 1024).is_err()); // 20 GB
    }
}
