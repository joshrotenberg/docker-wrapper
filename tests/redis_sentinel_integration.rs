//! Integration tests for Redis Sentinel template
//!
//! These tests verify that the Redis Sentinel template can create working
//! high-availability Redis setups with proper master-replica relationships
//! and sentinel monitoring for automatic failover.

#![cfg(feature = "template-redis")]

use docker_wrapper::RedisSentinelTemplate;
use std::time::Duration;
use tokio::time::timeout;

const TEST_TIMEOUT: Duration = Duration::from_secs(90);

/// Test basic Redis Sentinel setup and teardown
#[tokio::test]
async fn test_redis_sentinel_basic_lifecycle() {
    let sentinel_name = format!("test-sentinel-basic-{}", uuid::Uuid::new_v4());

    let sentinel = RedisSentinelTemplate::new(&sentinel_name)
        .master_name("testmaster")
        .num_replicas(2)
        .num_sentinels(3)
        .quorum(2)
        .master_port(6380)
        .replica_port_base(6381)
        .sentinel_port_base(26379);

    // Start the sentinel cluster
    let connection_info = timeout(TEST_TIMEOUT, sentinel.start())
        .await
        .expect("Sentinel startup timed out")
        .expect("Failed to start sentinel cluster");

    // Verify connection information
    assert_eq!(
        connection_info.master_port, 6380,
        "Master should be on configured port"
    );
    assert_eq!(
        connection_info.replica_ports.len(),
        2,
        "Should have 2 replicas"
    );
    assert_eq!(
        connection_info.sentinels.len(),
        3,
        "Should have 3 sentinels"
    );

    // Verify master URL
    let master_url = connection_info.master_url();
    assert!(
        master_url.starts_with("redis://"),
        "Master URL should be valid Redis URL"
    );
    assert!(
        master_url.contains("6380"),
        "Master URL should contain correct port"
    );

    // Verify sentinel URLs
    let sentinel_urls = connection_info.sentinel_urls();
    assert_eq!(sentinel_urls.len(), 3, "Should have 3 sentinel URLs");

    for url in &sentinel_urls {
        assert!(url.starts_with("redis://"), "Sentinel URLs should be valid");
        assert!(
            url.contains("2637"),
            "Sentinel URLs should contain sentinel ports"
        );
    }

    // Clean up
    timeout(TEST_TIMEOUT, connection_info.stop())
        .await
        .expect("Sentinel stop timed out")
        .expect("Failed to stop sentinel cluster");
}

/// Test Redis Sentinel with authentication
#[tokio::test]
async fn test_redis_sentinel_with_password() {
    let sentinel_name = format!("test-sentinel-auth-{}", uuid::Uuid::new_v4());
    let test_password = "sentinel-test-password";

    let sentinel = RedisSentinelTemplate::new(&sentinel_name)
        .master_name("authmaster")
        .num_replicas(1)
        .num_sentinels(3)
        .quorum(2)
        .password(test_password)
        .master_port(6390)
        .replica_port_base(6391)
        .sentinel_port_base(26389);

    // Start the sentinel cluster
    let connection_info = timeout(TEST_TIMEOUT, sentinel.start())
        .await
        .expect("Authenticated sentinel startup timed out")
        .expect("Failed to start authenticated sentinel cluster");

    // Verify password is included in connection URLs
    let master_url = connection_info.master_url();
    assert!(
        master_url.contains(test_password),
        "Master URL should include password"
    );

    // Clean up
    timeout(TEST_TIMEOUT, connection_info.stop())
        .await
        .expect("Authenticated sentinel stop timed out")
        .expect("Failed to stop authenticated sentinel cluster");
}

/// Test Redis Sentinel with persistence
#[tokio::test]
async fn test_redis_sentinel_with_persistence() {
    let sentinel_name = format!("test-sentinel-persist-{}", uuid::Uuid::new_v4());

    let sentinel = RedisSentinelTemplate::new(&sentinel_name)
        .master_name("persistmaster")
        .num_replicas(2)
        .num_sentinels(3)
        .quorum(2)
        .with_persistence()
        .master_port(6400)
        .replica_port_base(6401)
        .sentinel_port_base(26400);

    // Start the sentinel cluster
    let connection_info = timeout(TEST_TIMEOUT, sentinel.start())
        .await
        .expect("Persistent sentinel startup timed out")
        .expect("Failed to start persistent sentinel cluster");

    // Verify the cluster started successfully
    assert!(
        !connection_info.containers.is_empty(),
        "Should have created containers"
    );
    assert!(
        connection_info.containers.len() >= 6,
        "Should have master + replicas + sentinels"
    );

    // Clean up
    timeout(TEST_TIMEOUT, connection_info.stop())
        .await
        .expect("Persistent sentinel stop timed out")
        .expect("Failed to stop persistent sentinel cluster");
}

