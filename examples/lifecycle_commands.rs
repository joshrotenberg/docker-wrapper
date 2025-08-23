//! Example demonstrating container lifecycle commands including rm, kill, and logs

use docker_wrapper::command::DockerCommandV2;
use docker_wrapper::{ KillCommand, LogsCommand, RmCommand, RunCommand};
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

    // Send SIGTERM to the container
    println!("\nSending SIGTERM to container...");
    let _kill_result = KillCommand::new(container_name)
        .signal("SIGTERM")
        .run()
        .await?;
    println!("Container killed with SIGTERM signal");

    // Wait a moment for the container to stop
    sleep(Duration::from_secs(1)).await;

    // Remove the container
    println!("\nRemoving container...");
    let _rm_result = RmCommand::new(container_name).run().await?;
    println!("Container removed successfully");

    println!("\nLifecycle example completed!");
    Ok(())
}
