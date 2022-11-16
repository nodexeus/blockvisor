/// This module implements supervisor for node entry points. It spawn child processes as defined in
/// given config and watch them. Stopped child (whatever reason) is respawned with exponential backoff
/// timeout. Backoff timeout is reset after child stays alive for at least `backoff_timeout_ms`.
use crate::run_flag::RunFlag;
use async_trait::async_trait;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use serde::Deserialize;
use std::marker::PhantomData;
use std::time::SystemTime;
use tokio::process::Command;
use tokio::time::Duration;

#[async_trait]
pub trait Timer {
    fn now() -> SystemTime;
    async fn sleep(duration: Duration);
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    ///  if entry_point stay alive given amount of time (in miliseconds) backof is reset
    pub backoff_timeout_ms: u64,
    /// base time (in miliseconds) for backof, multiplied by consecutive power of 2 each time
    pub backoff_base_ms: u64,
    pub entry_point: Vec<Entrypoint>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Entrypoint {
    pub command: String,
    pub args: Vec<String>,
}

struct Backoff<T: Timer> {
    counter: u32,
    timestamp: SystemTime,
    backoff_base_ms: u64,
    reset_delay: Duration,
    phantom: PhantomData<T>,
}

impl<T: Timer> Backoff<T> {
    fn new(config: &Config) -> Self {
        Self {
            counter: 0,
            timestamp: SystemTime::now(),
            backoff_base_ms: config.backoff_base_ms,
            reset_delay: Duration::from_millis(config.backoff_timeout_ms),
            phantom: Default::default(),
        }
    }

    fn start(&mut self) {
        self.timestamp = T::now();
    }

    async fn wait(&mut self) {
        let now = T::now();
        let duration = now.duration_since(self.timestamp).unwrap_or_default();
        if duration > self.reset_delay {
            self.counter = 0;
        } else {
            T::sleep(Duration::from_millis(
                self.backoff_base_ms * 2u64.pow(self.counter),
            ))
            .await;
            self.counter += 1;
        }
    }
}

async fn run_entrypoint<T: Timer>(mut run: RunFlag, config: &Config, entrypoint: &Entrypoint) {
    let mut cmd = Command::new(&entrypoint.command);
    cmd.args(&entrypoint.args);
    let mut backoff = Backoff::<T>::new(config);
    while run.load() {
        backoff.start();
        if let Ok(mut child) = cmd.spawn() {
            tokio::select!(
                _ = child.wait() => {
                    backoff.wait().await;
                },
                _ = run.wait() => {
                    let _ = child.kill().await;
                },
            );
        } else {
            backoff.wait().await;
        }
    }
}

pub async fn run<T: Timer>(run: RunFlag, config: Config) -> eyre::Result<()> {
    let mut futures = FuturesUnordered::new();
    for entry_point in &config.entry_point {
        futures.push(run_entrypoint::<T>(run.clone(), &config, entry_point));
    }
    while (futures.next().await).is_some() {}
    Ok(())
}
