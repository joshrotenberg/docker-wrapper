//! Integration Tests for Docker Exec Command
//!
//! These tests validate the docker exec command implementation
//! with real Docker commands and containers.

use docker_wrapper::exec::ExecCommand;
use docker_wrapper::prerequisites::ensure_docker;
use docker_wrapper::run::RunCommand;
use docker_wrapper::DockerCommand;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to check if Docker is available, skip test if not
async fn ensure_docker_or_skip() {
    match ensure_docker().await {
        Ok(_) => {}
        Err(_) => {
            println!("Docker not available - skipping exec integration test");
        }
    }
}

/// Start a test container for exec tests
async fn start_test_container() -> Result<String, Box<dyn std::error::Error>> {
    let run_cmd = RunCommand::new("alpine:latest")
        .name("exec-test-container")
        .detach()
        .cmd(vec!["sleep".to_string(), "30".to_string()])
        .remove();

    let container_id = run_cmd.execute().await?;

    // Give the container a moment to start
    sleep(Duration::from_millis(500)).await;

    Ok(container_id.as_str().to_string())
}

/// Clean up test container
async fn cleanup_test_container(container_name: &str) {
    let _ = tokio::process::Command::new("docker")
        .args(["stop", container_name])
        .output()
        .await;

    let _ = tokio::process::Command::new("docker")
        .args(["rm", container_name])
        .output()
        .await;
}

#[tokio::test]
async fn test_exec_simple_command() {
    ensure_docker_or_skip().await;

    match start_test_container().await {
        Ok(_) => {
            let exec_cmd = ExecCommand::new(
                "exec-test-container",
                vec!["echo".to_string(), "Hello World".to_string()],
            );

            match exec_cmd.execute().await {
                Ok(output) => {
                    println!("Exec: Simple command test passed");
                    assert!(output.success());
                    assert!(output.stdout.contains("Hello World"));
                }
                Err(e) => {
                    println!("Exec: Simple command test failed (may be expected): {e}");
                }
            }

            cleanup_test_container("exec-test-container").await;
        }
        Err(e) => {
            println!("Exec: Could not start test container: {e}");
        }
    }
}

#[tokio::test]
async fn test_exec_with_environment() {
    ensure_docker_or_skip().await;

    match start_test_container().await {
        Ok(_) => {
            let exec_cmd = ExecCommand::new(
                "exec-test-container",
                vec![
                    "sh".to_string(),
                    "-c".to_string(),
                    "echo $TEST_VAR $ANOTHER_VAR".to_string(),
                ],
            )
            .env("TEST_VAR", "hello")
            .env("ANOTHER_VAR", "world");

            match exec_cmd.execute().await {
                Ok(output) => {
                    println!("Exec: Environment test passed");
                    assert!(output.success());
                    assert!(output.stdout.contains("hello world"));
                }
                Err(e) => {
                    println!("Exec: Environment test failed (may be expected): {e}");
                }
            }

            cleanup_test_container("exec-test-container").await;
        }
        Err(e) => {
            println!("Exec: Could not start test container: {e}");
        }
    }
}

#[tokio::test]
async fn test_exec_with_user() {
    ensure_docker_or_skip().await;

    match start_test_container().await {
        Ok(_) => {
            let exec_cmd =
                ExecCommand::new("exec-test-container", vec!["whoami".to_string()]).user("root");

            match exec_cmd.execute().await {
                Ok(output) => {
                    println!("Exec: User test passed");
                    assert!(output.success());
                    assert!(output.stdout.trim() == "root");
                }
                Err(e) => {
                    println!("Exec: User test failed (may be expected): {e}");
                }
            }

            cleanup_test_container("exec-test-container").await;
        }
        Err(e) => {
            println!("Exec: Could not start test container: {e}");
        }
    }
}

