//! Redis Enterprise instance management

use anyhow::{Context, Result};
use colored::*;
use docker_wrapper::{DockerCommand, RedisEnterpriseTemplate};
use std::collections::HashMap;

use crate::cli::{EnterpriseAction, EnterpriseStartArgs, InfoArgs, StopArgs};
use crate::config::{Config, ConnectionInfo, InstanceInfo, InstanceType};

pub async fn handle_action(action: EnterpriseAction, verbose: bool) -> Result<()> {
    match action {
        EnterpriseAction::Start(args) => start_enterprise(args, verbose).await,
        EnterpriseAction::Stop(args) => stop_enterprise(args, verbose).await,
        EnterpriseAction::Info(args) => info_enterprise(args, verbose).await,
    }
}

async fn start_enterprise(args: EnterpriseStartArgs, verbose: bool) -> Result<()> {
    let mut config = Config::load()?;

    // Generate name if not provided
    let name = args
        .name
        .unwrap_or_else(|| config.generate_name(&InstanceType::Enterprise));

    if verbose {
        println!(
            "{} Starting Redis Enterprise cluster: {}",
            "Starting".cyan(),
            name.bold()
        );
    }

    // Note for multi-node support: In a full implementation, we would need to:
    // 1. Create a Docker network for the nodes to communicate
    // 2. Start multiple containers with proper networking
    // 3. Form a cluster using the REST API between nodes
    // For now, we'll start with a single-node development cluster

    if args.nodes > 1 {
        println!(
            "{} Multi-node clusters require additional implementation. Starting single-node cluster.",
            "Note:".yellow()
        );
    }

    // Create Redis Enterprise template
    let mut enterprise = RedisEnterpriseTemplate::new(&name)
        .cluster_name(format!("{}-cluster", name))
        .accept_eula()
        .ui_port(args.port_base)
        .api_port(args.port_base + 1000) // API port is typically 1000 higher
        .database_port_start(args.db_port);

    // Set memory limit if specified
    if let Some(ref memory) = args.memory {
        enterprise = enterprise.memory_limit(memory);
    }

    // Set persistence volumes
    if args.persist {
        enterprise = enterprise
            .persistent_path(format!("{}-persistent", name))
            .ephemeral_path(format!("{}-ephemeral", name));
    }

    // Add initial database if requested
    if let Some(ref db_name) = args.create_db {
        enterprise = enterprise.with_database(db_name);
    }

    // Start the Enterprise cluster (unless containers-only mode)
    let connection_info = if args.containers_only {
        println!(
            "{} Starting in containers-only mode. Cluster formation skipped.",
            "Note:".yellow()
        );

        // Just start the container without bootstrapping
        use docker_wrapper::RunCommand;
        let container_name = format!("{}-enterprise", name);
        let mut cmd = RunCommand::new("redislabs/redis:latest")
            .name(&container_name)
            .port(args.port_base, 8443)
            .port(args.port_base + 1000, 9443)
            .detach()
            .cap_add("SYS_RESOURCE");

        // Add database ports
        for i in 0..10 {
            let port = args.db_port + i;
            cmd = cmd.port(port, port);
        }

        // Add volumes
        if args.persist {
            cmd = cmd
                .volume(format!("{}-persistent", name), "/var/opt/redislabs/persist")
                .volume(format!("{}-ephemeral", name), "/var/opt/redislabs/tmp");
        }

        // Add memory limit
        if let Some(ref memory) = args.memory {
            cmd = cmd.memory(memory);
        }

        let container_id = cmd
            .execute()
            .await
            .context("Failed to start Enterprise container")?;

        println!(
            "\n{} Redis Enterprise container started in manual mode.",
            "Info:".cyan()
        );
        println!(
            "  Access the UI at https://localhost:{} to complete setup",
            args.port_base
        );
        println!("  Container ID: {}", container_id.0);

        // Return basic connection info
        docker_wrapper::RedisEnterpriseConnectionInfo {
            name: name.clone(),
            container_name,
            cluster_name: format!("{}-cluster", name),
            ui_url: format!("https://localhost:{}", args.port_base),
            api_url: format!("https://localhost:{}", args.port_base + 1000),
            username: "admin@redis.local".to_string(),
            password: "<set during UI setup>".to_string(),
            database_port: None,
        }
    } else {
        // Full automatic cluster formation
        let conn_info = enterprise
            .start()
            .await
            .context("Failed to start Redis Enterprise cluster")?;

        if verbose {
            println!(
                "  {} Enterprise cluster bootstrapped successfully",
                "Success".green()
            );
            if args.create_db.is_some() {
                println!(
                    "  {} Database '{}' created on port {}",
                    "Database".green(),
                    args.create_db.as_ref().unwrap(),
                    args.db_port
                );
            }
        }

        conn_info
    };

    // Save instance information
    let mut metadata = HashMap::new();
    metadata.insert("nodes".to_string(), serde_json::json!(1));
    metadata.insert("ui_port".to_string(), serde_json::json!(args.port_base));
    metadata.insert(
        "api_port".to_string(),
        serde_json::json!(args.port_base + 1000),
    );
    metadata.insert(
        "cluster_name".to_string(),
        serde_json::json!(connection_info.cluster_name.clone()),
    );
    metadata.insert(
        "container_name".to_string(),
        serde_json::json!(connection_info.container_name.clone()),
    );
    if let Some(db_port) = connection_info.database_port {
        metadata.insert("database_port".to_string(), serde_json::json!(db_port));
    }
    if let Some(ref db_name) = args.create_db {
        metadata.insert("database_name".to_string(), serde_json::json!(db_name));
    }

    let instance = InstanceInfo {
        name: name.clone(),
        instance_type: InstanceType::Enterprise,
        created_at: chrono::Utc::now().to_rfc3339(),
        ports: vec![args.port_base, args.port_base + 1000, args.db_port],
        containers: vec![connection_info.container_name.clone()],
        connection_info: ConnectionInfo {
            host: "localhost".to_string(),
            port: args.db_port,
            password: Some(connection_info.password.clone()),
            url: if connection_info.database_port.is_some() {
                format!("redis://localhost:{}", args.db_port)
            } else {
                "<pending database creation>".to_string()
            },
            additional_ports: {
                let mut ports = HashMap::new();
                ports.insert("ui".to_string(), args.port_base);
                ports.insert("api".to_string(), args.port_base + 1000);
                ports
            },
        },
        metadata,
    };

    config.add_instance(instance);
    config.save()?;

    // Display success message
    println!(
        "\n{} Redis Enterprise cluster started successfully!",
        "Success:".green().bold()
    );
    println!("\n{}", "Connection Information:".bold().underline());
    println!("  {} {}", "UI:".cyan(), connection_info.ui_url);
    println!("  {} {}", "API:".cyan(), connection_info.api_url);
    println!("  {} {}", "Username:".cyan(), connection_info.username);
    println!("  {} {}", "Password:".cyan(), connection_info.password);

    if let Some(db_port) = connection_info.database_port {
        println!("\n{}", "Database:".bold().underline());
        println!(
            "  {} redis-cli -p {} -a {}",
            "Connect:".yellow(),
            db_port,
            "<password>"
        );
    }

    println!("\n{}", "Quick Commands:".bold().underline());
    println!(
        "  {} Open https://localhost:{} in your browser",
        "Access UI:".yellow(),
        args.port_base
    );
    println!("  {} redis-dev enterprise stop {}", "Stop:".yellow(), name);
    println!("  {} redis-dev enterprise info {}", "Info:".yellow(), name);

    Ok(())
}

