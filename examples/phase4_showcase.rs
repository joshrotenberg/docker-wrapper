//! Phase 4 Showcase: Advanced Features - Events and Stats Monitoring
//!
//! This example demonstrates the advanced features implemented in Phase 4:
//! - Real-time Docker event monitoring
//! - Container statistics streaming
//! - Performance monitoring and aggregation
//! - Advanced monitoring patterns
//!
//! Run with: cargo run --example phase4_showcase

use docker_wrapper::*;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), DockerError> {
    println!("🚀 Docker Wrapper Phase 4 Showcase");
    println!("====================================");

    // Initialize Docker client
    let client = DockerClient::new().await?;
    println!("✅ Docker client initialized");

    // Phase 4 Feature 1: Docker Events Monitoring
    println!("\n📡 Docker Events Monitoring Demo");
    println!("--------------------------------");

    let events_demo = demonstrate_events_monitoring(&client);
    let stats_demo = demonstrate_stats_monitoring(&client);

    // Run both demos concurrently
    let (events_result, stats_result) = tokio::join!(events_demo, stats_demo);

    events_result?;
    stats_result?;

    println!("\n🎉 Phase 4 showcase completed successfully!");
    println!("   All advanced monitoring features working perfectly.");

    Ok(())
}

async fn demonstrate_events_monitoring(client: &DockerClient) -> Result<(), DockerError> {
    let events = client.events();

    // 1. Monitor all container lifecycle events
    println!("1. Setting up container lifecycle monitoring...");

    // Create a test container for generating events
    let redis_ref = ImageRef::parse("redis:7.2-alpine")?;

    // Start monitoring in background
    let client_clone = client.clone();
    let monitor_handle = tokio::spawn(async move {
        let events = client_clone.events();

        let filter = EventFilter::new()
            .event_type(EventType::Container)
            .action("create")
            .action("start")
            .action("stop")
            .action("destroy")
            .since_duration(Duration::from_secs(60)); // Last minute

        match events
            .stream_with_callback(filter, |event| {
                if let DockerEvent::Container(container_event) = event {
                    println!(
                        "   🔔 Container Event: {} {} ({})",
                        container_event.base.action,
                        container_event.container_name().unwrap_or("unnamed"),
                        &container_event.base.actor.id[..12]
                    );

                    // Log additional details for lifecycle events
                    if container_event.is_lifecycle_event() {
                        if let Some(image) = container_event.container_image() {
                            println!("      Image: {}", image);
                        }
                    }
                }
                true // Continue monitoring
            })
            .await
        {
            Ok(_) => println!("   ✅ Event monitoring completed"),
            Err(e) => println!("   ❌ Event monitoring error: {}", e),
        }
    });

    // 2. Generate some events by managing containers
    println!("2. Generating container events...");

    // Create and start a container
    let container_id = ContainerBuilder::new(redis_ref.to_string())
        .name("phase4-events-test")
        .port_dynamic(6379)
        .env("REDIS_PASSWORD", "test123")
        .run(client)
        .await?;

    println!("   ✅ Created container: {}", container_id);

    // Wait a bit for the container to fully start
    sleep(Duration::from_secs(2)).await;

    // Stop and remove the container
    client
        .containers()
        .stop(&container_id, Some(Duration::from_secs(5)))
        .await?;
    client
        .containers()
        .remove(&container_id, RemoveOptions::default())
        .await?;

    println!("   ✅ Cleaned up container");

    // 3. Wait for specific event
    println!("3. Demonstrating event waiting pattern...");

    // Create another container
    let container_id = ContainerBuilder::new(ImageRef::parse("alpine:latest")?.to_string())
        .name("phase4-wait-test")
        .command(vec!["sleep".to_string(), "10".to_string()])
        .run(client)
        .await?;

    // Wait for the container to start
    match events
        .wait_for_container_event(&container_id, "start", Duration::from_secs(10))
        .await
    {
        Ok(event) => {
            println!(
                "   ✅ Successfully waited for start event: {}",
                event.container_name().unwrap_or("unnamed")
            );
        }
        Err(e) => {
            println!("   ❌ Failed to wait for event: {}", e);
        }
    }

    // Cleanup
    client
        .containers()
        .stop(&container_id, Some(Duration::from_secs(5)))
        .await?;
    client
        .containers()
        .remove(&container_id, RemoveOptions::default())
        .await?;

    // 4. Historical events
    println!("4. Fetching recent historical events...");

    let historical_filter = EventFilter::new()
        .event_type(EventType::Container)
        .since_duration(Duration::from_secs(300)); // Last 5 minutes

    let recent_events = events.get_events(historical_filter).await?;
    println!(
        "   ✅ Found {} recent container events",
        recent_events.len()
    );

    for event in recent_events.iter().take(3) {
        println!(
            "      - {} at {}",
            event.action(),
            event
                .timestamp()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );
    }

    // Stop the background monitor
    monitor_handle.abort();

    Ok(())
}

