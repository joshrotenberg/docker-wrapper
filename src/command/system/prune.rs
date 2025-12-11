//! Docker system prune command implementation.

use crate::command::{CommandExecutor, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;

/// Result from system prune operation
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PruneResult {
    /// List of deleted container IDs
    #[serde(default)]
    pub containers_deleted: Vec<String>,

    /// Space reclaimed from containers in bytes
    #[serde(default)]
    pub containers_space_reclaimed: u64,

    /// List of deleted image IDs
    #[serde(default)]
    pub images_deleted: Vec<String>,

    /// Space reclaimed from images in bytes
    #[serde(default)]
    pub images_space_reclaimed: u64,

    /// List of deleted network IDs
    #[serde(default)]
    pub networks_deleted: Vec<String>,

    /// List of deleted volume names
    #[serde(default)]
    pub volumes_deleted: Vec<String>,

    /// Space reclaimed from volumes in bytes
    #[serde(default)]
    pub volumes_space_reclaimed: u64,

    /// Build cache entries deleted
    #[serde(default)]
    pub build_cache_deleted: Vec<String>,

    /// Total space reclaimed in bytes
    #[serde(default)]
    pub space_reclaimed: u64,
}

/// Docker system prune command
///
/// Remove unused data including:
/// - All stopped containers
/// - All networks not used by at least one container
/// - All dangling images
/// - All dangling build cache
/// - Optionally all volumes not used by at least one container
#[derive(Debug, Clone)]
pub struct SystemPruneCommand {
    /// Remove all unused images, not just dangling ones
    all: bool,

    /// Prune volumes
    volumes: bool,

    /// Do not prompt for confirmation
    force: bool,

    /// Provide filter values
    filter: HashMap<String, String>,

    /// Command executor
    pub executor: CommandExecutor,
}

impl SystemPruneCommand {
    /// Create a new system prune command
    #[must_use]
    pub fn new() -> Self {
        Self {
            all: false,
            volumes: false,
            force: false,
            filter: HashMap::new(),
            executor: CommandExecutor::new(),
        }
    }

    /// Remove all unused images, not just dangling ones
    #[must_use]
    pub fn all(mut self) -> Self {
        self.all = true;
        self
    }

    /// Prune volumes
    #[must_use]
    pub fn volumes(mut self) -> Self {
        self.volumes = true;
        self
    }

    /// Do not prompt for confirmation
    #[must_use]
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Add a filter (e.g., "until=24h", "label=foo=bar")
    #[must_use]
    pub fn filter(mut self, key: &str, value: &str) -> Self {
        self.filter.insert(key.to_string(), value.to_string());
        self
    }

    /// Execute the system prune command
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute or if Docker is not available.
    pub async fn run(&self) -> Result<PruneResult> {
        self.execute().await
    }
}

impl Default for SystemPruneCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for SystemPruneCommand {
    type Output = PruneResult;

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["system".to_string(), "prune".to_string()];

        if self.all {
            args.push("--all".to_string());
        }

        if self.volumes {
            args.push("--volumes".to_string());
        }

        if self.force {
            args.push("--force".to_string());
        }

        for (key, value) in &self.filter {
            args.push("--filter".to_string());
            args.push(format!("{key}={value}"));
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
        let output = self
            .executor
            .execute_command(&command_name, command_args)
            .await?;

        // Parse the output to extract deleted items and space reclaimed
        // Docker system prune doesn't return JSON by default, so we need to parse text
        let stdout = &output.stdout;

        let mut result = PruneResult {
            containers_deleted: Vec::new(),
            containers_space_reclaimed: 0,
            images_deleted: Vec::new(),
            images_space_reclaimed: 0,
            networks_deleted: Vec::new(),
            volumes_deleted: Vec::new(),
            volumes_space_reclaimed: 0,
            build_cache_deleted: Vec::new(),
            space_reclaimed: 0,
        };

        // Parse the text output to extract information
        // This is a simplified parser - actual implementation would be more robust
        for line in stdout.lines() {
            if line.contains("Total reclaimed space:") {
                // Extract the space value
                if let Some(space_str) = line.split(':').nth(1) {
                    result.space_reclaimed = parse_size(space_str.trim());
                }
            }
            // Additional parsing for containers, images, etc. would go here
        }

        Ok(result)
    }
}

/// Parse size string (e.g., "1.5GB", "100MB") to bytes
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_precision_loss)]
fn parse_size(size_str: &str) -> u64 {
    // Remove any whitespace
    let size_str = size_str.trim();

    // Try to find the numeric part and unit
    let mut numeric_part = String::new();
    let mut unit_part = String::new();
    let mut found_dot = false;

    for ch in size_str.chars() {
        if ch.is_ascii_digit() || (ch == '.' && !found_dot) {
            numeric_part.push(ch);
            if ch == '.' {
                found_dot = true;
            }
        } else if ch.is_ascii_alphabetic() {
            unit_part.push(ch);
        }
    }

    // Parse the numeric value
    let value: f64 = numeric_part.parse().unwrap_or(0.0);

    // Convert based on unit
    let multiplier = match unit_part.to_uppercase().as_str() {
        "KB" | "K" => 1_024,
        "MB" | "M" => 1_024 * 1_024,
        "GB" | "G" => 1_024 * 1_024 * 1_024,
        "TB" | "T" => 1_024_u64.pow(4),
        _ => 1, // Includes "B" and empty string
    };

    (value * multiplier as f64) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("100B"), 100);
        assert_eq!(parse_size("1KB"), 1024);
        assert_eq!(parse_size("1.5KB"), 1536);
        assert_eq!(parse_size("2MB"), 2 * 1024 * 1024);
        assert_eq!(parse_size("1GB"), 1024 * 1024 * 1024);
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        {
            assert_eq!(parse_size("1.5GB"), (1.5 * 1024.0 * 1024.0 * 1024.0) as u64);
        }
    }

    #[test]
    fn test_system_prune_builder() {
        let cmd = SystemPruneCommand::new()
            .all()
            .volumes()
            .force()
            .filter("until", "24h");

        let args = cmd.build_command_args();
        assert_eq!(args[0], "system");
        assert!(args.contains(&"prune".to_string()));
        assert!(args.contains(&"--all".to_string()));
        assert!(args.contains(&"--volumes".to_string()));
        assert!(args.contains(&"--force".to_string()));
        assert!(args.contains(&"--filter".to_string()));
        assert!(args.contains(&"until=24h".to_string()));
    }
}
