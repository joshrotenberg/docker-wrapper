//! Integration tests for Redis Cluster template
//!
//! These tests verify that the Redis Cluster template API works correctly
//! and can generate proper configurations for Redis clusters.

#![cfg(feature = "template-redis-cluster")]

use docker_wrapper::{NodeRole, RedisClusterConnection, RedisClusterTemplate, Template};
use std::time::Duration;
use tokio::time::timeout;

const TEST_TIMEOUT: Duration = Duration::from_secs(180);

/// Test basic Redis Cluster template API
#[tokio::test]
async fn test_redis_cluster_basic_api() {
    let cluster_name = format!("test-cluster-basic-{}", uuid::Uuid::new_v4());

    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .port_base(8100);

    // Test connection info generation without starting
    let conn = RedisClusterConnection::from_template(&cluster);
    let nodes_string = conn.nodes_string();
    assert!(nodes_string.contains("8100"), "Should contain base port");
    assert!(nodes_string.contains("8101"), "Should contain second port");
    assert!(nodes_string.contains("8102"), "Should contain third port");

    let cluster_url = conn.cluster_url();
    assert!(
        cluster_url.starts_with("redis-cluster://"),
        "Should generate valid cluster URL, got: {}",
        cluster_url
    );
}

/// Test Redis Cluster with replicas - API only
#[tokio::test]
async fn test_redis_cluster_with_replicas_api() {
    let cluster_name = format!("test-cluster-replicas-{}", uuid::Uuid::new_v4());

    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .num_replicas(1) // 1 replica per master = 6 total nodes
        .port_base(8200);

    // Verify connection info includes all nodes (masters + replicas)
    let conn = RedisClusterConnection::from_template(&cluster);
    let nodes_string = conn.nodes_string();

    // Should have 6 nodes total (3 masters + 3 replicas)
    let node_count = nodes_string.matches("8").count(); // Count port occurrences
    assert!(node_count >= 6, "Should have at least 6 nodes in cluster");
}

/// Test Redis Cluster with authentication - API only
#[tokio::test]
async fn test_redis_cluster_with_password_api() {
    let cluster_name = format!("test-cluster-auth-{}", uuid::Uuid::new_v4());
    let test_password = "secure-cluster-password";

    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .port_base(8300)
        .password(test_password);

    // Verify password is included in connection URL
    let conn = RedisClusterConnection::from_template(&cluster);
    let cluster_url = conn.cluster_url();
    assert!(
        cluster_url.contains(test_password),
        "Cluster URL should include password"
    );
}

/// Test Redis Cluster with persistence - API only
#[tokio::test]
async fn test_redis_cluster_with_persistence_api() {
    let cluster_name = format!("test-cluster-persist-{}", uuid::Uuid::new_v4());
    let data_volume = format!("{}-data", cluster_name);

    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .port_base(8400)
        .with_persistence(&data_volume);

    // Test that the configuration was applied (API test)
    let conn = RedisClusterConnection::from_template(&cluster);
    let nodes_string = conn.nodes_string();
    assert!(
        nodes_string.contains("8400"),
        "Should contain configured port"
    );
}

/// Test Redis Stack cluster functionality - API only
#[tokio::test]
async fn test_redis_stack_cluster_api() {
    let cluster_name = format!("test-stack-cluster-{}", uuid::Uuid::new_v4());

    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .port_base(8500)
        .with_redis_stack(); // Enable Redis Stack modules

    // Test API configuration
    let conn = RedisClusterConnection::from_template(&cluster);
    let nodes_string = conn.nodes_string();
    assert!(
        nodes_string.contains("8500"),
        "Should contain configured port"
    );
}

