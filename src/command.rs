//! Command trait architecture for extensible Docker command implementations.
//!
//! This module provides a base trait that all Docker commands implement,
//! allowing for both structured high-level APIs and escape hatches for
//! any unimplemented options via raw arguments.

use crate::error::{Error, Result};
use crate::platform::PlatformInfo;
use async_trait::async_trait;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::process::Stdio;
use tokio::process::Command as TokioCommand;

// Re-export all command modules
pub mod attach;
pub mod bake;
pub mod build;
pub mod commit;
pub mod cp;
pub mod create;
pub mod diff;
pub mod events;
pub mod exec;
pub mod export;
pub mod history;
pub mod images;
pub mod import;
pub mod info;
pub mod inspect;
pub mod kill;
pub mod load;
pub mod login;
pub mod logout;
pub mod logs;
pub mod pause;
pub mod port;
pub mod ps;
pub mod pull;
pub mod push;
pub mod rename;
pub mod restart;
pub mod rm;
pub mod rmi;
pub mod run;
pub mod save;
pub mod search;
pub mod start;
pub mod stats;
pub mod stop;
pub mod tag;
pub mod top;
pub mod unpause;
pub mod update;
pub mod version;
pub mod wait;

/// Base trait for all Docker commands
#[async_trait]
pub trait DockerCommand {
    /// The output type this command produces
    type Output;

    /// Get the command name (e.g., "run", "exec", "ps")
    fn command_name(&self) -> &'static str;

    /// Build the command arguments
    fn build_args(&self) -> Vec<String>;

    /// Execute the command and return the typed output
    async fn execute(&self) -> Result<Self::Output>;

    /// Add a raw argument to the command (escape hatch)
    fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self;

    /// Add multiple raw arguments to the command (escape hatch)
    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;

    /// Add a flag option (e.g., --detach, --rm)
    fn flag(&mut self, flag: &str) -> &mut Self;

    /// Add a key-value option (e.g., --name value, --env key=value)
    fn option(&mut self, key: &str, value: &str) -> &mut Self;
}

/// Common functionality for executing Docker commands
#[derive(Debug, Clone)]
pub struct CommandExecutor {
    /// Additional raw arguments added via escape hatch
    pub raw_args: Vec<String>,
    /// Platform information for runtime abstraction
    pub platform_info: Option<PlatformInfo>,
}

impl CommandExecutor {
    /// Create a new command executor
    #[must_use]
    pub fn new() -> Self {
        Self {
            raw_args: Vec::new(),
            platform_info: None,
        }
    }

    /// Create a new command executor with platform detection
    ///
    /// # Errors
    ///
    /// Returns an error if platform detection fails
    pub fn with_platform() -> Result<Self> {
        let platform_info = PlatformInfo::detect()?;
        Ok(Self {
            raw_args: Vec::new(),
            platform_info: Some(platform_info),
        })
    }

    /// Set the platform information
    #[must_use]
    pub fn platform(mut self, platform_info: PlatformInfo) -> Self {
        self.platform_info = Some(platform_info);
        self
    }

    /// Get the runtime command to use
    fn get_runtime_command(&self) -> String {
        if let Some(ref platform_info) = self.platform_info {
            platform_info.runtime.command().to_string()
        } else {
            "docker".to_string()
        }
    }

