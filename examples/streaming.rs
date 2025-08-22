//! Example demonstrating streaming output from Docker commands
//!
//! This example shows how to use the streaming API to get real-time output
//! from Docker commands like build, run, and logs.
//!
//! Run with: cargo run --example streaming

use docker_wrapper::command::DockerCommand;
use docker_wrapper::{BuildCommand, LogsCommand, RunCommand};
use docker_wrapper::{OutputLine, StreamHandler, StreamableCommand};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Docker Streaming Example");
    println!("========================\n");

    // Example 1: Stream build output
    example_build_streaming().await?;

    // Example 2: Stream container output
    example_run_streaming().await?;

    // Example 3: Stream logs with filtering
    example_logs_filtering().await?;

    // Example 4: Channel-based streaming
    example_channel_streaming().await?;

    println!("\n✨ All streaming examples completed!");
    Ok(())
}

async fn example_build_streaming() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Example 1: Streaming Docker Build Output");
    println!("-------------------------------------------");

    // Create a simple Dockerfile for testing
    std::fs::write(
        "Dockerfile.streaming",
        r#"
FROM alpine:latest
RUN echo "Step 1: Installing packages..."
RUN echo "Step 2: Setting up application..."
RUN echo "Step 3: Configuring environment..."
CMD ["echo", "Build complete!"]
"#,
    )?;

    println!("Building image with streaming output...\n");

    // Stream build output to console
    let result = BuildCommand::new(".")
        .file("Dockerfile.streaming")
        .tag("streaming-example:latest")
        .stream(StreamHandler::print())
        .await?;

    if result.is_success() {
        println!("\n✅ Build completed successfully!");
    } else {
        println!("\n❌ Build failed with exit code: {}", result.exit_code);
    }

    // Clean up
    std::fs::remove_file("Dockerfile.streaming")?;

    Ok(())
}

async fn example_run_streaming() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🚀 Example 2: Streaming Container Output");
    println!("----------------------------------------");

    // Run a container that produces output over time
    println!("Running container with streaming output...\n");

    let line_count = Arc::new(AtomicUsize::new(0));
    let count_clone = line_count.clone();

    let result = RunCommand::new("alpine")
        .cmd(vec![
            "sh".to_string(),
            "-c".to_string(),
            "for i in 1 2 3 4 5; do echo \"Line $i\"; sleep 0.1; done".to_string(),
        ])
        .remove() // Remove container after exit
        .stream(move |line| match line {
            OutputLine::Stdout(text) => {
                println!("Container: {}", text);
                count_clone.fetch_add(1, Ordering::SeqCst);
            }
            OutputLine::Stderr(text) => {
                eprintln!("Container Error: {}", text);
            }
        })
        .await?;

    println!("\n✅ Container exited with code: {}", result.exit_code);
    println!(
        "   Processed {} lines of output",
        line_count.load(Ordering::SeqCst)
    );

    Ok(())
}

async fn example_logs_filtering() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📜 Example 3: Streaming Logs with Filtering");
    println!("-------------------------------------------");

    // First, create a container that generates logs
    println!("Creating a container for log streaming...");

    let container_name = "streaming-log-example";

    // Run a container in detached mode that generates logs
    RunCommand::new("alpine")
        .name(container_name)
        .detach()
        .remove()
        .cmd(vec!["sh".to_string(), "-c".to_string(), "for i in 1 2 3 4 5; do echo \"Log entry $i\"; echo \"Error: Something went wrong $i\" >&2; sleep 1; done".to_string()])
        .execute()
        .await?;

    println!("Streaming logs with custom filtering...\n");

    // Stream logs with a custom filter
    let mut error_count = 0;
    let mut info_count = 0;

    let _result = LogsCommand::new(container_name)
        .follow()
        .timestamps()
        .tail("all")
        .stream(move |line| match line {
            OutputLine::Stdout(text) => {
                if text.contains("Log entry") {
                    println!("[INFO] {}", text);
                    info_count += 1;
                }
            }
            OutputLine::Stderr(text) => {
                if text.contains("Error") {
                    eprintln!("[ERROR] {}", text);
                    error_count += 1;
                }
            }
        })
        .await;

    // Note: The logs command will continue until the container exits
    println!("\n✅ Log streaming completed");
    println!("   Info messages: {}", info_count);
    println!("   Error messages: {}", error_count);

    // Stop and remove the container
    let _ = std::process::Command::new("docker")
        .args(&["stop", container_name])
        .output();

    Ok(())
}

async fn example_channel_streaming() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📡 Example 4: Channel-based Streaming");
    println!("-------------------------------------");

    println!("Using channel to process output asynchronously...\n");

    // Run a command that produces output
    let (mut rx, _result) = RunCommand::new("alpine")
        .cmd(vec![
            "sh".to_string(),
            "-c".to_string(),
            "for i in 1 2 3; do echo \"Data: $i\"; sleep 0.5; done".to_string(),
        ])
        .remove()
        .stream_channel()
        .await?;

    // Process output from channel in a separate task
    let processor = tokio::spawn(async move {
        let mut lines = Vec::new();
        while let Some(line) = rx.recv().await {
            match line {
                OutputLine::Stdout(text) => {
                    println!("Received via channel: {}", text);
                    lines.push(text);
                }
                OutputLine::Stderr(text) => {
                    eprintln!("Error via channel: {}", text);
                }
            }
        }
        lines
    });

    // Wait for both the command and processor to complete
    let lines = processor.await?;

    println!("\n✅ Channel streaming completed");
    println!("   Collected {} lines via channel", lines.len());

    Ok(())
}

// Additional example: Progress tracking during build
#[allow(dead_code)]
async fn example_build_progress() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🏗️  Bonus: Build Progress Tracking");
    println!("---------------------------------");

    let mut current_step = 0;
    let total_steps = 5;

    let result = BuildCommand::new(".")
        .tag("progress-example:latest")
        .stream(move |line| {
            if let OutputLine::Stdout(text) = line {
                if text.contains("Step") {
                    current_step += 1;
                    let progress = (current_step as f32 / total_steps as f32) * 100.0;
                    println!("[{:.0}%] {}", progress, text);
                } else {
                    println!("       {}", text);
                }
            }
        })
        .await?;

    if result.is_success() {
        println!("\n✅ Build completed with progress tracking!");
    }

    Ok(())
}
