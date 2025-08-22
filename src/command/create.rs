//! Docker create command implementation.
//!
//! This module provides the `docker create` command for creating containers without starting them.

use super::{CommandExecutor, CommandOutput, DockerCommand, EnvironmentBuilder, PortBuilder};
use crate::error::Result;
use async_trait::async_trait;
use std::ffi::OsStr;

/// Docker create command builder
#[allow(clippy::struct_excessive_bools)]
///
/// Create a new container without starting it. This is useful for preparing
/// containers that will be started later.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::CreateCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a simple container
/// let result = CreateCommand::new("alpine:latest")
///     .name("my-container")
///     .run()
///     .await?;
///
/// println!("Created container: {}", result.container_id());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct CreateCommand {
    /// Docker image to create container from
    image: String,
    /// Container name
    name: Option<String>,
    /// Command to run in container
    command: Vec<String>,
    /// Environment variables
    env_builder: EnvironmentBuilder,
    /// Port mappings
    port_builder: PortBuilder,
    /// Working directory
    workdir: Option<String>,
    /// User specification
    user: Option<String>,
    /// Hostname
    hostname: Option<String>,
    /// Attach to STDIN
    attach_stdin: bool,
    /// Attach to STDOUT  
    attach_stdout: bool,
    /// Attach to STDERR
    attach_stderr: bool,
    /// Keep STDIN open
    interactive: bool,
    /// Allocate a pseudo-TTY
    tty: bool,
    /// Volume mounts
    volumes: Vec<String>,
    /// Labels
    labels: Vec<String>,
    /// Memory limit
    memory: Option<String>,
    /// CPU limits
    cpus: Option<String>,
    /// Network mode
    network: Option<String>,
    /// Command executor
    executor: CommandExecutor,
}

impl CreateCommand {
    /// Create a new create command
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::CreateCommand;
    ///
    /// let cmd = CreateCommand::new("nginx:latest");
    /// ```
    #[must_use]
    pub fn new(image: impl Into<String>) -> Self {
        Self {
            image: image.into(),
            name: None,
            command: Vec::new(),
            env_builder: EnvironmentBuilder::new(),
            port_builder: PortBuilder::new(),
            workdir: None,
            user: None,
            hostname: None,
            attach_stdin: false,
            attach_stdout: false,
            attach_stderr: false,
            interactive: false,
            tty: false,
            volumes: Vec::new(),
            labels: Vec::new(),
            memory: None,
            cpus: None,
            network: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Set the container name
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::CreateCommand;
    ///
    /// let cmd = CreateCommand::new("alpine:latest")
    ///     .name("my-container");
    /// ```
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the command to run in the container
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::CreateCommand;
    ///
    /// let cmd = CreateCommand::new("alpine:latest")
    ///     .cmd(vec!["echo", "hello world"]);
    /// ```
    #[must_use]
    pub fn cmd(mut self, command: Vec<impl Into<String>>) -> Self {
        self.command = command.into_iter().map(Into::into).collect();
        self
    }

    /// Add an environment variable
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::CreateCommand;
    ///
    /// let cmd = CreateCommand::new("alpine:latest")
    ///     .env("KEY", "value")
    ///     .env("DEBUG", "true");
    /// ```
    #[must_use]
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_builder = self.env_builder.var(key, value);
        self
    }

    /// Add a port mapping
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::CreateCommand;
    ///
    /// let cmd = CreateCommand::new("nginx:latest")
    ///     .port(8080, 80);
    /// ```
    #[must_use]
    pub fn port(mut self, host_port: u16, container_port: u16) -> Self {
        self.port_builder = self.port_builder.port(host_port, container_port);
        self
    }

    /// Set working directory
    #[must_use]
    pub fn workdir(mut self, workdir: impl Into<String>) -> Self {
        self.workdir = Some(workdir.into());
        self
    }

    /// Set user
    #[must_use]
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Set hostname
    #[must_use]
    pub fn hostname(mut self, hostname: impl Into<String>) -> Self {
        self.hostname = Some(hostname.into());
        self
    }

    /// Attach to STDIN
    #[must_use]
    pub fn attach_stdin(mut self) -> Self {
        self.attach_stdin = true;
        self
    }

    /// Attach to STDOUT
    #[must_use]
    pub fn attach_stdout(mut self) -> Self {
        self.attach_stdout = true;
        self
    }

    /// Attach to STDERR
    #[must_use]
    pub fn attach_stderr(mut self) -> Self {
        self.attach_stderr = true;
        self
    }

    /// Enable interactive mode
    #[must_use]
    pub fn interactive(mut self) -> Self {
        self.interactive = true;
        self
    }

    /// Allocate a pseudo-TTY
    #[must_use]
    pub fn tty(mut self) -> Self {
        self.tty = true;
        self
    }

    /// Add a volume mount
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::CreateCommand;
    ///
    /// let cmd = CreateCommand::new("alpine:latest")
    ///     .volume("/host/path:/container/path")
    ///     .volume("/host/data:/data:ro");
    /// ```
    #[must_use]
    pub fn volume(mut self, volume: impl Into<String>) -> Self {
        self.volumes.push(volume.into());
        self
    }

    /// Add a label
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.labels.push(label.into());
        self
    }

