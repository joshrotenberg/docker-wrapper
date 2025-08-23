//! Docker Compose command implementations.
//!
//! This module provides support for Docker Compose commands, enabling
//! multi-container application management.

use crate::error::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command as TokioCommand;

/// Base configuration for all compose commands
#[derive(Debug, Clone, Default)]
pub struct ComposeConfig {
    /// Compose file paths (-f, --file)
    pub files: Vec<PathBuf>,
    /// Project name (-p, --project-name)
    pub project_name: Option<String>,
    /// Project directory (--project-directory)
    pub project_directory: Option<PathBuf>,
    /// Profiles to enable (--profile)
    pub profiles: Vec<String>,
    /// Environment file (--env-file)
    pub env_file: Option<PathBuf>,
    /// Run in compatibility mode
    pub compatibility: bool,
    /// Execute in dry run mode
    pub dry_run: bool,
    /// Progress output type
    pub progress: Option<ProgressType>,
    /// ANSI control characters
    pub ansi: Option<AnsiMode>,
    /// Max parallelism (-1 for unlimited)
    pub parallel: Option<i32>,
}

/// Progress output type for compose commands
#[derive(Debug, Clone, Copy)]
pub enum ProgressType {
    /// Auto-detect
    Auto,
    /// TTY output
    Tty,
    /// Plain text output
    Plain,
    /// JSON output
    Json,
    /// Quiet mode
    Quiet,
}

impl std::fmt::Display for ProgressType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Tty => write!(f, "tty"),
            Self::Plain => write!(f, "plain"),
            Self::Json => write!(f, "json"),
            Self::Quiet => write!(f, "quiet"),
        }
    }
}

/// ANSI control character mode
#[derive(Debug, Clone, Copy)]
pub enum AnsiMode {
    /// Never print ANSI
    Never,
    /// Always print ANSI
    Always,
    /// Auto-detect
    Auto,
}

impl std::fmt::Display for AnsiMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Never => write!(f, "never"),
            Self::Always => write!(f, "always"),
            Self::Auto => write!(f, "auto"),
        }
    }
}

impl ComposeConfig {
    /// Create a new compose configuration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a compose file
    #[must_use]
    pub fn file(mut self, path: impl Into<PathBuf>) -> Self {
        self.files.push(path.into());
        self
    }

    /// Set project name
    #[must_use]
    pub fn project_name(mut self, name: impl Into<String>) -> Self {
        self.project_name = Some(name.into());
        self
    }

    /// Set project directory
    #[must_use]
    pub fn project_directory(mut self, dir: impl Into<PathBuf>) -> Self {
        self.project_directory = Some(dir.into());
        self
    }

    /// Add a profile
    #[must_use]
    pub fn profile(mut self, profile: impl Into<String>) -> Self {
        self.profiles.push(profile.into());
        self
    }

    /// Set environment file
    #[must_use]
    pub fn env_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.env_file = Some(path.into());
        self
    }

    /// Enable compatibility mode
    #[must_use]
    pub fn compatibility(mut self, enabled: bool) -> Self {
        self.compatibility = enabled;
        self
    }

    /// Enable dry run mode
    #[must_use]
    pub fn dry_run(mut self, enabled: bool) -> Self {
        self.dry_run = enabled;
        self
    }

    /// Build global arguments for compose commands
    #[must_use]
    pub fn build_global_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        for file in &self.files {
            args.push("--file".to_string());
            args.push(file.display().to_string());
        }

        if let Some(ref name) = self.project_name {
            args.push("--project-name".to_string());
            args.push(name.clone());
        }

        if let Some(ref dir) = self.project_directory {
            args.push("--project-directory".to_string());
            args.push(dir.display().to_string());
        }

        for profile in &self.profiles {
            args.push("--profile".to_string());
            args.push(profile.clone());
        }

        if let Some(ref env_file) = self.env_file {
            args.push("--env-file".to_string());
            args.push(env_file.display().to_string());
        }

        if self.compatibility {
            args.push("--compatibility".to_string());
        }

        if self.dry_run {
            args.push("--dry-run".to_string());
        }

        if let Some(ref progress) = self.progress {
            args.push("--progress".to_string());
            args.push(progress.to_string());
        }

        if let Some(ref ansi) = self.ansi {
            args.push("--ansi".to_string());
            args.push(ansi.to_string());
        }

        if let Some(parallel) = self.parallel {
            args.push("--parallel".to_string());
            args.push(parallel.to_string());
        }

        args
    }
}

