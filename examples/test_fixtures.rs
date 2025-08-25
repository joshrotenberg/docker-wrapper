//! Reusable test fixtures for common services
//!
//! This example shows how to create reusable fixtures for testing with
//! databases, caches, and other services using docker-wrapper.
//!
//! Note: This is a conceptual example. Actual implementation may vary.

#[allow(unused_imports)]
use docker_wrapper::{
    DockerCommand, ExecCommand, NetworkCreateCommand, NetworkRmCommand, RmCommand, RunCommand,
    StopCommand,
};
#[allow(unused_imports)]
use std::time::{Duration, Instant};
#[allow(unused_imports)]
use tokio::net::TcpStream;

/// Helper to generate unique names for parallel test safety
fn unique_name(prefix: &str) -> String {
    format!("{}-{}", prefix, uuid::Uuid::new_v4())
}

/// Wait for a port to become available
async fn wait_for_port(host: &str, port: u16, timeout: Duration) -> Result<(), String> {
    let start = Instant::now();

    while start.elapsed() < timeout {
        if TcpStream::connect(format!("{}:{}", host, port))
            .await
            .is_ok()
        {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Err(format!(
        "Port {}:{} not ready after {:?}",
        host, port, timeout
    ))
}

/// Redis test fixture
pub struct RedisFixture {
    container_name: String,
    port: u16,
    password: Option<String>,
}

impl RedisFixture {
    /// Create a new Redis fixture with defaults
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_config(6379, None).await
    }

    /// Create a Redis fixture with custom configuration
    pub async fn with_config(
        port: u16,
        password: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let container_name = unique_name("redis");

        let mut cmd = RunCommand::new("redis:7-alpine")
            .name(&container_name)
            .port(port, 6379)
            .detach()
            .remove();

        if let Some(ref pwd) = password {
            cmd = cmd.cmd(vec!["redis-server", "--requirepass", pwd]);
        }

        cmd.execute().await?;

        // Wait for Redis to be ready
        wait_for_port("localhost", port, Duration::from_secs(5))?;

        Ok(Self {
            container_name,
            port,
            password,
        })
    }

    /// Get the Redis connection string
    pub fn connection_string(&self) -> String {
        match &self.password {
            Some(pwd) => format!("redis://:{}@localhost:{}", pwd, self.port),
            None => format!("redis://localhost:{}", self.port),
        }
    }

    /// Execute a Redis command
    pub async fn exec(&self, args: Vec<&str>) -> Result<String, Box<dyn std::error::Error>> {
        let mut cmd_args = vec!["redis-cli"];

        if let Some(ref pwd) = self.password {
            cmd_args.push("-a");
            cmd_args.push(pwd);
        }

        cmd_args.extend(args);

        let output = ExecCommand::new(&self.container_name)
            .cmd(cmd_args)
            .execute()
            .await?;

        Ok(output.stdout)
    }

    /// Clean up the Redis container
    pub async fn cleanup(self) -> Result<(), Box<dyn std::error::Error>> {
        StopCommand::new(&self.container_name).execute().await?;
        Ok(())
    }
}

/// PostgreSQL test fixture
pub struct PostgresFixture {
    container_name: String,
    database: String,
    username: String,
    password: String,
    port: u16,
}

impl PostgresFixture {
    /// Create a new PostgreSQL fixture with defaults
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_config("testdb", "testuser", "testpass", 5432).await
    }

    /// Create a PostgreSQL fixture with custom configuration
    pub async fn with_config(
        database: &str,
        username: &str,
        password: &str,
        port: u16,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let container_name = unique_name("postgres");

        RunCommand::new("postgres:15-alpine")
            .name(&container_name)
            .env("POSTGRES_DB", database)
            .env("POSTGRES_USER", username)
            .env("POSTGRES_PASSWORD", password)
            .port(port, 5432)
            .detach()
            .remove()
            .execute()
            .await?;

        // Wait for PostgreSQL to be ready
        wait_for_port("localhost", port, Duration::from_secs(10))?;

        // Additional wait for PostgreSQL initialization
        tokio::time::sleep(Duration::from_secs(2)).await;

        Ok(Self {
            container_name,
            database: database.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            port,
        })
    }

    /// Get the PostgreSQL connection string
    pub fn connection_string(&self) -> String {
        format!(
            "postgresql://{}:{}@localhost:{}/{}",
            self.username, self.password, self.port, self.database
        )
    }

    /// Execute a SQL query
    pub async fn exec_sql(&self, sql: &str) -> Result<String, Box<dyn std::error::Error>> {
        let output = ExecCommand::new(&self.container_name)
            .cmd(vec![
                "psql",
                "-U",
                &self.username,
                "-d",
                &self.database,
                "-c",
                sql,
            ])
            .env("PGPASSWORD", &self.password)
            .execute()
            .await?;

        Ok(output.stdout)
    }

    /// Clean up the PostgreSQL container
    pub async fn cleanup(self) -> Result<(), Box<dyn std::error::Error>> {
        StopCommand::new(&self.container_name).execute().await?;
        Ok(())
    }
}

/// MongoDB test fixture
pub struct MongoFixture {
    container_name: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
}

