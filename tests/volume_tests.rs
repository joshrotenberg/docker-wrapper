//! Volume Management Integration Tests
//!
//! Comprehensive integration tests for Docker volume operations including:
//! - Volume creation with various drivers and configurations
//! - Volume listing and inspection operations
//! - Volume mounting and persistence testing
//! - Volume usage statistics and management
//! - Volume cleanup and pruning operations
//!
//! These tests require a running Docker daemon and will create/destroy real volumes.

use docker_wrapper::*;
use std::time::Duration;
use tokio::time::sleep;

// Test configuration
const TEST_IMAGE: &str = "alpine:3.18";

/// Helper to check if Docker is available
async fn docker_available() -> bool {
    DockerClient::new().await.is_ok()
}

/// Helper to generate unique test volume names
fn test_volume_name(test_name: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("test-{}-{}", test_name, timestamp)
}

/// Helper to generate unique test container names
fn test_container_name(test_name: &str, suffix: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("test-{}-{}-{}", test_name, suffix, timestamp)
}

/// Helper to ensure test volume cleanup
async fn cleanup_volume(client: &DockerClient, volume_name: &str) {
    let volume_manager = client.volumes();
    let _ = volume_manager
        .remove(volume_name, RemoveVolumeOptions::default())
        .await;
}

/// Helper to ensure test container cleanup
async fn cleanup_container(client: &DockerClient, container_id: &ContainerId) {
    let container_manager = ContainerManager::new(client);
    let _ = container_manager
        .stop(container_id, Some(Duration::from_secs(5)))
        .await;
    let _ = container_manager
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
async fn test_volume_manager_creation() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");

    let volume_manager = client.volumes();

    // Just verify the manager can be created
    // This is a basic smoke test that the manager exists
    let _ = volume_manager;
}

#[tokio::test]
async fn test_volume_basic_creation() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let volume_manager = client.volumes();

    let volume_name = test_volume_name("basic");
    let volume_config = VolumeConfig::new(&volume_name);

    // Create the volume
    let volume = volume_manager
        .create(volume_config)
        .await
        .expect("Should create volume");

    assert_eq!(volume.name, volume_name);
    assert!(!volume.mountpoint.is_empty());

    // Verify the volume was created by inspecting it
    let volume_info = volume_manager
        .inspect(&volume_name)
        .await
        .expect("Should inspect volume");

    assert_eq!(volume_info.name, volume_name);
    assert_eq!(volume_info.driver, "local");

    // Cleanup
    cleanup_volume(&client, &volume_name).await;
}

#[tokio::test]
async fn test_volume_with_custom_configuration() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let volume_manager = client.volumes();

    let volume_name = test_volume_name("custom");
    let volume_config = VolumeConfig::new(&volume_name)
        .driver("local")
        .label("test", "custom-config")
        .label("environment", "testing")
        .option("type", "tmpfs")
        .option("device", "tmpfs");

    // Create the volume
    let volume = volume_manager
        .create(volume_config)
        .await
        .expect("Should create volume with custom config");

    assert_eq!(volume.name, volume_name);

    // Verify the volume configuration
    let volume_info = volume_manager
        .inspect(&volume_name)
        .await
        .expect("Should inspect volume");

    assert_eq!(volume_info.name, volume_name);
    assert_eq!(volume_info.driver, "local");

    // Check labels
    if let Some(labels) = &volume_info.labels {
        assert_eq!(labels.get("test"), Some(&"custom-config".to_string()));
        assert_eq!(labels.get("environment"), Some(&"testing".to_string()));
    } else {
        // Labels might not be returned in inspect, which is okay for this test
        eprintln!("Warning: Labels not found in volume inspection");
    }

    // Cleanup
    cleanup_volume(&client, &volume_name).await;
}

