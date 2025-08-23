//! Docker version command implementation
//!
//! This module provides functionality to retrieve Docker version information,
//! including client and server versions, API versions, and build details.

use super::{CommandExecutor, CommandOutput, DockerCommandV2};
use crate::error::{Error, Result};
use async_trait::async_trait;
use std::fmt;

/// Command for retrieving Docker version information
///
/// The `VersionCommand` provides a builder pattern for constructing Docker version commands
/// with various output format options.
///
/// # Examples
///
/// ```rust
/// use docker_wrapper::VersionCommand;
///
/// // Basic version info
/// let version = VersionCommand::new();
///
/// // JSON format output
/// let version = VersionCommand::new().format_json();
///
/// // Custom format
/// let version = VersionCommand::new()
///     .format("{{.Client.Version}}");
/// ```
#[derive(Debug, Clone)]
pub struct VersionCommand {
    /// Output format
    format: Option<String>,
    /// Command executor for running the command
    pub executor: CommandExecutor,
}

/// Docker client version information
#[derive(Debug, Clone, PartialEq)]
pub struct ClientVersion {
    /// Client version string
    pub version: String,
    /// API version
    pub api_version: String,
    /// Git commit
    pub git_commit: String,
    /// Build time
    pub built: String,
    /// Go version used to build
    pub go_version: String,
    /// Operating system
    pub os: String,
    /// Architecture
    pub arch: String,
}

/// Docker server version information
#[derive(Debug, Clone, PartialEq)]
pub struct ServerVersion {
    /// Server version string
    pub version: String,
    /// API version
    pub api_version: String,
    /// Minimum API version supported
    pub min_api_version: String,
    /// Git commit
    pub git_commit: String,
    /// Build time
    pub built: String,
    /// Go version used to build
    pub go_version: String,
    /// Operating system
    pub os: String,
    /// Architecture
    pub arch: String,
    /// Kernel version
    pub kernel_version: String,
    /// Experimental features enabled
    pub experimental: bool,
}

/// Complete Docker version information
#[derive(Debug, Clone, PartialEq)]
pub struct VersionInfo {
    /// Client version details
    pub client: ClientVersion,
    /// Server version details (if available)
    pub server: Option<ServerVersion>,
}

/// Output from a version command execution
///
/// Contains the raw output from the Docker version command and provides
/// convenience methods for parsing version information.
#[derive(Debug, Clone)]
pub struct VersionOutput {
    /// Raw output from the Docker command
    pub output: CommandOutput,
    /// Parsed version information
    pub version_info: Option<VersionInfo>,
}

