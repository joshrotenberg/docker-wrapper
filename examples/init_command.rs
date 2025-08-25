//! Docker init command example
//!
//! This example demonstrates how to use the `docker init` command to initialize
//! a project with Docker-related starter files.
//!
//! Run with: cargo run --example init_command

use docker_wrapper::{DockerCommand, InitCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Docker Init Command Example");
    println!("===========================\n");

    // Get version information
    println!("Getting docker init version...");
    let version_output = InitCommand::new().version().execute().await?;

    if let Some(version) = version_output.version() {
        println!("Docker init version: {}\n", version);
    } else {
        println!("Could not retrieve version information\n");
    }

    // Show basic command construction
    println!("Available commands:");
    println!("  Basic init: {}", InitCommand::new());
    println!("  Version:    {}\n", InitCommand::new().version());

    println!("Notes:");
    println!("- The basic init command runs interactively");
    println!("- It creates Dockerfile, compose.yaml, .dockerignore, and README.Docker.md");
    println!("- Available templates: ASP.NET Core, Go, Java, Node, PHP, Python, Rust, Other");
    println!("- Run in a project directory to initialize Docker support");
    println!("\nExample usage in your project:");
    println!("  cd /path/to/your/project");
    println!("  docker init  # Interactive mode");

    Ok(())
}