    /// Execute a Docker command with the given arguments
    ///
    /// # Errors
    /// Returns an error if the Docker command fails to execute or returns a non-zero exit code
    pub async fn execute_command(
        &self,
        command_name: &str,
        args: Vec<String>,
    ) -> Result<CommandOutput> {
        // Prepend raw args (they should come before command-specific args)
        let mut all_args = self.raw_args.clone();
        all_args.extend(args);

        // Insert the command name at the beginning
        all_args.insert(0, command_name.to_string());

        let runtime_command = self.get_runtime_command();
        let mut command = TokioCommand::new(&runtime_command);

        // Set environment variables from platform info
        if let Some(ref platform_info) = self.platform_info {
            for (key, value) in platform_info.environment_vars() {
                command.env(key, value);
            }
        }

        let output = command
            .args(&all_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                Error::custom(format!(
                    "Failed to execute {runtime_command} {command_name}: {e}"
                ))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();
        let exit_code = output.status.code().unwrap_or(-1);

        if !success {
            return Err(Error::command_failed(
                format!("{} {}", runtime_command, all_args.join(" ")),
                exit_code,
                stdout,
                stderr,
            ));
        }

        Ok(CommandOutput {
            stdout,
            stderr,
            exit_code,
            success,
        })
    }

    /// Add a raw argument
    pub fn add_arg<S: AsRef<OsStr>>(&mut self, arg: S) {
        self.raw_args
            .push(arg.as_ref().to_string_lossy().to_string());
    }

    /// Add multiple raw arguments
    pub fn add_args<I, S>(&mut self, args: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for arg in args {
            self.add_arg(arg);
        }
    }

    /// Add a flag option
    pub fn add_flag(&mut self, flag: &str) {
        let flag_arg = if flag.starts_with('-') {
            flag.to_string()
        } else if flag.len() == 1 {
            format!("-{flag}")
        } else {
            format!("--{flag}")
        };
        self.raw_args.push(flag_arg);
    }

    /// Add a key-value option
    pub fn add_option(&mut self, key: &str, value: &str) {
        let key_arg = if key.starts_with('-') {
            key.to_string()
        } else if key.len() == 1 {
            format!("-{key}")
        } else {
            format!("--{key}")
        };
        self.raw_args.push(key_arg);
        self.raw_args.push(value.to_string());
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Output from executing a Docker command
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Standard output from the command
    pub stdout: String,
    /// Standard error from the command
    pub stderr: String,
    /// Exit code
    pub exit_code: i32,
    /// Whether the command was successful
    pub success: bool,
}

impl CommandOutput {
    /// Get stdout lines as a vector
    #[must_use]
    pub fn stdout_lines(&self) -> Vec<&str> {
        self.stdout.lines().collect()
    }

    /// Get stderr lines as a vector
    #[must_use]
    pub fn stderr_lines(&self) -> Vec<&str> {
        self.stderr.lines().collect()
    }

    /// Check if stdout is empty
    #[must_use]
    pub fn stdout_is_empty(&self) -> bool {
        self.stdout.trim().is_empty()
    }

    /// Check if stderr is empty
    #[must_use]
    pub fn stderr_is_empty(&self) -> bool {
        self.stderr.trim().is_empty()
    }
}

/// Helper for building environment variables
#[derive(Debug, Clone, Default)]
pub struct EnvironmentBuilder {
    vars: HashMap<String, String>,
}

impl EnvironmentBuilder {
    /// Create a new environment builder
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an environment variable
    #[must_use]
    pub fn var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.vars.insert(key.into(), value.into());
        self
    }

    /// Add multiple environment variables from a `HashMap`
    #[must_use]
    pub fn vars(mut self, vars: HashMap<String, String>) -> Self {
        self.vars.extend(vars);
        self
    }

    /// Build the environment arguments for Docker
    #[must_use]
    pub fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        for (key, value) in &self.vars {
            args.push("--env".to_string());
            args.push(format!("{key}={value}"));
        }
        args
    }

    /// Get the environment variables as a `HashMap`
    #[must_use]
    pub fn as_map(&self) -> &HashMap<String, String> {
        &self.vars
    }
}

/// Helper for building port mappings
#[derive(Debug, Clone, Default)]
pub struct PortBuilder {
    mappings: Vec<PortMapping>,
}

impl PortBuilder {
    /// Create a new port builder
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a port mapping
    #[must_use]
    pub fn port(mut self, host_port: u16, container_port: u16) -> Self {
        self.mappings.push(PortMapping {
            host_port: Some(host_port),
            container_port,
            protocol: Protocol::Tcp,
            host_ip: None,
        });
        self
    }

    /// Add a port mapping with protocol
    #[must_use]
    pub fn port_with_protocol(
        mut self,
        host_port: u16,
        container_port: u16,
        protocol: Protocol,
    ) -> Self {
        self.mappings.push(PortMapping {
            host_port: Some(host_port),
            container_port,
            protocol,
            host_ip: None,
        });
        self
    }

    /// Add a dynamic port mapping (Docker assigns host port)
    #[must_use]
    pub fn dynamic_port(mut self, container_port: u16) -> Self {
        self.mappings.push(PortMapping {
            host_port: None,
            container_port,
            protocol: Protocol::Tcp,
            host_ip: None,
        });
        self
    }

