//! Debugging and reliability features for Docker commands.

use crate::command::{CommandExecutor, CommandOutput};
use crate::error::Result;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

/// Configuration for dry-run mode and debugging
#[derive(Debug, Clone)]
pub struct DebugConfig {
    /// Enable dry-run mode (commands are not executed)
    pub dry_run: bool,

    /// Enable verbose output
    pub verbose: bool,

    /// Log all commands to this vector
    pub command_log: Arc<Mutex<Vec<String>>>,

    /// Custom prefix for dry-run output
    pub dry_run_prefix: String,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            dry_run: false,
            verbose: false,
            command_log: Arc::new(Mutex::new(Vec::new())),
            dry_run_prefix: "[DRY RUN]".to_string(),
        }
    }
}

impl DebugConfig {
    /// Create a new debug configuration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable dry-run mode
    #[must_use]
    pub fn dry_run(mut self, enabled: bool) -> Self {
        self.dry_run = enabled;
        self
    }

    /// Enable verbose output
    #[must_use]
    pub fn verbose(mut self, enabled: bool) -> Self {
        self.verbose = enabled;
        self
    }

    /// Set custom dry-run prefix
    #[must_use]
    pub fn dry_run_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.dry_run_prefix = prefix.into();
        self
    }

    /// Log a command
    pub fn log_command(&self, command: &str) {
        if let Ok(mut log) = self.command_log.lock() {
            log.push(command.to_string());
        }
    }

    /// Get logged commands
    #[must_use]
    pub fn get_command_log(&self) -> Vec<String> {
        self.command_log
            .lock()
            .map(|log| log.clone())
            .unwrap_or_default()
    }

    /// Clear command log
    pub fn clear_log(&self) {
        if let Ok(mut log) = self.command_log.lock() {
            log.clear();
        }
    }
}

/// Type alias for retry callback function
pub type RetryCallback = Arc<dyn Fn(u32, &str) + Send + Sync>;

/// Retry policy for handling transient failures
#[derive(Clone)]
pub struct RetryPolicy {
    /// Maximum number of attempts
    pub max_attempts: u32,

    /// Backoff strategy between retries
    pub backoff: BackoffStrategy,

    /// Callback for retry events
    pub on_retry: Option<RetryCallback>,
}

impl std::fmt::Debug for RetryPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetryPolicy")
            .field("max_attempts", &self.max_attempts)
            .field("backoff", &self.backoff)
            .field("on_retry", &self.on_retry.is_some())
            .finish()
    }
}

/// Backoff strategy for retries
#[derive(Debug, Clone)]
pub enum BackoffStrategy {
    /// Fixed delay between retries
    Fixed(Duration),

    /// Linear increase in delay
    Linear {
        /// Initial delay duration
        initial: Duration,
        /// Increment added for each retry
        increment: Duration,
    },

    /// Exponential backoff
    Exponential {
        /// Initial delay duration
        initial: Duration,
        /// Maximum delay duration
        max: Duration,
        /// Multiplier for exponential growth
        multiplier: f64,
    },
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff: BackoffStrategy::Exponential {
                initial: Duration::from_millis(100),
                max: Duration::from_secs(10),
                multiplier: 2.0,
            },
            on_retry: None,
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum attempts
    #[must_use]
    pub fn max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// Set backoff strategy
    #[must_use]
    pub fn backoff(mut self, strategy: BackoffStrategy) -> Self {
        self.backoff = strategy;
        self
    }

    /// Set retry callback
    #[must_use]
    pub fn on_retry<F>(mut self, callback: F) -> Self
    where
        F: Fn(u32, &str) + Send + Sync + 'static,
    {
        self.on_retry = Some(Arc::new(callback));
        self
    }

