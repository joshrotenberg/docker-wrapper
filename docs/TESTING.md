# Testing with docker-wrapper

This guide demonstrates best practices for using docker-wrapper in your test suites, providing patterns and examples for common testing scenarios.

## Table of Contents

- [Quick Start](#quick-start)
- [Testing Patterns](#testing-patterns)
- [Test Helpers](#test-helpers)
- [Framework Integration](#framework-integration)
- [CI/CD Integration](#cicd-integration)
- [Common Scenarios](#common-scenarios)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Quick Start

### Basic Test Setup

```rust
use docker_wrapper::{DockerCommand, RunCommand, RmCommand};
use tokio;

#[tokio::test]
async fn test_with_redis() {
    // Use unique names to avoid conflicts in parallel tests
    let container_name = format!("test-redis-{}", uuid::Uuid::new_v4());
    
    // Start container with auto-cleanup
    let output = RunCommand::new("redis:7-alpine")
        .name(&container_name)
        .detach()
        .remove()  // Auto-remove when stopped
        .execute()
        .await
        .expect("Failed to start Redis");
    
    // Your test logic here
    
    // Container automatically removed when it stops
}
```

## Testing Patterns

### 1. Unique Container Names

Always use unique container names to enable parallel test execution:

```rust
fn unique_container_name(prefix: &str) -> String {
    format!("{}-{}", prefix, uuid::Uuid::new_v4())
}

#[tokio::test]
async fn test_parallel_safe() {
    let name = unique_container_name("postgres");
    // Each test gets a unique container
}
```

### 2. Wait for Readiness

Implement readiness checks to ensure containers are ready before testing:

```rust
use std::time::{Duration, Instant};
use tokio::net::TcpStream;

async fn wait_for_port(host: &str, port: u16, timeout: Duration) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    
    while start.elapsed() < timeout {
        if TcpStream::connect(format!("{}:{}", host, port)).await.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    Err("Port not ready within timeout".into())
}

#[tokio::test]
async fn test_with_postgres() {
    let container_name = unique_container_name("postgres");
    
    RunCommand::new("postgres:15")
        .name(&container_name)
        .env("POSTGRES_PASSWORD", "test")
        .port(5432, 5432)
        .detach()
        .remove()
        .execute()
        .await
        .unwrap();
    
    // Wait for PostgreSQL to be ready
    wait_for_port("localhost", 5432, Duration::from_secs(10)).await.unwrap();
    
    // Now safe to connect and test
}
```

### 3. Cleanup Guards

Use RAII patterns to ensure cleanup even on test failure:

```rust
struct ContainerGuard {
    name: String,
}

impl ContainerGuard {
    async fn new(image: &str, name: String) -> Result<Self, docker_wrapper::Error> {
        RunCommand::new(image)
            .name(&name)
            .detach()
            .execute()
            .await?;
        
        Ok(Self { name })
    }
}

impl Drop for ContainerGuard {
    fn drop(&mut self) {
        // Schedule cleanup
        let name = self.name.clone();
        tokio::spawn(async move {
            let _ = StopCommand::new(&name).execute().await;
            let _ = RmCommand::new(&name).force().execute().await;
        });
    }
}
```

## Test Helpers

### Reusable Test Fixtures

Create fixtures for commonly tested services:

```rust
pub struct RedisFixture {
    container_name: String,
    port: u16,
}

impl RedisFixture {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let container_name = unique_container_name("redis");
        let port = 6379; // Or find free port dynamically
        
        RunCommand::new("redis:7-alpine")
            .name(&container_name)
            .port(port, 6379)
            .detach()
            .remove()
            .execute()
            .await?;
        
        wait_for_port("localhost", port, Duration::from_secs(5)).await?;
        
        Ok(Self { container_name, port })
    }
    
    pub fn connection_string(&self) -> String {
        format!("redis://localhost:{}", self.port)
    }
    
    pub async fn cleanup(self) -> Result<(), Box<dyn std::error::Error>> {
        StopCommand::new(&self.container_name).execute().await?;
        Ok(())
    }
}

#[tokio::test]
async fn test_redis_operations() {
    let redis = RedisFixture::new().await.unwrap();
    
    // Use redis.connection_string() to connect
    // Run your tests
    
    redis.cleanup().await.unwrap();
}
```

### Database Test Helpers

```rust
pub struct PostgresFixture {
    container_name: String,
    database: String,
    username: String,
    password: String,
    port: u16,
}

impl PostgresFixture {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let container_name = unique_container_name("postgres");
        let database = "testdb";
        let username = "testuser";
        let password = "testpass";
        let port = 5432; // Or find free port
        
        RunCommand::new("postgres:15")
            .name(&container_name)
            .env("POSTGRES_DB", database)
            .env("POSTGRES_USER", username)
            .env("POSTGRES_PASSWORD", password)
            .port(port, 5432)
            .detach()
            .remove()
            .execute()
            .await?;
        
        wait_for_port("localhost", port, Duration::from_secs(10)).await?;
        
        Ok(Self {
            container_name,
            database: database.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            port,
        })
    }
    
    pub fn connection_string(&self) -> String {
        format!(
            "postgresql://{}:{}@localhost:{}/{}",
            self.username, self.password, self.port, self.database
        )
    }
}
```

## Framework Integration

### With tokio::test

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use docker_wrapper::{DockerCommand, RunCommand};
    
    #[tokio::test]
    async fn test_async_operations() {
        // docker-wrapper is built on tokio, works seamlessly
        let output = RunCommand::new("alpine")
            .cmd(vec!["echo", "hello"])
            .execute()
            .await
            .unwrap();
        
        assert!(output.0.len() > 0);
    }
}
```

### With serial_test for Sequential Tests

```rust
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_uses_specific_port() {
    // Tests that need exclusive access to resources
    let _container = RunCommand::new("nginx")
        .port(8080, 80)  // Always uses port 8080
        .detach()
        .remove()
        .execute()
        .await
        .unwrap();
    
    // Test logic
}
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Set up Docker
        run: |
          docker version
          docker info
      
      - name: Run tests
        run: cargo test --all-features
        
      - name: Cleanup
        if: always()
        run: |
          # Clean up any remaining test containers
          docker ps -aq --filter "name=test-" | xargs -r docker rm -f
```

### Resource Limits in CI

```rust
#[cfg(test)]
mod tests {
    // Limit resources for CI environments
    const CI_MEMORY_LIMIT: &str = "256m";
    const CI_CPU_LIMIT: &str = "0.5";
    
    #[tokio::test]
    async fn test_with_limits() {
        let mut cmd = RunCommand::new("postgres:15");
        
        if std::env::var("CI").is_ok() {
            cmd = cmd
                .memory(CI_MEMORY_LIMIT)
                .cpus(CI_CPU_LIMIT);
        }
        
        cmd.execute().await.unwrap();
    }
}
```

## Common Scenarios

### Testing with Multiple Containers

```rust
#[tokio::test]
async fn test_app_with_dependencies() {
    // Create network for containers to communicate
    let network_name = format!("test-net-{}", uuid::Uuid::new_v4());
    NetworkCreateCommand::new(&network_name)
        .execute()
        .await
        .unwrap();
    
    // Start database
    let db_name = unique_container_name("postgres");
    RunCommand::new("postgres:15")
        .name(&db_name)
        .network(&network_name)
        .env("POSTGRES_PASSWORD", "test")
        .detach()
        .remove()
        .execute()
        .await
        .unwrap();
    
    // Start Redis
    let redis_name = unique_container_name("redis");
    RunCommand::new("redis:7")
        .name(&redis_name)
        .network(&network_name)
        .detach()
        .remove()
        .execute()
        .await
        .unwrap();
    
    // Start application
    let app_name = unique_container_name("app");
    RunCommand::new("my-app:latest")
        .name(&app_name)
        .network(&network_name)
        .env("DATABASE_URL", &format!("postgresql://postgres:test@{}/postgres", db_name))
        .env("REDIS_URL", &format!("redis://{}", redis_name))
        .port(8080, 8080)
        .detach()
        .remove()
        .execute()
        .await
        .unwrap();
    
    // Wait and test
    wait_for_port("localhost", 8080, Duration::from_secs(10)).await.unwrap();
    
    // Run tests against the application
    
    // Cleanup network
    NetworkRmCommand::new(&network_name)
        .execute()
        .await
        .unwrap();
}
```

### Testing with Volume Data

```rust
#[tokio::test]
async fn test_with_persistent_data() {
    let volume_name = format!("test-vol-{}", uuid::Uuid::new_v4());
    
    // Create volume
    VolumeCreateCommand::new(&volume_name)
        .execute()
        .await
        .unwrap();
    
    // Use volume in container
    let container_name = unique_container_name("postgres");
    RunCommand::new("postgres:15")
        .name(&container_name)
        .volume(&volume_name, "/var/lib/postgresql/data")
        .env("POSTGRES_PASSWORD", "test")
        .detach()
        .remove()
        .execute()
        .await
        .unwrap();
    
    // Test with persistent data
    
    // Cleanup
    VolumeRmCommand::new(&volume_name)
        .force()
        .execute()
        .await
        .unwrap();
}
```

### Integration Testing with Real Services

```rust
#[tokio::test]
#[ignore] // Run with --ignored flag for integration tests
async fn test_real_service_integration() {
    let kafka = KafkaFixture::new().await.unwrap();
    let postgres = PostgresFixture::new().await.unwrap();
    
    // Test actual service integration
    // This might take longer and use more resources
    
    kafka.cleanup().await.unwrap();
    postgres.cleanup().await.unwrap();
}
```

## Best Practices

### 1. Always Use Unique Names
- Prevents conflicts in parallel test execution
- Makes debugging easier with clear container identification

### 2. Set Appropriate Timeouts
- Use reasonable timeouts for container startup
- Adjust based on CI/CD environment constraints

### 3. Clean Up Resources
- Use `--rm` flag (`.remove()`) for automatic cleanup
- Implement cleanup in test teardown or Drop implementations
- Clean up networks and volumes in addition to containers

### 4. Use Health Checks
- Implement proper readiness checks before testing
- Don't rely on simple sleep delays

### 5. Resource Management
- Set memory and CPU limits in CI environments
- Use smaller images (alpine variants) when possible

### 6. Parallel Test Execution
- Design tests to run in parallel
- Use unique ports or let Docker assign them
- Avoid shared state between tests

## Troubleshooting

### Container Failed to Start

```rust
match RunCommand::new("postgres:15")
    .name("test-db")
    .execute()
    .await
{
    Ok(output) => println!("Container started: {}", output.0),
    Err(docker_wrapper::Error::CommandFailed { stderr, .. }) => {
        eprintln!("Failed to start container: {}", stderr);
        // Check logs for more details
        if let Ok(logs) = LogsCommand::new("test-db").execute().await {
            eprintln!("Container logs: {}", logs);
        }
    }
    Err(e) => eprintln!("Unexpected error: {}", e),
}
```

### Port Already in Use

```rust
use std::net::TcpListener;

fn find_free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[tokio::test]
async fn test_with_dynamic_port() {
    let port = find_free_port();
    
    RunCommand::new("nginx")
        .port(port, 80)
        .detach()
        .remove()
        .execute()
        .await
        .unwrap();
    
    // Use the dynamic port for testing
}
```

### Debugging Test Failures

```rust
#[tokio::test]
async fn test_with_debugging() {
    let container_name = unique_container_name("app");
    
    let result = RunCommand::new("my-app")
        .name(&container_name)
        .detach()
        .execute()
        .await;
    
    if result.is_err() {
        // Capture additional debugging information
        eprintln!("Container events:");
        if let Ok(events) = EventsCommand::new()
            .filter("container", &container_name)
            .execute()
            .await
        {
            for event in events.events {
                eprintln!("{:?}", event);
            }
        }
        
        // Check container inspect for more details
        if let Ok(inspect) = InspectCommand::new(&container_name)
            .execute()
            .await
        {
            eprintln!("Container state: {:?}", inspect);
        }
    }
    
    result.unwrap();
}
```

## Example Test Suite

Here's a complete example test suite showing these patterns in practice:

```rust
#[cfg(test)]
mod integration_tests {
    use docker_wrapper::*;
    use std::time::Duration;
    use tokio;
    
    // Test helpers module
    mod helpers {
        use super::*;
        
        pub fn unique_name(prefix: &str) -> String {
            format!("{}-{}", prefix, uuid::Uuid::new_v4())
        }
        
        pub async fn wait_for_port(
            host: &str,
            port: u16,
            timeout: Duration,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let start = std::time::Instant::now();
            
            while start.elapsed() < timeout {
                if tokio::net::TcpStream::connect(format!("{}:{}", host, port))
                    .await
                    .is_ok()
                {
                    return Ok(());
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            
            Err("Port not ready".into())
        }
    }
    
    use helpers::*;
    
    #[tokio::test]
    async fn test_redis_operations() {
        let name = unique_name("redis");
        
        // Start Redis
        RunCommand::new("redis:7-alpine")
            .name(&name)
            .port(6379, 6379)
            .detach()
            .remove()
            .execute()
            .await
            .expect("Failed to start Redis");
        
        // Wait for Redis to be ready
        wait_for_port("localhost", 6379, Duration::from_secs(5))
            .await
            .expect("Redis not ready");
        
        // Test Redis operations
        let output = ExecCommand::new(&name)
            .cmd(vec!["redis-cli", "PING"])
            .execute()
            .await
            .expect("Failed to execute command");
        
        assert_eq!(output.stdout.trim(), "PONG");
    }
    
    #[tokio::test]
    async fn test_postgres_with_init_script() {
        let name = unique_name("postgres");
        
        // Create init script
        let init_sql = r#"
            CREATE TABLE users (
                id SERIAL PRIMARY KEY,
                name VARCHAR(100)
            );
            INSERT INTO users (name) VALUES ('Alice'), ('Bob');
        "#;
        
        // Start PostgreSQL with init script
        RunCommand::new("postgres:15")
            .name(&name)
            .env("POSTGRES_PASSWORD", "test")
            .env("POSTGRES_DB", "testdb")
            .port(5432, 5432)
            .detach()
            .remove()
            .execute()
            .await
            .expect("Failed to start PostgreSQL");
        
        // Wait for PostgreSQL to be ready
        wait_for_port("localhost", 5432, Duration::from_secs(10))
            .await
            .expect("PostgreSQL not ready");
        
        // Additional wait for initialization
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Verify database setup
        let output = ExecCommand::new(&name)
            .cmd(vec![
                "psql",
                "-U",
                "postgres",
                "-d",
                "testdb",
                "-c",
                "SELECT COUNT(*) FROM users;",
            ])
            .env("PGPASSWORD", "test")
            .execute()
            .await
            .expect("Failed to query database");
        
        assert!(output.stdout.contains("2"));
    }
}
```

## Summary

Testing with docker-wrapper is straightforward and flexible. By following these patterns and best practices, you can create reliable, maintainable test suites that leverage Docker containers effectively. The key principles are:

1. Use unique names for parallel safety
2. Implement proper readiness checks
3. Clean up resources reliably
4. Handle errors gracefully
5. Design for CI/CD environments

For more examples, see the `examples/` directory in the docker-wrapper repository.