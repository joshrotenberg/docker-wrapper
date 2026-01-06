//! # Testing Utilities Example
//!
//! This example demonstrates how to use docker-wrapper's testing utilities
//! for integration testing with containers. These utilities provide RAII-style
//! lifecycle management, ensuring containers are automatically cleaned up.
//!
//! ## Features Demonstrated
//!
//! - `ContainerGuard`: Single container lifecycle management
//! - `ContainerGuardSet`: Multi-container test environments
//! - Automatic cleanup on drop (even on panic)
//! - Ready checks and connection strings
//! - Debug mode for failed tests
//!
//! ## Running the Examples
//!
//! Run specific tests:
//! ```bash
//! cargo test --example testing_utilities --features testing
//! ```
//!
//! Run with output:
//! ```bash
//! cargo test --example testing_utilities --features testing -- --nocapture
//! ```

#[cfg(test)]
use docker_wrapper::testing::{ContainerGuard, ContainerGuardSet};
#[cfg(test)]
use docker_wrapper::{RedisTemplate, Template};
#[cfg(test)]
use std::sync::atomic::{AtomicU16, Ordering};
#[cfg(test)]
use std::time::Duration;

/// Generate unique container names for parallel test safety.
#[cfg(test)]
fn unique_name(prefix: &str) -> String {
    format!("{}-{}", prefix, uuid::Uuid::new_v4().simple())
}

/// Generate unique ports for parallel test safety.
/// Starts at 40000 and increments for each call.
#[cfg(test)]
fn unique_port() -> u16 {
    static PORT: AtomicU16 = AtomicU16::new(40000);
    PORT.fetch_add(1, Ordering::SeqCst)
}

// ============================================================================
// Basic Usage
// ============================================================================

/// Basic container guard usage with automatic cleanup.
///
/// The container is automatically stopped and removed when the guard
/// goes out of scope, even if the test panics.
#[tokio::test]
async fn test_basic_container_guard() {
    let name = unique_name("basic-redis");
    let port = unique_port();

    // Create and start a Redis container with automatic lifecycle management
    let guard = ContainerGuard::new(RedisTemplate::new(&name).port(port))
        .wait_for_ready(true) // Wait for Redis to accept connections
        .start()
        .await
        .expect("Failed to start Redis container");

    // Container is ready - verify it's running
    assert!(
        guard.is_running().await.expect("Failed to check status"),
        "Container should be running"
    );

    // Access the underlying template if needed
    let template = guard.template();
    assert_eq!(template.config().name, name);

    // Container is automatically stopped and removed here
}

/// Using connection strings for easy service access.
#[tokio::test]
async fn test_connection_string() {
    let name = unique_name("conn-redis");
    let port = unique_port();

    let guard = ContainerGuard::new(RedisTemplate::new(&name).port(port))
        .wait_for_ready(true)
        .start()
        .await
        .expect("Failed to start Redis");

    // Get the connection string directly from the guard
    let conn = guard.connection_string();
    assert!(
        conn.starts_with("redis://"),
        "Should be a Redis connection URL"
    );
    assert!(
        conn.contains(&port.to_string()),
        "Should include the configured port"
    );

    println!("Redis available at: {}", conn);
}

// ============================================================================
// Configuration Options
// ============================================================================

/// Fast cleanup with custom stop timeout.
///
/// By default Docker waits 10 seconds for graceful shutdown. Use a shorter
/// timeout for faster test cleanup when you don't need graceful shutdown.
#[tokio::test]
async fn test_fast_cleanup() {
    let name = unique_name("fast-redis");
    let port = unique_port();

    let guard = ContainerGuard::new(RedisTemplate::new(&name).port(port))
        .wait_for_ready(true)
        .stop_timeout(Duration::from_secs(1)) // Fast 1-second timeout
        .start()
        .await
        .expect("Failed to start Redis");

    assert!(guard.is_running().await.unwrap());

    // Cleanup will be fast due to short timeout
}

/// Keep containers running for debugging failed tests.
///
/// When `keep_on_panic(true)` is set, the container won't be removed if
/// the test panics, allowing you to inspect its state for debugging.
#[tokio::test]
async fn test_debug_mode() {
    let name = unique_name("debug-redis");
    let port = unique_port();

    let guard = ContainerGuard::new(RedisTemplate::new(&name).port(port))
        .keep_on_panic(true) // Keep container if test panics
        .capture_logs(true) // Print logs on panic
        .wait_for_ready(true)
        .start()
        .await
        .expect("Failed to start Redis");

    // If this test panicked, the container would stay running
    // and logs would be printed to stderr for debugging
    assert!(guard.is_running().await.unwrap());

    // Normal exit - container is cleaned up
}

/// Container reuse for faster local development.
///
/// With `reuse_if_running(true)`, if a container with the same name is
/// already running, it will be reused instead of starting a new one.
#[tokio::test]
async fn test_container_reuse() {
    // Use a fixed name and port for reuse demonstration
    let name = unique_name("reuse-redis");
    let port = unique_port();

    // First run - starts a new container
    let guard1 = ContainerGuard::new(RedisTemplate::new(&name).port(port))
        .reuse_if_running(true)
        .stop_on_drop(false) // Don't stop so next test can reuse
        .remove_on_drop(false) // Don't remove so next test can reuse
        .wait_for_ready(true)
        .start()
        .await
        .expect("Failed to start Redis");

    let was_reused_first = guard1.was_reused();
    println!("First guard - was_reused: {}", was_reused_first);

    // Second guard - should reuse the running container
    let guard2 = ContainerGuard::new(RedisTemplate::new(&name).port(port))
        .reuse_if_running(true)
        .wait_for_ready(true)
        .start()
        .await
        .expect("Failed to start/reuse Redis");

    // This time it should be reused (if container was still running)
    println!("Second guard - was_reused: {}", guard2.was_reused());
    assert!(guard2.was_reused(), "Second guard should reuse container");

    // guard2 will clean up since we didn't disable cleanup
}

