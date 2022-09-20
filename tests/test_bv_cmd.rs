#[cfg(target_os = "linux")]
use assert_cmd::Command;
#[cfg(target_os = "linux")]
use assert_fs::TempDir;
#[cfg(target_os = "linux")]
use blockvisord::grpc::pb;
#[cfg(target_os = "linux")]
use futures_util::FutureExt;
#[cfg(target_os = "linux")]
use predicates::prelude::*;
#[cfg(target_os = "linux")]
use serial_test::serial;
#[cfg(target_os = "linux")]
use std::{net::ToSocketAddrs, sync::Arc};
#[cfg(target_os = "linux")]
use tokio::sync::{mpsc, Mutex};
#[cfg(target_os = "linux")]
use tokio::time::{sleep, Duration};
#[cfg(target_os = "linux")]
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
#[cfg(target_os = "linux")]
use tonic::transport::Server;

#[cfg(target_os = "linux")]
mod stub_server;

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
    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.arg("stop")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "blockvisor service stopped successfully",
        ));

    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.arg("start")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "blockvisor service started successfully",
        ));

    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.arg("start")
        .assert()
        .success()
        .stdout(predicate::str::contains("Service already running"));
}

#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_bv_cmd_node_lifecycle() {
    use std::str;
    use uuid::Uuid;

    let chain_id = Uuid::new_v4().to_string();

    println!("create a node");
    let mut cmd = Command::cargo_bin("bv").unwrap();
    let cmd = cmd.args(&["node", "create", &chain_id]);
    let output = cmd.output().unwrap();
    let stdout = str::from_utf8(&output.stdout).unwrap();
    let stderr = str::from_utf8(&output.stderr).unwrap();
    println!("create stdout: {stdout}");
    println!("create stderr: {stderr}");
    let vm_id = stdout
        .trim_start_matches(&format!("Created new node for `{chain_id}` chain with ID "))
        .split('`')
        .nth(1)
        .unwrap();
    println!("create vm_id: {vm_id}");

    println!("stop stopped node");
    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.args(&["node", "stop", vm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Stopped node"));

    println!("start stopped node");
    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.args(&["node", "start", vm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Started node"));

    println!("stop started node");
    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.args(&["node", "stop", vm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Stopped node"));

    println!("restart stopped node");
    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.args(&["node", "start", vm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Started node"));

    println!("delete started node");
    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.args(&["node", "delete", vm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted node"));
}

#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_bv_cmd_init_unknown_otp() {
    let tmp_dir = TempDir::new().unwrap();

    let otp = "NOT_FOUND";
    let (ifa, _ip) = &local_ip_address::list_afinet_netifas().unwrap()[0];
    let url = "http://localhost:8080";

    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.args(&["init", otp])
        .args(&["--ifa", ifa])
        .args(&["--url", url])
        .env("HOME", tmp_dir.as_os_str())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Record not found"));
}

#[tokio::test]
#[serial]
#[cfg(target_os = "linux")]
async fn test_bv_cmd_init_localhost() {
    pub mod ui_pb {
        // https://github.com/tokio-rs/prost/issues/661
        #![allow(clippy::derive_partial_eq_without_eq)]
        tonic::include_proto!("blockjoy.api.ui_v1");
    }
    use base64;
    use tonic::Request;
    use uuid::Uuid;

    let request_id = Uuid::new_v4().to_string();
    let request_id = ui_pb::Uuid {
        value: request_id.clone(),
    };

    let url = "http://localhost:8080";
    let email = "user1@example.com";
    let password = "user1pass";

    let mut client = ui_pb::user_service_client::UserServiceClient::connect(url)
        .await
        .unwrap();

    println!("create user");
    let create_user = ui_pb::CreateUserRequest {
        meta: Some(ui_pb::RequestMeta {
            id: Some(request_id.clone()),
            token: None,
            fields: vec![],
            pagination: None,
        }),
        user: Some(ui_pb::User {
            id: None,
            email: Some(email.to_string()),
            first_name: Some("first".to_string()),
            last_name: Some("last".to_string()),
            created_at: None,
            updated_at: None,
        }),
        password: password.to_string(),
        password_confirmation: password.to_string(),
    };
    let user: ui_pb::CreateUserResponse = client.create(create_user).await.unwrap().into_inner();
    println!("user created: {user:?}");
    assert_eq!(
        user.meta
            .as_ref()
            .unwrap()
            .origin_request_id
            .as_ref()
            .unwrap(),
        &request_id
    );

    let user_id = user
        .meta
        .unwrap()
        .messages
        .first()
        .expect("user_id in messages")
        .clone();

    println!("make admin");
    let db_url = "postgres://blockvisor:password@database:5432/blockvisor_db";
    let db_query = format!(
        r#"update tokens set role='admin'::enum_token_role where user_id='{user_id}'::uuid"#
    );

    Command::new("docker")
        .args(&[
            "compose", "run", "-it", "database", "psql", db_url, "-c", &db_query,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("UPDATE 1"));

    println!("login user");
    let mut client =
        ui_pb::authentication_service_client::AuthenticationServiceClient::connect(url)
            .await
            .unwrap();
    let login_user = ui_pb::LoginUserRequest {
        meta: Some(ui_pb::RequestMeta {
            id: Some(request_id.clone()),
            token: None,
            fields: vec![],
            pagination: None,
        }),
        email: email.to_string(),
        password: password.to_string(),
    };
    let login: ui_pb::LoginUserResponse = client.login(login_user).await.unwrap().into_inner();
    println!("user login: {login:?}");
    let token = login.token.unwrap();

    println!("get user organization id");
    let mut client = ui_pb::organization_service_client::OrganizationServiceClient::connect(url)
        .await
        .unwrap();

    let org_get = ui_pb::GetOrganizationsRequest {
        meta: Some(ui_pb::RequestMeta {
            id: Some(request_id.clone()),
            token: Some(token.clone()),
            fields: vec![],
            pagination: None,
        }),
    };
    let mut request = Request::new(org_get);
    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {}", base64::encode(token.value.clone()))
            .parse()
            .unwrap(),
    );

    let orgs: ui_pb::GetOrganizationsResponse = client.get(request).await.unwrap().into_inner();
    println!("user org: {orgs:?}");
    let org_id = orgs.organizations.first().unwrap().id.as_ref().unwrap();

    println!("create host provision");
    let mut client = ui_pb::host_provision_service_client::HostProvisionServiceClient::connect(url)
        .await
        .unwrap();

    let provision_create = ui_pb::CreateHostProvisionRequest {
        meta: Some(ui_pb::RequestMeta {
            id: Some(request_id.clone()),
            token: Some(token.clone()),
            fields: vec![],
            pagination: None,
        }),
        host_provision: Some(ui_pb::HostProvision {
            id: None,
            org_id: Some(org_id.clone()),
            host_id: None,
            created_at: None,
            claimed_at: None,
            install_cmd: Some("install cmd".to_string()),
        }),
    };
    let mut request = Request::new(provision_create);
    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {}", base64::encode(token.value.clone()))
            .parse()
            .unwrap(),
    );

    let provision: ui_pb::CreateHostProvisionResponse =
        client.create(request).await.unwrap().into_inner();

    println!("host provision: {provision:?}");
    let otp = provision.meta.unwrap().messages.first().unwrap().clone();

    let tmp_dir = TempDir::new().unwrap();

    println!("bv init");
    let (ifa, _ip) = &local_ip_address::list_afinet_netifas().unwrap()[0];
    let url = "http://localhost:8080";

    Command::cargo_bin("bv")
        .unwrap()
        .args(&["init", &otp])
        .args(&["--ifa", ifa])
        .args(&["--url", url])
        .env("HOME", tmp_dir.as_os_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuring blockvisor"));
}

#[tokio::test]
#[serial]
#[cfg(target_os = "linux")]
async fn test_bv_cmd_grpc_commands() {
    use stub_server::StubServer;
    use uuid::Uuid;

    let node_name = "beautiful-node-name".to_string();
    let node_id = Uuid::new_v4().to_string();
    let id = pb::Uuid {
        value: node_id.clone(),
    };
    let command_id = Uuid::new_v4().to_string();
    let meta = pb::CommandMeta {
        api_command_id: Some(pb::Uuid {
            value: command_id.clone(),
        }),
        created_at: None,
    };
    println!("delete existing node, if any");
    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.args(&["node", "delete", &node_name]).assert();

    println!("preparing server");
    let commands = vec![
        // create
        pb::Command {
            r#type: Some(pb::command::Type::Node(pb::NodeCommand {
                id: Some(id.clone()),
                meta: Some(meta.clone()),
                command: Some(pb::node_command::Command::Create(pb::NodeCreate {
                    name: node_name.clone(),
                    image: Some(pb::ContainerImage {
                        url: "helium/node/latest".to_string(),
                    }),
                    blockchain: "helium".to_string(),
                    r#type: pb::NodeType::Node.into(),
                })),
            })),
        },
        // create with same node id
        pb::Command {
            r#type: Some(pb::command::Type::Node(pb::NodeCommand {
                id: Some(id.clone()),
                meta: Some(meta.clone()),
                command: Some(pb::node_command::Command::Create(pb::NodeCreate {
                    name: "some-new-name".to_string(),
                    image: Some(pb::ContainerImage {
                        url: "helium/node/latest".to_string(),
                    }),
                    blockchain: "helium".to_string(),
                    r#type: pb::NodeType::Node.into(),
                })),
            })),
        },
        //  create with same node name
        pb::Command {
            r#type: Some(pb::command::Type::Node(pb::NodeCommand {
                id: Some(pb::Uuid {
                    value: Uuid::new_v4().to_string(),
                }),
                meta: Some(meta.clone()),
                command: Some(pb::node_command::Command::Create(pb::NodeCreate {
                    name: node_name.clone(),
                    image: Some(pb::ContainerImage {
                        url: "helium/node/latest".to_string(),
                    }),
                    blockchain: "helium".to_string(),
                    r#type: pb::NodeType::Node.into(),
                })),
            })),
        },
        // stop stopped
        pb::Command {
            r#type: Some(pb::command::Type::Node(pb::NodeCommand {
                id: Some(id.clone()),
                meta: Some(meta.clone()),
                command: Some(pb::node_command::Command::Stop(pb::NodeStop {})),
            })),
        },
        // start
        pb::Command {
            r#type: Some(pb::command::Type::Node(pb::NodeCommand {
                id: Some(id.clone()),
                meta: Some(meta.clone()),
                command: Some(pb::node_command::Command::Start(pb::NodeStart {})),
            })),
        },
        // start running
        pb::Command {
            r#type: Some(pb::command::Type::Node(pb::NodeCommand {
                id: Some(id.clone()),
                meta: Some(meta.clone()),
                command: Some(pb::node_command::Command::Start(pb::NodeStart {})),
            })),
        },
        // stop
        pb::Command {
            r#type: Some(pb::command::Type::Node(pb::NodeCommand {
                id: Some(id.clone()),
                meta: Some(meta.clone()),
                command: Some(pb::node_command::Command::Stop(pb::NodeStop {})),
            })),
        },
        // restart stopped
        pb::Command {
            r#type: Some(pb::command::Type::Node(pb::NodeCommand {
                id: Some(id.clone()),
                meta: Some(meta.clone()),
                command: Some(pb::node_command::Command::Restart(pb::NodeRestart {})),
            })),
        },
        // restart running
        pb::Command {
            r#type: Some(pb::command::Type::Node(pb::NodeCommand {
                id: Some(id.clone()),
                meta: Some(meta.clone()),
                command: Some(pb::node_command::Command::Restart(pb::NodeRestart {})),
            })),
        },
    ];

    let (updates_tx, updates_rx) = mpsc::channel(128);
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

    let server = StubServer {
        commands: Arc::new(Mutex::new(commands)),
        updates_tx,
        shutdown_tx,
    };

    let server_future = async {
        Server::builder()
            .max_concurrent_streams(1)
            .add_service(pb::command_flow_server::CommandFlowServer::new(server))
            .serve_with_shutdown(
                "0.0.0.0:8081".to_socket_addrs().unwrap().next().unwrap(),
                shutdown_rx.recv().map(drop),
            )
            .await
            .unwrap()
    };

    println!("run server");
    tokio::select! {
        _ = server_future => {},
        _ = sleep(Duration::from_secs(60)) => {},
    };

    println!("list created node");
    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.args(&["node", "list"])
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains(&node_id));

    println!("delete created node");
    let mut cmd = Command::cargo_bin("bv").unwrap();
    cmd.args(&["node", "delete", &node_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted node"));

    println!("check received updates");
    let updates: Vec<_> = ReceiverStream::new(updates_rx)
        .timeout(Duration::from_secs(1))
        .take_while(Result::is_ok)
        .collect()
        .await;
    let updates_count = updates.len();

    println!("got updates: {updates:?}");
    let expected_updates = vec![
        node_update(&node_id, pb::node_info::ContainerStatus::Creating),
        node_update(&node_id, pb::node_info::ContainerStatus::Stopped),
        success_command_update(&command_id),
        error_command_update(
            &command_id,
            format!("Node with id `{node_id}` exists"),
        ),
        error_command_update(
            &command_id,
            format!("Node with name `{node_name}` exists"),
        ),
        node_update(&node_id, pb::node_info::ContainerStatus::Stopping),
        node_update(&node_id, pb::node_info::ContainerStatus::Stopped),
        success_command_update(&command_id),
        node_update(&node_id, pb::node_info::ContainerStatus::Starting),
        node_update(&node_id, pb::node_info::ContainerStatus::Running),
        success_command_update(&command_id),
        node_update(&node_id, pb::node_info::ContainerStatus::Starting),
        error_command_update(
            &command_id,
            "Firecracker API call failed with status=400 Bad Request, body=Some(\"{\\\"fault_message\\\":\\\"The requested operation is not supported after starting the microVM.\\\"}\")".to_string(),
        ),
        node_update(&node_id, pb::node_info::ContainerStatus::Stopping),
        node_update(&node_id, pb::node_info::ContainerStatus::Stopped),
        success_command_update(&command_id),
        node_update(&node_id, pb::node_info::ContainerStatus::Stopping),
        node_update(&node_id, pb::node_info::ContainerStatus::Stopped),
        node_update(&node_id, pb::node_info::ContainerStatus::Starting),
        node_update(&node_id, pb::node_info::ContainerStatus::Running),
        success_command_update(&command_id),
        node_update(&node_id, pb::node_info::ContainerStatus::Stopping),
        node_update(&node_id, pb::node_info::ContainerStatus::Stopped),
        node_update(&node_id, pb::node_info::ContainerStatus::Starting),
        node_update(&node_id, pb::node_info::ContainerStatus::Running),
        success_command_update(&command_id),
        node_update(&node_id, pb::node_info::ContainerStatus::Deleting),
        node_update(&node_id, pb::node_info::ContainerStatus::Deleted),
    ];

    for (actual, expected) in updates.into_iter().zip(expected_updates) {
        assert_eq!(actual.unwrap(), expected);
    }

    assert_eq!(updates_count, 28);
}

#[cfg(target_os = "linux")]
fn node_update(node_id: &str, status: pb::node_info::ContainerStatus) -> pb::InfoUpdate {
    pb::InfoUpdate {
        info: Some(pb::info_update::Info::Node(pb::NodeInfo {
            id: Some(pb::Uuid {
                value: node_id.to_string(),
            }),
            container_status: Some(status.into()),
            ..Default::default()
        })),
    }
}

#[cfg(target_os = "linux")]
fn error_command_update(command_id: &str, message: String) -> pb::InfoUpdate {
    pb::InfoUpdate {
        info: Some(pb::info_update::Info::Command(pb::CommandInfo {
            id: Some(pb::Uuid {
                value: command_id.to_owned(),
            }),
            response: Some(message),
            exit_code: Some(1),
        })),
    }
}

#[cfg(target_os = "linux")]
fn success_command_update(command_id: &str) -> pb::InfoUpdate {
    pb::InfoUpdate {
        info: Some(pb::info_update::Info::Command(pb::CommandInfo {
            id: Some(pb::Uuid {
                value: command_id.to_owned(),
            }),
            response: None,
            exit_code: Some(0),
        })),
    }
}

#[tokio::test]
#[serial]
#[cfg(target_os = "linux")]
async fn test_bv_cmd_grpc_stub_init_reset() {
    use stub_server::StubHostsServer;

    let server = StubHostsServer {};

    let server_future = async {
        Server::builder()
            .max_concurrent_streams(1)
            .add_service(pb::hosts_server::HostsServer::new(server))
            .serve("0.0.0.0:8082".to_socket_addrs().unwrap().next().unwrap())
            .await
            .unwrap()
    };

    tokio::spawn(server_future);
    sleep(Duration::from_secs(5)).await;

    let _ = tokio::task::spawn_blocking(move || {
        let tmp_dir = TempDir::new().unwrap();
        let (ifa, _ip) = &local_ip_address::list_afinet_netifas().unwrap()[0];
        let url = "http://localhost:8082";
        let otp = "AWESOME";

        println!("bv init");
        Command::cargo_bin("bv")
            .unwrap()
            .args(&["init", otp])
            .args(&["--ifa", ifa])
            .args(&["--url", url])
            .env("HOME", tmp_dir.as_os_str())
            .assert()
            .success()
            .stdout(predicate::str::contains("Configuring blockvisor"));

        println!("bv reset");
        Command::cargo_bin("bv")
            .unwrap()
            .args(&["reset", "--yes"])
            .env("HOME", tmp_dir.as_os_str())
            .assert()
            .success()
            .stdout(predicate::str::contains("Deleting host"));
    })
    .await
    .unwrap();
}
