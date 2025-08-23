//! Docker info command implementation
//!
//! This module provides functionality to retrieve Docker system information,
//! including daemon configuration, storage details, and runtime information.

use super::{CommandExecutor, CommandOutput, DockerCommandV2};
use crate::error::{Error, Result};
use async_trait::async_trait;
use std::fmt;

/// Command for retrieving Docker system information
///
/// The `InfoCommand` provides a builder pattern for constructing Docker info commands
/// with various output format options.
///
/// # Examples
///
/// ```rust
/// use docker_wrapper::InfoCommand;
///
/// // Basic system info
/// let info = InfoCommand::new();
///
/// // JSON format output
/// let info = InfoCommand::new().format_json();
///
/// // Custom format
/// let info = InfoCommand::new()
///     .format("{{.ServerVersion}}");
/// ```
#[derive(Debug, Clone)]
pub struct InfoCommand {
    /// Output format
    format: Option<String>,
    /// Command executor for running the command
    pub executor: CommandExecutor,
}

/// Docker system information
#[derive(Debug, Clone, PartialEq)]
pub struct SystemInfo {
    /// Docker server version
    pub server_version: String,
    /// Storage driver in use
    pub storage_driver: String,
    /// Logging driver
    pub logging_driver: String,
    /// Cgroup driver
    pub cgroup_driver: String,
    /// Cgroup version
    pub cgroup_version: String,
    /// Number of containers
    pub containers: u32,
    /// Number of running containers
    pub containers_running: u32,
    /// Number of paused containers
    pub containers_paused: u32,
    /// Number of stopped containers
    pub containers_stopped: u32,
    /// Number of images
    pub images: u32,
    /// Docker root directory
    pub docker_root_dir: String,
    /// Debug mode enabled
    pub debug: bool,
    /// Experimental features enabled
    pub experimental: bool,
    /// Total memory
    pub mem_total: u64,
    /// Number of CPUs
    pub ncpu: u32,
    /// Operating system
    pub operating_system: String,
    /// OS type
    pub os_type: String,
    /// Architecture
    pub architecture: String,
    /// Kernel version
    pub kernel_version: String,
    /// Name (hostname)
    pub name: String,
    /// Docker daemon ID
    pub id: String,
}

/// Docker registry configuration
#[derive(Debug, Clone, PartialEq)]
pub struct RegistryConfig {
    /// Insecure registries
    pub insecure_registries: Vec<String>,
    /// Index configs
    pub index_configs: Vec<String>,
    /// Mirrors
    pub mirrors: Vec<String>,
}

/// Docker runtime information
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeInfo {
    /// Default runtime
    pub default_runtime: String,
    /// Available runtimes
    pub runtimes: Vec<String>,
}

/// Complete Docker system information
#[derive(Debug, Clone, PartialEq)]
pub struct DockerInfo {
    /// Basic system information
    pub system: SystemInfo,
    /// Registry configuration
    pub registry: Option<RegistryConfig>,
    /// Runtime information
    pub runtime: Option<RuntimeInfo>,
    /// Warnings from the Docker daemon
    pub warnings: Vec<String>,
}

/// Output from an info command execution
///
/// Contains the raw output from the Docker info command and provides
/// convenience methods for parsing system information.
#[derive(Debug, Clone)]
pub struct InfoOutput {
    /// Raw output from the Docker command
    pub output: CommandOutput,
    /// Parsed Docker information
    pub docker_info: Option<DockerInfo>,
}

impl InfoCommand {
    /// Creates a new info command
    ///
    /// # Examples
    ///
    /// ```rust
    /// use docker_wrapper::InfoCommand;
    ///
    /// let info = InfoCommand::new();
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
    /// use docker_wrapper::InfoCommand;
    ///
    /// let info = InfoCommand::new()
    ///     .format("{{.ServerVersion}}");
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
    /// use docker_wrapper::InfoCommand;
    ///
    /// let info = InfoCommand::new().format_json();
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

    /// Gets the command executor
    #[must_use]
    pub fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    /// Gets the command executor mutably
    pub fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    /// Builds the command arguments for Docker info
    #[must_use]
    pub fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["info".to_string()];

        // Add format option
        if let Some(ref format) = self.format {
            args.push("--format".to_string());
            args.push(format.clone());
        }

        // Add any additional raw arguments
        args.extend(self.executor.raw_args.clone());

