//! Docker swarm unlock command implementation.

use crate::command::{CommandExecutor, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Result of swarm unlock command
#[derive(Debug, Clone)]
pub struct SwarmUnlockResult {
    /// Whether the unlock was successful
    pub success: bool,
    /// Raw output from the command
    pub output: String,
}

/// Docker swarm unlock command builder
///
/// Unlock a locked swarm manager node.
#[derive(Debug, Clone, Default)]
pub struct SwarmUnlockCommand {
    /// Command executor
    pub executor: CommandExecutor,
}

impl SwarmUnlockCommand {
    /// Create a new swarm unlock command
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the command arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["swarm".to_string(), "unlock".to_string()];
        args.extend(self.executor.raw_args.clone());
        args
    }
}

#[async_trait]
impl DockerCommand for SwarmUnlockCommand {
    type Output = SwarmUnlockResult;

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
        Ok(SwarmUnlockResult {
            success: true,
            output: output.stdout,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_unlock_basic() {
        let cmd = SwarmUnlockCommand::new();
        let args = cmd.build_args();
        assert_eq!(args, vec!["swarm", "unlock"]);
    }
}
