//! Platform detection example
//!
//! This example demonstrates how to use the platform detection and runtime abstraction
//! features of docker-wrapper to automatically detect and adapt to different container
//! runtimes like Docker, Podman, Colima, OrbStack, etc.

use docker_wrapper::{DockerCommand, PlatformInfo, RunCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Docker Wrapper - Platform Detection Example");
    println!("==========================================");

    // Detect platform and runtime
    println!("\n1. Detecting platform and runtime...");
    match PlatformInfo::detect() {
        Ok(platform_info) => {
            println!("✅ Platform detected: {}", platform_info);
            println!("   - OS: {}", platform_info.platform);
            println!("   - Runtime: {}", platform_info.runtime);
            println!("   - Version: {}", platform_info.version);
            if platform_info.is_wsl {
                println!("   - Running in WSL");
            }
            println!("   - Socket path: {}", platform_info.socket_path.display());

            // Check if runtime is working
            println!("\n2. Checking runtime availability...");
            match platform_info.check_runtime() {
                Ok(()) => {
                    println!("✅ Runtime is available and working");
                }
                Err(e) => {
                    println!("❌ Runtime check failed: {}", e);
                    return Ok(());
                }
            }

            // Show environment variables that would be set
            println!("\n3. Runtime-specific environment variables:");
            let env_vars = platform_info.environment_vars();
            if env_vars.is_empty() {
                println!("   None required");
            } else {
                for (key, value) in &env_vars {
                    println!("   {}={}", key, value);
                }
            }

            // Show compose command for this runtime
            println!("\n4. Docker Compose support:");
            if platform_info.runtime.supports_compose() {
                let compose_cmd = platform_info.runtime.compose_command();
                println!("   ✅ Supported, command: {}", compose_cmd.join(" "));
            } else {
                println!("   ❌ Not supported");
            }

            // Run a simple container to test the detected runtime
            println!("\n5. Testing runtime with a simple container...");
            let run_cmd = RunCommand::new("alpine:latest")
                .name("platform-test")
                .cmd(vec![
                    "echo".to_string(),
                    "Hello from docker-wrapper!".to_string(),
                ])
                .remove(); // Auto-remove when done

            match run_cmd.execute().await {
                Ok(result) => {
                    println!("✅ Container executed successfully");
                    println!("   Container ID: {}", result.0);
                    println!(
                        "   The runtime {} is working correctly!",
                        platform_info.runtime
                    );
                }
                Err(e) => {
                    println!("❌ Failed to run container: {}", e);
                    println!("   This might indicate the runtime is not properly configured");
                }
            }
        }
        Err(e) => {
            println!("❌ Platform detection failed: {}", e);
            println!("This could mean:");
            println!("  - No container runtime (Docker/Podman) is installed");
            println!("  - The runtime is not in your PATH");
            println!("  - The runtime daemon is not running");
        }
    }

    println!("\n6. Manual platform detection examples:");

    // Example: Check if we're on macOS and might have multiple Docker options
    let platform = docker_wrapper::Platform::detect();
    match platform {
        docker_wrapper::Platform::MacOS => {
            println!("   Running on macOS - checking for common Docker setups:");

            // Common macOS Docker setups
            let setups = [
                (
                    "Docker Desktop",
                    std::path::Path::new("/Applications/Docker.app"),
                ),
                ("Colima", std::path::Path::new("/opt/homebrew/bin/colima")),
                (
                    "OrbStack",
                    std::path::Path::new("/Applications/OrbStack.app"),
                ),
                (
                    "Rancher Desktop",
                    std::path::Path::new("/Applications/Rancher Desktop.app"),
                ),
            ];

            for (name, path) in &setups {
                if path.exists() {
                    println!("   ✅ {} detected at {}", name, path.display());
                } else {
                    println!("   ❌ {} not found", name);
                }
            }
        }
        docker_wrapper::Platform::Linux => {
            println!("   Running on Linux - checking for Docker/Podman:");

            // Check common Linux container runtimes
            let runtimes = ["docker", "podman"];
            for runtime in &runtimes {
                if which::which(runtime).is_ok() {
                    println!("   ✅ {} found in PATH", runtime);
                } else {
                    println!("   ❌ {} not found in PATH", runtime);
                }
            }
        }
        docker_wrapper::Platform::Windows => {
            println!("   Running on Windows - Docker Desktop is the primary option");
        }
        other => {
            println!("   Running on {}", other);
        }
    }

    println!("\nPlatform detection example completed!");
    Ok(())
}
