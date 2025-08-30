# Redis Developer CLI Tool - Design Document

## Overview
A comprehensive CLI tool for Redis developers to quickly spin up various Redis deployment types for testing, development, and demos.

## Key Features

### 1. Deployment Types
- **Basic Redis**: Simple single-node Redis instance
- **Redis Stack**: Redis with modules (JSON, Search, TimeSeries, Graph, Bloom)
- **Redis Sentinel**: High-availability setup with master-replica and sentinels
- **Redis Cluster**: Multi-node sharded cluster
- **Redis Enterprise**: Enterprise-grade cluster with management UI

### 2. Common Features Across All Types
- **Redis Insight Integration**: Optional GUI for all deployment types
- **redis-cli Access**: Easy shell access to any instance
- **Persistence**: Optional data persistence with named volumes
- **Resource Management**: Memory and CPU limits
- **Networking**: Custom networks for multi-container setups
- **State Management**: Track all running instances

### 3. Configuration Options

#### CLI Arguments
```bash
# Basic Redis
redis-dev basic start --name my-redis --port 6379 --password secret --persist --shell

# Redis Stack with Insight
redis-dev stack start --demo-bundle --insight --shell

# Redis Cluster
redis-dev cluster start --masters 3 --replicas 1 --insight

# Redis Sentinel
redis-dev sentinel start --masters 1 --sentinels 3 --quorum 2

# Redis Enterprise
redis-dev enterprise start --nodes 3 --create-db mydb --db-memory 1gb
```

#### YAML Configuration
```yaml
# redis-dev.yml
version: '1.0'
deployments:
  - name: dev-redis
    type: basic
    port: 6379
    password: ${REDIS_PASSWORD}
    persist: true
    
  - name: test-cluster
    type: cluster
    masters: 3
    replicas: 1
    insight: true
    insight_port: 8001
    
  - name: prod-like
    type: enterprise
    nodes: 3
    databases:
      - name: api-cache
        memory: 1gb
        replication: true
      - name: session-store
        memory: 500mb
        persistence: aof
```

### 4. Command Structure

```
redis-dev
├── basic
│   ├── start      # Start a basic Redis instance
│   ├── stop       # Stop instance
│   ├── restart    # Restart instance
│   ├── info       # Show connection info
│   └── shell      # Connect to redis-cli
├── stack
│   ├── start      # Start Redis Stack
│   ├── stop       # Stop instance
│   ├── info       # Show info including modules
│   └── shell      # Connect to redis-cli
├── cluster
│   ├── start      # Start Redis Cluster
│   ├── stop       # Stop cluster
│   ├── info       # Show cluster topology
│   ├── rebalance  # Rebalance cluster slots
│   └── shell      # Connect to any node
├── sentinel
│   ├── start      # Start Sentinel setup
│   ├── stop       # Stop all components
│   ├── info       # Show Sentinel status
│   ├── failover   # Trigger manual failover
│   └── shell      # Connect to master
├── enterprise
│   ├── start      # Start Enterprise cluster
│   ├── stop       # Stop cluster
│   ├── info       # Show cluster info
│   ├── create-db  # Create a database
│   ├── list-dbs   # List databases
│   └── ui         # Open management UI
├── list           # List all running instances
├── cleanup        # Clean up instances
├── logs           # View logs
├── status         # Show status of all deployments
├── config
│   ├── load       # Load from YAML
│   ├── export     # Export current state
│   └── import     # Import configuration
└── shell          # Quick connect to any instance
```

### 5. Redis Insight Integration

All deployment types support optional Redis Insight:
- Automatic connection configuration
- Port management to avoid conflicts
- Shared network for connectivity
- Pre-configured connection profiles

### 6. Shell Management

Enhanced redis-cli integration:
- Auto-detect connection parameters
- Support for cluster mode (`-c`)
- Password handling
- Custom commands execution
- Interactive and non-interactive modes

### 7. State Management

Track instance state in `~/.config/redis-dev/`:
- `instances.json`: Active instances
- `config.yml`: User preferences
- `history.log`: Command history
- `connections/`: Saved connection profiles

### 8. Redis Enterprise Features

Special handling for Enterprise:
- Cluster bootstrapping via API
- Database creation and management
- Memory allocation and limits
- Replication configuration
- Module enablement
- User management
- TLS/SSL configuration

### 9. Advanced Features

- **Import/Export**: Save and share configurations
- **Templates**: Pre-defined deployment templates
- **Monitoring**: Basic metrics and health checks
- **Backup/Restore**: Data backup capabilities
- **Migration**: Move data between instances
- **Profiles**: Named configuration profiles

## Implementation Priority

1. **Phase 1**: Core functionality
   - [x] Basic Redis
   - [x] Redis Stack
   - [x] Redis Cluster
   - [ ] Redis Sentinel
   - [ ] Redis Enterprise

2. **Phase 2**: Enhanced features
   - [ ] YAML configuration
   - [ ] Redis Insight for all types
   - [ ] Shell management
   - [ ] Status monitoring

3. **Phase 3**: Advanced capabilities
   - [ ] Import/Export
   - [ ] Templates
   - [ ] Backup/Restore
   - [ ] Migration tools

## Technical Architecture

```
redis-dev/
├── src/
│   ├── main.rs           # Entry point
│   ├── cli.rs            # CLI definitions
│   ├── config.rs         # Configuration management
│   ├── commands/
│   │   ├── basic.rs      # Basic Redis commands
│   │   ├── stack.rs      # Redis Stack commands
│   │   ├── cluster.rs    # Cluster commands
│   │   ├── sentinel.rs   # Sentinel commands
│   │   ├── enterprise.rs # Enterprise commands
│   │   ├── insight.rs    # Insight management
│   │   └── shell.rs      # Shell integration
│   ├── yaml/
│   │   ├── parser.rs     # YAML parsing
│   │   └── validator.rs  # Configuration validation
│   └── utils/
│       ├── docker.rs     # Docker helpers
│       ├── network.rs    # Network management
│       └── ports.rs      # Port allocation
```

## Error Handling

- Clear error messages with recovery suggestions
- Rollback on partial failures
- Cleanup of orphaned resources
- Retry logic for transient failures

## Testing Strategy

- Unit tests for each command module
- Integration tests with Docker
- End-to-end deployment scenarios
- Performance benchmarks
- Cross-platform testing (Linux, macOS, Windows via WSL2)