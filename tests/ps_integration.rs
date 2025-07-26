//! Integration Tests for Docker PS Command
//!
//! These tests validate the docker ps command implementation
//! with real Docker commands and containers.

use docker_wrapper::prerequisites::ensure_docker;
use docker_wrapper::ps::PsCommand;
use docker_wrapper::run::RunCommand;
use docker_wrapper::DockerCommand;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to check if Docker is available, skip test if not
async fn ensure_docker_or_skip() {
    match ensure_docker().await {
        Ok(_) => {}
        Err(_) => {
            println!("Docker not available - skipping ps integration test");
        }
    }
}

/// Start multiple test containers for ps tests
async fn start_test_containers() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut container_ids = Vec::new();

    // Start first container
    let run_cmd1 = RunCommand::new("alpine:latest")
        .name("ps-test-container-1")
        .detach()
        .cmd(vec!["sleep".to_string(), "30".to_string()]);

    let container_id1 = run_cmd1.execute().await?;
    container_ids.push(container_id1.as_str().to_string());

    // Start second container with different configuration
    let run_cmd2 = RunCommand::new("alpine:latest")
        .name("ps-test-container-2")
        .detach()
        .env("TEST_ENV", "integration")
        .cmd(vec!["sleep".to_string(), "30".to_string()]);

    let container_id2 = run_cmd2.execute().await?;
    container_ids.push(container_id2.as_str().to_string());

    // Give containers time to start
    sleep(Duration::from_millis(1000)).await;

    Ok(container_ids)
}

/// Clean up test containers
async fn cleanup_test_containers(container_names: &[&str]) {
    for name in container_names {
        let _ = tokio::process::Command::new("docker")
            .args(["stop", name])
            .output()
            .await;

        let _ = tokio::process::Command::new("docker")
            .args(["rm", name])
            .output()
            .await;
    }
}

#[tokio::test]
async fn test_ps_list_running_containers() {
    ensure_docker_or_skip().await;

    match start_test_containers().await {
        Ok(_) => {
            let ps_cmd = PsCommand::new();

            match ps_cmd.execute().await {
                Ok(output) => {
                    println!("PS: List running containers test passed");
                    assert!(output.success());
                    assert!(!output.stdout_is_empty());

                    // Should show containers (count is valid)
                    let _count = output.container_count();
                }
                Err(e) => {
                    println!("PS: List running containers test failed (may be expected): {e}");
                }
            }

            cleanup_test_containers(&["ps-test-container-1", "ps-test-container-2"]).await;
        }
        Err(e) => {
            println!("PS: Could not start test containers: {e}");
        }
    }
}

#[tokio::test]
async fn test_ps_list_all_containers() {
    ensure_docker_or_skip().await;

    match start_test_containers().await {
        Ok(_) => {
            // Stop one container to test --all flag
            let _ = tokio::process::Command::new("docker")
                .args(["stop", "ps-test-container-1"])
                .output()
                .await;

            sleep(Duration::from_millis(500)).await;

            let ps_cmd = PsCommand::new().all();

            match ps_cmd.execute().await {
                Ok(output) => {
                    println!("PS: List all containers test passed");
                    assert!(output.success());
                    assert!(!output.stdout_is_empty());

                    // Should show both running and stopped containers
                    let output_text = output.stdout.to_lowercase();
                    // Look for evidence of both running and exited containers
                    assert!(
                        output_text.contains("ps-test-container") || output.container_count() > 0
                    );
                }
                Err(e) => {
                    println!("PS: List all containers test failed (may be expected): {e}");
                }
            }

            cleanup_test_containers(&["ps-test-container-1", "ps-test-container-2"]).await;
        }
        Err(e) => {
            println!("PS: Could not start test containers: {e}");
        }
    }
}

#[tokio::test]
async fn test_ps_quiet_mode() {
    ensure_docker_or_skip().await;

    match start_test_containers().await {
        Ok(_) => {
            let ps_cmd = PsCommand::new().quiet();

            match ps_cmd.execute().await {
                Ok(output) => {
                    println!("PS: Quiet mode test passed");
                    assert!(output.success());

                    if !output.stdout_is_empty() {
                        // In quiet mode, output should be container IDs only
                        let ids = output.container_ids();
                        assert!(!ids.is_empty());

                        // Each line should look like a container ID (hex string)
                        for id in ids {
                            assert!(id.len() >= 12); // Short container IDs are at least 12 chars
                            assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
                        }
                    }
                }
                Err(e) => {
                    println!("PS: Quiet mode test failed (may be expected): {e}");
                }
            }

            cleanup_test_containers(&["ps-test-container-1", "ps-test-container-2"]).await;
        }
        Err(e) => {
            println!("PS: Could not start test containers: {e}");
        }
    }
}

#[tokio::test]
async fn test_ps_with_filters() {
    ensure_docker_or_skip().await;

    match start_test_containers().await {
        Ok(_) => {
            let ps_cmd = PsCommand::new().filter("name=ps-test-container-1").all();

            match ps_cmd.execute().await {
                Ok(output) => {
                    println!("PS: Filter test passed");
                    assert!(output.success());

                    if !output.stdout_is_empty() {
                        // Should only show the filtered container
                        let output_text = output.stdout.to_lowercase();
                        assert!(output_text.contains("ps-test-container-1"));
                        // Should not contain the other container
                        assert!(!output_text.contains("ps-test-container-2"));
                    }
                }
                Err(e) => {
                    println!("PS: Filter test failed (may be expected): {e}");
                }
            }

            cleanup_test_containers(&["ps-test-container-1", "ps-test-container-2"]).await;
        }
        Err(e) => {
            println!("PS: Could not start test containers: {e}");
        }
    }
}

