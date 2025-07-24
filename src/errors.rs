//! Error types for the docker wrapper.
//!
//! This module provides comprehensive error handling for all Docker operations,
//! with detailed context and recovery information.

use std::time::Duration;
use thiserror::Error;

/// Result type alias for docker operations
pub type DockerResult<T> = Result<T, DockerError>;

/// Comprehensive error type for all Docker operations
#[derive(Debug, Error)]
pub enum DockerError {
    /// Docker command not found or not executable
    #[error("Docker command not found: {path}")]
    DockerNotFound {
        /// Path where Docker binary was expected
        path: String,
    },

    /// Docker daemon is not running or not accessible
    #[error("Docker daemon not accessible: {message}")]
    DaemonNotAccessible {
        /// Details about the connection failure
        message: String,
    },

    /// Docker version is not supported
    #[error("Docker version {version} is not supported (minimum: {minimum})")]
    UnsupportedVersion {
        /// The detected Docker version
        version: String,
        /// The minimum required version
        minimum: String,
    },

    /// Container-related errors
    #[error("Container not found: {id}")]
    ContainerNotFound {
        /// Container ID that was not found
        id: String,
    },

    /// Container with the specified name already exists
    #[error("Container already exists: {name}")]
    ContainerAlreadyExists {
        /// Name of the existing container
        name: String,
    },

    /// Container is not in running state
    #[error("Container is not running: {id}")]
    ContainerNotRunning {
        /// ID of the container that is not running
        id: String,
    },

    /// Container failed to start
    #[error("Container failed to start: {id}, reason: {reason}")]
    ContainerStartFailed {
        /// ID of the container that failed to start
        id: String,
        /// Reason for the start failure
        reason: String,
    },

    /// Image-related errors
    #[error("Image not found: {image}")]
    ImageNotFound {
        /// Image reference that was not found
        image: String,
    },

    /// Image pull operation failed
    #[error("Image pull failed: {image}, reason: {reason}")]
    ImagePullFailed {
        /// Image reference that failed to pull
        image: String,
        /// Reason for the pull failure
        reason: String,
    },

    /// Image build operation failed
    #[error("Image build failed: {context}, reason: {reason}")]
    ImageBuildFailed {
        /// Build context path
        context: String,
        /// Reason for the build failure
        reason: String,
    },

    /// Network-related errors
    #[error("Network not found: {id}")]
    NetworkNotFound {
        /// Network ID that was not found
        id: String,
    },

    /// Network with the specified name already exists
    #[error("Network already exists: {name}")]
    NetworkAlreadyExists {
        /// Name of the existing network
        name: String,
    },

    /// Failed to connect container to network
    #[error("Failed to connect container {container_id} to network {network_id}")]
    NetworkConnectionFailed {
        /// ID of the container that failed to connect
        container_id: String,
        /// ID of the network that failed to accept the connection
        network_id: String,
    },

    /// Volume-related errors
    #[error("Volume not found: {name}")]
    VolumeNotFound {
        /// Volume name that was not found
        name: String,
    },

    /// Volume with the specified name already exists
    #[error("Volume already exists: {name}")]
    VolumeAlreadyExists {
        /// Name of the existing volume
        name: String,
    },

    /// Volume mount operation failed
    #[error("Volume mount failed: {message} (target: {target}, reason: {reason})")]
    VolumeMountFailed {
        /// General failure message
        message: String,
        /// Mount target path
        target: String,
        /// Specific reason for mount failure
        reason: String,
    },

    /// Command execution errors
    #[error("Docker command failed: {command}")]
    CommandFailed {
        /// The command that failed
        command: String,
        /// Exit code returned by the command
        exit_code: i32,
        /// Standard output from the command
        stdout: String,
        /// Standard error from the command
        stderr: String,
    },

    /// Command execution timed out
    #[error("Command timed out: {command} (timeout: {timeout:?})")]
    CommandTimeout {
        /// The command that timed out
        command: String,
        /// The timeout duration that was exceeded
        timeout: Duration,
    },

    /// Command execution was interrupted
    #[error("Command was interrupted: {command}")]
    CommandInterrupted {
        /// The command that was interrupted
        command: String,
    },

    /// Configuration and validation errors
    #[error("Invalid configuration: {message}")]
    InvalidConfig {
        /// Details about the invalid configuration
        message: String,
    },

