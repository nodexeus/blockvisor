use crate::engine::{self, JobConfig, JobType, PosixSignal, RestartConfig};
use eyre::ensure;
use rhai::Dynamic;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

pub const COLD_INIT_JOB_NAME: &str = "download";
pub const DOWNLOAD_JOB_NAME: &str = "download";
pub const DOWNLOADING_STATE_NAME: &str = "downloading";
pub const UPLOAD_JOB_NAME: &str = "upload";
pub const UPLOADING_STATE_NAME: &str = "uploading";
pub const STARTING_STATE_NAME: &str = "starting";

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PluginConfig {
    /// List of configuration files to be rendered from template with provided params.
    pub config_files: Option<Vec<ConfigFile>>,
    /// List of auxiliary services that doesn't need init steps and doesn't use protocol data.
    pub aux_services: Option<Vec<AuxService>>,
    /// Node init actions.
    pub init: Option<Actions>,
    /// Download configuration.
    pub download: Option<Download>,
    /// Alternative download configuration.
    pub alternative_download: Option<AlternativeDownload>,
    /// List of post-download jobs.
    pub post_download: Option<Vec<Job>>,
    /// Alternative protocol data initialization.
    /// It is fallback job, run only if neither regular archive nor `alternative_download` is available.
    /// NOTE: `post_download` is not run after `cold_init`.
    pub cold_init: Option<ColdInit>,
    /// List of protocol services.
    pub services: Vec<Service>,
    /// List of pre-upload actions.
    pub pre_upload: Option<Actions>,
    /// Upload configuration.
    pub upload: Option<Upload>,
    /// List of post-upload jobs.
    pub post_upload: Option<Vec<Job>>,
    /// List of tasks to be scheduled on init.
    pub scheduled: Option<Vec<Task>>,
}

