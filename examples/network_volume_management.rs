//! Network and Volume Management Examples
//!
//! This example demonstrates Docker network and volume management patterns
//! using docker-wrapper's type-safe API.

use docker_wrapper::{
    DockerCommand, KillCommand, NetworkCreateCommand, NetworkInspectCommand, NetworkLsCommand,
    NetworkRmCommand, RmCommand, RunCommand, VolumeCreateCommand, VolumeInspectCommand,
    VolumeLsCommand, VolumeRmCommand,
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Docker Network and Volume Management Examples\n");

    // Example 1: Create and manage a custom network
    create_custom_network().await?;

    // Example 2: Create and use named volumes
    use_named_volumes().await?;

    // Example 3: Multi-container networking
    multi_container_networking().await?;

    // Example 4: Volume backup and restore pattern
    volume_backup_pattern().await?;

    // Cleanup
    cleanup().await?;

    println!("\nAll examples completed successfully!");
    Ok(())
}

/// Example 1: Create and manage a custom network
async fn create_custom_network() -> Result<(), Box<dyn Error>> {
    println!("=== Example 1: Custom Network Management ===\n");

    // Create a bridge network with custom subnet
    println!("Creating custom network 'app-network'...");
    NetworkCreateCommand::new("app-network")
        .driver("bridge")
        .subnet("172.20.0.0/16")
        .ip_range("172.20.240.0/20")
        .gateway("172.20.0.1")
        .execute()
        .await?;

    // List networks
    println!("Listing networks...");
    let output = NetworkLsCommand::new()
        .format("table {{.Name}}\t{{.Driver}}\t{{.Scope}}")
        .execute()
        .await?;
    println!("{}", output.stdout);

    // Inspect the network
    println!("Inspecting network details...");
    let output = NetworkInspectCommand::new("app-network").execute().await?;
    if !output.stdout.is_empty() {
        println!("Network details available in JSON format");
    }

    Ok(())
}

/// Example 2: Create and use named volumes
async fn use_named_volumes() -> Result<(), Box<dyn Error>> {
    println!("\n=== Example 2: Named Volume Management ===\n");

    // Create a named volume with labels
    println!("Creating named volume 'app-data'...");
    VolumeCreateCommand::new()
        .name("app-data")
        .driver("local")
        .label("app", "docker-wrapper-example")
        .label("environment", "development")
        .execute()
        .await?;

    // List volumes with filtering
    println!("Listing volumes with label filter...");
    let output = VolumeLsCommand::new()
        .filter("label", "app=docker-wrapper-example")
        .execute()
        .await?;
    println!("{}", output.stdout);

    // Use the volume in a container
    println!("Running container with named volume...");
    RunCommand::new("alpine:latest")
        .name("volume-test")
        .volume("app-data", "/data")
        .cmd(vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo 'Hello from volume!' > /data/message.txt && cat /data/message.txt".to_string(),
        ])
        .remove()
        .execute()
        .await?;

    // Inspect volume details
    println!("Inspecting volume...");
    let output = VolumeInspectCommand::new("app-data").execute().await?;
    if !output.stdout.is_empty() {
        println!("Volume details available in JSON format");
    }

    Ok(())
}

/// Example 3: Multi-container networking
async fn multi_container_networking() -> Result<(), Box<dyn Error>> {
    println!("\n=== Example 3: Multi-Container Networking ===\n");

    // Start a Redis server on the custom network
    println!("Starting Redis server...");
    let redis_output = RunCommand::new("redis:alpine")
        .name("redis-server")
        .network("app-network")
        .network_alias("redis")
        .detach()
        .execute()
        .await?;
    println!("Redis container: {}", &redis_output.0[..12]);

    // Start an application container that connects to Redis
    println!("Running client container...");
    RunCommand::new("redis:alpine")
        .name("redis-client")
        .network("app-network")
        .cmd(vec![
            "redis-cli".to_string(),
            "-h".to_string(),
            "redis".to_string(),
            "ping".to_string(),
        ])
        .remove()
        .execute()
        .await?;

    // Clean up Redis server
    println!("Stopping Redis server...");
    KillCommand::new("redis-server").execute().await?;

    RmCommand::new("redis-server").execute().await?;

    Ok(())
}

/// Example 4: Volume backup and restore pattern
async fn volume_backup_pattern() -> Result<(), Box<dyn Error>> {
    println!("\n=== Example 4: Volume Backup Pattern ===\n");

    // Create a volume with some data
    println!("Creating volume with sample data...");
    VolumeCreateCommand::new()
        .name("backup-source")
        .execute()
        .await?;

    // Add data to the volume
    RunCommand::new("alpine:latest")
        .volume("backup-source", "/source")
        .cmd(vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo 'Important data' > /source/data.txt && echo 'Config' > /source/config.txt"
                .to_string(),
        ])
        .remove()
        .execute()
        .await?;

    // Backup the volume to a tar file
    println!("Backing up volume to tar file...");
    RunCommand::new("alpine:latest")
        .volume("backup-source", "/source")
        .volume(".", "/backup")
        .cmd(vec![
            "tar".to_string(),
            "czf".to_string(),
            "/backup/backup.tar.gz".to_string(),
            "-C".to_string(),
            "/source".to_string(),
            ".".to_string(),
        ])
        .remove()
        .execute()
        .await?;

    println!("Backup created: ./backup.tar.gz");

    // Create a new volume and restore from backup
    println!("Creating restore target volume...");
    VolumeCreateCommand::new()
        .name("backup-target")
        .execute()
        .await?;

    // Restore data
    println!("Restoring data to new volume...");
    RunCommand::new("alpine:latest")
        .volume("backup-target", "/target")
        .volume(".", "/backup")
        .cmd(vec![
            "tar".to_string(),
            "xzf".to_string(),
            "/backup/backup.tar.gz".to_string(),
            "-C".to_string(),
            "/target".to_string(),
        ])
        .remove()
        .execute()
        .await?;

    // Verify restoration
    println!("Verifying restored data...");
    let output = RunCommand::new("alpine:latest")
        .volume("backup-target", "/target")
        .cmd(vec![
            "ls".to_string(),
            "-la".to_string(),
            "/target".to_string(),
        ])
        .remove()
        .execute()
        .await?;
    println!("Restored files:\n{}", output.0);

    Ok(())
}

/// Cleanup resources
async fn cleanup() -> Result<(), Box<dyn Error>> {
    println!("\n=== Cleaning up resources ===\n");

    // Remove network
    println!("Removing custom network...");
    let _ = NetworkRmCommand::new("app-network").execute().await;

    // Remove volumes
    println!("Removing volumes...");
    for volume in &["app-data", "backup-source", "backup-target"] {
        let _ = VolumeRmCommand::new(*volume).execute().await;
    }

    // Clean up backup file
    let _ = std::fs::remove_file("backup.tar.gz");

    println!("Cleanup completed!");
    Ok(())
}