    /// Container name is invalid
    #[error("Invalid container name '{name}': {reason}")]
    InvalidContainerName {
        /// The invalid container name
        name: String,
        /// Reason why the name is invalid
        reason: String,
    },

    /// Image reference is invalid
    #[error("Invalid image reference: {0}")]
    InvalidImageRef(String),

    /// Port mapping configuration is invalid
    #[error("Invalid port mapping '{mapping}': {reason}")]
    InvalidPortMapping {
        /// The invalid port mapping string
        mapping: String,
        /// Reason why the mapping is invalid
        reason: String,
    },

    /// Permission and access errors
    #[error("Permission denied: {operation}, reason: {reason}")]
    PermissionDenied {
        /// The operation that was denied
        operation: String,
        /// Reason for the permission denial
        reason: String,
    },

    /// Resource limit has been exceeded
    #[error("Resource limit exceeded for {resource}: {limit}")]
    ResourceLimitExceeded {
        /// The resource type that exceeded limits
        resource: String,
        /// The limit that was exceeded
        limit: String,
    },

    /// IO and system errors
    #[error("IO error during {operation}: {source}")]
    Io {
        /// The operation that caused the IO error
        operation: String,
        #[source]
        /// The underlying IO error
        source: std::io::Error,
    },

    /// JSON parsing errors
    #[error("JSON parsing error in {context}")]
    JsonError {
        /// Context where JSON parsing failed
        context: String,
        #[source]
        /// The underlying JSON parsing error
        source: serde_json::Error,
    },

    /// UTF-8 conversion errors
    #[error("UTF-8 conversion error in {context}: {source}")]
    Utf8 {
        /// Context where UTF-8 conversion failed
        context: String,
        #[source]
        /// The underlying UTF-8 conversion error
        source: std::string::FromUtf8Error,
    },

    /// Health check errors
    #[error("Health check failed: {message}")]
    HealthCheck {
        /// Details about the health check failure
        message: String,
    },

    /// Process execution errors
    #[error("Failed to spawn process: {message}")]
    ProcessSpawn {
        /// Details about the process spawn failure
        message: String,
    },

    /// Process wait operation failed
    #[error("Failed to wait for process: {message}")]
    ProcessWait {
        /// Details about the process wait failure
        message: String,
    },

    /// Parsing errors
    #[error("Parsing error: {message}")]
    Parsing {
        /// Details about the parsing error
        message: String,
    },

    /// Timeout errors
    #[error("Operation timed out: {message}")]
    Timeout {
        /// Details about the timeout
        message: String,
    },

    /// Network errors
    #[error("Network error: {message}")]
    Network {
        /// Details about the network error
        message: String,
    },

    /// Operation cancelled
    #[error("Operation cancelled: {message}")]
    Cancelled {
        /// Details about the cancelled operation
        message: String,
    },

    /// Resource not found
    #[error("Resource not found: {message}")]
    NotFound {
        /// Details about the missing resource
        message: String,
    },

    /// Execution errors
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Parse errors
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Generic errors with context
    #[error("Operation failed: {operation}, message: {message}")]
    Generic {
        /// The operation that failed
        operation: String,
        /// Error message details
        message: String,
    },
}

impl DockerError {
    /// Create a new docker not found error
    pub fn docker_not_found(path: impl Into<String>) -> Self {
        Self::DockerNotFound { path: path.into() }
    }

    /// Create a new daemon not accessible error
    pub fn daemon_not_accessible(message: impl Into<String>) -> Self {
        Self::DaemonNotAccessible {
            message: message.into(),
        }
    }

    /// Create a new unsupported version error
    pub fn unsupported_version(version: impl Into<String>, minimum: impl Into<String>) -> Self {
        Self::UnsupportedVersion {
            version: version.into(),
            minimum: minimum.into(),
        }
    }

    /// Create a new container not found error
    pub fn container_not_found(id: impl Into<String>) -> Self {
        Self::ContainerNotFound { id: id.into() }
    }

    /// Create a new container already exists error
    pub fn container_already_exists(name: impl Into<String>) -> Self {
        Self::ContainerAlreadyExists { name: name.into() }
    }

    /// Create a new container not running error
    pub fn container_not_running(id: impl Into<String>) -> Self {
        Self::ContainerNotRunning { id: id.into() }
    }

    /// Create a new container start failed error
    pub fn container_start_failed(id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ContainerStartFailed {
            id: id.into(),
            reason: reason.into(),
        }
    }

