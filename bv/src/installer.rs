use anyhow::{bail, ensure, Context, Error, Result};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

const SYSTEM_SERVICES: &str = "etc/systemd/system";
const SYSTEM_BIN: &str = "usr/bin";
const INSTALL_PATH: &str = "opt/blockvisor";
const BLACKLIST: &str = "blacklist";
const CURRENT_LINK: &str = "current";
const BACKUP_LINK: &str = "backup";
const INSTALLER_BIN: &str = "installer";
const FC_BIN: &str = "firecracker/bin";
const BLOCKVISOR_BIN: &str = "blockvisor/bin";
const BLOCKVISOR_SERVICES: &str = "blockvisor/services";
const THIS_VERSION: &str = env!("CARGO_PKG_VERSION");

struct InstallerPaths {
    system_services: PathBuf,
    system_bin: PathBuf,
    install_path: PathBuf,
    current: PathBuf,
    this_version: PathBuf,
    backup: PathBuf,
    blacklist: PathBuf,
}

impl Default for Installer {
    fn default() -> Self {
        Self::new(crate::env::ROOT_DIR.clone())
    }
}

#[derive(Debug, PartialEq)]
enum BackupStatus {
    Done,
    NothingToBackup,
    ThisIsRollback,
}

pub struct Installer {
    paths: InstallerPaths,
}

impl Installer {
    pub fn run(self) -> Result<()> {
        if self.is_blacklisted(THIS_VERSION)? {
            bail!("BV {THIS_VERSION} is on a blacklist - can't install")
        }
        println!("installing BV {THIS_VERSION} ...");
        let pre_install_status = || {
            self.move_bundle_to_install_path(
                env::current_exe().with_context(|| "failed to get current binary path")?,
            )?;
            self.backup_running_version()
        };
        match pre_install_status() {
            Ok(backup_status) => {
                let install_status = || {
                    // TODO let running version know about update: ask to stop pending actions
                    self.install_this_version()?;
                    Self::restart_blockvisor()?;
                    self.health_check()
                };
                match install_status() {
                    Ok(_) => {
                        // try cleanup after install, but cleanup result should not affect exit code
                        let _ = self
                            .cleanup() // do not interrupt cleanup on errors
                            .map_err(|err| println!("failed to cleanup after install with: {err}"));
                        Ok(())
                    }
                    Err(err) => self.handle_broken_installation(backup_status, err),
                }
            }
            Err(err) => {
                // TODO: try to send install failed status to API
                Err(err)
            }
        }
    }

    fn new(root: PathBuf) -> Self {
        let install_path = root.join(INSTALL_PATH);
        let current = install_path.join(CURRENT_LINK);
        let this_version = install_path.join(THIS_VERSION);
        let backup = install_path.join(BACKUP_LINK);
        let blacklist = install_path.join(BLACKLIST);

        Self {
            paths: InstallerPaths {
                system_services: root.join(SYSTEM_SERVICES),
                system_bin: root.join(SYSTEM_BIN),
                install_path,
                current,
                this_version,
                backup,
                blacklist,
            },
        }
    }

    fn move_bundle_to_install_path(&self, current_exe_path: PathBuf) -> Result<()> {
        let bin_path = fs::canonicalize(current_exe_path)
            .with_context(|| "non canonical current binary path")?;
        let bin_dir = bin_path.parent().expect("invalid current binary dir");
        if self.paths.this_version != bin_dir {
            println!(
                "move BV files from {} to install path {}",
                bin_dir.to_string_lossy(),
                self.paths.this_version.to_string_lossy()
            );
            fs::create_dir_all(&self.paths.install_path).expect("failed to create install path");
            let move_opt = fs_extra::dir::CopyOptions {
                overwrite: true,
                skip_exist: false,
                buffer_size: 64000,
                copy_inside: true,
                content_only: false,
                depth: 0,
            };
            fs_extra::dir::move_dir(bin_dir, &self.paths.this_version, &move_opt)
                .with_context(|| "failed to move files to install path")?;
        }
        Ok(())
    }

    fn handle_broken_installation(&self, backup_status: BackupStatus, err: Error) -> Result<()> {
        self.blacklist_this_version()?;

        match backup_status {
            BackupStatus::Done => {
                // TODO: try to send install failed status to API
                self.rollback()?;
                bail!("installation failed with: {err}, but rolled back to previous version")
            }
            BackupStatus::ThisIsRollback => {
                // TODO: try to send rollback failed status to API
                bail!("rollback failed - host needs manual fix: {err}")
            }
            BackupStatus::NothingToBackup => {
                // TODO: try to send install failed status to API
                bail!("installation failed: {err}");
            }
        }
    }

