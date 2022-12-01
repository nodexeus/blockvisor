use anyhow::Result;
use blockvisord::installer::Installer;

fn main() -> Result<()> {
    Installer::default().run()
}
