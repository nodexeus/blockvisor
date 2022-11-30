use crate::log_buffer::LogBuffer;
use crate::run_flag::RunFlag;
use async_trait::async_trait;
use babel_api::config::{Entrypoint, SupervisorConfig as Config};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use std::marker::PhantomData;
use std::process::Stdio;
use std::time::SystemTime;
use tokio::process::Command;
use tokio::sync::broadcast;
use tokio::time::Duration;
use tracing::{info, warn};

/// Time abstraction for better testing.
#[async_trait]
pub trait Timer {
    fn now() -> SystemTime;
    async fn sleep(duration: Duration);
}

/// This module implements supervisor for node entry points. It spawn child processes as defined in
/// given config and watch them. Stopped child (whatever reason) is respawned with exponential backoff
/// timeout. Backoff timeout is reset after child stays alive for at least `backoff_timeout_ms`.
pub struct Supervisor<T: Timer> {
    run: RunFlag,
    config: Config,
    log_buffer: LogBuffer,
    phantom: PhantomData<T>,
}

impl<T: Timer> Supervisor<T> {
    pub fn new(run: RunFlag, config: Config) -> Self {
        let log_buffer = LogBuffer::new(config.log_buffer_capacity_ln);
        Self {
            run,
            config,
            log_buffer,
            phantom: Default::default(),
        }
    }

    pub fn get_logs_rx(&self) -> broadcast::Receiver<String> {
        self.log_buffer.subscribe()
    }

    pub async fn run(self) -> eyre::Result<()> {
        let mut futures = FuturesUnordered::new();
        for entry_point in &self.config.entry_point {
            futures.push(self.run_entrypoint(entry_point));
        }
        while (futures.next().await).is_some() {}
        Ok(())
    }

    async fn run_entrypoint(&self, entrypoint: &Entrypoint) {
        let entry_name = format!("{} {}", entrypoint.command, entrypoint.args.join(" "));
        let mut cmd = Command::new(&entrypoint.command);
        cmd.args(&entrypoint.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut backoff = Backoff::<T>::new(&self.config);
        let mut run = self.run.clone();
        while run.load() {
            backoff.start();
            if let Ok(mut child) = cmd.spawn() {
                info!("Spawned entrypoint '{entry_name}'");
                let logs =
                    self.log_buffer
                        .attach(&entry_name, child.stdout.take(), child.stderr.take());
                tokio::select!(
                    _ = child.wait() => {
                        warn!("Entrypoint stopped unexpected '{entry_name}'");
                        logs.await;
                        backoff.wait().await;
                    },
                    _ = run.wait() => {
                        info!("Supervisor stopped, killing entrypoint '{entry_name}'");
                        let _ = child.kill().await;
                        logs.await;
                    },
                );
            } else {
                warn!("Failed to spawn entrypoint '{entry_name}'");
                backoff.wait().await;
            }
        }
    }
}

struct Backoff<T: Timer> {
    counter: u32,
    timestamp: SystemTime,
    backoff_base_ms: u64,
    reset_timeout: Duration,
    phantom: PhantomData<T>,
}

impl<T: Timer> Backoff<T> {
    fn new(config: &Config) -> Self {
        Self {
            counter: 0,
            timestamp: SystemTime::now(),
            backoff_base_ms: config.backoff_base_ms,
            reset_timeout: Duration::from_millis(config.backoff_timeout_ms),
            phantom: Default::default(),
        }
    }

    fn start(&mut self) {
        self.timestamp = T::now();
    }

