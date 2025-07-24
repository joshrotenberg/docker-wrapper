//! Docker events monitoring and streaming module.
//!
//! This module provides comprehensive Docker event monitoring capabilities including:
//! - Real-time event streaming from Docker daemon
//! - Event filtering by type, container, image, network, volume
//! - Container lifecycle event handling
//! - System event monitoring
//! - Event processing with custom callbacks
//!
//! # Example
//!
//! ```rust,no_run
//! use docker_wrapper::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), DockerError> {
//!     let client = DockerClient::new().await?;
//!     let event_monitor = client.events();
//!
//!     // Monitor all container events
//!     let filter = EventFilter::new()
//!         .event_type(EventType::Container)
//!         .action("start")
//!         .action("stop");
//!
//!     let mut stream = event_monitor.stream(filter).await?;
//!
//!     while let Some(event) = stream.next().await {
//!         match event? {
//!             DockerEvent::Container(container_event) => {
//!                 println!("Container {}: {}",
//!                     container_event.actor.id,
//!                     container_event.action
//!                 );
//!             }
//!             _ => {}
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

use crate::client::DockerClient;
use crate::errors::{DockerError, DockerResult};
use crate::executor::ExecutionConfig;
use crate::types::{ContainerId, NetworkId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::process::Child;
use tokio::sync::mpsc;
use tokio_stream::{Stream, wrappers::ReceiverStream};

/// Docker event types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    /// Container events (start, stop, create, destroy, etc.)
    Container,
    /// Image events (pull, push, delete, tag, etc.)
    Image,
    /// Network events (create, connect, disconnect, destroy, etc.)
    Network,
    /// Volume events (create, mount, unmount, destroy, etc.)
    Volume,
    /// Plugin events
    Plugin,
    /// Service events (for Swarm mode)
    Service,
    /// Node events (for Swarm mode)
    Node,
    /// Secret events (for Swarm mode)
    Secret,
    /// Config events (for Swarm mode)
    Config,
    /// Daemon events
    Daemon,
}

impl EventType {
    /// Get event type as string
    pub fn as_str(&self) -> &str {
        match self {
            Self::Container => "container",
            Self::Image => "image",
            Self::Network => "network",
            Self::Volume => "volume",
            Self::Plugin => "plugin",
            Self::Service => "service",
            Self::Node => "node",
            Self::Secret => "secret",
            Self::Config => "config",
            Self::Daemon => "daemon",
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Docker event filter for customizing event streams
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    /// Filter by event type
    pub event_types: Vec<EventType>,
    /// Filter by container ID or name
    pub containers: Vec<String>,
    /// Filter by image name
    pub images: Vec<String>,
    /// Filter by network ID or name
    pub networks: Vec<String>,
    /// Filter by volume name
    pub volumes: Vec<String>,
    /// Filter by event action
    pub actions: Vec<String>,
    /// Filter by labels
    pub labels: HashMap<String, Option<String>>,
    /// Start time for events (Unix timestamp)
    pub since: Option<u64>,
    /// End time for events (Unix timestamp)
    pub until: Option<u64>,
}

impl EventFilter {
    /// Create a new event filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by event type
    pub fn event_type(mut self, event_type: EventType) -> Self {
        self.event_types.push(event_type);
        self
    }

    /// Filter by multiple event types
    pub fn event_types(mut self, types: Vec<EventType>) -> Self {
        self.event_types.extend(types);
        self
    }

    /// Filter by container ID or name
    pub fn container(mut self, container: impl Into<String>) -> Self {
        self.containers.push(container.into());
        self
    }

    /// Filter by image name
    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.images.push(image.into());
        self
    }

    /// Filter by network ID or name
    pub fn network(mut self, network: impl Into<String>) -> Self {
        self.networks.push(network.into());
        self
    }

    /// Filter by volume name
    pub fn volume(mut self, volume: impl Into<String>) -> Self {
        self.volumes.push(volume.into());
        self
    }

    /// Filter by event action
    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.actions.push(action.into());
        self
    }

    /// Filter by label key only
    pub fn label_key(mut self, key: impl Into<String>) -> Self {
        self.labels.insert(key.into(), None);
        self
    }

