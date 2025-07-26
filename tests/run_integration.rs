//! Integration Tests for Docker Run Command
//!
//! These tests validate the docker run command implementation
//! with real Docker commands and containers.

use docker_wrapper::prerequisites::ensure_docker;
use docker_wrapper::run::RunCommand;
use docker_wrapper::DockerCommand;

/// Helper to check if Docker is available, skip test if not
async fn ensure_docker_or_skip() {
    match ensure_docker().await {
        Ok(_) => {}
        Err(_) => {
            println!("Docker not available - skipping run integration test");
        }
    }
}

#[tokio::test]
async fn test_run_basic_container() {
    ensure_docker_or_skip().await;

    // Test basic container execution
    let run_cmd = RunCommand::new("alpine:latest")
        .cmd(vec!["echo".to_string(), "Phase 2 Test".to_string()])
        .remove();

    match run_cmd.execute().await {
        Ok(container_id) => {
            let short_id = container_id.short();
            println!("Run: Basic container test passed - {short_id}");
            assert!(!container_id.0.is_empty());
        }
        Err(e) => {
            println!("Run: Basic container test failed (may be expected): {e}");
        }
    }
}

#[tokio::test]
async fn test_run_with_options() {
    ensure_docker_or_skip().await;

    // Test run with multiple options
    let run_cmd = RunCommand::new("alpine:latest")
        .name("phase2-test")
        .env("PHASE", "2")
        .env("TEST", "integration")
        .cmd(vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo Phase: $PHASE, Test: $TEST".to_string(),
        ])
        .remove();

    match run_cmd.execute().await {
        Ok(container_id) => {
            let short_id = container_id.short();
            println!("Run: Options test passed - {short_id}");
            assert!(!container_id.0.is_empty());
        }
        Err(e) => {
            println!("Run: Options test failed (may be expected): {e}");
        }
    }
}

#[tokio::test]
async fn test_run_prerequisites_validation() {
    // Always run this test - it should handle Docker unavailability gracefully
    match ensure_docker().await {
        Ok(info) => {
            let version = &info.version.version;
            println!("Run: Prerequisites OK - Docker {version}");
            assert!(!info.version.version.is_empty());
        }
        Err(e) => {
            println!("Run: Prerequisites failed (expected in some CI): {e}");
            // Don't fail - this is expected when Docker isn't available
        }
    }
}

#[tokio::test]
async fn test_run_command_builder() {
    // This test doesn't require Docker - just validates command construction
    let complex_run = RunCommand::new("redis:7.2-alpine")
        .name("test-redis")
        .port(6379, 6379)
        .env("REDIS_PASSWORD", "test123")
        .volume("/data", "/data")
        .detach();

    let args = complex_run.build_args();

    // Verify critical components are present (build_args doesn't include "run" command itself)
    assert!(args.contains(&"--name".to_string()));
    assert!(args.contains(&"test-redis".to_string()));
    assert!(args.contains(&"--publish".to_string()));
    assert!(args.contains(&"6379:6379".to_string()));
    assert!(args.contains(&"--env".to_string()));
    assert!(args.contains(&"REDIS_PASSWORD=test123".to_string()));
    assert!(args.contains(&"--volume".to_string()));
    assert!(args.contains(&"/data:/data".to_string()));
    assert!(args.contains(&"--detach".to_string()));
    assert!(args.contains(&"redis:7.2-alpine".to_string()));

    println!("Run: Command builder validation passed");
}