impl MongoFixture {
    /// Create a new MongoDB fixture without authentication
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_auth(None, None, 27017).await
    }

    /// Create a MongoDB fixture with authentication
    pub async fn with_auth(
        username: Option<&str>,
        password: Option<&str>,
        port: u16,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let container_name = unique_name("mongo");

        let mut cmd = RunCommand::new("mongo:6")
            .name(&container_name)
            .port(port, 27017)
            .detach()
            .remove();

        if let (Some(user), Some(pwd)) = (username, password) {
            cmd = cmd
                .env("MONGO_INITDB_ROOT_USERNAME", user)
                .env("MONGO_INITDB_ROOT_PASSWORD", pwd);
        }

        cmd.execute().await?;

        // Wait for MongoDB to be ready
        wait_for_port("localhost", port, Duration::from_secs(10))?;

        Ok(Self {
            container_name,
            port,
            username: username.map(String::from),
            password: password.map(String::from),
        })
    }

    /// Get the MongoDB connection string
    pub fn connection_string(&self) -> String {
        match (&self.username, &self.password) {
            (Some(user), Some(pwd)) => {
                format!("mongodb://{}:{}@localhost:{}", user, pwd, self.port)
            }
            _ => format!("mongodb://localhost:{}", self.port),
        }
    }

    /// Clean up the MongoDB container
    pub async fn cleanup(self) -> Result<(), Box<dyn std::error::Error>> {
        StopCommand::new(&self.container_name).execute().await?;
        Ok(())
    }
}

/// Multi-service test environment
pub struct TestEnvironment {
    network_name: String,
    containers: Vec<String>,
}

impl TestEnvironment {
    /// Create a new test environment with isolated network
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let network_name = unique_name("test-net");

        NetworkCreateCommand::new(&network_name).execute().await?;

        Ok(Self {
            network_name,
            containers: Vec::new(),
        })
    }

    /// Add a Redis service to the environment
    pub async fn with_redis(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let container_name = unique_name("env-redis");

        RunCommand::new("redis:7-alpine")
            .name(&container_name)
            .network(&self.network_name)
            .detach()
            .remove()
            .execute()
            .await?;

        self.containers.push(container_name.clone());
        Ok(container_name)
    }

    /// Add a PostgreSQL service to the environment
    pub async fn with_postgres(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let container_name = unique_name("env-postgres");

        RunCommand::new("postgres:15-alpine")
            .name(&container_name)
            .network(&self.network_name)
            .env("POSTGRES_PASSWORD", "test")
            .detach()
            .remove()
            .execute()
            .await?;

        self.containers.push(container_name.clone());
        Ok(container_name)
    }

    /// Get the network name for connecting additional containers
    pub fn network(&self) -> &str {
        &self.network_name
    }

    /// Clean up all containers and the network
    pub async fn cleanup(self) -> Result<(), Box<dyn std::error::Error>> {
        // Stop all containers
        for container in &self.containers {
            let _ = StopCommand::new(container).execute().await;
        }

        // Remove network
        NetworkRmCommand::new(&self.network_name).execute().await?;

        Ok(())
    }
}

// Example tests using the fixtures
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_with_redis_fixture() {
        let redis = RedisFixture::new()
            .await
            .expect("Failed to create Redis fixture");

        // Use the Redis connection string
        println!("Redis URL: {}", redis.connection_string());

        // Execute Redis commands
        let result = redis
            .exec(vec!["PING"])
            .await
            .expect("Failed to ping Redis");
        assert!(result.contains("PONG"));

        // Set and get a value
        redis
            .exec(vec!["SET", "test_key", "test_value"])
            .await
            .expect("Failed to set value");

        let value = redis
            .exec(vec!["GET", "test_key"])
            .await
            .expect("Failed to get value");
        assert!(value.contains("test_value"));

        redis.cleanup().await.expect("Failed to cleanup Redis");
    }

    #[tokio::test]
    async fn test_with_postgres_fixture() {
        let postgres = PostgresFixture::new()
            .await
            .expect("Failed to create PostgreSQL fixture");

        // Create a table
        postgres
            .exec_sql("CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(100))")
            .await
            .expect("Failed to create table");

        // Insert data
        postgres
            .exec_sql("INSERT INTO users (name) VALUES ('Alice'), ('Bob')")
            .await
            .expect("Failed to insert data");

        // Query data
        let result = postgres
            .exec_sql("SELECT COUNT(*) FROM users")
            .await
            .expect("Failed to query data");

        assert!(result.contains("2"));

        postgres
            .cleanup()
            .await
            .expect("Failed to cleanup PostgreSQL");
    }

    #[tokio::test]
    async fn test_with_environment() {
        let mut env = TestEnvironment::new()
            .await
            .expect("Failed to create test environment");

        // Add services
        let redis_name = env.with_redis().await.expect("Failed to add Redis");
        let postgres_name = env.with_postgres().await.expect("Failed to add PostgreSQL");

        println!("Started Redis: {}", redis_name);
        println!("Started PostgreSQL: {}", postgres_name);
        println!("Network: {}", env.network());

        // Services can communicate via container names on the test network
        // Your application can connect to these services

        env.cleanup().await.expect("Failed to cleanup environment");
    }
}

fn main() {
    println!("Test Fixtures Example");
    println!("====================");
    println!();
    println!("This example demonstrates reusable test fixtures for:");
    println!("- Redis");
    println!("- PostgreSQL");
    println!("- MongoDB");
    println!("- Multi-service environments");
    println!();
    println!("This is a conceptual example showing test fixture patterns.");
    println!("Implementation details may need adjustment for your specific use case.");
    println!();
    println!("See the source code for fixture patterns.");
}