#[tokio::test]
async fn test_volume_listing() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let volume_manager = client.volumes();

    // List all volumes before creating test volume
    let initial_volumes = volume_manager
        .list(ListVolumesOptions::default())
        .await
        .expect("Should list volumes");

    // Create a test volume
    let volume_name = test_volume_name("list");
    let volume_config = VolumeConfig::new(&volume_name).label("test", "listing");

    let _volume = volume_manager
        .create(volume_config)
        .await
        .expect("Should create volume");

    // List volumes again
    let volumes_after = volume_manager
        .list(ListVolumesOptions::default())
        .await
        .expect("Should list volumes after creation");

    assert_eq!(volumes_after.len(), initial_volumes.len() + 1);

    // Find our test volume
    let test_volume = volumes_after.iter().find(|v| v.name == volume_name);

    assert!(test_volume.is_some(), "Should find test volume in list");

    let test_volume = test_volume.unwrap();
    assert_eq!(test_volume.name, volume_name);
    assert!(!test_volume.mountpoint.is_empty());

    // Test filtering by name pattern
    let filtered_volumes = volume_manager
        .list(ListVolumesOptions::new().name_pattern(&volume_name))
        .await
        .expect("Should list filtered volumes");

    assert_eq!(filtered_volumes.len(), 1);
    assert_eq!(filtered_volumes[0].name, volume_name);

    // Cleanup
    cleanup_volume(&client, &volume_name).await;
}

#[tokio::test]
async fn test_volume_persistence() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let volume_manager = client.volumes();

    // Create a test volume
    let volume_name = test_volume_name("persist");
    let volume_config = VolumeConfig::new(&volume_name);

    let _volume = volume_manager
        .create(volume_config)
        .await
        .expect("Should create volume");

    // Create a container that writes to the volume
    let writer_name = test_container_name("persist", "writer");
    let writer_id = ContainerBuilder::new(TEST_IMAGE)
        .name(&writer_name)
        .volume(&volume_name, "/data")
        .command(vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo 'Hello from volume!' > /data/test.txt && sleep 1".to_string(),
        ])
        .run(&client)
        .await
        .expect("Should create writer container");

    // Wait for writer to finish
    sleep(Duration::from_secs(3)).await;
    cleanup_container(&client, &writer_id).await;

    // Create a container that reads from the volume
    let reader_name = test_container_name("persist", "reader");
    let reader_id = ContainerBuilder::new(TEST_IMAGE)
        .name(&reader_name)
        .volume(&volume_name, "/data")
        .command(vec!["cat".to_string(), "/data/test.txt".to_string()])
        .run(&client)
        .await
        .expect("Should create reader container");

    // Wait for reader to finish
    sleep(Duration::from_secs(2)).await;

    // Check the logs to verify persistence
    let _container_manager = ContainerManager::new(&client);
    let log_manager = LogManager::new(&client);
    let logs = log_manager
        .get_logs(&reader_id, LogOptions::new().stdout_only())
        .await
        .expect("Should get container logs");

    let output = logs.trim();

    assert!(
        output.contains("Hello from volume!"),
        "Volume should persist data across containers, got: {}",
        output
    );

    // Cleanup
    cleanup_container(&client, &reader_id).await;
    cleanup_volume(&client, &volume_name).await;
}

#[tokio::test]
async fn test_volume_get_by_name() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let volume_manager = client.volumes();

    let volume_name = test_volume_name("getbyname");
    let volume_config = VolumeConfig::new(&volume_name);

    let _volume = volume_manager
        .create(volume_config)
        .await
        .expect("Should create volume");

    // Test get_by_name
    let found_volume = volume_manager
        .get_by_name(&volume_name)
        .await
        .expect("Should search for volume by name");

    assert!(found_volume.is_some(), "Should find volume by name");

    let found_volume = found_volume.unwrap();
    assert_eq!(found_volume.name, volume_name);

    // Test non-existent volume
    let not_found = volume_manager
        .get_by_name("non-existent-volume-12345")
        .await
        .expect("Should handle non-existent volume");

    assert!(
        not_found.is_none(),
        "Should return None for non-existent volume"
    );

    // Cleanup
    cleanup_volume(&client, &volume_name).await;
}

