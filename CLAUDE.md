# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Docker Wrapper is a comprehensive Docker CLI wrapper for Rust that provides type-safe, async-first interfaces to Docker commands. The library implements 74 Docker and Docker Compose commands with builder pattern APIs and escape hatches for forward compatibility.

## Build and Test Commands

### Required Development Flow
```bash
# Before committing - ALWAYS run in this order:
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --lib --all-features
cargo test --test '*' --all-features
```

### Additional Testing Commands
```bash
# Test specific command module
cargo test command_name::tests
cargo test --test command_name_integration

# Run with verbose output
cargo test --verbose

# Build documentation
cargo doc --all-features --no-deps

# Check examples compile
cargo check --examples --all-features

# Measure test coverage
cargo tarpaulin
```

## Architecture

### Core Module Structure

The codebase implements a dual-trait architecture during migration:

- **`src/command.rs`**: Base traits and shared infrastructure
  - `DockerCommandV2` trait - new unified trait (46/74 commands migrated)
  - `DockerCommand` trait - legacy trait (being phased out)
  - `CommandExecutor` - handles raw command execution with public visibility
  - Helper builders: `EnvironmentBuilder`, `PortBuilder`
  
- **`src/command/*.rs`**: Individual Docker command implementations
  - Each command has builder struct following pattern: `{Command}Command`
  - Builder pattern with fluent API and method chaining
  - Typed output parsing with serde_json
  
- **`src/compose/*.rs`**: Docker Compose v2 command implementations
  - Separate modules for compose-specific functionality
  - Most compose commands now migrated to unified pattern in `src/command/compose_*.rs`

- **`src/error.rs`**: Error handling using `thiserror`
  - Categorized errors with context
  - Retryable error detection

### DockerCommandV2 Migration Pattern

When migrating commands from `DockerCommand` to `DockerCommandV2`:

1. Change trait from `DockerCommand` to `DockerCommandV2`
2. Make `executor` field public for extensibility
3. Add `get_executor()` and `get_executor_mut()` methods
4. Update `build_command_args()` to include command name as first argument
5. Add raw args support: `args.extend(self.executor.raw_args.clone())`
6. Update `execute()` method to use new pattern
7. Remove legacy trait method implementations (arg, args, flag, option)
8. Update test assertions to expect command name as first argument

### Migration Status

**Completed (52 commands)**: attach, build, commit, cp, create, diff, exec, export, kill, login, logout, logs, port, pull, push, restart, run, search, start, stats, stop, tag, images, all compose_* commands

**Remaining (22 commands)**: bake, events, history, import, info, inspect, load, network, pause, ps, rename, rm, rmi, save, system, top, unpause, update, version, volume, wait, container_prune, image_prune

## Key Implementation Rules

### Code Standards
- No emojis in code, commits, or documentation
- All public APIs must have doc comments with examples
- Use `#[must_use]` on builder methods
- Commands map directly to Docker CLI (e.g., `--name` becomes `.name()`)
- Escape hatch methods (`.arg()`, `.args()`) required for all commands
- Maintain 70% minimum test coverage

### Git Workflow
1. **ALWAYS** create feature branch first: `git checkout -b feat/description`
2. Use conventional commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`
3. Run fmt and clippy before every push
4. Never commit to main directly
5. Don't merge PRs without explicit permission

### Testing Requirements
- Unit tests alongside source files
- Integration tests in `tests/` directory (require Docker daemon)
- Doc tests for all examples
- Test both builder logic and command execution