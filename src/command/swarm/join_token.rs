//! Docker swarm join-token command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Node role for join token retrieval
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwarmNodeRole {
    /// Worker node role
    Worker,
    /// Manager node role
    Manager,
}

impl SwarmNodeRole {
    /// Get the role as a string
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Worker => "worker",
            Self::Manager => "manager",
        }
    }
}

impl std::fmt::Display for SwarmNodeRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Result of swarm join-token command
#[derive(Debug, Clone)]
pub struct SwarmJoinTokenResult {
    /// The join token
    pub token: Option<String>,
    /// The full join command (if not using --quiet)
    pub join_command: Option<String>,
    /// The role for which the token was retrieved
    pub role: SwarmNodeRole,
    /// Raw output from the command
    pub output: String,
}

impl SwarmJoinTokenResult {
    /// Parse the swarm join-token output
    fn parse(output: &CommandOutput, role: SwarmNodeRole, quiet: bool) -> Self {
        let stdout = output.stdout.trim();

        if quiet {
            // In quiet mode, output is just the token
            Self {
                token: Some(stdout.to_string()),
                join_command: None,
                role,
                output: stdout.to_string(),
            }
        } else {
            // Normal mode: parse the join command
            // Output format:
            // To add a worker to this swarm, run the following command:
            //
            //     docker swarm join --token SWMTKN-... 192.168.1.1:2377

            let mut token = None;
            let mut join_command = None;

            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("docker swarm join") {
                    join_command = Some(trimmed.to_string());

                    // Extract token from the join command
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    for (i, part) in parts.iter().enumerate() {
                        if *part == "--token" {
                            if let Some(t) = parts.get(i + 1) {
                                token = Some((*t).to_string());
                            }
                        }
                    }
                }
            }

            Self {
                token,
                join_command,
                role,
                output: stdout.to_string(),
            }
        }
    }
}

/// Docker swarm join-token command builder
///
/// Retrieves or rotates the join token for a swarm.
#[derive(Debug, Clone)]
pub struct SwarmJoinTokenCommand {
    /// The role (worker or manager)
    role: SwarmNodeRole,
    /// Only display the token (no join command)
    quiet: bool,
    /// Rotate the join token
    rotate: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl SwarmJoinTokenCommand {
    /// Create a new swarm join-token command for the specified role
    #[must_use]
    pub fn new(role: SwarmNodeRole) -> Self {
        Self {
            role,
            quiet: false,
            rotate: false,
            executor: CommandExecutor::default(),
        }
    }

    /// Create a command to get the worker join token
    #[must_use]
    pub fn worker() -> Self {
        Self::new(SwarmNodeRole::Worker)
    }

    /// Create a command to get the manager join token
    #[must_use]
    pub fn manager() -> Self {
        Self::new(SwarmNodeRole::Manager)
    }

    /// Only display the token (no join command)
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// Rotate the join token
    #[must_use]
    pub fn rotate(mut self) -> Self {
        self.rotate = true;
        self
    }

    /// Build the command arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["swarm".to_string(), "join-token".to_string()];

        if self.quiet {
            args.push("--quiet".to_string());
        }

        if self.rotate {
            args.push("--rotate".to_string());
        }

        args.push(self.role.as_str().to_string());

        args
    }
}

impl Default for SwarmJoinTokenCommand {
    fn default() -> Self {
        Self::worker()
    }
}

#[async_trait]
impl DockerCommand for SwarmJoinTokenCommand {
    type Output = SwarmJoinTokenResult;

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
        Ok(SwarmJoinTokenResult::parse(&output, self.role, self.quiet))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_token_worker() {
        let cmd = SwarmJoinTokenCommand::worker();
        let args = cmd.build_args();
        assert_eq!(args, vec!["swarm", "join-token", "worker"]);
    }

    #[test]
    fn test_join_token_manager() {
        let cmd = SwarmJoinTokenCommand::manager();
        let args = cmd.build_args();
        assert_eq!(args, vec!["swarm", "join-token", "manager"]);
    }

    #[test]
    fn test_join_token_quiet() {
        let cmd = SwarmJoinTokenCommand::worker().quiet();
        let args = cmd.build_args();
        assert!(args.contains(&"--quiet".to_string()));
        assert!(args.contains(&"worker".to_string()));
    }

    #[test]
    fn test_join_token_rotate() {
        let cmd = SwarmJoinTokenCommand::manager().rotate();
        let args = cmd.build_args();
        assert!(args.contains(&"--rotate".to_string()));
        assert!(args.contains(&"manager".to_string()));
    }

    #[test]
    fn test_join_token_all_options() {
        let cmd = SwarmJoinTokenCommand::worker().quiet().rotate();
        let args = cmd.build_args();
        assert_eq!(
            args,
            vec!["swarm", "join-token", "--quiet", "--rotate", "worker"]
        );
    }

    #[test]
    fn test_node_role_display() {
        assert_eq!(SwarmNodeRole::Worker.to_string(), "worker");
        assert_eq!(SwarmNodeRole::Manager.to_string(), "manager");
    }
}