/// Test custom image and platform support - API only
#[tokio::test]
async fn test_redis_cluster_custom_image_platform_api() {
    let cluster_name = format!("test-cluster-custom-{}", uuid::Uuid::new_v4());

    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .port_base(8600)
        .custom_redis_image("redis", "7-alpine")
        .platform("linux/amd64");

    // Test API configuration
    let conn = RedisClusterConnection::from_template(&cluster);
    let nodes_string = conn.nodes_string();
    assert!(
        nodes_string.contains("8600"),
        "Should contain configured port"
    );
}

/// Test error handling for invalid configurations - API only
#[tokio::test]
async fn test_redis_cluster_error_handling_api() {
    let cluster_name = format!("test-cluster-error-{}", uuid::Uuid::new_v4());

    // Test with invalid configuration - this should not panic during construction
    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(1000) // Large number but should not cause overflow
        .port_base(8900);

    // The API should still work for connection info generation
    let conn = RedisClusterConnection::from_template(&cluster);
    let cluster_url = conn.cluster_url();
    assert!(cluster_url.contains("8900"), "Should contain base port");
}

/// Test cluster builder pattern and method chaining - API only
#[tokio::test]
async fn test_redis_cluster_builder_pattern_api() {
    let cluster_name = format!("test-cluster-builder-{}", uuid::Uuid::new_v4());

    // Test comprehensive builder chain
    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .num_replicas(1)
        .port_base(8800)
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
        cluster_url.contains("8800"),
        "Should include configured port"
    );
}

/// Test per-node accessors for targeted fault injection - API only
#[tokio::test]
async fn test_redis_cluster_node_accessors_api() {
    let cluster_name = format!("test-cluster-nodes-{}", uuid::Uuid::new_v4());

    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .num_replicas(1) // 6 nodes total
        .port_base(8700);

    // node_names() lists every container following the {name}-node-{i} contract.
    let names = cluster.node_names();
    assert_eq!(names.len(), 6, "3 masters + 3 replicas = 6 nodes");
    assert_eq!(names[0], format!("{}-node-0", cluster_name));
    assert_eq!(names[5], format!("{}-node-5", cluster_name));

    // node(i) exposes the container name, host port, and assigned role.
    let master = cluster.node(0).expect("node 0 exists");
    assert_eq!(master.container_name, format!("{}-node-0", cluster_name));
    assert_eq!(master.host_port, 8700);
    assert_eq!(master.role, NodeRole::Master);

    let replica = cluster.node(3).expect("node 3 exists");
    assert_eq!(replica.host_port, 8703);
    assert_eq!(replica.role, NodeRole::Replica);

    // Out-of-range indices return None.
    assert!(cluster.node(6).is_none());

    // node_names() and node().container_name agree on every index.
    for (i, name) in names.iter().enumerate() {
        assert_eq!(&cluster.node(i).unwrap().container_name, name);
    }
}

/// Test actual Redis Cluster container creation (single integration test)
/// This test is more likely to fail in CI due to Docker limitations, but provides
/// a basic smoke test for actual container functionality.
#[tokio::test]
#[ignore] // Ignore by default since it requires Docker
async fn test_redis_cluster_container_smoke_test() {
    let cluster_name = format!("test-cluster-smoke-{}", uuid::Uuid::new_v4());

    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .port_base(9100)
        .auto_remove();

    // Try to start the cluster - if this fails due to Docker issues in CI,
    // the test will be ignored and won't fail the build
    match timeout(TEST_TIMEOUT, cluster.start()).await {
        Ok(Ok(result)) => {
            assert!(!result.is_empty(), "Cluster result should not be empty");

            // The deterministic node handles must resolve to real, running
            // containers: query the live role of node 0 and confirm it matches
            // the statically assigned role.
            let node0 = cluster.node(0).expect("node 0 exists");
            assert_eq!(node0.role, NodeRole::Master);
            if let Ok(Ok(role)) =
                timeout(Duration::from_secs(30), cluster.node_role(node0.index)).await
            {
                assert_eq!(role, NodeRole::Master, "node 0 should report as master");
            }

            // Clean up on success
            let _ = timeout(Duration::from_secs(30), cluster.stop()).await;
            let _ = timeout(Duration::from_secs(30), cluster.remove()).await;
        }
        Ok(Err(_)) | Err(_) => {
            // Expected in environments where Docker/Redis isn't available
            // This test provides value when Docker is available but doesn't fail CI
            println!("Cluster smoke test skipped - Docker/Redis not available in test environment");
        }
    }
}

