# docker-wrapper

[![Crates.io](https://img.shields.io/crates/v/docker-wrapper.svg)](https://crates.io/crates/docker-wrapper)
[![Documentation](https://docs.rs/docker-wrapper/badge.svg)](https://docs.rs/docker-wrapper)
[![CI](https://github.com/joshrotenberg/docker-wrapper/workflows/CI/badge.svg)](https://github.com/joshrotenberg/docker-wrapper/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A comprehensive, type-safe Docker CLI wrapper for Rust applications.

## Features

- Complete Docker CLI coverage (35+ commands)
- Type-safe builder pattern API
- Async/await support with Tokio
- Real-time output streaming
- Docker Compose support (optional)
- Zero unsafe code
- Extensive test coverage

## Installation

```toml
[dependencies]
docker-wrapper = "0.2"
tokio = { version = "1", features = ["full"] }
```

Enable Docker Compose support:

```toml
[dependencies]
docker-wrapper = { version = "0.2", features = ["compose"] }
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

## Documentation

For comprehensive documentation, examples, and API reference:

- **[API Documentation](https://docs.rs/docker-wrapper)** - Complete API reference with examples
- **[Examples](examples/)** - Working examples for common use cases
- **[GitHub Repository](https://github.com/joshrotenberg/docker-wrapper)** - Source code and issue tracking

## Examples

The `examples/` directory contains practical examples:

- `basic_usage.rs` - Common Docker operations
- `container_lifecycle.rs` - Container management
- `streaming.rs` - Real-time output streaming
- `docker_compose.rs` - Docker Compose usage (requires `compose` feature)
- `error_handling.rs` - Error handling patterns

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