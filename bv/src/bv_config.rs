use crate::{api_config::ApiConfig, services::AuthToken, utils};
use bv_utils::cmd::run_cmd;
use cidr_utils::cidr::IpCidr;
use eyre::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{fmt, net::IpAddr, path::Path, str::FromStr};
use sysinfo::{System, SystemExt};
use tokio::fs;
use tracing::debug;

/// Specific error types for network detection failures
#[derive(Debug)]
pub enum NetworkDetectionError {
    /// No default route found in routing table
    DefaultRouteNotFound {
        available_routes: Vec<String>,
    },
    /// Default route exists but missing gateway field
    DefaultRouteGatewayMissing {
        default_route_info: String,
    },
    /// Gateway IP address format is invalid
    InvalidGatewayIp {
        gateway_str: String,
        parse_error: String,
    },
    /// No network configuration found for specified interface
    InterfaceConfigNotFound {
        interface: String,
        available_interfaces: Vec<String>,
        searched_routes: Vec<String>,
    },
    /// Interface route missing prefsrc field
    InterfacePrefSrcMissing {
        interface: String,
        route_info: String,
    },
    /// Host IP address format is invalid
    InvalidHostIp {
        host_ip_str: String,
        parse_error: String,
    },
    /// CIDR format is invalid
    InvalidCidr {
        cidr_str: String,
        parse_error: String,
    },
    /// Failed to resolve IP range from CIDR
    IpRangeResolutionFailed {
        cidr: String,
        prefix: u8,
    },
    /// JSON parsing error
    JsonParseError(String),
}

impl fmt::Display for NetworkDetectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkDetectionError::DefaultRouteNotFound { available_routes } => {
                write!(f, "default route not found. Available routes: [{}]", 
                       available_routes.join(", "))
            }
            NetworkDetectionError::DefaultRouteGatewayMissing { default_route_info } => {
                write!(f, "default route 'gateway' field not found. Route info: {}", 
                       default_route_info)
            }
            NetworkDetectionError::InvalidGatewayIp { gateway_str, parse_error } => {
                write!(f, "failed to parse gateway ip '{}': {}", gateway_str, parse_error)
            }
            NetworkDetectionError::InterfaceConfigNotFound { interface, available_interfaces, searched_routes } => {
                write!(f, "failed to find network configuration for interface '{}'. Available interfaces: [{}]. Searched routes: [{}]", 
                       interface, available_interfaces.join(", "), searched_routes.join(", "))
            }
            NetworkDetectionError::InterfacePrefSrcMissing { interface, route_info } => {
                write!(f, "'prefsrc' not found for interface '{}'. Route info: {}", 
                       interface, route_info)
            }
            NetworkDetectionError::InvalidHostIp { host_ip_str, parse_error } => {
                write!(f, "cannot parse host ip '{}': {}", host_ip_str, parse_error)
            }
            NetworkDetectionError::InvalidCidr { cidr_str, parse_error } => {
                write!(f, "cannot parse '{}' as cidr: {}", cidr_str, parse_error)
            }
            NetworkDetectionError::IpRangeResolutionFailed { cidr, prefix } => {
                write!(f, "Failed to resolve ip range from CIDR {} (prefix: {})", cidr, prefix)
            }
            NetworkDetectionError::JsonParseError(msg) => {
                write!(f, "JSON parsing error: {}", msg)
            }
        }
    }
}

impl std::error::Error for NetworkDetectionError {}

pub const CONFIG_PATH: &str = "etc/blockvisor.json";
pub const DEFAULT_BRIDGE_IFACE: &str = "br0";

pub fn default_blockvisor_port() -> u16 {
    9001
}

pub fn default_iface() -> String {
    DEFAULT_BRIDGE_IFACE.to_string()
}

pub fn default_hostname() -> String {
    let mut sys = System::new_all();
    sys.refresh_all();
    sys.host_name().unwrap_or_default()
}

#[derive(Debug, Clone)]
pub struct SharedConfig {
    pub config: std::sync::Arc<tokio::sync::RwLock<Config>>,
    pub bv_root: std::path::PathBuf,
}

impl SharedConfig {
    pub fn new(config: Config, bv_root: std::path::PathBuf) -> Self {
        Self {
            config: std::sync::Arc::new(tokio::sync::RwLock::new(config)),
            bv_root,
        }
    }

    pub async fn read(&self) -> Config {
        self.config.read().await.clone()
    }

    pub async fn set_mqtt_url(&self, mqtt_url: Option<String>) {
        self.config.write().await.nodexeus_mqtt_url = mqtt_url;
    }