        args
    }

    /// Parses the info output
    fn parse_output(&self, output: &CommandOutput) -> Result<Option<DockerInfo>> {
        if let Some(ref format) = self.format {
            if format == "json" {
                return Self::parse_json_output(output);
            }
        }

        Ok(Self::parse_table_output(output))
    }

    /// Parses JSON formatted info output
    fn parse_json_output(output: &CommandOutput) -> Result<Option<DockerInfo>> {
        let parsed: serde_json::Value = serde_json::from_str(&output.stdout)
            .map_err(|e| Error::parse_error(format!("Failed to parse info JSON output: {e}")))?;

        let system = SystemInfo {
            server_version: parsed["ServerVersion"].as_str().unwrap_or("").to_string(),
            storage_driver: parsed["Driver"].as_str().unwrap_or("").to_string(),
            logging_driver: parsed["LoggingDriver"].as_str().unwrap_or("").to_string(),
            cgroup_driver: parsed["CgroupDriver"].as_str().unwrap_or("").to_string(),
            cgroup_version: parsed["CgroupVersion"].as_str().unwrap_or("").to_string(),
            containers: u32::try_from(parsed["Containers"].as_u64().unwrap_or(0)).unwrap_or(0),
            containers_running: u32::try_from(parsed["ContainersRunning"].as_u64().unwrap_or(0))
                .unwrap_or(0),
            containers_paused: u32::try_from(parsed["ContainersPaused"].as_u64().unwrap_or(0))
                .unwrap_or(0),
            containers_stopped: u32::try_from(parsed["ContainersStopped"].as_u64().unwrap_or(0))
                .unwrap_or(0),
            images: u32::try_from(parsed["Images"].as_u64().unwrap_or(0)).unwrap_or(0),
            docker_root_dir: parsed["DockerRootDir"].as_str().unwrap_or("").to_string(),
            debug: parsed["Debug"].as_bool().unwrap_or(false),
            experimental: parsed["ExperimentalBuild"].as_bool().unwrap_or(false),
            mem_total: parsed["MemTotal"].as_u64().unwrap_or(0),
            ncpu: u32::try_from(parsed["NCPU"].as_u64().unwrap_or(0)).unwrap_or(0),
            operating_system: parsed["OperatingSystem"].as_str().unwrap_or("").to_string(),
            os_type: parsed["OSType"].as_str().unwrap_or("").to_string(),
            architecture: parsed["Architecture"].as_str().unwrap_or("").to_string(),
            kernel_version: parsed["KernelVersion"].as_str().unwrap_or("").to_string(),
            name: parsed["Name"].as_str().unwrap_or("").to_string(),
            id: parsed["ID"].as_str().unwrap_or("").to_string(),
        };

        // Parse registry config
        let registry = parsed.get("RegistryConfig").map(|registry_data| {
            let insecure_registries = registry_data["InsecureRegistryCIDRs"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(String::from)
                        .collect()
                })
                .unwrap_or_default();

            let index_configs = registry_data["IndexConfigs"]
                .as_object()
                .map(|obj| obj.keys().map(String::from).collect())
                .unwrap_or_default();

            let mirrors = registry_data["Mirrors"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(String::from)
                        .collect()
                })
                .unwrap_or_default();

            RegistryConfig {
                insecure_registries,
                index_configs,
                mirrors,
            }
        });

        // Parse runtime info
        let runtime = parsed.get("Runtimes").map(|runtimes_data| {
            let default_runtime = parsed["DefaultRuntime"].as_str().unwrap_or("").to_string();

            let runtimes = runtimes_data
                .as_object()
                .map(|obj| obj.keys().map(String::from).collect())
                .unwrap_or_default();

            RuntimeInfo {
                default_runtime,
                runtimes,
            }
        });

        // Parse warnings
        let warnings = parsed["Warnings"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        Ok(Some(DockerInfo {
            system,
            registry,
            runtime,
            warnings,
        }))
    }

    /// Parses table formatted info output
    fn parse_table_output(output: &CommandOutput) -> Option<DockerInfo> {
        let lines: Vec<&str> = output.stdout.lines().collect();

        if lines.is_empty() {
            return None;
        }

        let mut data = std::collections::HashMap::new();
        let mut warnings = Vec::new();

        for line in lines {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            // Check for warnings
            if trimmed.starts_with("WARNING:") {
                warnings.push(trimmed.to_string());
                continue;
            }

            // Parse key-value pairs
            if let Some(colon_pos) = trimmed.find(':') {
                let key = trimmed[..colon_pos].trim();
                let value = trimmed[colon_pos + 1..].trim();
                data.insert(key.to_string(), value.to_string());
            }
        }

        let system = SystemInfo {
            server_version: data.get("Server Version").cloned().unwrap_or_default(),
            storage_driver: data.get("Storage Driver").cloned().unwrap_or_default(),
            logging_driver: data.get("Logging Driver").cloned().unwrap_or_default(),
            cgroup_driver: data.get("Cgroup Driver").cloned().unwrap_or_default(),
            cgroup_version: data.get("Cgroup Version").cloned().unwrap_or_default(),
            containers: data
                .get("Containers")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            containers_running: data
                .get("Running")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            containers_paused: data.get("Paused").and_then(|s| s.parse().ok()).unwrap_or(0),
            containers_stopped: data
                .get("Stopped")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            images: data.get("Images").and_then(|s| s.parse().ok()).unwrap_or(0),
            docker_root_dir: data.get("Docker Root Dir").cloned().unwrap_or_default(),
            debug: data.get("Debug Mode").is_some_and(|s| s == "true"),
            experimental: data.get("Experimental").is_some_and(|s| s == "true"),
            mem_total: data
                .get("Total Memory")
                .and_then(|s| s.split_whitespace().next())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            ncpu: data.get("CPUs").and_then(|s| s.parse().ok()).unwrap_or(0),
            operating_system: data.get("Operating System").cloned().unwrap_or_default(),
            os_type: data.get("OSType").cloned().unwrap_or_default(),
            architecture: data.get("Architecture").cloned().unwrap_or_default(),
            kernel_version: data.get("Kernel Version").cloned().unwrap_or_default(),
            name: data.get("Name").cloned().unwrap_or_default(),
            id: data.get("ID").cloned().unwrap_or_default(),
        };

        Some(DockerInfo {
            system,
            registry: None, // Not easily parseable from table format
            runtime: None,  // Not easily parseable from table format
            warnings,
        })
    }

    /// Gets the output format (if set)
    #[must_use]
    pub fn get_format(&self) -> Option<&str> {
        self.format.as_deref()
    }
}

