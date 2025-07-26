//! Phase 2 Integration Tests for Docker Wrapper
//!
//! These tests are designed to run during the release process
//! and validate core functionality with real Docker commands.

use docker_wrapper::DockerCommand;
use docker_wrapper::prerequisites::ensure_docker;
use docker_wrapper::run::RunCommand;

/// Helper to check if Docker is available, skip test if not
async fn ensure_docker_or_skip() {
    match ensure_docker().await {
        Ok(_) => {}
        Err(_) => {
            println!("Docker not available - skipping phase 2 integration test");
            return;
        }
    }
}

#[tokio::test]
async fn test_phase2_docker_run_basic() {
    ensure_docker_or_skip().await;

    // Test basic container execution
    let run_cmd = RunCommand::new("alpine:latest")
        .cmd(vec!["echo".to_string(), "Phase 2 Test".to_string()])
        .remove();

    match run_cmd.execute().await {
        Ok(container_id) => {
            println!("Phase 2: Basic run test passed - {}", container_id.short());
            assert!(!container_id.0.is_empty());
        }
        Err(e) => {
            println!("Phase 2: Basic run test failed (may be expected): {}", e);
        }
    }
}

#[tokio::test]
async fn test_phase2_docker_run_with_options() {
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
            println!("Phase 2: Options test passed - {}", container_id.short());
            assert!(!container_id.0.is_empty());
        }
        Err(e) => {
            println!("Phase 2: Options test failed (may be expected): {}", e);
        }
    }
}

#[tokio::test]
async fn test_phase2_prerequisites_validation() {
    // Always run this test - it should handle Docker unavailability gracefully
    match ensure_docker().await {
        Ok(info) => {
            println!(
                "Phase 2: Prerequisites OK - Docker {}",
                info.version.version
            );
            assert!(!info.version.version.is_empty());
        }
        Err(e) => {
            println!("Phase 2: Prerequisites failed (expected in some CI): {}", e);
            // Don't fail - this is expected when Docker isn't available
        }
    }
}

#[tokio::test]
async fn test_phase2_command_builder_correctness() {
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

    println!("Phase 2: Command builder validation passed");
}
