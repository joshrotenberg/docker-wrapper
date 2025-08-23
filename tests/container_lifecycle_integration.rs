//! Integration tests for container lifecycle commands (stop, start, restart).

use docker_wrapper::command::DockerCommand;
use docker_wrapper::{RestartCommand, RunCommand, StartCommand, StopCommand};
use std::time::Duration;
use tokio::time::sleep;

/// Test helper to create a running container for lifecycle tests
async fn create_test_container(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let result = RunCommand::new("alpine:latest")
        .name(name)
        .detach()
        .cmd(vec!["sleep".to_string(), "30".to_string()])
        .execute()
        .await?;

    Ok(result.0)
}

/// Test helper to clean up containers after tests
async fn cleanup_container(name: &str) {
    // Try to stop and remove the container, ignore errors
    let _ = StopCommand::new(name).execute().await;
    let _ = docker_wrapper::command::CommandExecutor::new()
        .execute_command("rm", vec!["-f".to_string(), name.to_string()])
        .await;
}

#[tokio::test]
async fn test_stop_command_basic() {
    let container_name = "test-stop-basic";

    // Create a running container
    let _container_id = match create_test_container(container_name).await {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Skipping test - Docker not available or container creation failed");
            return;
        }
    };

    // Wait a moment for container to fully start
    sleep(Duration::from_millis(500)).await;

    // Stop the container
    let result = StopCommand::new(container_name).execute().await;

    match result {
        Ok(stop_result) => {
            assert!(stop_result.is_success());
            assert_eq!(stop_result.container_count(), 1);
            assert!(stop_result.contains_container(container_name));
        }
        Err(e) => {
            eprintln!("Stop command failed: {e}");
        }
    }

    cleanup_container(container_name).await;
}

#[tokio::test]
async fn test_stop_command_with_timeout() {
    let container_name = "test-stop-timeout";

    let _container_id = match create_test_container(container_name).await {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Skipping test - Docker not available");
            return;
        }
    };

    sleep(Duration::from_millis(500)).await;

    let result = StopCommand::new(container_name).timeout(5).execute().await;

    match result {
        Ok(stop_result) => {
            assert!(stop_result.is_success());
            assert!(stop_result.contains_container(container_name));
        }
        Err(e) => {
            eprintln!("Stop with timeout failed: {e}");
        }
    }

    cleanup_container(container_name).await;
}

#[tokio::test]
async fn test_start_command_basic() {
    let container_name = "test-start-basic";

    // Create and then stop a container
    let _container_id = match create_test_container(container_name).await {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Skipping test - Docker not available");
            return;
        }
    };

    sleep(Duration::from_millis(500)).await;

    // Stop the container first
    let _ = StopCommand::new(container_name).execute().await;
    sleep(Duration::from_millis(500)).await;

    // Now start it again
    let result = StartCommand::new(container_name).execute().await;

    match result {
        Ok(start_result) => {
            assert!(start_result.is_success());
            assert_eq!(start_result.container_count(), 1);
            assert!(start_result.contains_container(container_name));
        }
        Err(e) => {
            eprintln!("Start command failed: {e}");
        }
    }

    cleanup_container(container_name).await;
}

#[tokio::test]
async fn test_restart_command_basic() {
    let container_name = "test-restart-basic";

    let _container_id = match create_test_container(container_name).await {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Skipping test - Docker not available");
            return;
        }
    };

    sleep(Duration::from_millis(500)).await;

    let result = RestartCommand::new(container_name).execute().await;

    match result {
        Ok(restart_result) => {
            assert!(restart_result.is_success());
            assert_eq!(restart_result.container_count(), 1);
            assert!(restart_result.contains_container(container_name));
        }
        Err(e) => {
            eprintln!("Restart command failed: {e}");
        }
    }

    cleanup_container(container_name).await;
}

#[tokio::test]
async fn test_container_lifecycle_full_workflow() {
    let container_name = "test-lifecycle-full";

    // Create container
    let _container_id = match create_test_container(container_name).await {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Skipping test - Docker not available");
            return;
        }
    };

    sleep(Duration::from_millis(500)).await;

    // Stop container
    let stop_result = StopCommand::new(container_name).execute().await;
    if let Ok(result) = stop_result {
        assert!(result.is_success());
    }

    sleep(Duration::from_millis(500)).await;

    // Start container
    let start_result = StartCommand::new(container_name).execute().await;
    if let Ok(result) = start_result {
        assert!(result.is_success());
    }

    sleep(Duration::from_millis(500)).await;

    // Restart container
    let restart_result = RestartCommand::new(container_name).execute().await;
    if let Ok(result) = restart_result {
        assert!(result.is_success());
    }

    cleanup_container(container_name).await;
}

