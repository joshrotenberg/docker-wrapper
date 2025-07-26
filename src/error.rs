//! Error types for the docker-wrapper crate.
//!
//! This module provides comprehensive error handling for all Docker operations,
//! with clear error messages and helpful context.

use thiserror::Error;

/// Result type for docker-wrapper operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for all docker-wrapper operations
#[derive(Error, Debug)]
pub enum Error {
    /// Docker binary not found in PATH
    #[error("Docker binary not found in PATH")]
    DockerNotFound,

    /// Docker daemon is not running
    #[error("Docker daemon is not running")]
    DaemonNotRunning,

    /// Docker version is not supported
    #[error("Docker version {found} is not supported (minimum: {minimum})")]
    UnsupportedVersion {
        /// The Docker version that was found
        found: String,
        /// The minimum required version
        minimum: String,
    },

    /// Failed to execute Docker command
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

    /// Failed to parse Docker output
    #[error("Failed to parse Docker output: {message}")]
    ParseError {
        /// Error message describing the parse failure
        message: String,
    },

    /// Invalid configuration provided
    #[error("Invalid configuration: {message}")]
    InvalidConfig {
        /// Error message describing the configuration issue
        message: String,
    },

    /// Docker container not found
    #[error("Container not found: {container_id}")]
    ContainerNotFound {
        /// The container ID that was not found
        container_id: String,
    },

    /// Docker image not found
    #[error("Image not found: {image}")]
    ImageNotFound {
        /// The image name that was not found
        image: String,
    },

    /// IO error occurred during operation
    #[error("IO error: {message}")]
    Io {
        /// Error message describing the IO failure
        message: String,
        /// The underlying IO error
        #[source]
        source: std::io::Error,
    },

    /// JSON parsing or serialization error
    #[error("JSON error: {message}")]
    Json {
        /// Error message describing the JSON failure
        message: String,
        /// The underlying JSON error
        #[source]
        source: serde_json::Error,
    },

    /// Operation timed out
    #[error("Operation timed out after {timeout_seconds} seconds")]
    Timeout {
        /// Number of seconds after which the operation timed out
        timeout_seconds: u64,
    },

    /// Operation was interrupted
    #[error("Operation was interrupted")]
    Interrupted,

    /// Generic error with custom message
    #[error("{message}")]
    Custom {
        /// Custom error message
        message: String,
    },
}

impl Error {
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

    /// Create a new parse error
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::ParseError {
            message: message.into(),
        }
    }

    /// Create a new invalid config error
    pub fn invalid_config(message: impl Into<String>) -> Self {
        Self::InvalidConfig {
            message: message.into(),
        }
    }

    /// Create a new container not found error
    pub fn container_not_found(container_id: impl Into<String>) -> Self {
        Self::ContainerNotFound {
            container_id: container_id.into(),
        }
    }

    /// Create a new image not found error
    pub fn image_not_found(image: impl Into<String>) -> Self {
        Self::ImageNotFound {
            image: image.into(),
        }
    }

    /// Create a new timeout error
    pub fn timeout(timeout_seconds: u64) -> Self {
        Self::Timeout { timeout_seconds }
    }

    /// Create a new custom error
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom {
            message: message.into(),
        }
    }

    /// Get the error category for logging and metrics
    pub fn category(&self) -> &'static str {
        match self {
            Self::DockerNotFound | Self::DaemonNotRunning | Self::UnsupportedVersion { .. } => {
                "prerequisites"
            }
            Self::CommandFailed { .. } | Self::Timeout { .. } | Self::Interrupted => "command",
            Self::ParseError { .. } | Self::Json { .. } => "parsing",
            Self::InvalidConfig { .. } => "config",
            Self::ContainerNotFound { .. } => "container",
            Self::ImageNotFound { .. } => "image",
            Self::Io { .. } => "io",
            Self::Custom { .. } => "custom",
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::CommandFailed { .. } | Self::Timeout { .. } | Self::Io { .. }
        )
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
            message: err.to_string(),
            source: err,
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json {
            message: err.to_string(),
            source: err,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categories() {
        assert_eq!(Error::DockerNotFound.category(), "prerequisites");
        assert_eq!(
            Error::command_failed("test", 1, "", "").category(),
            "command"
        );
        assert_eq!(Error::parse_error("test").category(), "parsing");
        assert_eq!(Error::invalid_config("test").category(), "config");
        assert_eq!(Error::container_not_found("test").category(), "container");
        assert_eq!(Error::image_not_found("test").category(), "image");
        assert_eq!(Error::custom("test").category(), "custom");
    }

    #[test]
    fn test_retryable_errors() {
        assert!(Error::command_failed("test", 1, "", "").is_retryable());
        assert!(Error::timeout(30).is_retryable());
        assert!(!Error::DockerNotFound.is_retryable());
        assert!(!Error::invalid_config("test").is_retryable());
    }

    #[test]
    fn test_error_constructors() {
        let cmd_err = Error::command_failed("docker run", 1, "output", "error");
        match cmd_err {
            Error::CommandFailed {
                command,
                exit_code,
                stdout,
                stderr,
            } => {
                assert_eq!(command, "docker run");
                assert_eq!(exit_code, 1);
                assert_eq!(stdout, "output");
                assert_eq!(stderr, "error");
            }
            _ => panic!("Wrong error type"),
        }

        let parse_err = Error::parse_error("invalid format");
        match parse_err {
            Error::ParseError { message } => {
                assert_eq!(message, "invalid format");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let docker_err: Error = io_err.into();

        match docker_err {
            Error::Io { message, .. } => {
                assert!(message.contains("file not found"));
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_from_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let docker_err: Error = json_err.into();

        match docker_err {
            Error::Json { message, .. } => {
                assert!(!message.is_empty());
            }
            _ => panic!("Wrong error type"),
        }
    }
}
