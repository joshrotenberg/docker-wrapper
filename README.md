# docker-wrapper

[![Crates.io](https://img.shields.io/crates/v/docker-wrapper.svg)](https://crates.io/crates/docker-wrapper)
[![Documentation](https://docs.rs/docker-wrapper/badge.svg)](https://docs.rs/docker-wrapper)
[![CI](https://github.com/joshrotenberg/docker-wrapper/workflows/CI/badge.svg)](https://github.com/joshrotenberg/docker-wrapper/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A comprehensive, type-safe Docker CLI wrapper for Rust applications.

## Features

- Comprehensive Docker CLI coverage (80+ commands implemented)
- Type-safe builder pattern API
- Async/await support with Tokio
- Real-time output streaming
- Docker Compose support (optional)
- Docker builder commands for build cache management
- Network and volume management
- Zero unsafe code
- Extensive test coverage (680+ tests)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
docker-wrapper = "0.4"
tokio = { version = "1", features = ["full"] }
```

**Minimum Supported Rust Version (MSRV):** 1.78.0

Enable Docker Compose support:

```toml
[dependencies]
docker-wrapper = { version = "0.4", features = ["compose"] }
```

## Quick Start

```rust
use docker_wrapper::{DockerCommand, RunCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Run a container
    let output = RunCommand::new("nginx:latest")
        .name("my-web-server")
        .port(8080, 80)
        .detach()
        .execute()
        .await?;

    println!("Container started: {}", output.container_id);
    Ok(())
}
```

### Docker Builder Example

```rust
use docker_wrapper::{DockerCommand, BuilderPruneCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Clean up build cache
    let result = BuilderPruneCommand::new()
        .all()
        .keep_storage("5GB")
        .force()
        .execute()
        .await?;

    println!("Reclaimed {} bytes of disk space", 
             result.space_reclaimed.unwrap_or(0));
    Ok(())
}
```

## When to Use docker-wrapper

docker-wrapper is ideal for:

- **CLI tools and automation scripts** - Familiar Docker CLI behavior
- **Docker Compose workflows** - Native compose command support  
- **Development tools** - Container management for dev environments
- **Shell script migration** - Type-safe Rust alternative to bash scripts
- **Cross-platform support** - Works with Docker, Podman, Colima, etc.

**Choosing between Docker Rust libraries?** See our comprehensive [**Comparison Guide**](docs/COMPARISON.md) comparing docker-wrapper vs bollard vs testcontainers-rs.

## Documentation

For comprehensive documentation, examples, and API reference:

- **[API Documentation](https://docs.rs/docker-wrapper)** - Complete API reference with examples
- **[Examples](examples/)** - Working examples for common use cases
- **[Comparison Guide](docs/COMPARISON.md)** - docker-wrapper vs other Docker Rust libraries
- **[Command Coverage](docs/DOCKER_COMMAND_COVERAGE.md)** - Docker CLI command implementation status
- **[GitHub Repository](https://github.com/joshrotenberg/docker-wrapper)** - Source code and issue tracking

## Examples

The `examples/` directory contains practical examples:

- `basic_usage.rs` - Common Docker operations
- `basic_docker_patterns.rs` - Essential Docker patterns and best practices
- `exec_examples.rs` - Container command execution
- `lifecycle_commands.rs` - Container lifecycle management
- `run_examples.rs` - Advanced run command usage
- `streaming.rs` - Real-time output streaming
- `docker_compose.rs` - Docker Compose usage (requires `compose` feature)
- `debugging_features.rs` - Debugging and inspection features
- `system_cleanup.rs` - System maintenance and cleanup
- `complete_run_coverage.rs` - Comprehensive run command options

Run examples:

```bash
cargo run --example basic_usage
cargo run --example streaming
cargo run --features compose --example docker_compose
```

## Contributing

Contributions are welcome! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

Licensed under the MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT).