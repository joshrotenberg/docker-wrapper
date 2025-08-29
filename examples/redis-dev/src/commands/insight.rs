//! Redis Insight container management for GUI access to Redis instances

use anyhow::{Context, Result};
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
    pub fn with_network(mut self, network: impl Into<String>) -> Self {
        self.network = Some(network.into());
        self
    }
}

/// Start a Redis Insight container
pub async fn start_insight(config: InsightConfig) -> Result<String> {
    let container_name = format!("{}-insight", config.name);
    
    let mut cmd = RunCommand::new("redis/redisinsight:latest")
        .name(&container_name)
        .port(config.port, 5540)  // RedisInsight runs on port 5540 inside container
        .detach()
        .remove();  // Auto-remove when stopped

    // Add network if specified
    if let Some(network) = config.network {
        cmd = cmd.network(&network);
    }

    // Set environment variables for Redis Insight
    cmd = cmd
        .env("REDISINSIGHT_PORT", "5540")
        .env("REDISINSIGHT_HOST", "0.0.0.0");

    let container_id = cmd
        .execute()
        .await
        .context("Failed to start Redis Insight container")?;

    Ok(container_id.0)
}

/// Stop a Redis Insight container
pub async fn stop_insight(name: &str) -> Result<()> {
    use docker_wrapper::{RmCommand, StopCommand};
    
    let container_name = format!("{}-insight", name);
    
    // Stop the container
    StopCommand::new(&container_name)
        .execute()
        .await
        .ok();  // Ignore if already stopped
    
    // Remove the container
    RmCommand::new(&container_name)
        .force()
        .execute()
        .await
        .ok();  // Ignore if already removed
    
    Ok(())
}

/// Add Redis connection to Insight via API (requires Insight to be running)
pub async fn add_connection_to_insight(
    insight_port: u16,
    redis_host: &str,
    redis_port: u16,
    password: Option<&str>,
    connection_name: &str,
) -> Result<()> {
    // Note: Redis Insight has an API for adding connections programmatically
    // This would typically be done via HTTP POST to the Insight API
    // For now, we'll just print instructions for manual configuration
    
    println!("\n📊 Redis Insight Configuration:");
    println!("1. Open http://localhost:{} in your browser", insight_port);
    println!("2. Click 'Add Redis Database'");
    println!("3. Enter the following details:");
    println!("   - Host: {}", redis_host);
    println!("   - Port: {}", redis_port);
    if let Some(pwd) = password {
        println!("   - Password: {}", pwd);
    }
    println!("   - Database Alias: {}", connection_name);
    println!("4. Click 'Add Redis Database' to connect");
    
    Ok(())
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
pub enum ConnectionType {
    Standalone,
    Cluster,
    Sentinel { sentinel_port: u16 },
    Enterprise,
}

/// Generate Insight connection instructions for various Redis types
pub fn generate_insight_instructions(
    insight_port: u16,
    connections: Vec<RedisConnection>,
) -> String {
    let mut instructions = format!("\n📊 Redis Insight is running at http://localhost:{}\n\n", insight_port);
    instructions.push_str("To add your Redis instances:\n\n");
    
    for (i, conn) in connections.iter().enumerate() {
        instructions.push_str(&format!("{}. {} ({:?}):\n", i + 1, conn.name, conn.connection_type));
        
        match conn.connection_type {
            ConnectionType::Standalone => {
                instructions.push_str(&format!("   - Connection Type: Standalone\n"));
                instructions.push_str(&format!("   - Host: {}\n", conn.host));
                instructions.push_str(&format!("   - Port: {}\n", conn.port));
            }
            ConnectionType::Cluster => {
                instructions.push_str(&format!("   - Connection Type: Redis Cluster\n"));
                instructions.push_str(&format!("   - Seed nodes: {}:{}\n", conn.host, conn.port));
            }
            ConnectionType::Sentinel { sentinel_port } => {
                instructions.push_str(&format!("   - Connection Type: Redis Sentinel\n"));
                instructions.push_str(&format!("   - Sentinel Host: {}\n", conn.host));
                instructions.push_str(&format!("   - Sentinel Port: {}\n", sentinel_port));
            }
            ConnectionType::Enterprise => {
                instructions.push_str(&format!("   - Connection Type: Redis Enterprise\n"));
                instructions.push_str(&format!("   - Host: {}\n", conn.host));
                instructions.push_str(&format!("   - Port: {}\n", conn.port));
            }
        }
        
        if let Some(ref pwd) = conn.password {
            instructions.push_str(&format!("   - Password: {}\n", pwd));
        }
        instructions.push_str("\n");
    }
    
    instructions
}

/// Check if Redis Insight container is running
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
pub async fn get_insight_info(name: &str) -> Result<HashMap<String, String>> {
    use docker_wrapper::InspectCommand;
    
    let container_name = format!("{}-insight", name);
    let result = InspectCommand::new(&container_name)
        .execute()
        .await
        .context("Failed to inspect Redis Insight container")?;
    
    // Parse JSON output to extract relevant info
    let containers: serde_json::Value = serde_json::from_str(&result.stdout)
        .context("Failed to parse inspect output")?;
    
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
                            if let Some(host_port) = first.get("HostPort").and_then(|p| p.as_str()) {
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