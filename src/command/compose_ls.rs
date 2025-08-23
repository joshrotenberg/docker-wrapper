//! Docker Compose ls command implementation using unified trait pattern.

use super::{CommandExecutor, ComposeCommand, ComposeConfig, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use serde::Deserialize;

/// Docker Compose ls command
///
/// List running compose projects.
#[derive(Debug, Clone, Default)]
pub struct ComposeLsCommand {
    /// Base command executor
    pub executor: CommandExecutor,
    /// Base compose configuration
    pub config: ComposeConfig,
    /// Show all projects (including stopped)
    pub all: bool,
    /// Filter by name
    pub filter: Option<String>,
    /// Format output (table, json)
    pub format: Option<LsFormat>,
    /// Only display project names
    pub quiet: bool,
}

/// Ls output format
#[derive(Debug, Clone, Copy)]
pub enum LsFormat {
    /// Table format (default)
    Table,
    /// JSON format
    Json,
}

impl LsFormat {
    /// Convert to command line argument
    #[must_use]
    pub fn as_arg(&self) -> &str {
        match self {
            Self::Table => "table",
            Self::Json => "json",
        }
    }
}

/// Compose project information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ComposeProject {
    /// Project name
    pub name: String,
    /// Status
    pub status: String,
    /// Configuration files
    #[serde(default)]
    pub config_files: String,
    /// Created timestamp
    #[serde(default)]
    pub created: String,
}

/// Result from ls command
#[derive(Debug, Clone)]
pub struct LsResult {
    /// List of compose projects
    pub projects: Vec<ComposeProject>,
    /// Raw output (for non-JSON formats)
    pub raw_output: String,
}

impl ComposeLsCommand {
    /// Create a new ls command
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            ..Default::default()
        }
    }

    /// Show all projects
    #[must_use]
    pub fn all(mut self) -> Self {
        self.all = true;
        self
    }

    /// Filter projects
    #[must_use]
    pub fn filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }

    /// Set output format
    #[must_use]
    pub fn format(mut self, format: LsFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Set output format to JSON
    #[must_use]
    pub fn format_json(mut self) -> Self {
        self.format = Some(LsFormat::Json);
        self
    }

    /// Only display project names
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }
}

#[async_trait]
impl DockerCommand for ComposeLsCommand {
    type Output = LsResult;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        // Use the ComposeCommand implementation explicitly
        <Self as ComposeCommand>::build_command_args(self)
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = <Self as ComposeCommand>::build_command_args(self);
        let output = self.execute_command(args).await?;

        // Parse JSON output if format is JSON
        let projects = if matches!(self.format, Some(LsFormat::Json)) {
            serde_json::from_str(&output.stdout).unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(LsResult {
            projects,
            raw_output: output.stdout,
        })
    }
}

impl ComposeCommand for ComposeLsCommand {
    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn subcommand(&self) -> &'static str {
        "ls"
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Add flags
        if self.all {
            args.push("--all".to_string());
        }
        if self.quiet {
            args.push("--quiet".to_string());
        }

        // Add filter
        if let Some(filter) = &self.filter {
            args.push("--filter".to_string());
            args.push(filter.clone());
        }

        // Add format
        if let Some(format) = &self.format {
            args.push("--format".to_string());
            args.push(format.as_arg().to_string());
        }

        args
    }
}

impl LsResult {
    /// Get project names
    #[must_use]
    pub fn project_names(&self) -> Vec<String> {
        self.projects.iter().map(|p| p.name.clone()).collect()
    }

    /// Check if a project exists
    #[must_use]
    pub fn has_project(&self, name: &str) -> bool {
        self.projects.iter().any(|p| p.name == name)
    }

    /// Get running projects
    #[must_use]
    pub fn running_projects(&self) -> Vec<&ComposeProject> {
        self.projects
            .iter()
            .filter(|p| p.status.contains("running"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ls_command_basic() {
        let cmd = ComposeLsCommand::new();
        let _args = cmd.build_subcommand_args();
        // No specific args for basic command

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"ls".to_string()));
    }

    #[test]
    fn test_ls_command_with_all() {
        let cmd = ComposeLsCommand::new().all();
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--all".to_string()));
    }

    #[test]
    fn test_ls_command_with_format() {
        let cmd = ComposeLsCommand::new().format_json();
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"json".to_string()));
    }

    #[test]
    fn test_ls_command_with_filter() {
        let cmd = ComposeLsCommand::new().filter("status=running").quiet();
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--filter".to_string()));
        assert!(args.contains(&"status=running".to_string()));
        assert!(args.contains(&"--quiet".to_string()));
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposeLsCommand::new()
            .file("docker-compose.yml")
            .project_name("my-project");

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"my-project".to_string()));
    }

    #[test]
    fn test_ls_result_helpers() {
        let result = LsResult {
            projects: vec![
                ComposeProject {
                    name: "web".to_string(),
                    status: "running(3)".to_string(),
                    config_files: "docker-compose.yml".to_string(),
                    created: "2025-08-23".to_string(),
                },
                ComposeProject {
                    name: "db".to_string(),
                    status: "exited(0)".to_string(),
                    config_files: "docker-compose.yml".to_string(),
                    created: "2025-08-23".to_string(),
                },
            ],
            raw_output: String::new(),
        };

        assert_eq!(result.project_names(), vec!["web", "db"]);
        assert!(result.has_project("web"));
        assert!(!result.has_project("cache"));
        assert_eq!(result.running_projects().len(), 1);
    }
}