    /// Filter by label key-value pair
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), Some(value.into()));
        self
    }

    /// Set start time for events
    pub fn since(mut self, timestamp: SystemTime) -> Self {
        self.since = Some(
            timestamp
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        self
    }

    /// Set end time for events
    pub fn until(mut self, timestamp: SystemTime) -> Self {
        self.until = Some(
            timestamp
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        self
    }

    /// Set relative start time (e.g., last 5 minutes)
    pub fn since_duration(mut self, duration: std::time::Duration) -> Self {
        let now = SystemTime::now();
        if let Some(start_time) = now.checked_sub(duration) {
            self.since = Some(
                start_time
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            );
        }
        self
    }
}

/// Actor information for Docker events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventActor {
    /// Actor ID (container ID, image ID, etc.)
    #[serde(rename = "ID")]
    pub id: String,
    /// Actor attributes (labels, names, etc.)
    #[serde(rename = "Attributes")]
    pub attributes: HashMap<String, String>,
}

impl EventActor {
    /// Get actor name from attributes
    pub fn name(&self) -> Option<&str> {
        self.attributes.get("name").map(String::as_str)
    }

    /// Get actor image from attributes (for containers)
    pub fn image(&self) -> Option<&str> {
        self.attributes.get("image").map(String::as_str)
    }

    /// Get label value
    pub fn label(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(String::as_str)
    }
}

/// Base Docker event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEvent {
    /// Event type
    #[serde(rename = "Type")]
    pub event_type: String,
    /// Event action
    #[serde(rename = "Action")]
    pub action: String,
    /// Actor information
    #[serde(rename = "Actor")]
    pub actor: EventActor,
    /// Event timestamp (Unix timestamp in nanoseconds)
    #[serde(rename = "time")]
    pub time: u64,
    /// Event timestamp in nanoseconds
    #[serde(rename = "timeNano")]
    pub time_nano: u64,
}

impl BaseEvent {
    /// Get event time as SystemTime
    pub fn timestamp(&self) -> SystemTime {
        UNIX_EPOCH + std::time::Duration::from_secs(self.time)
    }

    /// Get precise event time as SystemTime
    pub fn timestamp_precise(&self) -> SystemTime {
        UNIX_EPOCH + std::time::Duration::from_nanos(self.time_nano)
    }
}

/// Container-specific event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerEvent {
    /// Base event information
    #[serde(flatten)]
    pub base: BaseEvent,
}

impl ContainerEvent {
    /// Get container ID
    pub fn container_id(&self) -> ContainerId {
        ContainerId::new_unchecked(self.base.actor.id.clone())
    }

    /// Get container name
    pub fn container_name(&self) -> Option<&str> {
        self.base.actor.name()
    }

    /// Get container image
    pub fn container_image(&self) -> Option<&str> {
        self.base.actor.image()
    }

    /// Check if this is a lifecycle event
    pub fn is_lifecycle_event(&self) -> bool {
        matches!(
            self.base.action.as_str(),
            "create" | "start" | "stop" | "restart" | "pause" | "unpause" | "destroy" | "die"
        )
    }

    /// Check if container is starting
    pub fn is_starting(&self) -> bool {
        matches!(self.base.action.as_str(), "start" | "restart")
    }

    /// Check if container is stopping
    pub fn is_stopping(&self) -> bool {
        matches!(self.base.action.as_str(), "stop" | "die" | "destroy")
    }
}

/// Image-specific event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEvent {
    /// Base event information
    #[serde(flatten)]
    pub base: BaseEvent,
}

impl ImageEvent {
    /// Get image ID
    pub fn image_id(&self) -> &str {
        &self.base.actor.id
    }

    /// Get image name/tag
    pub fn image_name(&self) -> Option<&str> {
        self.base.actor.name()
    }

    /// Check if this is a pull event
    pub fn is_pull(&self) -> bool {
        self.base.action == "pull"
    }

    /// Check if this is a push event
    pub fn is_push(&self) -> bool {
        self.base.action == "push"
    }

    /// Check if this is a delete event
    pub fn is_delete(&self) -> bool {
        matches!(self.base.action.as_str(), "delete" | "untag")
    }
}

/// Network-specific event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvent {
    /// Base event information
    #[serde(flatten)]
    pub base: BaseEvent,
}

impl NetworkEvent {
    /// Get network ID
    pub fn network_id(&self) -> DockerResult<NetworkId> {
        NetworkId::new(&self.base.actor.id)
    }

    /// Get network name
    pub fn network_name(&self) -> Option<&str> {
        self.base.actor.name()
    }

    /// Get connected container (for connect/disconnect events)
    pub fn container(&self) -> Option<&str> {
        self.base
            .actor
            .attributes
            .get("container")
            .map(|s| s.as_str())
    }