    /// Build the port arguments for Docker
    #[must_use]
    pub fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        for mapping in &self.mappings {
            args.push("--publish".to_string());
            args.push(mapping.to_string());
        }
        args
    }

    /// Get the port mappings
    #[must_use]
    pub fn mappings(&self) -> &[PortMapping] {
        &self.mappings
    }
}

/// Port mapping configuration
#[derive(Debug, Clone)]
pub struct PortMapping {
    /// Host port (None for dynamic allocation)
    pub host_port: Option<u16>,
    /// Container port
    pub container_port: u16,
    /// Protocol (TCP or UDP)
    pub protocol: Protocol,
    /// Host IP to bind to (None for all interfaces)
    pub host_ip: Option<std::net::IpAddr>,
}

impl std::fmt::Display for PortMapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let protocol_suffix = match self.protocol {
            Protocol::Tcp => "",
            Protocol::Udp => "/udp",
        };

        if let Some(host_port) = self.host_port {
            if let Some(host_ip) = self.host_ip {
                write!(
                    f,
                    "{}:{}:{}{}",
                    host_ip, host_port, self.container_port, protocol_suffix
                )
            } else {
                write!(
                    f,
                    "{}:{}{}",
                    host_port, self.container_port, protocol_suffix
                )
            }
        } else {
            write!(f, "{}{}", self.container_port, protocol_suffix)
        }
    }
}

/// Network protocol for port mappings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// TCP protocol
    Tcp,
    /// UDP protocol
    Udp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_executor_args() {
        let mut executor = CommandExecutor::new();
        executor.add_arg("test");
        executor.add_args(vec!["arg1", "arg2"]);
        executor.add_flag("detach");
        executor.add_flag("d");
        executor.add_option("name", "test-container");

        assert_eq!(
            executor.raw_args,
            vec![
                "test",
                "arg1",
                "arg2",
                "--detach",
                "-d",
                "--name",
                "test-container"
            ]
        );
    }

    #[test]
    fn test_environment_builder() {
        let env = EnvironmentBuilder::new()
            .var("KEY1", "value1")
            .var("KEY2", "value2");

        let args = env.build_args();
        assert!(args.contains(&"--env".to_string()));
        assert!(args.contains(&"KEY1=value1".to_string()));
        assert!(args.contains(&"KEY2=value2".to_string()));
    }

    #[test]
    fn test_port_builder() {
        let ports = PortBuilder::new()
            .port(8080, 80)
            .dynamic_port(443)
            .port_with_protocol(8081, 81, Protocol::Udp);

        let args = ports.build_args();
        assert!(args.contains(&"--publish".to_string()));
        assert!(args.contains(&"8080:80".to_string()));
        assert!(args.contains(&"443".to_string()));
        assert!(args.contains(&"8081:81/udp".to_string()));
    }

    #[test]
    fn test_port_mapping_display() {
        let tcp_mapping = PortMapping {
            host_port: Some(8080),
            container_port: 80,
            protocol: Protocol::Tcp,
            host_ip: None,
        };
        assert_eq!(tcp_mapping.to_string(), "8080:80");

        let udp_mapping = PortMapping {
            host_port: Some(8081),
            container_port: 81,
            protocol: Protocol::Udp,
            host_ip: None,
        };
        assert_eq!(udp_mapping.to_string(), "8081:81/udp");

        let dynamic_mapping = PortMapping {
            host_port: None,
            container_port: 443,
            protocol: Protocol::Tcp,
            host_ip: None,
        };
        assert_eq!(dynamic_mapping.to_string(), "443");
    }

    #[test]
    fn test_command_output_helpers() {
        let output = CommandOutput {
            stdout: "line1\nline2".to_string(),
            stderr: "error1\nerror2".to_string(),
            exit_code: 0,
            success: true,
        };

        assert_eq!(output.stdout_lines(), vec!["line1", "line2"]);
        assert_eq!(output.stderr_lines(), vec!["error1", "error2"]);
        assert!(!output.stdout_is_empty());
        assert!(!output.stderr_is_empty());

        let empty_output = CommandOutput {
            stdout: "   ".to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };

        assert!(empty_output.stdout_is_empty());
        assert!(empty_output.stderr_is_empty());
    }
}
