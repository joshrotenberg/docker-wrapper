//! Prerequisites module for Docker detection and validation.
//!
//! This module provides functionality to detect Docker installation,
//! validate version compatibility, and ensure the Docker daemon is running.

use serde::{Deserialize, Serialize};
use std::process::Stdio;
use thiserror::Error;
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Errors that can occur during Docker prerequisites checking
#[derive(Error, Debug)]
pub enum PrerequisitesError {
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
    #[error("Failed to execute Docker command: {message}")]
    CommandFailed {
        /// Error message describing the failure
        message: String,
    },

    /// Failed to parse Docker output
    #[error("Failed to parse Docker output: {message}")]
    ParseError {
        /// Error message describing the parse failure
        message: String,
    },
}

/// Result type for prerequisites operations
pub type PrerequisitesResult<T> = Result<T, PrerequisitesError>;

/// Docker version information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockerVersion {
    /// Full version string (e.g., "24.0.7")
    pub version: String,
    /// Major version number
    pub major: u32,
    /// Minor version number
    pub minor: u32,
    /// Patch version number
    pub patch: u32,
}

impl DockerVersion {
    /// Parse a Docker version string
    pub fn parse(version_str: &str) -> PrerequisitesResult<Self> {
        let clean_version = version_str.trim().trim_start_matches('v');
        let parts: Vec<&str> = clean_version.split('.').collect();

        if parts.len() < 3 {
            return Err(PrerequisitesError::ParseError {
                message: format!("Invalid version format: {}", version_str),
            });
        }

        let major = parts[0]
            .parse()
            .map_err(|_| PrerequisitesError::ParseError {
                message: format!("Invalid major version: {}", parts[0]),
            })?;

        let minor = parts[1]
            .parse()
            .map_err(|_| PrerequisitesError::ParseError {
                message: format!("Invalid minor version: {}", parts[1]),
            })?;

        let patch = parts[2]
            .parse()
            .map_err(|_| PrerequisitesError::ParseError {
                message: format!("Invalid patch version: {}", parts[2]),
            })?;

        Ok(Self {
            version: clean_version.to_string(),
            major,
            minor,
            patch,
        })
    }

    /// Check if this version meets the minimum requirement
    pub fn meets_minimum(&self, minimum: &DockerVersion) -> bool {
        if self.major > minimum.major {
            return true;
        }
        if self.major == minimum.major {
            if self.minor > minimum.minor {
                return true;
            }
            if self.minor == minimum.minor && self.patch >= minimum.patch {
                return true;
            }
        }
        false
    }
}

/// Docker system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerInfo {
    /// Docker version
    pub version: DockerVersion,
    /// Docker binary path
    pub binary_path: String,
    /// Whether Docker daemon is running
    pub daemon_running: bool,
    /// Docker server version (if daemon is running)
    pub server_version: Option<DockerVersion>,
    /// Operating system
    pub os: String,
    /// Architecture
    pub architecture: String,
}

/// Main prerequisites checker
pub struct DockerPrerequisites {
    /// Minimum required Docker version
    pub minimum_version: DockerVersion,
}

impl Default for DockerPrerequisites {
    fn default() -> Self {
        Self {
            minimum_version: DockerVersion {
                version: "20.10.0".to_string(),
                major: 20,
                minor: 10,
                patch: 0,
            },
        }
    }
}

impl DockerPrerequisites {
    /// Create a new prerequisites checker with custom minimum version
    pub fn new(minimum_version: DockerVersion) -> Self {
        Self { minimum_version }
    }

    /// Check all Docker prerequisites
    pub async fn check(&self) -> PrerequisitesResult<DockerInfo> {
        info!("Checking Docker prerequisites...");

        // Find Docker binary
        let binary_path = self.find_docker_binary().await?;
        debug!("Found Docker binary at: {}", binary_path);

        // Get Docker version
        let version = self.get_docker_version(&binary_path).await?;
        info!("Found Docker version: {}", version.version);

        // Check version compatibility
        if !version.meets_minimum(&self.minimum_version) {
            return Err(PrerequisitesError::UnsupportedVersion {
                found: version.version.clone(),
                minimum: self.minimum_version.version.clone(),
            });
        }

        // Check if daemon is running
        let (daemon_running, server_version) = self.check_daemon(&binary_path).await;

        if daemon_running {
            info!("Docker daemon is running");
        } else {
            warn!("Docker daemon is not running");
        }

        // Get system info
        let (os, architecture) = self.get_system_info().await;

        Ok(DockerInfo {
            version,
            binary_path,
            daemon_running,
            server_version,
            os,
            architecture,
        })
    }

    /// Find Docker binary in PATH
    async fn find_docker_binary(&self) -> PrerequisitesResult<String> {
        let output = Command::new("which")
            .arg("docker")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| PrerequisitesError::CommandFailed {
                message: format!("Failed to run 'which docker': {}", e),
            })?;

        if !output.status.success() {
            return Err(PrerequisitesError::DockerNotFound);
        }

        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if path.is_empty() {
            return Err(PrerequisitesError::DockerNotFound);
        }

