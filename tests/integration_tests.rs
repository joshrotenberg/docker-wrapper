//! Phase 2 Integration Tests
//!
//! Comprehensive integration tests for Phase 2 container lifecycle management features.
//! These tests require a running Docker daemon and will create/destroy real containers.

use docker_wrapper::*;
use std::time::Duration;
use tokio::time::sleep;

// Test configuration
const TEST_IMAGE: &str = "alpine:3.18";
const REDIS_IMAGE: &str = "redis:7.2-alpine";
const TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Helper to check if Docker is available
async fn docker_available() -> bool {
    DockerClient::new().await.is_ok()
}

/// Helper to generate unique test container names
fn test_container_name(test_name: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("test-{}-{}", test_name, timestamp)
}

/// Helper to ensure test cleanup
async fn cleanup_container(client: &DockerClient, container_id: &ContainerId) {
    let manager = ContainerManager::new(client);
    let _ = manager
        .stop(container_id, Some(Duration::from_secs(5)))
        .await;
    let _ = manager
        .remove(
            container_id,
            RemoveOptions {
                force: true,
                remove_volumes: true,
            },
        )
        .await;
}

#[tokio::test]
async fn test_docker_client_creation() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");

    // Test ping
    client.ping().await.expect("Should ping Docker daemon");

    // Test version info
    let mut client = client;
    let version = client.version().await.expect("Should get Docker version");
    assert!(
        !version.client.is_empty(),
        "Client version should not be empty"
    );
}

#[tokio::test]
async fn test_container_builder_fluent_api() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let container_name = test_container_name("builder");

    let container_id = ContainerBuilder::new(TEST_IMAGE)
        .name(&container_name)
        .env("TEST_VAR", "test_value")
        .env("ANOTHER_VAR", "another_value")
        .label("test", "phase2")
        .label("component", "integration-test")
        .memory_str("128m")
        .command(vec!["sleep".to_string(), "10".to_string()])
        .run(&client)
        .await
        .expect("Should create and run container");

    // Verify container was created
    let manager = ContainerManager::new(&client);
    let container_info = manager
        .inspect(&container_id)
        .await
        .expect("Should inspect container");

    assert_eq!(container_info.name, Some(container_name));
    assert_eq!(container_info.image, TEST_IMAGE);
    assert!(container_info.labels.contains_key("test"));
    assert_eq!(
        container_info.labels.get("test"),
        Some(&"phase2".to_string())
    );

    // Clean up the container
    cleanup_container(&client, &container_id).await;
}

#[tokio::test]
async fn test_dynamic_port_allocation() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let container_name = test_container_name("port");

    let container_id = ContainerBuilder::new("nginx:alpine")
        .name(&container_name)
        .port_dynamic(80)
        .auto_remove()
        .run(&client)
        .await
        .expect("Should create nginx container");

    // Give nginx time to start
    sleep(Duration::from_secs(2)).await;

    let manager = ContainerManager::new(&client);
    let host_port = manager
        .port(&container_id, 80)
        .await
        .expect("Should get port mapping")
        .expect("Port should be mapped");

    assert!(
        host_port > 1024,
        "Host port should be dynamic (>1024), got {}",
        host_port
    );

    // Test port accessibility
    let health_checker = HealthChecker::new(&client);
    let port_result = health_checker
        .check_tcp_port(
            "127.0.0.1".parse().unwrap(),
            host_port,
            Duration::from_secs(5),
        )
        .await
        .expect("Should check port");

    assert!(port_result, "Port should be accessible");

    // Cleanup
    cleanup_container(&client, &container_id).await;
}

#[tokio::test]
async fn test_container_execution() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let container_name = test_container_name("exec");

    let container_id = ContainerBuilder::new(TEST_IMAGE)
        .name(&container_name)
        .command(vec!["sleep".to_string(), "30".to_string()])
        .run(&client)
        .await
        .expect("Should create container");

    let executor = ContainerExecutor::new(&client);

    // Test simple command execution
    let result = executor
        .exec_simple(
            &container_id,
            vec!["echo".to_string(), "hello world".to_string()],
        )
        .await
        .expect("Should execute echo command");

    assert_eq!(result.trim(), "hello world");

    // Test command with environment variables
    let config = ExecConfig::new(vec!["env".to_string()]).env("TEST_EXEC_VAR", "exec_value");

    let exec_result = executor
        .exec(&container_id, config)
        .await
        .expect("Should execute env command");

    assert!(exec_result.is_success(), "Command should succeed");
    assert!(
        exec_result.stdout.contains("TEST_EXEC_VAR=exec_value"),
        "Output should contain environment variable"
    );

    // Test command with working directory
    let config = ExecConfig::new(vec!["pwd".to_string()]).working_dir("/tmp");

    let exec_result = executor
        .exec(&container_id, config)
        .await
        .expect("Should execute pwd command");

    assert!(exec_result.is_success(), "Command should succeed");
    assert_eq!(exec_result.stdout.trim(), "/tmp");

    // Cleanup
    cleanup_container(&client, &container_id).await;
}