    pub async fn token(&self) -> Result<AuthToken, tonic::Status> {
        let api_config = self.read().await.api_config.clone();
        Ok(AuthToken(if api_config.token_expired()? {
            let token = {
                let mut write_lock = self.config.write().await;
                // A concurrent update may have written to the jwt field, check if the token has become
                // unexpired while we have unique access.
                if write_lock.api_config.token_expired()? {
                    write_lock.api_config.refresh_token().await?;
                }
                write_lock.api_config.token.clone()
            };
            self.read()
                .await
                .save(&self.bv_root)
                .await
                .map_err(|err| tonic::Status::internal(format!("failed to save token: {err:#}")))?;
            token
        } else {
            api_config.token
        }))
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ApptainerConfig {
    pub extra_args: Option<Vec<String>>,
    pub host_network: bool,
    pub cpu_limit: bool,
    pub memory_limit: bool,
}

impl Default for ApptainerConfig {
    fn default() -> Self {
        Self {
            extra_args: None,
            host_network: false,
            cpu_limit: true,
            memory_limit: true,
        }
    }
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    /// Host uuid
    pub id: String,
    /// Org id the host belongs to.
    pub org_id: Option<String>,
    /// Org uuid to which private host belongs to.
    pub private_org_id: Option<String>,
    /// Host name
    #[serde(default = "default_hostname")]
    pub name: String,
    #[serde(flatten)]
    pub api_config: ApiConfig,
    /// Url for mqtt broker to receive commands and updates from.
    pub nodexeus_mqtt_url: Option<String>,
    /// Self update check interval - how often blockvisor shall check for new version of itself
    pub update_check_interval_secs: Option<u64>,
    /// Port to be used by blockvisor internal service
    /// 0 has special meaning - pick first free port
    #[serde(default = "default_blockvisor_port")]
    pub blockvisor_port: u16,
    /// Network interface name
    #[serde(default = "default_iface")]
    pub iface: String,
    #[serde(default)]
    pub net_conf: NetConf,
    /// Host's cluster id
    pub cluster_id: Option<String>,
    /// Cluster gossip listen port
    pub cluster_port: Option<u32>,
    /// Addresses of the seed nodes for cluster discovery and announcements
    pub cluster_seed_urls: Option<Vec<String>>,
    /// Apptainer configuration
    pub apptainer: ApptainerConfig,
    /// Run in maintenance mode - use on your own risk.
    #[serde(default)]
    pub maintenance_mode: bool,
}

impl Config {
    pub async fn load(bv_root: &Path) -> Result<Config> {
        let path = bv_root.join(CONFIG_PATH);
        debug!("Reading host config: {}", path.display());
        let config = fs::read_to_string(&path)
            .await
            .with_context(|| format!("failed to read host config: {}", path.display()))?;
        let mut config: Config = serde_json::from_str(&config)
            .with_context(|| format!("failed to parse host config: {}", path.display()))?;
        if config.net_conf.host_ip.is_unspecified() {
            config.net_conf = NetConf::new(&config.iface).await?;
        }
        config.save(bv_root).await?;
        Ok(config)
    }

    pub async fn save(&self, bv_root: &Path) -> Result<()> {
        let path = bv_root.join(CONFIG_PATH);
        let parent = path.parent().unwrap();
        debug!("Ensuring config dir is present: {}", parent.display());
        fs::create_dir_all(parent).await?;
        debug!("Writing host config: {}", path.display());
        utils::careful_save(&path, serde_json::to_string(&self)?.as_bytes()).await?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct NetConf {
    pub gateway_ip: IpAddr,
    pub host_ip: IpAddr,
    pub prefix: u8,
    pub available_ips: Vec<IpAddr>,
}

impl Default for NetConf {
    fn default() -> Self {
        Self {
            gateway_ip: IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
            host_ip: IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
            prefix: 32,
            available_ips: vec![],
        }
    }
}

impl NetConf {
    pub async fn new(ifa_name: &str) -> Result<Self> {
        Self::from_json(&run_cmd("ip", ["--json", "route"]).await?, ifa_name)
    }

    fn from_json(json: &str, ifa_name: &str) -> Result<Self> {
        debug!("Starting network detection for interface: {}", ifa_name);
        
        let routes = serde_json::from_str::<Vec<IpRoute>>(json)
            .map_err(|e| NetworkDetectionError::JsonParseError(e.to_string()))?;
        
        debug!("Parsed {} routes from JSON", routes.len());
        
        // Phase 1: Extract gateway from default route
        debug!("Phase 1: Extracting gateway from default route");
        let gateway = extract_gateway(&routes)
            .with_context(|| "Failed to extract gateway IP from routing information")?;
        debug!("Successfully detected gateway: {}", gateway);
        
        // Phase 2: Find host network configuration
        debug!("Phase 2: Finding host network configuration for interface: {}", ifa_name);
        let host_network = extract_host_network(&routes, ifa_name)
            .with_context(|| format!("Failed to extract host network configuration for interface '{}'", ifa_name))?;
        debug!("Successfully detected host network: host_ip={}, cidr={}, prefix={}", 
               host_network.host_ip, host_network.cidr, host_network.prefix);
        
        // Phase 3: Calculate available IPs
        debug!("Phase 3: Calculating available IPs");
        let available_ips = calculate_available_ips(&host_network, &gateway)
            .with_context(|| "Failed to calculate available IP addresses")?;
        debug!("Successfully calculated {} available IPs", available_ips.len());
        
        let net_conf = NetConf {
            gateway_ip: gateway,
            host_ip: host_network.host_ip,
            prefix: host_network.prefix,
            available_ips,
        };
        
        debug!("Network detection completed successfully: gateway={}, host={}, prefix={}, available_ips_count={}", 
               net_conf.gateway_ip, net_conf.host_ip, net_conf.prefix, net_conf.available_ips.len());
        
        Ok(net_conf)
    }

    pub fn override_host_ip(&mut self, value: &str) -> Result<()> {
        self.host_ip =
            IpAddr::from_str(value).with_context(|| format!("invalid host ip '{value}'"))?;
        Ok(())
    }

    pub fn override_gateway_ip(&mut self, value: &str) -> Result<()> {
        self.gateway_ip =
            IpAddr::from_str(value).with_context(|| format!("invalid gateway ip '{value}'"))?;
        Ok(())
    }

    pub fn override_ips(&mut self, value: &str) -> Result<()> {
        self.available_ips.clear();
        for item in value.split(",") {
            if item.contains("/") {
                let cidr = IpCidr::from_str(item)
                    .with_context(|| format!("cannot parse '{item}' as cidr"))?;
                let prefix = get_bits(&cidr);
                let mut ips = cidr.iter();
                if prefix <= 30 {
                    // For routing mask values <= 30, first and last IPs are
                    // base and broadcast addresses and are unusable.
                    ips.next();
                    ips.next_back();
                }
                self.available_ips.append(&mut ips.collect::<Vec<_>>());
            } else if item.contains("-") {
                let mut split = item.split("-");
                let ip_from = split
                    .next()
                    .ok_or(anyhow!("missing from"))
                    .and_then(|ip| IpAddr::from_str(ip).map_err(|err| anyhow!("{err:#}")))
                    .with_context(|| format!("invalid ip range '{item}'"))?;
                let ip_to = split
                    .next()
                    .ok_or(anyhow!("missing to"))
                    .and_then(|ip| IpAddr::from_str(ip).map_err(|err| anyhow!("{err:#}")))
                    .with_context(|| format!("invalid ip range '{item}'"))?;
                if split.next().is_some() {
                    bail!("invalid ip range '{item}'")
                }
                self.available_ips
                    .append(&mut ips_from_range(ip_from, ip_to)?);
            } else {
                self.available_ips.push(
                    IpAddr::from_str(item)
                        .with_context(|| format!("invalid ip provided '{item}'"))?,
                );
            }
        }
        self.available_ips
            .retain(|ip| *ip != self.host_ip && *ip != self.gateway_ip);
        Ok(())
    }
}

/// Internal data structure to hold intermediate host network data
struct HostNetwork {
    host_ip: IpAddr,
    cidr: IpCidr,
    prefix: u8,
}

/// Extract gateway IP from default route
fn extract_gateway(routes: &[IpRoute]) -> Result<IpAddr> {
    // Find default route
    let default_route = routes
        .iter()
        .find(|route| route.dst.as_ref().is_some_and(|v| v == "default"))
        .ok_or_else(|| {
            let available_routes: Vec<String> = routes
                .iter()
                .filter_map(|route| route.dst.as_ref())
                .map(|dst| dst.clone())
                .collect();
            NetworkDetectionError::DefaultRouteNotFound { available_routes }
        })?;
    
    // Extract gateway from default route
    let gateway_str = default_route
        .gateway
        .as_ref()
        .ok_or_else(|| {
            let default_route_info = format!(
                "dst: {:?}, dev: {:?}, protocol: {:?}",
                default_route.dst, default_route.dev, default_route.protocol
            );
            NetworkDetectionError::DefaultRouteGatewayMissing { default_route_info }
        })?;
    
    // Parse gateway IP
    IpAddr::from_str(gateway_str).map_err(|e| {
        NetworkDetectionError::InvalidGatewayIp {
            gateway_str: gateway_str.to_string(),
            parse_error: e.to_string(),
        }
        .into()
    })
}

/// Extract host network configuration from interface routes
fn extract_host_network(routes: &[IpRoute], ifa_name: &str) -> Result<HostNetwork> {
    let mut searched_routes = Vec::new();
    let mut available_interfaces = std::collections::HashSet::new();
    
    for route in routes {
        // Collect available interfaces for debugging
        if let Some(dev) = &route.dev {
            available_interfaces.insert(dev.clone());
        }
        
        if is_dev_ip(route, ifa_name) {
            if let Some(dst) = &route.dst {
                // Parse CIDR
                let cidr = IpCidr::from_str(dst).map_err(|e| {
                    NetworkDetectionError::InvalidCidr {
                        cidr_str: dst.to_string(),
                        parse_error: e.to_string(),
                    }
                })?;
                
                // Extract prefsrc
                let prefsrc = route.prefsrc.as_ref().ok_or_else(|| {
                    let route_info = format!(
                        "dst: {:?}, dev: {:?}, protocol: {:?}",
                        route.dst, route.dev, route.protocol
                    );
                    NetworkDetectionError::InterfacePrefSrcMissing {
                        interface: ifa_name.to_string(),
                        route_info,
                    }
                })?;
                
                // Parse host IP
                let host_ip = IpAddr::from_str(prefsrc).map_err(|e| {
                    NetworkDetectionError::InvalidHostIp {
                        host_ip_str: prefsrc.to_string(),
                        parse_error: e.to_string(),
                    }
                })?;
                
                debug!(
                    "Successfully detected host network: interface={}, host_ip={}, cidr={}, prefix={}",
                    ifa_name, host_ip, cidr, get_bits(&cidr)
                );
                
                return Ok(HostNetwork {
                    host_ip,
                    cidr: cidr.clone(),
                    prefix: get_bits(&cidr),
                });
            }
        } else if let Some(dev) = &route.dev {
            // Track routes we searched for debugging
            if dev == ifa_name {
                searched_routes.push(format!(
                    "dst: {:?}, prefsrc: {:?}, protocol: {:?}",
                    route.dst, route.prefsrc, route.protocol
                ));
            }
        }
    }
    
    let available_interfaces: Vec<String> = available_interfaces.into_iter().collect();
    Err(NetworkDetectionError::InterfaceConfigNotFound {
        interface: ifa_name.to_string(),
        available_interfaces,
        searched_routes,
    }
    .into())
}

/// Calculate available IPs from host network, excluding host and gateway IPs
fn calculate_available_ips(host_network: &HostNetwork, gateway: &IpAddr) -> Result<Vec<IpAddr>> {
    let mut ips = host_network.cidr.iter();
    
    if host_network.prefix <= 30 {
        // For routing mask values <= 30, first and last IPs are
        // base and broadcast addresses and are unusable.
        ips.next();
        ips.next_back();
    }
    
    if let (Some(ip_from), Some(ip_to)) = (ips.next(), ips.next_back()) {
        let mut available_ips = ips_from_range(ip_from, ip_to)?;
        let original_count = available_ips.len();
        
        // Remove both host IP and gateway IP from available pool
        available_ips.retain(|ip| *ip != host_network.host_ip && *ip != *gateway);
        
        debug!(
            "Calculated available IPs: cidr={}, prefix={}, range={}..{}, total_before_exclusion={}, excluded_host={}, excluded_gateway={}, final_count={}",
            host_network.cidr, host_network.prefix, ip_from, ip_to, original_count, host_network.host_ip, gateway, available_ips.len()
        );
        
        Ok(available_ips)
    } else {
        // Handle edge cases like /32 and /31 networks where there are no available IPs for allocation
        // This is valid for host-network mode where BlockVisor doesn't need to allocate additional IPs
        debug!(
            "No IP range available for allocation: cidr={}, prefix={}, host_ip={}, gateway={}. Operating in host-network mode.",
            host_network.cidr, host_network.prefix, host_network.host_ip, gateway
        );
        
        Ok(Vec::new())
    }
}

fn ips_from_range(ip_from: IpAddr, ip_to: IpAddr) -> Result<Vec<IpAddr>> {
    Ok(match (ip_from, ip_to) {
        (IpAddr::V4(ip_from), IpAddr::V4(ip_to)) => Ok(ipnet::IpAddrRange::from(
            ipnet::Ipv4AddrRange::new(ip_from, ip_to),
        )),
        (IpAddr::V6(ip_from), IpAddr::V6(ip_to)) => Ok(ipnet::IpAddrRange::from(
            ipnet::Ipv6AddrRange::new(ip_from, ip_to),
        )),
        _ => Err(anyhow!("invalid ip range: ({ip_from}, {ip_to}")),
    }?
    .collect())
}

/// Struct to capture output of linux `ip --json route` command
#[derive(Deserialize, Serialize, Debug)]
struct IpRoute {
    pub dst: Option<String>,
    pub gateway: Option<String>,
    pub dev: Option<String>,
    pub prefsrc: Option<String>,
    pub protocol: Option<String>,
}

fn is_dev_ip(route: &IpRoute, dev: &str) -> bool {
    route.dev.as_deref() == Some(dev)
        && route
            .dst
            .as_ref()
            .is_some_and(|v| v != "default" && IpCidr::is_ip_cidr(v))
        && route.prefsrc.is_some()
        && route.protocol.as_deref() == Some("kernel")
}

fn get_bits(cidr: &IpCidr) -> u8 {
    match cidr {
        IpCidr::V4(cidr) => cidr.get_bits(),
        IpCidr::V6(cidr) => cidr.get_bits(),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_net_conf_from_json() {
        let json = r#"[
            {
               "dev" : "br0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "192.69.220.81",
               "protocol" : "static"
            },
            {
               "dev" : "br0",
               "dst" : "192.69.220.80/28",
               "flags" : [],
               "prefsrc" : "192.69.220.82",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]
         "#;
        let expected = NetConf {
            gateway_ip: IpAddr::from(Ipv4Addr::from_str("192.69.220.81").unwrap()),
            host_ip: IpAddr::from(Ipv4Addr::from_str("192.69.220.82").unwrap()),
            prefix: 28,
            available_ips: vec![
                IpAddr::from(Ipv4Addr::from_str("192.69.220.83").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.84").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.85").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.86").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.87").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.88").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.89").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.90").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.91").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.92").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.93").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.94").unwrap()),
            ],
        };
        assert_eq!(expected, NetConf::from_json(json, "br0").unwrap());
    }

    #[test]
    fn test_improved_error_messages_no_default_route() {
        let json = r#"[
            {
               "dev" : "br0",
               "dst" : "192.69.220.80/28",
               "flags" : [],
               "prefsrc" : "192.69.220.82",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        assert!(error_chain.contains("default route not found"));
        assert!(error_chain.contains("Available routes: [192.69.220.80/28]"));
    }

    #[test]
    fn test_improved_error_messages_missing_gateway() {
        let json = r#"[
            {
               "dev" : "br0",
               "dst" : "default",
               "flags" : [],
               "protocol" : "static"
            },
            {
               "dev" : "br0",
               "dst" : "192.69.220.80/28",
               "flags" : [],
               "prefsrc" : "192.69.220.82",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        assert!(error_chain.contains("default route 'gateway' field not found"));
        assert!(error_chain.contains("Route info:"));
        assert!(error_chain.contains("dst: Some(\"default\")"));
    }

    #[test]
    fn test_improved_error_messages_invalid_gateway_ip() {
        let json = r#"[
            {
               "dev" : "br0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "invalid-ip",
               "protocol" : "static"
            },
            {
               "dev" : "br0",
               "dst" : "192.69.220.80/28",
               "flags" : [],
               "prefsrc" : "192.69.220.82",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        assert!(error_chain.contains("failed to parse gateway ip 'invalid-ip'"));
    }

    #[test]
    fn test_improved_error_messages_interface_not_found() {
        let json = r#"[
            {
               "dev" : "br0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "192.69.220.81",
               "protocol" : "static"
            },
            {
               "dev" : "br0",
               "dst" : "192.69.220.80/28",
               "flags" : [],
               "prefsrc" : "192.69.220.82",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "eth0"); // Wrong interface
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        assert!(error_chain.contains("failed to find network configuration for interface 'eth0'"));
        assert!(error_chain.contains("Available interfaces: [br0]"));
    }

    #[test]
    fn test_improved_error_messages_interface_not_found_detailed() {
        let json = r#"[
            {
               "dev" : "br0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "192.69.220.81",
               "protocol" : "static"
            },
            {
               "dev" : "br0",
               "dst" : "192.69.220.80/28",
               "flags" : [],
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        // This should show detailed information about what routes were searched
        assert!(error_chain.contains("failed to find network configuration for interface 'br0'"));
        assert!(error_chain.contains("Available interfaces: [br0]"));
        assert!(error_chain.contains("Searched routes:"));
    }

    // ========== Gateway Outside Subnet Test Cases ==========

    #[test]
    fn test_gateway_outside_subnet_basic() {
        // Test case: Gateway in 10.0.0.0/24, host in 192.168.1.0/24
        let json = r#"[
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
        
        let result = NetConf::from_json(json, "br0").unwrap();
        
        assert_eq!(result.gateway_ip, IpAddr::from(Ipv4Addr::from_str("10.0.0.1").unwrap()));
        assert_eq!(result.host_ip, IpAddr::from(Ipv4Addr::from_str("192.168.1.100").unwrap()));
        assert_eq!(result.prefix, 24);
        
        // Verify available IPs exclude both host and gateway
        assert!(!result.available_ips.contains(&result.host_ip));
        assert!(!result.available_ips.contains(&result.gateway_ip));
        
        // Should have 253 available IPs (256 - network - broadcast - host)
        assert_eq!(result.available_ips.len(), 253);
        
        // Verify range starts from .1 and excludes .100 (host)
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.1.1").unwrap())));
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.1.99").unwrap())));
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.1.101").unwrap())));
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.1.254").unwrap())));
    }

    #[test]
    fn test_gateway_outside_subnet_cloud_environment() {
        // Test case: Simulating AWS/GCP where gateway is in different subnet
        // Gateway: 172.31.0.1, Host: 172.31.16.0/20 (172.31.16.5)
        let json = r#"[
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
        
        let result = NetConf::from_json(json, "eth0").unwrap();
        
        assert_eq!(result.gateway_ip, IpAddr::from(Ipv4Addr::from_str("172.31.0.1").unwrap()));
        assert_eq!(result.host_ip, IpAddr::from(Ipv4Addr::from_str("172.31.16.5").unwrap()));
        assert_eq!(result.prefix, 20);
        
        // Verify available IPs exclude both host and gateway
        assert!(!result.available_ips.contains(&result.host_ip));
        assert!(!result.available_ips.contains(&result.gateway_ip));
        
        // /20 subnet has 4094 usable IPs (4096 - network - broadcast)
        // Minus 1 for host = 4093 available IPs
        assert_eq!(result.available_ips.len(), 4093);
        
        // Verify range boundaries
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("172.31.16.1").unwrap())));
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("172.31.31.254").unwrap())));
        assert!(!result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("172.31.16.0").unwrap()))); // network
        assert!(!result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("172.31.31.255").unwrap()))); // broadcast
    }

    #[test]
    fn test_gateway_outside_subnet_docker_network() {
        // Test case: Docker bridge network scenario
        // Gateway: 172.17.0.1, Host: 172.18.0.0/16 (172.18.0.2)
        let json = r#"[
            {
               "dev" : "docker0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "172.17.0.1",
               "protocol" : "static"
            },
            {
               "dev" : "docker0",
               "dst" : "172.18.0.0/16",
               "flags" : [],
               "prefsrc" : "172.18.0.2",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "docker0").unwrap();
        
        assert_eq!(result.gateway_ip, IpAddr::from(Ipv4Addr::from_str("172.17.0.1").unwrap()));
        assert_eq!(result.host_ip, IpAddr::from(Ipv4Addr::from_str("172.18.0.2").unwrap()));
        assert_eq!(result.prefix, 16);
        
        // Verify available IPs exclude both host and gateway
        assert!(!result.available_ips.contains(&result.host_ip));
        assert!(!result.available_ips.contains(&result.gateway_ip));
        
        // /16 subnet has 65534 usable IPs (65536 - network - broadcast)
        // Minus 1 for host = 65533 available IPs
        assert_eq!(result.available_ips.len(), 65533);
    }

    #[test]
    fn test_gateway_outside_subnet_small_subnet() {
        // Test case: Small subnet /28 with gateway outside
        // Gateway: 10.1.1.1, Host: 192.168.100.16/28 (192.168.100.20)
        let json = r#"[
            {
               "dev" : "br0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "10.1.1.1",
               "protocol" : "static"
            },
            {
               "dev" : "br0",
               "dst" : "192.168.100.16/28",
               "flags" : [],
               "prefsrc" : "192.168.100.20",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0").unwrap();
        
        assert_eq!(result.gateway_ip, IpAddr::from(Ipv4Addr::from_str("10.1.1.1").unwrap()));
        assert_eq!(result.host_ip, IpAddr::from(Ipv4Addr::from_str("192.168.100.20").unwrap()));
        assert_eq!(result.prefix, 28);
        
        // Verify available IPs exclude both host and gateway
        assert!(!result.available_ips.contains(&result.host_ip));
        assert!(!result.available_ips.contains(&result.gateway_ip));
        
        // /28 subnet has 14 usable IPs (16 - network - broadcast)
        // Minus 1 for host = 13 available IPs
        assert_eq!(result.available_ips.len(), 13);
        
        // Verify specific IPs in the /28 range
        let expected_ips = vec![
            "192.168.100.17", "192.168.100.18", "192.168.100.19", // before host
            "192.168.100.21", "192.168.100.22", "192.168.100.23", // after host
            "192.168.100.24", "192.168.100.25", "192.168.100.26",
            "192.168.100.27", "192.168.100.28", "192.168.100.29", "192.168.100.30"
        ];
        
        for ip_str in expected_ips {
            let ip = IpAddr::from(Ipv4Addr::from_str(ip_str).unwrap());
            assert!(result.available_ips.contains(&ip), "Missing IP: {}", ip);
        }
    }

    #[test]
    fn test_gateway_outside_subnet_multiple_interfaces() {
        // Test case: Multiple interfaces, gateway outside target interface subnet
        let json = r#"[
            {
               "dev" : "eth0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "203.0.113.1",
               "protocol" : "static"
            },
            {
               "dev" : "eth0",
               "dst" : "203.0.113.0/24",
               "flags" : [],
               "prefsrc" : "203.0.113.10",
               "protocol" : "kernel",
               "scope" : "link"
            },
            {
               "dev" : "br0",
               "dst" : "10.0.0.0/8",
               "flags" : [],
               "prefsrc" : "10.0.1.50",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0").unwrap();
        
        // Gateway should be from default route (203.0.113.1)
        // Host should be from br0 interface (10.0.1.50)
        assert_eq!(result.gateway_ip, IpAddr::from(Ipv4Addr::from_str("203.0.113.1").unwrap()));
        assert_eq!(result.host_ip, IpAddr::from(Ipv4Addr::from_str("10.0.1.50").unwrap()));
        assert_eq!(result.prefix, 8);
        
        // Verify available IPs exclude both host and gateway
        assert!(!result.available_ips.contains(&result.host_ip));
        assert!(!result.available_ips.contains(&result.gateway_ip));
        
        // /8 subnet has 16777214 usable IPs (16777216 - network - broadcast)
        // Minus 1 for host = 16777213 available IPs
        assert_eq!(result.available_ips.len(), 16777213);
    }

    #[test]
    fn test_gateway_outside_subnet_ipv6_compatible() {
        // Test case: IPv4 with gateway far outside subnet range
        // Gateway: 8.8.8.8 (Google DNS), Host: 192.168.0.0/24
        let json = r#"[
            {
               "dev" : "wlan0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "8.8.8.8",
               "protocol" : "dhcp"
            },
            {
               "dev" : "wlan0",
               "dst" : "192.168.0.0/24",
               "flags" : [],
               "prefsrc" : "192.168.0.150",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "wlan0").unwrap();
        
        assert_eq!(result.gateway_ip, IpAddr::from(Ipv4Addr::from_str("8.8.8.8").unwrap()));
        assert_eq!(result.host_ip, IpAddr::from(Ipv4Addr::from_str("192.168.0.150").unwrap()));
        assert_eq!(result.prefix, 24);
        
        // Verify available IPs exclude both host and gateway
        assert!(!result.available_ips.contains(&result.host_ip));
        assert!(!result.available_ips.contains(&result.gateway_ip));
        
        // /24 subnet has 254 usable IPs (256 - network - broadcast)
        // Minus 1 for host = 253 available IPs
        assert_eq!(result.available_ips.len(), 253);
    }

    #[test]
    fn test_gateway_outside_subnet_edge_case_adjacent_subnets() {
        // Test case: Gateway in adjacent subnet
        // Gateway: 192.168.1.1 (/24), Host: 192.168.2.0/24 (192.168.2.10)
        let json = r#"[
            {
               "dev" : "br0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "192.168.1.1",
               "protocol" : "static"
            },
            {
               "dev" : "br0",
               "dst" : "192.168.2.0/24",
               "flags" : [],
               "prefsrc" : "192.168.2.10",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0").unwrap();
        
        assert_eq!(result.gateway_ip, IpAddr::from(Ipv4Addr::from_str("192.168.1.1").unwrap()));
        assert_eq!(result.host_ip, IpAddr::from(Ipv4Addr::from_str("192.168.2.10").unwrap()));
        assert_eq!(result.prefix, 24);
        
        // Verify available IPs exclude both host and gateway
        assert!(!result.available_ips.contains(&result.host_ip));
        assert!(!result.available_ips.contains(&result.gateway_ip));
        
        // Should have 253 available IPs in 192.168.2.0/24 range
        assert_eq!(result.available_ips.len(), 253);
        
        // Verify gateway is not in available IPs even though it's in adjacent subnet
        assert!(!result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.1.1").unwrap())));
        
        // Verify some expected IPs are available
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.2.1").unwrap())));
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.2.9").unwrap())));
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.2.11").unwrap())));
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.2.254").unwrap())));
    }

    #[test]
    fn test_gateway_outside_subnet_complex_routing() {
        // Test case: Complex routing scenario with multiple routes
        let json = r#"[
            {
               "dev" : "eth0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "10.0.0.1",
               "protocol" : "static"
            },
            {
               "dev" : "eth0",
               "dst" : "10.0.0.0/24",
               "flags" : [],
               "prefsrc" : "10.0.0.100",
               "protocol" : "kernel",
               "scope" : "link"
            },
            {
               "dev" : "br0",
               "dst" : "172.16.0.0/12",
               "flags" : [],
               "prefsrc" : "172.16.1.1",
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
            }
         ]"#;
        
        // Test with br0 interface - gateway should be from default route
        let result = NetConf::from_json(json, "br0").unwrap();
        
        assert_eq!(result.gateway_ip, IpAddr::from(Ipv4Addr::from_str("10.0.0.1").unwrap()));
        assert_eq!(result.host_ip, IpAddr::from(Ipv4Addr::from_str("172.16.1.1").unwrap()));
        assert_eq!(result.prefix, 12);
        
        // Verify available IPs exclude both host and gateway
        assert!(!result.available_ips.contains(&result.host_ip));
        assert!(!result.available_ips.contains(&result.gateway_ip));
        
        // /12 subnet has 1048574 usable IPs (1048576 - network - broadcast)
        // Minus 1 for host = 1048573 available IPs
        assert_eq!(result.available_ips.len(), 1048573);
    }

    #[test]
    fn test_gateway_outside_subnet_32_network() {
        // Test case: /32 network with gateway outside (common in sliced networks)
        // Example: Admin slices /24 into individual /32 subnets with shared gateway
        // Gateway: 10.0.0.1, Host: 10.0.0.100/32
        let json = r#"[
            {
               "dev" : "eth0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "10.0.0.1",
               "protocol" : "static"
            },
            {
               "dev" : "eth0",
               "dst" : "10.0.0.100/32",
               "flags" : [],
               "prefsrc" : "10.0.0.100",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "eth0").unwrap();
        
        assert_eq!(result.gateway_ip, IpAddr::from(Ipv4Addr::from_str("10.0.0.1").unwrap()));
        assert_eq!(result.host_ip, IpAddr::from(Ipv4Addr::from_str("10.0.0.100").unwrap()));
        assert_eq!(result.prefix, 32);
        
        // /32 network has no available IPs for allocation - operates in host-network mode
        assert_eq!(result.available_ips.len(), 0);
        
        // Verify both host and gateway are excluded from available pool (even though pool is empty)
        assert!(!result.available_ips.contains(&result.host_ip));
        assert!(!result.available_ips.contains(&result.gateway_ip));
    }

    // ========== Backward Compatibility Test Cases ==========

    #[test]
    fn test_backward_compatibility_gateway_inside_subnet_basic() {
        // Test case: Standard configuration where gateway is inside host subnet
        // This is the original working behavior that should be preserved
        let json = r#"[
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
        
        let result = NetConf::from_json(json, "br0").unwrap();
        
        assert_eq!(result.gateway_ip, IpAddr::from(Ipv4Addr::from_str("192.168.1.1").unwrap()));
        assert_eq!(result.host_ip, IpAddr::from(Ipv4Addr::from_str("192.168.1.100").unwrap()));
        assert_eq!(result.prefix, 24);
        
        // Verify available IPs exclude both host and gateway
        assert!(!result.available_ips.contains(&result.host_ip));
        assert!(!result.available_ips.contains(&result.gateway_ip));
        
        // Should have 252 available IPs (254 usable - host - gateway)
        assert_eq!(result.available_ips.len(), 252);
        
        // Verify specific IPs are available/excluded
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.1.2").unwrap())));
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.1.99").unwrap())));
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.1.101").unwrap())));
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.1.254").unwrap())));
        
        // Verify excluded IPs
        assert!(!result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.1.0").unwrap()))); // network
        assert!(!result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.1.1").unwrap()))); // gateway
        assert!(!result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.1.100").unwrap()))); // host
        assert!(!result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.1.255").unwrap()))); // broadcast
    }

    #[test]
    fn test_backward_compatibility_gateway_inside_subnet_small_network() {
        // Test case: Small /28 network with gateway inside (original test case format)
        let json = r#"[
            {
               "dev" : "br0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "192.69.220.81",
               "protocol" : "static"
            },
            {
               "dev" : "br0",
               "dst" : "192.69.220.80/28",
               "flags" : [],
               "prefsrc" : "192.69.220.82",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let expected = NetConf {
            gateway_ip: IpAddr::from(Ipv4Addr::from_str("192.69.220.81").unwrap()),
            host_ip: IpAddr::from(Ipv4Addr::from_str("192.69.220.82").unwrap()),
            prefix: 28,
            available_ips: vec![
                IpAddr::from(Ipv4Addr::from_str("192.69.220.83").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.84").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.85").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.86").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.87").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.88").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.89").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.90").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.91").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.92").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.93").unwrap()),
                IpAddr::from(Ipv4Addr::from_str("192.69.220.94").unwrap()),
            ],
        };
        
        let result = NetConf::from_json(json, "br0").unwrap();
        assert_eq!(expected, result);
        
        // Additional verification that this matches the original test exactly
        assert_eq!(result.available_ips.len(), 12); // 14 usable - host - gateway
        assert!(!result.available_ips.contains(&result.host_ip));
        assert!(!result.available_ips.contains(&result.gateway_ip));
    }

    #[test]
    fn test_backward_compatibility_gateway_inside_subnet_dhcp() {
        // Test case: DHCP configuration with gateway inside subnet
        let json = r#"[
            {
               "dev" : "eth0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "10.0.0.1",
               "protocol" : "dhcp"
            },
            {
               "dev" : "eth0",
               "dst" : "10.0.0.0/24",
               "flags" : [],
               "prefsrc" : "10.0.0.50",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "eth0").unwrap();
        
        assert_eq!(result.gateway_ip, IpAddr::from(Ipv4Addr::from_str("10.0.0.1").unwrap()));
        assert_eq!(result.host_ip, IpAddr::from(Ipv4Addr::from_str("10.0.0.50").unwrap()));
        assert_eq!(result.prefix, 24);
        
        // Should have 252 available IPs (254 usable - host - gateway)
        assert_eq!(result.available_ips.len(), 252);
        
        // Verify available IPs exclude both host and gateway
        assert!(!result.available_ips.contains(&result.host_ip));
        assert!(!result.available_ips.contains(&result.gateway_ip));
        
        // Verify expected range
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("10.0.0.2").unwrap())));
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("10.0.0.49").unwrap())));
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("10.0.0.51").unwrap())));
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("10.0.0.254").unwrap())));
    }

    #[test]
    fn test_backward_compatibility_gateway_inside_subnet_large_network() {
        // Test case: Large /16 network with gateway inside subnet
        let json = r#"[
            {
               "dev" : "br0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "172.16.0.1",
               "protocol" : "static"
            },
            {
               "dev" : "br0",
               "dst" : "172.16.0.0/16",
               "flags" : [],
               "prefsrc" : "172.16.1.100",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0").unwrap();
        
        assert_eq!(result.gateway_ip, IpAddr::from(Ipv4Addr::from_str("172.16.0.1").unwrap()));
        assert_eq!(result.host_ip, IpAddr::from(Ipv4Addr::from_str("172.16.1.100").unwrap()));
        assert_eq!(result.prefix, 16);
        
        // Should have 65532 available IPs (65534 usable - host - gateway)
        assert_eq!(result.available_ips.len(), 65532);
        
        // Verify available IPs exclude both host and gateway
        assert!(!result.available_ips.contains(&result.host_ip));
        assert!(!result.available_ips.contains(&result.gateway_ip));
        
        // Verify some expected IPs
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("172.16.0.2").unwrap())));
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("172.16.1.99").unwrap())));
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("172.16.1.101").unwrap())));
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("172.16.255.254").unwrap())));
    }

    #[test]
    fn test_backward_compatibility_gateway_inside_subnet_edge_cases() {
        // Test case: Gateway at first usable IP in subnet
        let json = r#"[
            {
               "dev" : "br0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "192.168.100.1",
               "protocol" : "static"
            },
            {
               "dev" : "br0",
               "dst" : "192.168.100.0/24",
               "flags" : [],
               "prefsrc" : "192.168.100.2",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0").unwrap();
        
        assert_eq!(result.gateway_ip, IpAddr::from(Ipv4Addr::from_str("192.168.100.1").unwrap()));
        assert_eq!(result.host_ip, IpAddr::from(Ipv4Addr::from_str("192.168.100.2").unwrap()));
        assert_eq!(result.prefix, 24);
        
        // Should have 252 available IPs (254 usable - host - gateway)
        assert_eq!(result.available_ips.len(), 252);
        
        // Verify available IPs exclude both host and gateway
        assert!(!result.available_ips.contains(&result.host_ip));
        assert!(!result.available_ips.contains(&result.gateway_ip));
        
        // Verify range starts from .3 since .1 (gateway) and .2 (host) are excluded
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.100.3").unwrap())));
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.100.254").unwrap())));
    }

    #[test]
    fn test_backward_compatibility_gateway_inside_subnet_host_at_end() {
        // Test case: Host at last usable IP in subnet
        let json = r#"[
            {
               "dev" : "br0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "192.168.200.1",
               "protocol" : "static"
            },
            {
               "dev" : "br0",
               "dst" : "192.168.200.0/24",
               "flags" : [],
               "prefsrc" : "192.168.200.254",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0").unwrap();
        
        assert_eq!(result.gateway_ip, IpAddr::from(Ipv4Addr::from_str("192.168.200.1").unwrap()));
        assert_eq!(result.host_ip, IpAddr::from(Ipv4Addr::from_str("192.168.200.254").unwrap()));
        assert_eq!(result.prefix, 24);
        
        // Should have 252 available IPs (254 usable - host - gateway)
        assert_eq!(result.available_ips.len(), 252);
        
        // Verify available IPs exclude both host and gateway
        assert!(!result.available_ips.contains(&result.host_ip));
        assert!(!result.available_ips.contains(&result.gateway_ip));
        
        // Verify range goes from .2 to .253 (excluding .1 gateway and .254 host)
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.200.2").unwrap())));
        assert!(result.available_ips.contains(&IpAddr::from(Ipv4Addr::from_str("192.168.200.253").unwrap())));
    }

    #[test]
    fn test_backward_compatibility_gateway_inside_subnet_very_small_network() {
        // Test case: /30 network (only 2 usable IPs) with gateway inside
        let json = r#"[
            {
               "dev" : "ptp0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "10.0.0.1",
               "protocol" : "static"
            },
            {
               "dev" : "ptp0",
               "dst" : "10.0.0.0/30",
               "flags" : [],
               "prefsrc" : "10.0.0.2",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "ptp0").unwrap();
        
        assert_eq!(result.gateway_ip, IpAddr::from(Ipv4Addr::from_str("10.0.0.1").unwrap()));
        assert_eq!(result.host_ip, IpAddr::from(Ipv4Addr::from_str("10.0.0.2").unwrap()));
        assert_eq!(result.prefix, 30);
        
        // /30 has 4 IPs total: .0 (network), .1 (gateway), .2 (host), .3 (broadcast)
        // No available IPs left after excluding network, broadcast, gateway, and host
        assert_eq!(result.available_ips.len(), 0);
        
        // Verify available IPs exclude both host and gateway
        assert!(!result.available_ips.contains(&result.host_ip));
        assert!(!result.available_ips.contains(&result.gateway_ip));
    }

    #[test]
    fn test_backward_compatibility_gateway_inside_subnet_31_network() {
        // Test case: /31 network (point-to-point, no network/broadcast addresses)
        let json = r#"[
            {
               "dev" : "ptp0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "10.0.0.0",
               "protocol" : "static"
            },
            {
               "dev" : "ptp0",
               "dst" : "10.0.0.0/31",
               "flags" : [],
               "prefsrc" : "10.0.0.1",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "ptp0").unwrap();
        
        assert_eq!(result.gateway_ip, IpAddr::from(Ipv4Addr::from_str("10.0.0.0").unwrap()));
        assert_eq!(result.host_ip, IpAddr::from(Ipv4Addr::from_str("10.0.0.1").unwrap()));
        assert_eq!(result.prefix, 31);
        
        // /31 has 2 IPs total, both used by gateway and host
        // No available IPs left
        assert_eq!(result.available_ips.len(), 0);
        
        // Verify available IPs exclude both host and gateway
        assert!(!result.available_ips.contains(&result.host_ip));
        assert!(!result.available_ips.contains(&result.gateway_ip));
    }

    #[test]
    fn test_backward_compatibility_gateway_inside_subnet_32_network_same_ip() {
        // Test case: /32 network where gateway and host are the same IP (loopback scenario)
        let json = r#"[
            {
               "dev" : "lo",
               "dst" : "default",
               "flags" : [],
               "gateway" : "127.0.0.1",
               "protocol" : "static"
            },
            {
               "dev" : "lo",
               "dst" : "127.0.0.1/32",
               "flags" : [],
               "prefsrc" : "127.0.0.1",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "lo").unwrap();
        
        assert_eq!(result.gateway_ip, IpAddr::from(Ipv4Addr::from_str("127.0.0.1").unwrap()));
        assert_eq!(result.host_ip, IpAddr::from(Ipv4Addr::from_str("127.0.0.1").unwrap()));
        assert_eq!(result.prefix, 32);
        
        // /32 has only 1 IP, used by both gateway and host (same IP)
        // No available IPs for allocation - operates in host-network mode
        assert_eq!(result.available_ips.len(), 0);
        
        // Verify the single IP is excluded from available pool
        assert!(!result.available_ips.contains(&result.host_ip));
        assert!(!result.available_ips.contains(&result.gateway_ip));
    }

    #[test]
    fn test_backward_compatibility_gateway_outside_subnet_32_network() {
        // Test case: /32 network with gateway outside (valid scenario)
        // Example: Admin slices /24 network into individual /32 subnets
        // Gateway: 192.168.1.1, Host: 192.168.1.35/32
        let json = r#"[
            {
               "dev" : "eth0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "192.168.1.1",
               "protocol" : "static"
            },
            {
               "dev" : "eth0",
               "dst" : "192.168.1.35/32",
               "flags" : [],
               "prefsrc" : "192.168.1.35",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "eth0").unwrap();
        
        assert_eq!(result.gateway_ip, IpAddr::from(Ipv4Addr::from_str("192.168.1.1").unwrap()));
        assert_eq!(result.host_ip, IpAddr::from(Ipv4Addr::from_str("192.168.1.35").unwrap()));
        assert_eq!(result.prefix, 32);
        
        // /32 has only 1 IP for the host, no IPs available for allocation
        // BlockVisor operates in host-network mode
        assert_eq!(result.available_ips.len(), 0);
        
        // Verify no IPs are available for allocation
        assert!(!result.available_ips.contains(&result.host_ip));
        assert!(!result.available_ips.contains(&result.gateway_ip));
    }

    #[test]
    fn test_backward_compatibility_multiple_interfaces_gateway_inside() {
        // Test case: Multiple interfaces with gateway inside one of them
        let json = r#"[
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
               "prefsrc" : "192.168.1.50",
               "protocol" : "kernel",
               "scope" : "link"
            },
            {
               "dev" : "br0",
               "dst" : "10.0.0.0/8",
               "flags" : [],
               "prefsrc" : "10.0.1.100",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        // Test with eth0 (gateway inside this subnet)
        let result_eth0 = NetConf::from_json(json, "eth0").unwrap();
        
        assert_eq!(result_eth0.gateway_ip, IpAddr::from(Ipv4Addr::from_str("192.168.1.1").unwrap()));
        assert_eq!(result_eth0.host_ip, IpAddr::from(Ipv4Addr::from_str("192.168.1.50").unwrap()));
        assert_eq!(result_eth0.prefix, 24);
        assert_eq!(result_eth0.available_ips.len(), 252); // 254 - host - gateway
        
        // Test with br0 (gateway outside this subnet)
        let result_br0 = NetConf::from_json(json, "br0").unwrap();
        
        assert_eq!(result_br0.gateway_ip, IpAddr::from(Ipv4Addr::from_str("192.168.1.1").unwrap()));
        assert_eq!(result_br0.host_ip, IpAddr::from(Ipv4Addr::from_str("10.0.1.100").unwrap()));
        assert_eq!(result_br0.prefix, 8);
        assert_eq!(result_br0.available_ips.len(), 16777213); // 16777214 - host (gateway not in this subnet)
        
        // Verify both exclude their respective host IPs
        assert!(!result_eth0.available_ips.contains(&result_eth0.host_ip));
        assert!(!result_eth0.available_ips.contains(&result_eth0.gateway_ip));
        assert!(!result_br0.available_ips.contains(&result_br0.host_ip));
        assert!(!result_br0.available_ips.contains(&result_br0.gateway_ip));
    }

    // ========== Additional Error Handling Test Cases ==========

    #[test]
    fn test_error_handling_missing_prefsrc_field() {
        // Test case: Interface route exists but missing prefsrc field
        // Note: Due to is_dev_ip filtering, missing prefsrc results in "interface not found"
        // rather than a specific "prefsrc missing" error
        let json = r#"[
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
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        // The route doesn't match is_dev_ip criteria (missing prefsrc)
        // So the error will be "failed to find network configuration for interface 'br0'"
        assert!(error_chain.contains("failed to find network configuration for interface 'br0'"));
        assert!(error_chain.contains("Available interfaces: [br0]"));
        assert!(error_chain.contains("Searched routes:"));
        assert!(error_chain.contains("prefsrc: None"));
    }

    #[test]
    fn test_error_handling_invalid_cidr_format() {
        // Test case: Invalid CIDR format that passes is_ip_cidr but fails actual parsing
        // Note: This test demonstrates that invalid CIDRs result in "interface not found" 
        // because is_dev_ip filters them out before they reach the CIDR parsing stage
        let json = r#"[
            {
               "dev" : "br0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "192.168.1.1",
               "protocol" : "static"
            },
            {
               "dev" : "br0",
               "dst" : "192.168.1.0/invalid",
               "flags" : [],
               "prefsrc" : "192.168.1.100",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        // The invalid CIDR will cause is_dev_ip to return false (IpCidr::is_ip_cidr fails)
        // So this results in "interface not found" error rather than specific CIDR error
        assert!(error_chain.contains("failed to find network configuration for interface 'br0'"));
        assert!(error_chain.contains("Available interfaces: [br0]"));
    }

    #[test]
    fn test_error_handling_invalid_host_ip_format() {
        // Test case: Invalid IP format in prefsrc field
        let json = r#"[
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
               "prefsrc" : "invalid-host-ip",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        assert!(error_chain.contains("cannot parse host ip 'invalid-host-ip'"));
    }

    #[test]
    fn test_error_handling_malformed_json() {
        // Test case: Malformed JSON input
        let json = r#"[
            {
               "dev" : "br0",
               "dst" : "default",
               "gateway" : "192.168.1.1"
               // Missing comma and closing brace
         ]"#;
        
        let result = NetConf::from_json(json, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        assert!(error_chain.contains("JSON parsing error"));
    }

    #[test]
    fn test_error_handling_empty_routes_array() {
        // Test case: Empty routes array
        let json = r#"[]"#;
        
        let result = NetConf::from_json(json, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        assert!(error_chain.contains("default route not found"));
        assert!(error_chain.contains("Available routes: []"));
    }

    #[test]
    fn test_error_handling_no_matching_interface_routes() {
        // Test case: Default route exists but no matching interface routes
        let json = r#"[
            {
               "dev" : "eth0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "192.168.1.1",
               "protocol" : "static"
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
        
        let result = NetConf::from_json(json, "br0"); // Wrong interface
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        assert!(error_chain.contains("failed to find network configuration for interface 'br0'"));
        assert!(error_chain.contains("Available interfaces: [eth0]"));
    }

    #[test]
    fn test_error_handling_interface_route_wrong_protocol() {
        // Test case: Interface route exists but wrong protocol (not "kernel")
        let json = r#"[
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
               "protocol" : "static",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        assert!(error_chain.contains("failed to find network configuration for interface 'br0'"));
        assert!(error_chain.contains("Available interfaces: [br0]"));
        assert!(error_chain.contains("Searched routes:"));
        assert!(error_chain.contains("protocol: Some(\"static\")"));
    }

    #[test]
    fn test_error_handling_interface_route_missing_dst() {
        // Test case: Interface route missing dst field
        let json = r#"[
            {
               "dev" : "br0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "192.168.1.1",
               "protocol" : "static"
            },
            {
               "dev" : "br0",
               "flags" : [],
               "prefsrc" : "192.168.1.100",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        assert!(error_chain.contains("failed to find network configuration for interface 'br0'"));
        assert!(error_chain.contains("Available interfaces: [br0]"));
        assert!(error_chain.contains("Searched routes:"));
        assert!(error_chain.contains("dst: None"));
    }

    #[test]
    fn test_error_handling_multiple_error_scenarios() {
        // Test case: Multiple potential error scenarios in one JSON
        let json = r#"[
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
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        // Should fail on missing default route first
        let result = NetConf::from_json(json, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        assert!(error_chain.contains("default route not found"));
        assert!(error_chain.contains("Available routes: [192.168.1.0/24, 10.0.0.0/8]"));
    }

    #[test]
    fn test_error_handling_invalid_cidr_range() {
        // Test case: CIDR that results in invalid IP range calculation
        let json = r#"[
            {
               "dev" : "br0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "192.168.1.1",
               "protocol" : "static"
            },
            {
               "dev" : "br0",
               "dst" : "192.168.1.255/32",
               "flags" : [],
               "prefsrc" : "192.168.1.255",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0");
        // This should succeed as /32 is valid, just results in 0 available IPs
        assert!(result.is_ok());
        let net_conf = result.unwrap();
        assert_eq!(net_conf.available_ips.len(), 0);
        assert_eq!(net_conf.prefix, 32);
    }

    #[test]
    fn test_error_handling_context_preservation() {
        // Test case: Verify error context is preserved through the chain
        let json = r#"[
            {
               "dev" : "br0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "not-an-ip-address",
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
        
        let result = NetConf::from_json(json, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        
        // Should contain both the high-level context and the specific error
        assert!(error_chain.contains("Failed to extract gateway IP from routing information"));
        assert!(error_chain.contains("failed to parse gateway ip 'not-an-ip-address'"));
    }

    #[test]
    fn test_error_handling_detailed_interface_search_info() {
        // Test case: Verify detailed interface search information in errors
        let json = r#"[
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
               "protocol" : "kernel",
               "scope" : "link"
            },
            {
               "dev" : "br0",
               "dst" : "10.0.0.0/8",
               "flags" : [],
               "prefsrc" : "10.0.0.1",
               "protocol" : "static",
               "scope" : "link"
            },
            {
               "dev" : "eth0",
               "dst" : "172.16.0.0/16",
               "flags" : [],
               "prefsrc" : "172.16.1.1",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        
        // Should show available interfaces
        assert!(error_chain.contains("Available interfaces: [br0, eth0]"));
        
        // Should show searched routes for br0 interface
        assert!(error_chain.contains("Searched routes:"));
        assert!(error_chain.contains("dst: Some(\"192.168.1.0/24\"), prefsrc: None"));
        assert!(error_chain.contains("dst: Some(\"10.0.0.0/8\"), prefsrc: Some(\"10.0.0.1\"), protocol: Some(\"static\")"));
        
        // Should not include eth0 routes in searched routes
        assert!(!error_chain.contains("172.16.0.0/16"));
    }

    #[test]
    fn test_error_handling_invalid_cidr_prefix_too_large() {
        // Test case: CIDR with prefix larger than valid (e.g., /33 for IPv4)
        // Note: Invalid CIDRs are filtered out by is_dev_ip, resulting in "interface not found"
        let json = r#"[
            {
               "dev" : "br0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "192.168.1.1",
               "protocol" : "static"
            },
            {
               "dev" : "br0",
               "dst" : "192.168.1.0/33",
               "flags" : [],
               "prefsrc" : "192.168.1.100",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        // Invalid CIDR causes is_dev_ip to return false, resulting in interface not found
        assert!(error_chain.contains("failed to find network configuration for interface 'br0'"));
        assert!(error_chain.contains("Available interfaces: [br0]"));
    }

    #[test]
    fn test_error_handling_invalid_cidr_base_address() {
        // Test case: CIDR with invalid base IP address
        // Note: Invalid CIDRs are filtered out by is_dev_ip, resulting in "interface not found"
        let json = r#"[
            {
               "dev" : "br0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "192.168.1.1",
               "protocol" : "static"
            },
            {
               "dev" : "br0",
               "dst" : "999.999.999.999/24",
               "flags" : [],
               "prefsrc" : "192.168.1.100",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        // Invalid CIDR causes is_dev_ip to return false, resulting in interface not found
        assert!(error_chain.contains("failed to find network configuration for interface 'br0'"));
        assert!(error_chain.contains("Available interfaces: [br0]"));
    }

    #[test]
    fn test_error_handling_routes_with_no_dev_field() {
        // Test case: Routes missing dev field entirely
        let json = r#"[
            {
               "dst" : "default",
               "flags" : [],
               "gateway" : "192.168.1.1",
               "protocol" : "static"
            },
            {
               "dst" : "192.168.1.0/24",
               "flags" : [],
               "prefsrc" : "192.168.1.100",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(json, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        assert!(error_chain.contains("failed to find network configuration for interface 'br0'"));
        assert!(error_chain.contains("Available interfaces: []"));
    }

    #[test]
    fn test_error_handling_mixed_valid_invalid_routes() {
        // Test case: Mix of valid and invalid routes
        let json = r#"[
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
            },
            {
               "dev" : "eth0",
               "dst" : "invalid-cidr-format",
               "flags" : [],
               "prefsrc" : "10.0.0.1",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        // Should succeed for br0 despite invalid eth0 route
        let result = NetConf::from_json(json, "br0");
        assert!(result.is_ok());
        let net_conf = result.unwrap();
        assert_eq!(net_conf.gateway_ip, IpAddr::from(Ipv4Addr::from_str("192.168.1.1").unwrap()));
        assert_eq!(net_conf.host_ip, IpAddr::from(Ipv4Addr::from_str("192.168.1.100").unwrap()));
        
        // Should fail for eth0 due to invalid CIDR
        let result_eth0 = NetConf::from_json(json, "eth0");
        assert!(result_eth0.is_err());
        let error_chain = format!("{:#}", result_eth0.unwrap_err());
        assert!(error_chain.contains("failed to find network configuration for interface 'eth0'"));
    }

    #[test]
    fn test_error_handling_specific_invalid_host_ip_parsing() {
        // Test case: Valid route structure that triggers specific host IP parsing error
        // This test uses a mock approach to demonstrate the specific error path
        // In practice, this error would be caught by JSON parsing or earlier validation
        let json = r#"[
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
        
        // This should succeed with valid data
        let result = NetConf::from_json(json, "br0");
        assert!(result.is_ok());
        
        // Test with invalid host IP in prefsrc - this will be caught by is_dev_ip filtering
        // since invalid IPs in JSON would cause JSON parsing to fail or be filtered out
        let invalid_json = r#"[
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
               "prefsrc" : "not-an-ip",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(invalid_json, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        // This will trigger the specific host IP parsing error
        assert!(error_chain.contains("cannot parse host ip 'not-an-ip'"));
    }

    #[test]
    fn test_error_handling_comprehensive_error_scenarios() {
        // Test case: Comprehensive test covering multiple error scenarios
        
        // Test 1: Missing default route
        let no_default = r#"[
            {
               "dev" : "br0",
               "dst" : "192.168.1.0/24",
               "flags" : [],
               "prefsrc" : "192.168.1.100",
               "protocol" : "kernel",
               "scope" : "link"
            }
         ]"#;
        
        let result = NetConf::from_json(no_default, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        assert!(error_chain.contains("default route not found"));
        assert!(error_chain.contains("Available routes: [192.168.1.0/24]"));
        
        // Test 2: Missing gateway field in default route
        let no_gateway = r#"[
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
        
        let result = NetConf::from_json(no_gateway, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        assert!(error_chain.contains("default route 'gateway' field not found"));
        
        // Test 3: Invalid gateway IP format
        let invalid_gateway = r#"[
            {
               "dev" : "br0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "invalid-gateway-ip",
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
        
        let result = NetConf::from_json(invalid_gateway, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        assert!(error_chain.contains("failed to parse gateway ip 'invalid-gateway-ip'"));
        
        // Test 4: Interface not found
        let wrong_interface = r#"[
            {
               "dev" : "eth0",
               "dst" : "default",
               "flags" : [],
               "gateway" : "192.168.1.1",
               "protocol" : "static"
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
        
        let result = NetConf::from_json(wrong_interface, "br0");
        assert!(result.is_err());
        let error_chain = format!("{:#}", result.unwrap_err());
        assert!(error_chain.contains("failed to find network configuration for interface 'br0'"));
        assert!(error_chain.contains("Available interfaces: [eth0]"));
    }
}
