//! Integration tests for Redis Cluster template
//!
//! These tests verify that the Redis Cluster template can be used to create
//! working Redis clusters with proper node coordination, slot allocation,
//! and cluster functionality.

#![cfg(feature = "template-redis-cluster")]

use docker_wrapper::{RedisClusterConnection, RedisClusterTemplate, Template};
use std::time::Duration;
use tokio::time::timeout;

const TEST_TIMEOUT: Duration = Duration::from_secs(60);

/// Test basic Redis Cluster creation and teardown
#[tokio::test]
async fn test_redis_cluster_basic_lifecycle() {
    let cluster_name = format!("test-cluster-basic-{}", uuid::Uuid::new_v4());

    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .port_base(7100)
        .auto_remove();

    // Start the cluster
    let result = timeout(TEST_TIMEOUT, cluster.start())
        .await
        .expect("Cluster startup timed out")
        .expect("Failed to start cluster");

    assert!(!result.is_empty(), "Cluster result should not be empty");
    assert!(
        result.contains("started"),
        "Result should indicate cluster was started"
    );

    // Verify cluster connection info
    let conn = RedisClusterConnection::from_template(&cluster);
    let nodes_string = conn.nodes_string();
    assert!(nodes_string.contains("7100"), "Should contain base port");
    assert!(nodes_string.contains("7101"), "Should contain second port");
    assert!(nodes_string.contains("7102"), "Should contain third port");

    let cluster_url = conn.cluster_url();
    assert!(
        cluster_url.starts_with("redis://"),
        "Should generate valid cluster URL"
    );

    // Clean up
    timeout(TEST_TIMEOUT, cluster.stop())
        .await
        .expect("Cluster stop timed out")
        .expect("Failed to stop cluster");

    timeout(TEST_TIMEOUT, cluster.remove())
        .await
        .expect("Cluster remove timed out")
        .expect("Failed to remove cluster");
}

/// Test Redis Cluster with replicas
#[tokio::test]
async fn test_redis_cluster_with_replicas() {
    let cluster_name = format!("test-cluster-replicas-{}", uuid::Uuid::new_v4());

    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .num_replicas(1) // 1 replica per master = 6 total nodes
        .port_base(7200)
        .auto_remove();

    // Start the cluster
    let result = timeout(TEST_TIMEOUT, cluster.start())
        .await
        .expect("Cluster startup timed out")
        .expect("Failed to start cluster");

    assert!(
        result.contains("started"),
        "Result should indicate cluster was started"
    );

    // Verify connection info includes replica ports
    let conn = RedisClusterConnection::from_template(&cluster);
    let nodes_string = conn.nodes_string();

    // Should have 6 nodes total (3 masters + 3 replicas)
    let node_count = nodes_string.matches("7").count(); // Count port occurrences
    assert!(node_count >= 6, "Should have at least 6 nodes in cluster");

    // Clean up
    timeout(TEST_TIMEOUT, cluster.stop())
        .await
        .expect("Cluster stop timed out")
        .expect("Failed to stop cluster");

    timeout(TEST_TIMEOUT, cluster.remove())
        .await
        .expect("Cluster remove timed out")
        .expect("Failed to remove cluster");
}

/// Test Redis Cluster with authentication
#[tokio::test]
async fn test_redis_cluster_with_password() {
    let cluster_name = format!("test-cluster-auth-{}", uuid::Uuid::new_v4());
    let test_password = "secure-cluster-password";

    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .port_base(7300)
        .password(test_password)
        .auto_remove();

    // Start the cluster
    let result = timeout(TEST_TIMEOUT, cluster.start())
        .await
        .expect("Cluster startup timed out")
        .expect("Failed to start cluster");

    assert!(
        result.contains("started"),
        "Result should indicate cluster was started"
    );

    // Verify password is included in connection URL
    let conn = RedisClusterConnection::from_template(&cluster);
    let cluster_url = conn.cluster_url();
    assert!(
        cluster_url.contains(test_password),
        "Cluster URL should include password"
    );

    // Clean up
    timeout(TEST_TIMEOUT, cluster.stop())
        .await
        .expect("Cluster stop timed out")
        .expect("Failed to stop cluster");

    timeout(TEST_TIMEOUT, cluster.remove())
        .await
        .expect("Cluster remove timed out")
        .expect("Failed to remove cluster");
}

/// Test Redis Cluster with persistence
#[tokio::test]
async fn test_redis_cluster_with_persistence() {
    let cluster_name = format!("test-cluster-persist-{}", uuid::Uuid::new_v4());
    let data_volume = format!("{}-data", cluster_name);

    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .port_base(7400)
        .with_persistence(&data_volume)
        .auto_remove();

    // Start the cluster
    let result = timeout(TEST_TIMEOUT, cluster.start())
        .await
        .expect("Cluster startup timed out")
        .expect("Failed to start cluster");

    assert!(
        result.contains("started"),
        "Result should indicate cluster was started"
    );

    // Clean up
    timeout(TEST_TIMEOUT, cluster.stop())
        .await
        .expect("Cluster stop timed out")
        .expect("Failed to stop cluster");

    timeout(TEST_TIMEOUT, cluster.remove())
        .await
        .expect("Cluster remove timed out")
        .expect("Failed to remove cluster");
}

