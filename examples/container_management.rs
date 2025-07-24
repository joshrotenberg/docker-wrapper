//! Container Lifecycle Management Demo
//!
//! This example demonstrates the comprehensive container lifecycle management
//! capabilities, including:
//!
//! - Advanced container configuration with ContainerBuilder
//! - Container execution with streaming I/O
//! - Health checking and readiness waiting
//! - Log streaming and monitoring
//! - Container cleanup

use docker_wrapper::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for better debugging
    tracing_subscriber::fmt::init();

    println!("🚀 Container Lifecycle Management Demo");
    println!("======================================");

    // Create Docker client
    let client = DockerClient::new().await?;
    println!("✅ Connected to Docker daemon");

    // Feature 1: Advanced Container Configuration
    println!("\n📦 Creating Redis container with advanced configuration...");

    let container_id = ContainerBuilder::new("redis:7.2-alpine")
        .name("container-lifecycle-demo")
        .env("REDIS_PASSWORD", "supersecret")
        .port_dynamic(6379) // Dynamic port allocation
        .memory_str("256m") // Human-readable memory limit
        .auto_remove() // Cleanup automatically
        .label("demo", "container-lifecycle")
        .label("component", "redis")
        .command(vec![
            "redis-server".to_string(),
            "--requirepass".to_string(),
            "supersecret".to_string(),
            "--appendonly".to_string(),
            "yes".to_string(),
        ])
        .run(&client)
        .await?;

    println!("✅ Container created with ID: {}", container_id);

    // Feature 2: Health Checking and Readiness
    println!("\n🏥 Waiting for Redis to be ready...");

    let health_checker = HealthChecker::new(&client);

    // Wait for the port to be accessible
    health_checker
        .wait_for_port(&container_id, 6379, Duration::from_secs(30))
        .await?;

    println!("✅ Redis port is accessible");

    // Get the mapped host port
    let container_manager = ContainerManager::new(&client);
    let host_port = container_manager
        .port(&container_id, 6379)
        .await?
        .expect("Port should be mapped");

    println!("📍 Redis is accessible on host port: {}", host_port);

    // Feature 3: Container Execution
    println!("\n⚡ Testing Redis with container execution...");

    let executor = ContainerExecutor::new(&client);

    // Test Redis connectivity
    let ping_result = executor
        .exec_simple(
            &container_id,
            vec![
                "redis-cli".to_string(),
                "-a".to_string(),
                "supersecret".to_string(),
                "ping".to_string(),
            ],
        )
        .await?;

    println!("📤 Redis PING result: {}", ping_result.trim());

    // Set a test value
    let _set_result = executor
        .exec_simple(
            &container_id,
            vec![
                "redis-cli".to_string(),
                "-a".to_string(),
                "supersecret".to_string(),
                "set".to_string(),
                "demo:key".to_string(),
                "ContainerSuccess".to_string(),
            ],
        )
        .await?;

    println!("✅ Set test key in Redis");

    // Get the test value
    let get_result = executor
        .exec_simple(
            &container_id,
            vec![
                "redis-cli".to_string(),
                "-a".to_string(),
                "supersecret".to_string(),
                "get".to_string(),
                "demo:key".to_string(),
            ],
        )
        .await?;

    println!("📥 Retrieved value: {}", get_result.trim());

    // Feature 4: Log Management
    println!("\n📜 Demonstrating log management...");

    let log_manager = LogManager::new(&client);

    // Get recent logs
    let recent_logs = log_manager.get_recent_logs(&container_id, 10).await?;

    println!("📋 Recent log entries ({} total):", recent_logs.len());
    for (i, entry) in recent_logs.iter().take(5).enumerate() {
        println!(
            "  {}. [{}] {}",
            i + 1,
            entry.source,
            entry.message.chars().take(80).collect::<String>()
        );
    }

    // Feature 5: Advanced Health Checks
    println!("\n🔍 Testing composite health checks...");

    // Create a composite health check
    let composite_check = HealthCheck::all(vec![
        HealthCheck::port(6379),
        HealthCheck::command(vec![
            "redis-cli".to_string(),
            "-a".to_string(),
            "supersecret".to_string(),
            "ping".to_string(),
        ]),
    ]);

    let health_result = health_checker
        .check_health(&container_id, composite_check)
        .await?;

    if health_result.healthy {
        println!(
            "✅ Composite health check passed in {:?}",
            health_result.duration
        );
    } else {
        println!(
            "❌ Composite health check failed: {}",
            health_result.message
        );
    }

    // Feature 6: Streaming Execution
    println!("\n🌊 Demonstrating streaming execution...");

    // Run a command that produces streaming output
    let mut output_lines = Vec::new();

    let stream_result = executor.exec_streaming(
        &container_id,
        ExecConfig::from_command_str("redis-cli -a supersecret monitor").tty(),
        |output| {
            match output {
                ExecOutput::Stdout(line) => {
                    println!("📤 Redis Monitor: {}", line);
                    output_lines.push(line);
                }
                ExecOutput::Stderr(line) => {
                    println!("📥 Redis Error: {}", line);
                }
            }

            // Stop after collecting a few lines
            if output_lines.len() >= 3 {
                Err(DockerError::cancelled("Demo complete".to_string()))
            } else {
                Ok(())
            }
        },
    );

    // Run the streaming command with a timeout
    let timeout_result = tokio::time::timeout(Duration::from_secs(5), stream_result).await;

    match timeout_result {
        Ok(Err(e)) if e.to_string().contains("Demo complete") => {
            println!("✅ Streaming execution demo completed");
        }
        Ok(Ok(_)) => {
            println!("✅ Streaming execution completed normally");
        }
        Err(_) => {
            println!("⏰ Streaming execution timed out (expected)");
        }
        Ok(Err(e)) => {
            println!("⚠️  Streaming execution error: {}", e);
        }
    }

    // Generate some Redis activity for log streaming demo
    println!("\n📊 Generating Redis activity for log demo...");
    for i in 1..=5 {
        let _ = executor
            .exec_simple(
                &container_id,
                vec![
                    "redis-cli".to_string(),
                    "-a".to_string(),
                    "supersecret".to_string(),
                    "set".to_string(),
                    format!("activity:{}", i),
                    format!("value_{}", i),
                ],
            )
            .await;

        sleep(Duration::from_millis(100)).await;
    }

    // Feature 7: Log Streaming
    println!("\n📺 Demonstrating log streaming...");

    let mut log_count = 0;
    let log_stream_result = log_manager.stream_logs(
        &container_id,
        LogOptions::new().follow().timestamps().tail(5),
        |entry| {
            log_count += 1;
            println!(
                "📜 [{}] {}: {}",
                log_count,
                entry.source,
                entry.message.chars().take(100).collect::<String>()
            );

            // Stop after collecting some logs
            if log_count >= 10 {
                Err(DockerError::cancelled("Log demo complete".to_string()))
            } else {
                Ok(())
            }
        },
    );

    // Run log streaming with timeout
    let log_timeout_result = tokio::time::timeout(Duration::from_secs(3), log_stream_result).await;

    match log_timeout_result {
        Ok(Err(e)) if e.to_string().contains("Log demo complete") => {
            println!("✅ Log streaming demo completed");
        }
        Err(_) => {
            println!("⏰ Log streaming timed out (expected)");
        }
        _ => {
            println!("✅ Log streaming completed");
        }
    }

    // Feature 8: Container Inspection
    println!("\n🔍 Container inspection...");

    let container_info = container_manager.inspect(&container_id).await?;
    println!("📋 Container Details:");
    println!("  • Name: {:?}", container_info.name);
    println!("  • Status: {}", container_info.status);
    println!("  • Image: {}", container_info.image);
    println!("  • Labels: {} defined", container_info.labels.len());
    println!("  • Networks: {:?}", container_info.networks);
    println!("  • Ports: {} mapped", container_info.ports.len());

    // Feature 9: Resource Monitoring
    println!("\n📈 Demonstrating resource monitoring...");

    // This would be expanded with actual resource monitoring in a real implementation
    println!("💡 Resource monitoring capabilities:");
    println!("  • Memory limit: 256MB (configured)");
    println!("  • CPU shares: Default");
    println!("  • Port mappings: Dynamic allocation");
    println!("  • Volume mounts: None (in-memory only)");

    // Cleanup
    println!("\n🧹 Cleaning up...");

    // Stop the container
    container_manager
        .stop(&container_id, Some(Duration::from_secs(10)))
        .await?;

    println!("✅ Container stopped gracefully");

    // Container will be automatically removed due to auto_remove flag
    println!("✅ Container will be automatically removed (auto_remove enabled)");

    println!("\n🎉 Container Management Demo Complete!");
    println!("======================================");
    println!("Features demonstrated:");
    println!("✅ Advanced container configuration with fluent API");
    println!("✅ Dynamic port allocation and mapping");
    println!("✅ Health checking with composite checks");
    println!("✅ Container command execution");
    println!("✅ Streaming command execution with real-time output");
    println!("✅ Log management and streaming");
    println!("✅ Container lifecycle management");
    println!("✅ Resource limits and monitoring");
    println!("✅ Graceful cleanup and auto-removal");

    Ok(())
}