    /// Calculate delay for attempt number
    #[must_use]
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        match &self.backoff {
            BackoffStrategy::Fixed(delay) => *delay,

            BackoffStrategy::Linear { initial, increment } => {
                *initial + (*increment * (attempt - 1))
            }

            BackoffStrategy::Exponential {
                initial,
                max,
                multiplier,
            } => {
                #[allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]
                let delay_ms =
                    initial.as_millis() as f64 * multiplier.powi(attempt.saturating_sub(1) as i32);
                #[allow(clippy::cast_precision_loss)]
                let capped_ms = delay_ms.min(max.as_millis() as f64);
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                Duration::from_millis(capped_ms as u64)
            }
        }
    }

    /// Check if an error is retryable
    #[must_use]
    pub fn is_retryable(error_str: &str) -> bool {
        // Common retryable Docker errors
        error_str.contains("connection refused")
            || error_str.contains("timeout")
            || error_str.contains("temporarily unavailable")
            || error_str.contains("resource temporarily unavailable")
            || error_str.contains("Cannot connect to the Docker daemon")
            || error_str.contains("503 Service Unavailable")
            || error_str.contains("502 Bad Gateway")
    }
}

/// Enhanced command executor with debugging features
#[derive(Debug, Clone)]
pub struct DebugExecutor {
    /// Base executor
    pub executor: CommandExecutor,

    /// Debug configuration
    pub debug_config: DebugConfig,

    /// Retry policy
    pub retry_policy: Option<RetryPolicy>,
}

impl DebugExecutor {
    /// Create a new debug executor
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            debug_config: DebugConfig::default(),
            retry_policy: None,
        }
    }

    /// Enable dry-run mode
    #[must_use]
    pub fn dry_run(mut self, enabled: bool) -> Self {
        self.debug_config = self.debug_config.dry_run(enabled);
        self
    }

    /// Enable verbose mode
    #[must_use]
    pub fn verbose(mut self, enabled: bool) -> Self {
        self.debug_config = self.debug_config.verbose(enabled);
        self
    }

    /// Set retry policy
    #[must_use]
    pub fn with_retry(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = Some(policy);
        self
    }

    /// Execute a command with debug features
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails after all retry attempts
    pub async fn execute_command(
        &self,
        command_name: &str,
        args: Vec<String>,
    ) -> Result<CommandOutput> {
        let command_str = format!("docker {} {}", command_name, args.join(" "));

        // Log the command
        self.debug_config.log_command(&command_str);

        // Verbose output
        if self.debug_config.verbose {
            eprintln!("[VERBOSE] Executing: {command_str}");
        }

        // Dry-run mode
        if self.debug_config.dry_run {
            let message = format!(
                "{} Would execute: {}",
                self.debug_config.dry_run_prefix, command_str
            );
            eprintln!("{message}");

            return Ok(CommandOutput {
                stdout: message,
                stderr: String::new(),
                exit_code: 0,
                success: true,
            });
        }

        // Execute with retry if configured
        if let Some(ref policy) = self.retry_policy {
            self.execute_with_retry(command_name, args, policy).await
        } else {
            self.executor.execute_command(command_name, args).await
        }
    }

    /// Execute command with retry logic
    async fn execute_with_retry(
        &self,
        command_name: &str,
        args: Vec<String>,
        policy: &RetryPolicy,
    ) -> Result<CommandOutput> {
        let mut attempt = 0;
        let mut last_error = None;

        while attempt < policy.max_attempts {
            attempt += 1;

            if self.debug_config.verbose && attempt > 1 {
                eprintln!(
                    "[VERBOSE] Retry attempt {}/{}",
                    attempt, policy.max_attempts
                );
            }

            match self
                .executor
                .execute_command(command_name, args.clone())
                .await
            {
                Ok(output) => return Ok(output),
                Err(e) => {
                    let error_str = e.to_string();

                    // Check if retryable
                    if !RetryPolicy::is_retryable(&error_str) {
                        return Err(e);
                    }

                    // Last attempt?
                    if attempt >= policy.max_attempts {
                        return Err(e);
                    }

                    // Call retry callback
                    if let Some(ref callback) = policy.on_retry {
                        callback(attempt, &error_str);
                    }

                    // Calculate and apply delay
                    let delay = policy.calculate_delay(attempt);
                    if self.debug_config.verbose {
                        eprintln!("[VERBOSE] Waiting {delay:?} before retry");
                    }
                    sleep(delay).await;

                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| crate::error::Error::custom("Retry failed")))
    }

    /// Get command history
    #[must_use]
    pub fn get_command_log(&self) -> Vec<String> {
        self.debug_config.get_command_log()
    }

    /// Clear command history
    pub fn clear_log(&self) {
        self.debug_config.clear_log();
    }
}

