//! Docker build command implementation.
//!
//! This module provides a comprehensive implementation of the `docker build` command
//! with support for all native options and an extensible architecture for any additional options.

use crate::command::{CommandExecutor, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;

/// Docker build command builder with fluent API
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct BuildCommand {
    /// Build context (path, URL, or stdin)
    context: String,
    /// Command executor for extensibility
    executor: CommandExecutor,
    /// Custom host-to-IP mappings
    add_hosts: Vec<String>,
    /// Build-time variables
    build_args: HashMap<String, String>,
    /// Images to consider as cache sources
    cache_from: Vec<String>,
    /// Parent cgroup for RUN instructions
    cgroup_parent: Option<String>,
    /// Compress the build context using gzip
    compress: bool,
    /// CPU limits
    cpu_period: Option<i64>,
    cpu_quota: Option<i64>,
    cpu_shares: Option<i64>,
    cpuset_cpus: Option<String>,
    cpuset_mems: Option<String>,
    /// Skip image verification
    disable_content_trust: bool,
    /// Name of the Dockerfile
    file: Option<PathBuf>,
    /// Always remove intermediate containers
    force_rm: bool,
    /// Write the image ID to file
    iidfile: Option<PathBuf>,
    /// Container isolation technology
    isolation: Option<String>,
    /// Set metadata for an image
    labels: HashMap<String, String>,
    /// Memory limit
    memory: Option<String>,
    /// Memory + swap limit
    memory_swap: Option<String>,
    /// Networking mode for RUN instructions
    network: Option<String>,
    /// Do not use cache when building
    no_cache: bool,
    /// Set platform for multi-platform builds
    platform: Option<String>,
    /// Always attempt to pull newer base images
    pull: bool,
    /// Suppress build output and print image ID on success
    quiet: bool,
    /// Remove intermediate containers after successful build
    rm: bool,
    /// Security options
    security_opts: Vec<String>,
    /// Size of /dev/shm
    shm_size: Option<String>,
    /// Name and tag for the image
    tags: Vec<String>,
    /// Target build stage
    target: Option<String>,
    /// Ulimit options
    ulimits: Vec<String>,
    /// Extra privileged entitlements
    allow: Vec<String>,
    /// Annotations to add to the image
    annotations: Vec<String>,
    /// Attestation parameters
    attestations: Vec<String>,
    /// Additional build contexts
    build_contexts: Vec<String>,
    /// Override the configured builder
    builder: Option<String>,
    /// Cache export destinations
    cache_to: Vec<String>,
    /// Method for evaluating build
    call: Option<String>,
    /// Shorthand for "--call=check"
    check: bool,
    /// Shorthand for "--output=type=docker"
    load: bool,
    /// Write build result metadata to file
    metadata_file: Option<PathBuf>,
    /// Do not cache specified stages
    no_cache_filter: Vec<String>,
    /// Type of progress output
    progress: Option<String>,
    /// Shorthand for "--attest=type=provenance"
    provenance: Option<String>,
    /// Shorthand for "--output=type=registry"
    push: bool,
    /// Shorthand for "--attest=type=sbom"
    sbom: Option<String>,
    /// Secrets to expose to the build
    secrets: Vec<String>,
    /// SSH agent socket or keys to expose
    ssh: Vec<String>,
}

/// Output from docker build command
#[derive(Debug, Clone)]
pub struct BuildOutput {
    /// The raw stdout from the command
    pub stdout: String,
    /// The raw stderr from the command
    pub stderr: String,
    /// Exit code from the command
    pub exit_code: i32,
    /// Built image ID (extracted from output)
    pub image_id: Option<String>,
}

impl BuildOutput {
    /// Check if the build executed successfully
    #[must_use]
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// Get combined output (stdout + stderr)
    #[must_use]
    pub fn combined_output(&self) -> String {
        if self.stderr.is_empty() {
            self.stdout.clone()
        } else if self.stdout.is_empty() {
            self.stderr.clone()
        } else {
            format!("{}\n{}", self.stdout, self.stderr)
        }
    }

    /// Check if stdout is empty (ignoring whitespace)
    #[must_use]
    pub fn stdout_is_empty(&self) -> bool {
        self.stdout.trim().is_empty()
    }

    /// Check if stderr is empty (ignoring whitespace)
    #[must_use]
    pub fn stderr_is_empty(&self) -> bool {
        self.stderr.trim().is_empty()
    }

    /// Extract image ID from build output (best effort)
    fn extract_image_id(output: &str) -> Option<String> {
        // Look for patterns like "Successfully built abc123def456" or "sha256:..."
        for line in output.lines() {
            if line.contains("Successfully built ") {
                if let Some(id) = line.split("Successfully built ").nth(1) {
                    return Some(id.trim().to_string());
                }
            }
            if line.starts_with("sha256:") {
                return Some(line.trim().to_string());
            }
        }
        None
    }
}

impl BuildCommand {
    /// Create a new build command for the specified context
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".");
    /// ```
    pub fn new(context: impl Into<String>) -> Self {
        Self {
            context: context.into(),
            executor: CommandExecutor::new(),
            add_hosts: Vec::new(),
            build_args: HashMap::new(),
            cache_from: Vec::new(),
            cgroup_parent: None,
            compress: false,
            cpu_period: None,
            cpu_quota: None,
            cpu_shares: None,
            cpuset_cpus: None,
            cpuset_mems: None,
            disable_content_trust: false,
            file: None,
            force_rm: false,
            iidfile: None,
            isolation: None,
            labels: HashMap::new(),
            memory: None,
            memory_swap: None,
            network: None,
            no_cache: false,
            platform: None,
            pull: false,
            quiet: false,
            rm: true, // Default is true
            security_opts: Vec::new(),
            shm_size: None,
            tags: Vec::new(),
            target: None,
            ulimits: Vec::new(),
            allow: Vec::new(),
            annotations: Vec::new(),
            attestations: Vec::new(),
            build_contexts: Vec::new(),
            builder: None,
            cache_to: Vec::new(),
            call: None,
            check: false,
            load: false,
            metadata_file: None,
            no_cache_filter: Vec::new(),
            progress: None,
            provenance: None,
            push: false,
            sbom: None,
            secrets: Vec::new(),
            ssh: Vec::new(),
        }
    }

    /// Add a custom host-to-IP mapping
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .add_host("myhost:192.168.1.100");
    /// ```
    #[must_use]
    pub fn add_host(mut self, host: impl Into<String>) -> Self {
        self.add_hosts.push(host.into());
        self
    }

    /// Set a build-time variable
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .build_arg("VERSION", "1.0.0")
    ///     .build_arg("DEBUG", "true");
    /// ```
    #[must_use]
    pub fn build_arg(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.build_args.insert(key.into(), value.into());
        self
    }

    /// Set multiple build-time variables
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    /// use std::collections::HashMap;
    ///
    /// let mut args = HashMap::new();
    /// args.insert("VERSION".to_string(), "1.0.0".to_string());
    /// args.insert("DEBUG".to_string(), "true".to_string());
    ///
    /// let build_cmd = BuildCommand::new(".").build_args_map(args);
    /// ```
    #[must_use]
    pub fn build_args_map(mut self, args: HashMap<String, String>) -> Self {
        self.build_args.extend(args);
        self
    }

    /// Add an image to consider as cache source
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .cache_from("myapp:cache");
    /// ```
    #[must_use]
    pub fn cache_from(mut self, image: impl Into<String>) -> Self {
        self.cache_from.push(image.into());
        self
    }

    /// Set the parent cgroup for RUN instructions
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .cgroup_parent("/docker");
    /// ```
    #[must_use]
    pub fn cgroup_parent(mut self, parent: impl Into<String>) -> Self {
        self.cgroup_parent = Some(parent.into());
        self
    }

    /// Compress the build context using gzip
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".").compress();
    /// ```
    #[must_use]
    pub fn compress(mut self) -> Self {
        self.compress = true;
        self
    }

    /// Set CPU period limit
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .cpu_period(100000);
    /// ```
    #[must_use]
    pub fn cpu_period(mut self, period: i64) -> Self {
        self.cpu_period = Some(period);
        self
    }

    /// Set CPU quota limit
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .cpu_quota(50000);
    /// ```
    #[must_use]
    pub fn cpu_quota(mut self, quota: i64) -> Self {
        self.cpu_quota = Some(quota);
        self
    }

    /// Set CPU shares (relative weight)
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .cpu_shares(512);
    /// ```
    #[must_use]
    pub fn cpu_shares(mut self, shares: i64) -> Self {
        self.cpu_shares = Some(shares);
        self
    }

    /// Set CPUs in which to allow execution
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .cpuset_cpus("0-3");
    /// ```
    #[must_use]
    pub fn cpuset_cpus(mut self, cpus: impl Into<String>) -> Self {
        self.cpuset_cpus = Some(cpus.into());
        self
    }

    /// Set MEMs in which to allow execution
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .cpuset_mems("0-1");
    /// ```
    #[must_use]
    pub fn cpuset_mems(mut self, mems: impl Into<String>) -> Self {
        self.cpuset_mems = Some(mems.into());
        self
    }

    /// Skip image verification
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .disable_content_trust();
    /// ```
    #[must_use]
    pub fn disable_content_trust(mut self) -> Self {
        self.disable_content_trust = true;
        self
    }

    /// Set the name/path of the Dockerfile
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .file("Dockerfile.prod");
    /// ```
    #[must_use]
    pub fn file(mut self, dockerfile: impl Into<PathBuf>) -> Self {
        self.file = Some(dockerfile.into());
        self
    }

    /// Always remove intermediate containers
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .force_rm();
    /// ```
    #[must_use]
    pub fn force_rm(mut self) -> Self {
        self.force_rm = true;
        self
    }

    /// Write the image ID to a file
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .iidfile("/tmp/image_id.txt");
    /// ```
    #[must_use]
    pub fn iidfile(mut self, file: impl Into<PathBuf>) -> Self {
        self.iidfile = Some(file.into());
        self
    }

    /// Set container isolation technology
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .isolation("hyperv");
    /// ```
    #[must_use]
    pub fn isolation(mut self, isolation: impl Into<String>) -> Self {
        self.isolation = Some(isolation.into());
        self
    }

    /// Set metadata label for the image
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .label("version", "1.0.0")
    ///     .label("maintainer", "team@example.com");
    /// ```
    #[must_use]
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Set multiple metadata labels
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    /// use std::collections::HashMap;
    ///
    /// let mut labels = HashMap::new();
    /// labels.insert("version".to_string(), "1.0.0".to_string());
    /// labels.insert("env".to_string(), "production".to_string());
    ///
    /// let build_cmd = BuildCommand::new(".").labels(labels);
    /// ```
    #[must_use]
    pub fn labels(mut self, labels: HashMap<String, String>) -> Self {
        self.labels.extend(labels);
        self
    }

    /// Set memory limit
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .memory("1g");
    /// ```
    #[must_use]
    pub fn memory(mut self, limit: impl Into<String>) -> Self {
        self.memory = Some(limit.into());
        self
    }

    /// Set memory + swap limit
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .memory_swap("2g");
    /// ```
    #[must_use]
    pub fn memory_swap(mut self, limit: impl Into<String>) -> Self {
        self.memory_swap = Some(limit.into());
        self
    }

    /// Set networking mode for RUN instructions
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .network("host");
    /// ```
    #[must_use]
    pub fn network(mut self, mode: impl Into<String>) -> Self {
        self.network = Some(mode.into());
        self
    }

    /// Do not use cache when building
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .no_cache();
    /// ```
    #[must_use]
    pub fn no_cache(mut self) -> Self {
        self.no_cache = true;
        self
    }

    /// Set platform for multi-platform builds
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .platform("linux/amd64");
    /// ```
    #[must_use]
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Always attempt to pull newer base images
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .pull();
    /// ```
    #[must_use]
    pub fn pull(mut self) -> Self {
        self.pull = true;
        self
    }

    /// Suppress build output and print image ID on success
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .quiet();
    /// ```
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// Remove intermediate containers after successful build (default: true)
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .no_rm(); // Don't remove intermediate containers
    /// ```
    #[must_use]
    pub fn no_rm(mut self) -> Self {
        self.rm = false;
        self
    }

    /// Add a security option
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .security_opt("seccomp=unconfined");
    /// ```
    #[must_use]
    pub fn security_opt(mut self, opt: impl Into<String>) -> Self {
        self.security_opts.push(opt.into());
        self
    }

    /// Set size of /dev/shm
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .shm_size("128m");
    /// ```
    #[must_use]
    pub fn shm_size(mut self, size: impl Into<String>) -> Self {
        self.shm_size = Some(size.into());
        self
    }

    /// Add a name and tag for the image
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .tag("myapp:latest")
    ///     .tag("myapp:1.0.0");
    /// ```
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set multiple tags for the image
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let tags = vec!["myapp:latest".to_string(), "myapp:1.0.0".to_string()];
    /// let build_cmd = BuildCommand::new(".").tags(tags);
    /// ```
    #[must_use]
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags.extend(tags);
        self
    }

    /// Set the target build stage
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .target("production");
    /// ```
    #[must_use]
    pub fn target(mut self, stage: impl Into<String>) -> Self {
        self.target = Some(stage.into());
        self
    }

    /// Add a ulimit option
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .ulimit("nofile=65536:65536");
    /// ```
    #[must_use]
    pub fn ulimit(mut self, limit: impl Into<String>) -> Self {
        self.ulimits.push(limit.into());
        self
    }

    /// Add an extra privileged entitlement
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .allow("network.host");
    /// ```
    #[must_use]
    pub fn allow(mut self, entitlement: impl Into<String>) -> Self {
        self.allow.push(entitlement.into());
        self
    }

    /// Add an annotation to the image
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .annotation("org.opencontainers.image.title=MyApp");
    /// ```
    #[must_use]
    pub fn annotation(mut self, annotation: impl Into<String>) -> Self {
        self.annotations.push(annotation.into());
        self
    }

    /// Add attestation parameters
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .attest("type=provenance,mode=max");
    /// ```
    #[must_use]
    pub fn attest(mut self, attestation: impl Into<String>) -> Self {
        self.attestations.push(attestation.into());
        self
    }

    /// Add additional build context
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .build_context("mycontext=../path");
    /// ```
    #[must_use]
    pub fn build_context(mut self, context: impl Into<String>) -> Self {
        self.build_contexts.push(context.into());
        self
    }

    /// Override the configured builder
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .builder("mybuilder");
    /// ```
    #[must_use]
    pub fn builder(mut self, builder: impl Into<String>) -> Self {
        self.builder = Some(builder.into());
        self
    }

    /// Add cache export destination
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .cache_to("type=registry,ref=myregistry/cache");
    /// ```
    #[must_use]
    pub fn cache_to(mut self, destination: impl Into<String>) -> Self {
        self.cache_to.push(destination.into());
        self
    }

    /// Set method for evaluating build
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .call("check");
    /// ```
    #[must_use]
    pub fn call(mut self, method: impl Into<String>) -> Self {
        self.call = Some(method.into());
        self
    }

    /// Enable check mode (shorthand for "--call=check")
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .check();
    /// ```
    #[must_use]
    pub fn check(mut self) -> Self {
        self.check = true;
        self
    }

    /// Enable load mode (shorthand for "--output=type=docker")
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .load();
    /// ```
    #[must_use]
    pub fn load(mut self) -> Self {
        self.load = true;
        self
    }

    /// Write build result metadata to file
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .metadata_file("/tmp/metadata.json");
    /// ```
    #[must_use]
    pub fn metadata_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.metadata_file = Some(file.into());
        self
    }

    /// Do not cache specified stage
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .no_cache_filter("build-stage");
    /// ```
    #[must_use]
    pub fn no_cache_filter(mut self, stage: impl Into<String>) -> Self {
        self.no_cache_filter.push(stage.into());
        self
    }

    /// Set type of progress output
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .progress("plain");
    /// ```
    #[must_use]
    pub fn progress(mut self, progress_type: impl Into<String>) -> Self {
        self.progress = Some(progress_type.into());
        self
    }

    /// Set provenance attestation (shorthand for "--attest=type=provenance")
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .provenance("mode=max");
    /// ```
    #[must_use]
    pub fn provenance(mut self, provenance: impl Into<String>) -> Self {
        self.provenance = Some(provenance.into());
        self
    }

    /// Enable push mode (shorthand for "--output=type=registry")
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .push();
    /// ```
    #[must_use]
    pub fn push(mut self) -> Self {
        self.push = true;
        self
    }

    /// Set SBOM attestation (shorthand for "--attest=type=sbom")
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .sbom("generator=image");
    /// ```
    #[must_use]
    pub fn sbom(mut self, sbom: impl Into<String>) -> Self {
        self.sbom = Some(sbom.into());
        self
    }

    /// Add secret to expose to the build
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .secret("id=mysecret,src=/local/secret");
    /// ```
    #[must_use]
    pub fn secret(mut self, secret: impl Into<String>) -> Self {
        self.secrets.push(secret.into());
        self
    }

    /// Add SSH agent socket or keys to expose
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::build::BuildCommand;
    ///
    /// let build_cmd = BuildCommand::new(".")
    ///     .ssh("default");
    /// ```
    #[must_use]
    pub fn ssh(mut self, ssh: impl Into<String>) -> Self {
        self.ssh.push(ssh.into());
        self
    }
}