    /// Set memory limit
    #[must_use]
    pub fn memory(mut self, memory: impl Into<String>) -> Self {
        self.memory = Some(memory.into());
        self
    }

    /// Set CPU limit
    #[must_use]
    pub fn cpus(mut self, cpus: impl Into<String>) -> Self {
        self.cpus = Some(cpus.into());
        self
    }

    /// Set network mode
    #[must_use]
    pub fn network(mut self, network: impl Into<String>) -> Self {
        self.network = Some(network.into());
        self
    }

    /// Execute the create command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - The specified image doesn't exist
    /// - Invalid configuration options
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::CreateCommand;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = CreateCommand::new("alpine:latest")
    ///     .name("test-container")
    ///     .cmd(vec!["echo", "hello"])
    ///     .run()
    ///     .await?;
    ///
    /// if result.success() {
    ///     println!("Created container: {}", result.container_id());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<CreateResult> {
        let output = self.execute().await?;

        // Parse container ID from output
        let container_id = output.stdout.trim().to_string();

        Ok(CreateResult {
            output,
            container_id,
        })
    }
}

#[async_trait]
impl DockerCommand for CreateCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "create"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(ref name) = self.name {
            args.push("--name".to_string());
            args.push(name.clone());
        }

        // Environment variables
        args.extend(self.env_builder.build_args());

        // Port mappings
        args.extend(self.port_builder.build_args());

        if let Some(ref workdir) = self.workdir {
            args.push("--workdir".to_string());
            args.push(workdir.clone());
        }

        if let Some(ref user) = self.user {
            args.push("--user".to_string());
            args.push(user.clone());
        }

        if let Some(ref hostname) = self.hostname {
            args.push("--hostname".to_string());
            args.push(hostname.clone());
        }

        if self.attach_stdin {
            args.push("--attach".to_string());
            args.push("STDIN".to_string());
        }

        if self.attach_stdout {
            args.push("--attach".to_string());
            args.push("STDOUT".to_string());
        }

        if self.attach_stderr {
            args.push("--attach".to_string());
            args.push("STDERR".to_string());
        }

        if self.interactive {
            args.push("--interactive".to_string());
        }

        if self.tty {
            args.push("--tty".to_string());
        }

        for volume in &self.volumes {
            args.push("--volume".to_string());
            args.push(volume.clone());
        }

        for label in &self.labels {
            args.push("--label".to_string());
            args.push(label.clone());
        }

        if let Some(ref memory) = self.memory {
            args.push("--memory".to_string());
            args.push(memory.clone());
        }

        if let Some(ref cpus) = self.cpus {
            args.push("--cpus".to_string());
            args.push(cpus.clone());
        }

        if let Some(ref network) = self.network {
            args.push("--network".to_string());
            args.push(network.clone());
        }

        // Add image
        args.push(self.image.clone());

        // Add command
        args.extend(self.command.clone());

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        self.executor
            .execute_command(self.command_name(), self.build_args())
            .await
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

