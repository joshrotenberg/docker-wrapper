//! Example demonstrating container lifecycle commands including rm, kill, and logs

use docker_wrapper::command::DockerCommand;
use docker_wrapper::{KillCommand, LogsCommand, RmCommand, RunCommand, StopCommand};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Docker Container Lifecycle Example");
    println!("===================================\n");

    // Create a test container name
    let container_name = "lifecycle-example";

    // Clean up any existing container with this name (silently)
    let _ = RmCommand::new(container_name).force().run().await;

    // Run a container in detached mode
    println!("Starting container '{}'...", container_name);
    let run_result = RunCommand::new("alpine:latest")
        .name(container_name)
        .detach()
        .cmd(vec![
            "sh".to_string(),
            "-c".to_string(),
            "for i in $(seq 1 10); do echo \"Log message $i\"; sleep 1; done".to_string(),
        ])
        .execute()
        .await?;
    println!("Container started with ID: {}", run_result.0);

    // Give it a moment to generate some logs
    sleep(Duration::from_secs(3)).await;

    // View container logs
    println!("\nFetching container logs...");
    let logs = LogsCommand::new(container_name)
        .timestamps()
        .tail("5")
        .execute()
        .await?;
    println!("Recent logs:\n{}", logs.stdout);

    // Send SIGKILL to the container (demonstrates kill command)
    println!("\nSending SIGKILL to container...");
    let _kill_result = KillCommand::new(container_name)
        .signal("SIGKILL")
        .run()
        .await?;
    println!("Container killed with SIGKILL signal");

    // Remove the container
    println!("\nRemoving container...");
    let _rm_result = RmCommand::new(container_name).run().await?;
    println!("Container removed successfully");

    // Demonstrate the proper stop -> rm pattern
    println!("\n--- Demonstrating stop -> rm pattern ---\n");

    // Start another container
    println!("Starting another container...");
    let run_result = RunCommand::new("alpine:latest")
        .name(container_name)
        .detach()
        .cmd(vec![
            "sh".to_string(),
            "-c".to_string(),
            "while true; do echo 'Running...'; sleep 1; done".to_string(),
        ])
        .execute()
        .await?;
    println!("Container started with ID: {}", run_result.0);

    sleep(Duration::from_secs(2)).await;

    // Stop the container gracefully (sends SIGTERM, waits, then SIGKILL)
    println!("\nStopping container gracefully...");
    let stop_result = StopCommand::new(container_name)
        .timeout(5)
        .execute()
        .await?;
    println!("Container stopped: {:?}", stop_result.stopped_containers);

    // Now remove it (no force needed since it's stopped)
    println!("\nRemoving container...");
    let _rm_result = RmCommand::new(container_name).run().await?;
    println!("Container removed successfully");

    println!("\nLifecycle example completed!");
    Ok(())
}
