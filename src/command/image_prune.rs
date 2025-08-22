//! Docker image prune command implementation.

use crate::command::{CommandExecutor, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;

/// Deleted image information
#[derive(Debug, Clone, Deserialize)]
pub struct DeletedImage {
    /// Untagged image references
    #[serde(default, rename = "Untagged")]
    pub untagged: Option<String>,

    /// Deleted image layers
    #[serde(default, rename = "Deleted")]
    pub deleted: Option<String>,
}

/// Result from image prune operation
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ImagePruneResult {
    /// List of deleted images
    #[serde(default)]
    pub images_deleted: Vec<DeletedImage>,

    /// Total space reclaimed in bytes
    #[serde(default)]
    pub space_reclaimed: u64,
}

/// Docker image prune command
///
/// Remove unused images
#[derive(Debug, Clone)]
pub struct ImagePruneCommand {
    /// Remove all unused images, not just dangling ones
    all: bool,

    /// Do not prompt for confirmation
    force: bool,

    /// Provide filter values
    filter: HashMap<String, String>,

    /// Command executor
    executor: CommandExecutor,
}

impl ImagePruneCommand {
    /// Create a new image prune command
    #[must_use]
    pub fn new() -> Self {
        Self {
            all: false,
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

    /// Only remove dangling images (default behavior)
    #[must_use]
    pub fn dangling_only(mut self) -> Self {
        self.filter
            .insert("dangling".to_string(), "true".to_string());
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

    /// Prune images older than the specified duration
    #[must_use]
    pub fn until(mut self, duration: &str) -> Self {
        self.filter
            .insert("until".to_string(), duration.to_string());
        self
    }

    /// Prune images with the specified label
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

    /// Execute the image prune command
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute or if Docker is not available.
    pub async fn run(&self) -> Result<ImagePruneResult> {
        self.execute().await
    }
}

impl Default for ImagePruneCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ImagePruneCommand {
    type Output = ImagePruneResult;

    fn command_name(&self) -> &'static str {
        "image"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["prune".to_string()];

        if self.all {
            args.push("--all".to_string());
        }

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

        // Parse the output to extract deleted images and space reclaimed
        let mut result = ImagePruneResult {
            images_deleted: Vec::new(),
            space_reclaimed: 0,
        };

        // Docker returns text output, we need to parse it
        let mut in_deleted_section = false;

        for line in stdout.lines() {
            if line.starts_with("Deleted Images:") {
                in_deleted_section = true;
                // Found deleted section header
            } else if line.starts_with("Total reclaimed space:") {
                in_deleted_section = false;
                // Extract the space value
                if let Some(space_str) = line.split(':').nth(1) {
                    result.space_reclaimed = parse_size(space_str.trim());
                }
            } else if in_deleted_section && !line.is_empty() {
                // Parse deleted or untagged entries
                if line.starts_with("deleted:") || line.starts_with("untagged:") {
                    let parts: Vec<&str> = line.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let entry_type = parts[0].trim();
                        let value = parts[1].trim().to_string();

                        if entry_type == "deleted" {
                            result.images_deleted.push(DeletedImage {
                                deleted: Some(value),
                                untagged: None,
                            });
                        } else if entry_type == "untagged" {
                            result.images_deleted.push(DeletedImage {
                                untagged: Some(value),
                                deleted: None,
                            });
                        }
                    }
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
    fn test_image_prune_builder() {
        let cmd = ImagePruneCommand::new()
            .all()
            .force()
            .until("7d")
            .with_label("deprecated", None);

        let args = cmd.build_args();
        assert!(args.contains(&"prune".to_string()));
        assert!(args.contains(&"--all".to_string()));
        assert!(args.contains(&"--force".to_string()));
        assert!(args.contains(&"--filter".to_string()));
    }

    #[test]
    fn test_dangling_only() {
        let cmd = ImagePruneCommand::new().dangling_only().force();

        let args = cmd.build_args();
        assert!(args.contains(&"prune".to_string()));
        assert!(args.contains(&"--force".to_string()));
        assert!(args.contains(&"dangling=true".to_string()));
    }
}
