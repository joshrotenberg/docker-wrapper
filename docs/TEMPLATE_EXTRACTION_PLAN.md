# Template Extraction Plan

## Overview

This document outlines the plan for extracting the Docker template system from `docker-wrapper` into a standalone crate, potentially named `docker-templates` or `container-templates`. This extraction is motivated by the significant interest shown in PR #140 and the value these templates provide as a distinct feature.

## Motivation

1. **Focused Purpose**: Templates serve a different use case than the core Docker CLI wrapper
2. **Independent Evolution**: Templates can evolve faster without affecting the core library
3. **Reduced Dependencies**: Users who only want templates don't need the full wrapper
4. **Community Interest**: PR #140 demonstrates demand for template-based container management tools

## Architecture Design

### Core Components to Extract

```
docker-templates/
├── src/
│   ├── lib.rs              # Main library exports
│   ├── template.rs         # Core Template trait and base types
│   ├── error.rs           # Template-specific errors
│   ├── health.rs          # Health checking and readiness probes
│   ├── connection.rs      # Connection string builders
│   ├── redis/            
│   │   ├── mod.rs         # Redis module exports
│   │   ├── basic.rs       # Basic Redis template
│   │   ├── stack.rs       # Redis Stack template
│   │   ├── cluster.rs     # Redis Cluster template
│   │   ├── sentinel.rs    # Redis Sentinel template
│   │   └── enterprise.rs  # Redis Enterprise template
│   ├── database/
│   │   ├── mod.rs
│   │   ├── postgres.rs    # PostgreSQL template
│   │   ├── mysql.rs       # MySQL template
│   │   └── mongodb.rs     # MongoDB template
│   └── web/
│       ├── mod.rs
│       └── nginx.rs       # Nginx template
```

### Dependency Structure

The extracted crate would depend on:
- `docker-wrapper` (for Docker command execution)
- `async-trait` (for async trait definitions)
- `tokio` (for async runtime)
- `serde` / `serde_json` (for configuration)
- `thiserror` (for error handling)
- `reqwest` (optional, for Redis Enterprise bootstrap)

### Key Features to Preserve

1. **Builder Pattern API**: Fluent configuration interface
2. **Health Checking**: `wait_for_ready()` with custom implementations
3. **Connection Management**: Connection string builders for each database
4. **Resource Limits**: Memory, CPU, and other constraints
5. **Networking**: Container network support
6. **Persistence**: Volume management
7. **Custom Images**: Support for custom Docker images and platforms

## Implementation Steps

### Phase 1: Preparation (In docker-wrapper)
- [x] Enhance template system with `wait_for_ready()` methods
- [x] Add comprehensive integration tests
- [x] Create showcase examples
- [ ] Document all template APIs thoroughly
- [ ] Ensure feature flag isolation

### Phase 2: Extraction
1. Create new repository/crate `docker-templates`
2. Copy template module structure
3. Update imports and dependencies
4. Add docker-wrapper as a dependency
5. Migrate tests and examples

### Phase 3: Integration
1. Replace docker-wrapper's template module with dependency on docker-templates
2. Maintain backward compatibility with re-exports
3. Update documentation and examples
4. Deprecation notices for old imports

### Phase 4: Enhancement
1. Add more templates (Elasticsearch, Kafka, RabbitMQ, etc.)
2. Implement template composition (multi-container setups)
3. Add template registry/discovery
4. Create CLI tool for template management

## API Design

### Core Template Trait

```rust
#[async_trait]
pub trait Template: Send + Sync {
    fn name(&self) -> &str;
    fn config(&self) -> &TemplateConfig;
    fn config_mut(&mut self) -> &mut TemplateConfig;
    
    async fn start(&self) -> Result<String>;
    async fn start_and_wait(&self) -> Result<String>;
    async fn stop(&self) -> Result<()>;
    async fn remove(&self) -> Result<()>;
    async fn is_running(&self) -> Result<bool>;
    async fn wait_for_ready(&self) -> Result<()>;
    async fn logs(&self, follow: bool, tail: Option<&str>) -> Result<LogOutput>;
    async fn exec(&self, command: Vec<&str>) -> Result<ExecOutput>;
}
```