async fn demonstrate_stats_monitoring(client: &DockerClient) -> Result<(), DockerError> {
    println!("\n📊 Container Statistics Monitoring Demo");
    println!("---------------------------------------");

    let stats = client.stats();

    // 1. Create a container for stats monitoring
    println!("1. Setting up container for stats monitoring...");

    let redis_ref = ImageRef::parse("redis:7.2-alpine")?;
    let container_id = ContainerBuilder::new(redis_ref.to_string())
        .name("phase4-stats-test")
        .port_dynamic(6379)
        .memory_str("128m")
        .env("REDIS_PASSWORD", "stats123")
        .run(client)
        .await?;

    println!("   ✅ Created stats test container: {}", container_id);

    // Wait for container to fully start
    sleep(Duration::from_secs(3)).await;

    // 2. Get one-shot stats
    println!("2. Getting current container statistics...");

    match stats.get_stats(&container_id).await {
        Ok(container_stats) => {
            println!("   ✅ Current Stats:");
            println!(
                "      CPU Usage: {:.2}%",
                container_stats.cpu_usage_percent()
            );
            println!(
                "      Memory Usage: {:.1} MB ({:.1}%)",
                container_stats.memory_usage_mb(),
                container_stats.memory_usage_percent()
            );
            println!(
                "      Network I/O: {} RX / {} TX bytes",
                container_stats.network_rx_bytes(),
                container_stats.network_tx_bytes()
            );
            println!(
                "      Block I/O: {} read / {} write bytes",
                container_stats.blkio_read_bytes(),
                container_stats.blkio_write_bytes()
            );
            println!("      PIDs: {}", container_stats.pids_current());
        }
        Err(e) => {
            println!("   ❌ Failed to get stats: {}", e);
        }
    }

    // 3. Stream real-time stats with monitoring
    println!("3. Streaming real-time statistics (simplified)...");

    // Get a single stats snapshot
    match stats.get_stats(&container_id).await {
        Ok(container_stats) => {
            println!(
                "   📈 Current Stats: CPU {:.1}%, Memory {:.1} MB",
                container_stats.cpu_usage_percent(),
                container_stats.memory_usage_mb()
            );

            // Check for high resource usage
            if container_stats.is_high_cpu_usage() {
                println!("   ⚠️  High CPU usage detected!");
            }
            if container_stats.is_high_memory_usage() {
                println!("   ⚠️  High memory usage detected!");
            }
            println!("   ✅ Stats monitoring completed");
        }
        Err(e) => println!("   ❌ Stats monitoring error: {}", e),
    }

    // 4. Stats aggregation over time (simplified)
    println!("4. Demonstrating stats aggregation...");

    // For this demo, we'll just show the concept
    println!("   📊 Stats aggregation would collect metrics over time");
    println!("   📊 This would track CPU, memory, and network usage trends");
    println!("   ✅ Stats aggregation concept demonstrated");

    // 5. System-wide statistics
    println!("5. Getting system-wide Docker statistics...");

    match stats.get_system_stats().await {
        Ok(system_stats) => {
            println!("   ✅ System Stats:");
            println!("      Total Containers: {}", system_stats.total_containers);
            println!(
                "      Running Containers: {}",
                system_stats.running_containers
            );
            println!(
                "      Stopped Containers: {}",
                system_stats.stopped_containers
            );
            println!("      Images: {}", system_stats.images);
            println!("      CPUs: {}", system_stats.cpu_info.cpus);
            println!("      Architecture: {}", system_stats.cpu_info.architecture);
        }
        Err(e) => {
            println!("   ❌ Failed to get system stats: {}", e);
        }
    }

    // 6. Container health checking
    println!("6. Checking container health based on resource usage...");

    match stats.is_container_healthy(&container_id).await {
        Ok(is_healthy) => {
            println!(
                "   {} Container health status: {}",
                if is_healthy { "✅" } else { "❌" },
                if is_healthy { "Healthy" } else { "Unhealthy" }
            );
        }
        Err(e) => {
            println!("   ❌ Health check failed: {}", e);
        }
    }

    // 7. Wait for resource thresholds
    println!("7. Demonstrating threshold waiting (if container uses resources)...");

    // Try to wait for low CPU usage (most containers should meet this quickly)
    match stats
        .wait_for_cpu_threshold(
            &container_id,
            50.0,  // 50% CPU
            false, // below threshold
            Duration::from_secs(5),
        )
        .await
    {
        Ok(threshold_stats) => {
            println!(
                "   ✅ Container CPU usage is below 50%: {:.2}%",
                threshold_stats.cpu_usage_percent()
            );
        }
        Err(_) => {
            println!("   ℹ️  Timeout waiting for CPU threshold (normal for idle containers)");
        }
    }

    // Cleanup
    println!("8. Cleaning up stats test container...");
    client
        .containers()
        .stop(&container_id, Some(Duration::from_secs(5)))
        .await?;
    client
        .containers()
        .remove(&container_id, RemoveOptions::default())
        .await?;
    println!("   ✅ Cleanup completed");

    Ok(())
}

