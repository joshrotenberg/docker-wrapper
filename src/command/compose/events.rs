//! Docker Compose events command implementation using unified trait pattern.

use crate::{
    compose::{ComposeCommand, ComposeConfig},
    error::Result,
    CommandExecutor, DockerCommand,
};
use async_trait::async_trait;
use serde::Deserialize;

/// Docker Compose events command builder.
#[derive(Debug, Clone)]
pub struct ComposeEventsCommand {
    /// Base command executor.
    pub executor: CommandExecutor,
    /// Base compose configuration.
    pub config: ComposeConfig,
    /// Output format as JSON.
    pub json: bool,
    /// Start timestamp.
    pub since: Option<String>,
    /// End timestamp.
    pub until: Option<String>,
    /// Services to get events for (empty for all).
    pub services: Vec<String>,
}

/// Event from Docker Compose.
#[derive(Debug, Clone, Deserialize)]
pub struct ComposeEvent {
    /// Time of the event.
    pub time: String,
    /// Type of the event.
    #[serde(rename = "type")]
    pub event_type: String,
    /// Action that occurred.
    pub action: String,
    /// Service name.
    pub service: Option<String>,
    /// Container ID.
    pub container: Option<String>,
    /// Additional attributes.
    pub attributes: Option<serde_json::Value>,
}

/// Result from compose events command.
#[derive(Debug, Clone)]
pub struct ComposeEventsResult {
    /// Raw stdout output.
    pub stdout: String,
    /// Raw stderr output.
    pub stderr: String,
    /// Success status.
    pub success: bool,
    /// Parsed events (if JSON format was used).
    pub events: Vec<ComposeEvent>,
    /// Services that were monitored.
    pub services: Vec<String>,
}

impl ComposeEventsCommand {
    /// Creates a new compose events command.
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            json: false,
            since: None,
            until: None,
            services: Vec::new(),
        }
    }

    /// Outputs events in JSON format.
    #[must_use]
    pub fn json(mut self) -> Self {
        self.json = true;
        self
    }

    /// Sets start timestamp for events.
    #[must_use]
    pub fn since(mut self, timestamp: impl Into<String>) -> Self {
        self.since = Some(timestamp.into());
        self
    }

    /// Sets end timestamp for events.
    #[must_use]
    pub fn until(mut self, timestamp: impl Into<String>) -> Self {
        self.until = Some(timestamp.into());
        self
    }

    /// Adds a service to monitor events for.
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Adds multiple services to monitor events for.
    #[must_use]
    pub fn services<I, S>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.services.extend(services.into_iter().map(Into::into));
        self
    }
}

impl Default for ComposeEventsCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposeEventsCommand {
    fn command_name() -> &'static str {
        <Self as ComposeCommand>::command_name()
    }
    type Output = ComposeEventsResult;

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        <Self as ComposeCommand>::build_command_args(self)
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = <Self as ComposeCommand>::build_command_args(self);
        let output = self.execute_command(args).await?;

        // Parse JSON events if JSON format was requested
        let events = if self.json {
            output
                .stdout
                .lines()
                .filter_map(|line| {
                    if line.trim().is_empty() {
                        None
                    } else {
                        serde_json::from_str(line).ok()
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        Ok(ComposeEventsResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            events,
            services: self.services.clone(),
        })
    }
}

impl ComposeCommand for ComposeEventsCommand {
    fn subcommand_name() -> &'static str {
        "events"
    }

    fn config(&self) -> &ComposeConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.json {
            args.push("--json".to_string());
        }

        if let Some(ref since) = self.since {
            args.push("--since".to_string());
            args.push(since.clone());
        }

        if let Some(ref until) = self.until {
            args.push("--until".to_string());
            args.push(until.clone());
        }

        // add service names at the end
        args.extend(self.services.clone());

        args
    }
}

impl ComposeEventsResult {
    /// Checks if the command was successful.
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Gets parsed events (if JSON format was used).
    #[must_use]
    pub fn events(&self) -> &[ComposeEvent] {
        &self.events
    }

    /// Gets the services that were monitored.
    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }

    /// Gets events for a specific service.
    #[must_use]
    pub fn events_for_service(&self, service: &str) -> Vec<&ComposeEvent> {
        self.events
            .iter()
            .filter(|event| event.service.as_ref().is_some_and(|s| s == service))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_events_basic() {
        let cmd = ComposeEventsCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"events".to_string()));
    }

    #[test]
    fn test_compose_events_with_json() {
        let cmd = ComposeEventsCommand::new().json();
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--json".to_string()));
    }

    #[test]
    fn test_compose_events_with_timestamps() {
        let cmd = ComposeEventsCommand::new()
            .since("2024-01-01T00:00:00")
            .until("2024-01-02T00:00:00");

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--since".to_string()));
        assert!(args.contains(&"2024-01-01T00:00:00".to_string()));
        assert!(args.contains(&"--until".to_string()));
        assert!(args.contains(&"2024-01-02T00:00:00".to_string()));
    }

    #[test]
    fn test_compose_events_with_services() {
        let cmd = ComposeEventsCommand::new().service("web").service("db");

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"db".to_string()));
    }

    #[test]
    fn test_compose_events_with_services_method() {
        let cmd = ComposeEventsCommand::new().services(vec!["cache", "queue"]);
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"cache".to_string()));
        assert!(args.contains(&"queue".to_string()));
    }

    #[test]
    fn test_compose_events_all_options() {
        let cmd = ComposeEventsCommand::new()
            .json()
            .since("2024-01-01")
            .until("2024-01-02")
            .services(vec!["web", "worker"]);

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--json".to_string()));
        assert!(args.contains(&"--since".to_string()));
        assert!(args.contains(&"2024-01-01".to_string()));
        assert!(args.contains(&"--until".to_string()));
        assert!(args.contains(&"2024-01-02".to_string()));
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"worker".to_string()));
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposeEventsCommand::new()
            .file("docker-compose.yml")
            .project_name("myapp")
            .json()
            .service("api");

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"myapp".to_string()));
        assert!(args.contains(&"--json".to_string()));
        assert!(args.contains(&"api".to_string()));
    }
}