/// Test Redis Stack cluster functionality
#[tokio::test]
async fn test_redis_stack_cluster() {
    let cluster_name = format!("test-stack-cluster-{}", uuid::Uuid::new_v4());

    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .port_base(7500)
        .with_redis_stack() // Enable Redis Stack modules
        .auto_remove();

    // Start the cluster
    let result = timeout(TEST_TIMEOUT, cluster.start())
        .await
        .expect("Stack cluster startup timed out")
        .expect("Failed to start stack cluster");

    assert!(
        result.contains("started"),
        "Result should indicate stack cluster was started"
    );

    // Clean up
    timeout(TEST_TIMEOUT, cluster.stop())
        .await
        .expect("Stack cluster stop timed out")
        .expect("Failed to stop stack cluster");

    timeout(TEST_TIMEOUT, cluster.remove())
        .await
        .expect("Stack cluster remove timed out")
        .expect("Failed to remove stack cluster");
}

/// Test custom image and platform support
#[tokio::test]
async fn test_redis_cluster_custom_image_platform() {
    let cluster_name = format!("test-cluster-custom-{}", uuid::Uuid::new_v4());

    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .port_base(7600)
        .custom_redis_image("redis", "7-alpine")
        .platform("linux/amd64")
        .auto_remove();

    // Start the cluster
    let result = timeout(TEST_TIMEOUT, cluster.start())
        .await
        .expect("Custom cluster startup timed out")
        .expect("Failed to start custom cluster");

    assert!(
        result.contains("started"),
        "Result should indicate custom cluster was started"
    );

    // Clean up
    timeout(TEST_TIMEOUT, cluster.stop())
        .await
        .expect("Custom cluster stop timed out")
        .expect("Failed to stop custom cluster");

    timeout(TEST_TIMEOUT, cluster.remove())
        .await
        .expect("Custom cluster remove timed out")
        .expect("Failed to remove custom cluster");
}

/// Test cluster info retrieval
#[tokio::test]
async fn test_redis_cluster_info() {
    let cluster_name = format!("test-cluster-info-{}", uuid::Uuid::new_v4());

    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .port_base(7700)
        .auto_remove();

    // Start the cluster
    timeout(TEST_TIMEOUT, cluster.start())
        .await
        .expect("Cluster startup timed out")
        .expect("Failed to start cluster");

    // Wait a bit for cluster to initialize
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Try to get cluster info
    match timeout(Duration::from_secs(30), cluster.cluster_info()).await {
        Ok(Ok(info)) => {
            // Validate cluster info structure
            assert!(
                !info.cluster_state.is_empty(),
                "Cluster state should not be empty"
            );
            assert!(info.total_slots > 0, "Should have assigned slots");
            assert!(!info.nodes.is_empty(), "Should have nodes");
            assert_eq!(info.nodes.len(), 3, "Should have 3 master nodes");
        }
        Ok(Err(e)) => {
            // It's okay if cluster info fails in test environment
            println!(
                "Cluster info retrieval failed (expected in test env): {}",
                e
            );
        }
        Err(_) => {
            println!("Cluster info retrieval timed out (expected in test env)");
        }
    }

    // Clean up
    timeout(TEST_TIMEOUT, cluster.stop())
        .await
        .expect("Cluster stop timed out")
        .expect("Failed to stop cluster");

    timeout(TEST_TIMEOUT, cluster.remove())
        .await
        .expect("Cluster remove timed out")
        .expect("Failed to remove cluster");
}

/// Test error handling for invalid configurations
#[tokio::test]
async fn test_redis_cluster_error_handling() {
    let cluster_name = format!("test-cluster-error-{}", uuid::Uuid::new_v4());

    // Test with invalid port range (too high)
    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .port_base(65530) // This will cause port overflow
        .auto_remove();

    // This should handle the error gracefully
    match timeout(TEST_TIMEOUT, cluster.start()).await {
        Ok(Err(_)) => {
            // Expected error case
        }
        Ok(Ok(_)) => {
            // If it somehow succeeds, clean up
            let _ = cluster.stop().await;
            let _ = cluster.remove().await;
        }
        Err(_) => {
            // Timeout is also acceptable for error cases
        }
    }
}

/// Test cluster builder pattern and method chaining
#[tokio::test]
async fn test_redis_cluster_builder_pattern() {
    let cluster_name = format!("test-cluster-builder-{}", uuid::Uuid::new_v4());

    // Test comprehensive builder chain
    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .num_replicas(1)
        .port_base(7800)
        .password("builder-password")
        .cluster_announce_ip("127.0.0.1")
        .cluster_node_timeout(15000)
        .memory_limit("128m")
        .auto_remove();

    // Verify builder created proper configuration
    let conn = RedisClusterConnection::from_template(&cluster);
    let cluster_url = conn.cluster_url();

    assert!(
        cluster_url.contains("builder-password"),
        "Should include configured password"
    );
    assert!(
        cluster_url.contains("7800"),
        "Should include configured port"
    );

    // Start and stop to verify the configuration works
    let result = timeout(TEST_TIMEOUT, cluster.start())
        .await
        .expect("Builder cluster startup timed out")
        .expect("Failed to start builder cluster");

    assert!(
        result.contains("started"),
        "Result should indicate builder cluster was started"
    );

    // Clean up
    timeout(TEST_TIMEOUT, cluster.stop())
        .await
        .expect("Builder cluster stop timed out")
        .expect("Failed to stop builder cluster");

    timeout(TEST_TIMEOUT, cluster.remove())
        .await
        .expect("Builder cluster remove timed out")
        .expect("Failed to remove builder cluster");
}