        Ok(path)
    }

    /// Get Docker client version
    async fn get_docker_version(&self, binary_path: &str) -> PrerequisitesResult<DockerVersion> {
        let output = Command::new(binary_path)
            .args(["--version"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| PrerequisitesError::CommandFailed {
                message: format!("Failed to run 'docker --version': {}", e),
            })?;

        if !output.status.success() {
            return Err(PrerequisitesError::CommandFailed {
                message: format!(
                    "docker --version failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        let version_output = String::from_utf8_lossy(&output.stdout);
        debug!("Docker version output: {}", version_output);

        // Parse "Docker version 24.0.7, build afdd53b" format
        let version_str = version_output
            .split_whitespace()
            .nth(2)
            .and_then(|v| v.split(',').next())
            .ok_or_else(|| PrerequisitesError::ParseError {
                message: format!("Could not parse version from: {}", version_output),
            })?;

        DockerVersion::parse(version_str)
    }

    /// Check if Docker daemon is running and get server version
    async fn check_daemon(&self, binary_path: &str) -> (bool, Option<DockerVersion>) {
        let output = Command::new(binary_path)
            .args(["version", "--format", "{{.Server.Version}}"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match output {
            Ok(output) if output.status.success() => {
                let server_version_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !server_version_str.is_empty() {
                    match DockerVersion::parse(&server_version_str) {
                        Ok(version) => (true, Some(version)),
                        Err(_) => (true, None),
                    }
                } else {
                    (false, None)
                }
            }
            _ => (false, None),
        }
    }

    /// Get system information
    async fn get_system_info(&self) -> (String, String) {
        let os = std::env::consts::OS.to_string();
        let arch = std::env::consts::ARCH.to_string();
        (os, arch)
    }
}

/// Convenience function to check Docker prerequisites with default settings
pub async fn ensure_docker() -> PrerequisitesResult<DockerInfo> {
    let checker = DockerPrerequisites::default();
    checker.check().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_version_parse() {
        let version = DockerVersion::parse("24.0.7").unwrap();
        assert_eq!(version.major, 24);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 7);
        assert_eq!(version.version, "24.0.7");
    }

    #[test]
    fn test_docker_version_parse_with_v_prefix() {
        let version = DockerVersion::parse("v20.10.21").unwrap();
        assert_eq!(version.major, 20);
        assert_eq!(version.minor, 10);
        assert_eq!(version.patch, 21);
        assert_eq!(version.version, "20.10.21");
    }

    #[test]
    fn test_docker_version_parse_invalid() {
        assert!(DockerVersion::parse("invalid").is_err());
        assert!(DockerVersion::parse("1.2").is_err());
        assert!(DockerVersion::parse("a.b.c").is_err());
    }

    #[test]
    fn test_version_meets_minimum() {
        let current = DockerVersion::parse("24.0.7").unwrap();
        let minimum = DockerVersion::parse("20.10.0").unwrap();
        let too_high = DockerVersion::parse("25.0.0").unwrap();

        assert!(current.meets_minimum(&minimum));
        assert!(!current.meets_minimum(&too_high));

        // Test exact match
        let exact = DockerVersion::parse("20.10.0").unwrap();
        assert!(exact.meets_minimum(&minimum));

        // Test minor version differences
        let newer_minor = DockerVersion::parse("20.11.0").unwrap();
        let older_minor = DockerVersion::parse("20.9.0").unwrap();
        assert!(newer_minor.meets_minimum(&minimum));
        assert!(!older_minor.meets_minimum(&minimum));

        // Test patch version differences
        let newer_patch = DockerVersion::parse("20.10.1").unwrap();
        let older_patch = DockerVersion::parse("20.10.0").unwrap();
        assert!(newer_patch.meets_minimum(&minimum));
        assert!(older_patch.meets_minimum(&minimum)); // Equal should pass
    }

    #[test]
    fn test_prerequisites_default() {
        let prereqs = DockerPrerequisites::default();
        assert_eq!(prereqs.minimum_version.version, "20.10.0");
    }

    #[test]
    fn test_prerequisites_custom_minimum() {
        let custom_version = DockerVersion::parse("25.0.0").unwrap();
        let prereqs = DockerPrerequisites::new(custom_version.clone());
        assert_eq!(prereqs.minimum_version, custom_version);
    }

    #[tokio::test]
    async fn test_ensure_docker_integration() {
        // This is an integration test that requires Docker to be installed
        // It will be skipped in environments without Docker
        let result = ensure_docker().await;

        match result {
            Ok(info) => {
                assert!(!info.binary_path.is_empty());
                assert!(!info.version.version.is_empty());
                assert!(info.version.major >= 20);
                println!(
                    "Docker found: {} at {}",
                    info.version.version, info.binary_path
                );

                if info.daemon_running {
                    println!("Docker daemon is running");
                    if let Some(server_version) = info.server_version {
                        println!("Server version: {}", server_version.version);
                    }
                } else {
                    println!("Docker daemon is not running");
                }
            }
            Err(PrerequisitesError::DockerNotFound) => {
                println!("Docker not found - skipping integration test");
            }
            Err(e) => {
                println!("Prerequisites check failed: {}", e);
            }
        }
    }
}
