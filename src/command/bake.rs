//! Docker Bake Command Implementation
//!
//! This module provides a comprehensive implementation of the `docker bake` command,
//! supporting all native Docker buildx bake options for building from configuration files.
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```no_run
//! use docker_wrapper::BakeCommand;
//! use docker_wrapper::DockerCommand;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Basic bake with default docker-bake.hcl file
//!     let bake_cmd = BakeCommand::new();
//!     let output = bake_cmd.execute().await?;
//!     println!("Bake completed: {}", output.success);
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Usage
//!
//! ```no_run
//! use docker_wrapper::BakeCommand;
//! use docker_wrapper::DockerCommand;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Advanced bake with custom file and targets
//!     let bake_cmd = BakeCommand::new()
//!         .file("docker-compose.yml")
//!         .file("custom-bake.hcl")
//!         .target("web")
//!         .target("api")
//!         .push()
//!         .no_cache()
//!         .set("web.platform", "linux/amd64,linux/arm64")
//!         .metadata_file("build-metadata.json");
//!
//!     let output = bake_cmd.execute().await?;
//!     println!("Multi-target bake completed: {}", output.success);
//!     Ok(())
//! }
//! ```

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// Docker Bake Command Builder
///
/// Implements the `docker bake` command for building from configuration files
/// like docker-compose.yml, docker-bake.hcl, or custom bake definitions.
///
/// # Docker Bake Overview
///
/// The bake command allows you to build multiple targets defined in configuration
/// files, supporting advanced features like:
/// - Multi-platform builds
/// - Build matrix configurations
/// - Shared build contexts
/// - Variable substitution
/// - Target dependencies
///
/// # Supported File Formats
///
/// - `docker-compose.yml` - Docker Compose service definitions
/// - `docker-bake.hcl` - HCL (`HashiCorp` Configuration Language) format
/// - `docker-bake.json` - JSON format
/// - Custom build definition files
///
/// # Examples
///
/// ```no_run
/// use docker_wrapper::BakeCommand;
/// use docker_wrapper::DockerCommand;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Build all targets from docker-compose.yml
///     let output = BakeCommand::new()
///         .file("docker-compose.yml")
///         .execute()
///         .await?;
///
///     println!("Bake success: {}", output.success);
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct BakeCommand {
    /// Build targets to build (defaults to all targets if empty)
    targets: Vec<String>,
    /// Build definition files
    files: Vec<String>,
    /// Resource access permissions
    allow: Vec<String>,
    /// Builder instance override
    builder: Option<String>,
    /// Evaluation method (build, check, outline, targets)
    call: Option<String>,
    /// Enable check mode (shorthand for --call=check)
    check: bool,
    /// Enable debug logging
    debug: bool,
    /// List targets or variables
    list: Option<String>,
    /// Load images to Docker daemon (shorthand for --set=*.output=type=docker)
    load: bool,
    /// Build result metadata file
    metadata_file: Option<String>,
    /// Disable build cache
    no_cache: bool,
    /// Print options without building
    print: bool,
    /// Progress output type
    progress: Option<String>,
    /// Provenance attestation (shorthand for --set=*.attest=type=provenance)
    provenance: Option<String>,
    /// Always pull referenced images
    pull: bool,
    /// Push images to registry (shorthand for --set=*.output=type=registry)
    push: bool,
    /// SBOM attestation (shorthand for --set=*.attest=type=sbom)
    sbom: Option<String>,
    /// Target value overrides (key=value pairs)
    set_values: HashMap<String, String>,
    /// Command executor for handling raw arguments and execution
    pub executor: CommandExecutor,
}

