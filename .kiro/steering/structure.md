# Project Structure

## Workspace Organization

This is a Cargo workspace with multiple crates organized by functionality:

### Core Crates
- **`bv/`** - Main BlockVisor daemon and CLI tools
  - `src/bin/` - Binary executables (blockvisord, bv, nib, etc.)
  - `src/services/` - Service implementations (API, MQTT, protocol, etc.)
  - `tests/` - Integration tests with test images
  - `data/` - Configuration templates and service files

- **`babel/`** - Protocol abstraction layer
  - `src/bin/` - Babel daemon and job runner binaries
  - Core job management and execution system

- **`babel_api/`** - Babel plugin API and Rhai integration
  - `examples/` - Rhai plugin examples and templates
  - `rhai_plugin_guide.md` - Plugin development documentation

### Utility Crates
- **`bv_utils/`** - Shared utilities (logging, RPC, system operations)
- **`bv_tests_utils/`** - Testing utilities and mocks

## Key Directories

### Protocol Definitions
- **`bv/proto/`** - Git submodule containing Protocol Buffer definitions
- Located at: `https://github.com/blockjoy/api-proto`

### Configuration & Data
- **`bv/data/`** - Template files for node configuration
  - `Dockerfile.template` - Container image template
  - `babel.yaml.template` - Babel configuration template
  - `main.rhai.template` - Plugin script template

### Documentation
- **Root level**: Architecture diagrams (`.jpg` files) and guides
- **`babel_api/`**: Plugin development guide and examples

## File Naming Conventions

### Rust Files
- Snake_case for module names and file names
- Service modules in `services/` subdirectory
- Binary executables in `src/bin/` directories

### Configuration Files
- `.template` suffix for template files
- YAML for configuration files (`babel.yaml`, `protocols.yaml`)
- TOML for Rust project files (`Cargo.toml`, `rustfmt.toml`)

### Documentation
- Markdown files (`.md`) for documentation
- Architecture diagrams as JPEG files
- Plugin examples in `babel_api/examples/`

## Important Paths

### Development
- `/target/` - Build artifacts (gitignored)
- `/.kiro/` - Kiro IDE configuration and steering rules

### Runtime (Host System)
- `/opt/blockvisor/` - Installation directory
- `/etc/blockvisor.json` - Host configuration
- `/var/lib/blockvisor/` - Runtime data and node storage
- `/var/lib/blockvisor/nodes/<uuid>/` - Per-node data directories

### Runtime (Container)
- `/usr/bin/babel*` - Babel binaries
- `/etc/babel.conf` - Babel configuration
- `/var/lib/babel/` - Babel runtime data
- `/blockjoy/` - Protocol data mount point