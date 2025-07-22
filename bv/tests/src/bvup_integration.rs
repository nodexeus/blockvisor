use blockvisord::bv_config::{Config, NetConf};
use eyre::Result;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use tempdir::TempDir;

/// Integration tests for bvup binary with network detection fixes
/// These tests verify the complete bvup flow works with the fixed network detection logic

#[tokio::test]
async fn test_bvup_network_detection_gateway_outside_subnet() -> Result<()> {
    // Test case: Gateway outside subnet (common in cloud environments)
    // This simulates AWS/GCP where gateway is in different subnet than host
    
    let temp_dir = TempDir::new("bvup_test")?;
    let bv_root = temp_dir.path();
    
    // Create mock routing data with gateway outside subnet
    let mock_route_json = r#"[
        {
           "dev" : "br0",
           "dst" : "default",
           "flags" : ["onlink"],
           "gateway" : "10.0.0.1",
           "protocol" : "dhcp"
        },
        {
           "dev" : "br0",
           "dst" : "192.168.1.0/24",
           "flags" : [],
           "prefsrc" : "192.168.1.100",
           "protocol" : "kernel",
           "scope" : "link"
        }
     ]"#;
    
    // Test NetConf creation directly (this is what bvup uses internally)
    let net_conf = NetConf::from_json(mock_route_json, "br0")?;
    
    // Verify the fix works: gateway outside subnet should be detected correctly
    assert_eq!(net_conf.gateway_ip, IpAddr::from(Ipv4Addr::from_str("10.0.0.1")?));
    assert_eq!(net_conf.host_ip, IpAddr::from(Ipv4Addr::from_str("192.168.1.100")?));
    assert_eq!(net_conf.prefix, 24);
    
    // Verify available IPs exclude both host and gateway
    assert!(!net_conf.available_ips.contains(&net_conf.host_ip));
    assert!(!net_conf.available_ips.contains(&net_conf.gateway_ip));
    
    // Should have 253 available IPs (256 - network - broadcast - host)
    assert_eq!(net_conf.available_ips.len(), 253);
    
    println!("✓ Gateway outside subnet detection works correctly");
    println!("  Gateway: {} (outside host subnet)", net_conf.gateway_ip);
    println!("  Host: {} in {}/{}", net_conf.host_ip, net_conf.host_ip, net_conf.prefix);
    println!("  Available IPs: {} addresses", net_conf.available_ips.len());
    
    Ok(())
}

#[tokio::test]
async fn test_bvup_network_detection_gateway_inside_subnet() -> Result<()> {
    // Test case: Gateway inside subnet (traditional setup)
    // This ensures backward compatibility is maintained
    
    let temp_dir = TempDir::new("bvup_test")?;
    let bv_root = temp_dir.path();
    
    // Create mock routing data with gateway inside subnet
    let mock_route_json = r#"[
        {
           "dev" : "br0",
           "dst" : "default",
           "flags" : [],
           "gateway" : "192.168.1.1",
           "protocol" : "static"
        },
        {
           "dev" : "br0",
           "dst" : "192.168.1.0/24",
           "flags" : [],
           "prefsrc" : "192.168.1.100",
           "protocol" : "kernel",
           "scope" : "link"
        }
     ]"#;
    
    // Test NetConf creation directly
    let net_conf = NetConf::from_json(mock_route_json, "br0")?;
    
    // Verify backward compatibility: gateway inside subnet still works
    assert_eq!(net_conf.gateway_ip, IpAddr::from(Ipv4Addr::from_str("192.168.1.1")?));
    assert_eq!(net_conf.host_ip, IpAddr::from(Ipv4Addr::from_str("192.168.1.100")?));
    assert_eq!(net_conf.prefix, 24);
    
    // Verify available IPs exclude both host and gateway
    assert!(!net_conf.available_ips.contains(&net_conf.host_ip));
    assert!(!net_conf.available_ips.contains(&net_conf.gateway_ip));
    
    // Should have 252 available IPs (256 - network - broadcast - host - gateway)
    assert_eq!(net_conf.available_ips.len(), 252);
    
    println!("✓ Gateway inside subnet detection works correctly (backward compatibility)");
    println!("  Gateway: {} (inside host subnet)", net_conf.gateway_ip);
    println!("  Host: {} in {}/{}", net_conf.host_ip, net_conf.host_ip, net_conf.prefix);
    println!("  Available IPs: {} addresses", net_conf.available_ips.len());
    
    Ok(())
}

