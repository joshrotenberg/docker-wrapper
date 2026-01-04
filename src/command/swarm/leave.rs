//! Docker swarm leave command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Result of swarm leave command
#[derive(Debug, Clone)]
pub struct SwarmLeaveResult {
    /// Whether the leave was successful
    pub success: bool,
    /// Raw output from the command
    pub output: String,
}

impl SwarmLeaveResult {
    /// Parse the swarm leave output
    fn parse(output: &CommandOutput) -> Self {
        Self {
            success: output.success,
            output: output.stdout.clone(),
        }
    }
}

/// Docker swarm leave command builder
#[derive(Debug, Clone, Default)]
pub struct SwarmLeaveCommand {
    /// Force leave even if this is the last manager
    force: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl SwarmLeaveCommand {
    /// Create a new swarm leave command
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Force leave even if this is the last manager or cluster will become unavailable
    #[must_use]
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Build the command arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["swarm".to_string(), "leave".to_string()];

        if self.force {
            args.push("--force".to_string());
        }

        args
    }
}

#[async_trait]
impl DockerCommand for SwarmLeaveCommand {
    type Output = SwarmLeaveResult;

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
        Ok(SwarmLeaveResult::parse(&output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_leave_basic() {
        let cmd = SwarmLeaveCommand::new();
        let args = cmd.build_args();
        assert_eq!(args, vec!["swarm", "leave"]);
    }

    #[test]
    fn test_swarm_leave_force() {
        let cmd = SwarmLeaveCommand::new().force();
        let args = cmd.build_args();
        assert!(args.contains(&"--force".to_string()));
    }
}