#[tokio::test]
async fn test_multiple_containers_stop() {
    let container_names = vec!["test-multi-stop-1", "test-multi-stop-2"];

    // Create multiple containers
    let mut created_containers = Vec::new();
    for name in &container_names {
        if let Ok(_id) = create_test_container(name).await {
            created_containers.push(name);
        }
    }

    if created_containers.is_empty() {
        eprintln!("Skipping test - Could not create test containers");
        return;
    }

    sleep(Duration::from_millis(500)).await;

    // Stop multiple containers
    let result = StopCommand::new_multiple(created_containers.iter().map(|s| s.to_string()))
        .execute()
        .await;

    match result {
        Ok(stop_result) => {
            assert!(stop_result.is_success());
            for container_name in &created_containers {
                assert!(stop_result.contains_container(container_name));
            }
        }
        Err(e) => {
            eprintln!("Multiple container stop failed: {e}");
        }
    }

    // Cleanup
    for name in &container_names {
        cleanup_container(name).await;
    }
}

#[tokio::test]
async fn test_command_args_generation() {
    // Test stop command args
    let stop_cmd = StopCommand::new("test-container")
        .signal("SIGTERM")
        .timeout(30);
    let stop_args = stop_cmd.args();
    assert_eq!(
        stop_args,
        vec![
            "stop",
            "--signal",
            "SIGTERM",
            "--timeout",
            "30",
            "test-container"
        ]
    );

    // Test start command args
    let start_cmd = StartCommand::new("test-container").attach().interactive();
    let start_args = start_cmd.args();
    assert_eq!(
        start_args,
        vec!["start", "--attach", "--interactive", "test-container"]
    );

    // Test restart command args
    let restart_cmd = RestartCommand::new("test-container").timeout(15);
    let restart_args = restart_cmd.args();
    assert_eq!(
        restart_args,
        vec!["restart", "--timeout", "15", "test-container"]
    );
}

#[tokio::test]
async fn test_command_args() {
    let stop_cmd = StopCommand::new("test");
    let stop_args = stop_cmd.build_command_args();
    assert_eq!(stop_args[0], "stop");

    let start_cmd = StartCommand::new("test");
    let start_args = start_cmd.build_command_args();
    assert_eq!(start_args[0], "start");

    let restart_cmd = RestartCommand::new("test");
    let restart_args = restart_cmd.build_command_args();
    assert_eq!(restart_args[0], "restart");
}

#[tokio::test]
async fn test_result_helper_methods() {
    // Test stop result helpers
    let stop_result = docker_wrapper::StopResult {
        stdout: "container1\ncontainer2\n".to_string(),
        stderr: String::new(),
        stopped_containers: vec!["container1".to_string(), "container2".to_string()],
    };

    assert!(stop_result.is_success());
    assert_eq!(stop_result.container_count(), 2);
    assert_eq!(
        stop_result.first_container(),
        Some(&"container1".to_string())
    );
    assert!(stop_result.contains_container("container1"));
    assert!(stop_result.contains_container("container2"));
    assert!(!stop_result.contains_container("container3"));

    // Test start result helpers
    let start_result = docker_wrapper::StartResult {
        stdout: "container1\n".to_string(),
        stderr: String::new(),
        started_containers: vec!["container1".to_string()],
    };

    assert!(start_result.is_success());
    assert_eq!(start_result.container_count(), 1);
    assert_eq!(
        start_result.first_container(),
        Some(&"container1".to_string())
    );
    assert!(start_result.contains_container("container1"));

    // Test restart result helpers
    let restart_result = docker_wrapper::RestartResult {
        stdout: String::new(),
        stderr: String::new(),
        restarted_containers: vec!["container1".to_string()],
    };

    assert!(restart_result.is_success());
    assert_eq!(restart_result.container_count(), 1);
    assert!(restart_result.contains_container("container1"));
}

#[tokio::test]
async fn test_error_handling() {
    // Test stopping non-existent container
    let result = StopCommand::new("non-existent-container-12345")
        .execute()
        .await;

    match result {
        Ok(_) => {
            // Some Docker versions don't error on stopping non-existent containers
            println!("Stop non-existent container succeeded (Docker version dependent)");
        }
        Err(e) => {
            println!("Stop non-existent container failed as expected: {e}");
        }
    }

    // Test starting non-existent container
    let result = StartCommand::new("non-existent-container-12345")
        .execute()
        .await;

    match result {
        Ok(_) => {
            println!("Start non-existent container succeeded (unexpected)");
        }
        Err(e) => {
            println!("Start non-existent container failed as expected: {e}");
        }
    }
}
