use daemonize::Daemonize;
use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

/// Configuration for daemon mode
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    pub pid_file: PathBuf,
    pub log_file: PathBuf,
    pub working_dir: PathBuf,
}

impl DaemonConfig {
    /// Create a new daemon configuration for the given output directory
    pub fn new(output_dir: PathBuf) -> Self {
        let snapper_dir = output_dir.join(".snapper");
        Self {
            pid_file: snapper_dir.join("snapper.pid"),
            log_file: snapper_dir.join("snapper.log"),
            working_dir: output_dir,
        }
    }

    /// Ensure the .snapper directory exists
    pub fn ensure_directories(&self) -> Result<()> {
        let snapper_dir = self.pid_file.parent().unwrap();
        fs::create_dir_all(snapper_dir)
            .with_context(|| format!("Failed to create directory: {:?}", snapper_dir))?;
        Ok(())
    }
}

/// State information for a daemonized download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonState {
    pub pid: u32,
    pub start_time: SystemTime,
    pub protocol: String,
    pub client: String,
    pub network: String,
    pub output_dir: PathBuf,
}

impl DaemonState {
    /// Create a new daemon state
    pub fn new(
        pid: u32,
        protocol: String,
        client: String,
        network: String,
        output_dir: PathBuf,
    ) -> Self {
        Self {
            pid,
            start_time: SystemTime::now(),
            protocol,
            client,
            network,
            output_dir,
        }
    }

    /// Save the daemon state to a file
    pub fn save(&self, output_dir: &PathBuf) -> Result<()> {
        let state_file = output_dir.join(".snapper").join("daemon.json");
        let json = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize daemon state")?;
        fs::write(&state_file, json)
            .with_context(|| format!("Failed to write daemon state to {:?}", state_file))?;
        Ok(())
    }

    /// Load the daemon state from a file
    pub fn load(output_dir: &PathBuf) -> Result<Option<Self>> {
        let state_file = output_dir.join(".snapper").join("daemon.json");
        if !state_file.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&state_file)
            .with_context(|| format!("Failed to read daemon state from {:?}", state_file))?;
        let state: DaemonState = serde_json::from_str(&json)
            .with_context(|| "Failed to deserialize daemon state")?;
        Ok(Some(state))
    }

    /// Remove the daemon state file
    pub fn remove(output_dir: &PathBuf) -> Result<()> {
        let state_file = output_dir.join(".snapper").join("daemon.json");
        if state_file.exists() {
            fs::remove_file(&state_file)
                .with_context(|| format!("Failed to remove daemon state file {:?}", state_file))?;
        }
        Ok(())
    }
}

/// Initialize daemon mode
pub fn daemonize(config: &DaemonConfig) -> Result<()> {
    config.ensure_directories()?;

    // Check if a daemon is already running
    if let Some(existing_pid) = read_pid_file(&config.pid_file)? {
        if is_process_running(existing_pid) {
            return Err(eyre::anyhow!(
                "Download already in progress (PID: {}). Use 'snapper status' to check progress.",
                existing_pid
            ));
        } else {
            // Stale PID file, clean it up
            cleanup_pid_file(&config.pid_file)?;
        }
    }

    // Create the daemonize instance
    let daemon = Daemonize::new()
        .pid_file(&config.pid_file)
        .working_directory(&config.working_dir)
        .stdout(
            fs::File::create(&config.log_file)
                .with_context(|| format!("Failed to create log file: {:?}", config.log_file))?,
        )
        .stderr(
            fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&config.log_file)
                .with_context(|| format!("Failed to open log file: {:?}", config.log_file))?,
        );

    // Fork the process
    daemon
        .start()
        .with_context(|| "Failed to daemonize process")?;

    Ok(())
}

/// Read the PID from the PID file
pub fn read_pid_file(pid_file: &PathBuf) -> Result<Option<u32>> {
    if !pid_file.exists() {
        return Ok(None);
    }

    let pid_str = fs::read_to_string(pid_file)
        .with_context(|| format!("Failed to read PID file: {:?}", pid_file))?;

    let pid: u32 = pid_str
        .trim()
        .parse()
        .with_context(|| format!("Invalid PID in file: {}", pid_str))?;

    Ok(Some(pid))
}

/// Check if a process with the given PID is running
pub fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        use std::process::Command;
        // Use kill -0 to check if process exists without actually sending a signal
        Command::new("kill")
            .args(["-0", &pid.to_string()])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[cfg(not(unix))]
    {
        // On non-Unix systems, we can't easily check if a process is running
        // So we'll assume it's not running if we can't check
        false
    }
}

/// Clean up the PID file
pub fn cleanup_pid_file(pid_file: &PathBuf) -> Result<()> {
    if pid_file.exists() {
        fs::remove_file(pid_file)
            .with_context(|| format!("Failed to remove PID file: {:?}", pid_file))?;
    }
    Ok(())
}

/// Set up signal handlers for graceful shutdown
pub fn setup_signal_handlers(pid_file: PathBuf, output_dir: PathBuf) {
    tokio::spawn(async move {
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to set up SIGTERM handler");
        let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
            .expect("Failed to set up SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => {
                eprintln!("Received SIGTERM, cleaning up...");
            }
            _ = sigint.recv() => {
                eprintln!("Received SIGINT, cleaning up...");
            }
        }

        // Clean up PID file and daemon state
        let _ = cleanup_pid_file(&pid_file);
        let _ = DaemonState::remove(&output_dir);
    });
}
