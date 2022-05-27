use serde::{Deserialize, Serialize};
use zbus::{
    dbus_proxy,
    zvariant::{OwnedObjectPath, Type},
    Result,
};

// TODO: Fix and use `zbus_systemd` crate instead.

/// Proxy object for `org.freedesktop.systemd1.Manager`.
#[dbus_proxy(
    interface = "org.freedesktop.systemd1.Manager",
    gen_blocking = false,
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
trait Manager {
    /// [📖](https://www.freedesktop.org/software/systemd/man/systemd.directives.html#StartUnit()) Call interface method `StartUnit`.
    fn start_unit(&self, name: &str, mode: UnitStartMode) -> Result<OwnedObjectPath>;

    /// [📖](https://www.freedesktop.org/software/systemd/man/systemd.directives.html#StopUnit()) Call interface method `StopUnit`.
    fn stop_unit(&self, name: &str, mode: UnitStopMode) -> Result<OwnedObjectPath>;

    /// [📖](https://www.freedesktop.org/software/systemd/man/systemd.directives.html#EnableUnitFiles()) Call interface method `EnableUnitFiles`.
    fn enable_unit_files(
        &self,
        files: &[&str],
        runtime_only: bool,
        force: bool,
    ) -> Result<EnableUnitFilesReturn>;
}

#[derive(Debug, Deserialize, Type)]
pub struct EnableUnitFilesReturn {
    /// Whether the unit files contained any enablement information (i.e. an [Install]) section.
    pub enablement_info: bool,
    /// The changes made as part of the call.
    pub changes: Vec<UnitChange>,
}

#[derive(Debug, Deserialize, Type)]
pub struct UnitChange {
    /// The type of the change.
    pub r#type: UnitChangeType,
    /// The file name of the symlink.
    pub source_file: String,
    /// The file name of the destination of the symlink.
    pub destination_file: String,
}

#[derive(Debug, Deserialize, Type)]
#[serde(rename_all = "kebab-case")]
#[zvariant(signature = "s")]
pub enum UnitChangeType {
    Symlink,
    Unlink,
}

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "kebab-case")]
#[zvariant(signature = "s")]
pub enum UnitStartMode {
    /// Will start the unit and its dependencies, possibly replacing already queued jobs that
    /// conflict with this.
    Replace,
    /// Will start the unit and its dependencies, but will fail if this would change an already
    /// queued job.
    Fail,
    /// Will start the unit in question and terminate all units that aren't dependencies of it.
    Isolate,
    /// Will start a unit but ignore all its dependencies (not recommended).
    IgnoreDependencies,
    /// Will start a unit but only ignore the requirement dependencies (not recommended).
    IgnoreRequirements,
}

/// Mode to use when stopping a unit.
///
/// Same as [`UnitStartMode`], except no `Isolate` variant.
#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "kebab-case")]
#[zvariant(signature = "s")]
pub enum UnitStopMode {
    /// Will stop the unit and its dependencies, possibly replacing already queued jobs that
    /// conflict with this.
    Replace,
    /// Will stop the unit and its dependencies, but will fail if this would change an already
    /// queued job.
    Fail,
    /// Will stop a unit but ignore all its dependencies (not recommended).
    IgnoreDependencies,
    /// Will stop a unit but only ignore the requirement dependencies (not recommended).
    IgnoreRequirements,
}