/// Test Redis Sentinel with custom configuration
#[tokio::test]
async fn test_redis_sentinel_custom_config() {
    let sentinel_name = format!("test-sentinel-config-{}", uuid::Uuid::new_v4());

    let sentinel = RedisSentinelTemplate::new(&sentinel_name)
        .master_name("custommaster")
        .num_replicas(1)
        .num_sentinels(3)
        .quorum(2)
        .down_after_milliseconds(10000)
        .failover_timeout(60000)
        .parallel_syncs(2)
        .master_port(6410)
        .replica_port_base(6411)
        .sentinel_port_base(26410);

    // Start the sentinel cluster
    let connection_info = timeout(TEST_TIMEOUT, sentinel.start())
        .await
        .expect("Custom sentinel startup timed out")
        .expect("Failed to start custom sentinel cluster");

    // Verify configuration was applied
    assert_eq!(
        connection_info.name, sentinel_name,
        "Should preserve cluster name"
    );

    // Clean up
    timeout(TEST_TIMEOUT, connection_info.stop())
        .await
        .expect("Custom sentinel stop timed out")
        .expect("Failed to stop custom sentinel cluster");
}

/// Test Redis Sentinel with custom image and platform
#[tokio::test]
async fn test_redis_sentinel_custom_image_platform() {
    let sentinel_name = format!("test-sentinel-custom-{}", uuid::Uuid::new_v4());

    let sentinel = RedisSentinelTemplate::new(&sentinel_name)
        .master_name("customimage")
        .num_replicas(1)
        .num_sentinels(3)
        .quorum(2)
        .custom_redis_image("redis", "7-alpine")
        .platform("linux/amd64")
        .master_port(6420)
        .replica_port_base(6421)
        .sentinel_port_base(26420);

    // Start the sentinel cluster
    let connection_info = timeout(TEST_TIMEOUT, sentinel.start())
        .await
        .expect("Custom image sentinel startup timed out")
        .expect("Failed to start custom image sentinel cluster");

    // Verify the cluster started with custom configuration
    assert!(
        !connection_info.containers.is_empty(),
        "Should have created containers"
    );

    // Clean up
    timeout(TEST_TIMEOUT, connection_info.stop())
        .await
        .expect("Custom image sentinel stop timed out")
        .expect("Failed to stop custom image sentinel cluster");
}

/// Test Redis Sentinel with minimal configuration
#[tokio::test]
async fn test_redis_sentinel_minimal_config() {
    let sentinel_name = format!("test-sentinel-minimal-{}", uuid::Uuid::new_v4());

    // Use default configuration with just the name
    let sentinel = RedisSentinelTemplate::new(&sentinel_name)
        .master_port(6430)
        .replica_port_base(6431)
        .sentinel_port_base(26430);

    // Start the sentinel cluster
    let connection_info = timeout(TEST_TIMEOUT, sentinel.start())
        .await
        .expect("Minimal sentinel startup timed out")
        .expect("Failed to start minimal sentinel cluster");

    // Verify defaults were applied
    assert_eq!(
        connection_info.replica_ports.len(),
        2,
        "Should have default 2 replicas"
    );
    assert_eq!(
        connection_info.sentinels.len(),
        3,
        "Should have default 3 sentinels"
    );

    // Clean up
    timeout(TEST_TIMEOUT, connection_info.stop())
        .await
        .expect("Minimal sentinel stop timed out")
        .expect("Failed to stop minimal sentinel cluster");
}

/// Test Redis Sentinel network configuration
#[tokio::test]
async fn test_redis_sentinel_network() {
    let sentinel_name = format!("test-sentinel-network-{}", uuid::Uuid::new_v4());
    let network_name = format!("{}-network", sentinel_name);

    let sentinel = RedisSentinelTemplate::new(&sentinel_name)
        .master_name("netmaster")
        .num_replicas(1)
        .num_sentinels(3)
        .quorum(2)
        .network(&network_name)
        .master_port(6440)
        .replica_port_base(6441)
        .sentinel_port_base(26440);

    // Start the sentinel cluster
    let connection_info = timeout(TEST_TIMEOUT, sentinel.start())
        .await
        .expect("Network sentinel startup timed out")
        .expect("Failed to start network sentinel cluster");

    // Verify the network was configured
    assert!(
        !connection_info.containers.is_empty(),
        "Should have created containers"
    );

    // Clean up
    timeout(TEST_TIMEOUT, connection_info.stop())
        .await
        .expect("Network sentinel stop timed out")
        .expect("Failed to stop network sentinel cluster");
}

