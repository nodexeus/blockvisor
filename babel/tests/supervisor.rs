use async_trait::async_trait;
use babel::run_flag::RunFlag;
use babel::supervisor;
use mockall::*;
use serial_test::serial;
use std::ops::Add;
use std::time::SystemTime;
use std::{env, fs};
use tokio::time::Duration;

mock! {
    pub TestTimer {}

    #[async_trait]
    impl supervisor::Timer for TestTimer {
        fn now() -> SystemTime;
        async fn sleep(duration: Duration);
    }
}

#[tokio::test]
#[serial]
async fn test_backoff_timeout_ms() -> eyre::Result<()> {
    let cfg = supervisor::Config {
        backoff_timeout_ms: 600,
        backoff_base_ms: 10,
        entry_point: vec![supervisor::Entrypoint {
            command: "echo".to_owned(),
            args: vec!["test".to_owned()],
        }],
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

    assert!(supervisor::run::<MockTestTimer>(run, cfg).await.is_ok());
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_exponential_backoff() -> eyre::Result<()> {
    let cfg = supervisor::Config {
        backoff_timeout_ms: 600,
        backoff_base_ms: 10,
        entry_point: vec![supervisor::Entrypoint {
            command: "echo".to_owned(),
            args: vec!["test".to_owned()],
        }],
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
    assert!(supervisor::run::<MockTestTimer>(run, cfg).await.is_ok());
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_multiple_entry_points() -> eyre::Result<()> {
    let file_path = env::temp_dir().join("test_multiple_entry_points");
    let cfg = supervisor::Config {
        backoff_timeout_ms: 600,
        backoff_base_ms: 10,
        entry_point: vec![
            supervisor::Entrypoint {
                command: "sleep".to_owned(),
                args: vec!["infinity".to_owned()],
            },
            supervisor::Entrypoint {
                command: "sleep".to_owned(),
                args: vec!["infinity".to_owned()],
            },
            supervisor::Entrypoint {
                command: "touch".to_owned(),
                args: vec![file_path.to_str().unwrap().to_string()],
            },
        ],
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
    assert!(supervisor::run::<MockTestTimer>(run, cfg).await.is_ok());
    assert!(!file_path.exists());
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_no_entry_points() -> eyre::Result<()> {
    let cfg = supervisor::Config {
        backoff_timeout_ms: 600,
        backoff_base_ms: 10,
        entry_point: vec![],
    };
    assert!(supervisor::run::<MockTestTimer>(Default::default(), cfg)
        .await
        .is_ok());
    Ok(())
}