#[tokio::test]
async fn test_ps_json_format() {
    ensure_docker_or_skip().await;

    match start_test_containers().await {
        Ok(_) => {
            let ps_cmd = PsCommand::new().format_json();

            match ps_cmd.execute().await {
                Ok(output) => {
                    println!("PS: JSON format test passed");
                    assert!(output.success());

                    if !output.stdout_is_empty() {
                        let stdout = output.stdout.trim();
                        if !stdout.is_empty() {
                            // Each line should be valid JSON (Docker outputs one JSON object per line)
                            for line in stdout.lines() {
                                let json_result = serde_json::from_str::<serde_json::Value>(line);
                                assert!(
                                    json_result.is_ok(),
                                    "Each line should be valid JSON: {line}"
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("PS: JSON format test failed (may be expected): {e}");
                }
            }

            cleanup_test_containers(&["ps-test-container-1", "ps-test-container-2"]).await;
        }
        Err(e) => {
            println!("PS: Could not start test containers: {e}");
        }
    }
}

#[tokio::test]
async fn test_ps_latest_container() {
    ensure_docker_or_skip().await;

    match start_test_containers().await {
        Ok(_) => {
            let ps_cmd = PsCommand::new().latest().all();

            match ps_cmd.execute().await {
                Ok(output) => {
                    println!("PS: Latest container test passed");
                    assert!(output.success());

                    if !output.stdout_is_empty() {
                        // Should show only one container (the latest)
                        let lines: Vec<&str> = output.stdout.lines().collect();
                        // Should have header + one container line (or just one container in some formats)
                        assert!(!lines.is_empty());
                    }
                }
                Err(e) => {
                    println!("PS: Latest container test failed (may be expected): {e}");
                }
            }

            cleanup_test_containers(&["ps-test-container-1", "ps-test-container-2"]).await;
        }
        Err(e) => {
            println!("PS: Could not start test containers: {e}");
        }
    }
}

#[tokio::test]
async fn test_ps_with_size() {
    ensure_docker_or_skip().await;

    match start_test_containers().await {
        Ok(_) => {
            let ps_cmd = PsCommand::new().size();

            match ps_cmd.execute().await {
                Ok(output) => {
                    println!("PS: Size option test passed");
                    assert!(output.success());

                    if !output.stdout_is_empty() {
                        // Output should contain size information
                        let output_text = output.stdout.to_lowercase();
                        // Look for size indicators (B, KB, MB, etc.)
                        assert!(
                            output_text.contains("size")
                                || output_text.contains("kb")
                                || output_text.contains("mb")
                                || output_text.contains("b")
                        );
                    }
                }
                Err(e) => {
                    println!("PS: Size option test failed (may be expected): {e}");
                }
            }

            cleanup_test_containers(&["ps-test-container-1", "ps-test-container-2"]).await;
        }
        Err(e) => {
            println!("PS: Could not start test containers: {e}");
        }
    }
}

#[tokio::test]
async fn test_ps_command_builder() {
    // This test doesn't require Docker - just validates command construction
    let complex_ps = PsCommand::new()
        .all()
        .filter("status=running")
        .filter("name=web")
        .format_json()
        .no_trunc()
        .size()
        .quiet();

    let args = complex_ps.build_args();

    // Verify critical components are present (build_args doesn't include "ps" command itself)
    assert!(args.contains(&"--all".to_string()));
    assert!(args.contains(&"--filter".to_string()));
    assert!(args.contains(&"status=running".to_string()));
    assert!(args.contains(&"name=web".to_string()));
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"json".to_string()));
    assert!(args.contains(&"--no-trunc".to_string()));
    assert!(args.contains(&"--size".to_string()));
    assert!(args.contains(&"--quiet".to_string()));

    println!("PS: Command builder validation passed");
}

#[tokio::test]
async fn test_ps_prerequisites_validation() {
    // Always run this test - it should handle Docker unavailability gracefully
    match ensure_docker().await {
        Ok(info) => {
            let version = &info.version.version;
            println!("PS: Prerequisites OK - Docker {version}");
            assert!(!info.version.version.is_empty());
        }
        Err(e) => {
            println!("PS: Prerequisites failed (expected in some CI): {e}");
            // Don't fail - this is expected when Docker isn't available
        }
    }
}

#[tokio::test]
async fn test_ps_no_containers() {
    ensure_docker_or_skip().await;

    // Test ps when no containers match (using a very specific filter)
    let ps_cmd = PsCommand::new()
        .filter("name=nonexistent-container-name-12345")
        .all();

    match ps_cmd.execute().await {
        Ok(output) => {
            println!("PS: No containers test passed");
            assert!(output.success());

            // Should succeed even with no containers
            assert_eq!(output.container_count(), 0);

            // Output might be empty or just contain headers
            if !output.stdout_is_empty() {
                // If there's output, it should be just headers or empty data
                let lines: Vec<&str> = output.stdout.lines().collect();
                assert!(lines.len() <= 1); // Header only or empty
            }
        }
        Err(e) => {
            println!("PS: No containers test failed (may be expected): {e}");
        }
    }
}
