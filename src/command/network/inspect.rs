//! Docker network inspect command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::ffi::OsStr;

/// Docker network inspect command builder
#[derive(Debug, Clone)]
pub struct NetworkInspectCommand {
    /// Networks to inspect
    networks: Vec<String>,
    /// Format output
    format: Option<String>,
    /// Include verbose information
    verbose: bool,
    /// Command executor
    executor: CommandExecutor,
}

impl NetworkInspectCommand {
    /// Create a new network inspect command
    #[must_use]
    pub fn new(network: impl Into<String>) -> Self {
        Self {
            networks: vec![network.into()],
            format: None,
            verbose: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Create a new network inspect command for multiple networks
    #[must_use]
    pub fn new_multiple(networks: Vec<String>) -> Self {
        Self {
            networks,
            format: None,
            verbose: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Add a network to inspect
    #[must_use]
    pub fn add_network(mut self, network: impl Into<String>) -> Self {
        self.networks.push(network.into());
        self
    }

    /// Set output format
    #[must_use]
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Include verbose information
    #[must_use]
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    /// Execute the command
    ///
    /// # Errors
    ///
    /// Returns an error if the Docker daemon is not running or the command fails
    pub async fn run(&self) -> Result<NetworkInspectOutput> {
        self.execute().await.map(NetworkInspectOutput::from)
    }
}

#[async_trait]
impl DockerCommand for NetworkInspectCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "network inspect"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["inspect".to_string()];

        if let Some(ref format) = self.format {
            args.push("--format".to_string());
            args.push(format.clone());
        }

        if self.verbose {
            args.push("--verbose".to_string());
        }

        for network in &self.networks {
            args.push(network.clone());
        }

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        self.executor
            .execute_command("network", self.build_args())
            .await
    }

    fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.executor.add_arg(arg);
        self
    }

    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.executor.add_args(args);
        self
    }

    fn flag(&mut self, flag: &str) -> &mut Self {
        self.executor.add_flag(flag);
        self
    }

    fn option(&mut self, key: &str, value: &str) -> &mut Self {
        self.executor.add_option(key, value);
        self
    }
}

/// Output from network inspect command
#[derive(Debug, Clone)]
pub struct NetworkInspectOutput {
    /// JSON data
    pub json: Option<Value>,
    /// Raw command output
    pub raw_output: CommandOutput,
}

impl From<CommandOutput> for NetworkInspectOutput {
    fn from(output: CommandOutput) -> Self {
        let json = if output.stdout.starts_with('[') || output.stdout.starts_with('{') {
            serde_json::from_str(&output.stdout).ok()
        } else {
            None
        };

        Self {
            json,
            raw_output: output,
        }
    }
}

impl NetworkInspectOutput {
    /// Check if the command was successful
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.raw_output.success
    }

    /// Get the JSON output
    #[must_use]
    pub fn json(&self) -> Option<&Value> {
        self.json.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_inspect_single() {
        let cmd = NetworkInspectCommand::new("my-network");
        let args = cmd.build_args();
        assert_eq!(args, vec!["inspect", "my-network"]);
    }

    #[test]
    fn test_network_inspect_multiple() {
        let cmd = NetworkInspectCommand::new_multiple(vec![
            "network1".to_string(),
            "network2".to_string(),
        ]);
        let args = cmd.build_args();
        assert_eq!(args, vec!["inspect", "network1", "network2"]);
    }

    #[test]
    fn test_network_inspect_with_format() {
        let cmd = NetworkInspectCommand::new("my-network").format("{{.Driver}}");
        let args = cmd.build_args();
        assert_eq!(
            args,
            vec!["inspect", "--format", "{{.Driver}}", "my-network"]
        );
    }

    #[test]
    fn test_network_inspect_verbose() {
        let cmd = NetworkInspectCommand::new("my-network").verbose();
        let args = cmd.build_args();
        assert_eq!(args, vec!["inspect", "--verbose", "my-network"]);
    }
}
