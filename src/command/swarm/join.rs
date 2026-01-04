//! Docker swarm join command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Result of swarm join command
#[derive(Debug, Clone)]
pub struct SwarmJoinResult {
    /// Whether the join was successful
    pub success: bool,
    /// Raw output from the command
    pub output: String,
}

impl SwarmJoinResult {
    /// Parse the swarm join output
    fn parse(output: &CommandOutput) -> Self {
        Self {
            success: output.success,
            output: output.stdout.clone(),
        }
    }
}

/// Docker swarm join command builder
#[derive(Debug, Clone)]
pub struct SwarmJoinCommand {
    /// The host:port of the manager to join
    remote_addr: String,
    /// Token for joining the swarm
    token: String,
    /// Advertised address (format: <ip|interface>[:port])
    advertise_addr: Option<String>,
    /// Availability of the node (active, pause, drain)
    availability: Option<String>,
    /// Address or interface to use for data path traffic
    data_path_addr: Option<String>,
    /// Listen address (format: <ip|interface>[:port])
    listen_addr: Option<String>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl SwarmJoinCommand {
    /// Create a new swarm join command
    ///
    /// # Arguments
    /// * `token` - The join token (worker or manager)
    /// * `remote_addr` - The address of a manager node (host:port)
    #[must_use]
    pub fn new(token: impl Into<String>, remote_addr: impl Into<String>) -> Self {
        Self {
            remote_addr: remote_addr.into(),
            token: token.into(),
            advertise_addr: None,
            availability: None,
            data_path_addr: None,
            listen_addr: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Set the advertised address
    #[must_use]
    pub fn advertise_addr(mut self, addr: impl Into<String>) -> Self {
        self.advertise_addr = Some(addr.into());
        self
    }

    /// Set the availability of the node
    #[must_use]
    pub fn availability(mut self, availability: impl Into<String>) -> Self {
        self.availability = Some(availability.into());
        self
    }

    /// Set the data path address
    #[must_use]
    pub fn data_path_addr(mut self, addr: impl Into<String>) -> Self {
        self.data_path_addr = Some(addr.into());
        self
    }

    /// Set the listen address
    #[must_use]
    pub fn listen_addr(mut self, addr: impl Into<String>) -> Self {
        self.listen_addr = Some(addr.into());
        self
    }

    /// Build the command arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["swarm".to_string(), "join".to_string()];

        if let Some(ref addr) = self.advertise_addr {
            args.push("--advertise-addr".to_string());
            args.push(addr.clone());
        }

        if let Some(ref availability) = self.availability {
            args.push("--availability".to_string());
            args.push(availability.clone());
        }

        if let Some(ref addr) = self.data_path_addr {
            args.push("--data-path-addr".to_string());
            args.push(addr.clone());
        }

        if let Some(ref addr) = self.listen_addr {
            args.push("--listen-addr".to_string());
            args.push(addr.clone());
        }

        // Token must come before remote address
        args.push("--token".to_string());
        args.push(self.token.clone());

        // Remote address is the last positional argument
        args.push(self.remote_addr.clone());

        args
    }
}

#[async_trait]
impl DockerCommand for SwarmJoinCommand {
    type Output = SwarmJoinResult;

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
        Ok(SwarmJoinResult::parse(&output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_join_basic() {
        let cmd = SwarmJoinCommand::new("SWMTKN-1-xxx", "192.168.1.1:2377");
        let args = cmd.build_args();
        assert_eq!(args[0], "swarm");
        assert_eq!(args[1], "join");
        assert!(args.contains(&"--token".to_string()));
        assert!(args.contains(&"SWMTKN-1-xxx".to_string()));
        assert!(args.contains(&"192.168.1.1:2377".to_string()));
    }

    #[test]
    fn test_swarm_join_with_options() {
        let cmd = SwarmJoinCommand::new("SWMTKN-1-xxx", "192.168.1.1:2377")
            .advertise_addr("192.168.1.2:2377")
            .availability("active")
            .data_path_addr("192.168.1.2")
            .listen_addr("0.0.0.0:2377");

        let args = cmd.build_args();
        assert!(args.contains(&"--advertise-addr".to_string()));
        assert!(args.contains(&"--availability".to_string()));
        assert!(args.contains(&"--data-path-addr".to_string()));
        assert!(args.contains(&"--listen-addr".to_string()));
    }
}