    /// Check if this is a connection event
    pub fn is_connection_event(&self) -> bool {
        matches!(self.base.action.as_str(), "connect" | "disconnect")
    }
}

/// Volume-specific event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeEvent {
    /// Base event information
    #[serde(flatten)]
    pub base: BaseEvent,
}

impl VolumeEvent {
    /// Get volume name
    pub fn volume_name(&self) -> &str {
        &self.base.actor.id
    }

    /// Check if this is a mount event
    pub fn is_mount(&self) -> bool {
        self.base.action == "mount"
    }

    /// Check if this is an unmount event
    pub fn is_unmount(&self) -> bool {
        self.base.action == "unmount"
    }
}

/// Unified Docker event enum
#[derive(Debug, Clone)]
pub enum DockerEvent {
    /// Container event
    Container(ContainerEvent),
    /// Image event
    Image(ImageEvent),
    /// Network event
    Network(NetworkEvent),
    /// Volume event
    Volume(VolumeEvent),
    /// Unknown event type
    Unknown(BaseEvent),
}

impl DockerEvent {
    /// Parse raw event JSON into typed event
    pub fn parse(json: &str) -> DockerResult<Self> {
        let base: BaseEvent = serde_json::from_str(json)
            .map_err(|e| DockerError::ParseError(format!("Invalid event JSON: {}", e)))?;

        match base.event_type.as_str() {
            "container" => Ok(Self::Container(ContainerEvent { base })),
            "image" => Ok(Self::Image(ImageEvent { base })),
            "network" => Ok(Self::Network(NetworkEvent { base })),
            "volume" => Ok(Self::Volume(VolumeEvent { base })),
            _ => Ok(Self::Unknown(base)),
        }
    }

    /// Get the base event information
    pub fn base(&self) -> &BaseEvent {
        match self {
            Self::Container(e) => &e.base,
            Self::Image(e) => &e.base,
            Self::Network(e) => &e.base,
            Self::Volume(e) => &e.base,
            Self::Unknown(e) => e,
        }
    }

    /// Get event timestamp
    pub fn timestamp(&self) -> SystemTime {
        self.base().timestamp()
    }

    /// Get event action
    pub fn action(&self) -> &str {
        &self.base().action
    }

    /// Get event type
    pub fn event_type(&self) -> &str {
        &self.base().event_type
    }
}

/// Event stream handle
pub struct EventStream {
    receiver: mpsc::Receiver<DockerResult<DockerEvent>>,
    _child: Child,
}

impl EventStream {
    /// Get the next event from the stream
    pub async fn next(&mut self) -> Option<DockerResult<DockerEvent>> {
        self.receiver.recv().await
    }

    /// Convert to a tokio Stream
    pub fn into_stream(self) -> ReceiverStream<DockerResult<DockerEvent>> {
        ReceiverStream::new(self.receiver)
    }
}

impl Stream for EventStream {
    type Item = DockerResult<DockerEvent>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}

/// Event statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct EventStats {
    /// Total events processed
    pub total_events: u64,
    /// Events by type
    pub events_by_type: HashMap<String, u64>,
    /// Events by action
    pub events_by_action: HashMap<String, u64>,
    /// Events in last minute
    pub events_last_minute: u64,
    /// Processing start time
    pub start_time: Option<SystemTime>,
}

impl EventStats {
    /// Create new event statistics
    pub fn new() -> Self {
        Self {
            start_time: Some(SystemTime::now()),
            ..Default::default()
        }
    }

    /// Record an event
    pub fn record_event(&mut self, event: &DockerEvent) {
        self.total_events += 1;

        let event_type = event.event_type().to_string();
        *self.events_by_type.entry(event_type).or_insert(0) += 1;

        let action = event.action().to_string();
        *self.events_by_action.entry(action).or_insert(0) += 1;
    }

    /// Get events per second
    pub fn events_per_second(&self) -> f64 {
        if let Some(start_time) = self.start_time {
            if let Ok(duration) = SystemTime::now().duration_since(start_time) {
                let seconds = duration.as_secs_f64();
                if seconds > 0.0 {
                    return self.total_events as f64 / seconds;
                }
            }
        }
        0.0
    }
}

/// Docker events manager
pub struct EventManager<'a> {
    client: &'a DockerClient,
}

impl<'a> EventManager<'a> {
    /// Create a new event manager
    pub fn new(client: &'a DockerClient) -> Self {
        Self { client }
    }

