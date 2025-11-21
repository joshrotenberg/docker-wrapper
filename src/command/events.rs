//! Docker events command implementation.
//!
//! This module provides the `docker events` command for getting real-time events from the Docker daemon.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Docker events command builder
///
/// Get real-time events from the Docker daemon.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::EventsCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Get all events
/// let events = EventsCommand::new()
///     .run()
///     .await?;
///
/// // Get container events only
/// let container_events = EventsCommand::new()
///     .filter("type", "container")
///     .format("json")
///     .run()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct EventsCommand {
    /// Event filters
    filters: Vec<(String, String)>,
    /// Output format
    format: Option<String>,
    /// Show events since timestamp
    since: Option<String>,
    /// Show events until timestamp  
    until: Option<String>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl EventsCommand {
    /// Create a new events command
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::EventsCommand;
    ///
    /// let cmd = EventsCommand::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            format: None,
            since: None,
            until: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Add a filter for events
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::EventsCommand;
    ///
    /// let cmd = EventsCommand::new()
    ///     .filter("type", "container")
    ///     .filter("event", "start")
    ///     .filter("container", "my-container");
    /// ```
    #[must_use]
    pub fn filter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.filters.push((key.into(), value.into()));
        self
    }

    /// Set output format
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::EventsCommand;
    ///
    /// // JSON format for programmatic parsing
    /// let cmd = EventsCommand::new().format("json");
    /// ```
    #[must_use]
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Show events created since this timestamp
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::EventsCommand;
    ///
    /// let cmd = EventsCommand::new()
    ///     .since("2023-01-01T00:00:00");
    /// ```
    #[must_use]
    pub fn since(mut self, since: impl Into<String>) -> Self {
        self.since = Some(since.into());
        self
    }

    /// Show events created until this timestamp
    #[must_use]
    pub fn until(mut self, until: impl Into<String>) -> Self {
        self.until = Some(until.into());
        self
    }

    /// Execute the events command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - Invalid filter or timestamp format
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::EventsCommand;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = EventsCommand::new()
    ///     .filter("type", "container")
    ///     .format("json")
    ///     .run()
    ///     .await?;
    ///
    /// if result.success() {
    ///     for event in result.parsed_events() {
    ///         println!("Event: {} on {}", event.action, event.actor.id);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<EventsResult> {
        let output = self.execute().await?;

        // Parse events if JSON format was used
        let parsed_events = if self.format.as_deref() == Some("json") {
            Self::parse_json_events(&output.stdout)
        } else {
            Vec::new()
        };

        Ok(EventsResult {
            output,
            parsed_events,
        })
    }

    /// Parse JSON events output into structured data
    fn parse_json_events(stdout: &str) -> Vec<DockerEvent> {
        let mut events = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Ok(event) = serde_json::from_str::<DockerEvent>(line) {
                events.push(event);
            }
        }

        events
    }
}

impl Default for EventsCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for EventsCommand {
    type Output = CommandOutput;

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["events".to_string()];

        for (key, value) in &self.filters {
            args.push("--filter".to_string());
            args.push(format!("{key}={value}"));
        }

        if let Some(ref format) = self.format {
            args.push("--format".to_string());
            args.push(format.clone());
        }

        if let Some(ref since) = self.since {
            args.push("--since".to_string());
            args.push(since.clone());
        }

        if let Some(ref until) = self.until {
            args.push("--until".to_string());
            args.push(until.clone());
        }

        args.extend(self.executor.raw_args.clone());
        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        let command_name = args[0].clone();
        let command_args = args[1..].to_vec();
        self.executor
            .execute_command(&command_name, command_args)
            .await
    }

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }
}

/// Result from the events command
#[derive(Debug, Clone)]
pub struct EventsResult {
    /// Raw command output
    pub output: CommandOutput,
    /// Parsed events (when JSON format is used)
    pub parsed_events: Vec<DockerEvent>,
}

impl EventsResult {
    /// Check if the events command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get parsed events (available when JSON format is used)
    #[must_use]
    pub fn parsed_events(&self) -> &[DockerEvent] {
        &self.parsed_events
    }

    /// Get the raw events output
    #[must_use]
    pub fn raw_output(&self) -> &str {
        &self.output.stdout
    }
}

/// Docker event information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerEvent {
    /// Event type (container, image, volume, network, etc.)
    #[serde(alias = "Type")]
    pub event_type: String,

    /// Action performed (start, stop, create, destroy, etc.)
    #[serde(alias = "Action")]
    pub action: String,

    /// Actor (object that performed the action)
    #[serde(alias = "Actor")]
    pub actor: EventActor,

    /// Timestamp of the event
    #[serde(alias = "time")]
    pub time: i64,

    /// Nanosecond timestamp
    #[serde(alias = "timeNano")]
    pub time_nano: i64,
}

/// Actor information for Docker events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventActor {
    /// Actor ID (container ID, image ID, etc.)
    #[serde(alias = "ID")]
    pub id: String,

    /// Actor attributes (name, image, etc.)
    #[serde(alias = "Attributes")]
    pub attributes: std::collections::HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_events_basic() {
        let cmd = EventsCommand::new();
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["events"]);
    }

    #[test]
    fn test_events_with_filters() {
        let cmd = EventsCommand::new()
            .filter("type", "container")
            .filter("event", "start");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec![
                "events",
                "--filter",
                "type=container",
                "--filter",
                "event=start"
            ]
        );
    }

    #[test]
    fn test_events_with_format() {
        let cmd = EventsCommand::new().format("json");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["events", "--format", "json"]);
    }

    #[test]
    fn test_events_with_since_until() {
        let cmd = EventsCommand::new()
            .since("2023-01-01T00:00:00")
            .until("2023-12-31T23:59:59");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec![
                "events",
                "--since",
                "2023-01-01T00:00:00",
                "--until",
                "2023-12-31T23:59:59"
            ]
        );
    }

    #[test]
    fn test_events_all_options() {
        let cmd = EventsCommand::new()
            .filter("type", "container")
            .filter("container", "my-app")
            .format("json")
            .since("1h")
            .until("now");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec![
                "events",
                "--filter",
                "type=container",
                "--filter",
                "container=my-app",
                "--format",
                "json",
                "--since",
                "1h",
                "--until",
                "now"
            ]
        );
    }

    #[test]
    fn test_parse_json_events() {
        let json_output = r#"{"Type":"container","Action":"start","Actor":{"ID":"abc123","Attributes":{"name":"test-container"}},"time":1640995200,"timeNano":1640995200000000000}"#;

        let events = EventsCommand::parse_json_events(json_output);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "container");
        assert_eq!(events[0].action, "start");
        assert_eq!(events[0].actor.id, "abc123");
    }

    #[test]
    fn test_parse_json_events_empty() {
        let events = EventsCommand::parse_json_events("");
        assert!(events.is_empty());
    }
}