impl BakeCommand {
    /// Create a new `BakeCommand` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
            files: Vec::new(),
            allow: Vec::new(),
            builder: None,
            call: None,
            check: false,
            debug: false,
            list: None,
            load: false,
            metadata_file: None,
            no_cache: false,
            print: false,
            progress: None,
            provenance: None,
            pull: false,
            push: false,
            sbom: None,
            set_values: HashMap::new(),
            executor: CommandExecutor::new(),
        }
    }

    /// Add a target to build
    ///
    /// Multiple targets can be specified. If no targets are specified,
    /// all targets defined in the bake file will be built.
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .target("web")
    ///     .target("api")
    ///     .target("worker");
    /// ```
    #[must_use]
    pub fn target<S: Into<String>>(mut self, target: S) -> Self {
        self.targets.push(target.into());
        self
    }

    /// Add multiple targets to build
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .targets(vec!["web", "api", "worker"]);
    /// ```
    #[must_use]
    pub fn targets<I, S>(mut self, targets: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.targets
            .extend(targets.into_iter().map(std::convert::Into::into));
        self
    }

    /// Add a build definition file
    ///
    /// Supports docker-compose.yml, docker-bake.hcl, docker-bake.json,
    /// and custom build definition files.
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .file("docker-compose.yml")
    ///     .file("custom-bake.hcl");
    /// ```
    #[must_use]
    pub fn file<S: Into<String>>(mut self, file: S) -> Self {
        self.files.push(file.into());
        self
    }

    /// Add multiple build definition files
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .files(vec!["docker-compose.yml", "override.yml"]);
    /// ```
    #[must_use]
    pub fn files<I, S>(mut self, files: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.files
            .extend(files.into_iter().map(std::convert::Into::into));
        self
    }

    /// Allow build to access specified resources
    ///
    /// Grants permission to access host resources during build.
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .allow("network.host")
    ///     .allow("security.insecure");
    /// ```
    #[must_use]
    pub fn allow<S: Into<String>>(mut self, resource: S) -> Self {
        self.allow.push(resource.into());
        self
    }

    /// Override the configured builder instance
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .builder("mybuilder");
    /// ```
    #[must_use]
    pub fn builder<S: Into<String>>(mut self, builder: S) -> Self {
        self.builder = Some(builder.into());
        self
    }

    /// Set method for evaluating build
    ///
    /// Valid values: "build", "check", "outline", "targets"
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .call("check"); // Validate build configuration
    /// ```
    #[must_use]
    pub fn call<S: Into<String>>(mut self, method: S) -> Self {
        self.call = Some(method.into());
        self
    }

    /// Enable check mode (shorthand for --call=check)
    ///
    /// Validates the build configuration without executing the build.
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .check();
    /// ```
    #[must_use]
    pub fn check(mut self) -> Self {
        self.check = true;
        self
    }

    /// Enable debug logging
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .debug();
    /// ```
    #[must_use]
    pub fn debug(mut self) -> Self {
        self.debug = true;
        self
    }

    /// List targets or variables
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .list("targets"); // List all available targets
    /// ```
    #[must_use]
    pub fn list<S: Into<String>>(mut self, list_type: S) -> Self {
        self.list = Some(list_type.into());
        self
    }

    /// Load images to Docker daemon (shorthand for --set=*.output=type=docker)
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .load();
    /// ```
    #[must_use]
    pub fn load(mut self) -> Self {
        self.load = true;
        self
    }

    /// Write build result metadata to a file
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .metadata_file("build-metadata.json");
    /// ```
    #[must_use]
    pub fn metadata_file<S: Into<String>>(mut self, file: S) -> Self {
        self.metadata_file = Some(file.into());
        self
    }

    /// Do not use cache when building images
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .no_cache();
    /// ```
    #[must_use]
    pub fn no_cache(mut self) -> Self {
        self.no_cache = true;
        self
    }

    /// Print the options without building
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .print();
    /// ```
    #[must_use]
    pub fn print(mut self) -> Self {
        self.print = true;
        self
    }

    /// Set type of progress output
    ///
    /// Valid values: "auto", "quiet", "plain", "tty", "rawjson"
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .progress("plain");
    /// ```
    #[must_use]
    pub fn progress<S: Into<String>>(mut self, progress_type: S) -> Self {
        self.progress = Some(progress_type.into());
        self
    }

    /// Set provenance attestation (shorthand for --set=*.attest=type=provenance)
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .provenance("mode=max");
    /// ```
    #[must_use]
    pub fn provenance<S: Into<String>>(mut self, provenance: S) -> Self {
        self.provenance = Some(provenance.into());
        self
    }

    /// Always attempt to pull all referenced images
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .pull();
    /// ```
    #[must_use]
    pub fn pull(mut self) -> Self {
        self.pull = true;
        self
    }

    /// Push images to registry (shorthand for --set=*.output=type=registry)
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .push();
    /// ```
    #[must_use]
    pub fn push(mut self) -> Self {
        self.push = true;
        self
    }

    /// Set SBOM attestation (shorthand for --set=*.attest=type=sbom)
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .sbom("generator=docker/buildkit");
    /// ```
    #[must_use]
    pub fn sbom<S: Into<String>>(mut self, sbom: S) -> Self {
        self.sbom = Some(sbom.into());
        self
    }

    /// Override target value (e.g., "targetpattern.key=value")
    ///
    /// This allows overriding any target configuration at build time.
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .set("web.platform", "linux/amd64,linux/arm64")
    ///     .set("*.output", "type=registry");
    /// ```
    #[must_use]
    pub fn set<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.set_values.insert(key.into(), value.into());
        self
    }

    /// Add multiple target value overrides
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use docker_wrapper::BakeCommand;
    ///
    /// let mut overrides = HashMap::new();
    /// overrides.insert("web.platform".to_string(), "linux/amd64".to_string());
    /// overrides.insert("api.platform".to_string(), "linux/arm64".to_string());
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .set_values(overrides);
    /// ```
    #[must_use]
    pub fn set_values<I, K, V>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.set_values
            .extend(values.into_iter().map(|(k, v)| (k.into(), v.into())));
        self
    }

    /// Get the number of targets
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .target("web")
    ///     .target("api");
    ///
    /// assert_eq!(bake_cmd.target_count(), 2);
    /// ```
    #[must_use]
    pub fn target_count(&self) -> usize {
        self.targets.len()
    }

    /// Get the list of targets
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .target("web")
    ///     .target("api");
    ///
    /// let targets = bake_cmd.get_targets();
    /// assert_eq!(targets, &["web", "api"]);
    /// ```
    #[must_use]
    pub fn get_targets(&self) -> &[String] {
        &self.targets
    }

    /// Get the list of build definition files
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new()
    ///     .file("docker-compose.yml");
    ///
    /// let files = bake_cmd.get_files();
    /// assert_eq!(files, &["docker-compose.yml"]);
    /// ```
    #[must_use]
    pub fn get_files(&self) -> &[String] {
        &self.files
    }

    /// Check if push mode is enabled
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new().push();
    /// assert!(bake_cmd.is_push_enabled());
    /// ```
    #[must_use]
    pub fn is_push_enabled(&self) -> bool {
        self.push
    }

    /// Check if load mode is enabled
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new().load();
    /// assert!(bake_cmd.is_load_enabled());
    /// ```
    #[must_use]
    pub fn is_load_enabled(&self) -> bool {
        self.load
    }

    /// Check if dry-run mode is enabled (print without building)
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::BakeCommand;
    ///
    /// let bake_cmd = BakeCommand::new().print();
    /// assert!(bake_cmd.is_dry_run());
    /// ```
    #[must_use]
    pub fn is_dry_run(&self) -> bool {
        self.print
    }
}

