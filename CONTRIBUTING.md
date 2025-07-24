# Contributing to Docker Wrapper

Thank you for your interest in contributing to Docker Wrapper! We welcome contributions from the community and are excited to see what you'll bring to the project.

## Getting Started

### Prerequisites

- **Rust**: Latest stable version (1.70+)
- **Docker**: Version 20.10+ installed and running
- **Git**: For version control

### Development Setup

1. **Fork and clone the repository:**
   ```bash
   git clone https://github.com/your-username/docker-wrapper.git
   cd docker-wrapper
   ```

2. **Install dependencies:**
   ```bash
   cargo build
   ```

3. **Run tests to ensure everything works:**
   ```bash
   cargo test
   ```

4. **Run examples to verify functionality:**
   ```bash
   cargo run --example basic_usage
   ```

## Development Workflow

### Making Changes

1. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following our coding standards (see below)

3. **Add tests** for new functionality

4. **Run the test suite:**
   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```

5. **Update documentation** if needed

6. **Commit your changes** with descriptive messages:
   ```bash
   git commit -m "feat: add real-time container health monitoring"
   ```

### Pull Request Process

1. **Push your branch:**
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create a Pull Request** with:
   - Clear title describing the change
   - Detailed description of what was changed and why
   - Reference to any related issues
   - Screenshots/examples if applicable

3. **Ensure CI passes** - all tests and checks must pass

4. **Respond to review feedback** promptly and thoughtfully

## 📝 Coding Standards

### Code Style

- **Follow Rust conventions** and use `cargo fmt`
- **Use `cargo clippy`** to catch common mistakes
- **Write descriptive variable and function names**
- **Add comprehensive documentation** for public APIs
- **Include doc examples** for complex functions

### Error Handling

- **Use the `DockerResult<T>` type** for fallible operations
- **Provide helpful error messages** with context
- **Use `thiserror` for custom error types**
- **Handle errors gracefully** without panicking

### Testing

- **Write unit tests** for all new functionality
- **Include integration tests** for end-to-end scenarios
- **Use descriptive test names** that explain what's being tested
- **Mock external dependencies** when appropriate
- **Test error conditions** as well as success paths

### Documentation

- **Document all public APIs** with rustdoc comments
- **Include usage examples** in documentation
- **Update README** if adding major features
- **Add examples** to the `examples/` directory for significant features

## Testing Guidelines

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test container

# Run integration tests (requires Docker)
cargo test --test integration

# Run with output
cargo test -- --nocapture
```

### Test Categories

1. **Unit Tests**: Test individual functions and modules
2. **Integration Tests**: Test complete workflows with real Docker
3. **Example Tests**: Ensure examples in documentation work
4. **Performance Tests**: Benchmark critical operations

### Writing Good Tests

```rust
#[tokio::test]
async fn test_container_lifecycle() {
    let client = DockerClient::new().await.unwrap();
    
    // Test container creation
    let container_id = ContainerBuilder::new("alpine:latest")
        .name("test-container")
        .run(&client)
        .await
        .unwrap();
    
    // Verify container exists
    let info = client.containers().inspect(&container_id).await.unwrap();
    assert_eq!(info.state.status, ContainerStatus::Running);
    
    // Cleanup
    client.containers().stop(&container_id, None).await.unwrap();
    client.containers().remove(&container_id, RemoveOptions::default()).await.unwrap();
}
```

## Areas for Contribution

### High Priority

- **Bug Fixes**: Issues labeled `bug`
- **Documentation**: Improving docs and examples
- **Test Coverage**: Adding tests for untested code
- **Performance**: Optimizing critical paths

### Medium Priority

- **New Features**: Docker Compose support, additional drivers
- **Developer Experience**: Better error messages, debugging tools
- **Platform Support**: Windows compatibility improvements
- **Ecosystem**: Kubernetes integration, cloud platform helpers

### Advanced Contributions

- **Architecture**: Core design improvements
- **Monitoring**: Advanced metrics and observability
- **Streaming**: Performance optimizations for event/stats streaming
- **Security**: Security audits and improvements

## Reporting Issues

### Bug Reports

When reporting bugs, please include:

- **Docker Wrapper version**
- **Rust version** (`rustc --version`)
- **Docker version** (`docker --version`)
- **Operating system**
- **Minimal reproduction case**
- **Expected vs actual behavior**
- **Relevant logs or error messages**

### Feature Requests

For feature requests, please include:

- **Use case description**
- **Proposed API design** (if applicable)
- **Alternative approaches considered**
- **Willingness to implement** (helps with prioritization)

## Commit Message Guidelines

We follow conventional commits for clear history:

- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `test:` - Test additions or improvements
- `refactor:` - Code refactoring without behavior changes
- `perf:` - Performance improvements
- `chore:` - Maintenance tasks

Examples:
```
feat: add support for Docker Compose services
fix: handle network connection timeout gracefully
docs: add examples for volume management
test: improve container lifecycle test coverage
```

## Recognition

Contributors are recognized in:

- **AUTHORS.md** - All contributors
- **Release notes** - Significant contributions
- **README.md** - Major feature contributors
- **Social media** - Highlighting great contributions

## Getting Help

Need help contributing? We're here to help:

- **GitHub Discussions**: For general questions and brainstorming
- **GitHub Issues**: For specific bugs or feature requests
- **Discord**: [Join our community](https://discord.gg/docker-wrapper) for real-time chat
- **Email**: maintainers@docker-wrapper.org for private matters

## Contribution Ideas for Beginners

Looking for a good first contribution? Try these:

1. **Fix typos** in documentation or comments
2. **Add examples** for existing features
3. **Improve error messages** to be more helpful
4. **Add unit tests** for untested functions
5. **Update dependencies** to latest versions
6. **Improve CI/CD** workflows

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). By participating, you agree to uphold this code.

## License

By contributing to Docker Wrapper, you agree that your contributions will be licensed under the same MIT OR Apache-2.0 dual license as the project.

---

Thank you for contributing to Docker Wrapper! Every contribution, no matter how small, makes a difference. Together, we're building the best Docker management library for Rust!