//! Redis Sentinel instance management

use anyhow::{Context, Result};
use colored::*;
use docker_wrapper::{DockerCommand, NetworkCreateCommand, RedisTemplate, Template};
use std::collections::HashMap;

use crate::cli::{InfoArgs, SentinelAction, SentinelStartArgs, StopArgs};
use crate::config::{generate_password, Config, ConnectionInfo, InstanceInfo, InstanceType};

pub async fn handle_action(action: SentinelAction, verbose: bool) -> Result<()> {
    match action {
        SentinelAction::Start(args) => start_sentinel(args, verbose).await,
        SentinelAction::Stop(args) => stop_sentinel(args, verbose).await,
        SentinelAction::Info(args) => info_sentinel(args, verbose).await,
    }
}

async fn start_sentinel(args: SentinelStartArgs, verbose: bool) -> Result<()> {
    let mut config = Config::load()?;

    // Generate name if not provided
    let name = args
        .name
        .unwrap_or_else(|| config.generate_name(&InstanceType::Sentinel));

    if verbose {
        println!(
            "{} Starting Redis Sentinel setup: {}",
            "Starting".cyan(),
            name.bold()
        );
    }

    // Generate password if not provided
    let password = args.password.unwrap_or_else(generate_password);

    // Create network for Sentinel setup
    let network_name = format!("{}-network", name);
    NetworkCreateCommand::new(&network_name)
        .driver("bridge")
        .execute()
        .await
        .context("Failed to create network for Sentinel setup")?;

    let mut container_ids = Vec::new();
    let mut ports_used = Vec::new();

    // Start Redis master(s)
    let masters = args.masters.max(1);
    for i in 0..masters {
        let master_name = format!("{}-master-{}", name, i + 1);
        let master_port = args.redis_port_base + i as u16;
        
        let mut master = RedisTemplate::new(&master_name)
            .port(master_port)
            .password(&password)
            .network(&network_name);

        if args.persist {
            master = master.with_persistence(format!("{}-data", master_name));
        }

        if let Some(ref memory) = args.memory {
            master = master.memory_limit(memory);
        }

        let container_id = master
            .start_and_wait()
            .await
            .context(format!("Failed to start Redis master {}", i + 1))?;

        container_ids.push(container_id);
        ports_used.push(master_port);

        if verbose {
            println!(
                "  {} Redis master {} on port {}",
                "Started".green(),
                i + 1,
                master_port
            );
        }
    }

    // Start Sentinel nodes
    let sentinels = args.sentinels.max(1);
    let mut sentinel_containers = Vec::new();
    
    for i in 0..sentinels {
        let sentinel_name = format!("{}-sentinel-{}", name, i + 1);
        let sentinel_port = args.sentinel_port_base + i as u16;

        // Create Sentinel configuration
        let mut sentinel_config = String::new();
        sentinel_config.push_str(&format!("port {}\n", sentinel_port));
        sentinel_config.push_str("sentinel announce-hostnames yes\n");
        sentinel_config.push_str("sentinel resolve-hostnames yes\n");
        
        // Monitor all masters
        for j in 0..masters {
            let master_name = format!("{}-master-{}", name, j + 1);
            let master_port = args.redis_port_base + j as u16;
            let quorum = (sentinels / 2) + 1; // Majority quorum
            
            sentinel_config.push_str(&format!(
                "sentinel monitor master-{} {} {} {}\n",
                j + 1,
                master_name,
                master_port,
                quorum
            ));
            
            if !password.is_empty() {
                sentinel_config.push_str(&format!(
                    "sentinel auth-pass master-{} {}\n",
                    j + 1,
                    password
                ));
            }
            
            sentinel_config.push_str(&format!(
                "sentinel down-after-milliseconds master-{} 5000\n",
                j + 1
            ));
            sentinel_config.push_str(&format!(
                "sentinel failover-timeout master-{} 10000\n", 
                j + 1
            ));
            sentinel_config.push_str(&format!(
                "sentinel parallel-syncs master-{} 1\n",
                j + 1
            ));
        }

        // Create a temporary config file
        let config_path = std::env::temp_dir().join(format!("{}.conf", sentinel_name));
        std::fs::write(&config_path, sentinel_config)
            .context("Failed to write Sentinel config")?;

        // Start Sentinel container
        use docker_wrapper::RunCommand;
        let sentinel_cmd = RunCommand::new("redis:7-alpine")
            .name(&sentinel_name)
            .network(&network_name)
            .port(sentinel_port, sentinel_port)
            .volume(config_path.to_str().unwrap(), "/etc/redis/sentinel.conf")
            .cmd(vec![
                "redis-sentinel".to_string(),
                "/etc/redis/sentinel.conf".to_string(),
            ])
            .detach();

        let container_id = sentinel_cmd
            .execute()
            .await
            .context(format!("Failed to start Sentinel {}", i + 1))?;

        sentinel_containers.push(container_id.0.clone());
        container_ids.push(container_id.0);
        ports_used.push(sentinel_port);

        if verbose {
            println!(
                "  {} Sentinel {} on port {}",
                "Started".green(),
                i + 1,
                sentinel_port
            );
        }

        // Give Sentinel time to start
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // Save instance information
    let mut metadata = HashMap::new();
    metadata.insert("masters".to_string(), serde_json::json!(masters));
    metadata.insert("sentinels".to_string(), serde_json::json!(sentinels));
    metadata.insert("network".to_string(), serde_json::json!(network_name));
    metadata.insert(
        "sentinel_containers".to_string(),
        serde_json::json!(sentinel_containers),
    );

    let instance = InstanceInfo {
        name: name.clone(),
        instance_type: InstanceType::Sentinel,
        created_at: chrono::Utc::now().to_rfc3339(),
        ports: ports_used,
        containers: container_ids,
        connection_info: ConnectionInfo {
            host: "localhost".to_string(),
            port: args.redis_port_base,
            password: Some(password.clone()),
            url: format!(
                "redis://:{}@localhost:{}",
                password, args.redis_port_base
            ),
            additional_ports: {
                let mut ports = HashMap::new();
                ports.insert("sentinel_base".to_string(), args.sentinel_port_base);
                ports
            },
        },
        metadata,
    };

    config.add_instance(instance);
    config.save()?;

    println!(
        "\n{} Redis Sentinel setup started successfully!",
        "Success:".green().bold()
    );
    println!("\n{}", "Connection Information:".bold().underline());
    println!("  {} redis://:{}@localhost:{}", "Master:".cyan(), password, args.redis_port_base);
    println!("  {} localhost:{}", "Sentinel:".cyan(), args.sentinel_port_base);
    println!("\n{}", "Components:".bold().underline());
    println!("  - {} Redis master(s)", masters);
    println!("  - {} Sentinel node(s)", sentinels);
    println!("\n{}", "Quick Commands:".bold().underline());
    println!(
        "  {} redis-cli -p {} -a {}",
        "Connect to master:".yellow(),
        args.redis_port_base,
        password
    );
    println!(
        "  {} redis-cli -p {} sentinel masters",
        "Check Sentinel:".yellow(),
        args.sentinel_port_base
    );
    println!(
        "  {} redis-dev sentinel stop {}",
        "Stop:".yellow(),
        name
    );

    Ok(())
}

async fn stop_sentinel(args: StopArgs, verbose: bool) -> Result<()> {
    let mut config = Config::load()?;

    // Find the instance
    let name = args
        .name
        .or_else(|| config.get_latest_instance(&InstanceType::Sentinel).map(|i| i.name.clone()));

    let name = name.context("No Sentinel instance found. Specify a name or start one first.")?;

    let instance = config
        .instances
        .get(&name)
        .context(format!("Sentinel instance '{}' not found", name))?
        .clone();

    if verbose {
        println!(
            "{} Stopping Sentinel setup: {}",
            "Stopping".yellow(),
            name.bold()
        );
    }

    // Stop all containers
    use docker_wrapper::{RmCommand, StopCommand};
    for container_id in &instance.containers {
        // Extract container name from ID (if needed)
        let container_name = container_id.split(':').next().unwrap_or(container_id);
        
        StopCommand::new(container_name)
            .execute()
            .await
            .ok(); // Ignore errors for already stopped containers
        
        RmCommand::new(container_name)
            .force()
            .execute()
            .await
            .ok();
    }

    // Remove network
    if let Some(network) = instance.metadata.get("network") {
        if let Some(network_name) = network.as_str() {
            use docker_wrapper::NetworkRmCommand;
            NetworkRmCommand::new(network_name)
                .execute()
                .await
                .ok();
        }
    }

    // Remove from config
    config.instances.remove(&name);
    config.save()?;

    println!(
        "{} Sentinel setup '{}' stopped and removed",
        "Success:".green().bold(),
        name
    );

    Ok(())
}

async fn info_sentinel(args: InfoArgs, verbose: bool) -> Result<()> {
    let config = Config::load()?;

    // Find the instance
    let name = args
        .name
        .or_else(|| config.get_latest_instance(&InstanceType::Sentinel).map(|i| i.name.clone()));

    let name = name.context("No Sentinel instance found. Specify a name or start one first.")?;

    let instance = config
        .instances
        .get(&name)
        .context(format!("Sentinel instance '{}' not found", name))?;

    println!("{}", "Redis Sentinel Information".bold().underline());
    println!("{} {}", "Name:".cyan(), instance.name);
    println!("{} {}", "Created:".cyan(), instance.created_at);
    println!(
        "{} {} masters, {} sentinels",
        "Configuration:".cyan(),
        instance.metadata.get("masters").and_then(|v| v.as_u64()).unwrap_or(0),
        instance.metadata.get("sentinels").and_then(|v| v.as_u64()).unwrap_or(0)
    );
    println!("{} {}", "Network:".cyan(), 
        instance.metadata.get("network").and_then(|v| v.as_str()).unwrap_or("unknown")
    );
    
    println!("\n{}", "Ports:".bold().underline());
    for port in &instance.ports {
        println!("  - {}", port);
    }

    println!("\n{}", "Connection:".bold().underline());
    println!("  {} {}", "Master URL:".cyan(), instance.connection_info.url);
    if let Some(sentinel_port) = instance.connection_info.additional_ports.get("sentinel_base") {
        println!("  {} localhost:{}", "Sentinel:".cyan(), sentinel_port);
    }

    if verbose {
        println!("\n{}", "Containers:".bold().underline());
        for container in &instance.containers {
            println!("  - {}", container);
        }
    }

    // Check Sentinel status
    if let Some(sentinel_containers) = instance.metadata.get("sentinel_containers") {
        if let Some(containers) = sentinel_containers.as_array() {
            if !containers.is_empty() {
                if let Some(first_sentinel) = containers.first().and_then(|v| v.as_str()) {
                    use docker_wrapper::{DockerCommand, ExecCommand};
                    let status = ExecCommand::new(
                        first_sentinel,
                        vec!["redis-cli".to_string(), "-p".to_string(), "26379".to_string(), "sentinel".to_string(), "masters".to_string()]
                    ).execute().await;
                    
                    if let Ok(result) = status {
                        if !result.stdout.is_empty() {
                            println!("\n{}", "Sentinel Status:".bold().underline());
                            // Parse and display key information
                            let lines: Vec<&str> = result.stdout.lines().collect();
                            for (i, line) in lines.iter().enumerate() {
                                if line.contains("name") {
                                    if let Some(name_line) = lines.get(i + 1) {
                                        println!("  Master: {}", name_line.trim());
                                    }
                                }
                                if line.contains("num-slaves") {
                                    if let Some(slaves_line) = lines.get(i + 1) {
                                        println!("  Replicas: {}", slaves_line.trim());
                                    }
                                }
                                if line.contains("num-other-sentinels") {
                                    if let Some(sentinels_line) = lines.get(i + 1) {
                                        println!("  Other Sentinels: {}", sentinels_line.trim());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}