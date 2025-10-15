//! Basic Redis instance management

use anyhow::{Context, Result};
use colored::*;
use docker_wrapper::{DockerCommand, RedisTemplate, Template};
use std::collections::HashMap;
use tokio::process::Command as ProcessCommand;
use tracing::{debug, warn};

use crate::cli::{BasicStartArgs, InfoArgs, RedisAction, StopArgs};
use crate::config::{generate_password, Config, ConnectionInfo, InstanceInfo, InstanceType};

pub async fn handle_action(action: RedisAction, verbose: bool) -> Result<()> {
    match action {
        RedisAction::Start(args) => start_basic(args, verbose).await,
        RedisAction::Stop(args) => stop_basic(args, verbose).await,
        RedisAction::Info(args) => info_basic(args, verbose).await,
    }
}

async fn start_basic(args: BasicStartArgs, verbose: bool) -> Result<()> {
    let mut config = Config::load()?;

    // Generate name if not provided
    let name = args
        .name
        .unwrap_or_else(|| config.generate_name(&InstanceType::Basic));

    if verbose {
        println!(
            "{} Starting basic Redis instance: {}",
            "Starting".cyan(),
            name.bold()
        );
    }

    // Generate password if not provided
    let password = args.password.unwrap_or_else(generate_password);

    // Create Redis template
    let mut template = RedisTemplate::new(&name)
        .port(args.port)
        .password(&password);

    if args.persist {
        template = template.with_persistence(format!("{}-data", name));
    }

    if let Some(ref memory) = args.memory {
        template = template.memory_limit(memory);
    }

    // Start the instance
    let result = match template.start().await {
        Ok(result) => result,
        Err(e) => {
            let error_msg = format!("{}", e);
            debug!("Full error message: {}", error_msg);

            // Clean up any failed container that might have been created
            if let Err(cleanup_err) = docker_wrapper::RmCommand::new(&name)
                .force()
                .execute()
                .await
            {
                warn!("Failed to clean up container {}: {}", name, cleanup_err);
            }

            // Rollback counter since we failed
            config
                .counters
                .entry(InstanceType::Basic.to_string())
                .and_modify(|c| {
                    if *c > 0 {
                        *c -= 1;
                    }
                });
            config.save()?;

            if error_msg.contains("is already in use by container")
                || error_msg.contains("Conflict")
                || error_msg.contains("already exists")
            {
                return Err(anyhow::anyhow!(
                    "Failed to start Redis instance '{}': Container name already exists. Use --name to specify a different name or run 'redis-dev cleanup' to clean up old instances.",
                    name
                ));
            } else if error_msg.contains("port is already allocated")
                || error_msg.contains("bind")
                || error_msg.contains("Bind for")
                || error_msg.contains("failed to set up container networking")
                || error_msg.contains("address already in use")
                || error_msg.contains("driver failed programming external connectivity")
            {
                return Err(anyhow::anyhow!(
                    "Failed to start Redis instance '{}': Port {} is already in use. Stop other Redis instances or use --port to specify a different port.",
                    name, args.port
                ));
            } else {
                return Err(anyhow::anyhow!(
                    "Failed to start Redis instance '{}': {}",
                    name,
                    e
                ));
            }
        }
    };

    if verbose {
        println!("{} {}", "Success:".green(), result);
    }

    // Start RedisInsight if requested
    let mut insight_container = None;
    if args.with_insight {
        use crate::commands::insight::{
            create_redis_connection, print_insight_instructions, start_insight, ConnectionType,
            InsightConfig,
        };

        let insight_config = InsightConfig::new(&name, args.insight_port);
        match start_insight(insight_config, verbose).await {
            Ok(container_id) => {
                insight_container = Some(container_id);

                // Create connection info for Insight
                let connections = vec![create_redis_connection(
                    name.clone(),
                    "host.docker.internal".to_string(), // Use host.docker.internal for Docker Desktop
                    args.port,
                    Some(password.clone()),
                    ConnectionType::Standalone,
                )];

                // Print instructions
                print_insight_instructions(args.insight_port, connections);
            }
            Err(e) => {
                warn!("Failed to start RedisInsight: {}", e);
                println!(
                    "{} RedisInsight failed to start: {}",
                    "Warning:".yellow(),
                    e
                );
            }
        }
    }

    // Store instance info
    let instance_info = InstanceInfo {
        name: name.clone(),
        instance_type: InstanceType::Basic,
        created_at: chrono::Utc::now().to_rfc3339(),
        ports: vec![args.port],
        containers: vec![name.clone()], // Container name same as instance name
        connection_info: ConnectionInfo {
            host: "localhost".to_string(),
            port: args.port,
            password: Some(password.clone()),
            url: format!("redis://default:{password}@localhost:{}", args.port),
            additional_ports: HashMap::new(),
        },
        metadata: {
            let mut map = HashMap::new();
            map.insert("persist".to_string(), serde_json::Value::Bool(args.persist));
            if let Some(memory) = &args.memory {
                map.insert(
                    "memory".to_string(),
                    serde_json::Value::String(memory.clone()),
                );
            }
            if let Some(ref container_id) = insight_container {
                map.insert(
                    "insight_container".to_string(),
                    serde_json::Value::String(container_id.clone()),
                );
                map.insert(
                    "insight_port".to_string(),
                    serde_json::Value::Number(args.insight_port.into()),
                );
            }
            map
        },
    };

    config.add_instance(instance_info);
    config.save()?;

    // Display connection info
    println!();
    println!(
        "{} Basic Redis instance started:",
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

    if args.persist {
        println!(
            "  {}: {}",
            "Data Volume".bold(),
            format!("{}-data", name).purple()
        );
    }

    // Connect to Redis shell if requested
    if args.shell {
        println!();
        println!("{} Connecting to redis-cli...", "Shell:".bold().green());
        println!();

        let status = ProcessCommand::new("redis-cli")
            .args([
                "-h",
                "localhost",
                "-p",
                &args.port.to_string(),
                "-a",
                &password,
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

async fn stop_basic(args: StopArgs, verbose: bool) -> Result<()> {
    let mut config = Config::load()?;

    // Get instance name
    let name = if let Some(name) = args.name {
        name
    } else {
        // Get the latest basic instance
        if let Some(instance) = config.get_latest_instance(&InstanceType::Basic) {
            instance.name.clone()
        } else {
            anyhow::bail!("No basic Redis instances found. Use --name to specify an instance.");
        }
    };

    // Check if instance exists
    let instance = config
        .get_instance(&name)
        .context("Instance not found")?
        .clone(); // Clone to avoid borrow issues

    if instance.instance_type != InstanceType::Basic {
        anyhow::bail!("Instance '{}' is not a basic Redis instance", name);
    }

    if verbose {
        println!(
            "{} Stopping basic Redis instance: {}",
            "Stopping".cyan(),
            name.bold()
        );
    }

    // Stop the container
    let stop_cmd = docker_wrapper::StopCommand::new(&name);
    stop_cmd
        .execute()
        .await
        .with_context(|| format!("Failed to stop Redis instance: {}", name))?;

    // Remove the container
    let rm_cmd = docker_wrapper::RmCommand::new(&name).force().volumes();
    rm_cmd
        .execute()
        .await
        .with_context(|| format!("Failed to remove Redis container: {}", name))?;

    // Stop and remove Insight container if it exists
    if let Some(insight_container) = instance.metadata.get("insight_container") {
        if let Some(_container_name) = insight_container.as_str() {
            if verbose {
                println!("  {} Stopping RedisInsight...", "Cleanup:".cyan());
            }

            // Use the insight module's stop function
            use crate::commands::insight::stop_insight;
            if let Err(e) = stop_insight(&name).await {
                warn!("Failed to stop RedisInsight: {}", e);
            }
        }
    }

    // Remove from config
    config.remove_instance(&name);
    config.save()?;

    println!(
        "{} Basic Redis instance '{}' stopped and removed",
        "Success:".green(),
        name.bold()
    );

    Ok(())
}

async fn info_basic(args: InfoArgs, verbose: bool) -> Result<()> {
    let config = Config::load()?;

    // Get instance name
    let name = if let Some(name) = args.name {
        name
    } else {
        // Get the latest basic instance
        if let Some(instance) = config.get_latest_instance(&InstanceType::Basic) {
            instance.name.clone()
        } else {
            anyhow::bail!("No basic Redis instances found. Use --name to specify an instance.");
        }
    };

    // Get instance info
    let instance = config.get_instance(&name).context("Instance not found")?;

    if instance.instance_type != InstanceType::Basic {
        anyhow::bail!("Instance '{}' is not a basic Redis instance", name);
    }

    // Display info based on format
    match args.format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(instance)?);
        }
        _ => {
            println!(
                "{} Basic Redis Instance: {}",
                "Info:".bold().cyan(),
                name.bold().green()
            );
            println!("  {}: {}", "Type".bold(), "Basic Redis".cyan());
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
                "Container".bold(),
                instance.containers.join(", ").purple()
            );

            if verbose {
                println!("  {}: {:?}", "Metadata".bold(), instance.metadata);
            }
        }
    }

    Ok(())
}