    /// Create a new image not found error
    pub fn image_not_found(image: impl Into<String>) -> Self {
        Self::ImageNotFound {
            image: image.into(),
        }
    }

    /// Create a new image pull failed error
    pub fn image_pull_failed(image: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ImagePullFailed {
            image: image.into(),
            reason: reason.into(),
        }
    }

    /// Create a new health check error
    pub fn health_check(message: impl Into<String>) -> Self {
        Self::HealthCheck {
            message: message.into(),
        }
    }

    /// Create a new process spawn error
    pub fn process_spawn(message: impl Into<String>) -> Self {
        Self::ProcessSpawn {
            message: message.into(),
        }
    }

    /// Create a new process wait error
    pub fn process_wait(message: impl Into<String>) -> Self {
        Self::ProcessWait {
            message: message.into(),
        }
    }

    /// Create a new parsing error
    pub fn parsing(message: impl Into<String>) -> Self {
        Self::Parsing {
            message: message.into(),
        }
    }

    /// Create a new timeout error
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::Timeout {
            message: message.into(),
        }
    }

    /// Create a new network error
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
        }
    }

    /// Create a new cancelled error
    pub fn cancelled(message: impl Into<String>) -> Self {
        Self::Cancelled {
            message: message.into(),
        }
    }

    /// Create a new not found error
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            message: message.into(),
        }
    }

    /// Create a new command failed error
    pub fn command_failed(
        command: impl Into<String>,
        exit_code: i32,
        stdout: impl Into<String>,
        stderr: impl Into<String>,
    ) -> Self {
        Self::CommandFailed {
            command: command.into(),
            exit_code,
            stdout: stdout.into(),
            stderr: stderr.into(),
        }
    }

    /// Creates an invalid image reference error
    pub fn invalid_image_ref(image: impl Into<String>) -> Self {
        Self::InvalidImageRef(image.into())
    }

    /// Create a new generic error
    pub fn generic(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Generic {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create a new command timeout error
    pub fn command_timeout(command: impl Into<String>, timeout: Duration) -> Self {
        Self::CommandTimeout {
            command: command.into(),
            timeout,
        }
    }

    /// Create a new invalid config error
    pub fn invalid_config(message: impl Into<String>) -> Self {
        Self::InvalidConfig {
            message: message.into(),
        }
    }

    /// Create a new permission denied error
    pub fn permission_denied(operation: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::PermissionDenied {
            operation: operation.into(),
            reason: reason.into(),
        }
    }

    /// Create a new IO error with context
    pub fn io(operation: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            operation: operation.into(),
            source,
        }
    }

    /// Create a new JSON error with context
    pub fn json(context: impl Into<String>, source: serde_json::Error) -> Self {
        Self::JsonError {
            context: context.into(),
            source,
        }
    }

    /// Create a new UTF-8 error with context
    pub fn utf8(context: impl Into<String>, source: std::string::FromUtf8Error) -> Self {
        Self::Utf8 {
            context: context.into(),
            source,
        }
    }

    /// Check if this error is recoverable (can be retried)
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        match self {
            // Network and temporary errors are usually recoverable
            Self::DaemonNotAccessible { .. }
            | Self::CommandTimeout { .. }
            | Self::CommandInterrupted { .. }
            | Self::ImagePullFailed { .. }
            | Self::ContainerStartFailed { .. }
            | Self::NetworkConnectionFailed { .. }
            | Self::Io { .. } => true,

            // Configuration and not found errors are usually not recoverable
            Self::DockerNotFound { .. } => false,
            Self::UnsupportedVersion { .. } => false,
            Self::ContainerNotFound { .. } => false,
            Self::ImageNotFound { .. } => false,
            Self::NetworkNotFound { .. } => false,
            Self::VolumeNotFound { .. } => false,
            Self::InvalidConfig { .. } => false,
            Self::InvalidContainerName { .. } => false,
            Self::InvalidImageRef { .. } => false,
            Self::InvalidPortMapping { .. } => false,
            Self::PermissionDenied { .. } => false,

            // Already exists errors might be recoverable depending on context
            Self::ContainerAlreadyExists { .. } => true,
            Self::NetworkAlreadyExists { .. } => true,
            Self::VolumeAlreadyExists { .. } => true,

            // Other errors default to not recoverable
            _ => false,
        }
    }

    /// Get the error category for grouping and handling
    #[must_use]
    pub fn category(&self) -> &'static str {
        match self {
            Self::DockerNotFound { .. } | Self::DaemonNotAccessible { .. } => "docker",
            Self::UnsupportedVersion { .. } => "version",
            Self::ContainerNotFound { .. }
            | Self::ContainerAlreadyExists { .. }
            | Self::ContainerNotRunning { .. }
            | Self::ContainerStartFailed { .. } => "container",
            Self::ImageNotFound { .. }
            | Self::ImagePullFailed { .. }
            | Self::ImageBuildFailed { .. } => "image",
            Self::NetworkNotFound { .. }
            | Self::NetworkAlreadyExists { .. }
            | Self::NetworkConnectionFailed { .. } => "network",
            Self::VolumeNotFound { .. }
            | Self::VolumeAlreadyExists { .. }
            | Self::VolumeMountFailed { .. } => "volume",
            Self::CommandFailed { .. }
            | Self::CommandTimeout { .. }
            | Self::CommandInterrupted { .. } => "command",
            Self::InvalidConfig { .. }
            | Self::InvalidContainerName { .. }
            | Self::InvalidImageRef { .. }
            | Self::InvalidPortMapping { .. } => "config",
            Self::PermissionDenied { .. } | Self::ResourceLimitExceeded { .. } => "permission",
            Self::Io { .. } => "io",
            Self::JsonError { .. } => "json",
            Self::Utf8 { .. } => "utf8",
            Self::HealthCheck { .. } => "health",
            Self::ProcessSpawn { .. } | Self::ProcessWait { .. } => "process",
            Self::Parsing { .. } => "parsing",
            Self::Timeout { .. } => "timeout",
            Self::Network { .. } => "network",
            Self::Cancelled { .. } => "cancelled",
            Self::NotFound { .. } => "not_found",
            Self::ExecutionError(_) => "execution",
            Self::ParseError(_) => "parse",
            Self::Generic { .. } => "generic",
        }
    }
}

