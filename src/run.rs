//! Docker run command implementation.
//!
//! This module provides a comprehensive implementation of the `docker run` command
//! with support for common options and an extensible architecture for any additional options.

use crate::command::{CommandExecutor, DockerCommand, EnvironmentBuilder, PortBuilder};
use crate::error::{Error, Result};
use async_trait::async_trait;
use std::ffi::OsStr;
use std::path::PathBuf;

/// Docker run command builder with fluent API
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct RunCommand {
    /// The Docker image to run
    image: String,
    /// Command executor for extensibility
    executor: CommandExecutor,
    /// Container name
    name: Option<String>,
    /// Run in detached mode
    detach: bool,
    /// Environment variables
    environment: EnvironmentBuilder,
    /// Port mappings
    ports: PortBuilder,
    /// Volume mounts
    volumes: Vec<VolumeMount>,
    /// Working directory
    workdir: Option<PathBuf>,
    /// Entrypoint override
    entrypoint: Option<String>,
    /// Command to run in container
    command: Option<Vec<String>>,
    /// Interactive mode
    interactive: bool,
    /// Allocate TTY
    tty: bool,
    /// Remove container on exit
    remove: bool,
}

/// Volume mount configuration
#[derive(Debug, Clone)]
pub struct VolumeMount {
    /// Source path on host or volume name
    pub source: String,
    /// Target path in container
    pub target: String,
    /// Mount type (bind, volume, tmpfs)
    pub mount_type: MountType,
    /// Read-only mount
    pub readonly: bool,
}

/// Type of volume mount
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountType {
    /// Bind mount from host filesystem
    Bind,
    /// Named volume
    Volume,
    /// Temporary filesystem
    Tmpfs,
}

impl std::fmt::Display for VolumeMount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let readonly_suffix = if self.readonly { ":ro" } else { "" };
        write!(f, "{}:{}{}", self.source, self.target, readonly_suffix)
    }
}

/// Container ID returned by docker run
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerId(pub String);

impl ContainerId {
    /// Get the container ID as a string
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the short form of the container ID (first 12 characters)
    #[must_use]
    pub fn short(&self) -> &str {
        if self.0.len() >= 12 {
            &self.0[..12]
        } else {
            &self.0
        }
    }
}

impl std::fmt::Display for ContainerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl RunCommand {
    /// Create a new run command for the specified image
    pub fn new(image: impl Into<String>) -> Self {
        Self {
            image: image.into(),
            executor: CommandExecutor::new(),
            name: None,
            detach: false,
            environment: EnvironmentBuilder::new(),
            ports: PortBuilder::new(),
            volumes: Vec::new(),
            workdir: None,
            entrypoint: None,
            command: None,
            interactive: false,
            tty: false,
            remove: false,
        }
    }

    /// Set the container name
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Run in detached mode (background)
    #[must_use]
    pub fn detach(mut self) -> Self {
        self.detach = true;
        self
    }

    /// Add an environment variable
    #[must_use]
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment = self.environment.var(key, value);
        self
    }

    /// Add multiple environment variables
    #[must_use]
    pub fn envs(mut self, vars: std::collections::HashMap<String, String>) -> Self {
        self.environment = self.environment.vars(vars);
        self
    }

    /// Add a port mapping
    #[must_use]
    pub fn port(mut self, host_port: u16, container_port: u16) -> Self {
        self.ports = self.ports.port(host_port, container_port);
        self
    }

    /// Add a dynamic port mapping (Docker assigns host port)
    #[must_use]
    pub fn dynamic_port(mut self, container_port: u16) -> Self {
        self.ports = self.ports.dynamic_port(container_port);
        self
    }

    /// Add a volume mount
    #[must_use]
    pub fn volume(mut self, source: impl Into<String>, target: impl Into<String>) -> Self {
        self.volumes.push(VolumeMount {
            source: source.into(),
            target: target.into(),
            mount_type: MountType::Volume,
            readonly: false,
        });
        self
    }

    /// Add a bind mount
    #[must_use]
    pub fn bind(mut self, source: impl Into<String>, target: impl Into<String>) -> Self {
        self.volumes.push(VolumeMount {
            source: source.into(),
            target: target.into(),
            mount_type: MountType::Bind,
            readonly: false,
        });
        self
    }

    /// Add a read-only volume mount
    #[must_use]
    pub fn volume_ro(mut self, source: impl Into<String>, target: impl Into<String>) -> Self {
        self.volumes.push(VolumeMount {
            source: source.into(),
            target: target.into(),
            mount_type: MountType::Volume,
            readonly: true,
        });
        self
    }

    /// Set working directory
    #[must_use]
    pub fn workdir(mut self, workdir: impl Into<PathBuf>) -> Self {
        self.workdir = Some(workdir.into());
        self
    }

    /// Override entrypoint
    #[must_use]
    pub fn entrypoint(mut self, entrypoint: impl Into<String>) -> Self {
        self.entrypoint = Some(entrypoint.into());
        self
    }

    /// Set command to run in container
    #[must_use]
    pub fn cmd(mut self, command: Vec<String>) -> Self {
        self.command = Some(command);
        self
    }

    /// Enable interactive mode
    #[must_use]
    pub fn interactive(mut self) -> Self {
        self.interactive = true;
        self
    }

    /// Allocate a TTY
    #[must_use]
    pub fn tty(mut self) -> Self {
        self.tty = true;
        self
    }

    /// Remove container automatically when it exits
    #[must_use]
    pub fn remove(mut self) -> Self {
        self.remove = true;
        self
    }

    /// Convenience method for interactive TTY mode
    #[must_use]
    pub fn it(self) -> Self {
        self.interactive().tty()
    }
}

