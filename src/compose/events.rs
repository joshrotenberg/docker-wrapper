//! Docker Compose events command implementation.

use crate::compose::{ComposeCommandV2 as ComposeCommand, ComposeConfig};
use crate::error::Result;
use async_trait::async_trait;
use serde::Deserialize;

/// Docker Compose events command
///
/// Stream container events for services.
#[derive(Debug, Clone, Default)]
pub struct ComposeEventsCommand {
    /// Base configuration
    pub config: ComposeConfig,
    /// Output format as JSON
    pub json: bool,
    /// Start timestamp
    pub since: Option<String>,
    /// End timestamp
    pub until: Option<String>,
    /// Services to get events for
    pub services: Vec<String>,
}

/// Event from Docker Compose
#[derive(Debug, Clone, Deserialize)]
pub struct ComposeEvent {
    /// Time of the event
    pub time: String,
    /// Type of the event
    #[serde(rename = "type")]
    pub event_type: String,
    /// Action that occurred
    pub action: String,
    /// Service name
    pub service: Option<String>,
    /// Container ID
    pub container: Option<String>,
    /// Additional attributes
    pub attributes: Option<serde_json::Value>,
}

/// Result from events command
#[derive(Debug, Clone)]
pub struct EventsResult {
    /// Raw output from the command
    pub output: String,
    /// Parsed events (if JSON format)
    pub events: Vec<ComposeEvent>,
}

impl ComposeEventsCommand {
    /// Create a new events command
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a compose file
    #[must_use]
    pub fn file<P: Into<std::path::PathBuf>>(mut self, file: P) -> Self {
        self.config.files.push(file.into());
        self
    }

    /// Set project name
    #[must_use]
    pub fn project_name(mut self, name: impl Into<String>) -> Self {
        self.config.project_name = Some(name.into());
        self
    }

    /// Output as JSON
    #[must_use]
    pub fn json(mut self) -> Self {
        self.json = true;
        self
    }

    /// Set start timestamp
    #[must_use]
    pub fn since(mut self, timestamp: impl Into<String>) -> Self {
        self.since = Some(timestamp.into());
        self
    }

    /// Set end timestamp
    #[must_use]
    pub fn until(mut self, timestamp: impl Into<String>) -> Self {
        self.until = Some(timestamp.into());
        self
    }

    /// Add a service to get events for
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Add multiple services
    #[must_use]
    pub fn services<I, S>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.services.extend(services.into_iter().map(Into::into));
        self
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["events".to_string()];

        // Add flags
        if self.json {
            args.push("--json".to_string());
        }

        // Add timestamps
        if let Some(since) = &self.since {
            args.push("--since".to_string());
            args.push(since.clone());
        }
        if let Some(until) = &self.until {
            args.push("--until".to_string());
            args.push(until.clone());
        }

        // Add services
        args.extend(self.services.clone());

        args
    }
}

#[async_trait]
impl ComposeCommand for ComposeEventsCommand {
    type Output = EventsResult;

    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    async fn execute_compose(&self, args: Vec<String>) -> Result<Self::Output> {
        let output = self.execute_compose_command(args).await?;

        // Parse events if JSON format
        let events = if self.json {
            output
                .stdout
                .lines()
                .filter_map(|line| serde_json::from_str(line).ok())
                .collect()
        } else {
            Vec::new()
        };

        Ok(EventsResult {
            output: output.stdout,
            events,
        })
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_args();
        self.execute_compose(args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_events_command_basic() {
        let cmd = ComposeEventsCommand::new();
        let args = cmd.build_args();
        assert_eq!(args[0], "events");
    }

    #[test]
    fn test_events_command_with_json() {
        let cmd = ComposeEventsCommand::new().json();
        let args = cmd.build_args();
        assert!(args.contains(&"--json".to_string()));
    }

    #[test]
    fn test_events_command_with_timestamps() {
        let cmd = ComposeEventsCommand::new()
            .since("2025-08-23T00:00:00")
            .until("2025-08-23T23:59:59");
        let args = cmd.build_args();
        assert!(args.contains(&"--since".to_string()));
        assert!(args.contains(&"2025-08-23T00:00:00".to_string()));
        assert!(args.contains(&"--until".to_string()));
        assert!(args.contains(&"2025-08-23T23:59:59".to_string()));
    }

    #[test]
    fn test_events_command_with_services() {
        let cmd = ComposeEventsCommand::new().service("web").service("db");
        let args = cmd.build_args();
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"db".to_string()));
    }
}
