# Changelog

All notable changes to redis-dev will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of redis-dev
- Basic Redis deployment support
- Redis Stack deployment support
- Redis Cluster deployment with automatic initialization
- Redis Sentinel high-availability setup
- Redis Enterprise cluster support
- YAML configuration for declarative deployments
- `deploy` command to deploy from YAML files
- `examples` command to generate example YAML configurations
- State management in `~/.redis-dev/instances.json`
- RedisInsight integration for all deployment types
- Lifecycle commands: start, stop, info, list, cleanup, logs
- Automatic password generation
- Port management and conflict avoidance
- Memory and resource limits
- Persistence support with named volumes
- Comprehensive integration tests for all deployment types

### Documentation
- Comprehensive README with usage examples
- DESIGN.md with architecture and implementation details
- Example YAML files for all deployment types
- Multi-deployment configuration examples

## [0.1.0] - TBD

Initial release.
