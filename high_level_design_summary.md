# Docker-Wrapper: Templates & Container Groups Design Summary

## Executive Overview

Docker-wrapper introduces a revolutionary three-tier architecture that bridges the gap between single container management and complex orchestration platforms. Our hybrid approach provides the right tool for each complexity level while maintaining API consistency and natural upgrade paths.

## Three-Tier Architecture

### Tier 1: Templates (Single Container)
**Purpose**: Type-safe, domain-specific container management with sensible defaults

**Key Features**:
- Single container execution with immediate feedback
- Domain-specific configuration methods (Redis persistence, Postgres databases)
- Handle-based operations with service-specific methods
- Zero-configuration defaults that "just work"

**Usage Pattern**:
```rust
// Immediate single service deployment
let redis = Redis::default()
    .version("7.2")
    .persistence(true)
    .password("secret")
    .start(&docker).await?;

println!("Connect: {}", redis.connection_string());
redis.ping().await?; // Service-specific operations
```

**Target Users**: Developers testing single services, simple applications, quick prototyping

---

### Tier 2: Container Groups (Multi-Container Orchestration)
**Purpose**: Programmatic multi-container orchestration with shared resources and custom logic

**Key Features**:
- Multiple containers with dependency resolution
- Shared networks and volumes (automatic creation and lifecycle)
- Environment variable inheritance and sharing
- Custom post-startup initialization hooks
- Programmatic logic (conditionals, loops, runtime decisions)
- Health check coordination

**Usage Pattern**:
```rust
let mut group = ContainerGroup::new("myapp");

// Add services using same templates
group.add("cache", Redis::default().persistence(true));
group.add("database", Postgres::default().database("myapp"));

// Runtime logic (impossible with static YAML)
if config.enable_monitoring {
    group.add("metrics", Prometheus::default().port(9090));
}

// Custom orchestration
group.post_startup(|handles| async move {
    handles.get("database").wait_healthy().await?;
    handles.exec("database", vec!["psql", "-c", "CREATE TABLE users"]).await?;
    Ok(())
});

let group_handle = group.start(&docker).await?;
```

**Target Users**: Multi-service applications, development environments, custom deployment logic

---

### Tier 3: Dynamic Compose (Full Compose + Runtime Logic)
**Purpose**: Generate docker-compose.yml files dynamically with full Compose features

**Key Features**:
- All Docker Compose capabilities (secrets, configs, deploy constraints)
- Runtime YAML generation with dynamic service inclusion
- Production-ready orchestration via Docker Compose
- Familiar Compose tooling compatibility
- Advanced networking and resource management

**Usage Pattern**:
```rust
let mut compose = DynamicCompose::new("production-app");

// Same templates, different execution context
compose.add_service("cache", Redis::default().memory_limit("512M"));
compose.add_service("database", Postgres::default().init_script("./schema.sql"));

// Runtime decisions (generates different YAML)
if env == "production" {
    compose.add_service("proxy", Nginx::default().ssl_certificates(vec!["prod_cert"]));
    compose.add_secret("api_key", SecretSource::Vault("prod/api_key"));
} else {
    compose.add_service("proxy", Nginx::default().self_signed_ssl());
}

// Generate and execute
let handle = compose.start(&docker).await?;
```

**Target Users**: Production deployments, complex resource requirements, teams using existing Compose workflows

## Dual-Mode Template Architecture

### Template Intelligence
Templates serve as the universal building blocks across all three tiers through a dual-mode design:

**Mode 1: Direct Execution**
```rust
// Templates run standalone via CLI wrapper
let redis = Redis::default().start(&docker).await?;
```

**Mode 2: Service Definition Generation**
```rust
// Same template generates Compose service
let compose_service = redis.to_compose_service();
// Automatically includes: image, ports, volumes, health checks, environment
```

### Template Benefits
- **Single Definition**: Write template once, use in multiple contexts
- **Domain Expertise**: Redis template knows about persistence, Postgres about databases
- **Sensible Defaults**: Zero-configuration to running container
- **Type Safety**: Compile-time validation of container configuration
- **Handle-Based Operations**: Service-specific methods like `redis.connection_string()`

