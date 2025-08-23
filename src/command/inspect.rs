//! Docker inspect command implementation.
//!
//! This module provides the `docker inspect` command for getting detailed information
//! about Docker objects (containers, images, volumes, networks, etc.).

use super::{CommandExecutor, CommandOutput, DockerCommandV2};
use crate::error::Result;
use async_trait::async_trait;
use serde_json::Value;

/// Docker inspect command builder
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::InspectCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Inspect a container
/// let info = InspectCommand::new("my-container")
///     .run()
///     .await?;
///
/// // Parse as JSON
/// let json = info.json()?;
/// println!("Container state: {}", json[0]["State"]["Status"]);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct InspectCommand {
    /// Objects to inspect (container/image/volume/network IDs or names)
    objects: Vec<String>,
    /// Output format
    format: Option<String>,
    /// Return size information
    size: bool,
    /// Type of object to inspect
    object_type: Option<String>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl InspectCommand {
    /// Create a new inspect command for a single object
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::InspectCommand;
    ///
    /// let cmd = InspectCommand::new("my-container");
    /// ```
    #[must_use]
    pub fn new(object: impl Into<String>) -> Self {
        Self {
            objects: vec![object.into()],
            format: None,
            size: false,
            object_type: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Create a new inspect command for multiple objects
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::InspectCommand;
    ///
    /// let cmd = InspectCommand::new_multiple(vec!["container1", "container2"]);
    /// ```
    #[must_use]
    pub fn new_multiple(objects: Vec<impl Into<String>>) -> Self {
        Self {
            objects: objects.into_iter().map(Into::into).collect(),
            format: None,
            size: false,
            object_type: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Add another object to inspect
    #[must_use]
    pub fn object(mut self, object: impl Into<String>) -> Self {
        self.objects.push(object.into());
        self
    }

    /// Set custom format string (Go template)
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::InspectCommand;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let cmd = InspectCommand::new("my-container")
    ///     .format("{{.State.Status}}");
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Display total file sizes
    #[must_use]
    pub fn size(mut self) -> Self {
        self.size = true;
        self
    }

    /// Specify the type of object to inspect
    ///
    /// Valid types: container, image, volume, network, plugin, node, service, etc.
    #[must_use]
    pub fn object_type(mut self, typ: impl Into<String>) -> Self {
        self.object_type = Some(typ.into());
        self
    }

    /// Execute the inspect command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - The specified object doesn't exist
    /// - The object type is invalid
    pub async fn run(&self) -> Result<InspectOutput> {
        let output = self.execute().await?;
        Ok(InspectOutput { output })
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

    /// Builds the command arguments for Docker inspect
    #[must_use]
    pub fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["inspect".to_string()];

        if let Some(ref format) = self.format {
            args.push("--format".to_string());
            args.push(format.clone());
        }

        if self.size {
            args.push("--size".to_string());
        }

        if let Some(ref typ) = self.object_type {
            args.push("--type".to_string());
            args.push(typ.clone());
        }

        // Add object names/IDs
        args.extend(self.objects.clone());

        // Add any additional raw arguments
        args.extend(self.executor.raw_args.clone());

        args
    }
}

#[async_trait]
impl DockerCommandV2 for InspectCommand {
    type Output = CommandOutput;

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
        self.execute_command(args).await
    }
}

/// Result from the inspect command
#[derive(Debug, Clone)]
pub struct InspectOutput {
    /// Raw command output
    pub output: CommandOutput,
}

impl InspectOutput {
    /// Parse the output as JSON
    ///
    /// # Errors
    /// Returns an error if the output is not valid JSON
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use docker_wrapper::InspectCommand;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let info = InspectCommand::new("my-container").run().await?;
    /// let json = info.json()?;
    /// println!("Container ID: {}", json[0]["Id"]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn json(&self) -> Result<Value> {
        serde_json::from_str(&self.output.stdout)
            .map_err(|e| crate::error::Error::parse_error(format!("Failed to parse JSON: {e}")))
    }

    /// Get raw stdout
    #[must_use]
    pub fn stdout(&self) -> &str {
        &self.output.stdout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspect_single_object() {
        let cmd = InspectCommand::new("test-container");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["inspect", "test-container"]);
    }

    #[test]
    fn test_inspect_multiple_objects() {
        let cmd = InspectCommand::new_multiple(vec!["container1", "image1", "volume1"]);
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["inspect", "container1", "image1", "volume1"]);
    }

    #[test]
    fn test_inspect_with_format() {
        let cmd = InspectCommand::new("test-container").format("{{.State.Status}}");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec!["inspect", "--format", "{{.State.Status}}", "test-container"]
        );
    }

    #[test]
    fn test_inspect_with_size() {
        let cmd = InspectCommand::new("test-image").size();
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["inspect", "--size", "test-image"]);
    }

    #[test]
    fn test_inspect_with_type() {
        let cmd = InspectCommand::new("my-network").object_type("network");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["inspect", "--type", "network", "my-network"]);
    }

    #[test]
    fn test_inspect_all_options() {
        let cmd = InspectCommand::new("test-container")
            .format("{{json .}}")
            .size()
            .object_type("container");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec![
                "inspect",
                "--format",
                "{{json .}}",
                "--size",
                "--type",
                "container",
                "test-container"
            ]
        );
    }
}
