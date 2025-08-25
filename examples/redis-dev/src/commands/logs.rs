//! Logs command for viewing Redis instance logs

use anyhow::{Context, Result};
use colored::*;
use tokio::process::Command;

use crate::config::Config;

pub async fn handle_logs(
    name: Option<String>,
    follow: bool,
    tail: u32,
    timestamps: bool,
    verbose: bool,
) -> Result<()> {
    let config = Config::load()?;

    // Determine which instance to show logs for
    let instance_name = if let Some(name) = name {
        // Validate the named instance exists
        if config.get_instance(&name).is_none() {
            anyhow::bail!("Instance '{}' not found. Use 'redis-dev list' to see available instances.", name);
        }
        name
    } else {
        // Get the most recent instance (across all types)
        if config.instances.is_empty() {
            anyhow::bail!("No Redis instances found. Use 'redis-dev basic start' or similar to create an instance.");
        }
        
        // Find the most recently created instance
        config.instances
            .values()
            .max_by_key(|instance| &instance.created_at)
            .map(|instance| instance.name.clone())
            .context("No instances found")?
    };

    // Get instance info to verify container name
    let instance = config.get_instance(&instance_name)
        .context("Instance not found")?;

    if verbose {
        println!("{} Showing logs for instance: {}", "Info:".cyan(), instance_name.bold());
        println!("  Type: {}", instance.instance_type.to_string().yellow());
        println!("  Containers: {}", instance.containers.join(", ").purple());
        println!();
    }

    // For cluster instances, show logs from the first container
    let container_name = &instance.containers[0];

    // Show appropriate message
    if follow {
        println!("{} Following logs for '{}' (press Ctrl+C to exit):", "Logs:".bold().blue(), instance_name);
        if !timestamps {
            println!("{} Tip: Use --timestamps to show log timestamps", "Tip:".dimmed());
        }
    } else {
        println!("{} Last {} lines for '{}':", "Logs:".bold().blue(), tail, instance_name);
    }
    
    println!("{} Redis typically produces few logs after startup unless there are connections or errors.", "Note:".dimmed());
    println!();

    // Build and execute docker logs command directly
    let mut cmd = Command::new("docker");
    cmd.arg("logs");
    
    if follow {
        cmd.arg("-f");
    }
    
    if timestamps {
        cmd.arg("--timestamps");
    }
    
    cmd.arg("--tail").arg(tail.to_string());
    cmd.arg(container_name);

    // Execute the command
    let status = cmd.status().await
        .context("Failed to execute docker logs command")?;
        
    if !status.success() {
        anyhow::bail!("Docker logs command failed for container '{}'", container_name);
    }

    Ok(())
}