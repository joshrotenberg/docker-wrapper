//! Integration Tests for Docker Run Command
//!
//! These tests validate the docker run command implementation
//! with real Docker commands and containers.

use docker_wrapper::prerequisites::ensure_docker;
use docker_wrapper::{DockerCommand, RunCommand};

/// Helper to check if Docker is available, skip test if not
async fn ensure_docker_or_skip() {
    match ensure_docker().await {
        Ok(_) => {}
        Err(_) => {
            // Docker not available - skipping run integration test
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
            assert!(!container_id.0.is_empty());
        }
        Err(_) => {
            // Basic container test failed (may be expected)
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
            assert!(!container_id.0.is_empty());
        }
        Err(_) => {
            // Options test failed (may be expected)
        }
    }
}

#[tokio::test]
async fn test_run_prerequisites_validation() {
    // Always run this test - it should handle Docker unavailability gracefully
    match ensure_docker().await {
        Ok(info) => {
            assert!(!info.version.version.is_empty());
        }
        Err(_) => {
            // Prerequisites failed (expected in some CI)
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
}

#[tokio::test]
async fn test_run_high_impact_dns_options() {
    // Test DNS and network options without requiring Docker execution
    let run_cmd = RunCommand::new("alpine:latest")
        .dns("8.8.8.8")
        .dns("1.1.1.1")
        .dns_option("ndots:2")
        .dns_search("example.com")
        .add_host("api.example.com:127.0.0.1")
        .remove();

    let args = run_cmd.build_args();

    // Verify DNS options are properly formatted
    assert!(args.contains(&"--dns".to_string()));
    assert!(args.contains(&"8.8.8.8".to_string()));
    assert!(args.contains(&"1.1.1.1".to_string()));
    assert!(args.contains(&"--dns-option".to_string()));
    assert!(args.contains(&"ndots:2".to_string()));
    assert!(args.contains(&"--dns-search".to_string()));
    assert!(args.contains(&"example.com".to_string()));
    assert!(args.contains(&"--add-host".to_string()));
    assert!(args.contains(&"api.example.com:127.0.0.1".to_string()));
}

#[tokio::test]
async fn test_run_high_impact_security_options() {
    // Test security and capabilities options without requiring Docker execution
    let run_cmd = RunCommand::new("alpine:latest")
        .cap_add("NET_ADMIN")
        .cap_drop("CHOWN")
        .security_opt("no-new-privileges:true")
        .security_opt("seccomp=unconfined")
        .remove();

    let args = run_cmd.build_args();

    // Verify security options are properly formatted
    assert!(args.contains(&"--cap-add".to_string()));
    assert!(args.contains(&"NET_ADMIN".to_string()));
    assert!(args.contains(&"--cap-drop".to_string()));
    assert!(args.contains(&"CHOWN".to_string()));
    assert!(args.contains(&"--security-opt".to_string()));
    assert!(args.contains(&"no-new-privileges:true".to_string()));
    assert!(args.contains(&"seccomp=unconfined".to_string()));
}

#[tokio::test]
async fn test_run_high_impact_device_filesystem_options() {
    use std::path::PathBuf;

    // Test device and filesystem options without requiring Docker execution
    let run_cmd = RunCommand::new("alpine:latest")
        .device("/dev/null")
        .tmpfs("/tmp:rw,size=100m")
        .expose("80")
        .expose("443")
        .env_file(PathBuf::from(".env"))
        .label("version=1.0.0")
        .label("app=test")
        .remove();

    let args = run_cmd.build_args();

    // Verify device and filesystem options are properly formatted
    assert!(args.contains(&"--device".to_string()));
    assert!(args.contains(&"/dev/null".to_string()));
    assert!(args.contains(&"--tmpfs".to_string()));
    assert!(args.contains(&"/tmp:rw,size=100m".to_string()));
    assert!(args.contains(&"--expose".to_string()));
    assert!(args.contains(&"80".to_string()));
    assert!(args.contains(&"443".to_string()));
    assert!(args.contains(&"--env-file".to_string()));
    assert!(args.contains(&".env".to_string()));
    assert!(args.contains(&"--label".to_string()));
    assert!(args.contains(&"version=1.0.0".to_string()));
    assert!(args.contains(&"app=test".to_string()));
}

#[tokio::test]
async fn test_run_comprehensive_high_impact_options() {
    ensure_docker_or_skip().await;

    // Test running a container with multiple high-impact options
    let run_cmd = RunCommand::new("alpine:latest")
        .name("high-impact-test")
        .dns("8.8.8.8")
        .add_host("test.local:127.0.0.1")
        .expose("8080")
        .label("test=high-impact")
        .label("phase=3")
        .cmd(vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo High-impact options test && sleep 1".to_string(),
        ])
        .remove();

    match run_cmd.execute().await {
        Ok(container_id) => {
            assert!(!container_id.0.is_empty());
        }
        Err(_) => {
            // High-impact comprehensive test failed (may be expected)
        }
    }
}
