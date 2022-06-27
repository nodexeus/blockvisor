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
fn test_bv_cmd_node_lifecycle() {
    use std::str;
    use uuid::Uuid;

    let chain_id = Uuid::new_v4().to_string();

    println!("start service");
    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.arg("start")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "blockvisor service started successfully",
        ));

    println!("create a node");
    let mut cmd = Command::cargo_bin("bv").unwrap();
    let cmd = cmd.args(&["node", "create", "--chain", &chain_id]);
    let output = cmd.output().unwrap();
    let stdout = str::from_utf8(&output.stdout).unwrap();
    println!("create output: {stdout}");
    let vm_id =
        &stdout.trim_start_matches(&format!("Created new node for `{chain_id}` chain with ID "));
    let vm_id = vm_id.trim().trim_matches('`');
    println!("create vm_id: {vm_id}");

    println!("stop stopped node");
    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.args(&["node", "stop", "--id", vm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Stopped node with ID"));

    println!("start stopped node");
    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.args(&["node", "start", "--id", vm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Started node with ID"));

    println!("stop started node");
    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.args(&["node", "stop", "--id", vm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Stopped node with ID"));

    // TODO: (re)start stopped node

    println!("delete started node");
    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.args(&["node", "delete", "--id", vm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted node with ID"));
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