impl Default for InfoCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl InfoOutput {
    /// Returns true if the info command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Gets the Docker server version
    #[must_use]
    pub fn server_version(&self) -> Option<&str> {
        self.docker_info
            .as_ref()
            .map(|info| info.system.server_version.as_str())
    }

    /// Gets the storage driver
    #[must_use]
    pub fn storage_driver(&self) -> Option<&str> {
        self.docker_info
            .as_ref()
            .map(|info| info.system.storage_driver.as_str())
    }

    /// Gets the total number of containers
    #[must_use]
    pub fn container_count(&self) -> u32 {
        self.docker_info
            .as_ref()
            .map_or(0, |info| info.system.containers)
    }

    /// Gets the number of running containers
    #[must_use]
    pub fn running_containers(&self) -> u32 {
        self.docker_info
            .as_ref()
            .map_or(0, |info| info.system.containers_running)
    }

    /// Gets the number of images
    #[must_use]
    pub fn image_count(&self) -> u32 {
        self.docker_info
            .as_ref()
            .map_or(0, |info| info.system.images)
    }

    /// Returns true if debug mode is enabled
    #[must_use]
    pub fn is_debug(&self) -> bool {
        self.docker_info
            .as_ref()
            .is_some_and(|info| info.system.debug)
    }

    /// Returns true if experimental features are enabled
    #[must_use]
    pub fn is_experimental(&self) -> bool {
        self.docker_info
            .as_ref()
            .is_some_and(|info| info.system.experimental)
    }

    /// Gets the operating system
    #[must_use]
    pub fn operating_system(&self) -> Option<&str> {
        self.docker_info
            .as_ref()
            .map(|info| info.system.operating_system.as_str())
    }

    /// Gets the architecture
    #[must_use]
    pub fn architecture(&self) -> Option<&str> {
        self.docker_info
            .as_ref()
            .map(|info| info.system.architecture.as_str())
    }

    /// Gets any warnings from the Docker daemon
    #[must_use]
    pub fn warnings(&self) -> Vec<&str> {
        self.docker_info
            .as_ref()
            .map(|info| info.warnings.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }

    /// Returns true if there are any warnings
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        self.docker_info
            .as_ref()
            .is_some_and(|info| !info.warnings.is_empty())
    }

    /// Gets system resource information (containers and images)
    #[must_use]
    pub fn resource_summary(&self) -> (u32, u32, u32) {
        if let Some(info) = &self.docker_info {
            (
                info.system.containers,
                info.system.containers_running,
                info.system.images,
            )
        } else {
            (0, 0, 0)
        }
    }
}

#[async_trait]
impl DockerCommandV2 for InfoCommand {
    type Output = InfoOutput;

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

        let docker_info = self.parse_output(&output)?;

        Ok(InfoOutput {
            output,
            docker_info,
        })
    }
}

