//! Core types and data structures for the docker wrapper.
//!
//! This module provides type-safe wrappers around Docker concepts like
//! container IDs, image references, and configuration structures.

use serde::{Deserialize, Serialize};

use std::fmt;
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{Duration, SystemTime};

use crate::errors::DockerError;

/// A validated Docker container ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContainerId(String);

impl ContainerId {
    /// Create a new container ID with validation
    pub fn new(id: impl Into<String>) -> Result<Self, DockerError> {
        let id = id.into();
        if id.is_empty() {
            return Err(DockerError::invalid_config("Container ID cannot be empty"));
        }
        if id.len() < 12 || id.len() > 64 {
            return Err(DockerError::invalid_config(
                "Container ID must be between 12 and 64 characters",
            ));
        }
        if !id
            .chars()
            .all(|c| c.is_ascii_hexdigit() && (c.is_ascii_digit() || c.is_ascii_lowercase()))
        {
            return Err(DockerError::invalid_config(
                "Container ID must contain only lowercase hexadecimal characters",
            ));
        }
        Ok(Self(id))
    }

    /// Create a container ID without validation (for internal use)
    pub(crate) fn new_unchecked(id: String) -> Self {
        Self(id)
    }

    /// Get the container ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the short form of the container ID (first 12 characters)
    pub fn short(&self) -> &str {
        &self.0[..12.min(self.0.len())]
    }
}

impl fmt::Display for ContainerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ContainerId {
    type Err = DockerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// A validated Docker network ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkId(String);

impl NetworkId {
    /// Create a new network ID
    pub fn new(id: impl Into<String>) -> Result<Self, DockerError> {
        let id = id.into();
        if id.is_empty() {
            return Err(DockerError::invalid_config("Network ID cannot be empty"));
        }
        Ok(Self(id))
    }

    /// Create a new NetworkId without validation (for internal use)
    #[allow(dead_code)]
    pub(crate) fn new_unchecked(id: String) -> Self {
        Self(id)
    }

    /// Get the network ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NetworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Container status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerStatus {
    /// Container has been created but not started
    Created,
    /// Container is running
    Running {
        /// When the container was started
        started_at: SystemTime,
    },
    /// Container is paused
    Paused,
    /// Container has exited
    Exited {
        /// Exit code returned by the container
        exit_code: i32,
        /// When the container finished execution
        finished_at: SystemTime,
    },
    /// Container is dead (usually due to OOM or similar)
    Dead,
    /// Container is restarting
    Restarting,
}

impl fmt::Display for ContainerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Running { .. } => write!(f, "running"),
            Self::Paused => write!(f, "paused"),
            Self::Exited { exit_code, .. } => write!(f, "exited({})", exit_code),
            Self::Dead => write!(f, "dead"),
            Self::Restarting => write!(f, "restarting"),
        }
    }
}

/// Network driver types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkDriver {
    /// Bridge network (default)
    Bridge,
    /// Host network (shares host networking)
    Host,
    /// No networking
    None,
    /// Custom driver
    Custom(String),
}

impl fmt::Display for NetworkDriver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bridge => write!(f, "bridge"),
            Self::Host => write!(f, "host"),
            Self::None => write!(f, "none"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Port mapping configuration
///
/// Represents a mapping between a host port and a container port.
/// This follows Docker's port mapping format: `host_port:container_port`
///
/// # Example
/// ```
/// use docker_wrapper::types::{PortMapping, Protocol};
///
/// // Map host port 8080 to container port 80 (HTTP)
/// let mapping = PortMapping::new(80).host_port(8080);
/// assert_eq!(mapping.host_port, Some(8080));
/// assert_eq!(mapping.container_port, 80);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortMapping {
    /// Host IP address to bind to (None for all interfaces)
    pub host_ip: Option<IpAddr>,
    /// Host port (None for dynamic allocation)
    pub host_port: Option<u16>,
    /// Container port
    pub container_port: u16,
    /// Protocol (TCP or UDP)
    pub protocol: Protocol,
}

impl PortMapping {
    /// Create a new port mapping
    #[must_use]
    pub fn new(container_port: u16) -> Self {
        Self {
            host_ip: None,
            host_port: None,
            container_port,
            protocol: Protocol::Tcp,
        }
    }

    /// Set the host port (0 for dynamic allocation)
    ///
    /// Sets the host machine port that will map to the container port.
    /// Use 0 or None for dynamic port allocation by Docker.
    ///
    /// # Arguments
    /// * `port` - Host port number (0 for dynamic allocation)
    ///
    /// # Example
    /// ```
    /// // Static port mapping: host 8080 -> container 80
    /// let mapping = PortMapping::new(80).host_port(8080);
    ///
    /// // Dynamic port mapping: random host port -> container 80
    /// let mapping = PortMapping::new(80).host_port(0);
    /// ```
    #[must_use]
    pub fn host_port(mut self, port: u16) -> Self {
        self.host_port = if port == 0 { None } else { Some(port) };
        self
    }