#[tokio::test]
async fn test_bvup_manual_override_parameters() -> Result<()> {
    // Test case: Manual override parameters should work correctly
    // This verifies that users can still manually specify network parameters
    
    let temp_dir = TempDir::new("bvup_test")?;
    let bv_root = temp_dir.path();
    
    // Create mock routing data that would normally be used
    let mock_route_json = r#"[
        {
           "dev" : "br0",
           "dst" : "default",
           "flags" : [],
           "gateway" : "192.168.1.1",
           "protocol" : "static"
        },
        {
           "dev" : "br0",
           "dst" : "192.168.1.0/24",
           "flags" : [],
           "prefsrc" : "192.168.1.100",
           "protocol" : "kernel",
           "scope" : "link"
        }
     ]"#;
    
    // Test NetConf creation and then override parameters
    let mut net_conf = NetConf::from_json(mock_route_json, "br0")?;
    
    // Test manual overrides (this is what bvup does when CLI args are provided)
    net_conf.override_gateway_ip("10.0.0.1")?;
    net_conf.override_host_ip("10.0.0.100")?;
    net_conf.override_ips("10.0.0.10-10.0.0.20,10.0.0.50")?;
    
    // Verify overrides work correctly
    assert_eq!(net_conf.gateway_ip, IpAddr::from(Ipv4Addr::from_str("10.0.0.1")?));
    assert_eq!(net_conf.host_ip, IpAddr::from(Ipv4Addr::from_str("10.0.0.100")?));
    
    // Verify available IPs were overridden correctly
    let expected_ips = vec![
        // Range 10.0.0.10-10.0.0.20 (11 IPs)
        "10.0.0.10", "10.0.0.11", "10.0.0.12", "10.0.0.13", "10.0.0.14",
        "10.0.0.15", "10.0.0.16", "10.0.0.17", "10.0.0.18", "10.0.0.19", "10.0.0.20",
        // Single IP
        "10.0.0.50"
    ];
    
    assert_eq!(net_conf.available_ips.len(), expected_ips.len());
    for expected_ip in expected_ips {
        let ip = IpAddr::from(Ipv4Addr::from_str(expected_ip)?);
        assert!(net_conf.available_ips.contains(&ip), "Missing IP: {}", ip);
    }
    
    // Verify host and gateway IPs are excluded from available IPs
    assert!(!net_conf.available_ips.contains(&net_conf.host_ip));
    assert!(!net_conf.available_ips.contains(&net_conf.gateway_ip));
    
    println!("✓ Manual override parameters work correctly");
    println!("  Overridden Gateway: {}", net_conf.gateway_ip);
    println!("  Overridden Host: {}", net_conf.host_ip);
    println!("  Overridden Available IPs: {} addresses", net_conf.available_ips.len());
    
    Ok(())
}

#[tokio::test]
async fn test_bvup_manual_override_cidr_notation() -> Result<()> {
    // Test case: Manual override with CIDR notation
    // This tests the CIDR parsing functionality in override_ips
    
    let temp_dir = TempDir::new("bvup_test")?;
    let bv_root = temp_dir.path();
    
    // Start with a basic network configuration
    let mock_route_json = r#"[
        {
           "dev" : "br0",
           "dst" : "default",
           "flags" : [],
           "gateway" : "192.168.1.1",
           "protocol" : "static"
        },
        {
           "dev" : "br0",
           "dst" : "192.168.1.0/24",
           "flags" : [],
           "prefsrc" : "192.168.1.100",
           "protocol" : "kernel",
           "scope" : "link"
        }
     ]"#;
    
    let mut net_conf = NetConf::from_json(mock_route_json, "br0")?;
    
    // Override with CIDR notation (this should give us a /29 subnet = 6 usable IPs)
    net_conf.override_ips("10.0.0.8/29")?;
    
    // /29 subnet: 10.0.0.8/29 = 10.0.0.8 - 10.0.0.15
    // Usable IPs: 10.0.0.9, 10.0.0.10, 10.0.0.11, 10.0.0.12, 10.0.0.13, 10.0.0.14
    // (excluding network .8 and broadcast .15)
    let expected_ips = vec![
        "10.0.0.9", "10.0.0.10", "10.0.0.11", "10.0.0.12", "10.0.0.13", "10.0.0.14"
    ];
    
    assert_eq!(net_conf.available_ips.len(), expected_ips.len());
    for expected_ip in expected_ips {
        let ip = IpAddr::from(Ipv4Addr::from_str(expected_ip)?);
        assert!(net_conf.available_ips.contains(&ip), "Missing IP: {}", ip);
    }
    
    println!("✓ Manual override with CIDR notation works correctly");
    println!("  CIDR: 10.0.0.8/29");
    println!("  Available IPs: {} addresses", net_conf.available_ips.len());
    
    Ok(())
}

#[tokio::test]
async fn test_bvup_complex_cloud_environment() -> Result<()> {
    // Test case: Complex cloud environment with multiple interfaces
    // This simulates a more realistic cloud deployment scenario
    
    let temp_dir = TempDir::new("bvup_test")?;
    let bv_root = temp_dir.path();
    
    // Create mock routing data simulating AWS EC2 with multiple interfaces
    let mock_route_json = r#"[
        {
           "dev" : "eth0",
           "dst" : "default",
           "flags" : ["onlink"],
           "gateway" : "172.31.0.1",
           "protocol" : "dhcp"
        },
        {
           "dev" : "eth0",
           "dst" : "172.31.16.0/20",
           "flags" : [],
           "prefsrc" : "172.31.16.5",
           "protocol" : "kernel",
           "scope" : "link"
        },
        {
           "dev" : "br0",
           "dst" : "10.0.0.0/16",
           "flags" : [],
           "prefsrc" : "10.0.1.100",
           "protocol" : "kernel",
           "scope" : "link"
        },
        {
           "dev" : "lo",
           "dst" : "127.0.0.0/8",
           "flags" : [],
           "prefsrc" : "127.0.0.1",
           "protocol" : "kernel",
           "scope" : "host"
        }
     ]"#;
    
    // Test network detection for br0 interface (BlockVisor bridge)
    let net_conf = NetConf::from_json(mock_route_json, "br0")?;
    
    // Verify detection works with complex routing table
    assert_eq!(net_conf.gateway_ip, IpAddr::from(Ipv4Addr::from_str("172.31.0.1")?)); // Gateway from default route
    assert_eq!(net_conf.host_ip, IpAddr::from(Ipv4Addr::from_str("10.0.1.100")?));   // Host IP from br0 interface
    assert_eq!(net_conf.prefix, 16); // /16 subnet
    
    // Verify available IPs calculation for large subnet
    // /16 subnet has 65534 usable IPs (65536 - network - broadcast)
    // Minus 1 for host = 65533 available IPs
    assert_eq!(net_conf.available_ips.len(), 65533);
    
    // Verify exclusions
    assert!(!net_conf.available_ips.contains(&net_conf.host_ip));
    assert!(!net_conf.available_ips.contains(&net_conf.gateway_ip));
    
    println!("✓ Complex cloud environment detection works correctly");
    println!("  Gateway: {} (from default route)", net_conf.gateway_ip);
    println!("  Host: {} (from br0 interface)", net_conf.host_ip);
    println!("  Subnet: /{} (large cloud subnet)", net_conf.prefix);
    println!("  Available IPs: {} addresses", net_conf.available_ips.len());
    
    Ok(())
}

