//! Docker Compose scale command implementation using unified trait pattern.

use crate::command::{CommandExecutor, ComposeCommand, ComposeConfig, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// Docker Compose scale command builder
#[derive(Debug, Clone)]
pub struct ComposeScaleCommand {
    /// Base command executor
    pub executor: CommandExecutor,
    /// Base compose configuration
    pub config: ComposeConfig,
    /// Service scaling configurations (`service_name` -> replicas)
    pub services: HashMap<String, u32>,
    /// Don't start linked services
    pub no_deps: bool,
}

/// Result from compose scale command
#[derive(Debug, Clone)]
pub struct ComposeScaleResult {
    /// Raw stdout output
    pub stdout: String,
    /// Raw stderr output
    pub stderr: String,
    /// Success status
    pub success: bool,
    /// Services that were scaled
    pub scaled_services: HashMap<String, u32>,
}

impl ComposeScaleCommand {
    /// Create a new compose scale command
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            services: HashMap::new(),
            no_deps: false,
        }
    }

    /// Add a service to scale
    #[must_use]
    pub fn service(mut self, service: impl Into<String>, replicas: u32) -> Self {
        self.services.insert(service.into(), replicas);
        self
    }

    /// Add multiple services to scale
    #[must_use]
    pub fn services(mut self, services: HashMap<String, u32>) -> Self {
        self.services.extend(services);
        self
    }

    /// Don't start linked services
    #[must_use]
    pub fn no_deps(mut self) -> Self {
        self.no_deps = true;
        self
    }
}

impl Default for ComposeScaleCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposeScaleCommand {
    type Output = ComposeScaleResult;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        <Self as ComposeCommand>::build_command_args(self)
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = <Self as ComposeCommand>::build_command_args(self);
        let output = self.execute_command(args).await?;

        Ok(ComposeScaleResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            scaled_services: self.services.clone(),
        })
    }
}

impl ComposeCommand for ComposeScaleCommand {
    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn subcommand(&self) -> &'static str {
        "scale"
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.no_deps {
            args.push("--no-deps".to_string());
        }

        for (service, replicas) in &self.services {
            args.push(format!("{service}={replicas}"));
        }

        args
    }
}

impl ComposeScaleResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Get the services that were scaled
    #[must_use]
    pub fn scaled_services(&self) -> &HashMap<String, u32> {
        &self.scaled_services
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_scale_basic() {
        let cmd = ComposeScaleCommand::new().service("web", 3);
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"web=3".to_string()));

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"scale".to_string()));
    }

    #[test]
    fn test_compose_scale_multiple_services() {
        let cmd = ComposeScaleCommand::new()
            .service("web", 3)
            .service("worker", 2);

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"web=3".to_string()));
        assert!(args.contains(&"worker=2".to_string()));
    }

    #[test]
    fn test_compose_scale_no_deps() {
        let cmd = ComposeScaleCommand::new().service("api", 5).no_deps();

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--no-deps".to_string()));
        assert!(args.contains(&"api=5".to_string()));
    }
}
