# Contributing

Thanks for your interest in contributing to docker-wrapper.

## Development Setup

**Prerequisites:**
- Rust 1.78+
- Docker 20.10+

**Setup:**
```bash
git clone https://github.com/your-username/docker-wrapper.git
cd docker-wrapper
cargo test  # Verify everything works
```

## Making Changes

1. **Create a branch:** `git checkout -b your-feature`
2. **Make changes** and add tests
3. **Run checks:** `cargo test && cargo clippy && cargo fmt`
4. **Commit:** Use conventional commits (`feat:`, `fix:`, `docs:`, etc.)
5. **Push and create PR**

## Code Standards

- Follow `cargo fmt` and `cargo clippy`
- Add tests for new functionality
- Document public APIs
- Handle errors with our `Result` type

## Running Tests

```bash
cargo test                    # Unit tests
cargo test --all-targets      # All tests (requires Docker)
```

## Areas to Contribute

- **Bug fixes** - Issues labeled `bug`
- **Documentation** - Examples and guides  
- **Tests** - Improve coverage
- **New features** - Additional Docker commands

## Getting Help

- **GitHub Issues** - Bug reports and feature requests
- **GitHub Discussions** - Questions and ideas

By contributing, you agree your contributions will be licensed under MIT OR Apache-2.0.