impl Default for BuildCommand {
    fn default() -> Self {
        Self::new(".")
    }
}

impl BuildCommand {
    fn add_basic_args(&self, args: &mut Vec<String>) {
        // Add host mappings
        for host in &self.add_hosts {
            args.push("--add-host".to_string());
            args.push(host.clone());
        }

        // Add build arguments
        for (key, value) in &self.build_args {
            args.push("--build-arg".to_string());
            args.push(format!("{key}={value}"));
        }

        // Add cache sources
        for cache in &self.cache_from {
            args.push("--cache-from".to_string());
            args.push(cache.clone());
        }

        if let Some(ref dockerfile) = self.file {
            args.push("--file".to_string());
            args.push(dockerfile.to_string_lossy().to_string());
        }

        if self.no_cache {
            args.push("--no-cache".to_string());
        }

        if self.pull {
            args.push("--pull".to_string());
        }

        if self.quiet {
            args.push("--quiet".to_string());
        }

        // Add tags
        for tag in &self.tags {
            args.push("--tag".to_string());
            args.push(tag.clone());
        }

        if let Some(ref target) = self.target {
            args.push("--target".to_string());
            args.push(target.clone());
        }
    }

    fn add_resource_args(&self, args: &mut Vec<String>) {
        if let Some(period) = self.cpu_period {
            args.push("--cpu-period".to_string());
            args.push(period.to_string());
        }

        if let Some(quota) = self.cpu_quota {
            args.push("--cpu-quota".to_string());
            args.push(quota.to_string());
        }

        if let Some(shares) = self.cpu_shares {
            args.push("--cpu-shares".to_string());
            args.push(shares.to_string());
        }

        if let Some(ref cpus) = self.cpuset_cpus {
            args.push("--cpuset-cpus".to_string());
            args.push(cpus.clone());
        }

        if let Some(ref mems) = self.cpuset_mems {
            args.push("--cpuset-mems".to_string());
            args.push(mems.clone());
        }

        if let Some(ref memory) = self.memory {
            args.push("--memory".to_string());
            args.push(memory.clone());
        }

        if let Some(ref swap) = self.memory_swap {
            args.push("--memory-swap".to_string());
            args.push(swap.clone());
        }

        if let Some(ref size) = self.shm_size {
            args.push("--shm-size".to_string());
            args.push(size.clone());
        }
    }

