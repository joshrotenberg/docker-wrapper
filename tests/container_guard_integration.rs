//! Integration tests for ContainerGuard

#![cfg(feature = "testing")]

use docker_wrapper::testing::ContainerGuard;
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