    /// Set the host IP
    #[must_use]
    pub fn host_ip(mut self, ip: IpAddr) -> Self {
        self.host_ip = Some(ip);
        self
    }

    /// Set the protocol
    #[must_use]
    pub fn protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = protocol;
        self
    }
}

/// Network protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    /// TCP protocol
    Tcp,
    /// UDP protocol
    Udp,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tcp => write!(f, "tcp"),
            Self::Udp => write!(f, "udp"),
        }
    }
}

/// Volume mount configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VolumeMount {
    /// Source of the mount
    pub source: VolumeSource,
    /// Target path in the container
    pub target: PathBuf,
    /// Mount mode
    pub mode: MountMode,
}

impl VolumeMount {
    /// Create a new volume mount
    #[must_use]
    pub fn new(source: VolumeSource, target: impl Into<PathBuf>) -> Self {
        Self {
            source,
            target: target.into(),
            mode: MountMode::ReadWrite,
        }
    }

    /// Set the mount as read-only
    #[must_use]
    pub fn read_only(mut self) -> Self {
        self.mode = MountMode::ReadOnly;
        self
    }

    /// Convert to CLI argument format (e.g., "volume:/path:ro")
    pub fn to_cli_arg(&self) -> String {
        match &self.source {
            VolumeSource::Named(name) => {
                if self.mode == MountMode::ReadOnly {
                    format!("{}:{}:ro", name, self.target.display())
                } else {
                    format!("{}:{}", name, self.target.display())
                }
            }
            VolumeSource::HostPath(path) => {
                if self.mode == MountMode::ReadOnly {
                    format!("{}:{}:ro", path.display(), self.target.display())
                } else {
                    format!("{}:{}", path.display(), self.target.display())
                }
            }
            VolumeSource::Anonymous => {
                if self.mode == MountMode::ReadOnly {
                    format!("{}:ro", self.target.display())
                } else {
                    self.target.to_string_lossy().to_string()
                }
            }
        }
    }
}

/// Volume source types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolumeSource {
    /// Named Docker volume
    Named(String),
    /// Host path
    HostPath(PathBuf),
    /// Anonymous volume
    Anonymous,
}

impl fmt::Display for VolumeSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Named(name) => write!(f, "{}", name),
            Self::HostPath(path) => write!(f, "{}", path.display()),
            Self::Anonymous => write!(f, "<anonymous>"),
        }
    }
}

impl VolumeSource {
    /// Create a named volume source
    #[must_use]
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }

    /// Create a host path volume source
    #[must_use]
    pub fn host_path(path: impl Into<PathBuf>) -> Self {
        Self::HostPath(path.into())
    }

    /// Create an anonymous volume source
    #[must_use]
    pub fn anonymous() -> Self {
        Self::Anonymous
    }
}

/// Volume mount mode
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MountMode {
    /// Read-write access
    ReadWrite,
    /// Read-only access
    ReadOnly,
}

impl fmt::Display for MountMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadWrite => write!(f, "rw"),
            Self::ReadOnly => write!(f, "ro"),
        }
    }
}

/// Resource limits for containers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Memory limit in bytes
    pub memory: Option<u64>,
    /// Memory + swap limit in bytes
    pub memory_swap: Option<u64>,
    /// CPU shares (relative weight)
    pub cpu_shares: Option<u64>,
    /// CPU quota in microseconds per period
    pub cpu_quota: Option<u64>,
    /// CPU period in microseconds
    pub cpu_period: Option<u64>,
    /// CPUs that can be used (e.g., "0-3" or "1,3")
    pub cpuset_cpus: Option<String>,
    /// Maximum number of PIDs
    pub pids_limit: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            memory: None,
            memory_swap: None,
            cpu_shares: None,
            cpu_quota: None,
            cpu_period: None,
            cpuset_cpus: None,
            pids_limit: None,
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Command to run for health check
    pub test: Vec<String>,
    /// Interval between health checks
    pub interval: Duration,
    /// Timeout for each health check
    pub timeout: Duration,
    /// Number of consecutive failures needed to consider container unhealthy
    pub retries: u32,
    /// Time to wait before running first health check
    pub start_period: Option<Duration>,
}

