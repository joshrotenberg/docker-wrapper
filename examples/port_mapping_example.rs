//! Example demonstrating the enhanced port mapping API
//!
//! This example shows how to:
//! - Use the `.rm()` alias for automatic container removal
//! - Use dynamic port allocation with `.port_dyn()`
//! - Get mapped ports directly from the ContainerId

use docker_wrapper::{DockerCommand, RunCommand, StopCommand};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Redis with dynamic port allocation...");

    // Start Redis with dynamic port allocation and auto-removal
    let container_id = RunCommand::new("redis:alpine")
        .name("test-redis")
        .port_dyn(6379) // Let Docker assign the host port
        .detach()
        .rm() // Container will be removed when it stops
        .execute()
        .await?;

    println!("Container started: {}", container_id.short());

    // Get the actual mapped port
    let port_mappings = container_id.port_mappings().await?;
    
    if let Some(mapping) = port_mappings.first() {
        println!(
            "Redis is available at {}:{}",
            mapping.host_ip, mapping.host_port
        );
        println!(
            "You can connect with: redis-cli -h {} -p {}",
            mapping.host_ip, mapping.host_port
        );
    }

    // Alternative: Get specific port mapping
    if let Some(mapping) = container_id.port_mapping(6379).await? {
        println!(
            "Port 6379 is mapped to: {}:{}",
            mapping.host_ip, mapping.host_port
        );
    }

    println!("\nContainer will run for 10 seconds...");
    sleep(Duration::from_secs(10)).await;

    // Stop the container (it will be automatically removed due to .rm())
    println!("Stopping container...");
    StopCommand::new(container_id.as_str())
        .execute()
        .await?;

    println!("Container stopped and removed automatically!");

    Ok(())
}