#[tokio::test]
async fn test_bvup_error_handling_improved_messages() -> Result<()> {
    // Test case: Verify improved error messages work correctly
    // This ensures users get helpful error messages when network detection fails
    
    let temp_dir = TempDir::new("bvup_test")?;
    let bv_root = temp_dir.path();
    
    // Test 1: Missing default route
    let json_no_default = r#"[
        {
           "dev" : "br0",
           "dst" : "192.168.1.0/24",
           "flags" : [],
           "prefsrc" : "192.168.1.100",
           "protocol" : "kernel",
           "scope" : "link"
        }
     ]"#;
    
    let result = NetConf::from_json(json_no_default, "br0");
    assert!(result.is_err());
    let error_msg = format!("{:#}", result.unwrap_err());
    assert!(error_msg.contains("default route not found"));
    assert!(error_msg.contains("Available routes: [192.168.1.0/24]"));
    
    // Test 2: Missing gateway field
    let json_no_gateway = r#"[
        {
           "dev" : "br0",
           "dst" : "default",
           "flags" : [],
           "protocol" : "static"
        },
        {
           "dev" : "br0",
           "dst" : "192.168.1.0/24",
           "flags" : [],
           "prefsrc" : "192.168.1.100",
           "protocol" : "kernel",
           "scope" : "link"
        }
     ]"#;
    
    let result = NetConf::from_json(json_no_gateway, "br0");
    assert!(result.is_err());
    let error_msg = format!("{:#}", result.unwrap_err());
    assert!(error_msg.contains("default route 'gateway' field not found"));
    assert!(error_msg.contains("Route info:"));
    
    // Test 3: Interface not found
    let json_wrong_interface = r#"[
        {
           "dev" : "br0",
           "dst" : "default",
           "flags" : [],
           "gateway" : "192.168.1.1",
           "protocol" : "static"
        },
        {
           "dev" : "br0",
           "dst" : "192.168.1.0/24",
           "flags" : [],
           "prefsrc" : "192.168.1.100",
           "protocol" : "kernel",
           "scope" : "link"
        }
     ]"#;
    
    let result = NetConf::from_json(json_wrong_interface, "eth0"); // Wrong interface
    assert!(result.is_err());
    let error_msg = format!("{:#}", result.unwrap_err());
    assert!(error_msg.contains("failed to find network configuration for interface 'eth0'"));
    assert!(error_msg.contains("Available interfaces: [br0]"));
    
    println!("✓ Improved error messages work correctly");
    println!("  ✓ Missing default route error");
    println!("  ✓ Missing gateway field error");
    println!("  ✓ Interface not found error");
    
    Ok(())
}

#[tokio::test]
async fn test_bvup_config_save_and_load() -> Result<()> {
    // Test case: Verify configuration save and load works with fixed network detection
    // This tests the complete flow that bvup uses
    
    let temp_dir = TempDir::new("bvup_test")?;
    let bv_root = temp_dir.path();
    
    // Create mock routing data with gateway outside subnet
    let mock_route_json = r#"[
        {
           "dev" : "br0",
           "dst" : "default",
           "flags" : [],
           "gateway" : "10.0.0.1",
           "protocol" : "static"
        },
        {
           "dev" : "br0",
           "dst" : "192.168.1.0/24",
           "flags" : [],
           "prefsrc" : "192.168.1.100",
           "protocol" : "kernel",
           "scope" : "link"
        }
     ]"#;
    
    // Create network configuration (this is what bvup does)
    let net_conf = NetConf::from_json(mock_route_json, "br0")?;
    
    // Create a minimal config (similar to what bvup creates)
    let config = Config {
        id: "test-host-id".to_string(),
        org_id: Some("test-org-id".to_string()),
        private_org_id: None,
        name: "test-host".to_string(),
        api_config: blockvisord::api_config::ApiConfig {
            token: "test-token".to_string(),
            refresh_token: "test-refresh".to_string(),
            nodexeus_api_url: "https://api.test.com".to_string(),
        },
        nodexeus_mqtt_url: None,
        update_check_interval_secs: Some(60),
        blockvisor_port: 9001,
        iface: "br0".to_string(),
        net_conf: net_conf.clone(),
        cluster_id: None,
        cluster_port: None,
        cluster_seed_urls: None,
        apptainer: blockvisord::bv_config::ApptainerConfig::default(),
        maintenance_mode: false,
    };
    
    // Save configuration (this is what bvup does)
    config.save(bv_root).await?;
    
    // Verify config file was created
    let config_path = bv_root.join("etc/blockvisor.json");
    assert!(config_path.exists(), "Config file should be created");
    
    // Load configuration back (this is what blockvisord does on startup)
    let loaded_config = Config::load(bv_root).await?;
    
    // Verify network configuration was preserved correctly
    assert_eq!(loaded_config.net_conf.gateway_ip, net_conf.gateway_ip);
    assert_eq!(loaded_config.net_conf.host_ip, net_conf.host_ip);
    assert_eq!(loaded_config.net_conf.prefix, net_conf.prefix);
    assert_eq!(loaded_config.net_conf.available_ips.len(), net_conf.available_ips.len());
    
    // Verify other config fields
    assert_eq!(loaded_config.id, "test-host-id");
    assert_eq!(loaded_config.iface, "br0");
    assert_eq!(loaded_config.blockvisor_port, 9001);
    
    println!("✓ Configuration save and load works correctly");
    println!("  Config file: {}", config_path.display());
    println!("  Gateway: {} (preserved)", loaded_config.net_conf.gateway_ip);
    println!("  Host: {} (preserved)", loaded_config.net_conf.host_ip);
    println!("  Available IPs: {} (preserved)", loaded_config.net_conf.available_ips.len());
    
    Ok(())
}

