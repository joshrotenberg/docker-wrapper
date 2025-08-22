//! Docker volume management commands.
//!
//! This module provides commands for managing Docker volumes.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::OsStr;

/// Docker volume create command
#[derive(Debug, Clone)]
pub struct VolumeCreateCommand {
    name: Option<String>,
    driver: Option<String>,
    driver_opts: HashMap<String, String>,
    labels: HashMap<String, String>,
    executor: CommandExecutor,
}

impl VolumeCreateCommand {
    /// Create a new volume create command
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: None,
            driver: None,
            driver_opts: HashMap::new(),
            labels: HashMap::new(),
            executor: CommandExecutor::new(),
        }
    }

    /// Set volume name
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the volume driver
    #[must_use]
    pub fn driver(mut self, driver: impl Into<String>) -> Self {
        self.driver = Some(driver.into());
        self
    }

    /// Add a driver option
    #[must_use]
    pub fn driver_opt(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.driver_opts.insert(key.into(), value.into());
        self
    }

    /// Add a label
    #[must_use]
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Execute the command
    ///
    /// # Errors
    ///
    /// Returns an error if the Docker daemon is not running or the command fails
    pub async fn run(&self) -> Result<VolumeCreateResult> {
        self.execute().await.map(VolumeCreateResult::from)
    }
}

impl Default for VolumeCreateCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for VolumeCreateCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "volume create"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["create".to_string()];

        if let Some(ref driver) = self.driver {
            args.push("--driver".to_string());
            args.push(driver.clone());
        }

        for (key, value) in &self.driver_opts {
            args.push("--opt".to_string());
            args.push(format!("{key}={value}"));
        }

        for (key, value) in &self.labels {
            args.push("--label".to_string());
            args.push(format!("{key}={value}"));
        }

        if let Some(ref name) = self.name {
            args.push(name.clone());
        }

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        self.executor
            .execute_command("volume", self.build_args())
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

/// Result from volume create
#[derive(Debug, Clone)]
pub struct VolumeCreateResult {
    /// The name of the created volume
    pub volume_name: String,
    /// Raw command output
    pub raw_output: CommandOutput,
}

impl From<CommandOutput> for VolumeCreateResult {
    fn from(output: CommandOutput) -> Self {
        Self {
            volume_name: output.stdout.trim().to_string(),
            raw_output: output,
        }
    }
}

/// Docker volume ls command
#[derive(Debug, Clone)]
pub struct VolumeLsCommand {
    filters: HashMap<String, String>,
    format: Option<String>,
    quiet: bool,
    executor: CommandExecutor,
}