    async fn wait(&mut self) {
        let now = T::now();
        let duration = now.duration_since(self.timestamp).unwrap_or_default();
        if duration > self.reset_timeout {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run_flag::RunFlag;
    use assert_fs::TempDir;
    use async_trait::async_trait;
    use mockall::*;
    use serial_test::serial;
    use std::fs;
    use std::ops::Add;
    use std::time::SystemTime;
    use tokio::time::Duration;

    mock! {
        pub TestTimer {}

        #[async_trait]
        impl Timer for TestTimer {
            fn now() -> SystemTime;
            async fn sleep(duration: Duration);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_backoff_timeout_ms() {
        let cfg = Config {
            backoff_timeout_ms: 600,
            backoff_base_ms: 10,
            entry_point: vec![Entrypoint {
                command: "echo".to_owned(),
                args: vec!["test".to_owned()],
            }],
            ..Default::default()
        };
        let run: RunFlag = Default::default();
        let mut test_run = run.clone();

        let now = SystemTime::now();

        let now_ctx = MockTestTimer::now_context();
        now_ctx.expect().once().returning(move || now);
        now_ctx.expect().once().returning(move || {
            test_run.stop();
            now.add(Duration::from_millis(cfg.backoff_timeout_ms + 1))
        });
        assert!(Supervisor::<MockTestTimer>::new(run, cfg)
            .run()
            .await
            .is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_exponential_backoff() {
        let cfg = Config {
            backoff_timeout_ms: 600,
            backoff_base_ms: 10,
            entry_point: vec![Entrypoint {
                command: "echo".to_owned(),
                args: vec!["test".to_owned()],
            }],
            ..Default::default()
        };
        let run: RunFlag = Default::default();
        let mut test_run = run.clone();

        let now = SystemTime::now();

        let now_ctx = MockTestTimer::now_context();
        let sleep_ctx = MockTestTimer::sleep_context();
        const RANGE: u32 = 8;
        for n in 0..RANGE {
            now_ctx.expect().times(2).returning(move || now);
            sleep_ctx
                .expect()
                .with(predicate::eq(Duration::from_millis(10 * 2u64.pow(n))))
                .returning(|_| ());
        }
        now_ctx.expect().times(2).returning(move || now);
        sleep_ctx
            .expect()
            .with(predicate::eq(Duration::from_millis(10 * 2u64.pow(RANGE))))
            .returning(move |_| {
                test_run.stop();
            });
        assert!(Supervisor::<MockTestTimer>::new(run, cfg)
            .run()
            .await
            .is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_multiple_entry_points() {
        let tmp_dir = TempDir::new().unwrap();
        let file_path = tmp_dir.join("test_multiple_entry_points");
        let cfg = Config {
            backoff_timeout_ms: 600,
            backoff_base_ms: 10,
            entry_point: vec![
                Entrypoint {
                    command: "sleep".to_owned(),
                    args: vec!["infinity".to_owned()],
                },
                Entrypoint {
                    command: "sleep".to_owned(),
                    args: vec!["infinity".to_owned()],
                },
                Entrypoint {
                    command: "touch".to_owned(),
                    args: vec![file_path.to_str().unwrap().to_string()],
                },
            ],
            ..Default::default()
        };
        let run: RunFlag = Default::default();
        let mut test_run = run.clone();

        let now = SystemTime::now();

        let now_ctx = MockTestTimer::now_context();
        now_ctx.expect().returning(move || now);
        let sleep_ctx = MockTestTimer::sleep_context();
        let first_file_path = file_path.clone();
        sleep_ctx.expect().once().returning(move |_| {
            assert!(first_file_path.exists());
            let _ = fs::remove_file(&first_file_path);
        });
        let second_file_path = file_path.clone();
        sleep_ctx.expect().once().returning(move |_| {
            assert!(second_file_path.exists());
            let _ = fs::remove_file(&second_file_path);
            test_run.stop();
        });
        let _ = fs::remove_file(&file_path);
        assert!(Supervisor::<MockTestTimer>::new(run, cfg)
            .run()
            .await
            .is_ok());
        assert!(!file_path.exists());
    }

    #[tokio::test]
    #[serial]
    async fn test_no_entry_points() {
        let cfg = Config {
            entry_point: vec![],
            ..Default::default()
        };
        assert!(Supervisor::<MockTestTimer>::new(Default::default(), cfg)
            .run()
            .await
            .is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_logs() {
        let cfg = Config {
            backoff_timeout_ms: 600,
            backoff_base_ms: 10,
            log_buffer_capacity_ln: 10,
            entry_point: vec![Entrypoint {
                command: "echo".to_owned(),
                args: vec!["test".to_owned()],
            }],
        };
        let run: RunFlag = Default::default();
        let mut test_run = run.clone();

        let now = SystemTime::now();

        let now_ctx = MockTestTimer::now_context();
        now_ctx.expect().returning(move || now);
        let sleep_ctx = MockTestTimer::sleep_context();
        sleep_ctx.expect().times(3).returning(|_| ());
        sleep_ctx.expect().returning(move |_| {
            test_run.stop();
        });
        let supervisor = Supervisor::<MockTestTimer>::new(run, cfg);
        let mut rx = supervisor.get_logs_rx();
        assert!(supervisor.run().await.is_ok());

        let mut lines = Vec::default();
        while let Ok(line) = rx.try_recv() {
            lines.push(line);
        }
        assert_eq!(vec!["test\n", "test\n", "test\n", "test\n"], lines);
    }
}
