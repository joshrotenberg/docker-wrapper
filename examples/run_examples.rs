//! Examples demonstrating the docker run command implementation.
//!
//! This example shows various ways to use the RunCommand with both
//! high-level structured APIs and extensible escape hatches.

use docker_wrapper::{ensure_docker, DockerCommand, RunCommand};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("Docker Run Command Examples");
    println!("===========================\n");

    // First ensure Docker is available
    match ensure_docker().await {
        Ok(info) => {
            println!(
                "Docker {} detected, daemon running: {}",
                info.version.version, info.daemon_running
            );
        }
        Err(e) => {
            println!("Docker not available: {}", e);
            return;
        }
    }

    println!();

    // Example 1: Simple container run
    println!("1. Simple container run:");
    let simple_run = RunCommand::new("alpine:latest")
        .cmd(vec!["echo".to_string(), "Hello from Alpine!".to_string()]);

    println!("Command: docker {}", simple_run.build_args().join(" "));

    match simple_run.execute().await {
        Ok(container_id) => {
            println!("Container created: {}", container_id.short());
        }
        Err(e) => {
            println!("Failed to run container: {}", e);
        }
    }

    println!();

    // Example 2: Web server with ports and environment
    println!("2. Web server with ports and environment:");
    let mut env_vars = HashMap::new();
    env_vars.insert("ENV".to_string(), "production".to_string());
    env_vars.insert("PORT".to_string(), "8080".to_string());

    let web_run = RunCommand::new("nginx:alpine")
        .name("example-nginx")
        .detach()
        .port(8080, 80)
        .envs(env_vars)
        .remove();

    println!("Command: docker {}", web_run.build_args().join(" "));

    match web_run.execute().await {
        Ok(container_id) => {
            println!("Web server started: {}", container_id);
            println!("Visit http://localhost:8080 to see it running");

            // Clean up - stop the container
            let stop_output = tokio::process::Command::new("docker")
                .args(["stop", container_id.as_str()])
                .output()
                .await;

            match stop_output {
                Ok(_) => println!("Container stopped"),
                Err(e) => println!("Failed to stop container: {}", e),
            }
        }
        Err(e) => {
            println!("Failed to start web server: {}", e);
        }
    }

    println!();

    // Example 3: Interactive container with TTY
    println!("3. Interactive container setup (not executed):");
    let interactive_run = RunCommand::new("ubuntu:latest")
        .name("interactive-ubuntu")
        .it() // Convenience method for --interactive --tty
        .workdir("/home")
        .env("USER", "developer")
        .remove();

    println!("Command: docker {}", interactive_run.build_args().join(" "));
    println!("This would start an interactive Ubuntu shell");

    println!();

    // Example 4: Container with volumes
    println!("4. Container with volume mounts:");
    let volume_run = RunCommand::new("postgres:13")
        .name("example-postgres")
        .detach()
        .env("POSTGRES_PASSWORD", "secret123")
        .env("POSTGRES_DB", "testdb")
        .port(5432, 5432)
        .volume("postgres-data", "/var/lib/postgresql/data")
        .bind("/tmp/postgres-logs", "/var/log/postgresql")
        .remove();

    println!("Command: docker {}", volume_run.build_args().join(" "));
    println!("This would start PostgreSQL with persistent data and log bind mount");

    println!();

    // Example 5: Extensible usage with custom options
    println!("5. Using extensibility for advanced options:");
    let mut advanced_run = RunCommand::new("redis:alpine")
        .name("advanced-redis")
        .detach()
        .port(6379, 6379);

    // Use the extensible API for advanced options not directly supported
    advanced_run
        .option("memory", "512m")
        .option("cpu-shares", "512")
        .flag("privileged")
        .option("restart", "unless-stopped")
        .arg("--log-driver=json-file")
        .option("log-opt", "max-size=10m");

    println!("Command: docker {}", advanced_run.build_args().join(" "));
    println!("This demonstrates using escape hatches for any Docker option");

    println!();

    // Example 6: Build args demonstration
    println!("6. Command argument breakdown:");
    let demo_run = RunCommand::new("busybox:latest")
        .name("demo-container")
        .env("DEBUG", "true")
        .port(8080, 80)
        .volume_ro("config", "/etc/config")
        .workdir("/app")
        .cmd(vec![
            "sh".to_string(),
            "-c".to_string(),
            "sleep 300".to_string(),
        ]);

    let args = demo_run.build_args();
    println!("Full argument list:");
    for (i, arg) in args.iter().enumerate() {
        println!("  [{}]: {}", i, arg);
    }

    println!();
    println!("Examples completed!");
    println!("\nNote: Some examples are for demonstration only and don't execute");
    println!("to avoid requiring specific images or leaving containers running.");
}
