//! Basic usage example for docker-wrapper prerequisites checking.
//!
//! This example demonstrates how to check Docker prerequisites
//! and handle various scenarios.

use docker_wrapper::{ensure_docker, DockerPrerequisites, Error};

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("Docker Wrapper - Prerequisites Example");
    println!("=====================================\n");

    // Example 1: Simple prerequisites check
    println!("1. Simple Docker check:");
    match ensure_docker().await {
        Ok(info) => {
            println!("✅ Docker is available!");
            println!("   Binary: {}", info.binary_path);
            println!("   Version: {}", info.version.version);
            println!("   OS: {}", info.os);
            println!("   Architecture: {}", info.architecture);

            if info.daemon_running {
                println!("   Daemon: Running");
                if let Some(server_version) = &info.server_version {
                    println!("   Server Version: {}", server_version.version);
                }
            } else {
                println!("   Daemon: Not running");
            }
        }
        Err(e) => {
            println!("❌ Docker check failed: {e}");
            handle_prerequisites_error(&e);
        }
    }

    println!();

    // Example 2: Custom minimum version check
    println!("2. Custom minimum version check (25.0.0):");
    let custom_version = match docker_wrapper::prerequisites::DockerVersion::parse("25.0.0") {
        Ok(version) => version,
        Err(e) => {
            println!("❌ Failed to parse version: {e}");
            return;
        }
    };

    let checker = DockerPrerequisites::new(custom_version);
    match checker.check().await {
        Ok(info) => {
            println!("✅ Docker meets high version requirement!");
            println!("   Version: {}", info.version.version);
        }
        Err(e) => {
            println!("❌ Docker version check failed: {e}");
            match &e {
                Error::UnsupportedVersion { found, minimum } => {
                    println!("   Found: {found}, Required: {minimum}");
                }
                _ => handle_prerequisites_error(&e),
            }
        }
    }

    println!();

    // Example 3: Detailed system information
    println!("3. Detailed system information:");
    match ensure_docker().await {
        Ok(info) => {
            println!("System Information:");
            println!("├── Operating System: {}", info.os);
            println!("├── Architecture: {}", info.architecture);
            println!("├── Docker Binary: {}", info.binary_path);
            println!("├── Client Version: {}", info.version.version);
            println!("│   ├── Major: {}", info.version.major);
            println!("│   ├── Minor: {}", info.version.minor);
            println!("│   └── Patch: {}", info.version.patch);

            if info.daemon_running {
                println!("└── Docker Daemon: Running");
                if let Some(server_version) = &info.server_version {
                    println!("    └── Server Version: {}", server_version.version);
                }
            } else {
                println!("└── Docker Daemon: Not Running");
            }
        }
        Err(e) => {
            println!("❌ Failed to get system information: {e}");
        }
    }
}

/// Handle different types of prerequisites errors with helpful messages
fn handle_prerequisites_error(error: &Error) {
    match error {
        Error::DockerNotFound => {
            println!("   💡 Install Docker from: https://docs.docker.com/get-docker/");
        }
        Error::DaemonNotRunning => {
            println!("   💡 Start Docker daemon with: sudo systemctl start docker");
            println!("   💡 Or start Docker Desktop application");
        }
        Error::UnsupportedVersion { found, minimum } => {
            println!("   💡 Update Docker to version {minimum} or higher");
            println!("   💡 Current version: {found}");
        }
        Error::CommandFailed { command, .. } => {
            println!("   💡 Command execution failed: {command}");
        }
        Error::ParseError { message } => {
            println!("   💡 Parse error: {message}");
        }
        _ => {
            println!("   💡 Unexpected error: {error}");
        }
    }
}
