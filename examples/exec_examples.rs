//! Examples demonstrating Docker exec command usage.
//!
//! This example shows various ways to use the exec command to execute
//! commands in running containers.

use docker_wrapper::command::DockerCommand;
use docker_wrapper::prerequisites::ensure_docker;
use docker_wrapper::{ExecCommand, RunCommand};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Docker Exec Examples");
    println!("======================");

    // Check Docker availability
    match ensure_docker().await {
        Ok(info) => {
            println!("✅ Docker is available");
            println!("   Version: {}", info.version.version);
        }
        Err(e) => {
            println!("❌ Docker not available: {e}");
            return Ok(());
        }
    }

    // Start a test container for our examples
    println!("\n📦 Starting test container...");
    let container_id = match start_test_container().await {
        Ok(id) => {
            println!("✅ Container started: {id}");
            id
        }
        Err(e) => {
            println!("❌ Failed to start container: {e}");
            return Ok(());
        }
    };

    // Example 1: Simple command execution
    println!("\n1️⃣ Simple Command Execution");
    println!("-----------------------------");
    simple_exec_example(&container_id).await;

    // Example 2: Interactive TTY example (non-blocking for demo)
    println!("\n2️⃣ Interactive TTY Command");
    println!("---------------------------");
    interactive_exec_example(&container_id).await;

    // Example 3: Environment variables
    println!("\n3️⃣ Environment Variables");
    println!("-------------------------");
    env_exec_example(&container_id).await;

    // Example 4: Different user
    println!("\n4️⃣ Running as Different User");
    println!("-----------------------------");
    user_exec_example(&container_id).await;

    // Example 5: Working directory
    println!("\n5️⃣ Custom Working Directory");
    println!("----------------------------");
    workdir_exec_example(&container_id).await;

    // Example 6: Detached execution
    println!("\n6️⃣ Detached Execution");
    println!("----------------------");
    detached_exec_example(&container_id).await;

    // Example 7: Privileged execution
    println!("\n7️⃣ Privileged Execution");
    println!("-----------------------");
    privileged_exec_example(&container_id).await;

    // Example 8: Complex command with multiple options
    println!("\n8️⃣ Complex Command Example");
    println!("---------------------------");
    complex_exec_example(&container_id).await;

    // Example 9: Error handling
    println!("\n9️⃣ Error Handling");
    println!("------------------");
    error_handling_example(&container_id).await;

    // Example 10: Command extensibility
    println!("\n🔟 Command Extensibility");
    println!("------------------------");
    extensibility_example();

    // Clean up
    println!("\n🧹 Cleaning up...");
    cleanup_container(&container_id).await;

    println!("\n✅ All examples completed!");
    Ok(())
}

async fn start_test_container() -> Result<String, Box<dyn std::error::Error>> {
    let run_cmd = RunCommand::new("alpine:latest")
        .name("exec-examples-container")
        .detach()
        .cmd(vec!["sleep".to_string(), "300".to_string()]) // 5 minutes
        .remove();

    let container_id = run_cmd.execute().await?;

    // Give the container time to start
    sleep(Duration::from_millis(1000)).await;

    Ok(container_id.as_str().to_string())
}

async fn simple_exec_example(container_id: &str) {
    let exec_cmd = ExecCommand::new(
        container_id,
        vec!["echo".to_string(), "Hello from container!".to_string()],
    );

    match exec_cmd.execute().await {
        Ok(output) => {
            println!("✅ Command executed successfully");
            println!("   Output: {}", output.stdout.trim());
            println!("   Success: {}", output.success());
        }
        Err(e) => {
            println!("❌ Command failed: {e}");
        }
    }
}

async fn interactive_exec_example(container_id: &str) {
    // Note: This demonstrates the command construction, but won't actually be interactive
    // in this example context since we're not connected to a real terminal
    let exec_cmd = ExecCommand::new(
        container_id,
        vec!["echo".to_string(), "Interactive mode".to_string()],
    )
    .interactive()
    .tty();

    match exec_cmd.execute().await {
        Ok(output) => {
            println!("✅ Interactive command configured");
            println!("   Output: {}", output.stdout.trim());
            println!("   (Note: True interactivity requires terminal connection)");
        }
        Err(e) => {
            println!("❌ Interactive command failed: {e}");
        }
    }
}

async fn env_exec_example(container_id: &str) {
    // Single environment variable
    let exec_cmd = ExecCommand::new(
        container_id,
        vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo \"Name: $NAME, Version: $VERSION\"".to_string(),
        ],
    )
    .env("NAME", "Docker Wrapper")
    .env("VERSION", "0.1.0");

    match exec_cmd.execute().await {
        Ok(output) => {
            println!("✅ Environment variables set");
            println!("   Output: {}", output.stdout.trim());
        }
        Err(e) => {
            println!("❌ Environment command failed: {e}");
        }
    }

    // Multiple environment variables from HashMap
    let mut env_vars = HashMap::new();
    env_vars.insert("DEBUG".to_string(), "true".to_string());
    env_vars.insert("LOG_LEVEL".to_string(), "info".to_string());

    let exec_cmd2 = ExecCommand::new(
        container_id,
        vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo \"Debug: $DEBUG, Log Level: $LOG_LEVEL\"".to_string(),
        ],
    )
    .envs(env_vars);

    match exec_cmd2.execute().await {
        Ok(output) => {
            println!("✅ Multiple environment variables set");
            println!("   Output: {}", output.stdout.trim());
        }
        Err(e) => {
            println!("❌ Multiple env command failed: {e}");
        }
    }
}