#[async_trait]
impl DockerCommand for RunCommand {
    type Output = ContainerId;

    fn command_name(&self) -> &'static str {
        "run"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Add flags
        if self.detach {
            args.push("--detach".to_string());
        }
        if self.interactive {
            args.push("--interactive".to_string());
        }
        if self.tty {
            args.push("--tty".to_string());
        }
        if self.remove {
            args.push("--rm".to_string());
        }

        // Add container name
        if let Some(ref name) = self.name {
            args.push("--name".to_string());
            args.push(name.clone());
        }

        // Add working directory
        if let Some(ref workdir) = self.workdir {
            args.push("--workdir".to_string());
            args.push(workdir.to_string_lossy().to_string());
        }

        // Add entrypoint
        if let Some(ref entrypoint) = self.entrypoint {
            args.push("--entrypoint".to_string());
            args.push(entrypoint.clone());
        }

        // Add environment variables
        args.extend(self.environment.build_args());

        // Add port mappings
        args.extend(self.ports.build_args());

        // Add volume mounts
        for volume in &self.volumes {
            args.push("--volume".to_string());
            args.push(volume.to_string());
        }

        // Add image
        args.push(self.image.clone());

        // Add command if specified
        if let Some(ref command) = self.command {
            args.extend(command.clone());
        }

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_args();
        let output = self
            .executor
            .execute_command(self.command_name(), args)
            .await?;

        // Parse container ID from output
        let container_id = output.stdout.trim().to_string();
        if container_id.is_empty() {
            return Err(Error::parse_error(
                "No container ID returned from docker run",
            ));
        }

        Ok(ContainerId(container_id))
    }

    fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.executor.add_arg(arg);
        self
    }

    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.executor.add_args(args);
        self
    }

    fn flag(&mut self, flag: &str) -> &mut Self {
        self.executor.add_flag(flag);
        self
    }

    fn option(&mut self, key: &str, value: &str) -> &mut Self {
        self.executor.add_option(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_command_builder() {
        let cmd = RunCommand::new("nginx:latest")
            .name("test-nginx")
            .detach()
            .env("ENV_VAR", "value")
            .port(8080, 80)
            .volume("data", "/var/data")
            .workdir("/app")
            .remove();

        let args = cmd.build_args();

        assert!(args.contains(&"--detach".to_string()));
        assert!(args.contains(&"--name".to_string()));
        assert!(args.contains(&"test-nginx".to_string()));
        assert!(args.contains(&"--env".to_string()));
        assert!(args.contains(&"ENV_VAR=value".to_string()));
        assert!(args.contains(&"--publish".to_string()));
        assert!(args.contains(&"8080:80".to_string()));
        assert!(args.contains(&"--volume".to_string()));
        assert!(args.contains(&"data:/var/data".to_string()));
        assert!(args.contains(&"--workdir".to_string()));
        assert!(args.contains(&"/app".to_string()));
        assert!(args.contains(&"--rm".to_string()));
        assert!(args.contains(&"nginx:latest".to_string()));
    }

    #[test]
    fn test_run_command_with_cmd() {
        let cmd =
            RunCommand::new("alpine:latest").cmd(vec!["echo".to_string(), "hello".to_string()]);

        let args = cmd.build_args();
        assert!(args.contains(&"alpine:latest".to_string()));
        assert!(args.contains(&"echo".to_string()));
        assert!(args.contains(&"hello".to_string()));
    }

    #[test]
    fn test_run_command_extensibility() {
        let mut cmd = RunCommand::new("test:latest");
        cmd.flag("privileged")
            .option("memory", "1g")
            .arg("--custom-option");

        // The extensibility is tested via the trait methods
        // Full integration testing would require actual Docker execution
    }

    #[test]
    fn test_volume_mount_display() {
        let volume = VolumeMount {
            source: "data".to_string(),
            target: "/var/data".to_string(),
            mount_type: MountType::Volume,
            readonly: false,
        };
        assert_eq!(volume.to_string(), "data:/var/data");

        let readonly_volume = VolumeMount {
            source: "/host/path".to_string(),
            target: "/container/path".to_string(),
            mount_type: MountType::Bind,
            readonly: true,
        };
        assert_eq!(readonly_volume.to_string(), "/host/path:/container/path:ro");
    }

    #[test]
    fn test_container_id() {
        let id = ContainerId("abcdef123456789".to_string());
        assert_eq!(id.as_str(), "abcdef123456789");
        assert_eq!(id.short(), "abcdef123456");
        assert_eq!(id.to_string(), "abcdef123456789");

        let short_id = ContainerId("abc".to_string());
        assert_eq!(short_id.short(), "abc");
    }

    #[test]
    fn test_it_convenience_method() {
        let cmd = RunCommand::new("alpine:latest").it();
        let args = cmd.build_args();
        assert!(args.contains(&"--interactive".to_string()));
        assert!(args.contains(&"--tty".to_string()));
    }
}