    fn is_blacklisted(&self, version: &str) -> Result<bool> {
        Ok(self.paths.blacklist.exists()
            && fs::read_to_string(&self.paths.blacklist)
                .with_context(|| "failed to read blacklist")?
                .contains(version))
    }

    fn backup_running_version(&self) -> Result<BackupStatus> {
        if let Some(running_version) = self.get_running_version()? {
            if self.is_blacklisted(running_version.as_str())? {
                Ok(BackupStatus::ThisIsRollback)
            } else {
                println!("backup previously installed BV {running_version}");
                let _ = fs::remove_file(&self.paths.backup);
                std::os::unix::fs::symlink(
                    fs::read_link(&self.paths.current)
                        .with_context(|| "invalid current version link")?,
                    &self.paths.backup,
                )
                .with_context(|| "failed to backup running version for rollback")?;
                Ok(BackupStatus::Done)
            }
        } else {
            Ok(BackupStatus::NothingToBackup)
        }
    }

    fn get_running_version(&self) -> Result<Option<String>> {
        if self.paths.current.exists() {
            // get running version if any
            let current_path_unlinked = self
                .paths
                .current
                .read_link()
                .with_context(|| "invalid current version link")?;
            Ok(current_path_unlinked
                .file_name()
                .and_then(|v| v.to_str().map(|v| v.to_owned())))
        } else {
            Ok(None)
        }
    }

