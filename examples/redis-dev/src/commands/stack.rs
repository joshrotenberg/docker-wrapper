//! Redis Stack instance management

use anyhow::{Context, Result};
use colored::*;
use docker_wrapper::{DockerCommand, RedisTemplate, Template};
use std::collections::HashMap;
use tokio::process::Command as ProcessCommand;

use crate::cli::{InfoArgs, StackAction, StackStartArgs, StopArgs};
use crate::config::{generate_password, Config, ConnectionInfo, InstanceInfo, InstanceType};

pub async fn handle_action(action: StackAction, verbose: bool) -> Result<()> {
    match action {
        StackAction::Start(args) => start_stack(args, verbose).await,
        StackAction::Stop(args) => stop_stack(args, verbose).await,
        StackAction::Info(args) => info_stack(args, verbose).await,
    }
}

async fn start_stack(args: StackStartArgs, verbose: bool) -> Result<()> {
    let mut config = Config::load()?;

    // Generate name if not provided
    let name = args
        .name
        .unwrap_or_else(|| config.generate_name(&InstanceType::Stack));

    if verbose {
        println!(
            "{} Starting Redis Stack instance: {}",
            "Starting".cyan(),
            name.bold()
        );
    }

    // Generate password if not provided
    let password = args.password.unwrap_or_else(generate_password);

    // Create Redis Stack template (Redis Stack support requires template enhancement)
    let mut template = RedisTemplate::new(&name)
        .port(args.port)
        .password(&password);
    // TODO: Add .with_redis_stack() once template supports it

    if args.persist {
        template = template.with_persistence(format!("{}-data", name));
    }

    if let Some(memory) = &args.memory {
        template = template.memory_limit(memory);
    }

    // TODO: Add Redis Insight support once template supports it
    // if args.insight {
    //     template = template.with_redis_insight().redis_insight_port(args.insight_port);
    // }

    // Start the instance
    if verbose {
        println!(
            "{} Initializing Redis Stack (this may take a moment)...",
            "Initializing".yellow()
        );
    }

    let result = match template.start().await {
        Ok(result) => result,
        Err(e) => {
            let error_msg = format!("{}", e);
            
            // Clean up any failed container that might have been created
            if let Err(cleanup_err) = docker_wrapper::RmCommand::new(&name).force().execute().await {
                if verbose {
                    println!("{} Failed to clean up container: {}", "Warning:".yellow(), cleanup_err);
                }
            }
            
            // Rollback counter since we failed
            config.counters.entry(InstanceType::Stack.to_string()).and_modify(|c| {
                if *c > 0 {
                    *c -= 1;
                }
            });
            config.save()?;
            
            if error_msg.contains("is already in use by container") || error_msg.contains("Conflict") {
                return Err(anyhow::anyhow!(
                    "Failed to start Redis Stack instance '{}': Container name already exists. Use --name to specify a different name or run 'redis-dev cleanup' to clean up old instances.",
                    name
                ));
            } else if error_msg.contains("port is already allocated") || 
                      error_msg.contains("bind") || 
                      error_msg.contains("Bind for") ||
                      error_msg.contains("failed to set up container networking") ||
                      error_msg.contains("address already in use") {
                return Err(anyhow::anyhow!(
                    "Failed to start Redis Stack instance '{}': Port {} is already in use. Stop other Redis instances or use --port to specify a different port.",
                    name, args.port
                ));
            } else {
                return Err(anyhow::anyhow!("Failed to start Redis Stack instance '{}': {}", name, e));
            }
        }
    };

    if verbose {
        println!("{} {}", "Success:".green(), result);
    }

    // Build containers list (insight not yet supported)
    let containers = vec![name.clone()];

    // Build additional ports info (insight not yet supported)
    let additional_ports = HashMap::new();
    // TODO: Add insight support
    // if args.insight {
    //     additional_ports.insert("redisinsight".to_string(), args.insight_port);
    // }

    // Store instance info
    let instance_info = InstanceInfo {
        name: name.clone(),
        instance_type: InstanceType::Stack,
        created_at: chrono::Utc::now().to_rfc3339(),
        ports: vec![args.port],
        containers,
        connection_info: ConnectionInfo {
            host: "localhost".to_string(),
            port: args.port,
            password: Some(password.clone()),
            url: format!("redis://default:{password}@localhost:{}", args.port),
            additional_ports,
        },
        metadata: {
            let mut map = HashMap::new();
            map.insert("persist".to_string(), serde_json::Value::Bool(args.persist));
            map.insert("insight".to_string(), serde_json::Value::Bool(args.insight));
            if let Some(memory) = args.memory {
                map.insert("memory".to_string(), serde_json::Value::String(memory));
            }
            // Track enabled modules
            let modules = vec!["JSON", "Search", "Graph", "TimeSeries", "Bloom"];
            map.insert(
                "modules".to_string(),
                serde_json::Value::Array(
                    modules
                        .into_iter()
                        .map(|m| serde_json::Value::String(m.to_string()))
                        .collect(),
                ),
            );
            map
        },
    };

    config.add_instance(instance_info);
    config.save()?;

    // Display connection info
    println!();
    println!(
        "{} Redis Stack instance started:",
        "Success:".bold().green()
    );
    println!("  {}: {}", "Name".bold(), name.green());
    println!(
        "  {}: {}:{}",
        "Address".bold(),
        "localhost".cyan(),
        args.port.to_string().cyan()
    );
    println!("  {}: {}", "Password".bold(), password.yellow());
    println!(
        "  {}: {}",
        "URL".bold(),
        format!("redis://default:{password}@localhost:{}", args.port).blue()
    );
    println!(
        "  {}: {}",
        "Modules".bold(),
        "JSON, Search, Graph, TimeSeries, Bloom".purple()
    );

    if args.persist {
        println!(
            "  {}: {}",
            "Data Volume".bold(),
            format!("{}-data", name).purple()
        );
    }

    // TODO: Add Redis Insight support
    // if args.insight {
    //     println!("  {}: http://localhost:{}", "RedisInsight".bold(), args.insight_port.to_string().magenta());
    // }

    println!();
    println!("{} Example commands:", "Examples:".bold().blue());
    println!(
        "  JSON: {}",
        "redis-cli JSON.SET user:1 $ '{\"name\":\"John\",\"age\":30}'".dimmed()
    );
    println!(
        "  Search: {}",
        "redis-cli FT.CREATE idx ON HASH PREFIX 1 user: SCHEMA name TEXT age NUMERIC".dimmed()
    );

    // Connect to Redis shell if requested
    if args.shell {
        println!();
        println!("{} Connecting to redis-cli...", "Shell:".bold().green());
        println!();
        
        let status = ProcessCommand::new("redis-cli")
            .args([
                "-h", "localhost",
                "-p", &args.port.to_string(),
                "-a", &password,
            ])
            .status()
            .await
            .context("Failed to start redis-cli")?;
            
        if !status.success() {
            println!("{} redis-cli exited with error", "Warning:".yellow());
        }
    }

    Ok(())
}