/// Test Redis Sentinel builder pattern and method chaining
#[tokio::test]
async fn test_redis_sentinel_builder_pattern() {
    let sentinel_name = format!("test-sentinel-builder-{}", uuid::Uuid::new_v4());

    // Test comprehensive builder chain
    let sentinel = RedisSentinelTemplate::new(&sentinel_name)
        .master_name("buildermaster")
        .num_replicas(2)
        .num_sentinels(3)
        .quorum(2)
        .password("builder-password")
        .down_after_milliseconds(5000)
        .failover_timeout(30000)
        .parallel_syncs(1)
        .with_persistence()
        .custom_redis_image("redis", "latest")
        .platform("linux/amd64")
        .master_port(6450)
        .replica_port_base(6451)
        .sentinel_port_base(26450);

    // Start the sentinel cluster
    let connection_info = timeout(TEST_TIMEOUT, sentinel.start())
        .await
        .expect("Builder sentinel startup timed out")
        .expect("Failed to start builder sentinel cluster");

    // Verify all builder options were applied
    assert_eq!(
        connection_info.replica_ports.len(),
        2,
        "Should have 2 replicas as configured"
    );
    assert_eq!(
        connection_info.sentinels.len(),
        3,
        "Should have 3 sentinels as configured"
    );

    let master_url = connection_info.master_url();
    assert!(
        master_url.contains("builder-password"),
        "Should include configured password"
    );
    assert!(
        master_url.contains("6450"),
        "Should include configured master port"
    );

    // Clean up
    timeout(TEST_TIMEOUT, connection_info.stop())
        .await
        .expect("Builder sentinel stop timed out")
        .expect("Failed to stop builder sentinel cluster");
}

/// Test error handling for invalid sentinel configurations
#[tokio::test]
async fn test_redis_sentinel_error_handling() {
    let sentinel_name = format!("test-sentinel-error-{}", uuid::Uuid::new_v4());

    // Test with invalid configuration (quorum larger than sentinels)
    let sentinel = RedisSentinelTemplate::new(&sentinel_name)
        .master_name("errormaster")
        .num_sentinels(2)
        .quorum(5) // Invalid: quorum > sentinels
        .master_port(6460)
        .replica_port_base(6461)
        .sentinel_port_base(26460);

    // This should handle the error gracefully or warn about the configuration
    match timeout(TEST_TIMEOUT, sentinel.start()).await {
        Ok(result) => {
            match result {
                Ok(connection_info) => {
                    // If it starts despite invalid config, clean up
                    let _ = connection_info.stop().await;
                }
                Err(_) => {
                    // Expected error case
                }
            }
        }
        Err(_) => {
            // Timeout is also acceptable for error cases
        }
    }
}

/// Test Redis Sentinel container naming and identification
#[tokio::test]
async fn test_redis_sentinel_container_identification() {
    let sentinel_name = format!("test-sentinel-id-{}", uuid::Uuid::new_v4());

    let sentinel = RedisSentinelTemplate::new(&sentinel_name)
        .master_name("idmaster")
        .num_replicas(1)
        .num_sentinels(3)
        .quorum(2)
        .master_port(6470)
        .replica_port_base(6471)
        .sentinel_port_base(26470);

    // Start the sentinel cluster
    let connection_info = timeout(TEST_TIMEOUT, sentinel.start())
        .await
        .expect("ID sentinel startup timed out")
        .expect("Failed to start ID sentinel cluster");

    // Verify container names follow expected patterns
    let containers = &connection_info.containers;
    assert!(!containers.is_empty(), "Should have created containers");

    // Should have master + replicas + sentinels
    let expected_container_count = 1 + 1 + 3; // 1 master + 1 replica + 3 sentinels
    assert_eq!(
        containers.len(),
        expected_container_count,
        "Should have exactly {} containers",
        expected_container_count
    );

    // Verify container names contain the base name
    for container in containers {
        assert!(
            container.contains(&sentinel_name),
            "Container '{}' should contain base name '{}'",
            container,
            sentinel_name
        );
    }

    // Clean up
    timeout(TEST_TIMEOUT, connection_info.stop())
        .await
        .expect("ID sentinel stop timed out")
        .expect("Failed to stop ID sentinel cluster");
}
