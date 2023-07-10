use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

#[test]
fn test_bvup_unknown_provision_token() {
    let tmp_dir = TempDir::new().unwrap();

    let provision_token = "NOT_FOUND";
    let (ifa, _ip) = &local_ip_address::list_afinet_netifas().unwrap()[0];
    let url = "http://localhost:8080";
    let mqtt = "mqtt://localhost:1883";

    let mut cmd = Command::cargo_bin("bvup").unwrap();
    cmd.args([provision_token, "--skip-download"])
        .args(["--ifa", ifa])
        .args(["--api", url])
        .args(["--mqtt", mqtt])
        .args(["--ip-gateway", "216.18.214.193"])
        .args(["--ip-range-from", "216.18.214.195"])
        .args(["--ip-range-to", "216.18.214.206"])
        .args(["--yes"])
        .env("BV_ROOT", tmp_dir.as_os_str())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid token"));
}
