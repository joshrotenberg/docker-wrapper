//! Docker container prune command implementation.

use crate::command::{CommandExecutor, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;

/// Result from container prune operation
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerPruneResult {
    /// List of deleted container IDs
    #[serde(default)]
    pub containers_deleted: Vec<String>,

    /// Total space reclaimed in bytes
    #[serde(default)]
    pub space_reclaimed: u64,
}

/// Docker container prune command
///
/// Remove all stopped containers
#[derive(Debug, Clone)]
pub struct ContainerPruneCommand {
    /// Do not prompt for confirmation
    force: bool,

    /// Provide filter values
    filter: HashMap<String, String>,

    /// Command executor
    executor: CommandExecutor,
}

impl ContainerPruneCommand {
    /// Create a new container prune command
    #[must_use]
    pub fn new() -> Self {
        Self {
            force: false,
            filter: HashMap::new(),
            executor: CommandExecutor::new(),
        }
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

    /// Prune containers older than the specified duration
    #[must_use]
    pub fn until(mut self, duration: &str) -> Self {
        self.filter
            .insert("until".to_string(), duration.to_string());
        self
    }

    /// Prune containers with the specified label
    #[must_use]
    pub fn with_label(mut self, key: &str, value: Option<&str>) -> Self {
        let label_filter = if let Some(val) = value {
            format!("{key}={val}")
        } else {
            key.to_string()
        };
        self.filter.insert("label".to_string(), label_filter);
        self
    }

    /// Execute the container prune command
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute or if Docker is not available.
    pub async fn run(&self) -> Result<ContainerPruneResult> {
        self.execute().await
    }
}

impl Default for ContainerPruneCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ContainerPruneCommand {
    type Output = ContainerPruneResult;

    fn command_name(&self) -> &'static str {
        "container"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["prune".to_string()];

        if self.force {
            args.push("--force".to_string());
        }

        for (key, value) in &self.filter {
            args.push("--filter".to_string());
            if key == "label" {
                args.push(value.clone());
            } else {
                args.push(format!("{key}={value}"));
            }
        }

        args
    }

    fn arg<S: AsRef<std::ffi::OsStr>>(&mut self, _arg: S) -> &mut Self {
        self
    }

    fn args<I, S>(&mut self, _args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self
    }

    fn flag(&mut self, _flag: &str) -> &mut Self {
        self
    }

    fn option(&mut self, _key: &str, _value: &str) -> &mut Self {
        self
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_args();
        let output = self
            .executor
            .execute_command(self.command_name(), args)
            .await?;
        let stdout = &output.stdout;

        // Parse the output to extract deleted containers and space reclaimed
        let mut result = ContainerPruneResult {
            containers_deleted: Vec::new(),
            space_reclaimed: 0,
        };

        // Docker returns text output, we need to parse it
        for line in stdout.lines() {
            if line.starts_with("Deleted Containers:") {
                // Next lines contain container IDs
                continue;
            }
            if line.starts_with("Total reclaimed space:") {
                // Extract the space value
                if let Some(space_str) = line.split(':').nth(1) {
                    result.space_reclaimed = parse_size(space_str.trim());
                }
            } else if !line.is_empty() && !line.contains("will be removed") {
                // This might be a container ID
                let id = line.trim();
                if id.len() == 12 || id.len() == 64 {
                    result.containers_deleted.push(id.to_string());
                }
            }
        }

        Ok(result)
    }
}

/// Parse size string (e.g., "1.5GB", "100MB") to bytes
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_precision_loss)]
fn parse_size(size_str: &str) -> u64 {
    let size_str = size_str.trim();
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

    let value: f64 = numeric_part.parse().unwrap_or(0.0);

    let multiplier = match unit_part.to_uppercase().as_str() {
        "KB" | "K" => 1_024,
        "MB" | "M" => 1_024 * 1_024,
        "GB" | "G" => 1_024 * 1_024 * 1_024,
        "TB" | "T" => 1_024_u64.pow(4),
        _ => 1,  // Includes "B" and empty string
    };

    (value * multiplier as f64) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_prune_builder() {
        let cmd = ContainerPruneCommand::new()
            .force()
            .until("24h")
            .with_label("temp", Some("true"));

        let args = cmd.build_args();
        assert!(args.contains(&"prune".to_string()));
        assert!(args.contains(&"--force".to_string()));
        assert!(args.contains(&"--filter".to_string()));
    }
}
