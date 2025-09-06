# Technology Stack

## Language & Runtime
- **Rust 1.86.0** - Primary language for all components
- **Rhai** - Scripting language for Babel plugins
- **Edition 2021** - Rust edition used across workspace

## Build System
- **Cargo workspace** - Multi-crate project structure
- **Target**: `x86_64-unknown-linux-musl` for release builds
- **Toolchain components**: cargo, rustfmt, clippy, rust-std, rustc

## Key Dependencies
- **Apptainer** - Container runtime for node isolation
- **Protocol Buffers** - API definitions (via git submodules)
- **SystemD** - Service management on Linux hosts
- **S3-compatible storage** - For protocol data snapshots

## Common Commands

### Development
```bash
# Build debug binaries
make build

# Build release binaries (with stripping and permissions)
make build-release

# Create deployment bundle
make bundle

# Create development bundle
make bundle-dev
```

### Release Management
```bash
# Create new release (requires git-conventional-commits and cargo-release)
make new-release

# Promote to staging/production
make promote-staging
make promote-prod
```

### Testing
```bash
# Run tests
cargo test

# Run integration tests
cargo test --test itests
```

## Code Style
- **rustfmt edition**: 2024
- Follow standard Rust conventions
- Use `clippy` for linting

## Build Artifacts
- Binaries are stripped and have setuid permissions set for release
- Bundle structure: `/tmp/bundle/{blockvisor,babel,installer}/`
- Documentation and examples included in bundles