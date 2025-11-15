use clap::{Parser, Subcommand};
use eyre::Result;
use std::path::PathBuf;

mod snapshot;

use snapshot::{snapshot_client::SnapshotClient, types::*};

#[derive(Parser)]
#[command(name = "snapper")]
#[command(about = "BlockVisor Snapshot Management Tool")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate with BlockVisor API
    Auth {
        #[command(subcommand)]
        auth_command: AuthCommands,
    },
    /// List available snapshots
    List {
        /// Filter by protocol (supports multi-word: "arbitrum one")
        #[arg(short, long)]
        protocol: Option<String>,
        /// Show detailed information including versions and sizes
        #[arg(short, long)]
        detailed: bool,
        /// Show only specific client
        #[arg(short, long)]
        client: Option<String>,
        /// Show only specific network
        #[arg(short, long)]
        network: Option<String>,
    },
    /// Download a snapshot
    Download {
        /// Protocol name (e.g., "ethereum", "arbitrum one")
        protocol: String,
        /// Client implementation (e.g., "geth", "reth", "nitro")
        client: String,
        /// Network (e.g., "mainnet", "sepolia", "holesky")
        #[arg(default_value = "mainnet")]
        network: String,
        /// Output directory for download
        #[arg(short, long)]
        output: PathBuf,
        /// Node type (e.g., "archive", "full") - auto-detected if not specified
        #[arg(long)]
        node_type: Option<String>,
        /// Number of parallel workers
        #[arg(short, long, default_value = "4")]
        workers: usize,
        /// Maximum parallel connections
        #[arg(long, default_value = "4")]
        max_connections: usize,
        /// Show what would be downloaded without actually downloading
        #[arg(long)]
        dry_run: bool,
        /// Run download in background (daemon mode)
        #[arg(short = 'd', long)]
        daemon: bool,
    },
    /// Resume an interrupted download
    Resume {
        /// Output directory where download was interrupted
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Show download progress and status
    Status {
        /// Output directory to check status for
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Create a new account
    Register {
        /// API endpoint URL
        #[arg(long, default_value = "https://api.nodexeus.io")]
        api_url: String,
    },
    /// Set up authentication
    Login {
        /// API endpoint URL
        #[arg(long, default_value = "https://api.nodexeus.io")]
        api_url: String,
    },
    /// Check authentication status
    Status,
    /// Clear stored credentials
    Logout,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth { auth_command } => handle_auth_command(auth_command).await,
        Commands::List {
            protocol,
            detailed,
            client,
            network,
        } => handle_list_command(protocol, detailed, client, network).await,
        Commands::Download {
            protocol,
            client,
            network,
            output,
            node_type,
            workers,
            max_connections,
            dry_run,
            daemon,
        } => {
            handle_download_command(
                protocol,
                client,
                network,
                output,
                node_type,
                workers,
                max_connections,
                dry_run,
                daemon,
            )
            .await
        }
        Commands::Resume { output } => handle_resume_command(output).await,
        Commands::Status { output } => handle_status_command(output).await,
    }
}

async fn handle_auth_command(auth_command: AuthCommands) -> Result<()> {
    let mut client = SnapshotClient::new().await?;

    match auth_command {
        AuthCommands::Register { api_url } => {
            client.register(api_url).await?;
        }
        AuthCommands::Login { api_url } => {
            client.login(api_url).await?;
            println!("✓ Authentication successful");
        }
        AuthCommands::Status => {
            let status = client.auth_status().await?;
            println!("Authentication status: {}", status);
        }
        AuthCommands::Logout => {
            client.logout().await?;
            println!("✓ Logged out successfully");
        }
    }

    Ok(())
}

async fn handle_list_command(
    protocol: Option<String>,
    detailed: bool,
    client: Option<String>,
    network: Option<String>,
) -> Result<()> {
    let snapshot_client = SnapshotClient::new().await?;

    let filter = ListFilter {
        protocol,
        client,
        network,
    };

    let groups = snapshot_client.list_snapshots(filter).await?;

    if detailed {
        print_detailed_snapshots(&groups)?;
    } else {
        print_simple_snapshots(&groups)?;
    }

    Ok(())
}

