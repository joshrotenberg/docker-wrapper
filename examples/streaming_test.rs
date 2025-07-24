//! Streaming Test
//!
//! Simple test to verify streaming command execution and log streaming fixes

use docker_wrapper::*;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing streaming fixes...");

    // Create Docker client
    let client = DockerClient::new().await?;
    println!("✅ Connected to Docker daemon");

    // Create a simple container
    println!("\n📦 Creating test container...");
    let container_id = ContainerBuilder::new("alpine:latest")
        .name("streaming-test")
        .auto_remove()
        .command(vec!["sleep".to_string(), "30".to_string()])
        .run(&client)
        .await?;

    println!("✅ Container created: {}", container_id.short());

    // Test 1: Simple exec (should work)
    println!("\n⚡ Testing simple exec...");
    let executor = ContainerExecutor::new(&client);
    let result = executor
        .exec_simple(
            &container_id,
            vec!["echo".to_string(), "Hello World".to_string()],
        )
        .await?;
    println!("✅ Simple exec result: {}", result.trim());

    // Test 2: Streaming exec (test our fix)
    println!("\n🌊 Testing streaming exec...");
    let mut output_count = 0;

    let stream_result = timeout(
        Duration::from_secs(3),
        executor.exec_streaming(
            &container_id,
            ExecConfig::from_command_str("echo line1; echo line2; echo line3"),
            |output| {
                match output {
                    ExecOutput::Stdout(line) => {
                        println!("📤 STDOUT: {}", line);
                        output_count += 1;
                    }
                    ExecOutput::Stderr(line) => {
                        println!("📥 STDERR: {}", line);
                    }
                }

                // Stop after a few lines
                if output_count >= 3 {
                    Err(DockerError::cancelled("Test complete".to_string()))
                } else {
                    Ok(())
                }
            },
        ),
    )
    .await;

    match stream_result {
        Ok(Err(e)) if e.to_string().contains("Test complete") => {
            println!("✅ Streaming exec completed successfully");
        }
        Ok(Ok(_)) => {
            println!("✅ Streaming exec completed normally");
        }
        Err(_) => {
            println!("⏰ Streaming exec timed out (may indicate command construction issue)");
        }
        Ok(Err(e)) => {
            println!("⚠️  Streaming exec error: {}", e);
        }
    }

    // Test 3: Log retrieval (test our fix)
    println!("\n📜 Testing log retrieval...");
    let log_manager = LogManager::new(&client);

    let logs_result = timeout(
        Duration::from_secs(5),
        log_manager.get_logs(&container_id, LogOptions::new().tail(5)),
    )
    .await;

    match logs_result {
        Ok(Ok(logs)) => {
            println!("✅ Retrieved {} bytes of logs", logs.len());
            if !logs.is_empty() {
                println!(
                    "📋 Log sample: {}",
                    logs.lines().next().unwrap_or("(empty)")
                );
            }
        }
        Ok(Err(e)) => {
            println!("⚠️  Log retrieval error: {}", e);
        }
        Err(_) => {
            println!("⏰ Log retrieval timed out (may indicate command construction issue)");
        }
    }

    // Test 4: Log streaming (test our fix)
    println!("\n📺 Testing log streaming...");
    let mut log_count = 0;

    let log_stream_result = timeout(
        Duration::from_secs(3),
        log_manager.stream_logs(&container_id, LogOptions::new().timestamps(), |entry| {
            log_count += 1;
            println!(
                "📜 [{}] {}: {}",
                log_count,
                entry.source,
                entry.message.chars().take(50).collect::<String>()
            );

            // Stop after a few entries
            if log_count >= 3 {
                Err(DockerError::cancelled("Log test complete".to_string()))
            } else {
                Ok(())
            }
        }),
    )
    .await;

    match log_stream_result {
        Ok(Err(e)) if e.to_string().contains("Log test complete") => {
            println!("✅ Log streaming completed successfully");
        }
        Ok(Ok(_)) => {
            println!("✅ Log streaming completed normally");
        }
        Err(_) => {
            println!("⏰ Log streaming timed out (may indicate command construction issue)");
        }
        Ok(Err(e)) => {
            println!("⚠️  Log streaming error: {}", e);
        }
    }

    // Cleanup
    println!("\n🧹 Cleaning up...");
    let manager = ContainerManager::new(&client);
    let _ = manager
        .stop(&container_id, Some(Duration::from_secs(5)))
        .await;
    println!("✅ Container will be auto-removed");

    println!("\n🎉 Streaming test complete!");
    println!("===============================");
    println!("If you see successful results above, the streaming fixes are working!");

    Ok(())
}