    /// Start streaming Docker events with optional filtering
    pub async fn stream(&self, filter: EventFilter) -> DockerResult<EventStream> {
        let mut args = vec![
            "events".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ];

        // Add event type filters
        for event_type in &filter.event_types {
            args.push("--filter".to_string());
            args.push(format!("type={}", event_type.as_str()));
        }

        // Add container filters
        for container in &filter.containers {
            args.push("--filter".to_string());
            args.push(format!("container={}", container));
        }

        // Add image filters
        for image in &filter.images {
            args.push("--filter".to_string());
            args.push(format!("image={}", image));
        }

        // Add network filters
        for network in &filter.networks {
            args.push("--filter".to_string());
            args.push(format!("network={}", network));
        }

        // Add volume filters
        for volume in &filter.volumes {
            args.push("--filter".to_string());
            args.push(format!("volume={}", volume));
        }

        // Add action filters
        for action in &filter.actions {
            args.push("--filter".to_string());
            args.push(format!("event={}", action));
        }

        // Add label filters
        for (key, value) in &filter.labels {
            args.push("--filter".to_string());
            if let Some(val) = value {
                args.push(format!("label={}={}", key, val));
            } else {
                args.push(format!("label={}", key));
            }
        }

        // Add time filters
        if let Some(since) = filter.since {
            args.push("--since".to_string());
            args.push(since.to_string());
        }

        if let Some(until) = filter.until {
            args.push("--until".to_string());
            args.push(until.to_string());
        }

        // Start streaming process
        let child = self
            .client
            .executor()
            .execute_streaming(&args, Some(ExecutionConfig::default()))
            .await?;

        let mut stdout = child.stdout;

        let (tx, rx) = mpsc::channel(1000);

        // Spawn task to process events
        tokio::spawn(async move {
            while let Some(line_result) = stdout.recv().await {
                match line_result {
                    Ok(line) => {
                        if line.trim().is_empty() {
                            continue;
                        }

                        let result = DockerEvent::parse(&line);
                        if tx.send(result).await.is_err() {
                            break; // Receiver dropped
                        }
                    }
                    Err(e) => {
                        if tx.send(Err(e)).await.is_err() {
                            break; // Receiver dropped
                        }
                    }
                }
            }
        });

        Ok(EventStream {
            receiver: rx,
            _child: child.child,
        })
    }