## Hybrid Orchestration Strategy

### Separate APIs with Shared Export
We chose the hybrid approach over unified or completely separate APIs:

**Container Groups**: Custom orchestration focus
- Direct CLI wrapper execution with custom coordination logic
- Post-startup hooks and dependency management
- Programmatic orchestration patterns

**Dynamic Compose**: Compose-first workflows  
- Full Compose feature support (secrets, configs, deploy constraints)
- Professional YAML generation with runtime logic
- Battle-tested Docker Compose execution

**Shared Capability**: Both implement `ToCompose` trait for export
```rust
// Any orchestration approach can export to Compose
group.export_compose("docker-compose.yml").await?;
compose.export_compose("docker-compose.yml").await?;
```

### Benefits of Hybrid Approach
1. **Clear Mental Model**: Users know exactly what each API provides
2. **Implementation Clarity**: No backend switching complexity
3. **Natural Migration**: Easy conversion between approaches as needs change
4. **Export Everywhere**: Always possible to generate Compose files

## Export & Integration Capabilities

### Universal Export Functionality
Every tier can generate docker-compose.yml files:

```rust
// Templates can export (single service)
redis.export_compose("redis-service.yml").await?;

// Container Groups can export (loses custom orchestration)  
group.export_compose("docker-compose.yml").await?;

// Dynamic Compose exports with full fidelity
compose.export()
    .production_mode()
    .with_comments(false)
    .to_file("docker-compose.prod.yml").await?;
```

### Integration with Existing Workflows
- **CI/CD Integration**: Generate Compose files in build pipelines
- **Team Collaboration**: Rust developers generate, ops teams deploy
- **Environment Variants**: Same code, different output files
- **Educational/Debugging**: Preview generated configurations

## Competitive Advantages

### Unique Market Position
Docker-wrapper is the only Rust library providing:
- CLI-familiar API with type safety and async/await
- Production-ready templates with domain-specific methods
- Multi-container orchestration with custom logic
- Dynamic Compose generation with runtime decisions
- Three-tier architecture serving different complexity levels

### vs. Existing Libraries
- **bollard**: Too low-level, requires Docker API knowledge
- **testcontainers-rs**: Testing-only, not production-ready
- **docktopus**: Early-stage, parsing-focused
- **Static Compose**: No runtime logic or type safety

## Implementation Strategy

### Phase 1: Templates (Tier 1) - 2-4 weeks
- Core templates: Redis, Postgres, Nginx, MySQL
- Handle pattern with service-specific methods
- Basic export capability via `ToCompose` trait

### Phase 2: Container Groups (Tier 2) - 4-6 weeks  
- CLI wrapper orchestration with network/volume management
- Custom post-startup hooks and dependency resolution
- Enhanced export with orchestration metadata

### Phase 3: Dynamic Compose (Tier 3) - 2-3 weeks
- Template → Compose service conversion
- YAML generation with production optimizations  
- Full Compose feature integration

### Total Timeline: 8-13 weeks for complete three-tier architecture

## Success Metrics

### Technical Metrics
- **API Consistency**: Same templates work across all tiers
- **Export Fidelity**: Generated Compose files are production-ready
- **Performance**: Container Groups achieve target orchestration speed
- **Type Safety**: Compile-time validation prevents runtime errors

### Adoption Metrics  
- **Tier Progression**: Users naturally upgrade between tiers
- **Export Usage**: High adoption of Compose generation features
- **Template Popularity**: Core templates see widespread usage
- **Community Growth**: Third-party template ecosystem development

## Strategic Vision

Docker-wrapper becomes the **universal Docker interface for Rust** by:
- Serving every use case from single containers to production deployment
- Providing familiar CLI-style API with modern type safety
- Enabling natural progression as application complexity grows  
- Bridging the gap between development and production workflows

This three-tier hybrid architecture positions docker-wrapper as the definitive choice for Docker integration in Rust applications, eliminating the need to choose between different specialized libraries.