impl Default for DebugExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for previewing commands without executing them
pub struct DryRunPreview {
    /// Commands that would be executed
    pub commands: Vec<String>,
}

impl DryRunPreview {
    /// Create a preview of commands
    #[must_use]
    pub fn new(commands: Vec<String>) -> Self {
        Self { commands }
    }

    /// Print the preview
    pub fn print(&self) {
        if self.commands.is_empty() {
            println!("No commands would be executed.");
            return;
        }

        println!("Would execute the following commands:");
        for (i, cmd) in self.commands.iter().enumerate() {
            println!("  {}. {}", i + 1, cmd);
        }
    }
}

impl std::fmt::Display for DryRunPreview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.commands.is_empty() {
            return write!(f, "No commands would be executed.");
        }

        writeln!(f, "Would execute the following commands:")?;
        for (i, cmd) in self.commands.iter().enumerate() {
            writeln!(f, "  {}. {}", i + 1, cmd)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_config() {
        let config = DebugConfig::new()
            .dry_run(true)
            .verbose(true)
            .dry_run_prefix("[TEST]");

        assert!(config.dry_run);
        assert!(config.verbose);
        assert_eq!(config.dry_run_prefix, "[TEST]");
    }

    #[test]
    fn test_retry_policy_delay() {
        // Fixed backoff
        let policy = RetryPolicy::new().backoff(BackoffStrategy::Fixed(Duration::from_millis(100)));
        assert_eq!(policy.calculate_delay(1), Duration::from_millis(100));
        assert_eq!(policy.calculate_delay(3), Duration::from_millis(100));

        // Linear backoff
        let policy = RetryPolicy::new().backoff(BackoffStrategy::Linear {
            initial: Duration::from_millis(100),
            increment: Duration::from_millis(50),
        });
        assert_eq!(policy.calculate_delay(1), Duration::from_millis(100));
        assert_eq!(policy.calculate_delay(2), Duration::from_millis(150));
        assert_eq!(policy.calculate_delay(3), Duration::from_millis(200));

        // Exponential backoff
        let policy = RetryPolicy::new().backoff(BackoffStrategy::Exponential {
            initial: Duration::from_millis(100),
            max: Duration::from_secs(1),
            multiplier: 2.0,
        });
        assert_eq!(policy.calculate_delay(1), Duration::from_millis(100));
        assert_eq!(policy.calculate_delay(2), Duration::from_millis(200));
        assert_eq!(policy.calculate_delay(3), Duration::from_millis(400));
        assert_eq!(policy.calculate_delay(5), Duration::from_secs(1)); // Capped at max
    }

    #[test]
    fn test_retryable_errors() {
        assert!(RetryPolicy::is_retryable("connection refused"));
        assert!(RetryPolicy::is_retryable("operation timeout"));
        assert!(RetryPolicy::is_retryable(
            "Cannot connect to the Docker daemon"
        ));
        assert!(!RetryPolicy::is_retryable("image not found"));
        assert!(!RetryPolicy::is_retryable("permission denied"));
    }

    #[test]
    fn test_command_logging() {
        let config = DebugConfig::new();
        config.log_command("docker ps -a");
        config.log_command("docker run nginx");

        let log = config.get_command_log();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0], "docker ps -a");
        assert_eq!(log[1], "docker run nginx");

        config.clear_log();
        assert_eq!(config.get_command_log().len(), 0);
    }

    #[test]
    fn test_dry_run_preview() {
        let commands = vec![
            "docker pull nginx".to_string(),
            "docker run -d nginx".to_string(),
        ];

        let preview = DryRunPreview::new(commands);
        let output = preview.to_string();

        assert!(output.contains("Would execute"));
        assert!(output.contains("1. docker pull nginx"));
        assert!(output.contains("2. docker run -d nginx"));
    }
}
