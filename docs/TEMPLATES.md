# Container Templates

Docker-wrapper provides pre-configured container templates that make it easy to spin up common services with sensible defaults and best practices built in.

## Overview

Templates provide:
- **Sensible Defaults**: Pre-configured settings for production and development use
- **Builder Pattern**: Fluent API for customizing configuration
- **Health Checks**: Automatic health monitoring for services
- **Connection Helpers**: Built-in connection string and URL generation
- **Custom Image Support**: Use your own Docker images and platforms
- **Persistence Options**: Easy volume mounting for data persistence

## Quick Start

Add template features to your `Cargo.toml`:

```toml
[dependencies]
# All templates
docker-wrapper = { version = "0.9", features = ["templates"] }

# Or individual templates
docker-wrapper = { version = "0.9", features = ["template-redis", "template-postgres"] }
```

Basic usage:

```rust
use docker_wrapper::{RedisTemplate, Template};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start Redis with default settings
    let redis = RedisTemplate::new("my-redis");
    let container_id = redis.start().await?;
    println!("Redis started: {}", container_id);
    
    // Clean up
    redis.stop().await?;
    Ok(())
}
```

## Available Templates

### Redis Templates

#### Basic Redis (`RedisTemplate`)
**Feature**: `template-redis`

```rust
use docker_wrapper::{RedisTemplate, Template};

let redis = RedisTemplate::new("my-redis")
    .port(6379)
    .password("secret")
    .memory_limit("256m")
    .with_persistence("redis-data")
    .custom_image("redis", "7-alpine")
    .platform("linux/amd64");

let container_id = redis.start().await?;
```

#### Redis Sentinel (`RedisSentinelTemplate`)
**Feature**: `template-redis`

High-availability Redis with automatic failover:

```rust
use docker_wrapper::RedisSentinelTemplate;

let sentinel = RedisSentinelTemplate::new("ha-redis")
    .master_name("mymaster")
    .num_replicas(2)
    .num_sentinels(3)
    .quorum(2)
    .password("secure-password")
    .with_persistence();

let connection_info = sentinel.start().await?;
println!("Master URL: {}", connection_info.master_url());
println!("Sentinel URLs: {:?}", connection_info.sentinel_urls());
```

#### Redis Cluster (`RedisClusterTemplate`)
**Feature**: `template-redis-cluster`

Sharded Redis cluster with automatic slot allocation:

```rust
use docker_wrapper::{RedisClusterTemplate, RedisClusterConnection};

let cluster = RedisClusterTemplate::new("redis-cluster")
    .num_masters(3)
    .num_replicas(1) // 1 replica per master
    .port_base(7000)
    .password("cluster-password")
    .with_persistence("cluster-data");

let result = cluster.start().await?;

// Get connection info
let conn = RedisClusterConnection::from_template(&cluster);
println!("Cluster nodes: {}", conn.nodes_string());
```

#### Redis Enterprise (`RedisEnterpriseTemplate`)
**Feature**: `template-redis-enterprise`

Redis Enterprise with cluster initialization:

```rust
use docker_wrapper::RedisEnterpriseTemplate;

let enterprise = RedisEnterpriseTemplate::new("redis-enterprise")
    .port(8443) // Admin UI port
    .redis_port(12000) // Database port
    .username("admin")
    .password("enterprise-password")
    .memory_limit("4g");

let connection_info = enterprise.start().await?;
println!("Admin UI: {}", connection_info.admin_url());
```

#### RedisInsight (`RedisInsightTemplate`)
**Feature**: `template-redis`

Redis management and monitoring UI:

```rust
use docker_wrapper::RedisInsightTemplate;

let insight = RedisInsightTemplate::new("redis-insight")
    .port(8001)
    .redis_host("redis")
    .redis_port(6379);

insight.start().await?;
println!("RedisInsight available at http://localhost:8001");
```

### Database Templates

#### PostgreSQL (`PostgresTemplate`)
**Feature**: `template-postgres`

```rust
use docker_wrapper::{PostgresTemplate, Template};

let postgres = PostgresTemplate::new("my-postgres")
    .port(5432)
    .database("myapp")
    .username("appuser")
    .password("apppass")
    .with_persistence("postgres-data")
    .custom_image("postgres", "15-alpine")
    .platform("linux/amd64");

let connection_info = postgres.start().await?;
println!("Connection URL: {}", connection_info.connection_url());
```

**Connection Details:**
- **Host**: `localhost` (or custom host)
- **Port**: `5432` (configurable)
- **URL Format**: `postgresql://username:password@host:port/database`

#### MySQL (`MysqlTemplate`)
**Feature**: `template-mysql`

```rust
use docker_wrapper::{MysqlTemplate, Template};

let mysql = MysqlTemplate::new("my-mysql")
    .port(3306)
    .database("myapp")
    .username("appuser")
    .password("apppass")
    .root_password("rootpass")
    .with_persistence("mysql-data");

let connection_info = mysql.start().await?;
println!("Connection URL: {}", connection_info.connection_url());
```

**Connection Details:**
- **Host**: `localhost`
- **Port**: `3306` (configurable)
- **URL Format**: `mysql://username:password@host:port/database`

#### MongoDB (`MongodbTemplate`)
**Feature**: `template-mongodb`

```rust
use docker_wrapper::{MongodbTemplate, Template};

let mongodb = MongodbTemplate::new("my-mongo")
    .port(27017)
    .database("myapp")
    .username("appuser")
    .password("apppass")
    .with_persistence("mongo-data");

let connection_info = mongodb.start().await?;
println!("Connection URL: {}", connection_info.connection_url());
```

