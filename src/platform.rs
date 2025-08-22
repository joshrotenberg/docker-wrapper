//! Platform detection and runtime abstraction for Docker environments.

use crate::error::{Error, Result};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Represents the detected container runtime
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Runtime {
    /// Docker runtime
    Docker,
    /// Podman runtime (Docker-compatible)
    Podman,
    /// Colima runtime (Docker-compatible on macOS)
    Colima,
    /// Rancher Desktop runtime
    RancherDesktop,
    /// `OrbStack` runtime (macOS)
    OrbStack,
    /// Docker Desktop
    DockerDesktop,
}

impl Runtime {
    /// Get the command name for this runtime
    #[must_use]
    pub fn command(&self) -> &str {
        match self {
            Runtime::Docker
            | Runtime::Colima
            | Runtime::RancherDesktop
            | Runtime::OrbStack
            | Runtime::DockerDesktop => "docker",
            Runtime::Podman => "podman",
        }
    }

    /// Check if this runtime supports Docker Compose
    #[must_use]
    pub fn supports_compose(&self) -> bool {
        matches!(
            self,
            Runtime::Docker
                | Runtime::DockerDesktop
                | Runtime::Colima
                | Runtime::RancherDesktop
                | Runtime::OrbStack
        )
    }

    /// Get compose command for this runtime
    #[must_use]
    pub fn compose_command(&self) -> Vec<String> {
        match self {
            Runtime::Podman => vec!["podman-compose".to_string()],
            Runtime::Docker
            | Runtime::DockerDesktop
            | Runtime::Colima
            | Runtime::RancherDesktop
            | Runtime::OrbStack => {
                vec!["docker".to_string(), "compose".to_string()]
            }
        }
    }
}

impl std::fmt::Display for Runtime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Runtime::Docker => write!(f, "Docker"),
            Runtime::Podman => write!(f, "Podman"),
            Runtime::Colima => write!(f, "Colima"),
            Runtime::RancherDesktop => write!(f, "Rancher Desktop"),
            Runtime::OrbStack => write!(f, "OrbStack"),
            Runtime::DockerDesktop => write!(f, "Docker Desktop"),
        }
    }
}

/// Operating system platform
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Platform {
    /// Linux
    Linux,
    /// macOS
    MacOS,
    /// Windows
    Windows,
    /// FreeBSD
    FreeBSD,
    /// Other/Unknown
    Other(String),
}

impl Platform {
    /// Detect the current platform
    #[must_use]
    pub fn detect() -> Self {
        match env::consts::OS {
            "linux" => Platform::Linux,
            "macos" | "darwin" => Platform::MacOS,
            "windows" => Platform::Windows,
            "freebsd" => Platform::FreeBSD,
            other => Platform::Other(other.to_string()),
        }
    }

    /// Check if running inside WSL
    #[must_use]
    pub fn is_wsl(&self) -> bool {
        if !matches!(self, Platform::Linux) {
            return false;
        }

        // Check for WSL-specific files/environment
        Path::new("/proc/sys/fs/binfmt_misc/WSLInterop").exists()
            || env::var("WSL_DISTRO_NAME").is_ok()
            || env::var("WSL_INTEROP").is_ok()
    }

    /// Get the default Docker socket path for this platform
    #[must_use]
    pub fn default_socket_path(&self) -> PathBuf {
        match self {
            Platform::MacOS => {
                // Check for various Docker socket locations on macOS
                let locations = [
                    "/var/run/docker.sock",
                    "/Users/$USER/.docker/run/docker.sock",
                    "/Users/$USER/.colima/docker.sock",
                    "/Users/$USER/.orbstack/run/docker.sock",
                ];

                for location in &locations {
                    let path = if location.contains("$USER") {
                        let user = env::var("USER").unwrap_or_else(|_| "unknown".to_string());
                        PathBuf::from(location.replace("$USER", &user))
                    } else {
                        PathBuf::from(location)
                    };

                    if path.exists() {
                        return path;
                    }
                }

                PathBuf::from("/var/run/docker.sock")
            }
            Platform::Windows => PathBuf::from("//./pipe/docker_engine"),
            Platform::Linux | Platform::FreeBSD | Platform::Other(_) => {
                PathBuf::from("/var/run/docker.sock")
            }
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Linux => write!(f, "Linux"),
            Platform::MacOS => write!(f, "macOS"),
            Platform::Windows => write!(f, "Windows"),
            Platform::FreeBSD => write!(f, "FreeBSD"),
            Platform::Other(s) => write!(f, "{s}"),
        }
    }
}

/// Platform and runtime detection
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    /// Operating system platform
    pub platform: Platform,
    /// Container runtime
    pub runtime: Runtime,
    /// Docker/runtime version
    pub version: String,
    /// Whether running in WSL
    pub is_wsl: bool,
    /// Docker socket path
    pub socket_path: PathBuf,
}

impl PlatformInfo {
    /// Detect platform and runtime information
    ///
    /// # Errors
    ///
    /// Returns an error if no container runtime is detected
    pub fn detect() -> Result<Self> {
        let platform = Platform::detect();
        let is_wsl = platform.is_wsl();
        let socket_path = Self::find_socket_path(&platform);

        // Detect runtime
        let runtime = Self::detect_runtime()?;
        let version = Self::get_runtime_version(&runtime)?;

        Ok(Self {
            platform,
            runtime,
            version,
            is_wsl,
            socket_path,
        })
    }

