//! Examples demonstrating debugging and reliability features.
//!
//! This example shows how to use dry-run mode, retry logic, and verbose debugging.

use docker_wrapper::command::DockerCommandV2;
use docker_wrapper::{
    BackoffStrategy, DebugConfig, DebugExecutor, DryRunPreview, PsCommand, PullCommand,
    RetryPolicy, RunCommand,
};
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("Docker Debugging Features Examples");
    println!("===================================\n");

    // Example 1: Dry-run mode
    println!("1. Dry-run mode - preview commands without executing:");

    let executor = DebugExecutor::new().dry_run(true).verbose(true);

    // This won't actually run the container
    let _result = RunCommand::new("nginx:latest")
        .name("test-nginx")
        .port(8080, 80)
        .detach()
        .execute_with_executor(&executor)
        .await;

    // Show command log
    println!("\nCommand log:");
    for cmd in executor.get_command_log() {
        println!("  - {}", cmd);
    }

    println!();

    // Example 2: Preview multiple operations
    println!("2. Preview multiple operations:");

    let executor = DebugExecutor::new().dry_run(true);

    // Simulate a workflow
    let _ = PullCommand::new("redis:latest")
        .execute_with_executor(&executor)
        .await;

    let _ = RunCommand::new("redis:latest")
        .name("cache")
        .port(6379, 6379)
        .execute_with_executor(&executor)
        .await;

    let _ = PsCommand::new()
        .all()
        .execute_with_executor(&executor)
        .await;

    let preview = DryRunPreview::new(executor.get_command_log());
    preview.print();

    println!();

    // Example 3: Retry with exponential backoff
    println!("3. Retry logic with exponential backoff:");

    let retry_policy = RetryPolicy::new()
        .max_attempts(3)
        .backoff(BackoffStrategy::Exponential {
            initial: Duration::from_millis(100),
            max: Duration::from_secs(5),
            multiplier: 2.0,
        })
        .on_retry(|attempt, error| {
            eprintln!("  Attempt {} failed: {}", attempt, error);
        });

    let executor = DebugExecutor::new().verbose(true).with_retry(retry_policy);

    // This will retry if Docker daemon is temporarily unavailable
    match PsCommand::new().execute_with_executor(&executor).await {
        Ok(_) => println!("  Command succeeded"),
        Err(e) => println!("  Command failed after retries: {}", e),
    }

    println!();

    // Example 4: Verbose mode for debugging
    println!("4. Verbose mode for debugging:");

    let executor = DebugExecutor::new().verbose(true);

    let _ = PsCommand::new()
        .all()
        .execute_with_executor(&executor)
        .await;

    println!();

    // Example 5: Custom retry configuration
    println!("5. Custom retry configuration:");

    // Linear backoff for quick retries
    let retry_policy = RetryPolicy::new()
        .max_attempts(5)
        .backoff(BackoffStrategy::Linear {
            initial: Duration::from_millis(50),
            increment: Duration::from_millis(50),
        });

    let _executor = DebugExecutor::new().with_retry(retry_policy);

    println!("  Using linear backoff: 50ms, 100ms, 150ms, 200ms, 250ms");

    // Fixed delay for consistent timing
    let retry_policy = RetryPolicy::new()
        .max_attempts(3)
        .backoff(BackoffStrategy::Fixed(Duration::from_secs(1)));

    let _executor = DebugExecutor::new().with_retry(retry_policy);

    println!("  Using fixed delay: 1s between each attempt");

    println!();

    // Example 6: Debug configuration
    println!("6. Debug configuration:");

    let debug_config = DebugConfig::new()
        .dry_run(true)
        .verbose(true)
        .dry_run_prefix("[PREVIEW]");

    let executor = DebugExecutor::new()
        .dry_run(debug_config.dry_run)
        .verbose(debug_config.verbose);

    let _ = RunCommand::new("alpine")
        .cmd(vec!["echo".to_string(), "Hello".to_string()])
        .execute_with_executor(&executor)
        .await;
}

// Helper trait to execute commands with a custom executor
// (In a real implementation, this would be integrated into the commands)
trait ExecuteWithExecutor {
    async fn execute_with_executor(&self, executor: &DebugExecutor) -> docker_wrapper::Result<()>;
}

impl ExecuteWithExecutor for RunCommand {
    async fn execute_with_executor(&self, executor: &DebugExecutor) -> docker_wrapper::Result<()> {
        let args = self.build_command_args();
        let _ = executor.execute_command("run", args).await?;
        Ok(())
    }
}

impl ExecuteWithExecutor for PullCommand {
    async fn execute_with_executor(&self, executor: &DebugExecutor) -> docker_wrapper::Result<()> {
        // PullCommand doesn't implement DockerCommandV2 yet, so create args manually
        let mut args = vec!["pull".to_string()];
        args.push(self.get_image().to_string());
        let _ = executor.execute_command("pull", args).await?;
        Ok(())
    }
}

impl ExecuteWithExecutor for PsCommand {
    async fn execute_with_executor(&self, executor: &DebugExecutor) -> docker_wrapper::Result<()> {
        // PsCommand doesn't implement DockerCommandV2 yet, so create args manually
        let args = vec!["ps".to_string()];
        let _ = executor.execute_command("ps", args).await?;
        Ok(())
    }
}