/// Manual cleanup when you need explicit control.
#[tokio::test]
async fn test_manual_cleanup() {
    let name = unique_name("manual-redis");
    let port = unique_port();

    let guard = ContainerGuard::new(RedisTemplate::new(&name).port(port))
        .wait_for_ready(true)
        .start()
        .await
        .expect("Failed to start Redis");

    // Do some work...
    assert!(guard.is_running().await.unwrap());

    // Explicitly clean up before the guard goes out of scope
    guard.cleanup().await.expect("Failed to cleanup");

    // Guard will not try to clean up again on drop (idempotent)
}

// ============================================================================
// Multi-Container Tests
// ============================================================================

/// Using ContainerGuardSet for multi-container test environments.
///
/// This is useful when your test needs multiple services that may need
/// to communicate with each other.
#[tokio::test]
async fn test_multi_container_set() {
    let network = unique_name("test-net");
    let redis1_name = unique_name("redis-primary");
    let redis2_name = unique_name("redis-replica");
    let port1 = unique_port();
    let port2 = unique_port();

    // Create a set with multiple containers on a shared network
    let guards = ContainerGuardSet::new()
        .with_network(&network) // Containers can communicate via this network
        .add(RedisTemplate::new(&redis1_name).port(port1))
        .add(RedisTemplate::new(&redis2_name).port(port2))
        .keep_on_panic(true) // Keep all containers for debugging
        .start_all()
        .await
        .expect("Failed to start container set");

    // Verify both containers are in the set
    assert!(guards.contains(&redis1_name));
    assert!(guards.contains(&redis2_name));
    assert_eq!(guards.len(), 2);

    // Get the shared network
    assert_eq!(guards.network(), Some(network.as_str()));

    // List all container names
    println!("Running containers:");
    for name in guards.names() {
        println!("  - {}", name);
    }

    // All containers and the network are cleaned up when guards is dropped
}

/// ContainerGuardSet with custom options.
#[tokio::test]
async fn test_guard_set_options() {
    let network = unique_name("options-net");
    let port = unique_port();

    let guards = ContainerGuardSet::new()
        .with_network(&network)
        .create_network(true) // Create network if it doesn't exist (default)
        .remove_network_on_drop(true) // Clean up network after test (default)
        .wait_for_ready(true) // Wait for all containers to be ready (default)
        .keep_on_panic(false) // Clean up even on panic
        .add(RedisTemplate::new(&unique_name("opt-redis")).port(port))
        .start_all()
        .await
        .expect("Failed to start");

    assert_eq!(guards.len(), 1);
    assert!(!guards.is_empty());
}

// ============================================================================
// Accessing Container Information
// ============================================================================

/// Accessing container details and logs.
#[tokio::test]
async fn test_container_info() {
    let name = unique_name("info-redis");
    let port = unique_port();

    let guard = ContainerGuard::new(RedisTemplate::new(&name).port(port))
        .wait_for_ready(true)
        .start()
        .await
        .expect("Failed to start Redis");

    // Get container ID (if available)
    if let Some(id) = guard.container_id() {
        println!("Container ID: {}", id);
        assert!(!id.is_empty());
    }

    // Get connection string (includes the port)
    let conn = guard.connection_string();
    println!("Connection string: {}", conn);
    assert!(conn.contains(&port.to_string()));

    // Query host port for a container port
    let host_port = guard
        .host_port(6379)
        .await
        .expect("Failed to get host port");
    println!(
        "Redis container port 6379 mapped to host port: {}",
        host_port
    );
    assert_eq!(host_port, port);

    // Get container logs
    let logs = guard.logs().await.expect("Failed to get logs");
    println!("Container logs:\n{}", logs);
    assert!(logs.contains("Ready to accept connections"));

    // Check running status
    assert!(guard.is_running().await.expect("Failed to check status"));
}

// ============================================================================
// Test Fixtures Pattern
// ============================================================================

/// Create reusable test fixtures for common setups.
#[cfg(test)]
async fn redis_fixture(name: &str, port: u16) -> ContainerGuard<RedisTemplate> {
    ContainerGuard::new(RedisTemplate::new(name).port(port))
        .wait_for_ready(true)
        .keep_on_panic(true)
        .capture_logs(true)
        .stop_timeout(Duration::from_secs(2))
        .start()
        .await
        .expect("Failed to create Redis fixture")
}

/// Using the test fixture pattern.
#[tokio::test]
async fn test_with_fixture() {
    let name = unique_name("fixture-redis");
    let port = unique_port();
    let redis = redis_fixture(&name, port).await;

    // Fixture is ready to use
    let conn = redis.connection_string();
    println!("Fixture ready at: {}", conn);

    // Clean up is automatic
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    println!("Testing Utilities Example");
    println!("=========================");
    println!();
    println!("This example demonstrates docker-wrapper's testing utilities");
    println!("for integration testing with automatic container lifecycle management.");
    println!();
    println!("Run the tests with:");
    println!("  cargo test --example testing_utilities --features testing");
    println!();
    println!("Run with output:");
    println!("  cargo test --example testing_utilities --features testing -- --nocapture");
    println!();
    println!("Features demonstrated:");
    println!("  - ContainerGuard: Single container RAII management");
    println!("  - ContainerGuardSet: Multi-container test environments");
    println!("  - Automatic cleanup on drop and panic");
    println!("  - Ready checks and connection strings");
    println!("  - Debug mode for failed tests");
    println!("  - Container reuse for faster development");
    println!("  - Test fixture patterns");
}
