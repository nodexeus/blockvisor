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
        /// Specific version to download (latest if not specified)
        #[arg(short, long)]
        version: Option<u64>,
        /// Number of parallel workers
        #[arg(short, long, default_value = "4")]
        workers: usize,
        /// Maximum parallel connections
        #[arg(long, default_value = "4")]
        max_connections: usize,
        /// Show what would be downloaded without actually downloading
        #[arg(long)]
        dry_run: bool,
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
            version,
            workers,
            max_connections,
            dry_run,
        } => {
            handle_download_command(
                protocol,
                client,
                network,
                output,
                node_type,
                version,
                workers,
                max_connections,
                dry_run,
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
    version: Option<u64>,
    workers: usize,
    max_connections: usize,
    dry_run: bool,
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
            .get_download_info(&protocol, &client, &network, &actual_node_type, version)
            .await?;
        print_download_info(&info)?;
        return Ok(());
    }

    snapshot_client
        .download_snapshot(
            &protocol,
            &client,
            &network,
            &actual_node_type,
            version,
            download_config,
        )
        .await?;

    Ok(())
}

async fn handle_resume_command(output: PathBuf) -> Result<()> {
    let snapshot_client = SnapshotClient::new().await?;
    snapshot_client.resume_download(output).await?;
    Ok(())
}

async fn handle_status_command(output: Option<PathBuf>) -> Result<()> {
    let snapshot_client = SnapshotClient::new().await?;
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
                    let versions_count = type_snapshots.len();
                    println!(
                        "    - {}/{}: v{} ({} version{})",
                        network_name,
                        node_type,
                        latest_version,
                        versions_count,
                        if versions_count == 1 { "" } else { "s" }
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
                        println!("  - Version {} - {:.1}GB", snapshot.version, size_gb);
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
        "Would download: {}/{}/{} version {}",
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