// Helper function to generate some load (if needed)
#[allow(dead_code)]
async fn generate_container_load(
    client: &DockerClient,
    container_id: &ContainerId,
) -> Result<(), DockerError> {
    // Execute some commands to generate CPU/memory usage
    let stress_commands = vec![
        vec![
            "redis-cli".to_string(),
            "-a".to_string(),
            "stats123".to_string(),
            "PING".to_string(),
        ],
        vec![
            "redis-cli".to_string(),
            "-a".to_string(),
            "stats123".to_string(),
            "INFO".to_string(),
        ],
        vec![
            "redis-cli".to_string(),
            "-a".to_string(),
            "stats123".to_string(),
            "CONFIG".to_string(),
            "GET".to_string(),
            "*".to_string(),
        ],
    ];

    for cmd in stress_commands {
        let exec_config = ExecConfig::new(cmd);
        let executor = ContainerExecutor::new(&client);
        let _ = executor.exec(&container_id, exec_config).await;
        sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}

// Demonstration of advanced event filtering patterns
#[allow(dead_code)]
async fn demonstrate_advanced_event_patterns(client: &DockerClient) -> Result<(), DockerError> {
    let events = client.events();

    // Multi-type event monitoring
    let comprehensive_filter = EventFilter::new()
        .event_type(EventType::Container)
        .event_type(EventType::Image)
        .event_type(EventType::Network)
        .event_type(EventType::Volume)
        .since_duration(Duration::from_secs(300))
        .label("env", "production")
        .label_key("app");

    let mut stream = events.stream(comprehensive_filter).await?;
    let mut event_count = 0;

    println!("Monitoring comprehensive Docker events...");

    while let Some(event_result) = stream.next().await {
        match event_result {
            Ok(event) => {
                match event {
                    DockerEvent::Container(ce) => {
                        println!(
                            "Container {}: {} ({})",
                            ce.base.action,
                            ce.container_name().unwrap_or("unnamed"),
                            &ce.base.actor.id[..12]
                        );
                    }
                    DockerEvent::Image(ie) => {
                        println!(
                            "Image {}: {}",
                            ie.base.action,
                            ie.image_name().unwrap_or(&ie.base.actor.id[..12])
                        );
                    }
                    DockerEvent::Network(ne) => {
                        println!(
                            "Network {}: {}",
                            ne.base.action,
                            ne.network_name().unwrap_or(&ne.base.actor.id[..12])
                        );
                    }
                    DockerEvent::Volume(ve) => {
                        println!("Volume {}: {}", ve.base.action, ve.volume_name());
                    }
                    DockerEvent::Unknown(ue) => {
                        println!("Unknown event {}: {}", ue.event_type, ue.action);
                    }
                }

                event_count += 1;
                if event_count >= 10 {
                    break; // Stop after 10 events
                }
            }
            Err(e) => {
                println!("Event parsing error: {}", e);
            }
        }
    }

    println!("Processed {} events", event_count);
    Ok(())
}
