//! Basic testing patterns with docker-wrapper
//!
//! This example demonstrates fundamental patterns for using docker-wrapper
//! in test scenarios, including container lifecycle management and cleanup.

use docker_wrapper::{DockerCommand, RunCommand};
#[allow(unused_imports)]
use docker_wrapper::{ExecCommand, LogsCommand, RmCommand, StopCommand};
#[allow(unused_imports)]
use std::time::Duration;

/// Generate a unique container name for parallel test execution
#[allow(dead_code)]
fn unique_container_name(prefix: &str) -> String {
    format!("{}-{}", prefix, uuid::Uuid::new_v4())
}

/// Basic test with automatic cleanup
#[tokio::test]
async fn test_with_auto_cleanup() {
    let container_name = unique_container_name("test-redis");

    // Start container with --rm flag for automatic cleanup
    let output = RunCommand::new("redis:7-alpine")
        .name(&container_name)
        .detach()
        .remove() // Automatically remove when stopped
        .execute()
        .await
        .expect("Failed to start Redis");

    println!("Started container: {}", output.0);

    // Wait for container to be ready
    tokio::time::sleep(Duration::from_secs(1)).await;

        // Execute test commands
        let result = ExecCommand::new(
            &container_name,
            vec!["redis-cli".to_string(), "PING".to_string()],
        )
        .execute()
        .await
        .expect("Failed to execute command");

        assert_eq!(result.stdout.trim(), "PONG");

    // Stop container (will be automatically removed due to --rm flag)
    StopCommand::new(&container_name)
        .execute()
        .await
        .expect("Failed to stop container");
}

/// Test with manual cleanup
#[tokio::test]
async fn test_with_manual_cleanup() {
    let container_name = unique_container_name("test-nginx");

    // Start container without auto-remove
    let output = RunCommand::new("nginx:alpine")
        .name(&container_name)
        .port(8080, 80)
        .detach()
        .execute()
        .await
        .expect("Failed to start Nginx");

    println!("Started container: {}", output.0);

    // Your test logic here
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Manual cleanup
    StopCommand::new(&container_name)
        .execute()
        .await
        .expect("Failed to stop container");

    RmCommand::new(&container_name)
        .force()
        .execute()
        .await
        .expect("Failed to remove container");
}

/// Test with error handling and debugging
#[tokio::test]
async fn test_with_error_handling() {
    let container_name = unique_container_name("test-postgres");

    // Attempt to start container
    let result = RunCommand::new("postgres:15")
        .name(&container_name)
        .env("POSTGRES_PASSWORD", "test")
        .detach()
        .remove()
        .execute()
        .await;

    match result {
        Ok(output) => {
            println!("Container started successfully: {}", output.0);

            // Wait for PostgreSQL to initialize
            tokio::time::sleep(Duration::from_secs(3)).await;

            // Test logic here

            // Stop container
            let _ = StopCommand::new(&container_name).execute().await;
        }
        Err(e) => {
            eprintln!("Failed to start container: {}", e);

            // Try to get logs for debugging
                if let Ok(logs) = LogsCommand::new(&container_name).tail("50").execute().await {
                    eprintln!("Container logs:\n{}", logs.stdout);
            }

            panic!("Test failed due to container startup error");
        }
    }
}

/// Test with parallel execution safety
#[tokio::test]
async fn test_parallel_safe_containers() {
    // Example showing parallel container testing
    // Uncomment and add 'futures' crate to use this pattern

    // // Each test gets its own uniquely named container
    // let futures = vec![
    //     start_test_container("test1"),
    //     start_test_container("test2"),
    //     start_test_container("test3"),
    // ];
    //
    // // Run all containers in parallel
    // let results = futures::future::join_all(futures).await;
    //
    // // Verify all started successfully
    // for (i, result) in results.iter().enumerate() {
    //     assert!(result.is_ok(), "Container {} failed to start", i + 1);
    // }
    //
    // // Clean up all containers
    // for result in results {
    //     if let Ok(name) = result {
    //         let _ = StopCommand::new(&name).execute().await;
    //     }
    // }

    // For now, just run them sequentially
    let result1 = start_test_container("test1").await;
    let result2 = start_test_container("test2").await;
    let result3 = start_test_container("test3").await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());

    // Cleanup
    if let Ok(name) = result1 {
        let _ = StopCommand::new(&name).execute().await;
    }
    if let Ok(name) = result2 {
        let _ = StopCommand::new(&name).execute().await;
    }
    if let Ok(name) = result3 {
        let _ = StopCommand::new(&name).execute().await;
    }
}

#[allow(dead_code)]
async fn start_test_container(test_id: &str) -> Result<String, docker_wrapper::Error> {
    let container_name = unique_container_name(&format!("parallel-{}", test_id));

    RunCommand::new("alpine")
        .name(&container_name)
        .cmd(vec!["sleep".to_string(), "10".to_string()])
        .detach()
        .remove()
        .execute()
        .await?;

    Ok(container_name)
}

/// Test with container logs for debugging
#[tokio::test]
async fn test_with_log_inspection() {
    let container_name = unique_container_name("test-app");

    // Start a container that produces logs
    RunCommand::new("alpine")
        .name(&container_name)
        .cmd(vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo 'Starting app...'; sleep 1; echo 'App ready!'".to_string(),
        ])
        .detach()
        .execute()
        .await
        .expect("Failed to start container");

    // Wait for container to produce logs
    tokio::time::sleep(Duration::from_secs(2)).await;

        // Fetch and inspect logs
        let logs = LogsCommand::new(&container_name)
            .execute()
            .await
            .expect("Failed to fetch logs");

        println!("Container logs:\n{}", logs.stdout);

        assert!(logs.stdout.contains("Starting app..."));
        assert!(logs.stdout.contains("App ready!"));

    // Cleanup
    let _ = RmCommand::new(&container_name).force().execute().await;
}

// Note: These are example test functions. In a real project, you would
// typically place these in your tests/ directory or in #[cfg(test)] modules.

#[tokio::main]
async fn main() {
    println!("Testing Basics Example");
    println!("======================");
    println!();
    println!("This example contains test patterns. Run with:");
    println!("  cargo test --example testing_basics");
    println!();
    println!("Or run individual tests:");
    println!("  cargo test --example testing_basics test_with_auto_cleanup");
    println!();
    println!("See the source code for testing patterns and best practices.");
}

// For demonstration purposes, we'll also include a test module
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn demonstration_test() {
        let container_name = unique_container_name("demo");

        // Start a simple container
        let output = RunCommand::new("alpine")
            .name(&container_name)
            .cmd(vec!["echo".to_string(), "Hello from test!".to_string()])
            .execute()
            .await
            .expect("Failed to run container");

        println!("Container output: {}", output.0);
        assert!(!output.0.is_empty());
    }
}
