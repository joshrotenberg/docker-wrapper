//! Redis Insight container management for GUI access to Redis instances

use anyhow::{Context, Result};
use colored::*;
use docker_wrapper::{DockerCommand, RunCommand};
use std::collections::HashMap;

/// Redis Insight configuration
pub struct InsightConfig {
    pub name: String,
    pub port: u16,
    pub network: Option<String>,
}

impl InsightConfig {
    /// Create a new Redis Insight configuration
    pub fn new(name: impl Into<String>, port: u16) -> Self {
        Self {
            name: name.into(),
            port,
            network: None,
        }
    }

    /// Set the network for Insight to connect to
    #[allow(dead_code)]
    pub fn with_network(mut self, network: impl Into<String>) -> Self {
        self.network = Some(network.into());
        self
    }
}

/// Start a Redis Insight container
pub async fn start_insight(config: InsightConfig, verbose: bool) -> Result<String> {
    let container_name = format!("{}-insight", config.name);

    if verbose {
        println!(
            "  {} Starting RedisInsight on port {}...",
            "Insight:".cyan(),
            config.port
        );
    }

    let mut cmd = RunCommand::new("redis/redisinsight:latest")
        .name(&container_name)
        .port(config.port, 5540) // RedisInsight runs on port 5540 inside container
        .detach();

    // Add network if specified
    if let Some(network) = &config.network {
        cmd = cmd.network(network);
    }

    // Set environment variables for Redis Insight
    cmd = cmd
        .env("REDISINSIGHT_PORT", "5540")
        .env("REDISINSIGHT_HOST", "0.0.0.0")
        .env("REDISINSIGHT_LOG_LEVEL", "warning"); // Reduce log noise

    let container_id = cmd
        .execute()
        .await
        .context("Failed to start Redis Insight container")?;

    if verbose {
        println!(
            "  {} RedisInsight container started: {}",
            "Success".green(),
            container_name
        );
    }

    Ok(container_id.0)
}

/// Stop a Redis Insight container
pub async fn stop_insight(name: &str) -> Result<()> {
    use docker_wrapper::{RmCommand, StopCommand};

    let container_name = format!("{}-insight", name);

    // Stop the container
    StopCommand::new(&container_name).execute().await.ok(); // Ignore if already stopped

    // Remove the container
    RmCommand::new(&container_name).force().execute().await.ok(); // Ignore if already removed

    Ok(())
}

/// Print instructions for configuring Redis Insight
pub fn print_insight_instructions(insight_port: u16, connections: Vec<RedisConnection>) {
    println!("\n{}", "RedisInsight GUI:".bold().underline());
    println!(
        "  {} http://localhost:{}",
        "Access at:".cyan(),
        insight_port
    );

    if !connections.is_empty() {
        println!("\n  {}", "To add Redis connections:".yellow());
        println!("  1. Click 'I already have a database'");
        println!("  2. Click 'Connect to Redis Database'");

        for conn in connections {
            println!("\n  {} {}:", "For".cyan(), conn.name);
            match conn.connection_type {
                ConnectionType::Standalone | ConnectionType::Enterprise => {
                    println!("    - Host: {}", conn.host);
                    println!("    - Port: {}", conn.port);
                }
                ConnectionType::Cluster => {
                    println!("    - Connection Type: OSS Cluster");
                    println!("    - Host: {}", conn.host);
                    println!("    - Port: {}", conn.port);
                }
                ConnectionType::Sentinel { sentinel_port } => {
                    println!("    - Connection Type: Sentinel");
                    println!("    - Sentinel Host: {}", conn.host);
                    println!("    - Sentinel Port: {}", sentinel_port);
                }
            }
            if let Some(ref pwd) = conn.password {
                println!("    - Password: {}", pwd);
            }
            println!("    - Database Alias: {}", conn.name);
        }
    }
}

/// Configuration for multiple Redis instances to add to Insight
pub struct RedisConnection {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub connection_type: ConnectionType,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ConnectionType {
    Standalone,
    Cluster,
    Sentinel { sentinel_port: u16 },
    Enterprise,
}

/// Create a Redis connection configuration for Insight
pub fn create_redis_connection(
    name: String,
    host: String,
    port: u16,
    password: Option<String>,
    connection_type: ConnectionType,
) -> RedisConnection {
    RedisConnection {
        name,
        host,
        port,
        password,
        connection_type,
    }
}

/// Check if Redis Insight container is running
#[allow(dead_code)]
pub async fn is_insight_running(name: &str) -> Result<bool> {
    use docker_wrapper::PsCommand;

    let container_name = format!("{}-insight", name);
    let output = PsCommand::new()
        .filter(format!("name={}", container_name))
        .quiet()
        .execute()
        .await?;

    Ok(!output.stdout.trim().is_empty())
}

/// Get Redis Insight container info
#[allow(dead_code)]
pub async fn get_insight_info(name: &str) -> Result<HashMap<String, String>> {
    use docker_wrapper::InspectCommand;

    let container_name = format!("{}-insight", name);
    let result = InspectCommand::new(&container_name)
        .execute()
        .await
        .context("Failed to inspect Redis Insight container")?;

    // Parse JSON output to extract relevant info
    let containers: serde_json::Value =
        serde_json::from_str(&result.stdout).context("Failed to parse inspect output")?;

    let mut info = HashMap::new();

    if let Some(container) = containers.as_array().and_then(|arr| arr.first()) {
        // Extract useful information
        if let Some(state) = container.get("State") {
            if let Some(status) = state.get("Status").and_then(|s| s.as_str()) {
                info.insert("status".to_string(), status.to_string());
            }
        }

        if let Some(config) = container.get("Config") {
            if let Some(image) = config.get("Image").and_then(|i| i.as_str()) {
                info.insert("image".to_string(), image.to_string());
            }
        }

        // Extract port mappings
        if let Some(network_settings) = container.get("NetworkSettings") {
            if let Some(ports) = network_settings.get("Ports") {
                if let Some(port_5540) = ports.get("5540/tcp") {
                    if let Some(mappings) = port_5540.as_array() {
                        if let Some(first) = mappings.first() {
                            if let Some(host_port) = first.get("HostPort").and_then(|p| p.as_str())
                            {
                                info.insert("port".to_string(), host_port.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(info)
}
