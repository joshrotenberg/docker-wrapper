//! Docker swarm unlock-key command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Result of swarm unlock-key command
#[derive(Debug, Clone)]
pub struct SwarmUnlockKeyResult {
    /// The unlock key
    pub key: Option<String>,
    /// Raw output from the command
    pub output: String,
}

impl SwarmUnlockKeyResult {
    /// Parse the swarm unlock-key output
    fn parse(output: &CommandOutput, quiet: bool) -> Self {
        let stdout = output.stdout.trim();

        let key = if quiet {
            // In quiet mode, output is just the key
            Some(stdout.to_string())
        } else {
            // Normal mode: parse the key from output
            // Output format:
            // To unlock a swarm manager after it restarts, run the `docker swarm unlock`
            // command and provide the following key:
            //
            //     SWMKEY-1-...

            let mut found_key = None;
            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("SWMKEY-") {
                    found_key = Some(trimmed.to_string());
                    break;
                }
            }
            found_key
        };

        Self {
            key,
            output: stdout.to_string(),
        }
    }
}

/// Docker swarm unlock-key command builder
///
/// Manage the unlock key for a locked swarm.
#[derive(Debug, Clone, Default)]
pub struct SwarmUnlockKeyCommand {
    /// Only display the key (no instructions)
    quiet: bool,
    /// Rotate the unlock key
    rotate: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl SwarmUnlockKeyCommand {
    /// Create a new swarm unlock-key command
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Only display the key (no instructions)
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// Rotate the unlock key
    #[must_use]
    pub fn rotate(mut self) -> Self {
        self.rotate = true;
        self
    }

    /// Build the command arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["swarm".to_string(), "unlock-key".to_string()];

        if self.quiet {
            args.push("--quiet".to_string());
        }

        if self.rotate {
            args.push("--rotate".to_string());
        }

        args
    }
}

#[async_trait]
impl DockerCommand for SwarmUnlockKeyCommand {
    type Output = SwarmUnlockKeyResult;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        self.build_args()
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_args();
        let output = self.execute_command(args).await?;
        Ok(SwarmUnlockKeyResult::parse(&output, self.quiet))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_unlock_key_basic() {
        let cmd = SwarmUnlockKeyCommand::new();
        let args = cmd.build_args();
        assert_eq!(args, vec!["swarm", "unlock-key"]);
    }

    #[test]
    fn test_swarm_unlock_key_quiet() {
        let cmd = SwarmUnlockKeyCommand::new().quiet();
        let args = cmd.build_args();
        assert!(args.contains(&"--quiet".to_string()));
    }

    #[test]
    fn test_swarm_unlock_key_rotate() {
        let cmd = SwarmUnlockKeyCommand::new().rotate();
        let args = cmd.build_args();
        assert!(args.contains(&"--rotate".to_string()));
    }

    #[test]
    fn test_swarm_unlock_key_all_options() {
        let cmd = SwarmUnlockKeyCommand::new().quiet().rotate();
        let args = cmd.build_args();
        assert_eq!(args, vec!["swarm", "unlock-key", "--quiet", "--rotate"]);
    }
}
