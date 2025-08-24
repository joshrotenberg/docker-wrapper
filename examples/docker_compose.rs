//! Docker Compose example demonstrating multi-container application management
//!
//! This example shows how to use the compose feature to manage
//! multi-container applications with Docker Compose.
//!
//! Run with: cargo run --example docker_compose --features compose

#[cfg(feature = "compose")]
use docker_wrapper::compose::down::RemoveImages;
#[cfg(feature = "compose")]
use docker_wrapper::compose::up::PullPolicy;
#[cfg(feature = "compose")]
use docker_wrapper::compose::{
    ComposeConfig, ComposeDownCommand, ComposeLogsCommand, ComposePsCommand, ComposeUpCommand,
};
#[cfg(feature = "compose")]
use std::time::Duration;

#[cfg(not(feature = "compose"))]
fn main() {
    println!("This example requires the 'compose' feature.");
    println!("Run with: cargo run --example docker_compose --features compose");
}

#[cfg(feature = "compose")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Docker Compose Management Example");
    println!("=================================\n");

    // Example 1: Basic compose up/down workflow
    println!("📦 Example 1: Basic Compose Workflow");
    println!("------------------------------------");

    // Configure compose with a specific file
    let config = ComposeConfig::new()
        .file("docker-compose.yml")
        .project_name("my-app");

    // Start services with compose up
    println!("Starting services with compose up...");
    let up_result = ComposeUpCommand::with_config(config.clone())
        .detach() // Run in background
        .build() // Build images if needed
        .remove_orphans() // Clean up orphan containers
        .wait() // Wait for services to be healthy
        .wait_timeout(Duration::from_secs(60))
        .pull(PullPolicy::Missing) // Pull missing images
        .run()
        .await;

    match up_result {
        Ok(result) if result.success() => {
            println!("✅ Services started successfully");
            if result.is_detached() {
                println!("   Running in detached mode");
            }
        }
        Ok(_) => println!("⚠️  Services started with warnings"),
        Err(e) => {
            println!("❌ Failed to start services: {}", e);
            // Continue with example using a simulated environment
        }
    }

    println!();

    // Example 2: Check service status
    println!("📋 Example 2: Checking Service Status");
    println!("-------------------------------------");

    let ps_result = ComposePsCommand::with_config(config.clone())
        .all() // Show all containers
        .json() // Use JSON format for structured data
        .run()
        .await;

    match ps_result {
        Ok(result) if result.success() => {
            println!("✅ Service status retrieved");
            if !result.containers().is_empty() {
                for container in result.containers() {
                    println!(
                        "   {} ({}): {} - Health: {}",
                        container.service,
                        container.name,
                        container.state,
                        container.health.as_deref().unwrap_or("N/A")
                    );
                }
            } else {
                // Fallback to text output
                for id in result.container_ids() {
                    println!("   Container: {}", id);
                }
            }
        }
        _ => println!("⚠️  Could not retrieve service status"),
    }

    println!();

    // Example 3: View logs
    println!("📜 Example 3: Viewing Service Logs");
    println!("----------------------------------");

    let logs_result = ComposeLogsCommand::with_config(config.clone())
        .tail("20") // Last 20 lines
        .timestamps() // Include timestamps
        .no_color() // Disable colors for cleaner output
        .service("web") // Specific service
        .run()
        .await;

    match logs_result {
        Ok(result) if result.success() => {
            println!("✅ Logs retrieved for service: {:?}", result.services());
            // In a real app, you'd process result.output.stdout
            println!("   [Log output would appear here]");
        }
        _ => println!("⚠️  Could not retrieve logs"),
    }

    println!();

    // Example 4: Advanced compose up with scaling
    println!("⚙️  Example 4: Advanced Compose Configuration");
    println!("---------------------------------------------");

    println!("Starting services with scaling...");
    let scaled_up = ComposeUpCommand::new()
        .file("docker-compose.yml")
        .project_name("scaled-app")
        .scale("web", 3) // Scale web service to 3 instances
        .scale("worker", 2) // Scale worker to 2 instances
        .detach()
        .no_recreate() // Don't recreate existing containers
        .run()
        .await;

    match scaled_up {
        Ok(result) if result.success() => {
            println!("✅ Services scaled successfully");
            println!("   web: 3 instances");
            println!("   worker: 2 instances");
        }
        _ => println!("⚠️  Scaling demonstration (would work with valid compose file)"),
    }

    println!();

    // Example 5: Selective service management
    println!("🎯 Example 5: Selective Service Management");
    println!("------------------------------------------");

    // Start only specific services
    let selective_up = ComposeUpCommand::new()
        .file("docker-compose.yml")
        .service("database") // Only start database
        .service("cache") // And cache
        .no_deps() // Don't start dependent services
        .detach()
        .run()
        .await;

    match selective_up {
        Ok(result) if result.success() => {
            println!("✅ Selected services started: {:?}", result.services());
        }
        _ => println!("⚠️  Selective start demonstration"),
    }

    println!();

    // Example 6: Clean shutdown
    println!("🧹 Example 6: Clean Shutdown");
    println!("----------------------------");

    let down_result = ComposeDownCommand::with_config(config.clone())
        .volumes() // Remove volumes
        .remove_images(RemoveImages::Local) // Remove local images
        .remove_orphans() // Remove orphan containers
        .timeout(Duration::from_secs(30)) // Graceful shutdown timeout
        .run()
        .await;

    match down_result {
        Ok(result) if result.success() => {
            println!("✅ Services stopped and cleaned up");
            if result.volumes_removed() {
                println!("   Volumes removed");
            }
            if result.images_removed() {
                println!("   Local images removed");
            }
        }
        _ => println!("⚠️  Cleanup demonstration"),
    }

    println!();

    // Example 7: Development workflow
    println!("💻 Example 7: Development Workflow");
    println!("----------------------------------");

    println!("Common development compose patterns:");
    println!();

    // Development mode with rebuild
    println!("1. Development mode with auto-rebuild:");
    println!("   ComposeUpCommand::new()");
    println!("       .file(\"docker-compose.dev.yml\")");
    println!("       .build() // Always rebuild");
    println!("       .force_recreate() // Fresh containers");
    println!("       .remove_orphans()");
    println!("       .run()");
    println!();

    // Production deployment
    println!("2. Production deployment:");
    println!("   ComposeUpCommand::new()");
    println!("       .file(\"docker-compose.prod.yml\")");
    println!("       .profile(\"production\") // Use production profile");
    println!("       .pull(PullPolicy::Always) // Always pull latest");
    println!("       .detach()");
    println!("       .wait() // Wait for health checks");
    println!("       .run()");
    println!();

    // Testing with isolated environment
    println!("3. Testing with isolated environment:");
    println!("   ComposeUpCommand::new()");
    println!("       .project_name(format!(\"test-{{}}\", uuid))");
    println!("       .abort_on_container_exit() // Stop when test completes");
    println!("       .exit_code_from(\"tests\") // Use test exit code");
    println!("       .run()");
    println!();

    // Monitoring logs in real-time
    println!("4. Real-time log monitoring:");
    println!("   ComposeLogsCommand::new()");
    println!("       .follow() // Follow log output");
    println!("       .timestamps()");
    println!("       .service(\"app\")");
    println!("       .since(\"10m\") // Last 10 minutes");
    println!("       .run()");

    println!("\n✨ Docker Compose example completed!");
    println!("\n💡 Key features demonstrated:");
    println!("   - Starting services with compose up");
    println!("   - Checking service status with compose ps");
    println!("   - Viewing logs with compose logs");
    println!("   - Scaling services");
    println!("   - Selective service management");
    println!("   - Clean shutdown with volume/image removal");
    println!("   - Development workflow patterns");

    Ok(())
}
