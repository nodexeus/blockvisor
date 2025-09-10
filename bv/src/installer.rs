use crate::{
    bv_config::{Config, CONFIG_PATH},
    internal_server, ServiceStatus,
};
use async_trait::async_trait;
use bv_utils::{timer::Timer, with_retry};
use eyre::ContextCompat;
use eyre::{anyhow, bail, ensure, Context, Error, Result};
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
    {env, fs},
};
use tonic::{codegen::InterceptedService, transport::Channel};
use tracing::{debug, info, warn};

const SYSTEM_SERVICES: &str = "etc/systemd/system";
const COMPLETION_DIR_BASH: &str = "usr/local/share/bash-completion/completions";
const BV_COMPLETION_FILE_BASH: &str = "bv.bash";
const NIB_COMPLETION_FILE_BASH: &str = "nib.bash";
const COMPLETION_DIR_ZSH: &str = "usr/local/share/zsh/site-functions";
const BV_COMPLETION_FILE_ZSH: &str = "_bv";
const NIB_COMPLETION_FILE_ZSH: &str = "_nib";
const SYSTEM_BIN: &str = "usr/bin";
pub const INSTALL_PATH: &str = "opt/blockvisor";
pub const BLACKLIST: &str = "blacklist";
const CURRENT_LINK: &str = "current";
const BACKUP_LINK: &str = "backup";
pub const INSTALLER_BIN: &str = "installer";
const BLOCKVISOR_BIN: &str = "blockvisor/bin";
const BLOCKVISOR_SERVICES: &str = "blockvisor/services";
const SH_COMPLETE: &str = "sh_complete";
const BLOCKVISOR_CONFIG: &str = "blockvisor.json";
const THIS_VERSION: &str = env!("CARGO_PKG_VERSION");
const BV_CONNECT_TIMEOUT: Duration = Duration::from_secs(1);
const BV_REQ_TIMEOUT: Duration = Duration::from_secs(1);
const BV_CHECK_INTERVAL: Duration = Duration::from_millis(100);
const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(180);
const PREPARE_FOR_UPDATE_TIMEOUT: Duration = Duration::from_secs(180);

struct InstallerPaths {
    bv_root: PathBuf,
    system_services: PathBuf,
    system_bin: PathBuf,
    install_path: PathBuf,
    current: PathBuf,
    this_version: PathBuf,
    backup: PathBuf,
    blacklist: PathBuf,
}

/// SystemCtl abstraction for better testing.
#[async_trait]
pub trait BvService {
    async fn reload(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    async fn start(&self) -> Result<()>;
    async fn enable(&self) -> Result<()>;
    async fn ensure_active(&self) -> Result<()>;
}

#[derive(Debug, PartialEq)]
enum BackupStatus {
    Done(String),
    NothingToBackup,
    ThisIsRollback,
}

pub struct Installer<T, S> {
    paths: InstallerPaths,
    config: Config,
    bv_client: internal_server::service_client::ServiceClient<
        InterceptedService<Channel, bv_utils::rpc::DefaultTimeout>,
    >,
    backup_status: BackupStatus,
    timer: T,
    bv_service: S,
}

impl<T: Timer, S: BvService> Installer<T, S> {
    pub async fn new(timer: T, bv_service: S, bv_root: &Path) -> Result<Self> {
        let config = Config::load(bv_root).await?;
        let channel = Channel::from_shared(format!("http://127.0.0.1:{}", config.blockvisor_port))?
            .connect_timeout(BV_CONNECT_TIMEOUT)
            .connect_lazy();
        Ok(Self::internal_new(
            timer, bv_service, bv_root, config, channel,
        ))
    }

    pub async fn run(mut self) -> Result<()> {
        if self.is_blacklisted(THIS_VERSION)? {
            bail!("BV {THIS_VERSION} is on a blacklist - can't install")
        }
        self.check_requirements().await.with_context(|| {
            format!("Host doesn't meet the requirements, see [Host Setup Guide]('https://github.com/nodexeus/blockvisor/releases/tag/{THIS_VERSION}') for more details.")
        })?;
        info!("installing BV {THIS_VERSION}...");

        self.preinstall()?;
        if let Err(err) = self.install().await {
            self.handle_broken_installation(err).await
        } else {
            // try cleanup after install, but cleanup result should not affect exit code
            self.cleanup() // do not interrupt cleanup on errors
                .unwrap_or_else(|err| warn!("failed to cleanup after install with: {err:#}"));
            Ok(())
        }
    }

