# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.3](https://github.com/joshrotenberg/docker-wrapper/compare/v0.2.2...v0.2.3) (2025-08-23)


### Bug Fixes

* Add issues permission and skip-labeling for release-please ([#74](https://github.com/joshrotenberg/docker-wrapper/issues/74)) ([a19fb78](https://github.com/joshrotenberg/docker-wrapper/commit/a19fb78cb2250dacb956b18ff5ad292dff4580b2))

## [0.2.2](https://github.com/joshrotenberg/docker-wrapper/compare/v0.2.1...v0.2.2) (2025-08-22)


### Features

* Add debugging and reliability features ([#51](https://github.com/joshrotenberg/docker-wrapper/issues/51)) ([#71](https://github.com/joshrotenberg/docker-wrapper/issues/71)) ([357a3a1](https://github.com/joshrotenberg/docker-wrapper/commit/357a3a130565317cffe959fc4fe118bb94680ab6))
* Add system cleanup and maintenance commands ([#50](https://github.com/joshrotenberg/docker-wrapper/issues/50)) ([#70](https://github.com/joshrotenberg/docker-wrapper/issues/70)) ([5e00e15](https://github.com/joshrotenberg/docker-wrapper/commit/5e00e157e64ce370bcb9707fae997c63c4c62fd3))


### Performance Improvements

* Optimize CI workflow runtime ([#72](https://github.com/joshrotenberg/docker-wrapper/issues/72)) ([f86dd9e](https://github.com/joshrotenberg/docker-wrapper/commit/f86dd9eea2fcdbd456e56f960a152ad104ece979))


### Documentation

* Fix bollard code examples and imports in comparison guide ([#68](https://github.com/joshrotenberg/docker-wrapper/issues/68)) ([52df4f5](https://github.com/joshrotenberg/docker-wrapper/commit/52df4f57f84835f90bca34b188fbe264a59b5bf7))

## [0.2.1](https://github.com/joshrotenberg/docker-wrapper/compare/v0.2.0...v0.2.1) (2025-08-22)


### Features

* Add container lifecycle commands (stop, start, restart) ([#32](https://github.com/joshrotenberg/docker-wrapper/issues/32)) ([0406d95](https://github.com/joshrotenberg/docker-wrapper/commit/0406d95a18ae9ff04fd3cbc99611dc6cd263d20f))
* Add Docker Compose support with feature gating ([#57](https://github.com/joshrotenberg/docker-wrapper/issues/57)) ([d800225](https://github.com/joshrotenberg/docker-wrapper/commit/d80022577b6f68caad7f67b89d3e4e7f604dd3e9)), closes [#36](https://github.com/joshrotenberg/docker-wrapper/issues/36)
* Add Docker network and volume management support ([#64](https://github.com/joshrotenberg/docker-wrapper/issues/64)) ([c69070f](https://github.com/joshrotenberg/docker-wrapper/commit/c69070fa7227708e2820c6f4eca9bb0876c22fb8))
* Add platform detection and runtime abstraction ([#65](https://github.com/joshrotenberg/docker-wrapper/issues/65)) ([ea09fdd](https://github.com/joshrotenberg/docker-wrapper/commit/ea09fdd095f2e50d1523810d55fedae8d3835fc9)), closes [#44](https://github.com/joshrotenberg/docker-wrapper/issues/44)
* Add streaming support for Docker command output ([#60](https://github.com/joshrotenberg/docker-wrapper/issues/60)) ([4642324](https://github.com/joshrotenberg/docker-wrapper/commit/464232457c9056ec876c95d2271316b0fe318d74))
* Complete 100% Docker CLI coverage implementation ([#54](https://github.com/joshrotenberg/docker-wrapper/issues/54)) ([b3d1f35](https://github.com/joshrotenberg/docker-wrapper/commit/b3d1f35c680e3ea4d0b11bd3b4c77cb04aff9133))


### Bug Fixes

* Update git-cliff-action to resolve Debian buster repository issues ([#66](https://github.com/joshrotenberg/docker-wrapper/issues/66)) ([51ba6b2](https://github.com/joshrotenberg/docker-wrapper/commit/51ba6b2733e7f60e0eb9f9138256176ce0b53491)), closes [#61](https://github.com/joshrotenberg/docker-wrapper/issues/61)


### Documentation

* Add comprehensive Docker library comparison guide ([#67](https://github.com/joshrotenberg/docker-wrapper/issues/67)) ([eb59742](https://github.com/joshrotenberg/docker-wrapper/commit/eb597422ba50fc34aca00c72bb6589a81b26907a))
* Comprehensive documentation improvements ([#62](https://github.com/joshrotenberg/docker-wrapper/issues/62)) ([63df3b2](https://github.com/joshrotenberg/docker-wrapper/commit/63df3b277d38a6a44a342f9fb1ac8b83a0d8babf))

## [0.2.0](https://github.com/joshrotenberg/docker-wrapper/compare/v0.1.0...v0.2.0) (2025-07-27)


### ⚠ BREAKING CHANGES

* Initial public release
* Initial public release
* None - pure test additions
* Remove 'phase' terminology throughout codebase
* Remove volume_tmp method reference from tests
* Context system now uses structured ADRs instead of bespoke markdown files
* Remove competitive analysis from public documentation

### Features

* add comprehensive ContainerManager tests - major coverage improvement ([53f653c](https://github.com/joshrotenberg/docker-wrapper/commit/53f653c421cf41d79a58c84ca069e7a5a385dee1))
* add comprehensive image operations testing infrastructure ([180d553](https://github.com/joshrotenberg/docker-wrapper/commit/180d553569e12e64079081a680cb20f0b8ba586c))
* add dependency management, fix tests, and improve CI caching ([219090a](https://github.com/joshrotenberg/docker-wrapper/commit/219090a18671d45d9922b2b7fdeabb0f18874b6f))
* add release-please automation and refactor context system ([d2e911f](https://github.com/joshrotenberg/docker-wrapper/commit/d2e911fc095ee6ff27b9948810ccd97aca7a896f))
* fix all image operations and enable comprehensive testing ✅ ([e7ab93a](https://github.com/joshrotenberg/docker-wrapper/commit/e7ab93ab83d97defa77653d5c8a66d37686cd0e9))
* significantly improve test coverage and remove phase naming ([a5e0325](https://github.com/joshrotenberg/docker-wrapper/commit/a5e0325b17607837c6b15b7866e78892c4d2af21))


### Bug Fixes

* remove duplicate ImageRef from types.rs to fix compilation ([78beff5](https://github.com/joshrotenberg/docker-wrapper/commit/78beff5bb79dac03e5a688510f290d0a544436c6))
* update Cargo.toml example names and fix unused variable warning ([5023ce7](https://github.com/joshrotenberg/docker-wrapper/commit/5023ce761ea5aa4be838eba2547008dcff49f53e))
* Update release-please changelog type ([#29](https://github.com/joshrotenberg/docker-wrapper/issues/29)) ([9ec4185](https://github.com/joshrotenberg/docker-wrapper/commit/9ec418514f93e386fea006709785277331207dc7))
* Update release-please workflow ([#28](https://github.com/joshrotenberg/docker-wrapper/issues/28)) ([345ff3b](https://github.com/joshrotenberg/docker-wrapper/commit/345ff3b475f7b33a13881a526872bfdcd5b65db2))


### Documentation

* add comprehensive Docker feature and test coverage matrix ([26e8d2e](https://github.com/joshrotenberg/docker-wrapper/commit/26e8d2e3fcff606d31accb84058c12b724620e4a))
* create focused test-redis command implementation matrix ([040bb85](https://github.com/joshrotenberg/docker-wrapper/commit/040bb85f824e400ce4e64ea76c5c3b7b22fd639a))
* prepare for 0.1.0 release - update dates, remove competitive analysis, reduce emoji usage ([6758490](https://github.com/joshrotenberg/docker-wrapper/commit/6758490abfdb35241d3224a8d2e35347f9565e53))

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