async fn user_exec_example(container_id: &str) {
    let exec_cmd = ExecCommand::new(container_id, vec!["whoami".to_string()]).user("root");

    match exec_cmd.execute().await {
        Ok(output) => {
            println!("✅ Command executed as specific user");
            println!("   User: {}", output.stdout.trim());
        }
        Err(e) => {
            println!("❌ User command failed: {e}");
        }
    }
}

async fn workdir_exec_example(container_id: &str) {
    let exec_cmd = ExecCommand::new(container_id, vec!["pwd".to_string()]).workdir("/tmp");

    match exec_cmd.execute().await {
        Ok(output) => {
            println!("✅ Command executed in custom working directory");
            println!("   Working directory: {}", output.stdout.trim());
        }
        Err(e) => {
            println!("❌ Workdir command failed: {e}");
        }
    }
}

async fn detached_exec_example(container_id: &str) {
    let exec_cmd =
        ExecCommand::new(container_id, vec!["sleep".to_string(), "2".to_string()]).detach();

    match exec_cmd.execute().await {
        Ok(output) => {
            println!("✅ Detached command executed");
            println!("   Exit code: {}", output.exit_code);
            println!("   (Detached commands return immediately)");
        }
        Err(e) => {
            println!("❌ Detached command failed: {e}");
        }
    }
}

async fn privileged_exec_example(container_id: &str) {
    let exec_cmd = ExecCommand::new(
        container_id,
        vec!["echo".to_string(), "Running with privileges".to_string()],
    )
    .privileged();

    match exec_cmd.execute().await {
        Ok(output) => {
            println!("✅ Privileged command executed");
            println!("   Output: {}", output.stdout.trim());
        }
        Err(e) => {
            println!("❌ Privileged command failed: {e}");
        }
    }
}

async fn complex_exec_example(container_id: &str) {
    let exec_cmd = ExecCommand::new(
        container_id,
        vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo \"Complex: $COMPLEX_VAR, User: $(whoami), Dir: $(pwd)\"".to_string(),
        ],
    )
    .env("COMPLEX_VAR", "test-value")
    .user("root")
    .workdir("/tmp")
    .it(); // Convenience method for interactive + tty

    match exec_cmd.execute().await {
        Ok(output) => {
            println!("✅ Complex command with multiple options");
            println!("   Output: {}", output.stdout.trim());

            // Demonstrate output analysis
            if output.success() {
                println!("   ✅ Command succeeded");
            } else {
                println!("   ❌ Command failed with exit code: {}", output.exit_code);
            }

            if !output.stderr_is_empty() {
                println!("   ⚠️  Stderr: {}", output.stderr.trim());
            }
        }
        Err(e) => {
            println!("❌ Complex command failed: {e}");
        }
    }
}

async fn error_handling_example(container_id: &str) {
    // Intentionally run a command that will fail
    let exec_cmd = ExecCommand::new(container_id, vec!["nonexistent-command".to_string()]);

    match exec_cmd.execute().await {
        Ok(output) => {
            println!("✅ Command executed (checking for errors)");

            if output.success() {
                println!("   ✅ Command succeeded unexpectedly");
            } else {
                println!("   ❌ Command failed as expected");
                println!("   Exit code: {}", output.exit_code);
                if !output.stderr.is_empty() {
                    println!("   Error: {}", output.stderr.trim());
                }
            }
        }
        Err(e) => {
            println!("❌ Command execution failed: {e}");
        }
    }
}

fn extensibility_example() {
    // Demonstrate the extensibility features
    let mut exec_cmd = ExecCommand::new("example-container", vec!["example-command".to_string()]);

    // Use the extensibility methods for hypothetical future options
    exec_cmd.flag("--some-future-flag");
    exec_cmd.option("--some-option", "value");
    exec_cmd.arg("extra-argument");

    let args = exec_cmd.build_command_args();

    println!("✅ Extensibility example - Generated arguments:");
    for (i, arg) in args.iter().enumerate() {
        println!("   [{i}]: {arg}");
    }

    println!("   This demonstrates how the command can be extended with");
    println!("   any Docker exec options not directly supported by the API");
}

async fn cleanup_container(container_id: &str) {
    // Stop the container
    let stop_result = tokio::process::Command::new("docker")
        .args(["stop", container_id])
        .output()
        .await;

    match stop_result {
        Ok(_) => println!("✅ Container stopped"),
        Err(e) => println!("⚠️  Failed to stop container: {e}"),
    }

    // Remove the container (it should auto-remove due to --rm flag, but just in case)
    let rm_result = tokio::process::Command::new("docker")
        .args(["rm", "-f", container_id])
        .output()
        .await;

    match rm_result {
        Ok(_) => println!("✅ Container removed"),
        Err(e) => println!("⚠️  Failed to remove container: {e}"),
    }
}
