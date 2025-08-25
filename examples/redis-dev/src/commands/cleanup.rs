//! Cleanup all Redis instances

use anyhow::Result;
use colored::*;
use docker_wrapper::DockerCommand;
use std::io::{self, Write};

use crate::config::{Config, InstanceType};

pub async fn handle_cleanup(force: bool, filter_type: Option<String>, verbose: bool) -> Result<()> {
    let mut config = Config::load()?;

    let instances = if let Some(type_filter) = &filter_type {
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
    }.into_iter().cloned().collect::<Vec<_>>();

    if instances.is_empty() {
        let filter_msg = if let Some(ref t) = filter_type {
            format!(" of type '{}'", t)
        } else {
            String::new()
        };
        println!("{} No Redis instances found{}", "Info:".blue(), filter_msg);
        return Ok(());
    }

    // Show what will be cleaned up
    println!(
        "{} {} to clean up:",
        "Cleanup:".bold().yellow(),
        if instances.len() == 1 {
            "instance"
        } else {
            "instances"
        }
        .bold()
    );
    println!();

    for instance in &instances {
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
            instance.name.yellow(),
            type_color
        );

        if verbose {
            println!(
                "    Containers: {}",
                instance.containers.join(", ").dimmed()
            );
        }
    }

    println!();

    // Confirmation unless --force
    if !force {
        print!("{} Are you sure? [y/N]: ", "Confirm:".bold().yellow());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("Cleanup cancelled.");
            return Ok(());
        }
    }

    println!("{} Cleaning up instances...", "Cleaning:".bold().yellow());
    println!();

    let mut cleaned_count = 0;
    let mut error_count = 0;

    for instance in instances {
        if verbose {
            println!(
                "{} Cleaning up: {}",
                "Processing".cyan(),
                instance.name.bold()
            );
        }

        // Stop and remove all containers for this instance
        for container in &instance.containers {
            // Stop container
            if let Err(e) = docker_wrapper::StopCommand::new(container).execute().await {
                if verbose {
                    println!(
                        "  {} Failed to stop {}: {}",
                        "Warning:".yellow(),
                        container,
                        e
                    );
                }
                error_count += 1;
                continue;
            }

            // Remove container
            if let Err(e) = docker_wrapper::RmCommand::new(container)
                .force()
                .volumes()
                .execute()
                .await
            {
                if verbose {
                    println!(
                        "  {} Failed to remove {}: {}",
                        "Warning:".yellow(),
                        container,
                        e
                    );
                }
                error_count += 1;
                continue;
            }

            if verbose {
                println!(
                    "  {} Removed container: {}",
                    "Removed:".green(),
                    container.dimmed()
                );
            }
        }

        // For cluster instances, also clean up networks
        if instance.instance_type == InstanceType::Cluster {
            let network_name = format!("{}-network", instance.name);
            if let Err(e) = docker_wrapper::NetworkRmCommand::new(&network_name)
                .execute()
                .await
            {
                if verbose {
                    println!(
                        "  {} Failed to remove network {}: {}",
                        "Warning:".yellow(),
                        network_name,
                        e
                    );
                }
                // Don't count network removal failures as critical
            } else if verbose {
                println!(
                    "  {} Removed network: {}",
                    "Removed:".green(),
                    network_name.dimmed()
                );
            }
        }

        // Remove from config
        config.remove_instance(&instance.name);
        cleaned_count += 1;

        println!(
            "{} Cleaned up: {}",
            "Success:".green(),
            instance.name.bold().green()
        );
    }

    // Save updated config
    config.save()?;

    println!();
    if error_count > 0 {
        println!(
            "{} Cleanup completed with {} errors. {} instances cleaned up.",
            "Warning:".yellow(),
            error_count.to_string().red(),
            cleaned_count.to_string().green()
        );
    } else {
        println!(
            "{} All {} instances cleaned up successfully!",
            "Success:".bold().green(),
            cleaned_count.to_string().green()
        );
    }

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