    fn internal_new(
        timer: T,
        bv_service: S,
        bv_root: &Path,
        config: Config,
        bv_channel: Channel,
    ) -> Self {
        let install_path = bv_root.join(INSTALL_PATH);
        let current = install_path.join(CURRENT_LINK);
        let this_version = install_path.join(THIS_VERSION);
        let backup = install_path.join(BACKUP_LINK);
        let blacklist = install_path.join(BLACKLIST);

        Self {
            paths: InstallerPaths {
                bv_root: bv_root.to_path_buf(),
                system_services: bv_root.join(SYSTEM_SERVICES),
                system_bin: bv_root.join(SYSTEM_BIN),
                install_path,
                current,
                this_version,
                backup,
                blacklist,
            },
            config,
            bv_client: internal_server::service_client::ServiceClient::with_interceptor(
                bv_channel,
                bv_utils::rpc::DefaultTimeout(BV_REQ_TIMEOUT),
            ),
            backup_status: BackupStatus::NothingToBackup,
            timer,
            bv_service,
        }
    }

    fn move_bundle_to_install_path(&self, current_exe_path: PathBuf) -> Result<()> {
        let bin_path = fs::canonicalize(&current_exe_path).with_context(|| {
            format!(
                "cannot normalise current binary path: {}",
                current_exe_path.display()
            )
        })?;
        let bin_dir = bin_path.parent().with_context(|| {
            format!(
                "invalid parent dir for file: {}",
                current_exe_path.display()
            )
        })?;
        if self.paths.this_version != bin_dir {
            info!(
                "move BV files from {} to install path {}",
                bin_dir.to_string_lossy(),
                self.paths.this_version.to_string_lossy()
            );
            fs::create_dir_all(&self.paths.install_path)
                .with_context(|| "failed to create install path")?;
            fs_extra::dir::move_dir(
                bin_dir,
                &self.paths.this_version,
                &fs_extra::dir::CopyOptions::default()
                    .copy_inside(true)
                    .content_only(true)
                    .overwrite(true),
            )
            .with_context(|| "failed to move files to install path")?;
        }
        Ok(())
    }

    async fn handle_broken_installation(&self, err: Error) -> Result<()> {
        warn!("installation failed with: {err:#}");
        self.blacklist_this_version()?;

        match self.backup_status {
            BackupStatus::Done(_) => {
                self.rollback().await?;
                bail!("installation failed with: {err:#}, but rolled back to previous version")
            }
            BackupStatus::ThisIsRollback => {
                bail!("rollback failed - host needs manual fix: {err:#}")
            }
            BackupStatus::NothingToBackup => {
                bail!("installation failed: {err:#}");
            }
        }
    }

    fn is_blacklisted(&self, version: &str) -> Result<bool> {
        let path = &self.paths.blacklist;
        Ok(path.exists()
            && fs::read_to_string(path)
                .with_context(|| format!("failed to read blacklist from: {}", path.display()))?
                .contains(version))
    }

    fn backup_running_version(&mut self) -> Result<()> {
        if let Some(running_version) = self.get_running_version()? {
            if self.is_blacklisted(&running_version)? {
                self.backup_status = BackupStatus::ThisIsRollback;
            } else {
                info!("backup previously installed BV {running_version}");
                let _ = fs::remove_file(&self.paths.backup);
                std::os::unix::fs::symlink(
                    fs::read_link(&self.paths.current)
                        .with_context(|| "invalid current version link")?,
                    &self.paths.backup,
                )
                .with_context(|| "failed to backup running version for rollback")?;
                self.backup_status = BackupStatus::Done(running_version);
            }
        } else {
            self.backup_status = BackupStatus::NothingToBackup;
        }
        Ok(())
    }

    fn get_running_version(&self) -> Result<Option<String>> {
        if self.paths.current.exists() {
            // get running version if any
            let current = &self.paths.current;
            let current_path_unlinked = current
                .read_link()
                .with_context(|| format!("invalid current version link: {}", current.display()))?;
            Ok(current_path_unlinked
                .file_name()
                .and_then(|v| v.to_str().map(|v| v.to_owned())))
        } else {
            Ok(None)
        }
    }

    fn preinstall(&mut self) -> Result<()> {
        self.move_bundle_to_install_path(
            env::current_exe().with_context(|| "failed to get current binary path")?,
        )?;
        self.backup_running_version()
    }

    async fn install(&mut self) -> Result<()> {
        self.prepare_running().await?;
        self.install_this_version()?;
        self.restart_and_reenable_blockvisor().await?;
        self.health_check().await
    }

    async fn prepare_running(&mut self) -> Result<()> {
        if self.bv_service.ensure_active().await.is_err() {
            return Ok(());
        }

        if let BackupStatus::Done(_) = self.backup_status {
            info!("prepare running BV for update");
            let timestamp = self.timer.now();
            let expired = || {
                let now = self.timer.now();
                let duration = now.duration_since(timestamp);
                duration > PREPARE_FOR_UPDATE_TIMEOUT
            };
            loop {
                match with_retry!(self.bv_client.start_update(())) {
                    Ok(resp) => {
                        let status = resp.into_inner();
                        if status == ServiceStatus::Updating {
                            break;
                        } else if expired() {
                            bail!("prepare running BV for update failed, BV start_update respond with {status:?}");
                        }
                    }
                    Err(err) => {
                        if expired() {
                            bail!("prepare running BV for update failed, BV start_update respond with {err:#}");
                        }
                    }
                }
                self.timer.sleep(BV_CHECK_INTERVAL);
            }
        }
        Ok(())
    }

