# Docker Wrapper

[![Crates.io](https://img.shields.io/crates/v/docker-wrapper.svg)](https://crates.io/crates/docker-wrapper)
[![Documentation](https://docs.rs/docker-wrapper/badge.svg)](https://docs.rs/docker-wrapper)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Build Status](https://github.com/joshrotenberg/docker-wrapper/workflows/CI/badge.svg)](https://github.com/joshrotenberg/docker-wrapper/actions)

A comprehensive Docker CLI wrapper for Rust with complete command coverage.

## Features

- **13 Docker commands** with 100%+ option coverage
- **393 comprehensive tests** (unit, integration, doctests)
- **Async-first** with tokio integration
- **Builder patterns** for all commands
- **Type-safe** error handling
- **Zero unsafe code**

## Commands

| Command | Options | Description |
|---------|---------|-------------|
| `run` | 108 methods | Container execution with full option support |
| `build` | 54 methods | Image building with Dockerfile support |
| `exec` | 16 methods | Execute commands in running containers |
| `images` | 25 methods | List and manage images |
| `ps` | 18 methods | List containers with filtering |
| `pull` | 10 methods | Pull images from registries |
| `push` | 10 methods | Push images to registries |
| `search` | 22 methods | Search Docker Hub |
| `login` | 10 methods | Registry authentication |
| `logout` | 8 methods | Registry logout |
| `bake` | 28 methods | Buildx bake support |
| `version` | 13 methods | Docker version information |
| `info` | 19 methods | Docker system information |

## Installation

```toml
[dependencies]
docker-wrapper = "0.1.0"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
```

## Quick Start

```rust
use docker_wrapper::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Run a container
    let result = RunCommand::new("alpine:latest")
        .name("test-container")
        .it()
        .remove()
        .cmd(vec!["echo".to_string(), "Hello World".to_string()])
        .run()
        .await?;

    println!("Container output: {}", result.stdout);
    Ok(())
}
```

## Architecture

Each command implements the `DockerCommand` trait with:
- **Builder pattern** for configuration
- **`run()` method** for execution
- **Typed output** parsing
- **Comprehensive error handling**

```rust
// All commands follow this pattern
let result = CommandBuilder::new(args)
    .option1(value)
    .option2(value)
    .run()
    .await?;
```

## Examples

See [examples/](examples/) directory for comprehensive usage examples:
- Basic container operations
- Advanced build configurations
- Registry operations
- Error handling patterns

## Documentation

- [API Documentation](https://docs.rs/docker-wrapper) - Complete API reference
- [Examples](examples/) - Usage examples for all commands

## Testing

```bash
# Run all tests
cargo test

# Run specific command tests
cargo test run::tests
cargo test --test run_integration

# Check code quality
cargo clippy --all-targets --all-features -- -D warnings
```

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.