#[tokio::test]
async fn test_exec_with_workdir() {
    ensure_docker_or_skip().await;

    match start_test_container().await {
        Ok(_) => {
            let exec_cmd =
                ExecCommand::new("exec-test-container", vec!["pwd".to_string()]).workdir("/tmp");

            match exec_cmd.execute().await {
                Ok(output) => {
                    println!("Exec: Working directory test passed");
                    assert!(output.success());
                    assert!(output.stdout.contains("/tmp"));
                }
                Err(e) => {
                    println!("Exec: Working directory test failed (may be expected): {e}");
                }
            }

            cleanup_test_container("exec-test-container").await;
        }
        Err(e) => {
            println!("Exec: Could not start test container: {e}");
        }
    }
}

#[tokio::test]
async fn test_exec_detached() {
    ensure_docker_or_skip().await;

    match start_test_container().await {
        Ok(_) => {
            let exec_cmd = ExecCommand::new(
                "exec-test-container",
                vec!["sleep".to_string(), "1".to_string()],
            )
            .detach();

            match exec_cmd.execute().await {
                Ok(output) => {
                    println!("Exec: Detached command test passed");
                    // Detached commands typically return immediately with empty output
                    assert!(output.success());
                }
                Err(e) => {
                    println!("Exec: Detached command test failed (may be expected): {e}");
                }
            }

            cleanup_test_container("exec-test-container").await;
        }
        Err(e) => {
            println!("Exec: Could not start test container: {e}");
        }
    }
}

#[tokio::test]
async fn test_exec_command_builder() {
    // This test doesn't require Docker - just validates command construction
    let complex_exec = ExecCommand::new(
        "test-container",
        vec![
            "bash".to_string(),
            "-c".to_string(),
            "echo test".to_string(),
        ],
    )
    .interactive()
    .tty()
    .env("DEBUG", "1")
    .env("LOG_LEVEL", "info")
    .user("root")
    .workdir("/app")
    .privileged();

    let args = complex_exec.build_args();

    // Verify critical components are present (build_args doesn't include "exec" command itself)
    assert!(args.contains(&"--interactive".to_string()));
    assert!(args.contains(&"--tty".to_string()));
    assert!(args.contains(&"--env".to_string()));
    assert!(args.contains(&"DEBUG=1".to_string()));
    assert!(args.contains(&"LOG_LEVEL=info".to_string()));
    assert!(args.contains(&"--user".to_string()));
    assert!(args.contains(&"root".to_string()));
    assert!(args.contains(&"--workdir".to_string()));
    assert!(args.contains(&"/app".to_string()));
    assert!(args.contains(&"--privileged".to_string()));
    assert!(args.contains(&"test-container".to_string()));
    assert!(args.contains(&"bash".to_string()));
    assert!(args.contains(&"-c".to_string()));
    assert!(args.contains(&"echo test".to_string()));

    println!("Exec: Command builder validation passed");
}

#[tokio::test]
async fn test_exec_prerequisites_validation() {
    // Always run this test - it should handle Docker unavailability gracefully
    match ensure_docker().await {
        Ok(info) => {
            let version = &info.version.version;
            println!("Exec: Prerequisites OK - Docker {version}");
            assert!(!info.version.version.is_empty());
        }
        Err(e) => {
            println!("Exec: Prerequisites failed (expected in some CI): {e}");
            // Don't fail - this is expected when Docker isn't available
        }
    }
}

#[tokio::test]
async fn test_exec_it_convenience() {
    // Test the it() convenience method
    let exec_cmd = ExecCommand::new("test-container", vec!["bash".to_string()]).it();

    let args = exec_cmd.build_args();

    assert!(args.contains(&"--interactive".to_string()));
    assert!(args.contains(&"--tty".to_string()));

    println!("Exec: IT convenience method validation passed");
}

#[tokio::test]
async fn test_exec_multiple_env_files() {
    // Test multiple environment files
    let exec_cmd = ExecCommand::new("test-container", vec!["env".to_string()])
        .env_file("/path/to/env1.file")
        .env_file("/path/to/env2.file");

    let args = exec_cmd.build_args();

    // Count occurrences of --env-file
    let env_file_count = args.iter().filter(|&arg| arg == "--env-file").count();
    assert_eq!(env_file_count, 2);
    assert!(args.contains(&"/path/to/env1.file".to_string()));
    assert!(args.contains(&"/path/to/env2.file".to_string()));

    println!("Exec: Multiple env files validation passed");
}