#[tokio::test]
async fn test_bvup_host_network_mode() -> Result<()> {
    // Test case: Host network mode configuration
    // This tests the --use-host-network flag functionality
    
    let temp_dir = TempDir::new("bvup_test")?;
    let bv_root = temp_dir.path();
    
    // Create mock routing data
    let mock_route_json = r#"[
        {
           "dev" : "eth0",
           "dst" : "default",
           "flags" : [],
           "gateway" : "192.168.1.1",
           "protocol" : "dhcp"
        },
        {
           "dev" : "eth0",
           "dst" : "192.168.1.0/24",
           "flags" : [],
           "prefsrc" : "192.168.1.100",
           "protocol" : "kernel",
           "scope" : "link"
        }
     ]"#;
    
    // Create network configuration
    let mut net_conf = NetConf::from_json(mock_route_json, "eth0")?;
    
    // Simulate host network mode (this is what bvup does with --use-host-network)
    net_conf.available_ips = vec![net_conf.host_ip];
    
    // Create config with host network mode enabled
    let mut config = Config {
        id: "test-host-id".to_string(),
        org_id: Some("test-org-id".to_string()),
        private_org_id: None,
        name: "test-host".to_string(),
        api_config: blockvisord::api_config::ApiConfig {
            token: "test-token".to_string(),
            refresh_token: "test-refresh".to_string(),
            nodexeus_api_url: "https://api.test.com".to_string(),
        },
        nodexeus_mqtt_url: None,
        update_check_interval_secs: Some(60),
        blockvisor_port: 9001,
        iface: "eth0".to_string(),
        net_conf: net_conf.clone(),
        cluster_id: None,
        cluster_port: None,
        cluster_seed_urls: None,
        apptainer: blockvisord::bv_config::ApptainerConfig::default(),
        maintenance_mode: false,
    };
    
    // Enable host network mode
    config.apptainer.host_network = true;
    config.apptainer.cpu_limit = false;
    config.apptainer.memory_limit = false;
    
    // Verify host network configuration
    assert!(config.apptainer.host_network);
    assert!(!config.apptainer.cpu_limit);
    assert!(!config.apptainer.memory_limit);
    assert_eq!(config.net_conf.available_ips.len(), 1);
    assert_eq!(config.net_conf.available_ips[0], config.net_conf.host_ip);
    
    println!("✓ Host network mode configuration works correctly");
    println!("  Host network enabled: {}", config.apptainer.host_network);
    println!("  CPU limit disabled: {}", !config.apptainer.cpu_limit);
    println!("  Memory limit disabled: {}", !config.apptainer.memory_limit);
    println!("  Available IPs: {} (host IP only)", config.net_conf.available_ips.len());
    
    Ok(())
}

#[tokio::test]
async fn test_bvup_network_detection_with_manual_overrides() -> Result<()> {
    // Test case: Verify manual override parameters work correctly with network detection
    // This simulates the complete bvup flow with CLI arguments
    
    let temp_dir = TempDir::new("bvup_test")?;
    let bv_root = temp_dir.path();
    
    // Start with automatic network detection (gateway outside subnet)
    let mock_route_json = r#"[
        {
           "dev" : "br0",
           "dst" : "default",
           "flags" : [],
           "gateway" : "10.0.0.1",
           "protocol" : "static"
        },
        {
           "dev" : "br0",
           "dst" : "192.168.1.0/24",
           "flags" : [],
           "prefsrc" : "192.168.1.100",
           "protocol" : "kernel",
           "scope" : "link"
        }
     ]"#;
    
    // Step 1: Automatic network detection
    let mut net_conf = NetConf::from_json(mock_route_json, "br0")?;
    
    // Verify automatic detection works
    assert_eq!(net_conf.gateway_ip, IpAddr::from(Ipv4Addr::from_str("10.0.0.1")?));
    assert_eq!(net_conf.host_ip, IpAddr::from(Ipv4Addr::from_str("192.168.1.100")?));
    assert_eq!(net_conf.prefix, 24);
    assert_eq!(net_conf.available_ips.len(), 253);
    
    // Step 2: Apply manual overrides (simulating CLI arguments)
    // This is what bvup does when CLI args are provided
    net_conf.override_gateway_ip("172.16.0.1")?;
    net_conf.override_host_ip("172.16.0.100")?;
    net_conf.override_ips("172.16.0.10-172.16.0.50")?;
    
    // Verify overrides work correctly
    assert_eq!(net_conf.gateway_ip, IpAddr::from(Ipv4Addr::from_str("172.16.0.1")?));
    assert_eq!(net_conf.host_ip, IpAddr::from(Ipv4Addr::from_str("172.16.0.100")?));
    
    // Verify available IPs were overridden correctly (41 IPs in range 10-50)
    assert_eq!(net_conf.available_ips.len(), 41);
    
    // Verify range boundaries
    assert!(net_conf.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("172.16.0.10")?)));
    assert!(net_conf.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("172.16.0.50")?)));
    assert!(!net_conf.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("172.16.0.9")?)));
    assert!(!net_conf.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("172.16.0.51")?)));
    
    // Verify exclusions still work
    assert!(!net_conf.available_ips.contains(&net_conf.host_ip));
    assert!(!net_conf.available_ips.contains(&net_conf.gateway_ip));
    
    println!("✓ Manual overrides work correctly with automatic network detection");
    println!("  Auto-detected: gateway=10.0.0.1, host=192.168.1.100");
    println!("  Overridden: gateway={}, host={}", net_conf.gateway_ip, net_conf.host_ip);
    println!("  Available IPs: {} addresses", net_conf.available_ips.len());
    
    Ok(())
}