async fn stop_enterprise(args: StopArgs, verbose: bool) -> Result<()> {
    let mut config = Config::load()?;

    // Find the instance
    let name = args.name.or_else(|| {
        config
            .get_latest_instance(&InstanceType::Enterprise)
            .map(|i| i.name.clone())
    });

    let name = name.context("No Enterprise instance found. Specify a name or start one first.")?;

    let instance = config
        .instances
        .get(&name)
        .context(format!("Enterprise instance '{}' not found", name))?
        .clone();

    if verbose {
        println!(
            "{} Stopping Enterprise cluster: {}",
            "Stopping".yellow(),
            name.bold()
        );
    }

    // Stop and remove containers
    use docker_wrapper::{RmCommand, StopCommand};
    for container in &instance.containers {
        StopCommand::new(container).execute().await.ok(); // Ignore errors for already stopped containers

        RmCommand::new(container).force().execute().await.ok();
    }

    // Remove volumes if they exist
    use docker_wrapper::VolumeRmCommand;
    let persistent_volume = format!("{}-persistent", name);
    let ephemeral_volume = format!("{}-ephemeral", name);

    VolumeRmCommand::new(&persistent_volume)
        .force()
        .execute()
        .await
        .ok();

    VolumeRmCommand::new(&ephemeral_volume)
        .force()
        .execute()
        .await
        .ok();

    // Remove from config
    config.instances.remove(&name);
    config.save()?;

    println!(
        "{} Enterprise cluster '{}' stopped and removed",
        "Success:".green().bold(),
        name
    );

    Ok(())
}

