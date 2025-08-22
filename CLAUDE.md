# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Docker Wrapper is a comprehensive Docker CLI wrapper for Rust that provides type-safe, async-first interfaces to Docker commands. The library implements 13+ Docker commands with 100% option coverage through a builder pattern API.

## Build and Test Commands

### Standard Development Flow
```bash
# Format code
cargo fmt --all -- --check

# Run clippy linting (required to pass)
cargo clippy --all-targets --all-features -- -D warnings

# Run unit tests
cargo test --lib --all-features

# Run integration tests (requires Docker)
cargo test --test '*' --all-features

# Run doc tests
cargo test --doc --all-features

# Build documentation
cargo doc --all-features --no-deps

# Check examples compile
cargo check --examples --all-features
```

### Running Specific Tests
```bash
# Test a specific command module
cargo test run::tests
cargo test --test run_integration

# Test with verbose output
cargo test --verbose
```

## Architecture

### Core Module Structure

The codebase follows a command-pattern architecture where each Docker command is a separate module implementing the `DockerCommand` trait:

- **`src/command.rs`**: Base trait and shared infrastructure
  - `DockerCommand` trait - all commands implement this
  - `CommandExecutor` - handles raw command execution
  - Helper builders: `EnvironmentBuilder`, `PortBuilder`
  
- **`src/command/*.rs`**: Individual command implementations
  - Each command has its own builder struct (e.g., `RunCommand`, `BuildCommand`)
  - Builder pattern with fluent API for configuration
  - Typed output parsing for each command

- **`src/error.rs`**: Comprehensive error handling using `thiserror`
  - Categorized errors with context
  - Retryable error detection

### Key Design Patterns

1. **Builder Pattern**: All commands use builder pattern with method chaining:
   ```rust
   RunCommand::new("alpine:latest")
       .name("test")
       .detach()
       .run()
       .await?
   ```

2. **Escape Hatches**: Every command supports raw arguments via `.arg()` and `.args()` methods for unimplemented options

3. **Async-First**: All command execution is async using tokio

4. **Type Safety**: Strongly typed outputs with JSON parsing where applicable

## Testing Strategy

- **Unit Tests**: Located alongside source files, test builder logic and argument construction
- **Integration Tests**: In `tests/` directory, require Docker daemon running
- **Doc Tests**: Examples in documentation are executable tests
- **Coverage**: Target 70% minimum coverage, use `cargo tarpaulin` for measurement

## Important Implementation Notes

- All public APIs must have doc comments
- Error messages should include helpful context
- Commands map directly to Docker CLI arguments (e.g., `--name` becomes `.name()`)
- Use `#[must_use]` on builder methods for better ergonomics
- Escape hatch methods (`.arg()`, `.args()`) enable forward compatibility
- things to remember: a branch and pr with conventional commits for everything, clippy and fmt clean, reasonable test coverage for everything, rustdoc should have a reasonable example for every function
- always run cargo fmt --all and cargo clippy  --workspace  --all-targets --all-features -- -D warnings before pushing
- no emojis
- always squash commits before a merge