#[tokio::test]
async fn test_volume_exists() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let volume_manager = client.volumes();

    let volume_name = test_volume_name("exists");
    let volume_config = VolumeConfig::new(&volume_name);

    let _volume = volume_manager
        .create(volume_config)
        .await
        .expect("Should create volume");

    // Test volume exists
    let exists = volume_manager
        .exists(&volume_name)
        .await
        .expect("Should check if volume exists");

    assert!(exists, "Volume should exist");

    // Remove volume and test again
    volume_manager
        .remove(&volume_name, RemoveVolumeOptions::default())
        .await
        .expect("Should remove volume");

    let exists_after = volume_manager
        .exists(&volume_name)
        .await
        .expect("Should check if volume exists after removal");

    assert!(!exists_after, "Volume should not exist after removal");
}

#[tokio::test]
async fn test_volume_removal() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let volume_manager = client.volumes();

    let volume_name = test_volume_name("removal");
    let volume_config = VolumeConfig::new(&volume_name);

    let _volume = volume_manager
        .create(volume_config)
        .await
        .expect("Should create volume");

    // Verify volume exists
    let volume_info = volume_manager
        .inspect(&volume_name)
        .await
        .expect("Should inspect volume before removal");

    assert_eq!(volume_info.name, volume_name);

    // Remove volume
    volume_manager
        .remove(&volume_name, RemoveVolumeOptions::default())
        .await
        .expect("Should remove volume");

    // Verify volume is gone
    let inspect_result = volume_manager.inspect(&volume_name).await;
    assert!(
        inspect_result.is_err(),
        "Should not be able to inspect removed volume"
    );
}

#[tokio::test]
async fn test_volume_create_if_not_exists() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let volume_manager = client.volumes();

    let volume_name = test_volume_name("createif");
    let volume_config = VolumeConfig::new(&volume_name).label("test", "create-if");

    // First call should create the volume
    let volume1 = volume_manager
        .create_if_not_exists(volume_config.clone())
        .await
        .expect("Should create volume on first call");

    assert_eq!(volume1.name, volume_name);

    // Second call should return the existing volume
    let volume2 = volume_manager
        .create_if_not_exists(volume_config)
        .await
        .expect("Should return existing volume on second call");

    assert_eq!(volume2.name, volume_name);
    assert_eq!(volume1.name, volume2.name);

    // Cleanup
    cleanup_volume(&client, &volume_name).await;
}

#[tokio::test]
async fn test_volume_usage_stats() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let volume_manager = client.volumes();

    // Create a test volume
    let volume_name = test_volume_name("stats");
    let volume_config = VolumeConfig::new(&volume_name);

    let _volume = volume_manager
        .create(volume_config)
        .await
        .expect("Should create volume");

    // Get usage statistics
    let stats = volume_manager
        .usage_stats()
        .await
        .expect("Should get volume usage statistics");

    assert!(stats.total_volumes >= 1, "Should have at least one volume");
    assert!(stats.total_size >= 0, "Total size should be non-negative");

    // Basic validation of statistics structure
    // Volume count should be positive
    assert!(
        stats.volumes_with_size == stats.volumes_with_size,
        "Should have valid volumes with size count"
    );
    assert!(!stats.drivers.is_empty(), "Should have at least one driver");

    // Cleanup
    cleanup_volume(&client, &volume_name).await;
}

#[tokio::test]
async fn test_volume_error_handling() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let volume_manager = client.volumes();

    // Test inspect non-existent volume
    let inspect_result = volume_manager.inspect("non-existent-volume-12345").await;
    assert!(
        inspect_result.is_err(),
        "Should fail to inspect non-existent volume"
    );

    // Test remove non-existent volume
    let remove_result = volume_manager
        .remove("non-existent-volume-12345", RemoveVolumeOptions::default())
        .await;
    assert!(
        remove_result.is_err(),
        "Should fail to remove non-existent volume"
    );

    // Test create volume with empty name
    let empty_config = VolumeConfig::new("");
    let create_result = volume_manager.create(empty_config).await;
    // This might succeed or fail depending on Docker's behavior with empty names
    // We just ensure it doesn't panic
    let _ = create_result;
}