impl Default for BakeCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for BakeCommand {
    type Output = CommandOutput;

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["bake".to_string()];

        // Add file arguments
        for file in &self.files {
            args.push("--file".to_string());
            args.push(file.clone());
        }

        // Add allow arguments
        for allow in &self.allow {
            args.push("--allow".to_string());
            args.push(allow.clone());
        }

        // Add builder
        if let Some(ref builder) = self.builder {
            args.push("--builder".to_string());
            args.push(builder.clone());
        }

        // Add call method
        if let Some(ref call) = self.call {
            args.push("--call".to_string());
            args.push(call.clone());
        }

        // Add check flag
        if self.check {
            args.push("--check".to_string());
        }

        // Add debug flag
        if self.debug {
            args.push("--debug".to_string());
        }

        // Add list
        if let Some(ref list) = self.list {
            args.push("--list".to_string());
            args.push(list.clone());
        }

        // Add load flag
        if self.load {
            args.push("--load".to_string());
        }

        // Add metadata file
        if let Some(ref metadata_file) = self.metadata_file {
            args.push("--metadata-file".to_string());
            args.push(metadata_file.clone());
        }

        // Add no-cache flag
        if self.no_cache {
            args.push("--no-cache".to_string());
        }

        // Add print flag
        if self.print {
            args.push("--print".to_string());
        }

        // Add progress
        if let Some(ref progress) = self.progress {
            args.push("--progress".to_string());
            args.push(progress.clone());
        }

        // Add provenance
        if let Some(ref provenance) = self.provenance {
            args.push("--provenance".to_string());
            args.push(provenance.clone());
        }

        // Add pull flag
        if self.pull {
            args.push("--pull".to_string());
        }

        // Add push flag
        if self.push {
            args.push("--push".to_string());
        }

        // Add sbom
        if let Some(ref sbom) = self.sbom {
            args.push("--sbom".to_string());
            args.push(sbom.clone());
        }

        // Add set values
        for (key, value) in &self.set_values {
            args.push("--set".to_string());
            args.push(format!("{key}={value}"));
        }

