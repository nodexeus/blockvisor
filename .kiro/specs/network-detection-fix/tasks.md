# Implementation Plan

- [x] 1. Create helper functions for network detection
  - Extract the network detection logic into separate, focused functions
  - Implement `extract_gateway()` function to get gateway from default route
  - Implement `extract_host_network()` function to get host network configuration
  - Implement `calculate_available_ips()` function to generate available IP list
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2_

- [x] 2. Add internal data structures
  - Create `HostNetwork` struct to hold intermediate host network data
  - Add proper error types for specific failure scenarios
  - _Requirements: 2.2, 4.1, 4.2, 4.3_

- [x] 3. Refactor NetConf::from_json() method
  - Replace the existing monolithic logic with calls to the new helper functions
  - Remove the `cidr.contains(gateway)` requirement that causes the bug
  - Implement the new phased approach: gateway detection, host network detection, available IP calculation
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3_

- [x] 4. Improve error handling and messages
  - Replace generic "failed to discover network config" with specific error messages
  - Add context to each error indicating what failed and why
  - Include detected values in error messages for debugging
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [x] 5. Add comprehensive test cases for gateway outside subnet scenarios
  - Create test cases where gateway is in different subnet than host
  - Test various network topologies (cloud environments, Docker networks)
  - Verify that available IPs correctly exclude both host and gateway IPs
  - _Requirements: 5.1, 1.1, 1.2, 1.3_

- [x] 6. Add backward compatibility test cases
  - Ensure existing test cases continue to pass
  - Add test cases for gateway inside subnet (current working behavior)
  - Verify no regression in standard network configurations
  - _Requirements: 5.2, 3.1, 3.2_

- [x] 7. Add error handling test cases
  - Test missing default route scenarios
  - Test missing gateway field in default route
  - Test missing interface routes
  - Test missing prefsrc field
  - Test invalid IP and CIDR formats
  - _Requirements: 5.3, 5.4, 4.1, 4.2, 4.3_

- [x] 8. Add real-world network configuration test cases
  - Create test data based on actual cloud provider network configurations
  - Test complex routing scenarios with multiple interfaces
  - Verify the fix works with various network topologies
  - _Requirements: 5.1, 1.1, 2.1_

- [x] 9. Update existing tests to use new error messages
  - Update any existing tests that expect the old generic error message
  - Ensure all tests pass with the new implementation
  - _Requirements: 3.3, 4.1_

- [x] 10. Integration testing with bvup binary
  - Test the complete bvup flow with the fixed network detection
  - Verify manual override parameters still work correctly
  - Test on systems with gateway outside subnet configuration
  - _Requirements: 1.1, 3.1, 3.2_