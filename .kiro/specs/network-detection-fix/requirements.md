# Requirements Document

## Introduction

The current network detection logic in `bvup` fails when the gateway IP address is outside the host's subnet CIDR range. This is a common configuration in cloud environments, Docker networks, and complex routing setups where the gateway resides in a different subnet than the host. The system currently requires the gateway to be within the same CIDR range as the host interface, causing network detection to fail and default to `0.0.0.0` values.

This feature will fix the network detection logic to properly handle scenarios where the gateway is outside the host's subnet while maintaining backward compatibility with existing configurations.

## Requirements

### Requirement 1

**User Story:** As a system administrator deploying BlockVisor on cloud infrastructure, I want the network detection to work correctly even when my gateway is in a different subnet, so that I don't have to manually configure network parameters.

#### Acceptance Criteria

1. WHEN the system runs `ip --json route` AND finds a default route with a gateway outside the host's subnet THEN the system SHALL successfully detect the gateway IP
2. WHEN the system detects network configuration AND the gateway is outside the host's CIDR range THEN the system SHALL still proceed with network configuration
3. WHEN the system processes routing information THEN the system SHALL separate gateway detection from host network detection logic

### Requirement 2

**User Story:** As a developer maintaining the BlockVisor codebase, I want the network detection logic to be robust and handle various network topologies, so that the system works reliably across different deployment environments.

#### Acceptance Criteria

1. WHEN the system parses routing information THEN the system SHALL extract gateway IP from any route with `dst: "default"` regardless of subnet containment
2. WHEN the system determines host network configuration THEN the system SHALL find the appropriate interface route with `prefsrc` field independently of gateway location
3. WHEN the system calculates available IPs THEN the system SHALL exclude both gateway and host IPs from the available pool regardless of subnet relationships

### Requirement 3

**User Story:** As a system administrator with existing working deployments, I want the network detection fix to maintain backward compatibility, so that my current configurations continue to work without changes.

#### Acceptance Criteria

1. WHEN the system encounters a gateway within the host's subnet THEN the system SHALL continue to work as before
2. WHEN the system processes standard network configurations THEN the system SHALL maintain the same behavior for existing working setups
3. WHEN the system fails to detect network configuration THEN the system SHALL provide clear error messages indicating what information could not be determined

### Requirement 4

**User Story:** As a system administrator troubleshooting network issues, I want better error handling and logging in network detection, so that I can understand why detection fails and what manual parameters I need to provide.

#### Acceptance Criteria

1. WHEN network detection fails THEN the system SHALL provide specific error messages indicating which network parameters could not be detected
2. WHEN the system cannot find a default route THEN the system SHALL report "default route not found" with guidance
3. WHEN the system cannot find interface configuration THEN the system SHALL report which interface was searched and what was missing
4. WHEN the system successfully detects network configuration THEN the system SHALL log the detected values for verification

### Requirement 5

**User Story:** As a developer testing network detection logic, I want comprehensive test coverage for various network topologies, so that I can ensure the fix works across different scenarios.

#### Acceptance Criteria

1. WHEN tests run THEN the system SHALL include test cases for gateway outside subnet scenarios
2. WHEN tests run THEN the system SHALL include test cases for gateway inside subnet scenarios (existing behavior)
3. WHEN tests run THEN the system SHALL include test cases for missing route information
4. WHEN tests run THEN the system SHALL include test cases for malformed routing data