    /// Find the Docker socket path
    fn find_socket_path(platform: &Platform) -> PathBuf {
        // Check DOCKER_HOST environment variable first
        if let Ok(docker_host) = env::var("DOCKER_HOST") {
            if docker_host.starts_with("unix://") {
                return PathBuf::from(docker_host.trim_start_matches("unix://"));
            }
        }

        platform.default_socket_path()
    }

    /// Detect the container runtime
    fn detect_runtime() -> Result<Runtime> {
        // Check for specific runtime environment variables
        if env::var("ORBSTACK_HOME").is_ok() {
            return Ok(Runtime::OrbStack);
        }

        if env::var("COLIMA_HOME").is_ok() {
            return Ok(Runtime::Colima);
        }

        // Try to detect by checking version output
        if let Ok(output) = Command::new("docker").arg("version").output() {
            let version_str = String::from_utf8_lossy(&output.stdout);

            if version_str.contains("Docker Desktop") {
                return Ok(Runtime::DockerDesktop);
            }

            if version_str.contains("Rancher Desktop") {
                return Ok(Runtime::RancherDesktop);
            }

            if version_str.contains("podman") {
                return Ok(Runtime::Podman);
            }

            if version_str.contains("colima") {
                return Ok(Runtime::Colima);
            }

            if version_str.contains("OrbStack") {
                return Ok(Runtime::OrbStack);
            }

            // Generic Docker
            if version_str.contains("Docker") {
                return Ok(Runtime::Docker);
            }
        }

        // Try podman as fallback
        if Command::new("podman").arg("version").output().is_ok() {
            return Ok(Runtime::Podman);
        }

        Err(Error::DockerNotFound)
    }

    /// Get runtime version
    fn get_runtime_version(runtime: &Runtime) -> Result<String> {
        let output = Command::new(runtime.command())
            .arg("version")
            .arg("--format")
            .arg("{{.Server.Version}}")
            .output()
            .map_err(|e| {
                Error::command_failed(
                    format!("{} version", runtime.command()),
                    -1,
                    "",
                    e.to_string(),
                )
            })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            // Fallback to parsing regular version output
            let output = Command::new(runtime.command())
                .arg("version")
                .output()
                .map_err(|e| {
                    Error::command_failed(
                        format!("{} version", runtime.command()),
                        -1,
                        "",
                        e.to_string(),
                    )
                })?;

            let version_str = String::from_utf8_lossy(&output.stdout);
            Ok(Self::parse_version(&version_str))
        }
    }

    /// Parse version from version string
    fn parse_version(version_str: &str) -> String {
        // Look for version patterns
        for line in version_str.lines() {
            if line.contains("Version:") {
                if let Some(version) = line.split(':').nth(1) {
                    return version.trim().to_string();
                }
            }
        }

        "unknown".to_string()
    }

    /// Check if the runtime is available and working
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime is not found or not running
    pub fn check_runtime(&self) -> Result<()> {
        let output = Command::new(self.runtime.command())
            .arg("info")
            .output()
            .map_err(|_| Error::DockerNotFound)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("Cannot connect to the Docker daemon") {
                return Err(Error::DaemonNotRunning);
            }
            return Err(Error::command_failed(
                format!("{} info", self.runtime.command()),
                -1,
                "",
                stderr,
            ));
        }

        Ok(())
    }

    /// Get runtime-specific environment variables
    #[must_use]
    pub fn environment_vars(&self) -> Vec<(String, String)> {
        let mut vars = Vec::new();

        // Add socket path if needed
        if self.socket_path.exists() {
            vars.push((
                "DOCKER_HOST".to_string(),
                format!("unix://{}", self.socket_path.display()),
            ));
        }

        // Add runtime-specific vars
        match self.runtime {
            Runtime::Podman => {
                vars.push(("DOCKER_BUILDKIT".to_string(), "0".to_string()));
            }
            Runtime::DockerDesktop | Runtime::Docker => {
                vars.push(("DOCKER_BUILDKIT".to_string(), "1".to_string()));
            }
            _ => {}
        }

        vars
    }
}

impl std::fmt::Display for PlatformInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} on {} (version: {})",
            self.runtime, self.platform, self.version
        )?;
        if self.is_wsl {
            write!(f, " [WSL]")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect();
        // Should detect something
        assert!(matches!(
            platform,
            Platform::Linux
                | Platform::MacOS
                | Platform::Windows
                | Platform::FreeBSD
                | Platform::Other(_)
        ));
    }

    #[test]
    fn test_runtime_command() {
        assert_eq!(Runtime::Docker.command(), "docker");
        assert_eq!(Runtime::Podman.command(), "podman");
        assert_eq!(Runtime::Colima.command(), "docker");
    }

    #[test]
    fn test_runtime_compose_support() {
        assert!(Runtime::Docker.supports_compose());
        assert!(Runtime::DockerDesktop.supports_compose());
        assert!(Runtime::Colima.supports_compose());
        assert!(!Runtime::Podman.supports_compose());
    }

    #[test]
    fn test_platform_display() {
        assert_eq!(Platform::Linux.to_string(), "Linux");
        assert_eq!(Platform::MacOS.to_string(), "macOS");
        assert_eq!(Platform::Windows.to_string(), "Windows");
    }

    #[test]
    fn test_runtime_display() {
        assert_eq!(Runtime::Docker.to_string(), "Docker");
        assert_eq!(Runtime::Podman.to_string(), "Podman");
        assert_eq!(Runtime::OrbStack.to_string(), "OrbStack");
    }
}