async fn stop_stack(args: StopArgs, verbose: bool) -> Result<()> {
    let mut config = Config::load()?;

    // Get instance name
    let name = if let Some(name) = args.name {
        name
    } else {
        // Get the latest stack instance
        if let Some(instance) = config.get_latest_instance(&InstanceType::Stack) {
            instance.name.clone()
        } else {
            anyhow::bail!("No Redis Stack instances found. Use --name to specify an instance.");
        }
    };

    // Check if instance exists
    let instance = config.get_instance(&name).context("Instance not found")?;

    if instance.instance_type != InstanceType::Stack {
        anyhow::bail!("Instance '{}' is not a Redis Stack instance", name);
    }

    if verbose {
        println!(
            "{} Stopping Redis Stack instance: {}",
            "Stopping".cyan(),
            name.bold()
        );
    }

    // Stop and remove all containers for this instance
    for container in &instance.containers {
        // Stop container
        let stop_cmd = docker_wrapper::StopCommand::new(container);
        stop_cmd
            .execute()
            .await
            .with_context(|| format!("Failed to stop container: {}", container))?;

        // Remove container
        let rm_cmd = docker_wrapper::RmCommand::new(container).force().volumes();
        rm_cmd
            .execute()
            .await
            .with_context(|| format!("Failed to remove container: {}", container))?;

        if verbose {
            println!(
                "  {} Removed container: {}",
                "Removed:".green(),
                container.dimmed()
            );
        }
    }

    // Remove from config
    config.remove_instance(&name);
    config.save()?;

    println!(
        "{} Redis Stack instance '{}' stopped and removed",
        "Success:".green(),
        name.bold()
    );

    Ok(())
}

async fn info_stack(args: InfoArgs, verbose: bool) -> Result<()> {
    let config = Config::load()?;

    // Get instance name
    let name = if let Some(name) = args.name {
        name
    } else {
        // Get the latest stack instance
        if let Some(instance) = config.get_latest_instance(&InstanceType::Stack) {
            instance.name.clone()
        } else {
            anyhow::bail!("No Redis Stack instances found. Use --name to specify an instance.");
        }
    };

    // Get instance info
    let instance = config.get_instance(&name).context("Instance not found")?;

    if instance.instance_type != InstanceType::Stack {
        anyhow::bail!("Instance '{}' is not a Redis Stack instance", name);
    }

    // Display info based on format
    match args.format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(instance)?);
        }
        _ => {
            println!(
                "{} Redis Stack Instance: {}",
                "Info:".bold().cyan(),
                name.bold().green()
            );
            println!("  {}: {}", "Type".bold(), "Redis Stack".magenta());
            println!("  {}: {}", "Created".bold(), instance.created_at.dimmed());
            println!(
                "  {}: {}:{}",
                "Address".bold(),
                instance.connection_info.host.cyan(),
                instance.connection_info.port.to_string().cyan()
            );

            if let Some(password) = &instance.connection_info.password {
                println!("  {}: {}", "Password".bold(), password.yellow());
            }

            println!(
                "  {}: {}",
                "URL".bold(),
                instance.connection_info.url.blue()
            );
            println!(
                "  {}: {}",
                "Containers".bold(),
                instance.containers.join(", ").purple()
            );

            // Show modules
            if let Some(modules) = instance.metadata.get("modules") {
                if let Some(modules_array) = modules.as_array() {
                    let module_names: Vec<String> = modules_array
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect();
                    println!(
                        "  {}: {}",
                        "Modules".bold(),
                        module_names.join(", ").purple()
                    );
                }
            }

            // Additional services
            if instance
                .metadata
                .get("insight")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                if let Some(insight_port) = instance
                    .connection_info
                    .additional_ports
                    .get("redisinsight")
                {
                    println!(
                        "  {}: http://localhost:{}",
                        "RedisInsight".bold(),
                        insight_port.to_string().magenta()
                    );
                }
            }

            if verbose {
                println!("  {}: {:?}", "All Metadata".bold(), instance.metadata);
            }
        }
    }

    Ok(())
}