    /// Stream events with callback processing
    pub async fn stream_with_callback<F>(
        &self,
        filter: EventFilter,
        mut callback: F,
    ) -> DockerResult<()>
    where
        F: FnMut(DockerEvent) -> bool + Send + 'static,
    {
        let mut stream = self.stream(filter).await?;

        while let Some(event_result) = stream.next().await {
            match event_result {
                Ok(event) => {
                    if !callback(event) {
                        break; // Stop if callback returns false
                    }
                }
                Err(e) => {
                    log::warn!("Event parsing error: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Get historical events (non-streaming)
    pub async fn get_events(&self, filter: EventFilter) -> DockerResult<Vec<DockerEvent>> {
        let mut stream = self.stream(filter).await?;
        let mut events = Vec::new();

        // Collect events for a short time or until stream ends
        let timeout = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while let Some(event_result) = stream.next().await {
                match event_result {
                    Ok(event) => events.push(event),
                    Err(e) => log::warn!("Event parsing error: {}", e),
                }
            }
        });

        let _ = timeout.await; // Ignore timeout error

        Ok(events)
    }

    /// Monitor container lifecycle events
    pub async fn monitor_container_lifecycle<F>(
        &self,
        container_filter: Option<String>,
        mut callback: F,
    ) -> DockerResult<()>
    where
        F: FnMut(ContainerEvent) -> bool + Send + 'static,
    {
        let mut filter = EventFilter::new()
            .event_type(EventType::Container)
            .action("create")
            .action("start")
            .action("restart")
            .action("stop")
            .action("die")
            .action("destroy");

        if let Some(container) = container_filter {
            filter = filter.container(container);
        }

        self.stream_with_callback(filter, move |event| {
            if let DockerEvent::Container(container_event) = event {
                callback(container_event)
            } else {
                true // Continue for non-container events
            }
        })
        .await
    }

    /// Wait for specific container event
    pub async fn wait_for_container_event(
        &self,
        container_id: &ContainerId,
        action: &str,
        timeout: std::time::Duration,
    ) -> DockerResult<ContainerEvent> {
        let filter = EventFilter::new()
            .event_type(EventType::Container)
            .container(container_id.as_str())
            .action(action);

        let mut stream = self.stream(filter).await?;

        let result = tokio::time::timeout(timeout, async {
            while let Some(event_result) = stream.next().await {
                if let Ok(DockerEvent::Container(container_event)) = event_result {
                    if container_event.base.action == action {
                        return Ok(container_event);
                    }
                }
            }
            Err(DockerError::Timeout {
                message: format!("Timeout waiting for {} event", action),
            })
        })
        .await;

        match result {
            Ok(event) => event,
            Err(_) => Err(DockerError::Timeout {
                message: format!("Timeout waiting for {} event", action),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_filter_builder() {
        let filter = EventFilter::new()
            .event_type(EventType::Container)
            .event_type(EventType::Image)
            .container("test-container")
            .action("start")
            .action("stop")
            .label("env", "test")
            .label_key("app");

        assert_eq!(filter.event_types.len(), 2);
        assert_eq!(filter.containers, vec!["test-container"]);
        assert_eq!(filter.actions, vec!["start", "stop"]);
        assert_eq!(filter.labels.len(), 2);
        assert_eq!(filter.labels.get("env"), Some(&Some("test".to_string())));
        assert_eq!(filter.labels.get("app"), Some(&None));
    }

    #[test]
    fn test_event_type_display() {
        assert_eq!(EventType::Container.to_string(), "container");
        assert_eq!(EventType::Image.to_string(), "image");
        assert_eq!(EventType::Network.to_string(), "network");
        assert_eq!(EventType::Volume.to_string(), "volume");
    }

    #[test]
    fn test_event_parsing() {
        let json = r#"{
            "Type": "container",
            "Action": "start",
            "Actor": {
                "ID": "abc123",
                "Attributes": {
                    "name": "test-container",
                    "image": "redis:alpine"
                }
            },
            "time": 1234567890,
            "timeNano": 1234567890000000000
        }"#;

        let event = DockerEvent::parse(json).unwrap();

        match event {
            DockerEvent::Container(container_event) => {
                assert_eq!(container_event.base.action, "start");
                assert_eq!(container_event.base.actor.id, "abc123");
                assert_eq!(container_event.container_name(), Some("test-container"));
                assert_eq!(container_event.container_image(), Some("redis:alpine"));
                assert!(container_event.is_starting());
                assert!(container_event.is_lifecycle_event());
            }
            _ => panic!("Expected container event"),
        }
    }

    #[test]
    fn test_event_stats() {
        let mut stats = EventStats::new();

        let json = r#"{
            "Type": "container",
            "Action": "start",
            "Actor": {"ID": "abc123", "Attributes": {}},
            "time": 1234567890,
            "timeNano": 1234567890000000000
        }"#;

        let event = DockerEvent::parse(json).unwrap();
        stats.record_event(&event);

        assert_eq!(stats.total_events, 1);
        assert_eq!(stats.events_by_type.get("container"), Some(&1));
        assert_eq!(stats.events_by_action.get("start"), Some(&1));
    }

    #[test]
    fn test_container_event_methods() {
        let json = r#"{
            "Type": "container",
            "Action": "stop",
            "Actor": {
                "ID": "abc123",
                "Attributes": {
                    "name": "test-container",
                    "image": "redis:alpine"
                }
            },
            "time": 1234567890,
            "timeNano": 1234567890000000000
        }"#;

        let event = DockerEvent::parse(json).unwrap();

        if let DockerEvent::Container(container_event) = event {
            assert!(container_event.is_stopping());
            assert!(!container_event.is_starting());
            assert!(container_event.is_lifecycle_event());
        }
    }

    #[test]
    fn test_image_event_methods() {
        let json = r#"{
            "Type": "image",
            "Action": "pull",
            "Actor": {
                "ID": "sha256:abc123",
                "Attributes": {
                    "name": "redis:alpine"
                }
            },
            "time": 1234567890,
            "timeNano": 1234567890000000000
        }"#;

        let event = DockerEvent::parse(json).unwrap();

        if let DockerEvent::Image(image_event) = event {
            assert!(image_event.is_pull());
            assert!(!image_event.is_push());
            assert!(!image_event.is_delete());
            assert_eq!(image_event.image_name(), Some("redis:alpine"));
        }
    }

    #[test]
    fn test_filter_since_duration() {
        let filter = EventFilter::new().since_duration(std::time::Duration::from_secs(300)); // Last 5 minutes

        assert!(filter.since.is_some());

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let since = filter.since.unwrap();

        // Should be approximately 5 minutes ago (within 1 second tolerance)
        assert!((now - since).abs_diff(300) <= 1);
    }
}
