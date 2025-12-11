//! Docker network ls command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Docker network ls command builder
#[derive(Debug, Clone)]
pub struct NetworkLsCommand {
    /// Filter output
    filters: HashMap<String, String>,
    /// Format output
    format: Option<String>,
    /// Don't truncate output
    no_trunc: bool,
    /// Only show IDs
    quiet: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl NetworkLsCommand {
    /// Create a new network ls command
    #[must_use]
    pub fn new() -> Self {
        Self {
            filters: HashMap::new(),
            format: None,
            no_trunc: false,
            quiet: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Add a filter
    #[must_use]
    pub fn filter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.filters.insert(key.into(), value.into());
        self
    }

    /// Filter by driver
    #[must_use]
    pub fn driver_filter(self, driver: impl Into<String>) -> Self {
        self.filter("driver", driver)
    }

    /// Filter by ID
    #[must_use]
    pub fn id_filter(self, id: impl Into<String>) -> Self {
        self.filter("id", id)
    }

    /// Filter by label
    #[must_use]
    pub fn label_filter(self, label: impl Into<String>) -> Self {
        self.filter("label", label)
    }

    /// Filter by name
    #[must_use]
    pub fn name_filter(self, name: impl Into<String>) -> Self {
        self.filter("name", name)
    }

    /// Filter by scope
    #[must_use]
    pub fn scope_filter(self, scope: impl Into<String>) -> Self {
        self.filter("scope", scope)
    }

    /// Filter by type (custom or builtin)
    #[must_use]
    pub fn type_filter(self, typ: impl Into<String>) -> Self {
        self.filter("type", typ)
    }

    /// Set output format
    #[must_use]
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Format output as JSON
    #[must_use]
    pub fn format_json(self) -> Self {
        self.format("json")
    }

    /// Don't truncate output
    #[must_use]
    pub fn no_trunc(mut self) -> Self {
        self.no_trunc = true;
        self
    }

    /// Only display network IDs
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// Execute the command
    ///
    /// # Errors
    ///
    /// Returns an error if the Docker daemon is not running or the command fails
    pub async fn run(&self) -> Result<NetworkLsOutput> {
        self.execute().await.map(NetworkLsOutput::from)
    }
}

impl Default for NetworkLsCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for NetworkLsCommand {
    type Output = CommandOutput;

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["network".to_string(), "ls".to_string()];

        for (key, value) in &self.filters {
            args.push("--filter".to_string());
            args.push(format!("{key}={value}"));
        }

        if let Some(ref format) = self.format {
            args.push("--format".to_string());
            args.push(format.clone());
        }

        if self.no_trunc {
            args.push("--no-trunc".to_string());
        }

        if self.quiet {
            args.push("--quiet".to_string());
        }

        args.extend(self.executor.raw_args.clone());
        args
    }

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        let command_name = args[0].clone();
        let command_args = args[1..].to_vec();
        self.executor
            .execute_command(&command_name, command_args)
            .await
    }
}

/// Information about a Docker network
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct NetworkInfo {
    /// Network ID
    #[serde(rename = "ID", default)]
    pub id: String,
    /// Network name
    #[serde(default)]
    pub name: String,
    /// Network driver
    #[serde(default)]
    pub driver: String,
    /// Network scope
    #[serde(default)]
    pub scope: String,
    /// IPv6 enabled
    #[serde(rename = "IPv6", default)]
    pub ipv6: bool,
    /// Internal network
    #[serde(default)]
    pub internal: bool,
    /// Attachable network
    #[serde(default)]
    pub attachable: bool,
    /// Ingress network
    #[serde(default)]
    pub ingress: bool,
    /// Creation time
    #[serde(rename = "CreatedAt", default)]
    pub created_at: String,
    /// Labels
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

/// Output from network ls command
#[derive(Debug, Clone)]
pub struct NetworkLsOutput {
    /// List of networks
    pub networks: Vec<NetworkInfo>,
    /// Raw command output
    pub raw_output: CommandOutput,
}

impl From<CommandOutput> for NetworkLsOutput {
    fn from(output: CommandOutput) -> Self {
        let networks = if output.stdout.starts_with('[') {
            // JSON format
            serde_json::from_str(&output.stdout).unwrap_or_default()
        } else if output.stdout.trim().is_empty() {
            vec![]
        } else {
            // Parse table format
            parse_table_output(&output.stdout)
        };

        Self {
            networks,
            raw_output: output,
        }
    }
}

impl NetworkLsOutput {
    /// Check if the command was successful
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.raw_output.success
    }

    /// Get network count
    #[must_use]
    pub fn count(&self) -> usize {
        self.networks.len()
    }

    /// Check if any networks exist
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.networks.is_empty()
    }

    /// Get network by name
    #[must_use]
    pub fn get_network(&self, name: &str) -> Option<&NetworkInfo> {
        self.networks.iter().find(|n| n.name == name)
    }

    /// Get network IDs
    #[must_use]
    pub fn ids(&self) -> Vec<String> {
        self.networks.iter().map(|n| n.id.clone()).collect()
    }
}

fn parse_table_output(output: &str) -> Vec<NetworkInfo> {
    let mut networks = Vec::new();
    let lines: Vec<&str> = output.lines().collect();

    if lines.len() <= 1 {
        return networks;
    }

    // Skip header line
    for line in lines.iter().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            networks.push(NetworkInfo {
                id: parts[0].to_string(),
                name: parts[1].to_string(),
                driver: parts[2].to_string(),
                scope: parts[3].to_string(),
                ipv6: false,
                internal: false,
                attachable: false,
                ingress: false,
                created_at: String::new(),
                labels: HashMap::new(),
            });
        }
    }

    networks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_ls_basic() {
        let cmd = NetworkLsCommand::new();
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["network", "ls"]);
    }

    #[test]
    fn test_network_ls_with_filters() {
        let cmd = NetworkLsCommand::new()
            .driver_filter("bridge")
            .name_filter("my-network");
        let args = cmd.build_command_args();
        assert!(args.contains(&"--filter".to_string()));
        assert!(args.iter().any(|a| a.contains("driver=bridge")));
        assert!(args.iter().any(|a| a.contains("name=my-network")));
    }

    #[test]
    fn test_network_ls_with_format() {
        let cmd = NetworkLsCommand::new().format_json();
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["network", "ls", "--format", "json"]);
    }

    #[test]
    fn test_network_ls_quiet() {
        let cmd = NetworkLsCommand::new().quiet();
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["network", "ls", "--quiet"]);
    }

    #[test]
    fn test_parse_table_output() {
        let output = "NETWORK ID     NAME      DRIVER    SCOPE
f2de39df4171   bridge    bridge    local
9fb1e39c5d12   host      host      local
94b82a6c5b45   none      null      local";

        let networks = parse_table_output(output);
        assert_eq!(networks.len(), 3);
        assert_eq!(networks[0].name, "bridge");
        assert_eq!(networks[1].name, "host");
        assert_eq!(networks[2].name, "none");
    }
}
