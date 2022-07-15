use anyhow::{bail, Context, Result};
use tokio::process::Command;
use tracing::trace;

/// Runs the specified command and returns error on failure.
pub async fn run_cmd(cmd: &str, args: &[&str]) -> Result<()> {
    let mut cmd = Command::new(cmd);
    cmd.args(args);
    trace!("Running command: `{:?}`", cmd);
    match cmd
        .status()
        .await
        .with_context(|| format!("Failed to run command `{:?}`", cmd))?
        .code()
    {
        Some(code) if code != 0 => bail!("Command `{:?}` failed with exit code {}", cmd, code),
        Some(_) => Ok(()),
        None => bail!("Command `{:?}` failed with no exit code", cmd),
    }
}