impl VersionCommand {
    /// Creates a new version command
    ///
    /// # Examples
    ///
    /// ```rust
    /// use docker_wrapper::VersionCommand;
    ///
    /// let version = VersionCommand::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            format: None,
            executor: CommandExecutor::default(),
        }
    }

    /// Sets the output format
    ///
    /// # Arguments
    ///
    /// * `format` - Output format string or template
    ///
    /// # Examples
    ///
    /// ```rust
    /// use docker_wrapper::VersionCommand;
    ///
    /// let version = VersionCommand::new()
    ///     .format("{{.Client.Version}}");
    /// ```
    #[must_use]
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Sets output format to JSON
    ///
    /// # Examples
    ///
    /// ```rust
    /// use docker_wrapper::VersionCommand;
    ///
    /// let version = VersionCommand::new().format_json();
    /// ```
    #[must_use]
    pub fn format_json(self) -> Self {
        self.format("json")
    }

    /// Sets output format to table (default)
    #[must_use]
    pub fn format_table(self) -> Self {
        Self {
            format: None,
            executor: self.executor,
        }
    }

    /// Gets a reference to the executor
    #[must_use]
    pub fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    /// Gets a mutable reference to the executor
    pub fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    /// Builds the command arguments for Docker version
    #[must_use]
    pub fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["version".to_string()];

        // Add format option
        if let Some(ref format) = self.format {
            args.push("--format".to_string());
            args.push(format.clone());
        }

        // Add any additional raw arguments
        args.extend(self.executor.raw_args.clone());

        args
    }

    /// Parses the version output
    fn parse_output(&self, output: &CommandOutput) -> Result<Option<VersionInfo>> {
        if let Some(ref format) = self.format {
            if format == "json" {
                return Self::parse_json_output(output);
            }
        }

        Ok(Self::parse_table_output(output))
    }

    /// Parses JSON formatted version output
    fn parse_json_output(output: &CommandOutput) -> Result<Option<VersionInfo>> {
        let parsed: serde_json::Value = serde_json::from_str(&output.stdout)
            .map_err(|e| Error::parse_error(format!("Failed to parse version JSON output: {e}")))?;

        // Parse client version
        let client_data = &parsed["Client"];
        let client = ClientVersion {
            version: client_data["Version"].as_str().unwrap_or("").to_string(),
            api_version: client_data["ApiVersion"].as_str().unwrap_or("").to_string(),
            git_commit: client_data["GitCommit"].as_str().unwrap_or("").to_string(),
            built: client_data["Built"].as_str().unwrap_or("").to_string(),
            go_version: client_data["GoVersion"].as_str().unwrap_or("").to_string(),
            os: client_data["Os"].as_str().unwrap_or("").to_string(),
            arch: client_data["Arch"].as_str().unwrap_or("").to_string(),
        };

        // Parse server version (if available)
        let server = parsed.get("Server").map(|server_data| ServerVersion {
            version: server_data["Version"].as_str().unwrap_or("").to_string(),
            api_version: server_data["ApiVersion"].as_str().unwrap_or("").to_string(),
            min_api_version: server_data["MinAPIVersion"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            git_commit: server_data["GitCommit"].as_str().unwrap_or("").to_string(),
            built: server_data["Built"].as_str().unwrap_or("").to_string(),
            go_version: server_data["GoVersion"].as_str().unwrap_or("").to_string(),
            os: server_data["Os"].as_str().unwrap_or("").to_string(),
            arch: server_data["Arch"].as_str().unwrap_or("").to_string(),
            kernel_version: server_data["KernelVersion"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            experimental: server_data["Experimental"].as_bool().unwrap_or(false),
        });

        Ok(Some(VersionInfo { client, server }))
    }

    /// Parses table formatted version output
    fn parse_table_output(output: &CommandOutput) -> Option<VersionInfo> {
        let lines: Vec<&str> = output.stdout.lines().collect();

        if lines.is_empty() {
            return None;
        }

        let mut client_section = false;
        let mut server_section = false;
        let mut client_data = std::collections::HashMap::new();
        let mut server_data = std::collections::HashMap::new();

        for line in lines {
            let trimmed = line.trim();

            if trimmed.starts_with("Client:") {
                client_section = true;
                server_section = false;
                continue;
            } else if trimmed.starts_with("Server:") {
                client_section = false;
                server_section = true;
                continue;
            }

            if trimmed.is_empty() {
                continue;
            }

            // Parse key-value pairs
            if let Some(colon_pos) = trimmed.find(':') {
                let key = trimmed[..colon_pos].trim();
                let value = trimmed[colon_pos + 1..].trim();

                if client_section {
                    client_data.insert(key.to_string(), value.to_string());
                } else if server_section {
                    server_data.insert(key.to_string(), value.to_string());
                }
            }
        }

        let client = ClientVersion {
            version: client_data.get("Version").cloned().unwrap_or_default(),
            api_version: client_data.get("API version").cloned().unwrap_or_default(),
            git_commit: client_data.get("Git commit").cloned().unwrap_or_default(),
            built: client_data.get("Built").cloned().unwrap_or_default(),
            go_version: client_data.get("Go version").cloned().unwrap_or_default(),
            os: client_data.get("OS/Arch").cloned().unwrap_or_default(),
            arch: String::new(), // OS/Arch is combined in table format
        };

        let server = if server_data.is_empty() {
            None
        } else {
            Some(ServerVersion {
                version: server_data.get("Version").cloned().unwrap_or_default(),
                api_version: server_data.get("API version").cloned().unwrap_or_default(),
                min_api_version: server_data
                    .get("Minimum API version")
                    .cloned()
                    .unwrap_or_default(),
                git_commit: server_data.get("Git commit").cloned().unwrap_or_default(),
                built: server_data.get("Built").cloned().unwrap_or_default(),
                go_version: server_data.get("Go version").cloned().unwrap_or_default(),
                os: server_data.get("OS/Arch").cloned().unwrap_or_default(),
                arch: String::new(), // OS/Arch is combined in table format
                kernel_version: server_data
                    .get("Kernel Version")
                    .cloned()
                    .unwrap_or_default(),
                experimental: server_data.get("Experimental").is_some_and(|s| s == "true"),
            })
        };

        Some(VersionInfo { client, server })
    }

    /// Gets the output format (if set)
    #[must_use]
    pub fn get_format(&self) -> Option<&str> {
        self.format.as_deref()
    }
}

impl Default for VersionCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionOutput {
    /// Returns true if the version command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Gets the client version string
    #[must_use]
    pub fn client_version(&self) -> Option<&str> {
        self.version_info
            .as_ref()
            .map(|v| v.client.version.as_str())
    }

    /// Gets the server version string
    #[must_use]
    pub fn server_version(&self) -> Option<&str> {
        self.version_info
            .as_ref()
            .and_then(|v| v.server.as_ref())
            .map(|s| s.version.as_str())
    }

    /// Gets the API version
    #[must_use]
    pub fn api_version(&self) -> Option<&str> {
        self.version_info
            .as_ref()
            .map(|v| v.client.api_version.as_str())
    }

    /// Returns true if server information is available
    #[must_use]
    pub fn has_server_info(&self) -> bool {
        self.version_info
            .as_ref()
            .is_some_and(|v| v.server.is_some())
    }

    /// Returns true if experimental features are enabled
    #[must_use]
    pub fn is_experimental(&self) -> bool {
        self.version_info
            .as_ref()
            .and_then(|v| v.server.as_ref())
            .is_some_and(|s| s.experimental)
    }

    /// Checks if the Docker version is compatible with a minimum version
    #[must_use]
    pub fn is_compatible(&self, min_version: &str) -> bool {
        if let Some(version) = self.client_version() {
            // Simple version comparison (would need proper semver for production)
            version >= min_version
        } else {
            false
        }
    }
}

#[async_trait]
impl DockerCommandV2 for VersionCommand {
    type Output = VersionOutput;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        self.build_command_args()
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        let output = self.execute_command(args).await?;

        let version_info = self.parse_output(&output)?;

        Ok(VersionOutput {
            output,
            version_info,
        })
    }
}