        // Add targets at the end
        args.extend(self.targets.clone());
        args.extend(self.executor.raw_args.clone());
        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        let command_name = args[0].clone();
        let command_args = args[1..].to_vec();
        self.executor
            .execute_command(&command_name, command_args)
            .await
    }

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bake_command_basic() {
        let bake_cmd = BakeCommand::new();
        let args = bake_cmd.build_command_args();

        assert_eq!(args, vec!["bake"]); // Only bake command for basic case
    }

    #[test]
    fn test_bake_command_with_targets() {
        let bake_cmd = BakeCommand::new()
            .target("web")
            .target("api")
            .target("worker");

        let args = bake_cmd.build_command_args();

        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"api".to_string()));
        assert!(args.contains(&"worker".to_string()));
        assert_eq!(bake_cmd.target_count(), 3);
    }

    #[test]
    fn test_bake_command_with_files() {
        let bake_cmd = BakeCommand::new()
            .file("docker-compose.yml")
            .file("custom-bake.hcl");

        let args = bake_cmd.build_command_args();

        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"custom-bake.hcl".to_string()));
        assert_eq!(bake_cmd.get_files().len(), 2);
    }

    #[test]
    fn test_bake_command_push_and_load() {
        let bake_cmd = BakeCommand::new().push().load();

        let args = bake_cmd.build_command_args();

        assert!(args.contains(&"--push".to_string()));
        assert!(args.contains(&"--load".to_string()));
        assert!(bake_cmd.is_push_enabled());
        assert!(bake_cmd.is_load_enabled());
    }

    #[test]
    fn test_bake_command_with_builder() {
        let bake_cmd = BakeCommand::new().builder("mybuilder");

        let args = bake_cmd.build_command_args();

        assert!(args.contains(&"--builder".to_string()));
        assert!(args.contains(&"mybuilder".to_string()));
    }

    #[test]
    fn test_bake_command_with_set_values() {
        let bake_cmd = BakeCommand::new()
            .set("web.platform", "linux/amd64,linux/arm64")
            .set("*.output", "type=registry");

        let args = bake_cmd.build_command_args();

        assert!(args.contains(&"--set".to_string()));
        assert!(args.contains(&"web.platform=linux/amd64,linux/arm64".to_string()));
        assert!(args.contains(&"*.output=type=registry".to_string()));
    }

    #[test]
    fn test_bake_command_flags() {
        let bake_cmd = BakeCommand::new().check().debug().no_cache().print().pull();

        let args = bake_cmd.build_command_args();

        assert!(args.contains(&"--check".to_string()));
        assert!(args.contains(&"--debug".to_string()));
        assert!(args.contains(&"--no-cache".to_string()));
        assert!(args.contains(&"--print".to_string()));
        assert!(args.contains(&"--pull".to_string()));
        assert!(bake_cmd.is_dry_run());
    }

    #[test]
    fn test_bake_command_with_metadata_file() {
        let bake_cmd = BakeCommand::new().metadata_file("build-metadata.json");

        let args = bake_cmd.build_command_args();

        assert!(args.contains(&"--metadata-file".to_string()));
        assert!(args.contains(&"build-metadata.json".to_string()));
    }

    #[test]
    fn test_bake_command_with_progress() {
        let bake_cmd = BakeCommand::new().progress("plain");

        let args = bake_cmd.build_command_args();

        assert!(args.contains(&"--progress".to_string()));
        assert!(args.contains(&"plain".to_string()));
    }

    #[test]
    fn test_bake_command_with_attestations() {
        let bake_cmd = BakeCommand::new()
            .provenance("mode=max")
            .sbom("generator=docker/buildkit");

        let args = bake_cmd.build_command_args();

        assert!(args.contains(&"--provenance".to_string()));
        assert!(args.contains(&"mode=max".to_string()));
        assert!(args.contains(&"--sbom".to_string()));
        assert!(args.contains(&"generator=docker/buildkit".to_string()));
    }

    #[test]
    fn test_bake_command_with_allow() {
        let bake_cmd = BakeCommand::new()
            .allow("network.host")
            .allow("security.insecure");

        let args = bake_cmd.build_command_args();

        assert!(args.contains(&"--allow".to_string()));
        assert!(args.contains(&"network.host".to_string()));
        assert!(args.contains(&"security.insecure".to_string()));
    }

    #[test]
    fn test_bake_command_with_call() {
        let bake_cmd = BakeCommand::new().call("check");

        let args = bake_cmd.build_command_args();

        assert!(args.contains(&"--call".to_string()));
        assert!(args.contains(&"check".to_string()));
    }

    #[test]
    fn test_bake_command_with_list() {
        let bake_cmd = BakeCommand::new().list("targets");

        let args = bake_cmd.build_command_args();

        assert!(args.contains(&"--list".to_string()));
        assert!(args.contains(&"targets".to_string()));
    }

    #[test]
    fn test_bake_command_extensibility() {
        let mut bake_cmd = BakeCommand::new();
        bake_cmd.get_executor_mut().add_arg("--experimental");
        bake_cmd
            .get_executor_mut()
            .add_args(vec!["--custom", "value"]);

        // Extensibility is handled through the executor's raw_args
        // The actual testing of raw args is done in command.rs tests
        // We can verify the executor methods are accessible
        println!("Extensibility methods called successfully");
    }
}