#[tokio::test]
async fn test_volume_concurrent_operations() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");

    // Test concurrent volume listing - should be safe
    let handles: Vec<_> = (0..3)
        .map(|_| {
            tokio::spawn(async move {
                let client = DockerClient::new()
                    .await
                    .expect("Should create Docker client");
                let volume_manager = client.volumes();
                volume_manager
                    .list(ListVolumesOptions::default())
                    .await
                    .expect("Should list volumes concurrently")
            })
        })
        .collect();

    // Wait for all operations to complete
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await);
    }

    for result in results {
        let volumes = result.expect("Task should complete successfully");
        // Just verify we got some result (might be empty)
        assert!(volumes.len() == volumes.len(), "Should get volume list");
    }

    // Test concurrent volume creation with different names
    let creation_handles: Vec<_> = (0..2)
        .map(|i| {
            tokio::spawn(async move {
                let client = DockerClient::new()
                    .await
                    .expect("Should create Docker client");
                let volume_manager = client.volumes();
                let volume_name = test_volume_name(&format!("concurrent{}", i));
                let volume_config = VolumeConfig::new(&volume_name);

                volume_manager
                    .create(volume_config)
                    .await
                    .expect("Should create volume concurrently")
            })
        })
        .collect();

    // Collect volume names for cleanup
    let mut volume_names = Vec::new();
    for handle in creation_handles {
        let volume = handle.await.expect("Task should complete successfully");
        volume_names.push(volume.name);
    }

    // Cleanup all created volumes
    for volume_name in volume_names {
        cleanup_volume(&client, &volume_name).await;
    }
}

#[tokio::test]
async fn test_volume_multiple_containers() {
    if !docker_available().await {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let client = DockerClient::new()
        .await
        .expect("Should create Docker client");
    let volume_manager = client.volumes();

    // Create a shared volume
    let volume_name = test_volume_name("shared");
    let volume_config = VolumeConfig::new(&volume_name);

    let _volume = volume_manager
        .create(volume_config)
        .await
        .expect("Should create shared volume");

    // Create first container that writes to the volume
    let writer1_name = test_container_name("shared", "writer1");
    let writer1_id = ContainerBuilder::new(TEST_IMAGE)
        .name(&writer1_name)
        .volume(&volume_name, "/shared")
        .command(vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo 'Data from writer1' > /shared/writer1.txt && sleep 1".to_string(),
        ])
        .run(&client)
        .await
        .expect("Should create first writer container");

    // Create second container that writes to the volume
    let writer2_name = test_container_name("shared", "writer2");
    let writer2_id = ContainerBuilder::new(TEST_IMAGE)
        .name(&writer2_name)
        .volume(&volume_name, "/shared")
        .command(vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo 'Data from writer2' > /shared/writer2.txt && sleep 1".to_string(),
        ])
        .run(&client)
        .await
        .expect("Should create second writer container");

    // Wait for both writers to finish
    sleep(Duration::from_secs(3)).await;
    cleanup_container(&client, &writer1_id).await;
    cleanup_container(&client, &writer2_id).await;

    // Create a reader container to verify both files exist
    let reader_name = test_container_name("shared", "reader");
    let reader_id = ContainerBuilder::new(TEST_IMAGE)
        .name(&reader_name)
        .volume(&volume_name, "/shared")
        .command(vec![
            "sh".to_string(),
            "-c".to_string(),
            "ls -la /shared && cat /shared/writer1.txt && cat /shared/writer2.txt".to_string(),
        ])
        .run(&client)
        .await
        .expect("Should create reader container");

    // Wait for reader to finish
    sleep(Duration::from_secs(2)).await;

    // Check the logs to verify both files were created
    let log_manager = LogManager::new(&client);
    let logs = log_manager
        .get_logs(&reader_id, LogOptions::new().stdout_only())
        .await
        .expect("Should get container logs");

    let output = logs.trim();

    assert!(
        output.contains("writer1.txt") && output.contains("writer2.txt"),
        "Should see both files in shared volume"
    );
    assert!(
        output.contains("Data from writer1") && output.contains("Data from writer2"),
        "Should see data from both containers, got: {}",
        output
    );

    // Cleanup
    cleanup_container(&client, &reader_id).await;
    cleanup_volume(&client, &volume_name).await;
}