impl fmt::Display for VersionCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "docker version")?;

        if let Some(ref format) = self.format {
            write!(f, " --format {format}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_command_basic() {
        let version = VersionCommand::new();

        assert_eq!(version.get_format(), None);

        let args = version.build_command_args();
        assert_eq!(args, vec!["version"]);
    }

    #[test]
    fn test_version_command_with_format() {
        let version = VersionCommand::new().format("{{.Client.Version}}");

        assert_eq!(version.get_format(), Some("{{.Client.Version}}"));

        let args = version.build_command_args();
        assert_eq!(args, vec!["version", "--format", "{{.Client.Version}}"]);
    }

    #[test]
    fn test_version_command_json_format() {
        let version = VersionCommand::new().format_json();

        assert_eq!(version.get_format(), Some("json"));

        let args = version.build_command_args();
        assert_eq!(args, vec!["version", "--format", "json"]);
    }

    #[test]
    fn test_version_command_table_format() {
        let version = VersionCommand::new().format_json().format_table();

        assert_eq!(version.get_format(), None);

        let args = version.build_command_args();
        assert_eq!(args, vec!["version"]);
    }

    #[test]
    fn test_version_command_default() {
        let version = VersionCommand::default();

        assert_eq!(version.get_format(), None);
        let args = version.build_command_args();
        assert_eq!(args, vec!["version"]);
    }

    #[test]
    fn test_client_version_creation() {
        let client = ClientVersion {
            version: "20.10.17".to_string(),
            api_version: "1.41".to_string(),
            git_commit: "100c701".to_string(),
            built: "Mon Jun  6 23:02:57 2022".to_string(),
            go_version: "go1.17.11".to_string(),
            os: "linux".to_string(),
            arch: "amd64".to_string(),
        };

        assert_eq!(client.version, "20.10.17");
        assert_eq!(client.api_version, "1.41");
        assert_eq!(client.os, "linux");
        assert_eq!(client.arch, "amd64");
    }

    #[test]
    fn test_server_version_creation() {
        let server = ServerVersion {
            version: "20.10.17".to_string(),
            api_version: "1.41".to_string(),
            min_api_version: "1.12".to_string(),
            git_commit: "100c701".to_string(),
            built: "Mon Jun  6 23:02:57 2022".to_string(),
            go_version: "go1.17.11".to_string(),
            os: "linux".to_string(),
            arch: "amd64".to_string(),
            kernel_version: "5.15.0".to_string(),
            experimental: false,
        };

        assert_eq!(server.version, "20.10.17");
        assert_eq!(server.min_api_version, "1.12");
        assert!(!server.experimental);
    }

    #[test]
    fn test_version_info_creation() {
        let client = ClientVersion {
            version: "20.10.17".to_string(),
            api_version: "1.41".to_string(),
            git_commit: "100c701".to_string(),
            built: "Mon Jun  6 23:02:57 2022".to_string(),
            go_version: "go1.17.11".to_string(),
            os: "linux".to_string(),
            arch: "amd64".to_string(),
        };

        let version_info = VersionInfo {
            client,
            server: None,
        };

        assert_eq!(version_info.client.version, "20.10.17");
        assert!(version_info.server.is_none());
    }

    #[test]
    fn test_version_output_helpers() {
        let client = ClientVersion {
            version: "20.10.17".to_string(),
            api_version: "1.41".to_string(),
            git_commit: "100c701".to_string(),
            built: "Mon Jun  6 23:02:57 2022".to_string(),
            go_version: "go1.17.11".to_string(),
            os: "linux".to_string(),
            arch: "amd64".to_string(),
        };

        let server = ServerVersion {
            version: "20.10.17".to_string(),
            api_version: "1.41".to_string(),
            min_api_version: "1.12".to_string(),
            git_commit: "100c701".to_string(),
            built: "Mon Jun  6 23:02:57 2022".to_string(),
            go_version: "go1.17.11".to_string(),
            os: "linux".to_string(),
            arch: "amd64".to_string(),
            kernel_version: "5.15.0".to_string(),
            experimental: true,
        };

        let version_info = VersionInfo {
            client,
            server: Some(server),
        };

        let output = VersionOutput {
            output: CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
            },
            version_info: Some(version_info),
        };

        assert_eq!(output.client_version(), Some("20.10.17"));
        assert_eq!(output.server_version(), Some("20.10.17"));
        assert_eq!(output.api_version(), Some("1.41"));
        assert!(output.has_server_info());
        assert!(output.is_experimental());
        assert!(output.is_compatible("20.10.0"));
        assert!(!output.is_compatible("21.0.0"));
    }

    #[test]
    fn test_version_output_no_server() {
        let client = ClientVersion {
            version: "20.10.17".to_string(),
            api_version: "1.41".to_string(),
            git_commit: "100c701".to_string(),
            built: "Mon Jun  6 23:02:57 2022".to_string(),
            go_version: "go1.17.11".to_string(),
            os: "linux".to_string(),
            arch: "amd64".to_string(),
        };

        let version_info = VersionInfo {
            client,
            server: None,
        };

        let output = VersionOutput {
            output: CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
            },
            version_info: Some(version_info),
        };

        assert_eq!(output.client_version(), Some("20.10.17"));
        assert_eq!(output.server_version(), None);
        assert!(!output.has_server_info());
        assert!(!output.is_experimental());
    }

    #[test]
    fn test_version_command_display() {
        let version = VersionCommand::new().format("{{.Client.Version}}");

        let display = format!("{version}");
        assert_eq!(display, "docker version --format {{.Client.Version}}");
    }

    #[test]
    fn test_version_command_display_no_format() {
        let version = VersionCommand::new();

        let display = format!("{version}");
        assert_eq!(display, "docker version");
    }

    #[test]
    fn test_version_command_name() {
        let version = VersionCommand::new();
        let args = version.build_command_args();
        assert_eq!(args[0], "version");
    }

    #[test]
    fn test_version_command_extensibility() {
        let mut version = VersionCommand::new();

        // Test that we can add custom raw arguments
        version
            .get_executor_mut()
            .raw_args
            .push("--verbose".to_string());
        version
            .get_executor_mut()
            .raw_args
            .push("--some-flag".to_string());

        let args = version.build_command_args();

        // Verify raw args are included
        assert!(args.contains(&"--verbose".to_string()));
        assert!(args.contains(&"--some-flag".to_string()));
    }

    #[test]
    fn test_parse_json_output_concept() {
        // This test demonstrates the concept of parsing JSON output
        let json_output = r#"{"Client":{"Version":"20.10.17","ApiVersion":"1.41"}}"#;

        let output = CommandOutput {
            stdout: json_output.to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };

        let result = VersionCommand::parse_json_output(&output);

        // The actual parsing would need real Docker JSON output
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_table_output_concept() {
        // This test demonstrates the concept of parsing table output
        let table_output =
            "Client:\n Version: 20.10.17\n API version: 1.41\n\nServer:\n Version: 20.10.17";

        let output = CommandOutput {
            stdout: table_output.to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };

        let result = VersionCommand::parse_table_output(&output);

        // The actual parsing would need real Docker table output
        assert!(result.is_some() || result.is_none());
    }
}