    fn add_advanced_args(&self, args: &mut Vec<String>) {
        self.add_container_args(args);
        self.add_metadata_args(args);
        self.add_buildx_args(args);
    }

    fn add_container_args(&self, args: &mut Vec<String>) {
        if let Some(ref parent) = self.cgroup_parent {
            args.push("--cgroup-parent".to_string());
            args.push(parent.clone());
        }

        if self.compress {
            args.push("--compress".to_string());
        }

        if self.disable_content_trust {
            args.push("--disable-content-trust".to_string());
        }

        if self.force_rm {
            args.push("--force-rm".to_string());
        }

        if let Some(ref file) = self.iidfile {
            args.push("--iidfile".to_string());
            args.push(file.to_string_lossy().to_string());
        }

        if let Some(ref isolation) = self.isolation {
            args.push("--isolation".to_string());
            args.push(isolation.clone());
        }

        if let Some(ref network) = self.network {
            args.push("--network".to_string());
            args.push(network.clone());
        }

        if let Some(ref platform) = self.platform {
            args.push("--platform".to_string());
            args.push(platform.clone());
        }

        if !self.rm {
            args.push("--rm=false".to_string());
        }

        // Add security options
        for opt in &self.security_opts {
            args.push("--security-opt".to_string());
            args.push(opt.clone());
        }

        // Add ulimits
        for limit in &self.ulimits {
            args.push("--ulimit".to_string());
            args.push(limit.clone());
        }
    }