/// Result from the create command
#[derive(Debug, Clone)]
pub struct CreateResult {
    /// Raw command output
    pub output: CommandOutput,
    /// ID of the created container
    pub container_id: String,
}

impl CreateResult {
    /// Check if the create was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success && !self.container_id.is_empty()
    }

    /// Get the created container ID
    #[must_use]
    pub fn container_id(&self) -> &str {
        &self.container_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_basic() {
        let cmd = CreateCommand::new("alpine:latest");
        let args = cmd.build_args();
        assert_eq!(args, vec!["alpine:latest"]);
    }

    #[test]
    fn test_create_with_name() {
        let cmd = CreateCommand::new("alpine:latest").name("test-container");
        let args = cmd.build_args();
        assert_eq!(args, vec!["--name", "test-container", "alpine:latest"]);
    }

    #[test]
    fn test_create_with_command() {
        let cmd = CreateCommand::new("alpine:latest").cmd(vec!["echo", "hello"]);
        let args = cmd.build_args();
        assert_eq!(args, vec!["alpine:latest", "echo", "hello"]);
    }

    #[test]
    fn test_create_with_env() {
        let cmd = CreateCommand::new("alpine:latest")
            .env("KEY1", "value1")
            .env("KEY2", "value2");
        let args = cmd.build_args();
        assert!(args.contains(&"--env".to_string()));
        assert!(args.contains(&"KEY1=value1".to_string()));
        assert!(args.contains(&"KEY2=value2".to_string()));
    }

    #[test]
    fn test_create_with_ports() {
        let cmd = CreateCommand::new("nginx:latest").port(8080, 80);
        let args = cmd.build_args();
        assert!(args.contains(&"--publish".to_string()));
        assert!(args.contains(&"8080:80".to_string()));
    }

    #[test]
    fn test_create_with_volumes() {
        let cmd = CreateCommand::new("alpine:latest")
            .volume("/host:/container")
            .volume("/data:/app/data:ro");
        let args = cmd.build_args();
        assert!(args.contains(&"--volume".to_string()));
        assert!(args.contains(&"/host:/container".to_string()));
        assert!(args.contains(&"/data:/app/data:ro".to_string()));
    }

    #[test]
    fn test_create_interactive_tty() {
        let cmd = CreateCommand::new("alpine:latest").interactive().tty();
        let args = cmd.build_args();
        assert!(args.contains(&"--interactive".to_string()));
        assert!(args.contains(&"--tty".to_string()));
    }

    #[test]
    fn test_create_all_options() {
        let cmd = CreateCommand::new("alpine:latest")
            .name("test-container")
            .cmd(vec!["sh", "-c", "echo hello"])
            .env("DEBUG", "true")
            .port(8080, 80)
            .workdir("/app")
            .user("1000:1000")
            .hostname("test-host")
            .interactive()
            .tty()
            .volume("/data:/app/data")
            .label("version=1.0")
            .memory("512m")
            .cpus("0.5")
            .network("bridge");

        let args = cmd.build_args();

        // Verify key arguments are present
        assert!(args.contains(&"--name".to_string()));
        assert!(args.contains(&"test-container".to_string()));
        assert!(args.contains(&"--workdir".to_string()));
        assert!(args.contains(&"/app".to_string()));
        assert!(args.contains(&"--interactive".to_string()));
        assert!(args.contains(&"--tty".to_string()));
        assert!(args.contains(&"alpine:latest".to_string()));
        assert!(args.contains(&"sh".to_string()));
        assert!(args.contains(&"-c".to_string()));
        assert!(args.contains(&"echo hello".to_string()));
    }
}
