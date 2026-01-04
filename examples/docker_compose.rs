//! Docker Compose example demonstrating multi-container application management
//!
//! This example shows how to use the compose feature to manage
//! multi-container applications with Docker Compose.
//!
//! Run with: cargo run --example docker_compose --features compose

#[cfg(feature = "compose")]
use docker_wrapper::command::ComposeCommand;
#[cfg(feature = "compose")]
use docker_wrapper::compose::{
    ComposeDownCommand, ComposeLogsCommand, ComposePsCommand, ComposeUpCommand, RemoveImages,
};
#[cfg(feature = "compose")]
use docker_wrapper::DockerCommand;
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
    println!("Example 1: Basic Compose Workflow");
    println!("------------------------------------");

    // Start services with compose up
    println!("Starting services with compose up...");
    let up_result = ComposeUpCommand::new()
        .file("docker-compose.yml")
        .project_name("my-app")
        .detach() // Run in background
        .build() // Build images if needed
        .remove_orphans() // Clean up orphan containers
        .wait() // Wait for services to be healthy
        .wait_timeout(Duration::from_secs(60))
        .execute()
        .await;

    match up_result {
        Ok(result) => {
            println!("Services started successfully");
            println!("Output: {}", result.stdout.trim());
        }
        Err(e) => {
            println!("Failed to start services: {}", e);
            // Continue with example using a simulated environment
        }
    }

    println!();

    // Example 2: Check service status
    println!("Example 2: Checking Service Status");
    println!("-------------------------------------");

    let ps_result = ComposePsCommand::new()
        .file("docker-compose.yml")
        .project_name("my-app")
        .all() // Show all containers
        .execute()
        .await;

    match ps_result {
        Ok(result) => {
            println!("Service status retrieved");
            println!("Output: {}", result.stdout.trim());
        }
        Err(e) => println!("Could not retrieve service status: {}", e),
    }

    println!();

    // Example 3: View logs
    println!("Example 3: Viewing Service Logs");
    println!("----------------------------------");

    let logs_result = ComposeLogsCommand::new()
        .file("docker-compose.yml")
        .project_name("my-app")
        .tail("20") // Last 20 lines
        .timestamps() // Include timestamps
        .no_color() // Disable colors for cleaner output
        .service("web") // Specific service
        .execute()
        .await;

    match logs_result {
        Ok(result) => {
            println!("Logs retrieved");
            println!("Output: {}", result.stdout.trim());
        }
        Err(e) => println!("Could not retrieve logs: {}", e),
    }

    println!();

    // Example 4: Advanced compose up with scaling
    println!("Example 4: Advanced Compose Configuration");
    println!("---------------------------------------------");

    println!("Starting services with scaling...");
    let scaled_up = ComposeUpCommand::new()
        .file("docker-compose.yml")
        .project_name("scaled-app")
        .scale("web", 3) // Scale web service to 3 instances
        .scale("worker", 2) // Scale worker to 2 instances
        .detach()
        .no_recreate() // Don't recreate existing containers
        .execute()
        .await;

    match scaled_up {
        Ok(_) => {
            println!("Services scaled successfully");
            println!("   web: 3 instances");
            println!("   worker: 2 instances");
        }
        Err(e) => println!(
            "Scaling failed (expected without valid compose file): {}",
            e
        ),
    }

    println!();

    // Example 5: Selective service management
    println!("Example 5: Selective Service Management");
    println!("------------------------------------------");

    // Start only specific services
    let selective_up = ComposeUpCommand::new()
        .file("docker-compose.yml")
        .service("database") // Only start database
        .service("cache") // And cache
        .no_deps() // Don't start dependent services
        .detach()
        .execute()
        .await;

    match selective_up {
        Ok(_) => println!("Selected services started"),
        Err(e) => println!(
            "Selective start failed (expected without valid compose file): {}",
            e
        ),
    }

    println!();

    // Example 6: Clean shutdown
    println!("Example 6: Clean Shutdown");
    println!("----------------------------");

    let down_result = ComposeDownCommand::new()
        .file("docker-compose.yml")
        .project_name("my-app")
        .volumes() // Remove volumes
        .remove_images(RemoveImages::Local) // Remove local images
        .remove_orphans() // Remove orphan containers
        .timeout(Duration::from_secs(30)) // Graceful shutdown timeout
        .execute()
        .await;

    match down_result {
        Ok(_) => println!("Services stopped and cleaned up"),
        Err(e) => println!("Cleanup failed (expected without running services): {}", e),
    }

    println!();

    // Example 7: Development workflow patterns
    println!("Example 7: Development Workflow Patterns");
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
    println!("       .execute()");
    println!();

    // Production deployment
    println!("2. Production deployment:");
    println!("   ComposeUpCommand::new()");
    println!("       .file(\"docker-compose.prod.yml\")");
    println!("       .detach()");
    println!("       .wait() // Wait for health checks");
    println!("       .execute()");
    println!();

    // Testing with isolated environment
    println!("3. Testing with isolated environment:");
    println!("   ComposeUpCommand::new()");
    println!("       .project_name(format!(\"test-{{}}\", uuid))");
    println!("       .abort_on_container_exit() // Stop when test completes");
    println!("       .exit_code_from(\"tests\") // Use test exit code");
    println!("       .execute()");
    println!();

    // Monitoring logs in real-time
    println!("4. Real-time log monitoring:");
    println!("   ComposeLogsCommand::new()");
    println!("       .follow() // Follow log output");
    println!("       .timestamps()");
    println!("       .service(\"app\")");
    println!("       .execute()");

    println!("\nDocker Compose example completed!");
    println!("\nKey features demonstrated:");
    println!("   - Starting services with compose up");
    println!("   - Checking service status with compose ps");
    println!("   - Viewing logs with compose logs");
    println!("   - Scaling services");
    println!("   - Selective service management");
    println!("   - Clean shutdown with volume/image removal");
    println!("   - Development workflow patterns");

    Ok(())
}
