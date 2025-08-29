//! Integration tests for Redis templates

#[cfg(feature = "template-redis")]
mod redis_template_tests {
    use docker_wrapper::{DockerCommand, RedisTemplate, Template};
    use std::time::Duration;
    use tokio::time::sleep;

    /// Generate a unique container name for tests
    fn test_container_name(suffix: &str) -> String {
        format!("test-redis-template-{}-{}", suffix, uuid::Uuid::new_v4())
    }

    /// Generate a random port for testing to avoid conflicts
    fn random_port() -> u16 {
        // Use a range that's unlikely to conflict with common services
        30000 + (uuid::Uuid::new_v4().as_u128() % 10000) as u16
    }

    #[tokio::test]
    async fn test_redis_basic_start_stop() -> Result<(), Box<dyn std::error::Error>> {
        let name = test_container_name("basic");
        let redis = RedisTemplate::new(&name).port(random_port());

        // Start the container and wait for it to be ready
        let container_id = redis.start_and_wait().await?;
        assert!(!container_id.is_empty());

        // Check if it's running
        assert!(redis.is_running().await?);

        // Test PING command
        let result = redis.exec(vec!["redis-cli", "ping"]).await?;
        assert_eq!(result.stdout.trim(), "PONG");

        // Clean up
        redis.stop().await?;
        sleep(Duration::from_millis(500)).await;
        assert!(!redis.is_running().await?);
        redis.remove().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_with_password() -> Result<(), Box<dyn std::error::Error>> {
        let name = test_container_name("password");
        let redis = RedisTemplate::new(&name)
            .port(random_port())
            .password("test_password");

        // Start and wait for ready
        let _container_id = redis.start_and_wait().await?;

        // Test auth with password
        let result = redis
            .exec(vec!["redis-cli", "-a", "test_password", "ping"])
            .await?;
        assert_eq!(result.stdout.trim(), "PONG");

        // Test without password should fail
        let result = redis.exec(vec!["redis-cli", "ping"]).await?;
        assert!(result.stdout.contains("NOAUTH") || result.stderr.contains("NOAUTH"));

        // Clean up
        redis.stop().await?;
        redis.remove().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_with_persistence() -> Result<(), Box<dyn std::error::Error>> {
        let name = test_container_name("persistence");
        let volume_name = format!("{}-data", name);

        let redis = RedisTemplate::new(&name)
            .port(random_port())
            .with_persistence(&volume_name);

        // Start and wait
        let _container_id = redis.start_and_wait().await?;

        // Set a value
        redis
            .exec(vec!["redis-cli", "SET", "test_key", "test_value"])
            .await?;

        // Verify the value
        let result = redis.exec(vec!["redis-cli", "GET", "test_key"]).await?;
        assert_eq!(result.stdout.trim(), "test_value");

        // Stop and remove container (but not volume)
        redis.stop().await?;
        redis.remove().await?;

        // Start a new container with same volume
        let redis2 = RedisTemplate::new(format!("{}-2", name))
            .port(random_port())
            .with_persistence(&volume_name);

        let _container_id2 = redis2.start_and_wait().await?;

        // Check if data persisted
        let result = redis2.exec(vec!["redis-cli", "GET", "test_key"]).await?;
        assert_eq!(result.stdout.trim(), "test_value");

        // Clean up
        redis2.stop().await?;
        redis2.remove().await?;

        // Clean up volume
        use docker_wrapper::VolumeRmCommand;
        VolumeRmCommand::new(&volume_name).force().execute().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_memory_limit() -> Result<(), Box<dyn std::error::Error>> {
        let name = test_container_name("memory");
        let redis = RedisTemplate::new(&name)
            .port(random_port())
            .memory_limit("128m");

        // Start and wait
        let _container_id = redis.start_and_wait().await?;

        // Check memory limit via docker inspect
        use docker_wrapper::InspectCommand;
        let inspect = InspectCommand::new(&name).execute().await?;

        // Parse JSON output
        let containers: serde_json::Value = serde_json::from_str(&inspect.stdout)?;
        if let Some(first) = containers.as_array().and_then(|arr| arr.first()) {
            if let Some(host_config) = first.get("HostConfig") {
                if let Some(memory) = host_config.get("Memory").and_then(|m| m.as_i64()) {
                    // 128MB = 134217728 bytes
                    assert_eq!(memory, 134217728);
                }
            }
        }

        // Clean up
        redis.stop().await?;
        redis.remove().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_custom_image() -> Result<(), Box<dyn std::error::Error>> {
        let name = test_container_name("custom");
        let redis = RedisTemplate::new(&name)
            .port(random_port())
            .custom_image("redis", "6-alpine");

        // Start and wait
        let _container_id = redis.start_and_wait().await?;

        // Verify Redis version
        let result = redis.exec(vec!["redis-cli", "INFO", "server"]).await?;
        assert!(result.stdout.contains("redis_version:6."));

        // Clean up
        redis.stop().await?;
        redis.remove().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_stack() -> Result<(), Box<dyn std::error::Error>> {
        let name = test_container_name("stack");
        let redis = RedisTemplate::new(&name)
            .port(random_port())
            .with_redis_stack();

        // Start and wait
        let _container_id = redis.start_and_wait().await?;

        // Check if Redis Stack modules are loaded
        let result = redis.exec(vec!["redis-cli", "MODULE", "LIST"]).await?;

        // Redis Stack should have modules like ReJSON, RediSearch, etc.
        assert!(
            result.stdout.contains("search")
                || result.stdout.contains("json")
                || result.stdout.contains("timeseries"),
            "Redis Stack modules not found in output: {}",
            result.stdout
        );

        // Clean up
        redis.stop().await?;
        redis.remove().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_logs() -> Result<(), Box<dyn std::error::Error>> {
        let name = test_container_name("logs");
        let redis = RedisTemplate::new(&name).port(random_port());

        // Start and wait
        let _container_id = redis.start_and_wait().await?;

        // Get logs
        let logs = redis.logs(false, Some("10")).await?;
        assert!(!logs.stdout.is_empty());
        assert!(logs.stdout.contains("Ready to accept connections"));

        // Clean up
        redis.stop().await?;
        redis.remove().await?;

        Ok(())
    }
}