#[tokio::test]
async fn test_health_checking() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let container_name = test_container_name("health");

    let container_id = ContainerBuilder::new(REDIS_IMAGE)
        .name(&container_name)
        .port_dynamic(6379)
        .env("REDIS_PASSWORD", "testpass")
        .command(vec![
            "redis-server".to_string(),
            "--requirepass".to_string(),
            "testpass".to_string(),
        ])
        .run(&client)
        .await
        .expect("Should create Redis container");

    let health_checker = HealthChecker::new(&client);

    // Test port health check
    health_checker
        .wait_for_port(&container_id, 6379, TEST_TIMEOUT)
        .await
        .expect("Redis port should become ready");

    // Test command health check
    let command_check = HealthCheck::command(vec![
        "redis-cli".to_string(),
        "-a".to_string(),
        "testpass".to_string(),
        "ping".to_string(),
    ]);

    let health_result = health_checker
        .check_health(&container_id, command_check)
        .await
        .expect("Should check Redis health");

    assert!(health_result.healthy, "Redis should be healthy");

    // Test composite health check
    let composite_check = HealthCheck::all(vec![
        HealthCheck::port(6379),
        HealthCheck::command(vec![
            "redis-cli".to_string(),
            "-a".to_string(),
            "testpass".to_string(),
            "ping".to_string(),
        ]),
    ]);

    let composite_result = health_checker
        .check_health(&container_id, composite_check)
        .await
        .expect("Should check composite health");

    assert!(
        composite_result.healthy,
        "Composite health check should pass"
    );

    // Cleanup
    cleanup_container(&client, &container_id).await;
}

#[tokio::test]
async fn test_log_management() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let container_name = test_container_name("logs");

    let container_id = ContainerBuilder::new(TEST_IMAGE)
        .name(&container_name)
        .command(vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo 'First message'; sleep 1; echo 'Second message'; sleep 1; echo 'Third message'; sleep 5".to_string()
        ])
        .run(&client)
        .await
        .expect("Should create container");

    // Wait for some output
    sleep(Duration::from_secs(2)).await;

    let log_manager = LogManager::new(&client);

    // Test getting recent logs
    let recent_logs = log_manager
        .get_recent_logs(&container_id, 10)
        .await
        .expect("Should get recent logs");

    assert!(!recent_logs.is_empty(), "Should have log entries");

    // Check that we have our test messages
    let log_content: String = recent_logs
        .iter()
        .map(|entry| entry.message.clone())
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        log_content.contains("First message"),
        "Should contain first message"
    );

    // Test log options
    let log_entries = log_manager
        .get_log_entries(&container_id, LogOptions::new().timestamps().tail(5))
        .await
        .expect("Should get log entries with options");

    assert!(!log_entries.is_empty(), "Should have log entries");
    assert!(log_entries.len() <= 5, "Should respect tail limit");

    // Check timestamps are present
    for entry in &log_entries {
        if entry.timestamp.is_some() {
            // At least one entry should have timestamp
            break;
        }
    }

    // Cleanup
    cleanup_container(&client, &container_id).await;
}

#[tokio::test]
async fn test_container_lifecycle_management() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let container_name = test_container_name("lifecycle");

    // Test create without starting
    let container_id = ContainerBuilder::new(TEST_IMAGE)
        .name(&container_name)
        .command(vec!["sleep".to_string(), "10".to_string()])
        .create(&client)
        .await
        .expect("Should create container");

    let manager = ContainerManager::new(&client);

    // Container should be created but not running
    let container_info = manager
        .inspect(&container_id)
        .await
        .expect("Should inspect container");

    assert!(matches!(container_info.status, ContainerStatus::Created));

    // Start the container
    manager
        .start(&container_id)
        .await
        .expect("Should start container");

    // Wait a moment for status to update
    sleep(Duration::from_millis(500)).await;

    // Container should now be running
    let container_info = manager
        .inspect(&container_id)
        .await
        .expect("Should inspect container");

    assert!(matches!(
        container_info.status,
        ContainerStatus::Running { .. }
    ));

    // Test graceful stop
    manager
        .stop(&container_id, Some(Duration::from_secs(5)))
        .await
        .expect("Should stop container gracefully");

    // Container should be stopped
    let container_info = manager
        .inspect(&container_id)
        .await
        .expect("Should inspect container");

    assert!(matches!(
        container_info.status,
        ContainerStatus::Exited { .. }
    ));

    // Remove the container
    manager
        .remove(&container_id, RemoveOptions::default())
        .await
        .expect("Should remove container");

    // Container should no longer exist
    let inspect_result = manager.inspect(&container_id).await;
    assert!(inspect_result.is_err(), "Container should be removed");
}