impl Default for HealthCheck {
    fn default() -> Self {
        Self {
            test: vec!["CMD".to_string(), "true".to_string()],
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(30),
            retries: 3,
            start_period: None,
        }
    }
}

/// Restart policy for containers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestartPolicy {
    /// Never restart
    No,
    /// Always restart
    Always,
    /// Restart unless stopped
    UnlessStopped,
    /// Restart on failure with optional max retry count
    OnFailure {
        /// Maximum number of retries before giving up
        max_retries: Option<u32>,
    },
}

impl Default for RestartPolicy {
    fn default() -> Self {
        Self::No
    }
}

impl fmt::Display for RestartPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::No => write!(f, "no"),
            Self::Always => write!(f, "always"),
            Self::UnlessStopped => write!(f, "unless-stopped"),
            Self::OnFailure { max_retries: None } => write!(f, "on-failure"),
            Self::OnFailure {
                max_retries: Some(retries),
            } => write!(f, "on-failure:{}", retries),
        }
    }
}

/// Collection of port mappings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortMappings(Vec<PortMapping>);

impl PortMappings {
    /// Create a new empty port mappings collection
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Add a port mapping
    #[must_use]
    pub fn add(mut self, mapping: PortMapping) -> Self {
        self.0.push(mapping);
        self
    }

    /// Add a simple port mapping (container port -> dynamic host port)
    #[must_use]
    pub fn add_port(self, container_port: u16) -> Self {
        self.add(PortMapping::new(container_port))
    }

    /// Add a port mapping with specific host port
    ///
    /// Maps host_port on the host to container_port inside the container.
    /// This follows Docker's `-p host_port:container_port` format.
    ///
    /// # Arguments
    /// * `host_port` - Port on the host machine (external port)
    /// * `container_port` - Port inside the container (internal port)
    ///
    /// # Example
    /// ```
    /// // Maps host port 8080 to container port 80
    /// let mappings = PortMappings::new().add_port_binding(8080, 80);
    /// ```
    #[must_use]
    pub fn add_port_binding(self, host_port: u16, container_port: u16) -> Self {
        self.add(PortMapping::new(container_port).host_port(host_port))
    }

    /// Get all port mappings
    pub fn mappings(&self) -> &[PortMapping] {
        &self.0
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of port mappings
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<Vec<PortMapping>> for PortMappings {
    fn from(mappings: Vec<PortMapping>) -> Self {
        Self(mappings)
    }
}

impl From<PortMappings> for Vec<PortMapping> {
    fn from(mappings: PortMappings) -> Self {
        mappings.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_id_validation() {
        // Valid container ID
        assert!(ContainerId::new("abcdef1234567890").is_ok());

        // Too short
        assert!(ContainerId::new("abc").is_err());

        // Too long
        assert!(ContainerId::new("a".repeat(100)).is_err());

        // Invalid characters
        assert!(ContainerId::new("invalid-chars!").is_err());

        // Empty
        assert!(ContainerId::new("").is_err());
    }

    #[test]
    fn test_port_mapping() {
        let mapping = PortMapping::new(8080)
            .host_port(8080)
            .protocol(Protocol::Tcp);

        assert_eq!(mapping.container_port, 8080);
        assert_eq!(mapping.host_port, Some(8080));
        assert_eq!(mapping.protocol, Protocol::Tcp);
    }

    #[test]
    fn test_port_mappings_collection() {
        let mappings = PortMappings::new()
            .add_port(8080)
            .add_port_binding(9090, 9000);

        assert_eq!(mappings.len(), 2);
        assert!(!mappings.is_empty());
    }

    #[test]
    fn test_volume_mount() {
        let mount = VolumeMount::new(
            VolumeSource::HostPath("/host/path".into()),
            "/container/path",
        )
        .read_only();

        assert_eq!(mount.mode, MountMode::ReadOnly);
        assert_eq!(mount.target, PathBuf::from("/container/path"));
    }

    #[test]
    fn test_container_status_display() {
        assert_eq!(ContainerStatus::Created.to_string(), "created");
        assert_eq!(
            ContainerStatus::Running {
                started_at: SystemTime::now()
            }
            .to_string(),
            "running"
        );
        assert_eq!(
            ContainerStatus::Exited {
                exit_code: 1,
                finished_at: SystemTime::now()
            }
            .to_string(),
            "exited(1)"
        );
    }

    #[test]
    fn test_restart_policy_display() {
        assert_eq!(RestartPolicy::No.to_string(), "no");
        assert_eq!(RestartPolicy::Always.to_string(), "always");
        assert_eq!(
            RestartPolicy::OnFailure {
                max_retries: Some(3)
            }
            .to_string(),
            "on-failure:3"
        );
    }
}