    fn add_metadata_args(&self, args: &mut Vec<String>) {
        // Add labels
        for (key, value) in &self.labels {
            args.push("--label".to_string());
            args.push(format!("{key}={value}"));
        }

        for annotation in &self.annotations {
            args.push("--annotation".to_string());
            args.push(annotation.clone());
        }

        if let Some(ref file) = self.metadata_file {
            args.push("--metadata-file".to_string());
            args.push(file.to_string_lossy().to_string());
        }
    }

    fn add_buildx_args(&self, args: &mut Vec<String>) {
        for allow in &self.allow {
            args.push("--allow".to_string());
            args.push(allow.clone());
        }

        for attest in &self.attestations {
            args.push("--attest".to_string());
            args.push(attest.clone());
        }

        for context in &self.build_contexts {
            args.push("--build-context".to_string());
            args.push(context.clone());
        }

        if let Some(ref builder) = self.builder {
            args.push("--builder".to_string());
            args.push(builder.clone());
        }

        for cache in &self.cache_to {
            args.push("--cache-to".to_string());
            args.push(cache.clone());
        }

        if let Some(ref call) = self.call {
            args.push("--call".to_string());
            args.push(call.clone());
        }

        if self.check {
            args.push("--check".to_string());
        }

        if self.load {
            args.push("--load".to_string());
        }

        for filter in &self.no_cache_filter {
            args.push("--no-cache-filter".to_string());
            args.push(filter.clone());
        }

        if let Some(ref progress) = self.progress {
            args.push("--progress".to_string());
            args.push(progress.clone());
        }

        if let Some(ref provenance) = self.provenance {
            args.push("--provenance".to_string());
            args.push(provenance.clone());
        }

        if self.push {
            args.push("--push".to_string());
        }

        if let Some(ref sbom) = self.sbom {
            args.push("--sbom".to_string());
            args.push(sbom.clone());
        }

        for secret in &self.secrets {
            args.push("--secret".to_string());
            args.push(secret.clone());
        }

        for ssh in &self.ssh {
            args.push("--ssh".to_string());
            args.push(ssh.clone());
        }
    }
}

