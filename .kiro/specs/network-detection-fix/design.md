# Design Document

## Overview

This design addresses the network detection failure in `bvup` when the gateway IP is outside the host's subnet CIDR range. The current implementation in `NetConf::from_json()` requires the gateway to be contained within the host's network CIDR, which fails in many real-world network configurations.

The solution separates gateway detection from host network detection, allowing them to be processed independently while maintaining backward compatibility.

## Architecture

### Current Architecture Issues

The current `NetConf::from_json()` function has a flawed architecture:

1. **Coupled Logic**: Gateway and host network detection are tightly coupled
2. **Incorrect Assumption**: Assumes gateway must be in the same subnet as the host
3. **Single Point of Failure**: If gateway is outside subnet, entire detection fails
4. **Poor Error Handling**: Generic "failed to discover network config" error

### New Architecture

The new design separates concerns into distinct phases:

1. **Gateway Detection Phase**: Extract gateway from default route independently
2. **Host Network Detection Phase**: Find host's network configuration from interface routes
3. **Available IP Calculation Phase**: Calculate available IPs from host's subnet
4. **Validation Phase**: Ensure all required information was found

## Components and Interfaces

### Modified NetConf::from_json() Function

```rust
fn from_json(json: &str, ifa_name: &str) -> Result<Self> {
    let routes = serde_json::from_str::<Vec<IpRoute>>(json)?;
    
    // Phase 1: Extract gateway from default route
    let gateway = extract_gateway(&routes)?;
    
    // Phase 2: Find host network configuration
    let host_network = extract_host_network(&routes, ifa_name)?;
    
    // Phase 3: Calculate available IPs
    let available_ips = calculate_available_ips(&host_network, &gateway)?;
    
    Ok(NetConf {
        gateway_ip: gateway,
        host_ip: host_network.host_ip,
        prefix: host_network.prefix,
        available_ips,
    })
}
```

### New Helper Functions

#### extract_gateway()
```rust
fn extract_gateway(routes: &[IpRoute]) -> Result<IpAddr> {
    let gateway_str = routes
        .iter()
        .find(|route| route.dst.as_ref().is_some_and(|v| v == "default"))
        .ok_or(anyhow!("default route not found"))?
        .gateway
        .as_ref()
        .ok_or(anyhow!("default route 'gateway' field not found"))?;
    
    IpAddr::from_str(gateway_str)
        .with_context(|| format!("failed to parse gateway ip '{gateway_str}'"))
}
```

#### extract_host_network()
```rust
struct HostNetwork {
    host_ip: IpAddr,
    cidr: IpCidr,
    prefix: u8,
}

fn extract_host_network(routes: &[IpRoute], ifa_name: &str) -> Result<HostNetwork> {
    for route in routes {
        if is_dev_ip(route, ifa_name) {
            if let Some(dst) = &route.dst {
                let cidr = IpCidr::from_str(dst)
                    .with_context(|| format!("cannot parse '{dst}' as cidr"))?;
                
                let prefsrc = route
                    .prefsrc
                    .as_ref()
                    .ok_or(anyhow!("'prefsrc' not found for interface '{ifa_name}'"))?;
                
                let host_ip = IpAddr::from_str(prefsrc)
                    .with_context(|| format!("cannot parse host ip '{prefsrc}'"))?;
                
                return Ok(HostNetwork {
                    host_ip,
                    cidr: cidr.clone(),
                    prefix: get_bits(&cidr),
                });
            }
        }
    }
    
    bail!("failed to find network configuration for interface '{ifa_name}'");
}
```

#### calculate_available_ips()
```rust
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
        // Remove both host IP and gateway IP from available pool
        available_ips.retain(|ip| *ip != host_network.host_ip && *ip != *gateway);
        Ok(available_ips)
    } else {
        bail!("Failed to resolve ip range from CIDR {}", host_network.cidr);
    }
}
```

## Data Models

### Existing Data Models
The existing `NetConf` struct remains unchanged to maintain API compatibility:

```rust
pub struct NetConf {
    pub gateway_ip: IpAddr,
    pub host_ip: IpAddr,
    pub prefix: u8,
    pub available_ips: Vec<IpAddr>,
}
```

### New Internal Data Models

```rust
struct HostNetwork {
    host_ip: IpAddr,
    cidr: IpCidr,
    prefix: u8,
}
```

## Error Handling

### Improved Error Messages

The new design provides specific error messages for each failure scenario:

1. **Gateway Detection Errors**:
   - "default route not found" - No route with `dst: "default"`
   - "default route 'gateway' field not found" - Default route missing gateway field
   - "failed to parse gateway ip '{ip}'" - Invalid gateway IP format

2. **Host Network Detection Errors**:
   - "failed to find network configuration for interface '{interface}'" - No matching interface route
   - "'prefsrc' not found for interface '{interface}'" - Route missing prefsrc field
   - "cannot parse host ip '{ip}'" - Invalid host IP format
   - "cannot parse '{cidr}' as cidr" - Invalid CIDR format

3. **Available IP Calculation Errors**:
   - "Failed to resolve ip range from CIDR {cidr}" - Cannot generate IP range

### Error Recovery

The function will attempt to provide as much information as possible even if some detection fails:

- If gateway detection fails but host network succeeds, provide detailed error about gateway
- If host network detection fails but gateway succeeds, provide detailed error about interface
- Include detected values in error messages for debugging

## Testing Strategy

### Unit Test Categories

1. **Gateway Outside Subnet Tests**:
   - Gateway in different subnet than host
   - Gateway in completely different IP range
   - Multiple interfaces with gateway outside all subnets

2. **Backward Compatibility Tests**:
   - Gateway inside host subnet (existing behavior)
   - Standard network configurations
   - Edge cases that currently work

3. **Error Handling Tests**:
   - Missing default route
   - Missing gateway field
   - Missing interface routes
   - Missing prefsrc field
   - Invalid IP formats
   - Invalid CIDR formats

4. **Real-World Scenario Tests**:
   - AWS/GCP/Azure network configurations
   - Docker bridge networks
   - Complex routing setups
   - Multiple interface configurations

### Test Data Examples

```rust
// Gateway outside subnet test case
let json_gateway_outside = r#"[
    {
        "dst": "default",
        "gateway": "10.0.0.1",
        "dev": "br0"
    },
    {
        "dst": "192.168.1.0/24",
        "dev": "br0",
        "prefsrc": "192.168.1.100",
        "protocol": "kernel"
    }
]"#;

// Gateway inside subnet test case (existing behavior)
let json_gateway_inside = r#"[
    {
        "dst": "default",
        "gateway": "192.168.1.1",
        "dev": "br0"
    },
    {
        "dst": "192.168.1.0/24",
        "dev": "br0",
        "prefsrc": "192.168.1.100",
        "protocol": "kernel"
    }
]"#;
```

### Integration Testing

- Test with actual `ip --json route` output from various environments
- Verify bvup works end-to-end with the fixed detection
- Test manual override parameters still work correctly

## Implementation Considerations

### Backward Compatibility

- Existing `NetConf` API remains unchanged
- All existing functionality preserved
- No changes to configuration file format
- Manual override parameters continue to work

### Performance

- Minimal performance impact (same number of route iterations)
- Cleaner separation of concerns improves maintainability
- Better error messages aid in debugging

### Security

- No security implications
- Same validation of IP addresses and CIDR ranges
- No new external dependencies