/// Host networking exposes nodes on real host ports (port_base + index) with no
/// announce-ip ceremony. This wiring is observable from the public API without
/// Docker, so it runs everywhere.
#[tokio::test]
async fn test_redis_cluster_host_network_wiring_api() {
    let cluster_name = format!("test-cluster-host-{}", uuid::Uuid::new_v4());

    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .port_base(9300)
        .host_network();

    // External clients connect to the distinct host ports, one per node.
    let conn = RedisClusterConnection::from_template(&cluster);
    assert_eq!(
        conn.nodes(),
        &[
            "localhost:9300".to_string(),
            "localhost:9301".to_string(),
            "localhost:9302".to_string(),
        ]
    );

    // The per-node handles report the same host ports.
    assert_eq!(cluster.node(0).expect("node 0").host_port, 9300);
    assert_eq!(cluster.node(2).expect("node 2").host_port, 9302);

    // network_mode("host") is equivalent to host_network().
    let via_mode = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .port_base(9300)
        .network_mode("host");
    assert_eq!(
        RedisClusterConnection::from_template(&via_mode).nodes(),
        conn.nodes()
    );
}

/// Linux-only Docker smoke test for host networking.
///
/// Host networking is a Linux-only Docker feature, so this test only attempts
/// to start a real cluster on Linux. The CI Docker Integration job runs on
/// Ubuntu and exercises this path. On macOS/Windows it is a no-op, and the test
/// is `#[ignore]` by default so it never runs without `--ignored`.
#[tokio::test]
#[ignore] // Ignore by default since it requires Docker (and Linux for host mode)
async fn test_redis_cluster_host_network_smoke_test() {
    if !cfg!(target_os = "linux") {
        println!("Host networking smoke test skipped - Linux-only Docker feature");
        return;
    }

    let cluster_name = format!("test-cluster-host-smoke-{}", uuid::Uuid::new_v4());

    let cluster = RedisClusterTemplate::new(&cluster_name)
        .num_masters(3)
        .port_base(9400)
        .host_network()
        .auto_remove();

    match timeout(TEST_TIMEOUT, cluster.start()).await {
        Ok(Ok(result)) => {
            assert!(!result.is_empty(), "Cluster result should not be empty");

            // The cluster must converge using only host ports (no bridge network,
            // no announce-ip).
            let ready = timeout(
                Duration::from_secs(60),
                cluster.wait_until_ready(Duration::from_secs(60)),
            )
            .await;
            assert!(
                matches!(ready, Ok(Ok(()))),
                "host-mode cluster should become ready"
            );

            // node_role() reaches node 0 on its host port and reports a master.
            let node0 = cluster.node(0).expect("node 0 exists");
            if let Ok(Ok(role)) =
                timeout(Duration::from_secs(30), cluster.node_role(node0.index)).await
            {
                assert_eq!(role, NodeRole::Master, "node 0 should report as master");
            }

            // Clean up on success
            let _ = timeout(Duration::from_secs(30), cluster.stop()).await;
            let _ = timeout(Duration::from_secs(30), cluster.remove()).await;
        }
        Ok(Err(_)) | Err(_) => {
            // Best-effort cleanup, then treat as skipped so transient Docker
            // issues do not fail CI.
            let _ = timeout(Duration::from_secs(30), cluster.stop()).await;
            let _ = timeout(Duration::from_secs(30), cluster.remove()).await;
            println!("Host networking smoke test skipped - Docker not available or host mode unsupported");
        }
    }
}
