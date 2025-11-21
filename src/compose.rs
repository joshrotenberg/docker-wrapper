use std::path::PathBuf;

use crate::{
    command::{AnsiMode, ProgressType},
    DockerCommand,
};

/// Base configuration for all compose commands.
#[derive(Debug, Clone, Default)]
pub struct ComposeConfig {
    /// Compose file paths (-f, --file).
    pub files: Vec<PathBuf>,
    /// Project name (-p, --project-name).
    pub project_name: Option<String>,
    /// Project directory (--project-directory).
    pub project_directory: Option<PathBuf>,
    /// Profiles to enable (--profile).
    pub profiles: Vec<String>,
    /// Environment file (--env-file).
    pub env_file: Option<PathBuf>,
    /// Run in compatibility mode.
    pub compatibility: bool,
    /// Execute in dry run mode.
    pub dry_run: bool,
    /// Progress output type.
    pub progress: Option<ProgressType>,
    /// ANSI control characters.
    pub ansi: Option<AnsiMode>,
    /// Max parallelism (-1 for unlimited).
    pub parallel: Option<i32>,
}

impl ComposeConfig {
    /// Creates a new compose configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a compose file.
    #[must_use]
    pub fn file(mut self, path: impl Into<PathBuf>) -> Self {
        self.files.push(path.into());
        self
    }

    /// Sets the project name.
    #[must_use]
    pub fn project_name(mut self, name: impl Into<String>) -> Self {
        self.project_name = Some(name.into());
        self
    }

    /// Sets the project directory.
    #[must_use]
    pub fn project_directory(mut self, dir: impl Into<PathBuf>) -> Self {
        self.project_directory = Some(dir.into());
        self
    }

    /// Adds a profile.
    #[must_use]
    pub fn profile(mut self, profile: impl Into<String>) -> Self {
        self.profiles.push(profile.into());
        self
    }

    /// Sets environment file.
    #[must_use]
    pub fn env_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.env_file = Some(path.into());
        self
    }

    /// Enables compatibility mode.
    #[must_use]
    pub fn compatibility(mut self) -> Self {
        self.compatibility = true;
        self
    }

    /// Enables dry run mode.
    #[must_use]
    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    /// Sets progress output type.
    #[must_use]
    pub fn progress(mut self, progress: ProgressType) -> Self {
        self.progress = Some(progress);
        self
    }

    /// Sets ANSI mode.
    #[must_use]
    pub fn ansi(mut self, ansi: AnsiMode) -> Self {
        self.ansi = Some(ansi);
        self
    }

    /// Sets max parallelism.
    #[must_use]
    pub fn parallel(mut self, parallel: i32) -> Self {
        self.parallel = Some(parallel);
        self
    }

    /// Builds global compose arguments.
    #[must_use]
    pub fn build_global_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Adds compose files.
        for file in &self.files {
            args.push("--file".to_string());
            args.push(file.to_string_lossy().to_string());
        }

        // Adds project name.
        if let Some(ref name) = self.project_name {
            args.push("--project-name".to_string());
            args.push(name.clone());
        }

        // Adds project directory.
        if let Some(ref dir) = self.project_directory {
            args.push("--project-directory".to_string());
            args.push(dir.to_string_lossy().to_string());
        }

        // Adds profiles.
        for profile in &self.profiles {
            args.push("--profile".to_string());
            args.push(profile.clone());
        }

        // Adds environment file.
        if let Some(ref env_file) = self.env_file {
            args.push("--env-file".to_string());
            args.push(env_file.to_string_lossy().to_string());
        }

        // Adds flags.
        if self.compatibility {
            args.push("--compatibility".to_string());
        }

        if self.dry_run {
            args.push("--dry-run".to_string());
        }

        // Adds progress type.
        if let Some(progress) = self.progress {
            args.push("--progress".to_string());
            args.push(progress.to_string());
        }

        // Adds ANSI mode.
        if let Some(ansi) = self.ansi {
            args.push("--ansi".to_string());
            args.push(ansi.to_string());
        }

        // Adds parallel limit.
        if let Some(parallel) = self.parallel {
            args.push("--parallel".to_string());
            args.push(parallel.to_string());
        }

        args
    }
}

/// Extended trait for Docker Compose commands.
pub trait ComposeCommand: DockerCommand {
    /// Gets the base command name. This should always be `compose`.
    fn command_name() -> &'static str {
        "compose"
    }

    /// Gets the full subcommand name (e.g., `up`, `down`), which will be appended to the base command.
    fn subcommand_name() -> &'static str;

    /// Gets the compose configuration.
    fn config(&self) -> &ComposeConfig;

    /// Gets the mutable compose configuration for builder pattern.
    fn config_mut(&mut self) -> &mut ComposeConfig;

    /// Builds command-specific arguments (without global compose args).
    fn build_subcommand_args(&self) -> Vec<String>;

    /// Builds complete command arguments including subcommand name and global args.
    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec![Self::subcommand_name().to_string()];

        // add global compose arguments
        args.extend(self.config().build_global_args());

        // add the subcommand
        args.push(self.subcommand().to_string());

        // add command-specific arguments
        args.extend(self.build_subcommand_args());

        // add raw arguments from executor
        args.extend(self.executor().raw_args.clone());

        args
    }

    /// Helper builder methods for common compose config options.
    #[must_use]
    fn file<P: Into<PathBuf>>(mut self, file: P) -> Self
    where
        Self: Sized,
    {
        self.config_mut().files.push(file.into());
        self
    }

    /// Sets project name for compose command.
    #[must_use]
    fn project_name(mut self, name: impl Into<String>) -> Self
    where
        Self: Sized,
    {
        self.config_mut().project_name = Some(name.into());
        self
    }
}