/// Execute a compose command with the given configuration and arguments
async fn execute_compose_command(
    config: &ComposeConfig,
    subcommand: &str,
    args: Vec<String>,
) -> Result<ComposeOutput> {
    let mut cmd = TokioCommand::new("docker");

    // Add "compose" as the first argument
    cmd.arg("compose");

    // Add global compose arguments
    for arg in config.build_global_args() {
        cmd.arg(arg);
    }

    // Add the subcommand
    cmd.arg(subcommand);

    // Add command-specific arguments
    for arg in args {
        cmd.arg(arg);
    }

    // Set up output pipes
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.output().await.map_err(|e| {
        crate::error::Error::custom(format!(
            "Failed to execute docker compose {subcommand}: {e}"
        ))
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();
    let exit_code = output.status.code().unwrap_or(-1);

    if !success && !stderr.contains("Gracefully stopping...") {
        return Err(crate::error::Error::command_failed(
            format!("docker compose {subcommand}"),
            exit_code,
            stdout.clone(),
            stderr.clone(),
        ));
    }

    Ok(ComposeOutput {
        stdout,
        stderr,
        exit_code,
        success,
    })
}

/// Common trait for all compose commands (existing pattern)
#[async_trait]
pub trait ComposeCommand {
    /// The output type this command produces
    type Output;

    /// Get the compose subcommand name (e.g., "up", "down", "ps")
    fn subcommand(&self) -> &'static str;

    /// Build command-specific arguments
    fn build_args(&self) -> Vec<String>;

    /// Execute the command
    async fn execute(&self) -> Result<Self::Output>;

    /// Get the compose configuration
    fn config(&self) -> &ComposeConfig;
}

/// Common trait for new compose commands
#[async_trait]
pub trait ComposeCommandV2 {
    /// The output type this command produces
    type Output;

    /// Get the compose configuration
    fn get_config(&self) -> &ComposeConfig;

    /// Get mutable compose configuration
    fn get_config_mut(&mut self) -> &mut ComposeConfig;

    /// Execute compose command with given arguments
    async fn execute_compose(&self, args: Vec<String>) -> Result<Self::Output>;

    /// Execute the command
    async fn execute(&self) -> Result<Self::Output>;

    /// Helper to execute compose command
    async fn execute_compose_command(&self, args: Vec<String>) -> Result<ComposeOutput> {
        let config = self.get_config();
        let mut cmd = TokioCommand::new("docker");

        // Add "compose" as the first argument
        cmd.arg("compose");

        // Add global compose arguments
        for arg in config.build_global_args() {
            cmd.arg(arg);
        }

        // Add command-specific arguments
        for arg in args {
            cmd.arg(arg);
        }

        // Set up output pipes
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd.output().await.map_err(|e| {
            crate::error::Error::custom(format!("Failed to execute docker compose: {e}"))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();
        let exit_code = output.status.code().unwrap_or(-1);

        if !success && !stderr.contains("Gracefully stopping...") {
            return Err(crate::error::Error::command_failed(
                "docker compose".to_string(),
                exit_code,
                stdout.clone(),
                stderr.clone(),
            ));
        }

        Ok(ComposeOutput {
            stdout,
            stderr,
            exit_code,
            success,
        })
    }
}

/// Output from a compose command
#[derive(Debug, Clone)]
pub struct ComposeOutput {
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit code
    pub exit_code: i32,
    /// Whether the command succeeded
    pub success: bool,
}

impl ComposeOutput {
    /// Get stdout lines
    #[must_use]
    pub fn stdout_lines(&self) -> Vec<&str> {
        self.stdout.lines().collect()
    }

    /// Get stderr lines
    #[must_use]
    pub fn stderr_lines(&self) -> Vec<&str> {
        self.stderr.lines().collect()
    }
}

// Re-export submodules
pub mod attach;
pub mod build;
pub mod config;
pub mod convert;
pub mod cp;
pub mod create;
pub mod down;
pub mod events;
pub mod exec;
pub mod images;
pub mod kill;
pub mod logs;
pub mod ls;
pub mod pause;
pub mod port;
pub mod ps;
pub mod push;
pub mod restart;
pub mod rm;
pub mod run;
pub mod scale;
pub mod start;
pub mod stop;
pub mod top;
pub mod unpause;
pub mod up;
pub mod version;
pub mod wait;
pub mod watch;

pub use attach::{AttachResult, ComposeAttachCommand};
pub use build::ComposeBuildCommand;
pub use config::{ComposeConfigCommand, ConfigFormat, ConfigResult};
pub use convert::{ComposeConvertCommand, ConvertFormat, ConvertResult};
pub use cp::{ComposeCpCommand, CpResult};
pub use create::{ComposeCreateCommand, CreateResult, PullPolicy};
pub use down::ComposeDownCommand;
pub use events::{ComposeEvent, ComposeEventsCommand, EventsResult};
pub use exec::ComposeExecCommand;
pub use images::{ComposeImagesCommand, ImageInfo, ImagesFormat, ImagesResult};
pub use kill::{ComposeKillCommand, KillResult};
pub use logs::ComposeLogsCommand;
pub use ls::{ComposeLsCommand, ComposeProject, LsFormat, LsResult};
pub use pause::{ComposePauseCommand, PauseResult};
pub use port::{ComposePortCommand, PortResult};
pub use ps::ComposePsCommand;
pub use push::{ComposePushCommand, PushResult};
pub use restart::ComposeRestartCommand;
pub use rm::{ComposeRmCommand, RmResult};
pub use run::ComposeRunCommand;
pub use scale::{ComposeScaleCommand, ScaleResult};
pub use start::ComposeStartCommand;
pub use stop::ComposeStopCommand;
pub use top::{ComposeTopCommand, TopResult};
pub use unpause::{ComposeUnpauseCommand, UnpauseResult};
pub use up::ComposeUpCommand;
pub use version::{ComposeVersionCommand, VersionFormat, VersionInfo, VersionResult};
pub use wait::{ComposeWaitCommand, WaitResult};
pub use watch::{ComposeWatchCommand, WatchResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_config_new() {
        let config = ComposeConfig::new();
        assert!(config.files.is_empty());
        assert!(config.project_name.is_none());
        assert!(config.project_directory.is_none());
        assert!(config.profiles.is_empty());
        assert!(config.env_file.is_none());
        assert!(!config.compatibility);
        assert!(!config.dry_run);
    }

    #[test]
    fn test_compose_config_builder() {
        let config = ComposeConfig::new()
            .file("docker-compose.yml")
            .file("docker-compose.override.yml")
            .project_name("myproject")
            .project_directory("/path/to/project")
            .profile("dev")
            .profile("debug")
            .env_file(".env")
            .compatibility(true)
            .dry_run(true);

        assert_eq!(config.files.len(), 2);
        assert_eq!(config.files[0].to_str().unwrap(), "docker-compose.yml");
        assert_eq!(
            config.files[1].to_str().unwrap(),
            "docker-compose.override.yml"
        );
        assert_eq!(config.project_name, Some("myproject".to_string()));
        assert_eq!(
            config.project_directory.unwrap().to_str().unwrap(),
            "/path/to/project"
        );
        assert_eq!(config.profiles, vec!["dev", "debug"]);
        assert_eq!(config.env_file.unwrap().to_str().unwrap(), ".env");
        assert!(config.compatibility);
        assert!(config.dry_run);
    }

    #[test]
    fn test_compose_config_build_global_args() {
        let config = ComposeConfig::new();
        let args = config.build_global_args();
        assert!(args.is_empty());
    }

    #[test]
    fn test_compose_config_build_global_args_with_files() {
        let config = ComposeConfig::new()
            .file("compose.yml")
            .file("compose.override.yml");

        let args = config.build_global_args();
        assert_eq!(
            args,
            vec!["--file", "compose.yml", "--file", "compose.override.yml"]
        );
    }

    #[test]
    fn test_compose_config_build_global_args_complete() {
        let config = ComposeConfig::new()
            .file("docker-compose.yml")
            .project_name("test")
            .project_directory("/app")
            .profile("prod")
            .env_file(".env.prod")
            .compatibility(true)
            .dry_run(true);

        let args = config.build_global_args();
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"test".to_string()));
        assert!(args.contains(&"--project-directory".to_string()));
        assert!(args.contains(&"/app".to_string()));
        assert!(args.contains(&"--profile".to_string()));
        assert!(args.contains(&"prod".to_string()));
        assert!(args.contains(&"--env-file".to_string()));
        assert!(args.contains(&".env.prod".to_string()));
        assert!(args.contains(&"--compatibility".to_string()));
        assert!(args.contains(&"--dry-run".to_string()));
    }

    #[test]
    fn test_compose_config_with_progress() {
        let mut config = ComposeConfig::new();
        config.progress = Some(ProgressType::Plain);

        let args = config.build_global_args();
        assert!(args.contains(&"--progress".to_string()));
        assert!(args.contains(&"plain".to_string()));
    }

    #[test]
    fn test_compose_config_with_ansi() {
        let mut config = ComposeConfig::new();
        config.ansi = Some(AnsiMode::Never);

        let args = config.build_global_args();
        assert!(args.contains(&"--ansi".to_string()));
        assert!(args.contains(&"never".to_string()));
    }

    #[test]
    fn test_compose_config_with_parallel() {
        let mut config = ComposeConfig::new();
        config.parallel = Some(4);

        let args = config.build_global_args();
        assert!(args.contains(&"--parallel".to_string()));
        assert!(args.contains(&"4".to_string()));
    }

    #[test]
    fn test_progress_type_display() {
        assert_eq!(ProgressType::Auto.to_string(), "auto");
        assert_eq!(ProgressType::Tty.to_string(), "tty");
        assert_eq!(ProgressType::Plain.to_string(), "plain");
        assert_eq!(ProgressType::Json.to_string(), "json");
        assert_eq!(ProgressType::Quiet.to_string(), "quiet");
    }

    #[test]
    fn test_ansi_mode_display() {
        assert_eq!(AnsiMode::Never.to_string(), "never");
        assert_eq!(AnsiMode::Always.to_string(), "always");
        assert_eq!(AnsiMode::Auto.to_string(), "auto");
    }

    #[test]
    fn test_compose_output_stdout_lines() {
        let output = ComposeOutput {
            stdout: "line1\nline2\nline3".to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };

        let lines = output.stdout_lines();
        assert_eq!(lines, vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn test_compose_output_stderr_lines() {
        let output = ComposeOutput {
            stdout: String::new(),
            stderr: "error1\nerror2".to_string(),
            exit_code: 1,
            success: false,
        };

        let lines = output.stderr_lines();
        assert_eq!(lines, vec!["error1", "error2"]);
    }

    #[test]
    fn test_compose_output_empty_lines() {
        let output = ComposeOutput {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };

        // Empty string produces no lines when split
        assert!(output.stdout_lines().is_empty());
        assert!(output.stderr_lines().is_empty());
    }

    #[test]
    fn test_compose_output_single_line() {
        let output = ComposeOutput {
            stdout: "single line".to_string(),
            stderr: "error line".to_string(),
            exit_code: 0,
            success: true,
        };

        assert_eq!(output.stdout_lines(), vec!["single line"]);
        assert_eq!(output.stderr_lines(), vec!["error line"]);
    }
}
