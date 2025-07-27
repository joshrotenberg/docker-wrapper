# Contributing

Thanks for your interest in contributing to docker-wrapper.

## Quick Start

**Prerequisites:** Rust 1.78+ and Docker 20.10+

```bash
git clone https://github.com/your-username/docker-wrapper.git
cd docker-wrapper
cargo test  # Verify everything works
```

## Development

1. **Create branch:** `git checkout -b your-feature`
2. **Make changes** and add tests
3. **Run checks:** `cargo test && cargo clippy && cargo fmt`
4. **Commit:** Use conventional commits (`feat:`, `fix:`, `docs:`)
5. **Push and create PR**

## Standards

- All tests must pass (256+ tests)
- Zero clippy warnings required
- Document public APIs with rustdoc
- Add integration tests for new commands
- Follow existing code patterns

## Current Status

- **Phase 2 Complete**: All 14 commands implemented with comprehensive tests
- **Decision Point**: Complete run command vs release preparation
- **Quality**: Production-ready with 0 warnings

See `.claude/CONTEXT.md` for detailed development context.