**Connection Details:**
- **Host**: `localhost`
- **Port**: `27017` (configurable)
- **URL Format**: `mongodb://username:password@host:port/database`

### Web Server Templates

#### Nginx (`NginxTemplate`)
**Feature**: `template-nginx`

```rust
use docker_wrapper::{NginxTemplate, Template};

let nginx = NginxTemplate::new("my-nginx")
    .port(80)
    .with_config("/path/to/nginx.conf")
    .serve_directory("/var/www/html")
    .with_ssl_cert("/path/to/cert.pem", "/path/to/key.pem");

nginx.start().await?;
println!("Nginx available at http://localhost:80");
```

## Common Template Features

### Custom Images and Platforms

All templates support custom Docker images and platform specifications:

```rust
let template = RedisTemplate::new("my-service")
    .custom_image("my-registry/redis", "custom-tag")
    .platform("linux/arm64"); // For ARM Macs
```

### Persistence

Enable data persistence with volume mounting:

```rust
let template = PostgresTemplate::new("my-db")
    .with_persistence("my-data-volume");
```

### Memory and Resource Limits

Configure container resource limits:

```rust
let template = RedisTemplate::new("my-redis")
    .memory_limit("512m")
    .cpu_limit("0.5");
```

### Health Checks

Templates automatically configure health checks for supported services:

```rust
let template = PostgresTemplate::new("my-db")
    .health_check_interval("30s")
    .health_check_timeout("10s")
    .health_check_retries(3);
```

## Template Lifecycle Management

### Starting Templates

```rust
// Simple start (returns container ID)
let container_id = template.start().await?;

// Start with connection info (for databases)
let connection_info = template.start().await?;
println!("URL: {}", connection_info.connection_url());
```

### Stopping and Cleanup

```rust
// Stop the container
template.stop().await?;

// Stop and remove the container
template.remove().await?;

// For templates with connection info
connection_info.stop().await?;
```

### Auto-removal

Configure containers to be automatically removed when stopped:

```rust
let template = RedisTemplate::new("temp-redis")
    .auto_remove(); // Container will be removed when stopped
```

## Advanced Usage

### Template Networks

Create isolated networks for multi-container setups:

```rust
let network_name = "my-app-network";

let postgres = PostgresTemplate::new("app-db")
    .network(network_name)
    .start().await?;

let redis = RedisTemplate::new("app-cache")
    .network(network_name)
    .start().await?;
```

### Template Composition

Combine multiple templates for complete application stacks:

```rust
use docker_wrapper::{PostgresTemplate, RedisTemplate, NginxTemplate, Template};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start database
    let db = PostgresTemplate::new("app-db")
        .database("myapp")
        .username("app")
        .password("secret")
        .with_persistence("db-data");
    
    let db_conn = db.start().await?;
    
    // Start cache
    let cache = RedisTemplate::new("app-cache")
        .password("cache-secret")
        .with_persistence("cache-data");
    
    let cache_id = cache.start().await?;
    
    // Start web server
    let web = NginxTemplate::new("app-web")
        .port(80)
        .serve_directory("./static");
    
    web.start().await?;
    
    println!("Application stack started!");
    println!("Database: {}", db_conn.connection_url());
    println!("Web: http://localhost:80");
    
    Ok(())
}
```

## Feature Flags

Templates use granular feature flags for selective compilation:

```toml
[dependencies.docker-wrapper]
version = "0.8"
features = [
    "templates",              # All templates
    "template-redis",         # Basic Redis + Sentinel + Insight
    "template-redis-cluster", # Redis Cluster
    "template-redis-enterprise", # Redis Enterprise
    "template-postgres",      # PostgreSQL
    "template-mysql",         # MySQL
    "template-mongodb",       # MongoDB
    "template-nginx",         # Nginx
]
```

## Best Practices

### 1. Use Descriptive Names
```rust
let user_cache = RedisTemplate::new("user-session-cache");
let order_db = PostgresTemplate::new("order-database");
```

### 2. Configure Resource Limits
```rust
let template = PostgresTemplate::new("my-db")
    .memory_limit("1g")
    .cpu_limit("0.8");
```

### 3. Enable Persistence for Stateful Services
```rust
let template = PostgresTemplate::new("production-db")
    .with_persistence("prod-db-data");
```

### 4. Use Auto-removal for Temporary Containers
```rust
let template = RedisTemplate::new("test-cache")
    .auto_remove(); // Cleaned up automatically
```

### 5. Secure with Passwords
```rust
let template = PostgresTemplate::new("secure-db")
    .password("strong-random-password")
    .root_password("even-stronger-root-password");
```

## Error Handling

Templates provide detailed error information:

```rust
match redis.start().await {
    Ok(container_id) => println!("Started: {}", container_id),
    Err(docker_wrapper::TemplateError::DockerError(e)) => {
        eprintln!("Docker command failed: {}", e);
    }
    Err(docker_wrapper::TemplateError::InvalidConfig(msg)) => {
        eprintln!("Configuration error: {}", msg);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Examples

See the [examples directory](../examples/) for complete working examples:

- [`redis_cluster.rs`](../examples/redis_cluster.rs) - Redis Cluster setup
- [`redis_enterprise_template.rs`](../examples/redis_enterprise_template.rs) - Redis Enterprise
- [`test_sentinel.rs`](../examples/test_sentinel.rs) - Redis Sentinel high availability
- [`template_usage.rs`](../examples/template_usage.rs) - Basic template usage

## Contributing

Template contributions are welcome! See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines on adding new templates.