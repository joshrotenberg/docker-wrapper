//! List all Redis instances

use anyhow::Result;
use colored::*;

use crate::config::{Config, InstanceType};

pub async fn handle_list(filter_type: Option<String>, verbose: bool) -> Result<()> {
    let config = Config::load()?;

    let instances = if let Some(type_filter) = filter_type {
        let instance_type = match type_filter.to_lowercase().as_str() {
            "basic" => InstanceType::Basic,
            "stack" => InstanceType::Stack,
            "cluster" => InstanceType::Cluster,
            "sentinel" => InstanceType::Sentinel,
            "enterprise" => InstanceType::Enterprise,
            _ => {
                println!("{} Invalid type filter: {}. Valid types: basic, stack, cluster, sentinel, enterprise", 
                    "Warning:".yellow(), type_filter.red());
                return Ok(());
            }
        };
        config.list_instances_by_type(&instance_type)
    } else {
        config.list_instances()
    };

    if instances.is_empty() {
        println!("{} No Redis instances found", "Info:".blue());
        println!("  Start one with: {}", "redis-dev basic start".green());
        return Ok(());
    }

    println!("{} Redis Instances", "List:".bold().cyan());
    println!();

    // Sort by creation time (newest first)
    let mut sorted_instances = instances.clone();
    sorted_instances.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    for instance in sorted_instances {
        let type_color = match instance.instance_type {
            InstanceType::Basic => "basic".cyan(),
            InstanceType::Stack => "stack".magenta(),
            InstanceType::Cluster => "cluster".yellow(),
            InstanceType::Sentinel => "sentinel".blue(),
            InstanceType::Enterprise => "enterprise".red(),
        };

        println!(
            "  {} {} ({})",
            get_type_icon(&instance.instance_type),
            instance.name.bold().green(),
            type_color
        );

        println!(
            "    {}: {}:{}",
            "Address".dimmed(),
            instance.connection_info.host.cyan(),
            instance.connection_info.port.to_string().cyan()
        );

        if verbose {
            println!(
                "    {}: {}",
                "Created".dimmed(),
                instance.created_at.dimmed()
            );
            println!(
                "    {}: {}",
                "Containers".dimmed(),
                instance.containers.join(", ").purple()
            );

            if !instance.connection_info.additional_ports.is_empty() {
                println!(
                    "    {}: {:?}",
                    "Additional Ports".dimmed(),
                    instance.connection_info.additional_ports
                );
            }
        }

        println!();
    }

    println!("Total: {} instances", instances.len().to_string().bold());

    Ok(())
}

fn get_type_icon(instance_type: &InstanceType) -> &'static str {
    match instance_type {
        InstanceType::Basic => "[B]",
        InstanceType::Stack => "[S]",
        InstanceType::Cluster => "[C]",
        InstanceType::Sentinel => "[N]",
        InstanceType::Enterprise => "[E]",
    }
}
