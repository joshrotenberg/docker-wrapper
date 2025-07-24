# ADR: Development Environment Setup and Guidelines

**Status**: Accepted
**Date**: 2025-07-24
**Author**: Development Team
**Tags**: [docs, development, environment, setup, tooling]

## Context

The docker-wrapper project requires a consistent development environment setup across all contributors to ensure code quality, compatibility, and efficient development workflows. Without standardized guidelines, developers may encounter inconsistencies in dependencies, tooling, and development practices that could lead to build failures, test inconsistencies, and reduced productivity.

## Decision

We establish comprehensive development environment setup guidelines that include:

1. **Standardized Prerequisites**: Rust 1.70+, Docker 20.10+, Git, and recommended IDE setup
2. **Consistent Development Workflow**: Standard commands for building, testing, and validation
3. **Architecture Documentation**: Clear module organization and development patterns
4. **Quality Gates**: Mandatory checks before contribution (tests, clippy, formatting)

### Development Environment Requirements

- **Rust**: 1.70+ (stable toolchain) for MSRV compatibility
- **Docker**: 20.10+ installed and running for integration tests
- **Git**: For version control and contribution workflow
- **IDE**: VS Code with rust-analyzer extension recommended

### Standard Workflow Commands

```bash
# Initial setup
git clone https://github.com/joshrotenberg/docker-wrapper.git
cd docker-wrapper
cargo build

# Development cycle
cargo test --lib                    # Unit tests
cargo test --test integration       # Integration tests (requires Docker)
cargo clippy                        # Linting
cargo fmt                          # Formatting
cargo doc --open                   # Documentation generation
```

### Module Architecture Understanding

- `src/client.rs` - Core Docker client with automatic binary detection
- `src/container/` - Container lifecycle management with builder patterns
- `src/image.rs` - Image operations and registry integration
- `src/network.rs` - Network management with multi-driver support
- `src/volume.rs` - Volume operations and storage management
- `src/events.rs` - Real-time Docker event streaming
- `src/stats.rs` - Container statistics and monitoring
- `src/types.rs` - Type-safe wrappers and core data structures
- `src/error.rs` - Comprehensive error handling system

## Consequences

### Positive Consequences
- Consistent development experience across all contributors
- Reduced setup friction for new developers
- Standardized quality gates ensure code consistency
- Clear architecture documentation improves maintainability
- Explicit MSRV support ensures broad compatibility

### Negative Consequences
- Additional setup overhead for first-time contributors
- Dependency on specific tool versions may require updates over time
- Docker requirement for full testing may limit some development environments

### Risks
- Tool version drift over time without maintenance
- Platform-specific setup issues not covered in generic guidelines
- Integration test failures due to Docker daemon configuration differences

## Alternatives Considered

### Alternative 1: Minimal Setup Documentation
- **Description**: Basic README with minimal setup instructions
- **Pros**: Simple, less maintenance overhead
- **Cons**: Inconsistent development environments, harder onboarding
- **Why rejected**: Would lead to support burden and inconsistent contributions

### Alternative 2: Containerized Development Environment
- **Description**: Docker-based development environment with all tools
- **Pros**: Perfect consistency, easy setup
- **Cons**: Performance overhead, complexity for simple changes
- **Why rejected**: Overkill for Rust development, would slow down development workflow

### Alternative 3: Multiple Platform-Specific Guides
- **Description**: Separate setup guides for each operating system
- **Pros**: Platform-optimized instructions
- **Cons**: Maintenance burden, fragmentation
- **Why rejected**: Rust toolchain is sufficiently cross-platform to use unified approach

## Implementation Notes

- Development guide will be maintained in the context system as an ADR
- Setup verification script may be added in the future for automated environment checking
- IDE configuration files (.vscode/settings.json) should be included in repository
- Regular review of tool versions and requirements as ecosystem evolves

## References

- [Rust Installation Guide](https://rustup.rs/)
- [Docker Installation](https://docs.docker.com/get-docker/)
- [VS Code Rust Extension](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [Project CONTRIBUTING.md](../../CONTRIBUTING.md)

## Status History

- 2025-07-24: Accepted - Converted from DEVELOPMENT_GUIDE.md to structured ADR format
- 2025-01-18: Initial implementation in DEVELOPMENT_GUIDE.md