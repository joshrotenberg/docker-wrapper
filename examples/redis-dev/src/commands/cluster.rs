//! Redis Cluster instance management

use anyhow::{Context, Result};
use colored::*;
use docker_wrapper::{DockerCommand, RedisClusterConnection, RedisClusterTemplate, Template};
use std::collections::HashMap;
use tokio::process::Command as ProcessCommand;

use crate::cli::{ClusterAction, ClusterStartArgs, InfoArgs, StopArgs};
use crate::config::{generate_password, Config, ConnectionInfo, InstanceInfo, InstanceType};

pub async fn handle_action(action: ClusterAction, verbose: bool) -> Result<()> {
    match action {
        ClusterAction::Start(args) => start_cluster(args, verbose).await,
        ClusterAction::Stop(args) => stop_cluster(args, verbose).await,
        ClusterAction::Info(args) => info_cluster(args, verbose).await,
    }
}

async fn start_cluster(args: ClusterStartArgs, verbose: bool) -> Result<()> {
    let mut config = Config::load()?;

    // Generate name if not provided
    let name = args
        .name
        .unwrap_or_else(|| config.generate_name(&InstanceType::Cluster));

    if verbose {
        println!(
            "{} Starting Redis Cluster: {}",
            "Starting".cyan(),
            name.bold()
        );
        println!(
            "  Masters: {}, Replicas: {}, Total nodes: {}",
            args.masters.to_string().green(),
            args.replicas.to_string().blue(),
            (args.masters + (args.masters * args.replicas))
                .to_string()
                .yellow()
        );
    }

    // Generate password if not provided
    let password = args.password.unwrap_or_else(generate_password);

    // Create Redis Cluster template
    let mut template = RedisClusterTemplate::new(&name)
        .num_masters(args.masters)
        .num_replicas(args.replicas)
        .port_base(args.port_base)
        .password(&password);

    if args.persist {
        template = template.with_persistence(format!("{}-data", name));
    }

    if let Some(memory) = &args.memory {
        template = template.memory_limit(memory);
    }

    if args.stack {
        template = template.with_redis_stack();
    }

    if args.insight {
        template = template
            .with_redis_insight()
            .redis_insight_port(args.insight_port);
    }

    // Start the cluster
    if verbose {
        println!(
            "{} Initializing cluster (this may take a moment)...",
            "Initializing".yellow()
        );
    }

    let result = match template.start().await {
        Ok(result) => result,
        Err(e) => {
            let error_msg = format!("{}", e);

            // Clean up any failed containers that might have been created
            let total_nodes = args.masters + (args.masters * args.replicas);
            for i in 0..total_nodes {
                let container_name = format!("{}-node-{}", name, i);
                if let Err(cleanup_err) = docker_wrapper::RmCommand::new(&container_name)
                    .force()
                    .execute()
                    .await
                {
                    if verbose {
                        println!(
                            "{} Failed to clean up container {}: {}",
                            "Warning:".yellow(),
                            container_name,
                            cleanup_err
                        );
                    }
                }
            }
            // Also clean up potential insight container
            if args.insight {
                let insight_name = format!("{}-insight", name);
                if let Err(cleanup_err) = docker_wrapper::RmCommand::new(&insight_name)
                    .force()
                    .execute()
                    .await
                {
                    if verbose {
                        println!(
                            "{} Failed to clean up container {}: {}",
                            "Warning:".yellow(),
                            insight_name,
                            cleanup_err
                        );
                    }
                }
            }

            // Rollback counter since we failed
            config
                .counters
                .entry(InstanceType::Cluster.to_string())
                .and_modify(|c| {
                    if *c > 0 {
                        *c -= 1;
                    }
                });
            config.save()?;

            if error_msg.contains("is already in use by container")
                || error_msg.contains("Conflict")
            {
                return Err(anyhow::anyhow!(
                    "Failed to start Redis Cluster '{}': Container name already exists. Use --name to specify a different name or run 'redis-dev cleanup' to clean up old instances.",
                    name
                ));
            } else if error_msg.contains("port is already allocated")
                || error_msg.contains("bind")
                || error_msg.contains("Bind for")
                || error_msg.contains("failed to set up container networking")
                || error_msg.contains("address already in use")
            {
                return Err(anyhow::anyhow!(
                    "Failed to start Redis Cluster '{}': Port range starting at {} is already in use. Stop other Redis instances or use --port-base to specify a different starting port.",
                    name, args.port_base
                ));
            } else {
                return Err(anyhow::anyhow!(
                    "Failed to start Redis Cluster '{}': {}",
                    name,
                    e
                ));
            }
        }
    };

    if verbose {
        println!("{} {}", "Success:".green(), result);
    }

    // Get connection info
    let connection = RedisClusterConnection::from_template(&template);

    // Build container list (node containers + optional insight)
    let mut containers = Vec::new();
    let total_nodes = args.masters + (args.masters * args.replicas);
    for i in 0..total_nodes {
        containers.push(format!("{}-node-{}", name, i));
    }
    if args.insight {
        containers.push(format!("{}-insight", name));
    }

    // Build ports list
    let mut ports = Vec::new();
    for i in 0..total_nodes {
        ports.push(args.port_base + i as u16);
    }

    // Build additional ports info
    let mut additional_ports = HashMap::new();
    if args.insight {
        additional_ports.insert("redisinsight".to_string(), args.insight_port);
    }

    // Store instance info
    let instance_info = InstanceInfo {
        name: name.clone(),
        instance_type: InstanceType::Cluster,
        created_at: chrono::Utc::now().to_rfc3339(),
        ports,
        containers,
        connection_info: ConnectionInfo {
            host: "localhost".to_string(),
            port: args.port_base, // Primary port
            password: Some(password.clone()),
            url: connection.cluster_url(),
            additional_ports,
        },
        metadata: {
            let mut map = HashMap::new();
            map.insert(
                "masters".to_string(),
                serde_json::Value::Number(args.masters.into()),
            );
            map.insert(
                "replicas".to_string(),
                serde_json::Value::Number(args.replicas.into()),
            );
            map.insert(
                "total_nodes".to_string(),
                serde_json::Value::Number(total_nodes.into()),
            );
            map.insert(
                "port_base".to_string(),
                serde_json::Value::Number(args.port_base.into()),
            );
            map.insert("persist".to_string(), serde_json::Value::Bool(args.persist));
            map.insert("stack".to_string(), serde_json::Value::Bool(args.stack));
            map.insert("insight".to_string(), serde_json::Value::Bool(args.insight));
            if let Some(memory) = args.memory {
                map.insert("memory".to_string(), serde_json::Value::String(memory));
            }
            map
        },
    };

    config.add_instance(instance_info);
    config.save()?;

    // Display connection info
    println!();
    println!("{} Redis Cluster started:", "Success:".bold().green());
    println!("  {}: {}", "Name".bold(), name.green());
    println!(
        "  {}: {} masters, {} replicas ({} total nodes)",
        "Topology".bold(),
        args.masters.to_string().green(),
        args.replicas.to_string().blue(),
        total_nodes.to_string().yellow()
    );
    println!(
        "  {}: localhost:{}-{}",
        "Ports".bold(),
        args.port_base.to_string().cyan(),
        (args.port_base + total_nodes as u16 - 1).to_string().cyan()
    );
    println!("  {}: {}", "Password".bold(), password.yellow());
    println!(
        "  {}: {}",
        "Cluster URL".bold(),
        connection.cluster_url().blue()
    );
    println!(
        "  {}: {}",
        "Nodes".bold(),
        connection.nodes_string().purple()
    );

    if args.persist {
        println!("  {}: {}-data-*", "Data Volumes".bold(), name.purple());
    }

    if args.stack {
        println!(
            "  {}: Redis Stack (JSON, Search, Graph, TimeSeries, Bloom)",
            "Modules".bold()
        );
    }

    if args.insight {
        println!(
            "  {}: http://localhost:{}",
            "RedisInsight".bold(),
            args.insight_port.to_string().magenta()
        );
    }

    // Connect to Redis cluster shell if requested (connect to first master node)
    if args.shell {
        println!();
        println!(
            "{} Connecting to redis-cli (cluster mode)...",
            "Shell:".bold().green()
        );
        println!();

        let status = ProcessCommand::new("redis-cli")
            .args([
                "-h",
                "localhost",
                "-p",
                &args.port_base.to_string(),
                "-a",
                &password,
                "-c", // Enable cluster mode
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

async fn stop_cluster(args: StopArgs, verbose: bool) -> Result<()> {
    let mut config = Config::load()?;

    // Get instance name
    let name = if let Some(name) = args.name {
        name
    } else {
        // Get the latest cluster instance
        if let Some(instance) = config.get_latest_instance(&InstanceType::Cluster) {
            instance.name.clone()
        } else {
            anyhow::bail!("No Redis Cluster instances found. Use --name to specify an instance.");
        }
    };

    // Check if instance exists
    let instance = config.get_instance(&name).context("Instance not found")?;

    if instance.instance_type != InstanceType::Cluster {
        anyhow::bail!("Instance '{}' is not a Redis Cluster instance", name);
    }

    if verbose {
        println!(
            "{} Stopping Redis Cluster: {}",
            "Stopping".cyan(),
            name.bold()
        );
    }

    // Create template to use its stop/remove methods
    let template = RedisClusterTemplate::new(&name); // Basic template for cleanup

    // Stop and remove the cluster
    template
        .stop()
        .await
        .with_context(|| format!("Failed to stop Redis Cluster: {}", name))?;

    template
        .remove()
        .await
        .with_context(|| format!("Failed to remove Redis Cluster: {}", name))?;

    // Remove from config
    config.remove_instance(&name);
    config.save()?;

    println!(
        "{} Redis Cluster '{}' stopped and removed",
        "Success:".green(),
        name.bold()
    );

    Ok(())
}

async fn info_cluster(args: InfoArgs, verbose: bool) -> Result<()> {
    let config = Config::load()?;

    // Get instance name
    let name = if let Some(name) = args.name {
        name
    } else {
        // Get the latest cluster instance
        if let Some(instance) = config.get_latest_instance(&InstanceType::Cluster) {
            instance.name.clone()
        } else {
            anyhow::bail!("No Redis Cluster instances found. Use --name to specify an instance.");
        }
    };

    // Get instance info
    let instance = config.get_instance(&name).context("Instance not found")?;

    if instance.instance_type != InstanceType::Cluster {
        anyhow::bail!("Instance '{}' is not a Redis Cluster instance", name);
    }

    // Display info based on format
    match args.format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(instance)?);
        }
        _ => {
            println!(
                "{} Redis Cluster: {}",
                "Info:".bold().cyan(),
                name.bold().green()
            );
            println!("  {}: {}", "Type".bold(), "Redis Cluster".yellow());
            println!("  {}: {}", "Created".bold(), instance.created_at.dimmed());

            // Extract topology info from metadata
            let masters = instance
                .metadata
                .get("masters")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let replicas = instance
                .metadata
                .get("replicas")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let total_nodes = instance
                .metadata
                .get("total_nodes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            println!(
                "  {}: {} masters, {} replicas ({} total)",
                "Topology".bold(),
                masters.to_string().green(),
                replicas.to_string().blue(),
                total_nodes.to_string().yellow()
            );

            println!(
                "  {}: {}",
                "Ports".bold(),
                instance
                    .ports
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
                    .cyan()
            );

            if let Some(password) = &instance.connection_info.password {
                println!("  {}: {}", "Password".bold(), password.yellow());
            }

            println!(
                "  {}: {}",
                "Cluster URL".bold(),
                instance.connection_info.url.blue()
            );
            println!(
                "  {}: {}",
                "Containers".bold(),
                instance.containers.join(", ").purple()
            );

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

            if instance
                .metadata
                .get("stack")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                println!("  {}: Redis Stack enabled", "Modules".bold());
            }

            if verbose {
                println!("  {}: {:?}", "All Metadata".bold(), instance.metadata);
            }
        }
    }

    Ok(())
}