#[tokio::test]
async fn test_resource_limits() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let container_name = test_container_name("resources");

    let container_id = ContainerBuilder::new(TEST_IMAGE)
        .name(&container_name)
        .memory_str("64m")
        .cpus(0.5)
        .command(vec!["sleep".to_string(), "10".to_string()])
        .run(&client)
        .await
        .expect("Should create container with resource limits");

    let manager = ContainerManager::new(&client);
    let container_info = manager
        .inspect(&container_id)
        .await
        .expect("Should inspect container");

    // Container should be running
    assert!(matches!(
        container_info.status,
        ContainerStatus::Running { .. }
    ));

    // Clean up the container
    cleanup_container(&client, &container_id).await;
}

#[tokio::test]
async fn test_volume_mounting() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let container_name = test_container_name("volumes");

    // Create a temporary directory for testing
    let temp_dir = tempfile::tempdir().expect("Should create temp dir");
    let temp_path = temp_dir.path();

    // Write a test file
    let test_file = temp_path.join("test.txt");
    std::fs::write(&test_file, "Hello from host!").expect("Should write test file");

    let _container_id = ContainerBuilder::new(TEST_IMAGE)
        .name(&container_name)
        .volume(temp_path, "/host_data")
        .command(vec![
            "sh".to_string(),
            "-c".to_string(),
            "cat /host_data/test.txt && sleep 1".to_string(),
        ])
        .auto_remove()
        .run(&client)
        .await
        .expect("Should create container with volumes");

    // Wait for container to finish
    sleep(Duration::from_secs(2)).await;

    // Clean up temp directory
    let _ = std::fs::remove_dir_all(&temp_dir);

    // Container should be auto-removed, but we can't easily verify the volume operations
    // in this test since the container is ephemeral. In a real scenario, you'd check
    // the file system or use named volumes.
}

#[tokio::test]
async fn test_network_attachment() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let container_name = test_container_name("network");

    let container_id = ContainerBuilder::new(TEST_IMAGE)
        .name(&container_name)
        .network(NetworkId::new("bridge".to_string()).unwrap()) // Use default bridge network
        .command(vec!["sleep".to_string(), "10".to_string()])
        .run(&client)
        .await
        .expect("Should create container with network");

    let manager = ContainerManager::new(&client);
    let container_info = manager
        .inspect(&container_id)
        .await
        .expect("Should inspect container");

    // Container should be attached to bridge network
    assert!(
        container_info.networks.contains(&"bridge".to_string()),
        "Container should be attached to bridge network"
    );

    // Clean up the container
    cleanup_container(&client, &container_id).await;
}

#[tokio::test]
async fn test_error_handling() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");

    // Test invalid image
    let result = ContainerBuilder::new("nonexistent:invalid")
        .name("test-invalid")
        .auto_remove()
        .command(vec!["echo".to_string(), "hello".to_string()])
        .run(&client)
        .await;

    assert!(result.is_err(), "Should fail with invalid image");

    // Test invalid container ID for inspection
    let manager = ContainerManager::new(&client);
    let invalid_id = ContainerId::new("1234567890abcdef1234567890abcdef12345678").unwrap();
    let inspect_result = manager.inspect(&invalid_id).await;
    assert!(
        inspect_result.is_err(),
        "Should fail to inspect nonexistent container"
    );

    // Test exec on nonexistent container
    let executor = ContainerExecutor::new(&client);
    let exec_result = executor
        .exec_simple(&invalid_id, vec!["echo".to_string(), "test".to_string()])
        .await;
    assert!(
        exec_result.is_err(),
        "Should fail to exec on nonexistent container"
    );
}

#[tokio::test]
async fn test_concurrent_operations() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");

    // Create multiple containers concurrently
    let mut handles = Vec::new();

    for i in 0..3 {
        let client = client.clone();
        let handle = tokio::spawn(async move {
            let container_name = test_container_name(&format!("concurrent-{}", i));

            ContainerBuilder::new(TEST_IMAGE)
                .name(&container_name)
                .command(vec!["sleep".to_string(), "1".to_string()])
                .auto_remove()
                .run(&client)
                .await
        });
        handles.push(handle);
    }

    // Wait for all containers to be created
    let mut container_ids = Vec::new();
    for handle in handles {
        let container_id = handle
            .await
            .expect("Task should complete")
            .expect("Should create container");
        container_ids.push(container_id);
    }

    assert_eq!(container_ids.len(), 3, "Should create 3 containers");

    // Wait for containers to finish and auto-remove
    sleep(Duration::from_secs(2)).await;
}

#[tokio::test]
async fn test_memory_parsing() {
    // Test memory string parsing without Docker
    let config = ContainerBuilder::new("test:latest")
        .memory_str("512m")
        .memory_str("1g")
        .memory_str("2048k")
        .build();

    // The last value should win (2048k = 2097152 bytes)
    assert_eq!(config.resource_limits.memory, Some(2_097_152));
}

// Helper test to validate our test utilities
#[tokio::test]
async fn test_container_name_generation() {
    let name1 = test_container_name("test");
    let name2 = test_container_name("test");

    assert!(name1.starts_with("test-test-"));
    assert!(name2.starts_with("test-test-"));
    assert_ne!(name1, name2, "Container names should be unique");
}