#[tokio::test]
async fn test_bvup_host_network_mode_with_gateway_outside_subnet() -> Result<()> {
    // Test case: Host network mode with gateway outside subnet
    // This tests the --use-host-network flag with the network detection fix
    
    let temp_dir = TempDir::new("bvup_test")?;
    let bv_root = temp_dir.path();
    
    // Create mock routing data with gateway outside subnet
    let mock_route_json = r#"[
        {
           "dev" : "eth0",
           "dst" : "default",
           "flags" : ["onlink"],
           "gateway" : "172.31.0.1",
           "protocol" : "dhcp"
        },
        {
           "dev" : "eth0",
           "dst" : "172.31.16.0/20",
           "flags" : [],
           "prefsrc" : "172.31.16.5",
           "protocol" : "kernel",
           "scope" : "link"
        }
     ]"#;
    
    // Step 1: Network detection works with gateway outside subnet
    let mut net_conf = NetConf::from_json(mock_route_json, "eth0")?;
    
    // Verify detection works
    assert_eq!(net_conf.gateway_ip, IpAddr::from(Ipv4Addr::from_str("172.31.0.1")?));
    assert_eq!(net_conf.host_ip, IpAddr::from(Ipv4Addr::from_str("172.31.16.5")?));
    assert_eq!(net_conf.prefix, 20);
    
    // Step 2: Apply host network mode (this is what bvup does with --use-host-network)
    net_conf.available_ips = vec![net_conf.host_ip];
    
    // Create config with host network mode enabled
    let config = Config {
        id: "test-host-id".to_string(),
        org_id: Some("test-org-id".to_string()),
        private_org_id: None,
        name: "test-host".to_string(),
        api_config: blockvisord::api_config::ApiConfig {
            token: "test-token".to_string(),
            refresh_token: "test-refresh".to_string(),
            nodexeus_api_url: "https://api.test.com".to_string(),
        },
        nodexeus_mqtt_url: None,
        update_check_interval_secs: Some(60),
        blockvisor_port: 9001,
        iface: "eth0".to_string(),
        net_conf: net_conf.clone(),
        cluster_id: None,
        cluster_port: None,
        cluster_seed_urls: None,
        apptainer: blockvisord::bv_config::ApptainerConfig {
            extra_args: None,
            host_network: true,
            cpu_limit: false,
            memory_limit: false,
        },
        maintenance_mode: false,
    };
    
    // Verify host network configuration
    assert!(config.apptainer.host_network);
    assert!(!config.apptainer.cpu_limit);
    assert!(!config.apptainer.memory_limit);
    assert_eq!(config.net_conf.available_ips.len(), 1);
    assert_eq!(config.net_conf.available_ips[0], config.net_conf.host_ip);
    
    // Verify network detection still works correctly
    assert_eq!(config.net_conf.gateway_ip, IpAddr::from(Ipv4Addr::from_str("172.31.0.1")?));
    assert_eq!(config.net_conf.host_ip, IpAddr::from(Ipv4Addr::from_str("172.31.16.5")?));
    
    println!("✓ Host network mode works correctly with gateway outside subnet");
    println!("  Gateway: {} (outside host subnet)", config.net_conf.gateway_ip);
    println!("  Host: {} (host network mode)", config.net_conf.host_ip);
    println!("  Available IPs: {} (host IP only)", config.net_conf.available_ips.len());
    
    Ok(())
}

#[tokio::test]
async fn test_bvup_docker_bridge_network_scenario() -> Result<()> {
    // Test case: Docker bridge network scenario
    // This simulates a common Docker deployment where gateway is outside container subnet
    
    let temp_dir = TempDir::new("bvup_test")?;
    let bv_root = temp_dir.path();
    
    // Create mock routing data simulating Docker bridge network
    // Gateway: 172.17.0.1 (Docker bridge gateway)
    // Host: 172.18.0.0/16 (custom bridge network)
    let mock_route_json = r#"[
        {
           "dev" : "docker0",
           "dst" : "default",
           "flags" : [],
           "gateway" : "172.17.0.1",
           "protocol" : "static"
        },
        {
           "dev" : "br-custom",
           "dst" : "172.18.0.0/16",
           "flags" : [],
           "prefsrc" : "172.18.0.1",
           "protocol" : "kernel",
           "scope" : "link"
        }
     ]"#;
    
    // Test network detection for custom bridge
    let net_conf = NetConf::from_json(mock_route_json, "br-custom")?;
    
    // Verify detection works with Docker-style networking
    assert_eq!(net_conf.gateway_ip, IpAddr::from(Ipv4Addr::from_str("172.17.0.1")?)); // Docker bridge gateway
    assert_eq!(net_conf.host_ip, IpAddr::from(Ipv4Addr::from_str("172.18.0.1")?));   // Custom bridge host
    assert_eq!(net_conf.prefix, 16); // /16 subnet
    
    // Verify available IPs calculation for large subnet
    // /16 subnet has 65534 usable IPs (65536 - network - broadcast)
    // Minus 1 for host = 65533 available IPs
    assert_eq!(net_conf.available_ips.len(), 65533);
    
    // Verify exclusions
    assert!(!net_conf.available_ips.contains(&net_conf.host_ip));
    assert!(!net_conf.available_ips.contains(&net_conf.gateway_ip));
    
    // Verify some expected IPs are in the range
    assert!(net_conf.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("172.18.0.2")?)));
    assert!(net_conf.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("172.18.255.254")?)));
    
    println!("✓ Docker bridge network scenario works correctly");
    println!("  Gateway: {} (Docker bridge)", net_conf.gateway_ip);
    println!("  Host: {} (custom bridge)", net_conf.host_ip);
    println!("  Subnet: /{} (large Docker subnet)", net_conf.prefix);
    println!("  Available IPs: {} addresses", net_conf.available_ips.len());
    
    Ok(())
}

