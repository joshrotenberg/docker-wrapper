# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of docker-wrapper
- Complete container lifecycle management
- Comprehensive image operations with registry support
- Advanced network management with custom drivers
- Full volume management with multiple backends
- Real-time Docker event monitoring and streaming
- Live container statistics with historical aggregation
- Type-safe APIs with builder patterns
- Production-ready error handling and resource cleanup

### Features by Phase

#### Phase 1: Foundation
- Docker client with automatic binary detection
- Process executor with async streaming support
- Core type system with newtype wrappers
- Comprehensive error handling with thiserror
- Basic container operations (run, stop, remove)

#### Phase 2: Container Lifecycle Management
- Advanced container builder with fluent API
- Environment variable and volume mounting
- Network attachment and resource limits
- Container execution with streaming I/O
- Health checking with multiple strategies
- Log streaming with filtering options
- Port management with dynamic allocation

#### Phase 3: Image, Network & Volume Management
- **Image Management**:
  - Image pulling with progress tracking
  - Image building from Dockerfiles
  - Image tagging, inspection, and removal
  - Registry authentication support
  - Image history and export/import operations

- **Network Management**:
  - Network creation with multiple drivers (bridge, overlay, macvlan)
  - Container network connection/disconnection
  - IPAM configuration with custom subnets
  - Network inspection and cleanup operations

- **Volume Management**:
  - Volume creation with multiple drivers
  - Volume mounting specifications
  - Volume inspection and cleanup
  - Usage statistics and batch operations

#### Phase 4: Advanced Features & Monitoring
- **Event Monitoring**:
  - Real-time Docker event streaming
  - Comprehensive event filtering (type, time, labels)
  - Container lifecycle event handling
  - Event waiting patterns for synchronization

- **Statistics Monitoring**:
  - Real-time container resource metrics
  - CPU, memory, network, and disk I/O tracking
  - Historical data aggregation with time windows
  - Resource threshold monitoring and alerts
  - System-wide Docker statistics

### Technical Achievements
- Zero unsafe code with memory safety guarantees
- Async-first design with tokio integration
- Streaming architecture with bounded memory usage
- Type-safe error handling with comprehensive context
- Resource cleanup automation with RAII patterns

## [0.1.0] - 2025-07-24

### Added
- Initial public release
- Complete Docker ecosystem management
- Production-ready monitoring capabilities
- Comprehensive documentation and examples

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for information on how to contribute to this project.