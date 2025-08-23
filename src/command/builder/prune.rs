//! Docker builder prune command
//!
//! Remove build cache

use crate::command::{CommandExecutor, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// `docker builder prune` command to remove build cache
///
/// # Example
/// ```no_run
/// use docker_wrapper::command::builder::BuilderPruneCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Remove all build cache
/// let result = BuilderPruneCommand::new()
///     .all()
///     .force()
///     .execute()
///     .await?;
///
/// println!("Reclaimed {} bytes", result.space_reclaimed.unwrap_or(0));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct BuilderPruneCommand {
    /// Remove all unused build cache, not just dangling ones
    all: bool,
    /// Provide filter values
    filters: HashMap<String, String>,
    /// Do not prompt for confirmation
    force: bool,
    /// Amount of disk storage to keep for cache
    keep_storage: Option<String>,
    /// Command executor
    pub executor: CommandExecutor,
}

/// Result of builder prune operation
#[derive(Debug)]
pub struct BuilderPruneResult {
    /// IDs of deleted build cache entries
    pub deleted_cache_ids: Vec<String>,
    /// Amount of disk space reclaimed in bytes
    pub space_reclaimed: Option<u64>,
    /// Human-readable space reclaimed (e.g., "2.5GB")
    pub space_reclaimed_str: Option<String>,
    /// Standard output from the command
    pub stdout: String,
    /// Standard error from the command
    pub stderr: String,
    /// Exit code
    pub exit_code: i32,
}

impl BuilderPruneCommand {
    /// Create a new builder prune command
    #[must_use]
    pub fn new() -> Self {
        Self {
            all: false,
            filters: HashMap::new(),
            force: false,
            keep_storage: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Remove all unused build cache, not just dangling ones
    #[must_use]
    pub fn all(mut self) -> Self {
        self.all = true;
        self
    }

    /// Add a filter to the prune operation
    ///
    /// Common filters:
    /// - `until=<timestamp>` - only remove cache created before given timestamp
    /// - `until=24h` - only remove cache older than 24 hours
    #[must_use]
    pub fn filter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.filters.insert(key.into(), value.into());
        self
    }

    /// Do not prompt for confirmation
    #[must_use]
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Amount of disk storage to keep for cache
    ///
    /// # Example
    /// ```no_run
    /// # use docker_wrapper::command::builder::BuilderPruneCommand;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// BuilderPruneCommand::new()
    ///     .keep_storage("5GB")
    ///     .execute()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn keep_storage(mut self, size: impl Into<String>) -> Self {
        self.keep_storage = Some(size.into());
        self
    }

    /// Parse the prune output to extract cache IDs and space reclaimed
    fn parse_output(output: &str) -> (Vec<String>, Option<u64>, Option<String>) {
        let mut cache_ids = Vec::new();
        let mut space_reclaimed = None;
        let mut space_reclaimed_str = None;

        for line in output.lines() {
            // Parse deleted cache entries (format: "Deleted: sha256:...")
            if line.starts_with("Deleted:") || line.starts_with("deleted:") {
                if let Some(id) = line.split_whitespace().nth(1) {
                    cache_ids.push(id.to_string());
                }
            }

            // Parse total reclaimed space
            if line.contains("Total reclaimed space:") || line.contains("total reclaimed space:") {
                space_reclaimed_str = line.split(':').nth(1).map(|s| s.trim().to_string());

                // Try to parse the bytes value
                if let Some(size_str) = &space_reclaimed_str {
                    space_reclaimed = parse_size(size_str);
                }
            }
        }

        (cache_ids, space_reclaimed, space_reclaimed_str)
    }
}

impl Default for BuilderPruneCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for BuilderPruneCommand {
    type Output = BuilderPruneResult;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["builder".to_string(), "prune".to_string()];

        if self.all {
            args.push("--all".to_string());
        }

        for (key, value) in &self.filters {
            args.push("--filter".to_string());
            args.push(format!("{key}={value}"));
        }

        if self.force {
            args.push("--force".to_string());
        }

        if let Some(storage) = &self.keep_storage {
            args.push("--keep-storage".to_string());
            args.push(storage.clone());
        }

        // Add any raw arguments
        args.extend(self.executor.raw_args.clone());

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        let output = self.executor.execute_command("docker", args).await?;

        let (deleted_cache_ids, space_reclaimed, space_reclaimed_str) =
            Self::parse_output(&output.stdout);

        Ok(BuilderPruneResult {
            deleted_cache_ids,
            space_reclaimed,
            space_reclaimed_str,
            stdout: output.stdout,
            stderr: output.stderr,
            exit_code: output.exit_code,
        })
    }
}

/// Parse a size string (e.g., "2.5GB", "100MB") into bytes
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_precision_loss)]
fn parse_size(size_str: &str) -> Option<u64> {
    let size_str = size_str.trim();

    // Try to extract number and unit
    let (num_str, unit) = if let Some(pos) = size_str.find(|c: char| c.is_alphabetic()) {
        (&size_str[..pos], &size_str[pos..])
    } else {
        return size_str.parse().ok();
    };

    let number: f64 = num_str.trim().parse().ok()?;

    let multiplier = match unit.to_uppercase().as_str() {
        "B" | "" => 1.0,
        "KB" | "K" => 1024.0,
        "MB" | "M" => 1024.0 * 1024.0,
        "GB" | "G" => 1024.0 * 1024.0 * 1024.0,
        "TB" | "T" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };

    if number.is_sign_negative() || !number.is_finite() {
        return None;
    }
    let result = (number * multiplier).round();
    if result > u64::MAX as f64 {
        return None;
    }
    Some(result as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_prune_basic() {
        let cmd = BuilderPruneCommand::new();
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["builder", "prune"]);
    }

    #[test]
    fn test_builder_prune_all_options() {
        let cmd = BuilderPruneCommand::new()
            .all()
            .filter("until", "24h")
            .force()
            .keep_storage("5GB");

        let args = cmd.build_command_args();
        assert!(args.contains(&"--all".to_string()));
        assert!(args.contains(&"--filter".to_string()));
        assert!(args.contains(&"until=24h".to_string()));
        assert!(args.contains(&"--force".to_string()));
        assert!(args.contains(&"--keep-storage".to_string()));
        assert!(args.contains(&"5GB".to_string()));
    }

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("100"), Some(100));
        assert_eq!(parse_size("1KB"), Some(1024));
        assert_eq!(parse_size("1.5KB"), Some(1536));
        assert_eq!(parse_size("2MB"), Some(2 * 1024 * 1024));
        assert_eq!(parse_size("1GB"), Some(1024 * 1024 * 1024));
        assert_eq!(parse_size("2.5GB"), Some(2_684_354_560));
        assert_eq!(parse_size("1TB"), Some(1_099_511_627_776));
    }

    #[test]
    fn test_parse_output() {
        let output = r"Deleted: sha256:abc123
Deleted: sha256:def456
Total reclaimed space: 2.5GB";

        let (ids, bytes, str_val) = BuilderPruneCommand::parse_output(output);
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"sha256:abc123".to_string()));
        assert!(ids.contains(&"sha256:def456".to_string()));
        assert_eq!(bytes, Some(2_684_354_560));
        assert_eq!(str_val, Some("2.5GB".to_string()));
    }

    #[test]
    fn test_builder_prune_extensibility() {
        let mut cmd = BuilderPruneCommand::new();
        cmd.get_executor_mut()
            .raw_args
            .push("--custom-flag".to_string());

        let args = cmd.build_command_args();
        assert!(args.contains(&"--custom-flag".to_string()));
    }
}
