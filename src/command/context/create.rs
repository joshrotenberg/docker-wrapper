//! Docker context create command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker context create command builder
///
/// Create a new Docker context.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::{ContextCreateCommand, DockerCommand};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a context for a remote Docker daemon
/// ContextCreateCommand::new("production")
///     .description("Production environment")
///     .docker_host("ssh://user@remote-host")
///     .execute()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ContextCreateCommand {
    /// Context name
    name: String,
    /// Context description
    description: Option<String>,
    /// Docker host
    docker_host: Option<String>,
    /// Default stack orchestrator
    default_stack_orchestrator: Option<String>,
    /// Docker API endpoint
    docker_api_endpoint: Option<String>,
    /// Kubernetes config file
    kubernetes_config_file: Option<String>,
    /// Kubernetes context
    kubernetes_context: Option<String>,
    /// Kubernetes namespace
    kubernetes_namespace: Option<String>,
    /// Kubernetes API endpoint
    kubernetes_api_endpoint: Option<String>,
    /// Create context from existing context
    from: Option<String>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl ContextCreateCommand {
    /// Create a new context create command
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
            from: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Set context description
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set Docker host
    #[must_use]
    pub fn docker_host(mut self, host: impl Into<String>) -> Self {
        self.docker_host = Some(host.into());
        self
    }

    /// Set default stack orchestrator (swarm|kubernetes|all)
    #[must_use]
    pub fn default_stack_orchestrator(mut self, orchestrator: impl Into<String>) -> Self {
        self.default_stack_orchestrator = Some(orchestrator.into());
        self
    }

    /// Set Docker API endpoint
    #[must_use]
    pub fn docker_api_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.docker_api_endpoint = Some(endpoint.into());
        self
    }

    /// Set Kubernetes config file
    #[must_use]
    pub fn kubernetes_config_file(mut self, file: impl Into<String>) -> Self {
        self.kubernetes_config_file = Some(file.into());
        self
    }

    /// Set Kubernetes context
    #[must_use]
    pub fn kubernetes_context(mut self, context: impl Into<String>) -> Self {
        self.kubernetes_context = Some(context.into());
        self
    }

    /// Set Kubernetes namespace
    #[must_use]
    pub fn kubernetes_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.kubernetes_namespace = Some(namespace.into());
        self
    }

    /// Set Kubernetes API endpoint
    #[must_use]
    pub fn kubernetes_api_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.kubernetes_api_endpoint = Some(endpoint.into());
        self
    }

    /// Create context from existing context
    #[must_use]
    pub fn from(mut self, context: impl Into<String>) -> Self {
        self.from = Some(context.into());
        self
    }
}

#[async_trait]
impl DockerCommand for ContextCreateCommand {
    type Output = CommandOutput;

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["context".to_string(), "create".to_string()];

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

        if let Some(from) = &self.from {
            args.push("--from".to_string());
            args.push(from.clone());
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
    fn test_context_create_basic() {
        let cmd = ContextCreateCommand::new("test-context");
        let args = cmd.build_command_args();
        assert_eq!(args[0], "context");
        assert_eq!(args[1], "create");
        assert!(args.contains(&"test-context".to_string()));
    }

    #[test]
    fn test_context_create_with_description() {
        let cmd =
            ContextCreateCommand::new("test-context").description("Test context for development");
        let args = cmd.build_command_args();
        assert!(args.contains(&"--description".to_string()));
        assert!(args.contains(&"Test context for development".to_string()));
    }

    #[test]
    fn test_context_create_with_docker_host() {
        let cmd = ContextCreateCommand::new("remote").docker_host("ssh://user@remote-host");
        let args = cmd.build_command_args();
        assert!(args.contains(&"--docker".to_string()));
        assert!(args.contains(&"host=ssh://user@remote-host".to_string()));
    }

    #[test]
    fn test_context_create_from_existing() {
        let cmd = ContextCreateCommand::new("new-context").from("default");
        let args = cmd.build_command_args();
        assert!(args.contains(&"--from".to_string()));
        assert!(args.contains(&"default".to_string()));
    }
}