impl fmt::Display for InfoCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "docker info")?;

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
    fn test_info_command_basic() {
        let info = InfoCommand::new();

        assert_eq!(info.get_format(), None);

        let args = info.build_command_args();
        assert_eq!(args, vec!["info"]);
    }

    #[test]
    fn test_info_command_with_format() {
        let info = InfoCommand::new().format("{{.ServerVersion}}");

        assert_eq!(info.get_format(), Some("{{.ServerVersion}}"));

        let args = info.build_command_args();
        assert_eq!(args, vec!["info", "--format", "{{.ServerVersion}}"]);
    }

    #[test]
    fn test_info_command_json_format() {
        let info = InfoCommand::new().format_json();

        assert_eq!(info.get_format(), Some("json"));

        let args = info.build_command_args();
        assert_eq!(args, vec!["info", "--format", "json"]);
    }

    #[test]
    fn test_info_command_table_format() {
        let info = InfoCommand::new().format_json().format_table();

        assert_eq!(info.get_format(), None);

        let args = info.build_command_args();
        assert_eq!(args, vec!["info"]);
    }

    #[test]
    fn test_info_command_default() {
        let info = InfoCommand::default();

        assert_eq!(info.get_format(), None);
        let args = info.build_command_args();
        assert_eq!(args, vec!["info"]);
    }

    #[test]
    fn test_system_info_creation() {
        let system = SystemInfo {
            server_version: "20.10.17".to_string(),
            storage_driver: "overlay2".to_string(),
            logging_driver: "json-file".to_string(),
            cgroup_driver: "systemd".to_string(),
            cgroup_version: "2".to_string(),
            containers: 10,
            containers_running: 3,
            containers_paused: 0,
            containers_stopped: 7,
            images: 25,
            docker_root_dir: "/var/lib/docker".to_string(),
            debug: false,
            experimental: false,
            mem_total: 8_589_934_592,
            ncpu: 8,
            operating_system: "Ubuntu 20.04.4 LTS".to_string(),
            os_type: "linux".to_string(),
            architecture: "x86_64".to_string(),
            kernel_version: "5.15.0-56-generic".to_string(),
            name: "docker-host".to_string(),
            id: "ABCD:1234:5678:90EF".to_string(),
        };

        assert_eq!(system.server_version, "20.10.17");
        assert_eq!(system.storage_driver, "overlay2");
        assert_eq!(system.containers, 10);
        assert_eq!(system.containers_running, 3);
        assert_eq!(system.images, 25);
        assert!(!system.debug);
        assert!(!system.experimental);
    }

    #[test]
    fn test_registry_config_creation() {
        let registry = RegistryConfig {
            insecure_registries: vec!["localhost:5000".to_string()],
            index_configs: vec!["https://index.docker.io/v1/".to_string()],
            mirrors: vec!["https://mirror.gcr.io".to_string()],
        };

        assert_eq!(registry.insecure_registries.len(), 1);
        assert_eq!(registry.index_configs.len(), 1);
        assert_eq!(registry.mirrors.len(), 1);
    }

    #[test]
    fn test_runtime_info_creation() {
        let runtime = RuntimeInfo {
            default_runtime: "runc".to_string(),
            runtimes: vec!["runc".to_string(), "nvidia".to_string()],
        };

        assert_eq!(runtime.default_runtime, "runc");
        assert_eq!(runtime.runtimes.len(), 2);
    }

    #[test]
    fn test_docker_info_creation() {
        let system = SystemInfo {
            server_version: "20.10.17".to_string(),
            storage_driver: "overlay2".to_string(),
            logging_driver: "json-file".to_string(),
            cgroup_driver: "systemd".to_string(),
            cgroup_version: "2".to_string(),
            containers: 5,
            containers_running: 2,
            containers_paused: 0,
            containers_stopped: 3,
            images: 10,
            docker_root_dir: "/var/lib/docker".to_string(),
            debug: true,
            experimental: true,
            mem_total: 8_589_934_592,
            ncpu: 4,
            operating_system: "Ubuntu 20.04".to_string(),
            os_type: "linux".to_string(),
            architecture: "x86_64".to_string(),
            kernel_version: "5.15.0".to_string(),
            name: "test-host".to_string(),
            id: "TEST:1234".to_string(),
        };

        let docker_info = DockerInfo {
            system,
            registry: None,
            runtime: None,
            warnings: vec!["Test warning".to_string()],
        };

        assert_eq!(docker_info.system.server_version, "20.10.17");
        assert_eq!(docker_info.warnings.len(), 1);
        assert!(docker_info.registry.is_none());
        assert!(docker_info.runtime.is_none());
    }

    #[test]
    fn test_info_output_helpers() {
        let system = SystemInfo {
            server_version: "20.10.17".to_string(),
            storage_driver: "overlay2".to_string(),
            logging_driver: "json-file".to_string(),
            cgroup_driver: "systemd".to_string(),
            cgroup_version: "2".to_string(),
            containers: 15,
            containers_running: 5,
            containers_paused: 1,
            containers_stopped: 9,
            images: 30,
            docker_root_dir: "/var/lib/docker".to_string(),
            debug: true,
            experimental: false,
            mem_total: 8_589_934_592,
            ncpu: 8,
            operating_system: "Ubuntu 22.04 LTS".to_string(),
            os_type: "linux".to_string(),
            architecture: "x86_64".to_string(),
            kernel_version: "5.15.0-56-generic".to_string(),
            name: "test-docker".to_string(),
            id: "TEST:ABCD:1234".to_string(),
        };

        let docker_info = DockerInfo {
            system,
            registry: None,
            runtime: None,
            warnings: vec![
                "WARNING: No swap limit support".to_string(),
                "WARNING: No memory limit support".to_string(),
            ],
        };

        let output = InfoOutput {
            output: CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
            },
            docker_info: Some(docker_info),
        };

        assert_eq!(output.server_version(), Some("20.10.17"));
        assert_eq!(output.storage_driver(), Some("overlay2"));
        assert_eq!(output.container_count(), 15);
        assert_eq!(output.running_containers(), 5);
        assert_eq!(output.image_count(), 30);
        assert!(output.is_debug());
        assert!(!output.is_experimental());
        assert_eq!(output.operating_system(), Some("Ubuntu 22.04 LTS"));
        assert_eq!(output.architecture(), Some("x86_64"));
        assert!(output.has_warnings());
        assert_eq!(output.warnings().len(), 2);

        let (total, running, images) = output.resource_summary();
        assert_eq!(total, 15);
        assert_eq!(running, 5);
        assert_eq!(images, 30);
    }

    #[test]
    fn test_info_output_no_data() {
        let output = InfoOutput {
            output: CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
            },
            docker_info: None,
        };

        assert_eq!(output.server_version(), None);
        assert_eq!(output.storage_driver(), None);
        assert_eq!(output.container_count(), 0);
        assert_eq!(output.running_containers(), 0);
        assert_eq!(output.image_count(), 0);
        assert!(!output.is_debug());
        assert!(!output.is_experimental());
        assert!(!output.has_warnings());
        assert_eq!(output.warnings().len(), 0);

        let (total, running, images) = output.resource_summary();
        assert_eq!(total, 0);
        assert_eq!(running, 0);
        assert_eq!(images, 0);
    }

    #[test]
    fn test_info_command_display() {
        let info = InfoCommand::new().format("{{.ServerVersion}}");

        let display = format!("{info}");
        assert_eq!(display, "docker info --format {{.ServerVersion}}");
    }

    #[test]
    fn test_info_command_display_no_format() {
        let info = InfoCommand::new();

        let display = format!("{info}");
        assert_eq!(display, "docker info");
    }

    #[test]
    fn test_info_command_name() {
        let info = InfoCommand::new();
        let args = info.build_command_args();
        assert_eq!(args[0], "info");
    }

    #[test]
    fn test_info_command_extensibility() {
        let mut info = InfoCommand::new();

        // Test that we can add custom raw arguments
        info.get_executor_mut()
            .raw_args
            .push("--verbose".to_string());
        info.get_executor_mut()
            .raw_args
            .push("--some-flag".to_string());

        let args = info.build_command_args();

        // Verify raw args are included
        assert!(args.contains(&"--verbose".to_string()));
        assert!(args.contains(&"--some-flag".to_string()));
    }

    #[test]
    fn test_parse_json_output_concept() {
        // This test demonstrates the concept of parsing JSON output
        let json_output = r#"{"ServerVersion":"20.10.17","Driver":"overlay2","Containers":5}"#;

        let output = CommandOutput {
            stdout: json_output.to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };

        let result = InfoCommand::parse_json_output(&output);

        // The actual parsing would need real Docker JSON output
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_table_output_concept() {
        // This test demonstrates the concept of parsing table output
        let table_output = "Server Version: 20.10.17\nStorage Driver: overlay2\nContainers: 5\nRunning: 2\nImages: 10\nWARNING: Test warning";

        let output = CommandOutput {
            stdout: table_output.to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };

        let result = InfoCommand::parse_table_output(&output);

        // The actual parsing would need real Docker table output
        assert!(result.is_some() || result.is_none());
    }
}
