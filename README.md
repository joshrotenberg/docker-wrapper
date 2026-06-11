# docker-wrapper

[![Crates.io](https://img.shields.io/crates/v/docker-wrapper.svg)](https://crates.io/crates/docker-wrapper)
[![Documentation](https://docs.rs/docker-wrapper/badge.svg)](https://docs.rs/docker-wrapper)
[![CI](https://github.com/joshrotenberg/docker-wrapper/workflows/CI/badge.svg)](https://github.com/joshrotenberg/docker-wrapper/actions)
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

A type-safe Docker CLI wrapper for Rust.

## Installation

```toml
[dependencies]
docker-wrapper = "0.10"
tokio = { version = "1", features = ["full"] }
```

**MSRV:** 1.88.0

## Quick Start

```rust
use docker_wrapper::{DockerCommand, RunCommand, StopCommand, RmCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Run a container
    let container = RunCommand::new("nginx:alpine")
        .name("my-nginx")
        .port(8080, 80)
        .detach()
        .execute()
        .await?;

    println!("Started: {}", container.short());

    // Stop and remove
    StopCommand::new("my-nginx").execute().await?;
    RmCommand::new("my-nginx").execute().await?;

    Ok(())
}
```

> **Note:** Import `DockerCommand` trait to use `.execute()` on commands.

## Features

```toml
# Docker Compose support
docker-wrapper = { version = "0.10", features = ["compose"] }

# Container templates (Redis, PostgreSQL, etc.)
docker-wrapper = { version = "0.10", features = ["templates"] }
```

### Compose Example

```rust
use docker_wrapper::{DockerCommand, compose::{ComposeUpCommand, ComposeDownCommand, ComposeCommand}};

ComposeUpCommand::new()
    .file("docker-compose.yml")
    .detach()
    .execute()
    .await?;
```

### Templates Example

```rust
use docker_wrapper::{RedisTemplate, Template};

let redis = RedisTemplate::new("my-redis")
    .port(6379)
    .password("secret");

let id = redis.start().await?;
redis.stop().await?;
```

## Tracing

Every command invocation emits [`tracing`](https://docs.rs/tracing) spans and
events so you can observe Docker CLI activity in production. Instrumentation is
enabled by default via the `tracing` cargo feature; compile with
`--no-default-features` to drop the dependency entirely.

```toml
# Default: tracing instrumentation is on
docker-wrapper = "0.10"

# Opt out
docker-wrapper = { version = "0.10", default-features = false, features = ["compose"] }
```

### What gets emitted

Each call to `DockerCommand::execute` and `StreamableCommand::stream` is
wrapped in a span:

| Span                | Entered by                         | Fields                                                            |
|---------------------|------------------------------------|-------------------------------------------------------------------|
| `docker.command`    | `CommandExecutor::execute_command` | `command`, `args_count`, `platform`, `runtime`, `timeout_secs`    |
| `docker.process`    | process spawn                      | `full_command`                                                    |
| `docker.timeout`    | timeout-wrapped execution          | `timeout_secs`                                                    |
| `docker.stream`     | streaming execution                | `command`, `mode` (`handler` or `channel`)                        |

Within the `docker.command` span, events are emitted as:

- `info` on success: `exit_code`, `duration_ms`, `stdout_len`, `stderr_len`.
- `warn` on non-zero exit / spawn failure: `exit_code`, `duration_ms`,
  `stderr_snippet` (truncated to ~512 bytes), plus the error message.
- `trace` for raw stdout / stderr payloads.

Streaming variants additionally emit `debug!` for every stdout/stderr line
(set `RUST_LOG=docker_wrapper=debug` to see them), so noisy builds can be
filtered down by level.

### Subscribing

```rust,no_run
use tracing_subscriber::EnvFilter;

tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "docker_wrapper=info".into()))
    .init();
```

Useful `RUST_LOG` filters:

```bash
# All docker-wrapper activity
RUST_LOG=docker_wrapper=trace cargo run

# Just the top-level execute spans + completion events
RUST_LOG=docker_wrapper=info cargo run

# Stream commands with per-line output
RUST_LOG=docker_wrapper::stream=debug cargo run
```

## Why docker-wrapper?

This crate wraps the Docker CLI rather than calling the Docker API directly (like [bollard](https://crates.io/crates/bollard)).

| | docker-wrapper | bollard |
|---|---|---|
| **Approach** | Shells out to `docker` CLI | Calls Docker REST API directly |
| **Setup** | Just needs `docker` in PATH | Needs API socket access |
| **Compose** | Native `docker compose` support | Not supported |
| **Compatibility** | Works with Docker, Podman, Colima, etc. | Docker API only |
| **Performance** | Process spawn overhead | Direct API calls |
| **Use case** | CLI tools, scripts, dev tooling | High-performance services |

**Choose docker-wrapper when:** You're building CLI tools, need Compose support, want to work with Docker alternatives, or are migrating shell scripts to Rust.

**Choose bollard when:** You need maximum performance, direct API access, or are building a long-running service with many Docker operations.

## Documentation

- **[API Reference](https://docs.rs/docker-wrapper)** - Complete documentation with examples
- **[Examples](examples/)** - Working code examples

## License

MIT OR Apache-2.0
