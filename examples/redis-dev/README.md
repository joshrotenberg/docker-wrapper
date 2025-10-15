# redis-dev

[![Crates.io](https://img.shields.io/crates/v/redis-dev.svg)](https://crates.io/crates/redis-dev)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A powerful CLI tool for Redis developers to quickly spin up various Redis deployments for development, testing, and demos.

## Features

- **Multiple Deployment Types**: Basic Redis, Redis Stack, Redis Cluster, Redis Sentinel, Redis Enterprise
- **YAML Configuration**: Declarative infrastructure-as-code for Redis deployments
- **State Management**: Track and manage all running instances
- **RedisInsight Integration**: Optional GUI for all deployment types
- **Lifecycle Management**: Start, stop, list, clean, and view logs
- **Resource Control**: Memory limits, port management, persistence options
- **Developer-Friendly**: Automatic password generation, connection strings, quick commands

## Installation

### From crates.io

```bash
cargo install redis-dev
```

### From source

```bash
git clone https://github.com/joshrotenberg/redis-dev
cd redis-dev
cargo install --path .
```

## Prerequisites

- **Docker**: Must be installed and running
- **Rust** (for building from source): 1.89.0 or later

## Quick Start

### Basic Redis

```bash
# Start a basic Redis instance
redis-dev basic start --name my-redis --port 6379 --password secret

# Stop it
redis-dev basic stop my-redis

# Get connection info
redis-dev basic info my-redis
```

### Redis Stack (with modules)

```bash
# Start Redis Stack with JSON, Search, TimeSeries, etc.
redis-dev stack start --name my-stack --port 6380

# With RedisInsight GUI
redis-dev stack start --name my-stack --with-insight
```

### Redis Cluster

```bash
# Start a 3-master, 1-replica cluster
redis-dev cluster start --name my-cluster --masters 3 --replicas 1

# With Redis Stack modules
redis-dev cluster start --name my-cluster --masters 3 --stack
```

### Redis Sentinel

```bash
# High-availability setup with Sentinel
redis-dev sentinel start --name my-sentinel --masters 1 --sentinels 3
```

### Redis Enterprise

```bash
# Enterprise cluster with management UI
redis-dev enterprise start --name my-enterprise --nodes 3
```

## YAML Configuration

For complex setups, use YAML configuration files:

```bash
# Generate example YAML files
redis-dev examples

# Deploy from YAML
redis-dev deploy -f examples/basic.yaml

# Deploy multiple instances
redis-dev deploy -f examples/multi-deployment.yaml
```

### Example YAML (basic.yaml)

```yaml
api-version: v1
deployments:
  - name: my-redis
    type: basic
    port: 6379
    persist: true
    memory: "512m"
    with-insight: true
```

### Example YAML (cluster.yaml)

```yaml
api-version: v1
deployments:
  - name: my-cluster
    type: cluster
    masters: 3
    replicas: 1
    port-base: 7000
    persist: true
    memory: "256m"
    stack: false
    with-insight: true
```

### Multi-Deployment YAML

```yaml
api-version: v1
deployments:
  - name: dev-redis
    type: basic
    port: 6379
    persist: true
    
  - name: test-cluster
    type: cluster
    masters: 3
    replicas: 1
    port-base: 7000
    
  - name: sentinel-ha
    type: sentinel
    masters: 1
    sentinels: 3
    redis-port-base: 8000
    sentinel-port-base: 26379
```

## Commands

### Instance Management

```bash
# List all running instances
redis-dev list

# View logs
redis-dev logs my-redis --follow

# Clean up all instances
redis-dev cleanup

# Clean up specific type
redis-dev cleanup --type cluster
```

### Basic Redis

```bash
redis-dev basic start [OPTIONS]
  --name <NAME>          Instance name
  --port <PORT>          Port (default: 6379)
  --password <PASS>      Password (auto-generated if not provided)
  --persist              Enable persistence
  --memory <MEMORY>      Memory limit (e.g., "512m", "2g")
  --with-insight         Start RedisInsight GUI

redis-dev basic stop <NAME>
redis-dev basic info <NAME>
```

### Redis Stack

```bash
redis-dev stack start [OPTIONS]
  --name <NAME>          Instance name
  --port <PORT>          Port (default: 6380)
  --with-insight         Start RedisInsight GUI
  --persist              Enable persistence
```

### Redis Cluster

```bash
redis-dev cluster start [OPTIONS]
  --name <NAME>          Cluster name
  --masters <N>          Number of master nodes (default: 3)
  --replicas <N>         Replicas per master (default: 1)
  --port-base <PORT>     Starting port (default: 7000)
  --stack                Use Redis Stack images
  --with-insight         Start RedisInsight GUI
  --persist              Enable persistence

redis-dev cluster stop <NAME>
redis-dev cluster info <NAME>
```

### Redis Sentinel

```bash
redis-dev sentinel start [OPTIONS]
  --name <NAME>          Sentinel setup name
  --masters <N>          Number of masters (default: 1)
  --sentinels <N>        Number of sentinels (default: 3)
  --redis-port-base <P>  Redis starting port (default: 8000)
  --sentinel-port-base   Sentinel starting port (default: 26379)

redis-dev sentinel stop <NAME>
redis-dev sentinel info <NAME>
```

### Redis Enterprise

```bash
redis-dev enterprise start [OPTIONS]
  --name <NAME>          Enterprise cluster name
  --nodes <N>            Number of nodes (default: 3)
  --ui-port <PORT>       Management UI port (default: 8443)
  --db-port <PORT>       Database port (default: 12000)

redis-dev enterprise stop <NAME>
redis-dev enterprise info <NAME>
```

## Configuration and State

redis-dev stores instance state in `~/.redis-dev/instances.json`. This allows you to:

- Track all running instances
- Resume management after CLI restarts
- Share connection information across terminal sessions

## RedisInsight Integration

Add `--with-insight` to any deployment to start RedisInsight:

```bash
redis-dev basic start --name my-redis --with-insight
```

RedisInsight will be available at `http://localhost:5540` (or custom port with `--insight-port`)

## Use Cases

### Development

```bash
# Quick Redis for development
redis-dev basic start --name dev

# Access connection string
redis-dev basic info dev
```

### Testing

```bash
# Test with cluster
redis-dev cluster start --name test-cluster --masters 3 --replicas 1

# Run your tests
pytest

# Clean up
redis-dev cleanup --force
```

### Demos

```bash
# Complex demo setup via YAML
redis-dev deploy -f demo-setup.yaml

# Show everything running
redis-dev list
```

### Learning Redis

```bash
# Try Redis Stack modules
redis-dev stack start --name learn-stack --with-insight

# Explore via RedisInsight GUI
# http://localhost:5540
```

## Tips

1. **Auto-generated passwords**: If you don't specify `--password`, redis-dev generates a secure random password
2. **Connection strings**: Use `redis-dev <type> info <name>` to get full connection information
3. **Port conflicts**: redis-dev automatically handles port allocation to avoid conflicts
4. **Cleanup**: Always run `redis-dev cleanup` when done to free resources
5. **Persistence**: Add `--persist` to keep data between restarts

## Troubleshooting

### Docker not found

```bash
# Ensure Docker is running
docker ps

# If not installed, visit https://docker.com
```

### Port already in use

```bash
# Check what's using the port
lsof -i :<PORT>

# Use a different port
redis-dev basic start --port 6380
```

### Instance won't start

```bash
# Check logs
redis-dev logs <instance-name>

# Clean up and retry
redis-dev cleanup
```

## Architecture

redis-dev uses [docker-wrapper](https://github.com/joshrotenberg/docker-wrapper) for Docker orchestration and provides:

- **Template-based deployments**: Pre-configured Redis setups
- **Networking**: Automatic network creation for multi-container setups
- **Health checks**: Waits for containers to be ready
- **State persistence**: Tracks instances across CLI sessions

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Acknowledgments

Built with [docker-wrapper](https://github.com/joshrotenberg/docker-wrapper), a comprehensive Docker CLI wrapper for Rust.

## Support

- Report issues: [GitHub Issues](https://github.com/joshrotenberg/redis-dev/issues)
- Discussion: [GitHub Discussions](https://github.com/joshrotenberg/redis-dev/discussions)