impl PluginConfig {
    pub fn validate(&self) -> eyre::Result<()> {
        // Scheduled tasks name uniqueness
        if let Some(scheduled) = &self.scheduled {
            let mut unique = HashSet::new();
            ensure!(
                scheduled.iter().all(|task| unique.insert(&task.name)),
                "Scheduled tasks names are not unique"
            );
        }
        // Jobs name uniqueness
        let mut unique = HashSet::new();
        unique.insert(UPLOAD_JOB_NAME);
        unique.insert(DOWNLOAD_JOB_NAME);
        if let Some(aux_services) = &self.aux_services {
            ensure!(
                aux_services
                    .iter()
                    .all(|service| unique.insert(&service.name)),
                "Auxiliary services names are not unique"
            );
        }
        if let Some(init) = &self.init {
            ensure!(
                init.jobs.iter().all(|job| unique.insert(&job.name)),
                "Init jobs names are not unique"
            );
        }
        if let Some(post_download) = &self.post_download {
            ensure!(
                post_download.iter().all(|job| unique.insert(&job.name)),
                "Post-download jobs names are not unique"
            );
        }
        ensure!(
            self.services
                .iter()
                .all(|service| unique.insert(&service.name)),
            "Services names are not unique"
        );
        
        // Validate service configurations (R1 requirements)
        for service in &self.services {
            // R1.2: Validate store_key is present for archived services
            if service.archive && service.store_key.is_none() {
                return Err(eyre::anyhow!(
                    "Service '{}' has archive=true but no store_key specified", 
                    service.name
                ));
            }
            
            // R1.1: Validate data_dir format if specified
            if let Some(data_dir) = &service.data_dir {
                ensure!(
                    !data_dir.is_empty() && !data_dir.starts_with('/') && !data_dir.contains(".."),
                    "Service '{}' data_dir '{}' must be a relative path without '..' components",
                    service.name,
                    data_dir
                );
            }
            
            // R1.2: Validate store_key format if specified
            if let Some(store_key) = &service.store_key {
                ensure!(
                    !store_key.is_empty() && store_key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
                    "Service '{}' store_key '{}' must contain only alphanumeric characters, hyphens, and underscores",
                    service.name,
                    store_key
                );
            }
        }
        if let Some(pre_upload) = &self.pre_upload {
            ensure!(
                pre_upload.jobs.iter().all(|job| unique.insert(&job.name)),
                "Pre-upload jobs names are not unique"
            );
        }
        if let Some(post_upload) = &self.post_upload {
            ensure!(
                post_upload.iter().all(|job| unique.insert(&job.name)),
                "Post-upload jobs names are not unique"
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    /// It assumes that file pointed by `template` argument exists.
    pub template: PathBuf,
    /// File pointed by `destination` path will be overwritten if exists.
    pub destination: PathBuf,
    /// Map object serializable to JSON.
    /// See [Tera Docs](https://keats.github.io/tera/docs/#templates) for details on templating syntax.
    pub params: Dynamic,
}

/// Definition of auxiliary service that doesn't need init steps and doesn't use protocol data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuxService {
    /// Name of service. This will be used as Job name, so make sure it is unique enough,
    /// to not collide with other jobs.
    pub name: String,
    /// Sh script body. It shall be blocking (foreground) process.
    pub run_sh: String,
    /// Service restart config.
    pub restart_config: Option<RestartConfig>,
    /// Job shutdown timeout - how long it may take to gracefully shutdown the job.
    /// After given time job won't be killed, but babel will rise the error.
    /// If not set default to 60s.
    pub shutdown_timeout_secs: Option<u64>,
    /// POSIX signal that will be sent to child processes on job shutdown.
    /// See [man7](https://man7.org/linux/man-pages/man7/signal.7.html) for possible values.
    /// If not set default to `SIGTERM`.
    pub shutdown_signal: Option<PosixSignal>,
    /// Run job as a different user.
    pub run_as: Option<String>,
    /// Capacity of log buffer (in megabytes).
    pub log_buffer_capacity_mb: Option<usize>,
    /// Prepend timestamp to each log, or not.
    pub log_timestamp: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    /// Unique name of the task.
    pub name: String,
    /// Cron schedule expression.
    pub schedule: String,
    /// Function name to be executed according to schedule.
    pub function: String,
    /// Parameter to ba passed to function.
    pub param: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Actions {
    /// List of sh commands to be executed first.
    pub commands: Vec<String>,
    /// List of long-running tasks (aka jobs) that must be finished before protocol services start.
    pub jobs: Vec<Job>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RestartPolicy {
    /// Indicates that this job will never be restarted, whether succeeded or not - appropriate for jobs
    /// that can't be simply restarted on failure (e.g. need some manual actions).
    Never,
    /// Job is restarted only if `exit_code != 0`.
    OnFailure(RestartConfig),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    /// Unique job name.
    pub name: String,
    /// Sh script body. It shall be blocking (foreground) process.
    pub run_sh: String,
    /// Job restart policy.
    pub restart: Option<RestartPolicy>,
    /// Job shutdown timeout - how long it may take to gracefully shutdown the job.
    /// After given time job won't be killed, but babel will rise the error.
    /// If not set default to 60s.
    pub shutdown_timeout_secs: Option<u64>,
    /// POSIX signal that will be sent to child processes on job shutdown.
    /// See [man7](https://man7.org/linux/man-pages/man7/signal.7.html) for possible values.
    /// If not set default to `SIGTERM`.
    pub shutdown_signal: Option<PosixSignal>,
    /// List of job names that this job needs to be finished before start.
    pub needs: Option<Vec<String>>,
    /// Run job as a different user.
    pub run_as: Option<String>,
    /// Capacity of log buffer (in megabytes).
    pub log_buffer_capacity_mb: Option<usize>,
    /// Prepend timestamp to each log, or not.
    pub log_timestamp: Option<bool>,
    /// Flag indicating if job uses protocol data.
    /// Job that uses protocol data, automatically create lock that prevents
    /// re-initialization of the data after job is started.
    /// Default to `false`.
    pub use_protocol_data: Option<bool>,
    /// Indicate if job should run only once.
    /// One-time jobs never run again, even after node upgrade.
    pub one_time: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Service {
    /// Unique job name.
    pub name: String,
    /// Sh script body. It shall be blocking (foreground) process.
    pub run_sh: String,
    /// Service restart config.
    pub restart_config: Option<RestartConfig>,
    /// Job shutdown timeout - how long it may take to gracefully shutdown the job.
    /// After given time job won't be killed, but babel will rise the error.
    /// If not set default to 60s.
    pub shutdown_timeout_secs: Option<u64>,
    /// POSIX signal that will be sent to child processes on job shutdown.
    /// See [man7](https://man7.org/linux/man-pages/man7/signal.7.html) for possible values.
    /// If not set default to `SIGTERM`.
    pub shutdown_signal: Option<PosixSignal>,
    /// Run job as a different user.
    pub run_as: Option<String>,
    /// Flag indicating if service uses protocol data.
    /// Services that uses protocol data won't be started until all init steps and download
    /// is finished. They are also stopped while uploading protocol archive.
    /// Default to true if not set.
    #[serde(default = "default_use_protocol_data")]
    pub use_protocol_data: bool,
    /// Capacity of log buffer (in megabytes).
    pub log_buffer_capacity_mb: Option<usize>,
    /// Prepend timestamp to each log, or not.
    pub log_timestamp: Option<bool>,
    /// Data directory for this service relative to protocol_data_path.
    /// If not specified, defaults to the service name.
    /// R1.1: Per-client data directories
    pub data_dir: Option<String>,
    /// Store key identifier for this service's snapshot archive.
    /// Should be resolved from variant-specific configuration.
    /// R1.2: Individual store key identifiers
    pub store_key: Option<String>,
    /// Whether this service should be archived.
    /// R1.3: Archive control (defaults to true)
    #[serde(default = "default_archive")]
    pub archive: bool,
}
fn default_use_protocol_data() -> bool {
    true
}

fn default_archive() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Download {
    /// Download restart config.
    pub restart_config: Option<RestartConfig>,
    /// Maximum number of parallel opened connections.
    pub max_connections: Option<usize>,
    /// Maximum number of parallel workers.
    pub max_runners: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlternativeDownload {
    /// Sh script body. It shall be blocking (foreground) process.
    pub run_sh: String,
    /// AlternativeDownload restart config.
    pub restart_config: Option<RestartConfig>,
    /// Run job as a different user.
    pub run_as: Option<String>,
    /// Capacity of log buffer (in megabytes).
    pub log_buffer_capacity_mb: Option<usize>,
    /// Prepend timestamp to each log, or not.
    pub log_timestamp: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColdInit {
    /// Sh script body. It shall be blocking (foreground) process.
    pub run_sh: String,
    /// ColdInit restart config.
    pub restart_config: Option<RestartConfig>,
    /// Run job as a different user.
    pub run_as: Option<String>,
    /// Capacity of log buffer (in megabytes).
    pub log_buffer_capacity_mb: Option<usize>,
    /// Prepend timestamp to each log, or not.
    pub log_timestamp: Option<bool>,
    /// Indicate if job should run only once.
    /// One-time jobs never run again, even after node upgrade.
    pub one_time: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Upload {
    /// Upload restart config.
    pub restart_config: Option<RestartConfig>,
    /// List of exclude patterns. Files in `source` directory that match any of pattern,
    /// won't be taken into account.
    pub exclude: Option<Vec<String>>,
    /// Compression to be used on chunks.
    pub compression: Option<Compression>,
    /// Maximum number of parallel opened connections.
    pub max_connections: Option<usize>,
    /// Maximum number of parallel workers.
    pub max_runners: Option<usize>,
    /// Number of chunks that protocol data should be split into.
    /// Recommended chunk size is about 500MB.
    pub number_of_chunks: Option<u32>,
    /// Seconds after which presigned urls in generated `UploadManifest` may expire.
    pub url_expires_secs: Option<u32>,
    /// Version number for uploaded data. Auto-assigned if `None`.
    pub data_version: Option<u64>,
}

/// Type of compression used on chunk data.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Compression {
    NONE,
    ZSTD(i32),
}

pub fn build_job_config(job: Job) -> JobConfig {
    JobConfig {
        job_type: JobType::RunSh(job.run_sh),
        restart: job
            .restart
            .map(|restart| match restart {
                RestartPolicy::Never => engine::RestartPolicy::Never,
                RestartPolicy::OnFailure(config) => engine::RestartPolicy::OnFailure(config),
            })
            .unwrap_or(engine::RestartPolicy::Never),
        shutdown_timeout_secs: job.shutdown_timeout_secs,
        shutdown_signal: job.shutdown_signal,
        needs: job.needs,
        wait_for: None,
        run_as: job.run_as,
        log_buffer_capacity_mb: job.log_buffer_capacity_mb,
        log_timestamp: job.log_timestamp,
        use_protocol_data: job.use_protocol_data,
        one_time: job.one_time,
    }
}

pub fn build_download_job_config(download: Option<Download>, init_jobs: Vec<String>) -> JobConfig {
    const DEFAULT_RESTART_CONFIG: RestartConfig = RestartConfig {
        backoff_timeout_ms: 600_000,
        backoff_base_ms: 500,
        max_retries: Some(10),
    };
    if let Some(download) = download {
        JobConfig {
            job_type: JobType::Download {
                max_connections: download.max_connections,
                max_runners: download.max_runners,
            },
            restart: engine::RestartPolicy::OnFailure(
                download.restart_config.unwrap_or(DEFAULT_RESTART_CONFIG),
            ),
            shutdown_timeout_secs: None,
            shutdown_signal: None,
            needs: Some(init_jobs),
            wait_for: None,
            run_as: None,
            log_buffer_capacity_mb: None,
            log_timestamp: None,
            use_protocol_data: None,
            one_time: None,
        }
    } else {
        JobConfig {
            job_type: JobType::Download {
                max_connections: None,
                max_runners: None,
            },
            restart: engine::RestartPolicy::OnFailure(DEFAULT_RESTART_CONFIG),
            shutdown_timeout_secs: None,
            shutdown_signal: None,
            needs: Some(init_jobs),
            wait_for: None,
            run_as: None,
            log_buffer_capacity_mb: None,
            log_timestamp: None,
            use_protocol_data: None,
            one_time: None,
        }
    }
}

pub fn build_alternative_download_job_config(
    alternative_download: AlternativeDownload,
    init_jobs: Vec<String>,
) -> JobConfig {
    JobConfig {
        job_type: JobType::RunSh(alternative_download.run_sh),
        restart: if let Some(restart) = alternative_download.restart_config {
            engine::RestartPolicy::OnFailure(restart)
        } else {
            engine::RestartPolicy::Never
        },
        shutdown_timeout_secs: None,
        shutdown_signal: None,
        needs: Some(init_jobs),
        wait_for: None,
        run_as: alternative_download.run_as,
        log_buffer_capacity_mb: alternative_download.log_buffer_capacity_mb,
        log_timestamp: alternative_download.log_timestamp,
        use_protocol_data: None,
        one_time: None,
    }
}

pub fn build_cold_init_job_config(cold_init: ColdInit, init_jobs: Vec<String>) -> JobConfig {
    JobConfig {
        job_type: JobType::RunSh(cold_init.run_sh),
        restart: if let Some(restart) = cold_init.restart_config {
            engine::RestartPolicy::OnFailure(restart)
        } else {
            engine::RestartPolicy::Never
        },
        shutdown_timeout_secs: None,
        shutdown_signal: None,
        needs: Some(init_jobs),
        wait_for: None,
        run_as: cold_init.run_as,
        log_buffer_capacity_mb: cold_init.log_buffer_capacity_mb,
        log_timestamp: cold_init.log_timestamp,
        use_protocol_data: None,
        one_time: cold_init.one_time,
    }
}

pub fn build_service_job_config(
    service: Service,
    needs: Vec<String>,
    wait_for: Vec<String>,
) -> JobConfig {
    JobConfig {
        job_type: JobType::RunSh(service.run_sh),
        restart: engine::RestartPolicy::Always(service.restart_config.unwrap_or(RestartConfig {
            backoff_timeout_ms: 60_000,
            backoff_base_ms: 1_000,
            max_retries: None,
        })),
        shutdown_timeout_secs: service.shutdown_timeout_secs,
        shutdown_signal: service.shutdown_signal,
        needs: if service.use_protocol_data {
            Some(needs)
        } else {
            None
        },
        wait_for: if service.use_protocol_data {
            Some(wait_for)
        } else {
            None
        },
        run_as: service.run_as,
        log_buffer_capacity_mb: service.log_buffer_capacity_mb,
        log_timestamp: service.log_timestamp,
        use_protocol_data: Some(service.use_protocol_data),
        one_time: None,
    }
}

pub fn build_upload_job_config(value: Option<Upload>, pre_upload_jobs: Vec<String>) -> JobConfig {
    const DEFAULT_RESTART_CONFIG: RestartConfig = RestartConfig {
        backoff_timeout_ms: 600_000,
        backoff_base_ms: 500,
        max_retries: Some(10),
    };
    const DEFAULT_COMPRESSION: engine::Compression = engine::Compression::ZSTD(3);
    if let Some(upload) = value {
        JobConfig {
            job_type: JobType::Upload {
                exclude: upload.exclude,
                compression: match upload.compression {
                    None => Some(DEFAULT_COMPRESSION),
                    Some(Compression::ZSTD(level)) => Some(engine::Compression::ZSTD(level)),
                    Some(Compression::NONE) => None,
                },
                max_connections: upload.max_connections,
                max_runners: upload.max_runners,
                number_of_chunks: upload.number_of_chunks,
                url_expires_secs: upload.url_expires_secs,
                data_version: upload.data_version,
            },
            restart: engine::RestartPolicy::OnFailure(
                upload.restart_config.unwrap_or(DEFAULT_RESTART_CONFIG),
            ),
            shutdown_timeout_secs: None,
            shutdown_signal: None,
            needs: Some(pre_upload_jobs),
            wait_for: None,
            run_as: None,
            log_buffer_capacity_mb: None,
            log_timestamp: None,
            use_protocol_data: None,
            one_time: None,
        }
    } else {
        JobConfig {
            job_type: JobType::Upload {
                exclude: None,
                compression: Some(DEFAULT_COMPRESSION),
                max_connections: None,
                max_runners: None,
                number_of_chunks: None,
                url_expires_secs: None,
                data_version: None,
            },
            restart: engine::RestartPolicy::OnFailure(DEFAULT_RESTART_CONFIG),
            shutdown_timeout_secs: None,
            shutdown_signal: None,
            needs: Some(pre_upload_jobs),
            wait_for: None,
            run_as: None,
            log_buffer_capacity_mb: None,
            log_timestamp: None,
            use_protocol_data: None,
            one_time: None,
        }
    }
}

/// R2.4: Build job config for multi-client upload
pub fn build_multi_client_upload_job_config(value: Option<Upload>, pre_upload_jobs: Vec<String>) -> JobConfig {
    const DEFAULT_RESTART_CONFIG: RestartConfig = RestartConfig {
        backoff_timeout_ms: 600_000,
        backoff_base_ms: 500,
        max_retries: Some(10),
    };
    if let Some(upload) = value {
        JobConfig {
            job_type: JobType::MultiClientUpload {
                max_connections: upload.max_connections,
                max_runners: upload.max_runners,
                url_expires_secs: upload.url_expires_secs,
                data_version: upload.data_version,
            },
            restart: engine::RestartPolicy::OnFailure(
                upload.restart_config.unwrap_or(DEFAULT_RESTART_CONFIG),
            ),
            shutdown_timeout_secs: None,
            shutdown_signal: None,
            needs: Some(pre_upload_jobs),
            wait_for: None,
            run_as: None,
            log_buffer_capacity_mb: None,
            log_timestamp: None,
            use_protocol_data: None,
            one_time: None,
        }
    } else {
        JobConfig {
            job_type: JobType::MultiClientUpload {
                max_connections: None,
                max_runners: None,
                url_expires_secs: None,
                data_version: None,
            },
            restart: engine::RestartPolicy::OnFailure(DEFAULT_RESTART_CONFIG),
            shutdown_timeout_secs: None,
            shutdown_signal: None,
            needs: Some(pre_upload_jobs),
            wait_for: None,
            run_as: None,
            log_buffer_capacity_mb: None,
            log_timestamp: None,
            use_protocol_data: None,
            one_time: None,
        }
    }
}

/// R3.1: Build job config for multi-client download
pub fn build_multi_client_download_job_config(value: Option<Download>, init_jobs: Vec<String>) -> JobConfig {
    const DEFAULT_RESTART_CONFIG: RestartConfig = RestartConfig {
        backoff_timeout_ms: 600_000,
        backoff_base_ms: 500,
        max_retries: Some(10),
    };
    if let Some(download) = value {
        JobConfig {
            job_type: JobType::MultiClientDownload {
                max_connections: download.max_connections,
                max_runners: download.max_runners,
            },
            restart: engine::RestartPolicy::OnFailure(
                download.restart_config.unwrap_or(DEFAULT_RESTART_CONFIG),
            ),
            shutdown_timeout_secs: None,
            shutdown_signal: None,
            needs: Some(init_jobs),
            wait_for: None,
            run_as: None,
            log_buffer_capacity_mb: None,
            log_timestamp: None,
            use_protocol_data: None,
            one_time: None,
        }
    } else {
        JobConfig {
            job_type: JobType::MultiClientDownload {
                max_connections: None,
                max_runners: None,
            },
            restart: engine::RestartPolicy::OnFailure(DEFAULT_RESTART_CONFIG),
            shutdown_timeout_secs: None,
            shutdown_signal: None,
            needs: Some(init_jobs),
            wait_for: None,
            run_as: None,
            log_buffer_capacity_mb: None,
            log_timestamp: None,
            use_protocol_data: None,
            one_time: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_service_default_values() {
        let json = r#"{
            "name": "test_service",
            "run_sh": "/usr/bin/test"
        }"#;
        
        let service: Service = serde_json::from_str(json).unwrap();
        assert_eq!(service.name, "test_service");
        assert_eq!(service.run_sh, "/usr/bin/test");
        assert!(service.use_protocol_data); // default_use_protocol_data
        assert!(service.archive); // default_archive
        assert!(service.data_dir.is_none());
        assert!(service.store_key.is_none());
    }

    #[test]
    fn test_service_with_all_fields() {
        let json = r#"{
            "name": "reth",
            "run_sh": "/usr/bin/reth node",
            "use_protocol_data": false,
            "archive": false,
            "data_dir": "ethereum-data",
            "store_key": "ethereum-reth-mainnet-v1"
        }"#;
        
        let service: Service = serde_json::from_str(json).unwrap();
        assert_eq!(service.name, "reth");
        assert_eq!(service.run_sh, "/usr/bin/reth node");
        assert!(!service.use_protocol_data);
        assert!(!service.archive);
        assert_eq!(service.data_dir, Some("ethereum-data".to_string()));
        assert_eq!(service.store_key, Some("ethereum-reth-mainnet-v1".to_string()));
    }

    #[test]
    fn test_service_serialization() {
        let service = Service {
            name: "lighthouse".to_string(),
            run_sh: "/usr/bin/lighthouse bn".to_string(),
            restart_config: None,
            shutdown_timeout_secs: None,
            shutdown_signal: None,
            run_as: None,
            use_protocol_data: true,
            log_buffer_capacity_mb: None,
            log_timestamp: None,
            data_dir: Some("beacon".to_string()),
            store_key: Some("ethereum-lighthouse-mainnet-v1".to_string()),
            archive: true,
        };

        let json = serde_json::to_string(&service).unwrap();
        let deserialized: Service = serde_json::from_str(&json).unwrap();
        
        assert_eq!(service.name, deserialized.name);
        assert_eq!(service.data_dir, deserialized.data_dir);
        assert_eq!(service.store_key, deserialized.store_key);
        assert_eq!(service.archive, deserialized.archive);
    }

    #[test]
    fn test_plugin_config_with_multi_client_services() {
        let json = r#"{
            "services": [
                {
                    "name": "reth",
                    "run_sh": "/usr/bin/reth node",
                    "store_key": "ethereum-reth-mainnet-v1",
                    "archive": true
                },
                {
                    "name": "lighthouse",
                    "run_sh": "/usr/bin/lighthouse bn",
                    "data_dir": "beacon",
                    "store_key": "ethereum-lighthouse-mainnet-v1",
                    "archive": true
                },
                {
                    "name": "monitoring",
                    "run_sh": "/usr/bin/prometheus",
                    "archive": false
                }
            ]
        }"#;
        
        let config: PluginConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.services.len(), 3);
        
        // Check reth service
        let reth = &config.services[0];
        assert_eq!(reth.name, "reth");
        assert_eq!(reth.store_key, Some("ethereum-reth-mainnet-v1".to_string()));
        assert!(reth.archive);
        assert!(reth.data_dir.is_none()); // Should default to service name
        
        // Check lighthouse service
        let lighthouse = &config.services[1];
        assert_eq!(lighthouse.name, "lighthouse");
        assert_eq!(lighthouse.data_dir, Some("beacon".to_string()));
        assert_eq!(lighthouse.store_key, Some("ethereum-lighthouse-mainnet-v1".to_string()));
        assert!(lighthouse.archive);
        
        // Check monitoring service (not archived)
        let monitoring = &config.services[2];
        assert_eq!(monitoring.name, "monitoring");
        assert!(!monitoring.archive);
        assert!(monitoring.store_key.is_none());
    }

    #[test]
    fn test_validate_services() {
        let valid_service = Service {
            name: "valid_service".to_string(),
            run_sh: "/usr/bin/valid".to_string(),
            restart_config: None,
            shutdown_timeout_secs: None,
            shutdown_signal: None,
            run_as: None,
            use_protocol_data: true,
            log_buffer_capacity_mb: None,
            log_timestamp: None,
            data_dir: Some("valid-dir".to_string()),
            store_key: Some("valid-store-key-v1".to_string()),
            archive: true,
        };
        
        let config = PluginConfig {
            config_files: None,
            aux_services: None,
            init: None,
            download: None,
            alternative_download: None,
            post_download: None,
            cold_init: None,
            services: vec![valid_service],
            pre_upload: None,
            upload: None,
            post_upload: None,
            scheduled: None,
        };
        
        // Should not panic or fail
        assert!(config.services.len() == 1);
        assert!(config.services[0].archive);
        assert!(config.services[0].store_key.is_some());
    }

    #[test]
    fn test_default_functions() {
        assert!(default_use_protocol_data());
        assert!(default_archive());
    }

    #[test] 
    fn test_service_build_job_config() {
        let service = Service {
            name: "test_service".to_string(),
            run_sh: "/usr/bin/test".to_string(),
            restart_config: None,
            shutdown_timeout_secs: Some(30),
            shutdown_signal: None,
            run_as: None,
            use_protocol_data: true,
            log_buffer_capacity_mb: Some(10),
            log_timestamp: Some(true),
            data_dir: Some("test-data".to_string()),
            store_key: Some("test-store-key-v1".to_string()),
            archive: true,
        };
        
        let needs = vec!["init1".to_string(), "init2".to_string()];
        let wait_for = vec!["download".to_string()];
        
        let job_config = build_service_job_config(service, needs.clone(), wait_for.clone());
        
        assert!(matches!(job_config.job_type, JobType::RunSh(_)));
        assert!(matches!(job_config.restart, engine::RestartPolicy::Always(_)));
        assert_eq!(job_config.shutdown_timeout_secs, Some(30));
        assert_eq!(job_config.needs, Some(needs));
        assert_eq!(job_config.wait_for, Some(wait_for));
        assert_eq!(job_config.log_buffer_capacity_mb, Some(10));
        assert_eq!(job_config.log_timestamp, Some(true));
        assert_eq!(job_config.use_protocol_data, Some(true));
    }

    #[test]
    fn test_service_without_protocol_data() {
        let service = Service {
            name: "auxiliary_service".to_string(),
            run_sh: "/usr/bin/aux".to_string(),
            restart_config: None,
            shutdown_timeout_secs: None,
            shutdown_signal: None,
            run_as: None,
            use_protocol_data: false,
            log_buffer_capacity_mb: None,
            log_timestamp: None,
            data_dir: None,
            store_key: None,
            archive: false,
        };
        
        let needs = vec!["init1".to_string()];
        let wait_for = vec!["download".to_string()];
        
        let job_config = build_service_job_config(service, needs, wait_for);
        
        // Service that doesn't use protocol data shouldn't have needs or wait_for
        assert_eq!(job_config.needs, None);
        assert_eq!(job_config.wait_for, None);
        assert_eq!(job_config.use_protocol_data, Some(false));
    }
}