#[async_trait]
impl DockerCommand for BuildCommand {
    type Output = BuildOutput;

    fn command_name(&self) -> &'static str {
        "build"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        self.add_basic_args(&mut args);
        self.add_resource_args(&mut args);
        self.add_advanced_args(&mut args);

        // Add any additional raw arguments
        args.extend(self.executor.raw_args.clone());

        // Add build context (must be last)
        args.push(self.context.clone());

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_args();
        let output = self
            .executor
            .execute_command(self.command_name(), args)
            .await?;

        // Extract image ID from output
        let image_id = if self.quiet {
            // In quiet mode, the output should be just the image ID
            Some(output.stdout.trim().to_string())
        } else {
            let combined = if output.stderr.is_empty() {
                output.stdout.clone()
            } else if output.stdout.is_empty() {
                output.stderr.clone()
            } else {
                format!("{}\n{}", output.stdout, output.stderr)
            };
            BuildOutput::extract_image_id(&combined)
        };

        Ok(BuildOutput {
            stdout: output.stdout,
            stderr: output.stderr,
            exit_code: output.exit_code,
            image_id,
        })
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
    fn test_build_command_basic() {
        let cmd = BuildCommand::new(".").tag("myapp:latest").no_cache().pull();

        let args = cmd.build_args();

        assert!(args.contains(&"--tag".to_string()));
        assert!(args.contains(&"myapp:latest".to_string()));
        assert!(args.contains(&"--no-cache".to_string()));
        assert!(args.contains(&"--pull".to_string()));
        assert!(args.contains(&".".to_string()));
    }