    fn install_this_version(&self) -> Result<()> {
        println!("update system symlinks");
        // switch current to this version
        let _ = fs::remove_file(&self.paths.current);
        std::os::unix::fs::symlink(&self.paths.this_version, &self.paths.current)
            .with_context(|| "failed to switch current version")?;

        let symlink_all = |src, dst: &PathBuf| {
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
        symlink_all(FC_BIN, &self.paths.system_bin)?;
        symlink_all(BLOCKVISOR_SERVICES, &self.paths.system_services)
    }

    fn restart_blockvisor() -> Result<()> {
        println!("request blockvisor service restart (systemctl restart blockvisor)");
        let status_code = Command::new("systemctl")
            .args(["restart", "blockvisor"])
            .status()
            .with_context(|| "failed to restart blockvisor service")?
            .code()
            .with_context(|| "failed to get systemctl exit status code")?;
        ensure!(
            status_code != 0,
            "blockvisor service restart failed with exit code {status_code}"
        );
        Ok(())
    }

    fn health_check(&self) -> Result<()> {
        println!("verify newly installed BV");
        // TODO: wait for new blockvisor start and respond with health check status
        Ok(())
    }

    fn rollback(&self) -> Result<()> {
        let backup_installer = self.paths.backup.join(INSTALLER_BIN);
        ensure!(backup_installer.exists(), "no backup found");
        let status_code = Command::new(backup_installer)
            .status()
            .with_context(|| "failed to launch backup installer")?
            .code()
            .with_context(|| "failed to get backup installer exit status code")?;
        ensure!(
            status_code != 0,
            "backup installer failed with exit code {status_code}"
        );
        Ok(())
    }

    fn blacklist_this_version(&self) -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&self.paths.blacklist)?;
        writeln!(file, "{THIS_VERSION}")
            .with_context(|| "install failed, but can't blacklist broken version")
    }

    fn cleanup(&self) -> Result<()> {
        println!("vleanup old BV files:");
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
                println!("remove {}", entry.path().to_string_lossy());
                let _ = if entry.path().is_dir() {
                    fs::remove_dir_all(entry.path())
                } else {
                    fs::remove_file(entry.path())
                }
                // do not interrupt cleanup on errors
                .map_err(|err| {
                    let path = entry.path();
                    println!(
                        "failed to cleanup after install, can't remove {} with: {}",
                        path.to_string_lossy(),
                        err
                    )
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use assert_fs::TempDir;
    use std::os::unix::fs::OpenOptionsExt;

    fn touch_file(path: &PathBuf) -> std::io::Result<fs::File> {
        fs::OpenOptions::new().create(true).write(true).open(path)
    }

    #[test]
    fn test_backup_running_version() -> Result<()> {
        let tmp_root = TempDir::new()?.to_path_buf();
        let installer = Installer::new(tmp_root);

        fs::create_dir_all(&installer.paths.install_path)?;
        assert_eq!(
            BackupStatus::NothingToBackup,
            installer.backup_running_version()?
        );

        fs::create_dir_all(&installer.paths.this_version)?;
        std::os::unix::fs::symlink(&installer.paths.this_version, &installer.paths.current)?;
        assert_eq!(BackupStatus::Done, installer.backup_running_version()?);
        assert_eq!(
            &installer.paths.this_version,
            &installer.paths.backup.read_link()?
        );

        installer.blacklist_this_version()?;
        assert_eq!(
            BackupStatus::ThisIsRollback,
            installer.backup_running_version()?
        );

        Ok(())
    }

    #[test]
    fn test_move_bundle_to_install_path() -> Result<()> {
        let tmp_root = TempDir::new()?.to_path_buf();
        let bundle_path = tmp_root.join("bundle");
        let installer = Installer::new(tmp_root);

        assert!(installer
            .move_bundle_to_install_path(bundle_path.join("installer"))
            .is_err());

        fs::create_dir_all(&bundle_path.join("some_dir/with_subdir"))?;
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

    #[test]
    fn test_install_this_version() -> Result<()> {
        let tmp_root = TempDir::new()?.to_path_buf();
        let installer = Installer::new(tmp_root);

        assert!(installer.install_this_version().is_err());

        let this_path = &installer.paths.this_version;
        fs::create_dir_all(this_path)?;
        assert!(installer.install_this_version().is_err());

        fs::create_dir_all(this_path.join(BLOCKVISOR_BIN))?;
        fs::create_dir_all(this_path.join(BLOCKVISOR_SERVICES))?;
        fs::create_dir_all(this_path.join(FC_BIN))?;
        installer.install_this_version()?;

        touch_file(&this_path.join(BLOCKVISOR_BIN).join("some_bin"))?;
        touch_file(&this_path.join(BLOCKVISOR_SERVICES).join("some_service"))?;
        touch_file(&this_path.join(FC_BIN).join("firecracker"))?;
        assert!(installer.install_this_version().is_err());

        fs::create_dir_all(&installer.paths.system_bin)?;
        fs::create_dir_all(&installer.paths.system_services)?;
        installer.install_this_version()?;

        assert!(installer.paths.system_bin.join("some_bin").exists());
        assert!(installer.paths.system_bin.join("firecracker").exists());
        assert!(installer
            .paths
            .system_services
            .join("some_service")
            .exists());

        Ok(())
    }

    #[test]
    fn test_broken_installation() -> Result<()> {
        let tmp_root = TempDir::new()?.to_path_buf();
        let installer = Installer::new(tmp_root.clone());

        assert!(installer
            .handle_broken_installation(BackupStatus::ThisIsRollback, anyhow!("error"))
            .is_err());
        assert!(!installer.is_blacklisted(THIS_VERSION)?);

        fs::create_dir_all(&installer.paths.install_path)?;
        assert!(installer
            .handle_broken_installation(BackupStatus::NothingToBackup, anyhow!("error"))
            .is_err());
        assert!(installer.is_blacklisted(THIS_VERSION)?);

        fs::create_dir_all(&installer.paths.backup)?;
        {
            // create dummy installer that will touch test file as a proof it was called
            let mut backup_installer = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .mode(0o770)
                .open(&installer.paths.backup.join(INSTALLER_BIN))?;
            writeln!(backup_installer, "#!/bin/sh")?;
            writeln!(
                backup_installer,
                "touch {}",
                tmp_root.join("dummy_installer").to_str().unwrap()
            )?;
            writeln!(backup_installer, "exit 1")?;
        }
        let _ = fs::remove_file(tmp_root.join("dummy_installer"));
        assert!(installer
            .handle_broken_installation(BackupStatus::Done, anyhow!("error"))
            .is_err());
        assert!(tmp_root.join("dummy_installer").exists());

        Ok(())
    }

    #[test]
    fn test_cleanup() -> Result<()> {
        let tmp_root = TempDir::new()?.to_path_buf();
        let installer = Installer::new(tmp_root);

        // cant cleanup non existing dir nothing
        assert!(installer.cleanup().is_err());

        fs::create_dir_all(&installer.paths.install_path)?;

        // cleanup empty dir
        installer.cleanup()?;

        touch_file(&installer.paths.install_path.join("some_file"))?;
        fs::create_dir_all(&installer.paths.install_path.join("some_dir"))?;
        touch_file(
            &installer
                .paths
                .install_path
                .join("some_dir")
                .join("another_file"),
        )?;
        std::os::unix::fs::symlink(
            &installer.paths.install_path.join("some_dir"),
            &installer.paths.install_path.join("dir_link"),
        )?;
        std::os::unix::fs::symlink(
            &installer.paths.install_path.join("some_file"),
            &installer.paths.install_path.join("file_link"),
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
