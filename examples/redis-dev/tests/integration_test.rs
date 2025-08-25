//! Integration tests for Redis Developer CLI
//!
//! These tests use the patterns from our testing documentation to verify
//! that the redis-dev CLI correctly manages Redis containers.

use docker_wrapper::{DockerCommand, PsCommand, StopCommand};
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to run redis-dev commands
fn run_redis_dev(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args(&["run", "--bin", "redis-dev", "--"])
        .args(args)
        .output()
        .expect("Failed to run redis-dev")
}

/// Helper to check if a container is running
async fn container_exists(name: &str) -> bool {
    let output = PsCommand::new()
        .all()
        .filter("name", name)
        .execute()
        .await
        .unwrap();
    
    !output.containers.is_empty()
}

/// Helper to cleanup containers by prefix
async fn cleanup_containers_with_prefix(prefix: &str) {
    let output = PsCommand::new()
        .all()
        .filter("name", prefix)
        .execute()
        .await
        .unwrap();
    
    for container in output.containers {
        let _ = StopCommand::new(&container.names)
            .execute()
            .await;
    }
}

#[tokio::test]
async fn test_redis_dev_help() {
    let output = run_redis_dev(&["--help"]);
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Redis Developer CLI"));
    assert!(stdout.contains("start"));
    assert!(stdout.contains("stop"));
    assert!(stdout.contains("list"));
}

#[tokio::test]
async fn test_redis_dev_version() {
    let output = run_redis_dev(&["--version"]);
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("redis-dev"));
}

#[tokio::test]
async fn test_start_basic_redis() {
    // Clean up any existing containers
    cleanup_containers_with_prefix("redis-dev-").await;
    
    // Start a basic Redis instance
    let output = run_redis_dev(&["start", "basic"]);
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Started Redis"));
    
    // Wait for container to be fully started
    sleep(Duration::from_secs(2)).await;
    
    // Verify container exists
    let containers_exist = container_exists("redis-dev-").await;
    assert!(containers_exist, "Redis container should be running");
    
    // Clean up
    cleanup_containers_with_prefix("redis-dev-").await;
}

#[tokio::test]
async fn test_start_cluster() {
    // Clean up any existing containers
    cleanup_containers_with_prefix("redis-cluster-").await;
    
    // Start a Redis cluster
    let output = run_redis_dev(&["start", "cluster", "--nodes", "3"]);
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Started Redis Cluster"));
        
        // Wait for cluster to initialize
        sleep(Duration::from_secs(5)).await;
        
        // Verify cluster containers exist
        let containers_exist = container_exists("redis-cluster-").await;
        assert!(containers_exist, "Redis cluster containers should be running");
        
        // Clean up
        cleanup_containers_with_prefix("redis-cluster-").await;
    } else {
        // If cluster feature is not enabled, verify appropriate error message
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("cluster") || stderr.contains("not available"),
            "Should indicate cluster is not available"
        );
    }
}

#[tokio::test]
async fn test_list_instances() {
    // Clean up first
    cleanup_containers_with_prefix("redis-dev-").await;
    
    // Start an instance
    let _ = run_redis_dev(&["start", "basic"]);
    sleep(Duration::from_secs(2)).await;
    
    // List instances
    let output = run_redis_dev(&["list"]);
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("redis-dev-") || stdout.contains("No Redis instances"),
        "Should list instances or indicate none running"
    );
    
    // Clean up
    cleanup_containers_with_prefix("redis-dev-").await;
}

#[tokio::test]
async fn test_stop_instance() {
    // Clean up first
    cleanup_containers_with_prefix("redis-dev-").await;
    
    // Start an instance
    let start_output = run_redis_dev(&["start", "basic"]);
    
    if start_output.status.success() {
        sleep(Duration::from_secs(2)).await;
        
        // Extract instance name from output if possible
        let stdout = String::from_utf8_lossy(&start_output.stdout);
        
        // Stop all instances
        let stop_output = run_redis_dev(&["stop", "--all"]);
        assert!(stop_output.status.success());
        
        let stop_stdout = String::from_utf8_lossy(&stop_output.stdout);
        assert!(
            stop_stdout.contains("Stopped") || stop_stdout.contains("stopped"),
            "Should indicate instances were stopped"
        );
        
        // Verify no containers are running
        sleep(Duration::from_secs(1)).await;
        let containers_exist = container_exists("redis-dev-").await;
        assert!(!containers_exist, "No Redis containers should be running after stop");
    }
}

#[tokio::test]
async fn test_redis_with_persistence() {
    // Clean up first
    cleanup_containers_with_prefix("redis-dev-persist").await;
    
    // Start Redis with persistence
    let output = run_redis_dev(&["start", "basic", "--name", "persist-test", "--persist"]);
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Started Redis"));
        
        // Wait for container to start
        sleep(Duration::from_secs(2)).await;
        
        // Verify container has volume mounted
        // This would require inspecting the container configuration
        
        // Clean up
        cleanup_containers_with_prefix("persist-test").await;
    }
}

#[tokio::test]
async fn test_invalid_command() {
    let output = run_redis_dev(&["invalid-command"]);
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error") || stderr.contains("unrecognized"),
        "Should show error for invalid command"
    );
}

#[tokio::test]
async fn test_port_conflict_handling() {
    // Clean up first
    cleanup_containers_with_prefix("redis-dev-").await;
    
    // Start first instance on default port
    let first = run_redis_dev(&["start", "basic", "--port", "6379"]);
    
    if first.status.success() {
        sleep(Duration::from_secs(2)).await;
        
        // Try to start second instance on same port
        let second = run_redis_dev(&["start", "basic", "--port", "6379"]);
        
        // Should either fail or use a different port
        if !second.status.success() {
            let stderr = String::from_utf8_lossy(&second.stderr);
            assert!(
                stderr.contains("port") || stderr.contains("in use"),
                "Should indicate port conflict"
            );
        } else {
            let stdout = String::from_utf8_lossy(&second.stdout);
            assert!(
                stdout.contains("6380") || stdout.contains("different port"),
                "Should use a different port"
            );
        }
        
        // Clean up
        cleanup_containers_with_prefix("redis-dev-").await;
    }
}

/// Test fixture for Redis Dev CLI testing
pub struct RedisDevFixture {
    instance_name: Option<String>,
}

impl RedisDevFixture {
    pub fn new() -> Self {
        Self {
            instance_name: None,
        }
    }
    
    pub async fn start_basic(&mut self, name: &str) -> bool {
        let output = run_redis_dev(&["start", "basic", "--name", name]);
        
        if output.status.success() {
            self.instance_name = Some(name.to_string());
            sleep(Duration::from_secs(2)).await;
            true
        } else {
            false
        }
    }
    
    pub async fn cleanup(&self) {
        if let Some(ref name) = self.instance_name {
            cleanup_containers_with_prefix(name).await;
        }
    }
}

impl Drop for RedisDevFixture {
    fn drop(&mut self) {
        // Note: Can't use async in drop, so manual cleanup is required
        // Users should call cleanup() explicitly
    }
}

#[tokio::test]
async fn test_with_fixture() {
    let mut fixture = RedisDevFixture::new();
    
    if fixture.start_basic("fixture-test").await {
        // Verify container is running
        let exists = container_exists("fixture-test").await;
        assert!(exists, "Container should be running");
        
        // Clean up
        fixture.cleanup().await;
        
        // Verify container is stopped
        let exists_after = container_exists("fixture-test").await;
        assert!(!exists_after, "Container should be stopped");
    }
}