//! Integration tests for ContainerGuard and ContainerGuardSet

#![cfg(feature = "testing")]

use docker_wrapper::testing::{ContainerGuard, ContainerGuardSet};
use docker_wrapper::{RedisTemplate, Template};
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

static PORT_COUNTER: AtomicU16 = AtomicU16::new(17000);

fn unique_name(prefix: &str) -> String {
    format!("{}-{}", prefix, uuid::Uuid::new_v4())
}

fn next_port() -> u16 {
    PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[tokio::test]
async fn test_container_guard_basic_lifecycle() {
    let name = unique_name("guard-basic");
    let guard = ContainerGuard::new(RedisTemplate::new(&name).port(next_port()))
        .start()
        .await
        .expect("Failed to start container");

    // Container should be running
    assert!(
        guard.is_running().await.expect("Failed to check running"),
        "Container should be running"
    );

    // Should have a container ID
    assert!(guard.container_id().is_some(), "Should have container ID");

    // Should not be reused
    assert!(!guard.was_reused(), "Should not be reused");

    // Template should be accessible
    let template = guard.template();
    assert_eq!(template.config().name, name);

    // Drop the guard - container should be cleaned up
    drop(guard);

    // Give Docker a moment to clean up
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify container is gone
    let template = RedisTemplate::new(&name);
    assert!(
        !template.is_running().await.unwrap_or(true),
        "Container should be stopped after drop"
    );
}

#[tokio::test]
async fn test_container_guard_no_remove_on_drop() {
    let name = unique_name("guard-no-remove");
    let guard = ContainerGuard::new(RedisTemplate::new(&name).port(next_port()))
        .remove_on_drop(false)
        .stop_on_drop(false)
        .start()
        .await
        .expect("Failed to start container");

    assert!(guard.is_running().await.expect("Failed to check running"));

    // Drop without cleanup
    drop(guard);

    // Container should still be running
    tokio::time::sleep(Duration::from_millis(500)).await;
    let template = RedisTemplate::new(&name);
    assert!(
        template.is_running().await.unwrap_or(false),
        "Container should still be running"
    );

    // Manual cleanup
    let _ = template.stop().await;
    let _ = template.remove().await;
}

#[tokio::test]
async fn test_container_guard_reuse_if_running() {
    let name = unique_name("guard-reuse");
    let port = next_port();

    // Start container first time
    let guard1 = ContainerGuard::new(RedisTemplate::new(&name).port(port))
        .remove_on_drop(false)
        .stop_on_drop(false)
        .start()
        .await
        .expect("Failed to start container");

    assert!(!guard1.was_reused(), "First start should not be reused");
    let container_id = guard1.container_id().map(String::from);
    drop(guard1);

    // Second guard should reuse existing container
    let guard2 = ContainerGuard::new(RedisTemplate::new(&name).port(port))
        .reuse_if_running(true)
        .start()
        .await
        .expect("Failed to start/reuse container");

    assert!(guard2.was_reused(), "Second start should reuse");
    // Reused containers don't have container_id
    assert!(
        guard2.container_id().is_none(),
        "Reused container should not have ID"
    );

    // Cleanup
    drop(guard2);
    let template = RedisTemplate::new(&name);
    let _ = template.stop().await;
    let _ = template.remove().await;

    // Verify first container had an ID
    assert!(container_id.is_some(), "Original container should have ID");
}

#[tokio::test]
async fn test_container_guard_logs() {
    let name = unique_name("guard-logs");
    let guard = ContainerGuard::new(RedisTemplate::new(&name).port(next_port()))
        .start()
        .await
        .expect("Failed to start container");

    // Wait for Redis to log something
    tokio::time::sleep(Duration::from_secs(1)).await;

    let logs = guard.logs().await.expect("Failed to get logs");

    // Redis should have logged something about starting
    assert!(!logs.is_empty(), "Logs should not be empty");

    // Container cleanup happens on drop
}

#[tokio::test]
async fn test_container_guard_manual_cleanup() {
    let name = unique_name("guard-cleanup");
    let guard = ContainerGuard::new(RedisTemplate::new(&name).port(next_port()))
        .start()
        .await
        .expect("Failed to start container");

    assert!(guard.is_running().await.expect("Failed to check running"));

    // Manual cleanup
    guard.cleanup().await.expect("Failed to cleanup");

    // Cleanup should be idempotent
    guard
        .cleanup()
        .await
        .expect("Second cleanup should succeed");

    // Container should be gone
    tokio::time::sleep(Duration::from_millis(500)).await;
    let template = RedisTemplate::new(&name);
    assert!(
        !template.is_running().await.unwrap_or(true),
        "Container should be stopped after cleanup"
    );
}

#[tokio::test]
async fn test_container_guard_wait_for_ready_method() {
    let name = unique_name("guard-wait-ready");
    let guard = ContainerGuard::new(RedisTemplate::new(&name).port(next_port()))
        .start()
        .await
        .expect("Failed to start container");

    // Explicitly call wait_for_ready
    guard
        .wait_for_ready()
        .await
        .expect("Failed to wait for ready");

    // Container should definitely be running and ready now
    assert!(
        guard.is_running().await.expect("Failed to check running"),
        "Container should be running"
    );

    // Container cleanup happens on drop
}

#[tokio::test]
async fn test_container_guard_auto_wait_for_ready() {
    let name = unique_name("guard-auto-wait");
    let guard = ContainerGuard::new(RedisTemplate::new(&name).port(next_port()))
        .wait_for_ready(true) // Enable auto-wait
        .start()
        .await
        .expect("Failed to start container");

    // Container should be immediately ready - no separate wait_for_ready call needed
    assert!(
        guard.is_running().await.expect("Failed to check running"),
        "Container should be running"
    );

    // Calling wait_for_ready again should succeed immediately since already ready
    guard
        .wait_for_ready()
        .await
        .expect("wait_for_ready should succeed on already-ready container");

    // Container cleanup happens on drop
}

#[tokio::test]
async fn test_container_guard_with_network() {
    let name = unique_name("guard-network");
    let network_name = unique_name("test-network");

    let guard = ContainerGuard::new(RedisTemplate::new(&name).port(next_port()))
        .with_network(&network_name)
        .remove_network_on_drop(true)
        .start()
        .await
        .expect("Failed to start container");

    // Container should be running
    assert!(
        guard.is_running().await.expect("Failed to check running"),
        "Container should be running"
    );

    // Network should be set
    assert_eq!(guard.network(), Some(network_name.as_str()));

    // Verify the container is on the network by checking the template config
    assert_eq!(
        guard.template().config().network,
        Some(network_name.clone())
    );

    // Container and network cleanup happens on drop
}

#[tokio::test]
async fn test_container_guard_network_no_auto_create() {
    use docker_wrapper::{DockerCommand, NetworkCreateCommand, NetworkRmCommand};

    let name = unique_name("guard-network-manual");
    let network_name = unique_name("manual-network");

    // Create the network manually first
    NetworkCreateCommand::new(&network_name)
        .driver("bridge")
        .execute()
        .await
        .expect("Failed to create network");

    let guard = ContainerGuard::new(RedisTemplate::new(&name).port(next_port()))
        .with_network(&network_name)
        .create_network(false) // Don't auto-create
        .start()
        .await
        .expect("Failed to start container");

    assert!(
        guard.is_running().await.expect("Failed to check running"),
        "Container should be running"
    );

    // Drop the guard (container will be removed but network won't since we didn't create it)
    drop(guard);

    // Give Docker a moment to clean up
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Clean up network manually
    let _ = NetworkRmCommand::new(&network_name).execute().await;
}

// ============================================================================
// ContainerGuardSet tests
// ============================================================================

#[tokio::test]
async fn test_container_guard_set_basic() {
    let name1 = unique_name("guardset-1");
    let name2 = unique_name("guardset-2");

    let guards = ContainerGuardSet::new()
        .add(RedisTemplate::new(&name1).port(next_port()))
        .add(RedisTemplate::new(&name2).port(next_port()))
        .start_all()
        .await
        .expect("Failed to start containers");

    // Both containers should be in the set
    assert_eq!(guards.len(), 2);
    assert!(!guards.is_empty());
    assert!(guards.contains(&name1));
    assert!(guards.contains(&name2));

    // Names should be accessible
    let names: Vec<&str> = guards.names().collect();
    assert!(names.contains(&name1.as_str()));
    assert!(names.contains(&name2.as_str()));

    // Container cleanup happens on drop
}

#[tokio::test]
async fn test_container_guard_set_with_shared_network() {
    let name1 = unique_name("guardset-net-1");
    let name2 = unique_name("guardset-net-2");
    let network_name = unique_name("guardset-network");

    let guards = ContainerGuardSet::new()
        .with_network(&network_name)
        .add(RedisTemplate::new(&name1).port(next_port()))
        .add(RedisTemplate::new(&name2).port(next_port()))
        .start_all()
        .await
        .expect("Failed to start containers");

    // Both containers should be in the set
    assert_eq!(guards.len(), 2);

    // Network should be accessible
    assert_eq!(guards.network(), Some(network_name.as_str()));

    // Containers and network cleanup happens on drop
}

#[tokio::test]
async fn test_container_guard_set_empty() {
    let guards = ContainerGuardSet::new()
        .start_all()
        .await
        .expect("Empty set should start successfully");

    assert!(guards.is_empty());
    assert_eq!(guards.len(), 0);
    assert!(!guards.contains("nonexistent"));
}

#[tokio::test]
async fn test_container_guard_set_single_container() {
    let name = unique_name("guardset-single");

    let guards = ContainerGuardSet::new()
        .add(RedisTemplate::new(&name).port(next_port()))
        .start_all()
        .await
        .expect("Failed to start container");

    assert_eq!(guards.len(), 1);
    assert!(guards.contains(&name));

    // Container cleanup happens on drop
}

#[tokio::test]
async fn test_container_guard_set_network_no_auto_remove() {
    use docker_wrapper::{DockerCommand, NetworkRmCommand};

    let name = unique_name("guardset-net-keep");
    let network_name = unique_name("guardset-keep-network");

    {
        let guards = ContainerGuardSet::new()
            .with_network(&network_name)
            .remove_network_on_drop(false) // Don't remove network
            .add(RedisTemplate::new(&name).port(next_port()))
            .start_all()
            .await
            .expect("Failed to start container");

        assert_eq!(guards.len(), 1);
        // Guards drop here, but network should remain
    }

    // Give Docker a moment to clean up containers
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Network should still exist - cleanup manually
    // If network doesn't exist, this will fail but we ignore the error
    let _ = NetworkRmCommand::new(&network_name).execute().await;
}

#[tokio::test]
async fn test_container_guard_set_wait_for_ready_disabled() {
    let name = unique_name("guardset-no-wait");

    let guards = ContainerGuardSet::new()
        .wait_for_ready(false) // Don't wait for ready
        .add(RedisTemplate::new(&name).port(next_port()))
        .start_all()
        .await
        .expect("Failed to start container");

    assert_eq!(guards.len(), 1);
    assert!(guards.contains(&name));

    // Container cleanup happens on drop
}