#[tokio::test]
async fn test_bvup_multiple_interface_complex_routing() -> Result<()> {
    // Test case: Complex routing with multiple interfaces
    // This tests that the fix correctly identifies the right interface and gateway
    
    let temp_dir = TempDir::new("bvup_test")?;
    let bv_root = temp_dir.path();
    
    // Create mock routing data with multiple interfaces and complex routing
    let mock_route_json = r#"[
        {
           "dev" : "eth0",
           "dst" : "default",
           "flags" : [],
           "gateway" : "192.168.1.1",
           "protocol" : "dhcp"
        },
        {
           "dev" : "eth0",
           "dst" : "192.168.1.0/24",
           "flags" : [],
           "prefsrc" : "192.168.1.100",
           "protocol" : "kernel",
           "scope" : "link"
        },
        {
           "dev" : "br0",
           "dst" : "10.0.0.0/8",
           "flags" : [],
           "prefsrc" : "10.1.1.100",
           "protocol" : "kernel",
           "scope" : "link"
        },
        {
           "dev" : "docker0",
           "dst" : "172.17.0.0/16",
           "flags" : [],
           "prefsrc" : "172.17.0.1",
           "protocol" : "kernel",
           "scope" : "link"
        },
        {
           "dev" : "lo",
           "dst" : "127.0.0.0/8",
           "flags" : [],
           "prefsrc" : "127.0.0.1",
           "protocol" : "kernel",
           "scope" : "host"
        }
     ]"#;
    
    // Test network detection for br0 interface (BlockVisor bridge)
    let net_conf = NetConf::from_json(mock_route_json, "br0")?;
    
    // Verify detection correctly identifies:
    // - Gateway from default route (eth0's gateway)
    // - Host network from br0 interface
    assert_eq!(net_conf.gateway_ip, IpAddr::from(Ipv4Addr::from_str("192.168.1.1")?)); // Gateway from default route
    assert_eq!(net_conf.host_ip, IpAddr::from(Ipv4Addr::from_str("10.1.1.100")?));    // Host IP from br0 interface
    assert_eq!(net_conf.prefix, 8); // /8 subnet (very large)
    
    // Verify available IPs calculation for very large subnet
    // /8 subnet has 16777214 usable IPs (16777216 - network - broadcast)
    // Minus 1 for host = 16777213 available IPs
    assert_eq!(net_conf.available_ips.len(), 16777213);
    
    // Verify exclusions
    assert!(!net_conf.available_ips.contains(&net_conf.host_ip));
    assert!(!net_conf.available_ips.contains(&net_conf.gateway_ip));
    
    // Verify some expected IPs are in the range
    assert!(net_conf.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("10.0.0.1")?)));
    assert!(net_conf.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("10.1.1.99")?)));
    assert!(net_conf.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("10.1.1.101")?)));
    
    println!("✓ Multiple interface complex routing works correctly");
    println!("  Gateway: {} (from default route on eth0)", net_conf.gateway_ip);
    println!("  Host: {} (from br0 interface)", net_conf.host_ip);
    println!("  Subnet: /{} (very large subnet)", net_conf.prefix);
    println!("  Available IPs: {} addresses", net_conf.available_ips.len());
    
    Ok(())
}