    #[test]
    fn test_build_command_with_dockerfile() {
        let cmd = BuildCommand::new("/path/to/context")
            .file("Dockerfile.prod")
            .tag("myapp:prod");

        let args = cmd.build_args();

        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"Dockerfile.prod".to_string()));
        assert!(args.contains(&"--tag".to_string()));
        assert!(args.contains(&"myapp:prod".to_string()));
        assert!(args.contains(&"/path/to/context".to_string()));
    }

    #[test]
    fn test_build_command_with_build_args() {
        let mut build_args = HashMap::new();
        build_args.insert("VERSION".to_string(), "1.0.0".to_string());
        build_args.insert("DEBUG".to_string(), "true".to_string());

        let cmd = BuildCommand::new(".")
            .build_args_map(build_args)
            .build_arg("EXTRA", "value");

        let args = cmd.build_args();

        assert!(args.contains(&"--build-arg".to_string()));
        assert!(args.contains(&"VERSION=1.0.0".to_string()));
        assert!(args.contains(&"DEBUG=true".to_string()));
        assert!(args.contains(&"EXTRA=value".to_string()));
    }

    #[test]
    fn test_build_command_with_labels() {
        let cmd = BuildCommand::new(".")
            .label("version", "1.0.0")
            .label("maintainer", "team@example.com");

        let args = cmd.build_args();

        assert!(args.contains(&"--label".to_string()));
        assert!(args.contains(&"version=1.0.0".to_string()));
        assert!(args.contains(&"maintainer=team@example.com".to_string()));
    }

    #[test]
    fn test_build_command_resource_limits() {
        let cmd = BuildCommand::new(".")
            .memory("1g")
            .cpu_shares(512)
            .cpuset_cpus("0-3");

        let args = cmd.build_args();

        assert!(args.contains(&"--memory".to_string()));
        assert!(args.contains(&"1g".to_string()));
        assert!(args.contains(&"--cpu-shares".to_string()));
        assert!(args.contains(&"512".to_string()));
        assert!(args.contains(&"--cpuset-cpus".to_string()));
        assert!(args.contains(&"0-3".to_string()));
    }

    #[test]
    fn test_build_command_advanced_options() {
        let cmd = BuildCommand::new(".")
            .platform("linux/amd64")
            .target("production")
            .network("host")
            .cache_from("myapp:cache")
            .security_opt("seccomp=unconfined");

        let args = cmd.build_args();

        assert!(args.contains(&"--platform".to_string()));
        assert!(args.contains(&"linux/amd64".to_string()));
        assert!(args.contains(&"--target".to_string()));
        assert!(args.contains(&"production".to_string()));
        assert!(args.contains(&"--network".to_string()));
        assert!(args.contains(&"host".to_string()));
        assert!(args.contains(&"--cache-from".to_string()));
        assert!(args.contains(&"myapp:cache".to_string()));
        assert!(args.contains(&"--security-opt".to_string()));
        assert!(args.contains(&"seccomp=unconfined".to_string()));
    }

    #[test]
    fn test_build_command_multiple_tags() {
        let tags = vec!["myapp:latest".to_string(), "myapp:1.0.0".to_string()];
        let cmd = BuildCommand::new(".").tags(tags);

        let args = cmd.build_args();

        let tag_count = args.iter().filter(|&arg| arg == "--tag").count();
        assert_eq!(tag_count, 2);
        assert!(args.contains(&"myapp:latest".to_string()));
        assert!(args.contains(&"myapp:1.0.0".to_string()));
    }

    #[test]
    fn test_build_command_quiet_mode() {
        let cmd = BuildCommand::new(".").quiet().tag("myapp:test");

        let args = cmd.build_args();

        assert!(args.contains(&"--quiet".to_string()));
        assert!(args.contains(&"--tag".to_string()));
        assert!(args.contains(&"myapp:test".to_string()));
    }

    #[test]
    fn test_build_command_no_rm() {
        let cmd = BuildCommand::new(".").no_rm();

        let args = cmd.build_args();

        assert!(args.contains(&"--rm=false".to_string()));
    }

    #[test]
    fn test_build_command_context_position() {
        let cmd = BuildCommand::new("/custom/context")
            .tag("test:latest")
            .no_cache();

        let args = cmd.build_args();

        // Context should be the last argument
        assert_eq!(args.last(), Some(&"/custom/context".to_string()));
    }

    #[test]
    fn test_build_output_helpers() {
        let output = BuildOutput {
            stdout: "Successfully built abc123def456".to_string(),
            stderr: String::new(),
            exit_code: 0,
            image_id: Some("abc123def456".to_string()),
        };

        assert!(output.success());
        assert!(!output.stdout_is_empty());
        assert!(output.stderr_is_empty());
        assert_eq!(output.image_id, Some("abc123def456".to_string()));
    }

    #[test]
    fn test_build_command_extensibility() {
        let mut cmd = BuildCommand::new(".");

        // Test extensibility methods
        cmd.flag("--some-flag");
        cmd.option("--some-option", "value");
        cmd.arg("extra-arg");

        let args = cmd.build_args();

        assert!(args.contains(&"--some-flag".to_string()));
        assert!(args.contains(&"--some-option".to_string()));
        assert!(args.contains(&"value".to_string()));
        assert!(args.contains(&"extra-arg".to_string()));
    }

    #[test]
    fn test_image_id_extraction() {
        let output1 = "Step 1/3 : FROM alpine\nSuccessfully built abc123def456\n";
        let id1 = BuildOutput::extract_image_id(output1);
        assert_eq!(id1, Some("abc123def456".to_string()));

        let output2 = "sha256:1234567890abcdef\n";
        let id2 = BuildOutput::extract_image_id(output2);
        assert_eq!(id2, Some("sha256:1234567890abcdef".to_string()));

        let output3 = "No image ID found here";
        let id3 = BuildOutput::extract_image_id(output3);
        assert_eq!(id3, None);
    }

    #[test]
    fn test_build_command_modern_buildx_options() {
        let cmd = BuildCommand::new(".")
            .allow("network.host")
            .annotation("org.opencontainers.image.title=MyApp")
            .attest("type=provenance,mode=max")
            .build_context("mycontext=../path")
            .builder("mybuilder")
            .cache_to("type=registry,ref=myregistry/cache")
            .call("check")
            .check()
            .load()
            .metadata_file("/tmp/metadata.json")
            .no_cache_filter("build-stage")
            .progress("plain")
            .provenance("mode=max")
            .push()
            .sbom("generator=image")
            .secret("id=mysecret,src=/local/secret")
            .ssh("default");

        let args = cmd.build_args();

        assert!(args.contains(&"--allow".to_string()));
        assert!(args.contains(&"network.host".to_string()));
        assert!(args.contains(&"--annotation".to_string()));
        assert!(args.contains(&"org.opencontainers.image.title=MyApp".to_string()));
        assert!(args.contains(&"--attest".to_string()));
        assert!(args.contains(&"type=provenance,mode=max".to_string()));
        assert!(args.contains(&"--build-context".to_string()));
        assert!(args.contains(&"mycontext=../path".to_string()));
        assert!(args.contains(&"--builder".to_string()));
        assert!(args.contains(&"mybuilder".to_string()));
        assert!(args.contains(&"--cache-to".to_string()));
        assert!(args.contains(&"type=registry,ref=myregistry/cache".to_string()));
        assert!(args.contains(&"--call".to_string()));
        assert!(args.contains(&"check".to_string()));
        assert!(args.contains(&"--check".to_string()));
        assert!(args.contains(&"--load".to_string()));
        assert!(args.contains(&"--metadata-file".to_string()));
        assert!(args.contains(&"/tmp/metadata.json".to_string()));
        assert!(args.contains(&"--no-cache-filter".to_string()));
        assert!(args.contains(&"build-stage".to_string()));
        assert!(args.contains(&"--progress".to_string()));
        assert!(args.contains(&"plain".to_string()));
        assert!(args.contains(&"--provenance".to_string()));
        assert!(args.contains(&"mode=max".to_string()));
        assert!(args.contains(&"--push".to_string()));
        assert!(args.contains(&"--sbom".to_string()));
        assert!(args.contains(&"generator=image".to_string()));
        assert!(args.contains(&"--secret".to_string()));
        assert!(args.contains(&"id=mysecret,src=/local/secret".to_string()));
        assert!(args.contains(&"--ssh".to_string()));
        assert!(args.contains(&"default".to_string()));
    }
}
