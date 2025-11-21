//! Docker context update command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker context update command builder
///
/// Update an existing Docker context.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::{ContextUpdateCommand, DockerCommand};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Update a context's description
/// ContextUpdateCommand::new("production")
///     .description("Updated production environment")
///     .execute()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ContextUpdateCommand {
    /// Context name to update
    name: String,
    /// New description
    description: Option<String>,
    /// New Docker host
    docker_host: Option<String>,
    /// New default stack orchestrator
    default_stack_orchestrator: Option<String>,
    /// New Docker API endpoint
    docker_api_endpoint: Option<String>,
    /// New Kubernetes config file
    kubernetes_config_file: Option<String>,
    /// New Kubernetes context
    kubernetes_context: Option<String>,
    /// New Kubernetes namespace
    kubernetes_namespace: Option<String>,
    /// New Kubernetes API endpoint
    kubernetes_api_endpoint: Option<String>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl ContextUpdateCommand {
    /// Create a new context update command
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            docker_host: None,
            default_stack_orchestrator: None,
            docker_api_endpoint: None,
            kubernetes_config_file: None,
            kubernetes_context: None,
            kubernetes_namespace: None,
            kubernetes_api_endpoint: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Update context description
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Update Docker host
    #[must_use]
    pub fn docker_host(mut self, host: impl Into<String>) -> Self {
        self.docker_host = Some(host.into());
        self
    }

    /// Update default stack orchestrator (swarm|kubernetes|all)
    #[must_use]
    pub fn default_stack_orchestrator(mut self, orchestrator: impl Into<String>) -> Self {
        self.default_stack_orchestrator = Some(orchestrator.into());
        self
    }

    /// Update Docker API endpoint
    #[must_use]
    pub fn docker_api_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.docker_api_endpoint = Some(endpoint.into());
        self
    }

    /// Update Kubernetes config file
    #[must_use]
    pub fn kubernetes_config_file(mut self, file: impl Into<String>) -> Self {
        self.kubernetes_config_file = Some(file.into());
        self
    }

    /// Update Kubernetes context
    #[must_use]
    pub fn kubernetes_context(mut self, context: impl Into<String>) -> Self {
        self.kubernetes_context = Some(context.into());
        self
    }

    /// Update Kubernetes namespace
    #[must_use]
    pub fn kubernetes_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.kubernetes_namespace = Some(namespace.into());
        self
    }

    /// Update Kubernetes API endpoint
    #[must_use]
    pub fn kubernetes_api_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.kubernetes_api_endpoint = Some(endpoint.into());
        self
    }
}

#[async_trait]
impl DockerCommand for ContextUpdateCommand {
    type Output = CommandOutput;

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["context".to_string(), "update".to_string()];

        if let Some(desc) = &self.description {
            args.push("--description".to_string());
            args.push(desc.clone());
        }

        if let Some(host) = &self.docker_host {
            args.push("--docker".to_string());
            args.push(format!("host={host}"));
        }

        if let Some(orchestrator) = &self.default_stack_orchestrator {
            args.push("--default-stack-orchestrator".to_string());
            args.push(orchestrator.clone());
        }

        if let Some(endpoint) = &self.docker_api_endpoint {
            args.push("--docker".to_string());
            args.push(format!("api-endpoint={endpoint}"));
        }

        if let Some(file) = &self.kubernetes_config_file {
            args.push("--kubernetes".to_string());
            args.push(format!("config-file={file}"));
        }

        if let Some(context) = &self.kubernetes_context {
            args.push("--kubernetes".to_string());
            args.push(format!("context={context}"));
        }

        if let Some(namespace) = &self.kubernetes_namespace {
            args.push("--kubernetes".to_string());
            args.push(format!("namespace={namespace}"));
        }

        if let Some(endpoint) = &self.kubernetes_api_endpoint {
            args.push("--kubernetes".to_string());
            args.push(format!("api-endpoint={endpoint}"));
        }

        args.push(self.name.clone());

        args.extend(self.executor.raw_args.clone());
        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        self.execute_command(args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_update_basic() {
        let cmd = ContextUpdateCommand::new("test-context");
        let args = cmd.build_command_args();
        assert_eq!(args[0], "context");
        assert_eq!(args[1], "update");
        assert!(args.contains(&"test-context".to_string()));
    }

    #[test]
    fn test_context_update_with_description() {
        let cmd = ContextUpdateCommand::new("test-context").description("Updated description");
        let args = cmd.build_command_args();
        assert!(args.contains(&"--description".to_string()));
        assert!(args.contains(&"Updated description".to_string()));
    }

    #[test]
    fn test_context_update_with_docker_host() {
        let cmd = ContextUpdateCommand::new("remote").docker_host("tcp://127.0.0.1:2376");
        let args = cmd.build_command_args();
        assert!(args.contains(&"--docker".to_string()));
        assert!(args.contains(&"host=tcp://127.0.0.1:2376".to_string()));
    }
}