async fn handle_download_command(
    protocol: String,
    client: String,
    network: String,
    output: PathBuf,
    node_type: Option<String>,
    workers: usize,
    max_connections: usize,
    dry_run: bool,
    daemon: bool,
) -> Result<()> {
    let snapshot_client = SnapshotClient::new().await?;

    // Determine the node type - auto-detect if not specified
    let actual_node_type = match node_type {
        Some(nt) => nt,
        None => {
            // List available snapshots to find what node types are available
            let filter = ListFilter {
                protocol: Some(protocol.clone()),
                client: Some(client.clone()),
                network: Some(network.clone()),
            };
            let groups = snapshot_client.list_snapshots(filter).await?;

            // Find the available node types
            let mut available_types = std::collections::HashSet::new();
            for group in &groups {
                for (_, client_group) in &group.clients {
                    for (_, snapshots) in &client_group.networks {
                        for snapshot in snapshots {
                            available_types.insert(snapshot.node_type.clone());
                        }
                    }
                }
            }

            // Decide which node type to use
            match available_types.len() {
                0 => {
                    return Err(eyre::anyhow!(
                        "No snapshots found for {}/{}/{}",
                        protocol,
                        client,
                        network
                    ));
                }
                1 => {
                    // Only one type available, use it
                    let node_type = available_types.into_iter().next().unwrap();
                    println!("Auto-detected node type: {}", node_type);
                    node_type
                }
                _ => {
                    // Multiple types available
                    if available_types.contains("archive") {
                        println!("Multiple node types available. Using 'archive' by default.");
                        println!(
                            "Available types: {}",
                            available_types
                                .iter()
                                .cloned()
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                        "archive".to_string()
                    } else {
                        // No archive type, pick the first one
                        let available_list: Vec<String> = available_types.iter().cloned().collect();
                        let node_type = available_list[0].clone();
                        println!(
                            "Using node type: {} (available: {})",
                            node_type,
                            available_list.join(", ")
                        );
                        node_type
                    }
                }
            }
        }
    };

    let download_config = DownloadConfig {
        workers,
        max_connections,
        output_dir: output,
    };

    if dry_run {
        let info = snapshot_client
            .get_download_info(&protocol, &client, &network, &actual_node_type, None)
            .await?;
        print_download_info(&info)?;
        return Ok(());
    }

    // Handle daemon mode
    if daemon {
        use snapshot::daemon::{daemonize, DaemonConfig, DaemonState, setup_signal_handlers};

        let daemon_config = DaemonConfig::new(download_config.output_dir.clone());
        
        // Daemonize the process
        match daemonize(&daemon_config) {
            Ok(_) => {
                // We're now in the child process
                // Set up signal handlers for graceful shutdown
                setup_signal_handlers(
                    daemon_config.pid_file.clone(),
                    daemon_config.working_dir.clone(),
                );

                // Save daemon state
                let daemon_state = DaemonState::new(
                    std::process::id(),
                    protocol.clone(),
                    client.clone(),
                    network.clone(),
                    download_config.output_dir.clone(),
                );
                daemon_state.save(&download_config.output_dir)?;

                // Log that we're starting
                println!("Starting download in daemon mode (PID: {})", std::process::id());
                println!("Log file: {:?}", daemon_config.log_file);
            }
            Err(e) => {
                eprintln!("Failed to daemonize: {}", e);
                eprintln!("Falling back to foreground execution");
            }
        }
    }

    // Execute download and ensure cleanup happens on completion or error
    let download_result = snapshot_client
        .download_snapshot(
            &protocol,
            &client,
            &network,
            &actual_node_type,
            None,
            download_config.clone(),
            daemon,
        )
        .await;

    // Clean up daemon state on completion (success or failure)
    if daemon {
        use snapshot::daemon::{cleanup_pid_file, DaemonConfig, DaemonState};
        let daemon_config = DaemonConfig::new(download_config.output_dir.clone());
        let _ = cleanup_pid_file(&daemon_config.pid_file);
        let _ = DaemonState::remove(&download_config.output_dir);
    }

    // Return the download result
    download_result
}

async fn handle_resume_command(output: PathBuf) -> Result<()> {
    let snapshot_client = SnapshotClient::new().await?;
    snapshot_client.resume_download(output).await?;
    Ok(())
}

async fn handle_status_command(output: Option<PathBuf>) -> Result<()> {
    use snapshot::daemon::{DaemonState, is_process_running, read_pid_file, DaemonConfig};
    
    let snapshot_client = SnapshotClient::new().await?;
    
    // Check for daemon state if output directory is provided
    if let Some(ref output_dir) = output {
        // Check if there's a daemon running
        let daemon_config = DaemonConfig::new(output_dir.clone());
        
        if let Some(pid) = read_pid_file(&daemon_config.pid_file)? {
            if is_process_running(pid) {
                println!("Daemon status: Running (PID: {})", pid);
                
                // Load daemon state if available
                if let Some(daemon_state) = DaemonState::load(output_dir)? {
                    println!("Protocol: {}/{}/{}", 
                        daemon_state.protocol, 
                        daemon_state.client, 
                        daemon_state.network
                    );
                    
                    let elapsed = daemon_state.start_time.elapsed()
                        .unwrap_or(std::time::Duration::from_secs(0));
                    println!("Running for: {} seconds", elapsed.as_secs());
                }
                
                println!("Log file: {:?}", daemon_config.log_file);
                println!();
            } else {
                println!("Warning: Stale PID file found (process {} not running)", pid);
                println!();
            }
        }
    }
    
    let status = snapshot_client.get_status(output).await?;
    print_status(&status)?;
    Ok(())
}

fn print_simple_snapshots(groups: &[ProtocolGroup]) -> Result<()> {
    println!("Available Snapshots:\n");

    for group in groups {
        println!("Protocol: {}", group.protocol);
        for (client_name, client_group) in &group.clients {
            println!("  Client: {}", client_name);
            for (network_name, snapshots) in &client_group.networks {
                // Group snapshots by node type
                let mut by_node_type: std::collections::HashMap<String, Vec<&SnapshotMetadata>> =
                    std::collections::HashMap::new();
                for snapshot in snapshots {
                    by_node_type
                        .entry(snapshot.node_type.clone())
                        .or_default()
                        .push(snapshot);
                }

                for (node_type, type_snapshots) in by_node_type {
                    let latest_version =
                        type_snapshots.iter().map(|s| s.version).max().unwrap_or(1);
                    println!(
                        "    - {}/{}: v{} (latest)",
                        network_name,
                        node_type,
                        latest_version
                    );
                }
            }
        }
        println!();
    }

    Ok(())
}

fn print_detailed_snapshots(groups: &[ProtocolGroup]) -> Result<()> {
    for group in groups {
        for (client_name, client_group) in &group.clients {
            for (network_name, snapshots) in &client_group.networks {
                // Group snapshots by node type
                let mut by_node_type: std::collections::HashMap<String, Vec<&SnapshotMetadata>> =
                    std::collections::HashMap::new();
                for snapshot in snapshots {
                    by_node_type
                        .entry(snapshot.node_type.clone())
                        .or_default()
                        .push(snapshot);
                }

                for (node_type, mut type_snapshots) in by_node_type {
                    println!(
                        "{}/{}/{}/{}:",
                        group.protocol, client_name, network_name, node_type
                    );

                    // Sort by version
                    type_snapshots.sort_by_key(|s| s.version);

                    for snapshot in type_snapshots {
                        let size_gb = snapshot.total_size as f64 / (1024.0 * 1024.0 * 1024.0);
                        println!("  - Version {} (latest) - {:.1}GB", snapshot.version, size_gb);
                    }
                    println!();
                }
            }
        }
    }

    Ok(())
}

fn print_download_info(info: &DownloadInfo) -> Result<()> {
    println!(
        "Would download: {}/{}/{} (latest version {})",
        info.protocol, info.client, info.network, info.version
    );
    println!("Archive ID: {}", info.archive_id);
    println!("Full path: {}", info.full_path);

    let size_gb = info.total_size as f64 / (1024.0 * 1024.0 * 1024.0);
    println!("Total size: {:.1}GB", size_gb);

    if let Some(compression) = &info.compression {
        println!("Compression: {:?}", compression);
    }

    println!("Chunks: {}", info.chunks);

    // Rough time estimate based on 100MB/s average
    let estimated_minutes = (info.total_size as f64) / (100.0 * 1024.0 * 1024.0 * 60.0);
    println!(
        "Estimated time: {:.0}-{:.0} minutes",
        estimated_minutes * 0.5,
        estimated_minutes * 2.0
    );

    Ok(())
}

fn print_status(status: &DownloadStatus) -> Result<()> {
    match status {
        DownloadStatus::NotStarted => {
            println!("No download in progress");
        }
        DownloadStatus::InProgress {
            progress_percent,
            downloaded_bytes,
            total_bytes,
            speed_mbps,
            eta_minutes,
            chunks_complete,
            total_chunks,
        } => {
            println!("Download status: In progress");
            println!(
                "Progress: {}% complete ({:.1}GB/{:.1}GB downloaded)",
                progress_percent,
                *downloaded_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
                *total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
            );
            println!("Speed: {:.1}MB/s", speed_mbps);
            println!("ETA: {} minutes", eta_minutes);
            println!("Chunks: {}/{} complete", chunks_complete, total_chunks);
        }
        DownloadStatus::Completed => {
            println!("Download status: Completed");
        }
        DownloadStatus::Failed { error } => {
            println!("Download status: Failed");
            println!("Error: {}", error);
        }
    }

    Ok(())
}