    fn install_this_version(&self) -> Result<()> {
        info!("update system symlinks");
        // switch current to this version
        let _ = fs::remove_file(&self.paths.current);
        std::os::unix::fs::symlink(&self.paths.this_version, &self.paths.current)
            .with_context(|| "failed to switch current version")?;

        let symlink_all = |src, dst: &PathBuf| -> Result<()> {
            for entry in self
                .paths
                .current
                .join(src)
                .read_dir()
                .with_context(|| format!("failed to get list of installed files in {src}"))?
            {
                let entry = entry.with_context(|| format!("failed to get file entry in {src}"))?;
                let link_path = dst.join(entry.file_name());
                let _ = fs::remove_file(&link_path);
                std::os::unix::fs::symlink(entry.path(), &link_path).with_context(|| {
                    format!(
                        "failed to link {} to {}",
                        entry.path().to_string_lossy(),
                        link_path.to_string_lossy()
                    )
                })?;
            }
            Ok(())
        };
        symlink_all(BLOCKVISOR_BIN, &self.paths.system_bin)?;
        symlink_all(BLOCKVISOR_SERVICES, &self.paths.system_services)?;

        let symlink_sh_complete = |dir, file| {
            let from = self.paths.current.join(SH_COMPLETE).join(file);
            if from.exists() {
                let completion_dir = self.paths.bv_root.join(dir);
                let _ = fs::create_dir_all(&completion_dir);
                let to = completion_dir.join(file);
                let _ = fs::remove_file(&to);
                let _ = std::os::unix::fs::symlink(from, to);
            }
        };
        symlink_sh_complete(COMPLETION_DIR_BASH, BV_COMPLETION_FILE_BASH);
        symlink_sh_complete(COMPLETION_DIR_ZSH, BV_COMPLETION_FILE_ZSH);
        symlink_sh_complete(COMPLETION_DIR_BASH, NIB_COMPLETION_FILE_BASH);
        symlink_sh_complete(COMPLETION_DIR_ZSH, NIB_COMPLETION_FILE_ZSH);
        Ok(())
    }

    async fn restart_and_reenable_blockvisor(&self) -> Result<()> {
        info!("Reloading blockvisor service configuration...");
        match self.bv_service.reload().await {
            Ok(_) => info!("Successfully reloaded service configuration"),
            Err(e) => warn!("Service reload had an issue: {:#}", e),
        }
    
        info!("Stopping blockvisor service...");
        self.bv_service.stop().await
            .with_context(|| "Failed to stop blockvisor service")?;
        info!("Blockvisor service stopped successfully");
    
        // Add a delay to ensure the service has fully stopped
        info!("Waiting for service to fully terminate...");
        self.timer.sleep(Duration::from_secs(2));
    
        info!("Backing up and migrating blockvisor data...");
        self.backup_and_migrate_bv_data().await
            .with_context(|| "Failed to backup and migrate blockvisor data")?;
        info!("Data backup and migration completed successfully");
    
    // Add another delay before starting the service
        self.timer.sleep(Duration::from_secs(1));
    
        info!("Starting blockvisor service with new version...");
        self.bv_service.start().await
            .with_context(|| "Failed to start blockvisor service with new version")?;
        info!("Blockvisor service started successfully");
    
        info!("Enabling blockvisor service...");
        self.bv_service.enable().await
            .with_context(|| "Failed to enable blockvisor service")?;
        info!("Blockvisor service enabled successfully");
    
        Ok(())
    }

    async fn backup_and_migrate_bv_data(&self) -> Result<()> {
        if let BackupStatus::Done(_) = self.backup_status {
            // backup blockvisor config since new version may modify/break it
            let config_path = self.paths.bv_root.join(CONFIG_PATH);
            let backup_config_path = self.paths.backup.join(BLOCKVISOR_CONFIG);
            fs::copy(&config_path, &backup_config_path).with_context(|| {
                format!(
                    "failed to copy bv config `{}` to backup location `{}`",
                    config_path.display(),
                    backup_config_path.display()
                )
            })?;

            // migrate config to new format if needed
        };

        Ok(())
    }

