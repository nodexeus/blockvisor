use anyhow::Result;
use blockvisord::installer;
use blockvisord::installer::Installer;
use std::thread::sleep;
use std::time::{Duration, Instant};

struct SysTimer;

impl installer::Timer for SysTimer {
    fn now() -> Instant {
        Instant::now()
    }
    fn sleep(duration: Duration) {
        sleep(duration)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    Installer::<SysTimer>::default().run().await
}
