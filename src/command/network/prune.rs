//! Docker network prune command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// Docker network prune command builder
#[derive(Debug, Clone)]
pub struct NetworkPruneCommand {
    /// Remove networks created before given timestamp
    until: Option<String>,
    /// Filter values
    filters: HashMap<String, String>,
    /// Do not prompt for confirmation
    force: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl NetworkPruneCommand {
    /// Create a new network prune command
    #[must_use]
    pub fn new() -> Self {
        Self {
            until: None,
            filters: HashMap::new(),
            force: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Remove networks created before given timestamp
    #[must_use]
    pub fn until(mut self, timestamp: impl Into<String>) -> Self {
        self.until = Some(timestamp.into());
        self
    }

    /// Add a filter
    #[must_use]
    pub fn filter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.filters.insert(key.into(), value.into());
        self
    }

    /// Filter by label
    #[must_use]
    pub fn label_filter(self, label: impl Into<String>) -> Self {
        self.filter("label", label)
    }

    /// Do not prompt for confirmation
    #[must_use]
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Execute the command
    ///
    /// # Errors
    ///
    /// Returns an error if the Docker daemon is not running or the command fails
    pub async fn run(&self) -> Result<NetworkPruneResult> {
        self.execute().await.map(NetworkPruneResult::from)
    }
}

impl Default for NetworkPruneCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for NetworkPruneCommand {
    type Output = CommandOutput;

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["network".to_string(), "prune".to_string()];

        if let Some(ref until) = self.until {
            args.push("--filter".to_string());
            args.push(format!("until={until}"));
        }

        for (key, value) in &self.filters {
            args.push("--filter".to_string());
            args.push(format!("{key}={value}"));
        }

        if self.force {
            args.push("--force".to_string());
        }

        args.extend(self.executor.raw_args.clone());
        args
    }

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
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

/// Result from network prune command
#[derive(Debug, Clone)]
pub struct NetworkPruneResult {
    /// Deleted networks
    pub deleted_networks: Vec<String>,
    /// Space reclaimed in bytes
    pub space_reclaimed: Option<u64>,
    /// Raw command output
    pub raw_output: CommandOutput,
}

impl From<CommandOutput> for NetworkPruneResult {
    fn from(output: CommandOutput) -> Self {
        let mut deleted_networks = Vec::new();
        let mut space_reclaimed = None;

        for line in output.stdout.lines() {
            if line.starts_with("Deleted Networks:") {
                continue;
            }
            if line.contains("Total reclaimed space:") {
                // Parse space from line like "Total reclaimed space: 1.234MB"
                if let Some(space_str) = line.split(':').nth(1) {
                    space_reclaimed = parse_size(space_str.trim());
                }
            } else if !line.trim().is_empty() && !line.contains("WARNING") {
                deleted_networks.push(line.trim().to_string());
            }
        }

        Self {
            deleted_networks,
            space_reclaimed,
            raw_output: output,
        }
    }
}

impl NetworkPruneResult {
    /// Check if the command was successful
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.raw_output.success
    }

    /// Get count of deleted networks
    #[must_use]
    pub fn count(&self) -> usize {
        self.deleted_networks.len()
    }
}

fn parse_size(size_str: &str) -> Option<u64> {
    // Simple parser for Docker size strings (e.g., "1.234MB", "567KB", "2GB")
    let size_str = size_str.trim();
    if size_str == "0B" {
        return Some(0);
    }

    let (num_part, unit_part) = size_str.split_at(
        size_str
            .rfind(|c: char| c.is_ascii_digit() || c == '.')
            .map_or(0, |i| i + 1),
    );

    let number: f64 = num_part.parse().ok()?;
    let multiplier = match unit_part.to_uppercase().as_str() {
        "B" => 1,
        "KB" => 1_000,
        "MB" => 1_000_000,
        "GB" => 1_000_000_000,
        _ => return None,
    };

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Some((number * f64::from(multiplier)) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_prune_basic() {
        let cmd = NetworkPruneCommand::new();
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["network", "prune"]);
    }

    #[test]
    fn test_network_prune_force() {
        let cmd = NetworkPruneCommand::new().force();
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["network", "prune", "--force"]);
    }

    #[test]
    fn test_network_prune_with_filters() {
        let cmd = NetworkPruneCommand::new()
            .until("24h")
            .label_filter("env=test");
        let args = cmd.build_command_args();
        assert!(args.contains(&"--filter".to_string()));
        assert!(args.iter().any(|a| a.contains("until=24h")));
        assert!(args.iter().any(|a| a.contains("label=env=test")));
    }

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("0B"), Some(0));
        assert_eq!(parse_size("100B"), Some(100));
        assert_eq!(parse_size("1.5KB"), Some(1_500));
        assert_eq!(parse_size("2MB"), Some(2_000_000));
        assert_eq!(parse_size("1.234GB"), Some(1_234_000_000));
    }
}