/// Context information for better error reporting
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// The operation being performed
    pub operation: String,
    /// Optional container ID for context
    pub container_id: Option<String>,
    /// Optional image reference for context
    pub image: Option<String>,
    /// Optional network ID for context
    pub network_id: Option<String>,
    /// Additional context information
    pub additional: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            container_id: None,
            image: None,
            network_id: None,
            additional: std::collections::HashMap::new(),
        }
    }

    /// Add container ID to context
    pub fn with_container_id(mut self, id: impl Into<String>) -> Self {
        self.container_id = Some(id.into());
        self
    }

    /// Add image reference to context
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = Some(image.into());
        self
    }

    /// Add network ID to context
    pub fn with_network_id(mut self, id: impl Into<String>) -> Self {
        self.network_id = Some(id.into());
        self
    }

    /// Add additional context information
    pub fn with_additional(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.additional.insert(key.into(), value.into());
        self
    }
}

/// Extension trait for adding context to errors
pub trait ErrorExt<T> {
    /// Add context to the error
    fn with_context(self, context: ErrorContext) -> DockerResult<T>;
}

impl<T> ErrorExt<T> for DockerResult<T> {
    fn with_context(self, _context: ErrorContext) -> DockerResult<T> {
        self.map_err(|err| {
            // For now, just return the original error
            // In the future, we could wrap it with additional context
            err
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = DockerError::container_not_found("test-container");
        assert_eq!(err.category(), "container");
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_recoverable_errors() {
        let recoverable = DockerError::command_timeout("docker run", Duration::from_secs(30));
        assert!(recoverable.is_recoverable());

        let not_recoverable = DockerError::docker_not_found("/usr/bin/docker");
        assert!(!not_recoverable.is_recoverable());
    }

    #[test]
    fn test_error_categories() {
        assert_eq!(
            DockerError::container_not_found("test").category(),
            "container"
        );
        assert_eq!(DockerError::image_not_found("redis").category(), "image");
        assert_eq!(DockerError::docker_not_found("docker").category(), "docker");
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("test_operation")
            .with_container_id("container123")
            .with_image("redis:alpine")
            .with_additional("extra", "info");

        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.container_id, Some("container123".to_string()));
        assert_eq!(context.image, Some("redis:alpine".to_string()));
        assert_eq!(context.additional.get("extra"), Some(&"info".to_string()));
    }
}