### Template Builder Pattern

```rust
// Example usage after extraction
use docker_templates::{RedisTemplate, Template};

let redis = RedisTemplate::builder("my-redis")
    .port(6379)
    .password("secure")
    .with_persistence("redis-data")
    .memory_limit("256m")
    .build();

let container_id = redis.start_and_wait().await?;
```

## Benefits of Extraction

### For Users
- **Simpler API**: Focus on templates without Docker CLI complexity
- **Better Documentation**: Dedicated docs for template usage
- **Faster Iteration**: More frequent releases with new templates
- **Lighter Dependencies**: Only what's needed for templates

### For Maintainers
- **Clearer Scope**: Separate concerns between CLI wrapper and templates
- **Easier Testing**: Isolated template tests
- **Community Contributions**: Lower barrier for adding new templates
- **Independent Versioning**: Templates can have their own version scheme

## Migration Path

### For Existing Users

```rust
// Before (in docker-wrapper)
use docker_wrapper::{RedisTemplate, Template};

// After (with extracted crate)
use docker_templates::{RedisTemplate, Template};
// Or with compatibility layer
use docker_wrapper::templates::{RedisTemplate, Template};
```

### Compatibility Strategy

1. **Version 1.0**: Full extraction with docker-wrapper re-exports
2. **Version 1.1**: Deprecation warnings for docker-wrapper imports
3. **Version 2.0**: Remove templates from docker-wrapper entirely

## Example Projects Using Extracted Templates

### 1. Redis Development Tool (from PR #140)
```rust
// redis-dev CLI tool
use docker_templates::redis::{RedisTemplate, RedisClusterTemplate};
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    redis_type: RedisType,
}

enum RedisType {
    Basic { name: String },
    Cluster { nodes: u8 },
}
```

### 2. Database Testing Framework
```rust
// test-containers-rs integration
use docker_templates::{PostgresTemplate, Template};
use test_containers::TestContainer;

#[test]
async fn test_with_postgres() {
    let postgres = PostgresTemplate::new("test-db")
        .database("test")
        .into_test_container();
    
    // Run tests with postgres
}
```

### 3. Local Development Environment
```rust
// dev-env tool
use docker_templates::{Template, TemplateGroup};

let dev_env = TemplateGroup::new()
    .add_redis("cache")
    .add_postgres("main-db")
    .add_nginx("proxy")
    .with_network("dev-network");

dev_env.start_all().await?;
```

## Success Metrics

- **Adoption**: Number of downloads on crates.io
- **Community**: Contributors adding new templates
- **Integration**: Other projects depending on docker-templates
- **Performance**: Template startup time < 2 seconds
- **Reliability**: 99% test success rate

## Timeline

- **Week 1-2**: Finalize template enhancements in docker-wrapper
- **Week 3-4**: Create docker-templates repository and initial structure
- **Week 5-6**: Migrate code and tests
- **Week 7-8**: Integration testing and documentation
- **Week 9-10**: Beta release and community feedback
- **Week 11-12**: Stable 1.0 release

## Risks and Mitigation

### Risk: Breaking Changes
**Mitigation**: Maintain compatibility layer in docker-wrapper for 2 major versions

### Risk: Maintenance Burden
**Mitigation**: Establish clear contribution guidelines and automated testing

### Risk: Feature Creep
**Mitigation**: Strict scope definition - focus only on container templates

## Conclusion

Extracting the template system into a standalone crate will benefit both users and maintainers by providing a focused, well-documented solution for container template management. The extraction plan ensures smooth migration while opening opportunities for rapid innovation in the template ecosystem.