    async fn health_check(&mut self) -> Result<()> {
        info!("verify newly installed BV");
        let timestamp = self.timer.now();
        let expired = || {
            let now = self.timer.now();
            let duration = now.duration_since(timestamp);
            duration > HEALTH_CHECK_TIMEOUT
        };
        loop {
            match with_retry!(self.bv_client.health(())) {
                Ok(resp) => {
                    let status = resp.into_inner();
                    if status == ServiceStatus::Ok {
                        break;
                    } else if expired() {
                        bail!(
                            "installed BV health check failed, BV health respond with {status:?}"
                        );
                    }
                }
                Err(err) => {
                    if expired() {
                        bail!("installed BV health check failed, BV health respond with {err:#}");
                    }
                }
            }
            self.timer.sleep(BV_CHECK_INTERVAL);
        }
        Ok(())
    }

    async fn rollback(&self) -> Result<()> {
        info!("Starting rollback process due to installation failure");
        // stop broken version first
        info!("Stopping broken installation...");
        self.bv_service
            .stop()
            .await
            .with_context(|| "failed to stop broken installation - can't continue rollback")?;
        info!("Successfully stopped broken installation");

        // Add a delay to ensure the service has fully stopped
        info!("Waiting for service to fully terminate...");
        self.timer.sleep(Duration::from_secs(2));
    
        info!("Rolling back blockvisor data...");
        self.rollback_bv_data()
            .with_context(|| "failed to rollback BV data")?;
        info!("Successfully rolled back blockvisor data");

        // Release the lock file if it exists to prevent "already locked" errors
        let lock_path = self.paths.bv_root.join(".lock");
        if lock_path.exists() {
            info!("Found existing lock file, removing it before running backup installer");
            match fs::remove_file(&lock_path) {
                Ok(_) => info!("Successfully removed lock file"),
                Err(e) => warn!("Failed to remove lock file: {:#}, continuing anyway", e),
            }
        }

        let backup_installer = self.paths.backup.join(INSTALLER_BIN);
        ensure!(backup_installer.exists(), "no backup found");

        info!("Running backup installer: {}", backup_installer.display());
        let output = Command::new(&backup_installer)
            .output()
            .with_context(|| "failed to launch backup installer")?;

        let status_code = output.status.code()
            .ok_or_else(|| anyhow!("failed to get backup installer exit status code"))?;

        // Log stdout and stderr for debugging
        if !output.stdout.is_empty() {
            info!("Backup installer stdout: {}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            warn!("Backup installer stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
    
        ensure!(
            status_code == 0,
            "backup installer failed with exit code {status_code}"
        );
    
        info!("Rollback completed successfully");
        // let status_code = Command::new(backup_installer)
        //     .status()
        //     .with_context(|| "failed to launch backup installer")?
        //     .code()
        //     .ok_or_else(|| anyhow!("failed to get backup installer exit status code"))?;
        // ensure!(
        //     status_code == 0,
        //     "backup installer failed with exit code {status_code}"
        // );
        Ok(())
    }

    fn rollback_bv_data(&self) -> Result<()> {
        // rollback state/config

        // rollback blockvisor config - if any
        let backup_config_path = self.paths.backup.join(BLOCKVISOR_CONFIG);
        if backup_config_path.exists() {
            fs::copy(backup_config_path, self.paths.bv_root.join(CONFIG_PATH))?;
        }
        Ok(())
    }

    fn blacklist_this_version(&self) -> Result<()> {
        let path = &self.paths.blacklist;
        let mut file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)?;
        writeln!(file, "{THIS_VERSION}").with_context(|| {
            format!(
                "install failed, but can't write version to blacklist: {}",
                path.display()
            )
        })
    }

    fn cleanup(&self) -> Result<()> {
        info!("cleanup old BV files:");
        self.cleanup_bv_data_backup()?;
        let persistent = [
            &self.paths.blacklist,
            &self.paths.current,
            &self.paths.this_version,
        ];
        for entry in self
            .paths
            .install_path
            .read_dir()
            .with_context(|| "failed to get cleanup list")?
        {
            let entry = entry.with_context(|| "failed to get cleanup list entry")?;

            if !persistent.contains(&&entry.path()) {
                debug!("remove {}", entry.path().to_string_lossy());
                let _ = if entry.path().is_dir() {
                    fs::remove_dir_all(entry.path())
                } else {
                    fs::remove_file(entry.path())
                }
                // do not interrupt cleanup on errors
                .map_err(|err| {
                    let path = entry.path();
                    warn!(
                        "failed to cleanup after install, can't remove {} with: {:#}",
                        path.to_string_lossy(),
                        err
                    )
                });
            }
        }
        Ok(())
    }

    fn cleanup_bv_data_backup(&self) -> Result<()> {
        // cleanup state/config conversion remnants

        Ok(())
    }

    async fn check_requirements(&self) -> Result<()> {
        info!("checking BV {THIS_VERSION} requirements ...");
        if semver::Version::parse(
            bv_utils::cmd::run_cmd("uname", ["-r"])
                .await?
                .split("-")
                .next()
                .ok_or(anyhow!("can't check kernel version"))?,
        )? < semver::Version::new(5, 10, 0)
        {
            bail!("Linux kernel 5.10+ is required");
        }
        check_cli_dependencies().await?;
        self.check_network_setup().await?;
        Ok(())
    }

    async fn check_network_setup(&self) -> Result<()> {
        bv_utils::cmd::run_cmd("ip", ["link", "show", &self.config.iface])
            .await
            .with_context(|| format!("bridge interface '{}' not configured", self.config.iface))?;
        Ok(())
    }
}

async fn check_cli_dependencies() -> Result<()> {
    // smoke test for all CLI tools used in BV
    for cmd in ["apptainer", "pigz", "tar", "systemctl"] {
        bv_utils::cmd::run_cmd(cmd, ["--version"]).await?;
    }
    for cmd in ["ip"] {
        bv_utils::cmd::run_cmd(cmd, ["-V"]).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hosts;
    use crate::internal_server;
    use crate::node_metrics;
    use crate::node_state;
    use assert_fs::TempDir;
    use babel_api::engine::JobsInfo;
    use bv_tests_utils::rpc::test_channel;
    use bv_tests_utils::start_test_server;
    use bv_utils::timer::MockTimer;
    use eyre::anyhow;
    use mockall::*;
    use std::ops::Add;
    use std::os::unix::fs::OpenOptionsExt;
    use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
    use std::sync::Arc;
    use std::thread::sleep;
    use std::time::Instant;
    use uuid::Uuid;

    mock! {
        pub TestBV {}

        #[tonic::async_trait]
        impl internal_server::service_server::Service for TestBV {
            async fn info(
                &self,
                request: tonic::Request<()>,
            ) -> Result<tonic::Response<String>, tonic::Status>;
            async fn health(
                &self,
                request: tonic::Request<()>,
            ) -> Result<tonic::Response<ServiceStatus>, tonic::Status>;
            async fn start_update(
                &self,
                request: tonic::Request<()>,
            ) -> Result<tonic::Response<ServiceStatus>, tonic::Status>;
            async fn get_host_metrics(
                &self,
                request: tonic::Request<()>,
            ) -> Result<tonic::Response<hosts::HostMetrics>, tonic::Status>;
            async fn get_node(
                &self,
                _request: tonic::Request<Uuid>,
            ) -> Result<tonic::Response<internal_server::NodeDisplayInfo>, tonic::Status>;
            async fn get_nodes(
                &self,
                _request: tonic::Request<bool>,
            ) -> Result<tonic::Response<Vec<internal_server::NodeDisplayInfo>>, tonic::Status>;
            async fn create_node(
                &self,
                request: tonic::Request<internal_server::CreateNodeRequest>,
            ) -> Result<tonic::Response<internal_server::NodeDisplayInfo>, tonic::Status>;
            async fn create_dev_node(
                &self,
                request: tonic::Request<node_state::NodeState>,
            ) -> Result<tonic::Response<internal_server::NodeDisplayInfo>, tonic::Status>;
            async fn delete_node(&self, request: tonic::Request<Uuid>) -> Result<tonic::Response<()>, tonic::Status>;
            async fn start_node(&self, request: tonic::Request<Uuid>) -> Result<tonic::Response<()>, tonic::Status>;
            async fn stop_node(&self, request: tonic::Request<(Uuid, bool)>) -> Result<tonic::Response<()>, tonic::Status>;
            async fn upgrade_node(&self, request: tonic::Request<(Uuid, Option<String>, Option<u64>)>) -> Result<tonic::Response<()>, tonic::Status>;
            async fn upgrade_dev_node(
                &self,
                request: tonic::Request<node_state::NodeState>,
            ) -> Result<tonic::Response<internal_server::NodeDisplayInfo>, tonic::Status>;
            async fn get_node_jobs(
                &self,
                request: tonic::Request<Uuid>,
            ) -> Result<tonic::Response<JobsInfo>, tonic::Status>;
            async fn get_node_job_info(
                &self,
                request: tonic::Request<(Uuid, String)>,
            ) -> Result<tonic::Response<babel_api::engine::JobInfo>, tonic::Status>;
            async fn start_node_job(
                &self,
                request: tonic::Request<(Uuid, String)>,
            ) -> Result<tonic::Response<()>, tonic::Status>;
            async fn stop_node_job(
                &self,
                request: tonic::Request<(Uuid, String)>,
            ) -> Result<tonic::Response<()>, tonic::Status>;
            async fn skip_node_job(
                &self,
                request: tonic::Request<(Uuid, String)>,
            ) -> Result<tonic::Response<()>, tonic::Status>;
            async fn cleanup_node_job(
                &self,
                request: tonic::Request<(Uuid, String)>,
            ) -> Result<tonic::Response<()>, tonic::Status>;
            async fn get_node_id_for_name(
                &self,
                request: tonic::Request<String>,
            ) -> Result<tonic::Response<String>, tonic::Status>;
            async fn list_capabilities(
                &self,
                request: tonic::Request<Uuid>,
            ) -> Result<tonic::Response<Vec<String>>, tonic::Status>;
            async fn run(
                &self,
                request: tonic::Request<(Uuid, String, String)>,
            ) -> Result<tonic::Response<String>, tonic::Status>;
            async fn reload_plugin(
                &self,
                request: tonic::Request<Uuid>
            ) -> Result<tonic::Response<()>, tonic::Status>;
            async fn get_node_metrics(
                &self,
                request: tonic::Request<Uuid>,
            ) -> Result<tonic::Response<node_metrics::Metric>, tonic::Status>;
            async fn get_cluster_status(
                &self,
                request: tonic::Request<()>,
            ) -> Result<tonic::Response<String>, tonic::Status>;
        }
    }

    mock! {
        pub TestBvService {}

        #[async_trait]
        impl BvService for TestBvService {
            async fn reload(&self) -> Result<()>;
            async fn stop(&self) -> Result<()>;
            async fn start(&self) -> Result<()>;
            async fn enable(&self) -> Result<()>;
            async fn ensure_active(&self) -> Result<()>;
        }
    }

    fn touch_file(path: &PathBuf) -> std::io::Result<fs::File> {
        fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
    }

    /// Common staff to setup for all tests like sut (installer in that case),
    /// path to root dir used in test, instance of AsyncPanicChecker to make sure that all panics
    /// from other threads will be propagated.
    struct TestEnv {
        tmp_root: PathBuf,
        _async_panic_checker: bv_tests_utils::AsyncPanicChecker,
    }

    impl TestEnv {
        fn new() -> Result<Self> {
            let tmp_root = TempDir::new()?.to_path_buf();
            let _ = fs::create_dir_all(&tmp_root);

            Ok(Self {
                tmp_root,
                _async_panic_checker: Default::default(),
            })
        }

        fn start_test_server(&self, bv_mock: MockTestBV) -> bv_tests_utils::rpc::TestServer {
            start_test_server!(
                &self.tmp_root,
                internal_server::service_server::ServiceServer::new(bv_mock)
            )
        }

        fn build_installer(
            &self,
            timer: MockTimer,
            bv_service: MockTestBvService,
        ) -> Installer<MockTimer, MockTestBvService> {
            Installer::internal_new(
                timer,
                bv_service,
                &self.tmp_root,
                Config {
                    iface: "br0".to_string(),
                    ..Default::default()
                },
                test_channel(&self.tmp_root),
            )
        }
    }

    #[tokio::test]
    async fn test_prepare_running_none() -> Result<()> {
        let test_env = TestEnv::new()?;

        let mut service_mock = MockTestBvService::new();
        service_mock
            .expect_ensure_active()
            .times(2)
            .returning(|| Ok(()));
        let mut installer = test_env.build_installer(MockTimer::new(), service_mock);
        installer.backup_status = BackupStatus::NothingToBackup;
        installer.prepare_running().await?;
        installer.backup_status = BackupStatus::ThisIsRollback;
        installer.prepare_running().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_prepare_running_ok() -> Result<()> {
        let test_env = TestEnv::new()?;

        let mut bv_mock = MockTestBV::new();
        bv_mock
            .expect_start_update()
            .once()
            .returning(|_| Ok(tonic::Response::new(ServiceStatus::Updating)));
        let mut timer_mock = MockTimer::new();
        timer_mock.expect_sleep().returning(|_| ());
        let now = Instant::now();
        timer_mock.expect_now().returning(move || now);
        let server = test_env.start_test_server(bv_mock);
        let mut service_mock = MockTestBvService::new();
        service_mock.expect_ensure_active().return_once(|| Ok(()));
        let mut installer = test_env.build_installer(timer_mock, service_mock);
        installer.backup_status = BackupStatus::Done(THIS_VERSION.to_owned());
        installer.prepare_running().await?;
        server.assert().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_prepare_running_timeout() -> Result<()> {
        let test_env = TestEnv::new()?;
        let mut bv_mock = MockTestBV::new();
        let update_start_called = Arc::new(AtomicBool::new(false));
        let update_start_called_flag = update_start_called.clone();
        bv_mock.expect_start_update().once().returning(move |_| {
            update_start_called_flag.store(true, Relaxed);
            Ok(tonic::Response::new(ServiceStatus::Ok))
        });
        let mut timer_mock = MockTimer::new();
        let now = Instant::now();
        timer_mock.expect_now().once().returning(move || now);
        timer_mock
            .expect_sleep()
            .with(predicate::eq(BV_CHECK_INTERVAL))
            .returning(|_| ());
        timer_mock.expect_now().returning(move || {
            if update_start_called.load(Relaxed) {
                now.add(PREPARE_FOR_UPDATE_TIMEOUT)
                    .add(Duration::from_secs(1))
            } else {
                now
            }
        });
        let server = test_env.start_test_server(bv_mock);
        let mut service_mock = MockTestBvService::new();
        service_mock.expect_ensure_active().return_once(|| Ok(()));
        let mut installer = test_env.build_installer(timer_mock, service_mock);
        installer.backup_status = BackupStatus::Done(THIS_VERSION.to_owned());
        let _ = installer.prepare_running().await.unwrap_err();
        server.assert().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_health_check_ok() -> Result<()> {
        let test_env = TestEnv::new()?;

        let mut bv_mock = MockTestBV::new();
        bv_mock
            .expect_health()
            .times(2)
            .returning(|_| Ok(tonic::Response::new(ServiceStatus::Ok)));
        let mut timer_mock = MockTimer::new();
        timer_mock.expect_now().returning(Instant::now);
        let server = test_env.start_test_server(bv_mock);
        let mut client =
            internal_server::service_client::ServiceClient::new(test_channel(&test_env.tmp_root));
        while client.health(()).await.is_err() {
            sleep(Duration::from_millis(10));
        }

        test_env
            .build_installer(timer_mock, MockTestBvService::new())
            .health_check()
            .await?;
        server.assert().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_health_check_timeout() -> Result<()> {
        let test_env = TestEnv::new()?;
        let mut bv_mock = MockTestBV::new();
        bv_mock
            .expect_health()
            .returning(|_| Ok(tonic::Response::new(ServiceStatus::Updating)));
        let mut timer_mock = MockTimer::new();
        let now = Instant::now();
        timer_mock.expect_now().once().returning(move || now);
        timer_mock
            .expect_sleep()
            .with(predicate::eq(BV_CHECK_INTERVAL))
            .returning(|_| ());
        timer_mock
            .expect_now()
            .once()
            .returning(move || now.add(HEALTH_CHECK_TIMEOUT).add(Duration::from_secs(1)));
        let server = test_env.start_test_server(bv_mock);
        let mut client =
            internal_server::service_client::ServiceClient::new(test_channel(&test_env.tmp_root));
        while client.health(()).await.is_err() {
            sleep(Duration::from_millis(10));
        }

        let _ = test_env
            .build_installer(timer_mock, MockTestBvService::new())
            .health_check()
            .await
            .unwrap_err();
        server.assert().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_backup_running_version() -> Result<()> {
        let test_env = TestEnv::new()?;
        let mut installer = test_env.build_installer(MockTimer::new(), MockTestBvService::new());

        fs::create_dir_all(&installer.paths.install_path)?;
        installer.backup_running_version()?;
        assert_eq!(BackupStatus::NothingToBackup, installer.backup_status);

        fs::create_dir_all(&installer.paths.this_version)?;
        std::os::unix::fs::symlink(&installer.paths.this_version, &installer.paths.current)?;
        installer.backup_running_version()?;
        assert_eq!(
            BackupStatus::Done(THIS_VERSION.to_owned()),
            installer.backup_status
        );
        assert_eq!(
            &installer.paths.this_version,
            &installer.paths.backup.read_link()?
        );

        installer.blacklist_this_version()?;
        installer.backup_running_version()?;
        assert_eq!(BackupStatus::ThisIsRollback, installer.backup_status);

        Ok(())
    }

    #[tokio::test]
    async fn test_move_bundle_to_install_path() -> Result<()> {
        let test_env = TestEnv::new()?;
        let installer = test_env.build_installer(MockTimer::new(), MockTestBvService::new());
        let bundle_path = test_env.tmp_root.join("bundle");

        let _ = installer
            .move_bundle_to_install_path(bundle_path.join("installer"))
            .unwrap_err();

        fs::create_dir_all(bundle_path.join("some_dir/with_subdir"))?;
        touch_file(&bundle_path.join("installer"))?;
        touch_file(&bundle_path.join("some_file"))?;
        touch_file(&bundle_path.join("some_dir/sub_file"))?;

        installer.move_bundle_to_install_path(bundle_path.join("installer"))?;
        installer.move_bundle_to_install_path(installer.paths.this_version.join("installer"))?;

        assert!(installer.paths.this_version.join("installer").exists());
        assert!(installer.paths.this_version.join("some_file").exists());
        assert!(installer
            .paths
            .this_version
            .join("some_dir/with_subdir")
            .exists());
        assert!(installer
            .paths
            .this_version
            .join("some_dir/sub_file")
            .exists());

        Ok(())
    }

    #[tokio::test]
    async fn test_install_this_version() -> Result<()> {
        let test_env = TestEnv::new()?;
        let installer = test_env.build_installer(MockTimer::new(), MockTestBvService::new());

        let _ = installer.install_this_version().unwrap_err();

        let this_path = &installer.paths.this_version;
        fs::create_dir_all(this_path)?;
        let _ = installer.install_this_version().unwrap_err();

        fs::create_dir_all(this_path.join(BLOCKVISOR_BIN))?;
        fs::create_dir_all(this_path.join(BLOCKVISOR_SERVICES))?;
        installer.install_this_version()?;

        touch_file(&this_path.join(BLOCKVISOR_BIN).join("some_bin"))?;
        touch_file(&this_path.join(BLOCKVISOR_SERVICES).join("some_service"))?;
        let _ = installer.install_this_version().unwrap_err();

        fs::create_dir_all(&installer.paths.system_bin)?;
        fs::create_dir_all(&installer.paths.system_services)?;
        installer.install_this_version()?;

        assert!(installer.paths.system_bin.join("some_bin").exists());
        assert!(installer
            .paths
            .system_services
            .join("some_service")
            .exists());

        Ok(())
    }

    #[tokio::test]
    async fn test_broken_installation() -> Result<()> {
        let test_env = TestEnv::new()?;
        let mut service_mock = MockTestBvService::new();
        service_mock.expect_reload().return_once(|| Ok(()));
        service_mock.expect_stop().return_once(|| Ok(()));
        service_mock.expect_stop().return_once(|| Ok(()));
        service_mock.expect_enable().return_once(|| Ok(()));
        let mut installer = test_env.build_installer(MockTimer::new(), service_mock);

        installer.backup_status = BackupStatus::ThisIsRollback;
        let _ = installer
            .handle_broken_installation(anyhow!("error"))
            .await
            .unwrap_err();
        assert!(!installer.is_blacklisted(THIS_VERSION)?);

        fs::create_dir_all(&installer.paths.install_path)?;
        installer.backup_status = BackupStatus::NothingToBackup;
        let _ = installer
            .handle_broken_installation(anyhow!("error"))
            .await
            .unwrap_err();
        assert!(installer.is_blacklisted(THIS_VERSION)?);

        fs::create_dir_all(&installer.paths.backup)?;
        fs::create_dir_all(installer.paths.bv_root.join("etc"))?;
        fs::File::create(installer.paths.backup.join(BLOCKVISOR_CONFIG))?;
        {
            // create dummy installer that will touch test file as a proof it was called
            let mut backup_installer = fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .mode(0o770)
                .open(installer.paths.backup.join(INSTALLER_BIN))?;
            writeln!(backup_installer, "#!/bin/sh")?;
            writeln!(
                backup_installer,
                "touch {}",
                test_env.tmp_root.join("dummy_installer").to_str().unwrap()
            )?;
            writeln!(backup_installer, "exit 1")?;
        }
        let _ = fs::remove_file(test_env.tmp_root.join("dummy_installer"));
        installer.backup_status = BackupStatus::Done(THIS_VERSION.to_owned());
        let _ = installer
            .handle_broken_installation(anyhow!("error"))
            .await
            .unwrap_err();
        assert!(test_env.tmp_root.join("dummy_installer").exists());

        Ok(())
    }

    #[tokio::test]
    async fn test_cleanup() -> Result<()> {
        let test_env = TestEnv::new()?;
        let installer = test_env.build_installer(MockTimer::new(), MockTestBvService::new());

        // cant cleanup non existing dir nothing
        let _ = installer.cleanup().unwrap_err();

        fs::create_dir_all(&installer.paths.install_path)?;

        // cleanup empty dir
        installer.cleanup()?;

        touch_file(&installer.paths.install_path.join("some_file"))?;
        fs::create_dir_all(installer.paths.install_path.join("some_dir"))?;
        touch_file(
            &installer
                .paths
                .install_path
                .join("some_dir")
                .join("another_file"),
        )?;
        std::os::unix::fs::symlink(
            installer.paths.install_path.join("some_dir"),
            installer.paths.install_path.join("dir_link"),
        )?;
        std::os::unix::fs::symlink(
            installer.paths.install_path.join("some_file"),
            installer.paths.install_path.join("file_link"),
        )?;
        fs::create_dir_all(&installer.paths.this_version)?;
        std::os::unix::fs::symlink(&installer.paths.this_version, &installer.paths.current)?;
        touch_file(&installer.paths.blacklist)?;
        installer.cleanup()?;

        let mut remaining = installer
            .paths
            .install_path
            .read_dir()?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;
        remaining.sort();
        assert_eq!(3, remaining.len());
        assert!(remaining.contains(&installer.paths.blacklist));
        assert!(remaining.contains(&installer.paths.current));
        assert!(remaining.contains(&installer.paths.this_version));

        Ok(())
    }
}