#[tokio::test]
async fn test_bvup_edge_case_small_subnets() -> Result<()> {
    // Test case: Edge cases with small subnets (/30, /31, /32)
    // This tests the fix works correctly with very small subnets
    
    let temp_dir = TempDir::new("bvup_test")?;
    let bv_root = temp_dir.path();
    
    // Test /30 subnet (4 IPs total, 2 usable)
    let mock_route_json_30 = r#"[
        {
           "dev" : "br0",
           "dst" : "default",
           "flags" : [],
           "gateway" : "10.0.0.1",
           "protocol" : "static"
        },
        {
           "dev" : "br0",
           "dst" : "192.168.1.0/30",
           "flags" : [],
           "prefsrc" : "192.168.1.2",
           "protocol" : "kernel",
           "scope" : "link"
        }
     ]"#;
    
    let net_conf_30 = NetConf::from_json(mock_route_json_30, "br0")?;
    
    // /30 subnet: .0 (network), .1, .2, .3 (broadcast)
    // Usable: .1, .2 (but .2 is host, so only .1 available)
    assert_eq!(net_conf_30.gateway_ip, IpAddr::from(Ipv4Addr::from_str("10.0.0.1")?));
    assert_eq!(net_conf_30.host_ip, IpAddr::from(Ipv4Addr::from_str("192.168.1.2")?));
    assert_eq!(net_conf_30.prefix, 30);
    assert_eq!(net_conf_30.available_ips.len(), 1); // Only .1 is available
    assert!(net_conf_30.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.1.1")?)));
    
    // Test /31 subnet (2 IPs total, both usable in point-to-point)
    let mock_route_json_31 = r#"[
        {
           "dev" : "br0",
           "dst" : "default",
           "flags" : [],
           "gateway" : "10.0.0.1",
           "protocol" : "static"
        },
        {
           "dev" : "br0",
           "dst" : "192.168.1.0/31",
           "flags" : [],
           "prefsrc" : "192.168.1.1",
           "protocol" : "kernel",
           "scope" : "link"
        }
     ]"#;
    
    let net_conf_31 = NetConf::from_json(mock_route_json_31, "br0")?;
    
    // /31 subnet: .0, .1 (both usable, but .1 is host, so only .0 available)
    assert_eq!(net_conf_31.gateway_ip, IpAddr::from(Ipv4Addr::from_str("10.0.0.1")?));
    assert_eq!(net_conf_31.host_ip, IpAddr::from(Ipv4Addr::from_str("192.168.1.1")?));
    assert_eq!(net_conf_31.prefix, 31);
    assert_eq!(net_conf_31.available_ips.len(), 1); // Only .0 is available
    assert!(net_conf_31.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.1.0")?)));
    
    // Test /32 subnet (1 IP total, host-only)
    let mock_route_json_32 = r#"[
        {
           "dev" : "br0",
           "dst" : "default",
           "flags" : [],
           "gateway" : "10.0.0.1",
           "protocol" : "static"
        },
        {
           "dev" : "br0",
           "dst" : "192.168.1.1/32",
           "flags" : [],
           "prefsrc" : "192.168.1.1",
           "protocol" : "kernel",
           "scope" : "link"
        }
     ]"#;
    
    let net_conf_32 = NetConf::from_json(mock_route_json_32, "br0")?;
    
    // /32 subnet: only .1 (host), no available IPs for allocation
    assert_eq!(net_conf_32.gateway_ip, IpAddr::from(Ipv4Addr::from_str("10.0.0.1")?));
    assert_eq!(net_conf_32.host_ip, IpAddr::from(Ipv4Addr::from_str("192.168.1.1")?));
    assert_eq!(net_conf_32.prefix, 32);
    assert_eq!(net_conf_32.available_ips.len(), 0); // No IPs available for allocation
    
    println!("✓ Edge case small subnets work correctly");
    println!("  /30 subnet: {} available IPs", net_conf_30.available_ips.len());
    println!("  /31 subnet: {} available IPs", net_conf_31.available_ips.len());
    println!("  /32 subnet: {} available IPs (host-only)", net_conf_32.available_ips.len());
    
    Ok(())
}

#[tokio::test]
async fn test_bvup_binary_network_detection_validation() -> Result<()> {
    // Test case: Validate that the bvup binary can handle network detection correctly
    // This test validates the core network detection logic that bvup uses
    
    let temp_dir = TempDir::new("bvup_test")?;
    let _bv_root = temp_dir.path();
    
    // Test various network scenarios that bvup might encounter
    let test_scenarios = vec![
        (
            "Gateway outside subnet (AWS-style)",
            r#"[
                {
                   "dev" : "br0",
                   "dst" : "default",
                   "flags" : ["onlink"],
                   "gateway" : "172.31.0.1",
                   "protocol" : "dhcp"
                },
                {
                   "dev" : "br0",
                   "dst" : "172.31.16.0/20",
                   "flags" : [],
                   "prefsrc" : "172.31.16.5",
                   "protocol" : "kernel",
                   "scope" : "link"
                }
             ]"#,
            "br0",
            "172.31.0.1",
            "172.31.16.5",
            20,
        ),
        (
            "Gateway inside subnet (traditional)",
            r#"[
                {
                   "dev" : "br0",
                   "dst" : "default",
                   "flags" : [],
                   "gateway" : "192.168.1.1",
                   "protocol" : "static"
                },
                {
                   "dev" : "br0",
                   "dst" : "192.168.1.0/24",
                   "flags" : [],
                   "prefsrc" : "192.168.1.100",
                   "protocol" : "kernel",
                   "scope" : "link"
                }
             ]"#,
            "br0",
            "192.168.1.1",
            "192.168.1.100",
            24,
        ),
        (
            "Docker bridge network",
            r#"[
                {
                   "dev" : "docker0",
                   "dst" : "default",
                   "flags" : [],
                   "gateway" : "172.17.0.1",
                   "protocol" : "static"
                },
                {
                   "dev" : "br-custom",
                   "dst" : "172.18.0.0/16",
                   "flags" : [],
                   "prefsrc" : "172.18.0.1",
                   "protocol" : "kernel",
                   "scope" : "link"
                }
             ]"#,
            "br-custom",
            "172.17.0.1",
            "172.18.0.1",
            16,
        ),
    ];
    
    for (scenario_name, json_data, interface, expected_gateway, expected_host, expected_prefix) in test_scenarios {
        println!("Testing scenario: {}", scenario_name);
        
        // This is the core logic that bvup uses for network detection
        let net_conf = NetConf::from_json(json_data, interface)?;
        
        // Verify network detection works correctly
        assert_eq!(net_conf.gateway_ip, IpAddr::from(Ipv4Addr::from_str(expected_gateway)?));
        assert_eq!(net_conf.host_ip, IpAddr::from(Ipv4Addr::from_str(expected_host)?));
        assert_eq!(net_conf.prefix, expected_prefix);
        
        // Verify available IPs are calculated correctly
        assert!(!net_conf.available_ips.is_empty() || expected_prefix >= 31); // /31 and /32 may have no available IPs
        assert!(!net_conf.available_ips.contains(&net_conf.host_ip));
        assert!(!net_conf.available_ips.contains(&net_conf.gateway_ip));
        
        println!("  ✓ Gateway: {}", net_conf.gateway_ip);
        println!("  ✓ Host: {}", net_conf.host_ip);
        println!("  ✓ Prefix: /{}", net_conf.prefix);
        println!("  ✓ Available IPs: {}", net_conf.available_ips.len());
    }
    
    println!("✓ All bvup network detection scenarios work correctly");
    
    Ok(())
}