impl VolumeLsCommand {
    /// Create a new volume ls command
    #[must_use]
    pub fn new() -> Self {
        Self {
            filters: HashMap::new(),
            format: None,
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

    /// Set format
    #[must_use]
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Only display volume names
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
    pub async fn run(&self) -> Result<VolumeLsOutput> {
        self.execute().await.map(VolumeLsOutput::from)
    }
}

impl Default for VolumeLsCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for VolumeLsCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "volume ls"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["ls".to_string()];

        for (key, value) in &self.filters {
            args.push("--filter".to_string());
            args.push(format!("{key}={value}"));
        }

        if let Some(ref format) = self.format {
            args.push("--format".to_string());
            args.push(format.clone());
        }

        if self.quiet {
            args.push("--quiet".to_string());
        }

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        self.executor
            .execute_command("volume", self.build_args())
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

/// Volume information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VolumeInfo {
    /// Volume driver
    #[serde(default)]
    pub driver: String,
    /// Volume name
    #[serde(default)]
    pub name: String,
    /// Mount point on the host
    #[serde(default)]
    pub mountpoint: String,
    /// Volume scope (local, global)
    #[serde(default)]
    pub scope: String,
    /// Volume labels
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

/// Output from volume ls
#[derive(Debug, Clone)]
pub struct VolumeLsOutput {
    /// List of volumes
    pub volumes: Vec<VolumeInfo>,
    /// Raw command output
    pub raw_output: CommandOutput,
}

impl From<CommandOutput> for VolumeLsOutput {
    fn from(output: CommandOutput) -> Self {
        let volumes = if output.stdout.starts_with('[') {
            serde_json::from_str(&output.stdout).unwrap_or_default()
        } else {
            vec![]
        };

        Self {
            volumes,
            raw_output: output,
        }
    }
}

/// Docker volume rm command
#[derive(Debug, Clone)]
pub struct VolumeRmCommand {
    volumes: Vec<String>,
    force: bool,
    executor: CommandExecutor,
}

impl VolumeRmCommand {
    /// Create a new volume rm command
    #[must_use]
    pub fn new(volume: impl Into<String>) -> Self {
        Self {
            volumes: vec![volume.into()],
            force: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Add a volume to remove
    #[must_use]
    pub fn add_volume(mut self, volume: impl Into<String>) -> Self {
        self.volumes.push(volume.into());
        self
    }

    /// Force removal
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
    pub async fn run(&self) -> Result<VolumeRmResult> {
        self.execute().await.map(VolumeRmResult::from)
    }
}

#[async_trait]
impl DockerCommand for VolumeRmCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "volume rm"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["rm".to_string()];

        if self.force {
            args.push("--force".to_string());
        }

        for volume in &self.volumes {
            args.push(volume.clone());
        }

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        self.executor
            .execute_command("volume", self.build_args())
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

/// Result from volume rm
#[derive(Debug, Clone)]
pub struct VolumeRmResult {
    /// Names of removed volumes
    pub removed_volumes: Vec<String>,
    /// Raw command output
    pub raw_output: CommandOutput,
}

impl From<CommandOutput> for VolumeRmResult {
    fn from(output: CommandOutput) -> Self {
        let removed_volumes = output
            .stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(String::from)
            .collect();

        Self {
            removed_volumes,
            raw_output: output,
        }
    }
}

/// Docker volume inspect command
#[derive(Debug, Clone)]
pub struct VolumeInspectCommand {
    volumes: Vec<String>,
    format: Option<String>,
    executor: CommandExecutor,
}

impl VolumeInspectCommand {
    /// Create a new volume inspect command
    #[must_use]
    pub fn new(volume: impl Into<String>) -> Self {
        Self {
            volumes: vec![volume.into()],
            format: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Set format
    #[must_use]
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Execute the command
    ///
    /// # Errors
    ///
    /// Returns an error if the Docker daemon is not running or the command fails
    pub async fn run(&self) -> Result<VolumeInspectOutput> {
        self.execute().await.map(VolumeInspectOutput::from)
    }
}

#[async_trait]
impl DockerCommand for VolumeInspectCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "volume inspect"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["inspect".to_string()];

        if let Some(ref format) = self.format {
            args.push("--format".to_string());
            args.push(format.clone());
        }

        for volume in &self.volumes {
            args.push(volume.clone());
        }

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        self.executor
            .execute_command("volume", self.build_args())
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

/// Output from volume inspect
#[derive(Debug, Clone)]
pub struct VolumeInspectOutput {
    /// Parsed JSON output
    pub json: Option<Value>,
    /// Raw command output
    pub raw_output: CommandOutput,
}

impl From<CommandOutput> for VolumeInspectOutput {
    fn from(output: CommandOutput) -> Self {
        let json = serde_json::from_str(&output.stdout).ok();
        Self {
            json,
            raw_output: output,
        }
    }
}

/// Docker volume prune command
#[derive(Debug, Clone)]
pub struct VolumePruneCommand {
    all: bool,
    filters: HashMap<String, String>,
    force: bool,
    executor: CommandExecutor,
}

impl VolumePruneCommand {
    /// Create a new volume prune command
    #[must_use]
    pub fn new() -> Self {
        Self {
            all: false,
            filters: HashMap::new(),
            force: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Remove all unused volumes
    #[must_use]
    pub fn all(mut self) -> Self {
        self.all = true;
        self
    }

    /// Add a filter
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

    /// Execute the command
    ///
    /// # Errors
    ///
    /// Returns an error if the Docker daemon is not running or the command fails
    pub async fn run(&self) -> Result<VolumePruneResult> {
        self.execute().await.map(VolumePruneResult::from)
    }
}

impl Default for VolumePruneCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for VolumePruneCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "volume prune"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["prune".to_string()];

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

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        self.executor
            .execute_command("volume", self.build_args())
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

/// Result from volume prune
#[derive(Debug, Clone)]
pub struct VolumePruneResult {
    /// Names of deleted volumes
    pub deleted_volumes: Vec<String>,
    /// Amount of disk space reclaimed in bytes
    pub space_reclaimed: Option<u64>,
    /// Raw command output
    pub raw_output: CommandOutput,
}

impl From<CommandOutput> for VolumePruneResult {
    fn from(output: CommandOutput) -> Self {
        let deleted_volumes = Vec::new(); // Parse from output if needed
        Self {
            deleted_volumes,
            space_reclaimed: None,
            raw_output: output,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_create() {
        let cmd = VolumeCreateCommand::new().name("my-volume");
        let args = cmd.build_args();
        assert_eq!(args, vec!["create", "my-volume"]);
    }

    #[test]
    fn test_volume_ls() {
        let cmd = VolumeLsCommand::new();
        let args = cmd.build_args();
        assert_eq!(args, vec!["ls"]);
    }

    #[test]
    fn test_volume_rm() {
        let cmd = VolumeRmCommand::new("my-volume");
        let args = cmd.build_args();
        assert_eq!(args, vec!["rm", "my-volume"]);
    }

    #[test]
    fn test_volume_inspect() {
        let cmd = VolumeInspectCommand::new("my-volume");
        let args = cmd.build_args();
        assert_eq!(args, vec!["inspect", "my-volume"]);
    }

    #[test]
    fn test_volume_prune() {
        let cmd = VolumePruneCommand::new().force();
        let args = cmd.build_args();
        assert_eq!(args, vec!["prune", "--force"]);
    }
}
