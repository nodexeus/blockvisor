#[cfg(target_os = "linux")]
use assert_cmd::Command;
#[cfg(target_os = "linux")]
use assert_fs::TempDir;
#[cfg(target_os = "linux")]
use predicates::prelude::*;
#[cfg(target_os = "linux")]
use serial_test::serial;

#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_bv_cmd_start_no_init() {
    let tmp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.arg("start")
        .env("HOME", tmp_dir.as_os_str())
        .assert()
        .failure()
        .stderr("Error: Host is not registered, please run `init` first\n");
}

#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_bv_cmd_restart() {
    // FIXME: investigate why test is not stable without sleeps
    use std::{thread::sleep, time::Duration};

    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.arg("stop")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "blockvisor service stopped successfully",
        ));
    sleep(Duration::from_secs(1));

    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.arg("start")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "blockvisor service started successfully",
        ));
    sleep(Duration::from_secs(1));
}

#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_bv_cmd_init_unknown_otp() {
    let tmp_dir = TempDir::new().unwrap();

    let otp = "UNKNOWN";
    let (ifa, _ip) = &local_ip_address::list_afinet_netifas().unwrap()[0];
    let url = "https://api.blockvisor.dev";

    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.arg("init")
        .args(&["--otp", otp])
        .args(&["--ifa", ifa])
        .args(&["--url", url])
        .env("HOME", tmp_dir.as_os_str())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Record not found"));
}