async fn info_enterprise(args: InfoArgs, verbose: bool) -> Result<()> {
    let config = Config::load()?;

    // Find the instance
    let name = args.name.or_else(|| {
        config
            .get_latest_instance(&InstanceType::Enterprise)
            .map(|i| i.name.clone())
    });

    let name = name.context("No Enterprise instance found. Specify a name or start one first.")?;

    let instance = config
        .instances
        .get(&name)
        .context(format!("Enterprise instance '{}' not found", name))?;

    println!("{}", "Redis Enterprise Information".bold().underline());
    println!("{} {}", "Name:".cyan(), instance.name);
    println!("{} {}", "Created:".cyan(), instance.created_at);
    println!(
        "{} {}",
        "Cluster Name:".cyan(),
        instance
            .metadata
            .get("cluster_name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
    );
    println!(
        "{} {} node(s)",
        "Nodes:".cyan(),
        instance
            .metadata
            .get("nodes")
            .and_then(|v| v.as_u64())
            .unwrap_or(1)
    );

    println!("\n{}", "Access Points:".bold().underline());
    if let Some(ui_port) = instance.connection_info.additional_ports.get("ui") {
        println!("  {} https://localhost:{}", "UI:".cyan(), ui_port);
    }
    if let Some(api_port) = instance.connection_info.additional_ports.get("api") {
        println!("  {} https://localhost:{}", "API:".cyan(), api_port);
    }

    if let Some(db_name) = instance.metadata.get("database_name") {
        println!("\n{}", "Database:".bold().underline());
        println!(
            "  {} {}",
            "Name:".cyan(),
            db_name.as_str().unwrap_or("unknown")
        );
        if let Some(db_port) = instance.metadata.get("database_port") {
            println!("  {} {}", "Port:".cyan(), db_port.as_u64().unwrap_or(0));
        }
    }

    if verbose {
        println!("\n{}", "Containers:".bold().underline());
        for container in &instance.containers {
            println!("  - {}", container);
        }

        // Check if container is running
        use docker_wrapper::PsCommand;
        if let Some(container_name) = instance.containers.first() {
            let ps_result = PsCommand::new()
                .filter(format!("name={}", container_name))
                .quiet()
                .execute()
                .await;

            if let Ok(output) = ps_result {
                if !output.stdout.trim().is_empty() {
                    println!("\n{} Container is running", "Status:".green());
                } else {
                    println!("\n{} Container is stopped", "Status:".red());
                }
            }
        }
    }

    Ok(())
}