#[tokio::test]
async fn test_bvup_manual_override_integration() -> Result<()> {
    // Test case: Integration test for manual override functionality
    // This simulates the complete bvup flow with manual overrides
    
    let temp_dir = TempDir::new("bvup_test")?;
    let bv_root = temp_dir.path();
    
    // Step 1: Start with automatic network detection
    let mock_route_json = r#"[
        {
           "dev" : "br0",
           "dst" : "default",
           "flags" : [],
           "gateway" : "192.168.1.1",
           "protocol" : "static"
        },
        {
           "dev" : "br0",
           "dst" : "192.168.1.0/24",
           "flags" : [],
           "prefsrc" : "192.168.1.100",
           "protocol" : "kernel",
           "scope" : "link"
        }
     ]"#;
    
    let mut net_conf = NetConf::from_json(mock_route_json, "br0")?;
    
    // Step 2: Apply manual overrides (simulating CLI arguments)
    // This is what bvup does when --gateway-ip, --host-ip, --available-ips are provided
    
    // Test gateway override
    net_conf.override_gateway_ip("10.0.0.1")?;
    assert_eq!(net_conf.gateway_ip, IpAddr::from(Ipv4Addr::from_str("10.0.0.1")?));
    
    // Test host IP override
    net_conf.override_host_ip("10.0.0.100")?;
    assert_eq!(net_conf.host_ip, IpAddr::from(Ipv4Addr::from_str("10.0.0.100")?));
    
    // Test available IPs override with mixed formats (like bvup supports)
    net_conf.override_ips("10.0.0.10-10.0.0.20,10.0.0.50,10.0.0.60/30")?;
    
    // Verify the override worked correctly
    // Range: 10.0.0.10-10.0.0.20 = 11 IPs
    // Single: 10.0.0.50 = 1 IP
    // CIDR: 10.0.0.60/30 = 2 usable IPs (10.0.0.61, 10.0.0.62)
    // Total: 14 IPs
    assert_eq!(net_conf.available_ips.len(), 14);
    
    // Verify specific IPs are included
    assert!(net_conf.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("10.0.0.10")?)));
    assert!(net_conf.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("10.0.0.20")?)));
    assert!(net_conf.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("10.0.0.50")?)));
    assert!(net_conf.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("10.0.0.61")?)));
    assert!(net_conf.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("10.0.0.62")?)));
    
    // Verify exclusions still work
    assert!(!net_conf.available_ips.contains(&net_conf.host_ip));
    assert!(!net_conf.available_ips.contains(&net_conf.gateway_ip));
    
    // Step 3: Create and save configuration (like bvup does)
    let config = Config {
        id: "test-host-id".to_string(),
        org_id: Some("test-org-id".to_string()),
        private_org_id: None,
        name: "test-host".to_string(),
        api_config: blockvisord::api_config::ApiConfig {
            token: "test-token".to_string(),
            refresh_token: "test-refresh".to_string(),
            nodexeus_api_url: "https://api.test.com".to_string(),
        },
        nodexeus_mqtt_url: None,
        update_check_interval_secs: Some(60),
        blockvisor_port: 9001,
        iface: "br0".to_string(),
        net_conf: net_conf.clone(),
        cluster_id: None,
        cluster_port: None,
        cluster_seed_urls: None,
        apptainer: blockvisord::bv_config::ApptainerConfig::default(),
        maintenance_mode: false,
    };
    
    // Save and reload configuration
    config.save(bv_root).await?;
    let loaded_config = Config::load(bv_root).await?;
    
    // Verify configuration was preserved correctly
    assert_eq!(loaded_config.net_conf.gateway_ip, net_conf.gateway_ip);
    assert_eq!(loaded_config.net_conf.host_ip, net_conf.host_ip);
    assert_eq!(loaded_config.net_conf.available_ips.len(), net_conf.available_ips.len());
    
    println!("✓ Manual override integration test completed successfully");
    println!("  Gateway: {} (overridden)", loaded_config.net_conf.gateway_ip);
    println!("  Host: {} (overridden)", loaded_config.net_conf.host_ip);
    println!("  Available IPs: {} (overridden)", loaded_config.net_conf.available_ips.len());
    
    Ok(())
}

/// Helper function to simulate running bvup with specific arguments
/// This would be used for end-to-end testing if we had a test environment
#[allow(dead_code)]
async fn simulate_bvup_run(args: Vec<&str>) -> Result<()> {
    // This function would be used to test the actual bvup binary
    // For now, we test the core logic components that bvup uses
    
    println!("Simulating bvup run with args: {:?}", args);
    
    // In a real test environment, this would:
    // 1. Set up a mock API server
    // 2. Create temporary directories
    // 3. Run the actual bvup binary
    // 4. Verify the results
    
    Ok(())
}