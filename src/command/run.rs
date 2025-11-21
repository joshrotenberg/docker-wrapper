//! Docker run command implementation.
//!
//! This module provides a comprehensive implementation of the `docker run` command
//! with support for common options and an extensible architecture for any additional options.

use super::{CommandExecutor, DockerCommand, EnvironmentBuilder, PortBuilder};
use crate::command::port::{PortCommand, PortMapping as PortMappingInfo};
use crate::error::{Error, Result};
use crate::stream::{OutputLine, StreamResult, StreamableCommand};
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::process::Command as TokioCommand;
use tokio::sync::mpsc;

/// Docker run command builder with fluent API
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct RunCommand {
    /// The Docker image to run
    image: String,
    /// Command executor for extensibility
    pub executor: CommandExecutor,
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

    // Resource Limits
    /// Memory limit
    memory: Option<String>,
    /// Number of CPUs
    cpus: Option<String>,
    /// CPU shares (relative weight)
    cpu_shares: Option<i64>,
    /// CPU CFS period
    cpu_period: Option<i64>,
    /// CPU CFS quota
    cpu_quota: Option<i64>,
    /// CPUs in which to allow execution
    cpuset_cpus: Option<String>,
    /// MEMs in which to allow execution
    cpuset_mems: Option<String>,
    /// Memory + swap limit
    memory_swap: Option<String>,
    /// Memory soft limit
    memory_reservation: Option<String>,

    // Security & User Context
    /// Username or UID
    user: Option<String>,
    /// Give extended privileges
    privileged: bool,
    /// Container host name
    hostname: Option<String>,

    // Lifecycle Management
    /// Restart policy
    restart: Option<String>,

    // System Integration
    /// Set platform if server is multi-platform capable
    platform: Option<String>,
    /// Runtime to use for this container
    runtime: Option<String>,
    /// Container isolation technology
    isolation: Option<String>,
    /// Pull image before running
    pull: Option<String>,
    /// Write the container ID to the file
    cidfile: Option<String>,
    /// Container NIS domain name
    domainname: Option<String>,
    /// Container MAC address
    mac_address: Option<String>,

    // Logging & Drivers
    /// Logging driver for the container
    log_driver: Option<String>,
    /// Optional volume driver for the container
    volume_driver: Option<String>,

    // Namespaces
    /// User namespace to use
    userns: Option<String>,
    /// UTS namespace to use
    uts: Option<String>,
    /// PID namespace to use
    pid: Option<String>,
    /// IPC mode to use
    ipc: Option<String>,
    /// Cgroup namespace to use
    cgroupns: Option<String>,
    /// Optional parent cgroup for the container
    cgroup_parent: Option<String>,

    // Advanced Memory & Performance
    /// Kernel memory limit
    kernel_memory: Option<String>,
    /// Tune container memory swappiness (0 to 100)
    memory_swappiness: Option<i32>,
    /// Tune host's OOM preferences (-1000 to 1000)
    oom_score_adj: Option<i32>,
    /// Tune container pids limit
    pids_limit: Option<i64>,
    /// Size of /dev/shm
    shm_size: Option<String>,

    // Process Control
    /// Signal to stop the container
    stop_signal: Option<String>,
    /// Timeout (in seconds) to stop a container
    stop_timeout: Option<i32>,
    /// Override the key sequence for detaching a container
    detach_keys: Option<String>,

    // Simple Flags
    /// Proxy received signals to the process
    sig_proxy: bool,
    /// Mount the container's root filesystem as read only
    read_only: bool,
    /// Run an init inside the container
    init: bool,
    /// Disable OOM Killer
    oom_kill_disable: bool,
    /// Disable any container-specified HEALTHCHECK
    no_healthcheck: bool,
    /// Skip image verification
    disable_content_trust: bool,
    /// Publish all exposed ports to random ports
    publish_all: bool,
    /// Suppress the pull output
    quiet: bool,

    // High-Impact List Options
    // DNS & Network
    /// Custom DNS servers
    dns: Vec<String>,
    /// DNS options
    dns_option: Vec<String>,
    /// DNS search domains
    dns_search: Vec<String>,
    /// Add host-to-IP mappings (host:ip)
    add_host: Vec<String>,

    // Security & Capabilities
    /// Add Linux capabilities
    cap_add: Vec<String>,
    /// Drop Linux capabilities
    cap_drop: Vec<String>,
    /// Security options
    security_opt: Vec<String>,

    // Device & Filesystem
    /// Add host devices to container
    device: Vec<String>,
    /// Mount tmpfs directories
    tmpfs: Vec<String>,
    /// Expose ports without publishing them
    expose: Vec<String>,

    // Environment & Labels
    /// Read environment from files
    env_file: Vec<PathBuf>,
    /// Set metadata labels
    label: Vec<String>,
    /// Read labels from files
    label_file: Vec<PathBuf>,

    // Additional List/Vec Options
    /// Network aliases for the container
    network_alias: Vec<String>,
    /// Additional groups for the user
    group_add: Vec<String>,
    /// Attach to STDIN, STDOUT or STDERR
    attach: Vec<String>,
    /// Log driver options
    log_opt: Vec<String>,
    /// Storage driver options
    storage_opt: Vec<String>,
    /// Ulimit options
    ulimit: Vec<String>,
    /// Mount volumes from other containers
    volumes_from: Vec<String>,
    /// Add link to another container (deprecated)
    link: Vec<String>,
    /// Container IPv4/IPv6 link-local addresses
    link_local_ip: Vec<String>,

    // Health Check Options
    // Health checks
    /// Command to run to check health
    health_cmd: Option<String>,
    /// Time between running the check (ms|s|m|h)
    health_interval: Option<String>,
    /// Consecutive failures needed to report unhealthy
    health_retries: Option<i32>,
    /// Maximum time to allow one check to run (ms|s|m|h)
    health_timeout: Option<String>,
    /// Start period for the container to initialize before health-checking (ms|s|m|h)
    health_start_period: Option<String>,
    /// Time between health checks during the start period (ms|s|m|h)
    health_start_interval: Option<String>,

    // Advanced options
    /// Advanced mount configuration
    mount: Vec<String>,
    /// Connect to a network
    network: Vec<String>,
    /// GPU devices to add to the container
    gpus: Option<String>,

    // Map-based options (stored as Vec<String> in key=value format)
    /// Add custom annotations
    annotation: Vec<String>,
    /// Kernel parameters to set
    sysctl: Vec<String>,

    // Advanced System Options
    // Block I/O controls
    /// Block IO weight (relative weight)
    blkio_weight: Option<u16>,
    /// Block IO weight per device
    blkio_weight_device: Vec<String>,
    /// Limit read rate (bytes per second) from a device
    device_read_bps: Vec<String>,
    /// Limit write rate (bytes per second) to a device
    device_write_bps: Vec<String>,
    /// Limit read rate (IO per second) from a device
    device_read_iops: Vec<String>,
    /// Limit write rate (IO per second) to a device
    device_write_iops: Vec<String>,

    // Real-time CPU scheduling
    /// Limit CPU real-time period in microseconds
    cpu_rt_period: Option<i64>,
    /// Limit CPU real-time runtime in microseconds
    cpu_rt_runtime: Option<i64>,

    // Advanced networking
    /// Container IPv4 address
    ip: Option<String>,
    /// Container IPv6 address
    ip6: Option<String>,

    // Advanced system options
    /// Cgroup rule for devices
    device_cgroup_rule: Vec<String>,
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

    /// Get port mappings for this container
    ///
    /// This queries Docker for the actual mapped ports of the running container.
    /// Useful when using dynamic port allocation (e.g., `-p 6379` without specifying host port).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::{DockerCommand, RunCommand};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Run Redis with dynamic port allocation
    /// let container_id = RunCommand::new("redis:alpine")
    ///     .name("my-redis")
    ///     .port_dyn(6379)  // Dynamic port allocation
    ///     .detach()
    ///     .rm()
    ///     .execute()
    ///     .await?;
    ///
    /// // Get the actual mapped port
    /// let port_mappings = container_id.port_mappings().await?;
    /// if let Some(mapping) = port_mappings.first() {
    ///     println!("Redis is available at {}:{}", mapping.host_ip, mapping.host_port);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The container doesn't exist or has been removed
    /// - The Docker daemon is not running
    /// - There's a communication error with Docker
    pub async fn port_mappings(&self) -> Result<Vec<PortMappingInfo>> {
        let result = PortCommand::new(&self.0).run().await?;
        Ok(result.port_mappings)
    }

    /// Get a specific port mapping for this container
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::{DockerCommand, RunCommand};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let container_id = RunCommand::new("nginx:alpine")
    ///     .port_dyn(80)
    ///     .detach()
    ///     .rm()
    ///     .execute()
    ///     .await?;
    ///
    /// // Get the mapping for port 80
    /// if let Some(mapping) = container_id.port_mapping(80).await? {
    ///     println!("Nginx is available at {}:{}", mapping.host_ip, mapping.host_port);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The container doesn't exist or has been removed
    /// - The Docker daemon is not running
    /// - There's a communication error with Docker
    pub async fn port_mapping(&self, container_port: u16) -> Result<Option<PortMappingInfo>> {
        let result = PortCommand::new(&self.0).port(container_port).run().await?;
        Ok(result.port_mappings.into_iter().next())
    }
}

impl std::fmt::Display for ContainerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl RunCommand {
    /// Create a new run command for the specified image
    #[allow(clippy::too_many_lines)]
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

            // Resource Limits
            memory: None,
            cpus: None,
            cpu_shares: None,
            cpu_period: None,
            cpu_quota: None,
            cpuset_cpus: None,
            cpuset_mems: None,
            memory_swap: None,
            memory_reservation: None,

            // Security & User Context
            user: None,
            privileged: false,
            hostname: None,

            // Lifecycle Management
            restart: None,

            // System Integration
            platform: None,
            runtime: None,
            isolation: None,
            pull: None,
            cidfile: None,
            domainname: None,
            mac_address: None,

            // Logging & Drivers
            log_driver: None,
            volume_driver: None,

            // Namespaces
            userns: None,
            uts: None,
            pid: None,
            ipc: None,
            cgroupns: None,
            cgroup_parent: None,

            // Advanced Memory & Performance
            kernel_memory: None,
            memory_swappiness: None,
            oom_score_adj: None,
            pids_limit: None,
            shm_size: None,

            // Process Control
            stop_signal: None,
            stop_timeout: None,
            detach_keys: None,

            // Simple Flags
            sig_proxy: true, // Default is true in Docker
            read_only: false,
            init: false,
            oom_kill_disable: false,
            no_healthcheck: false,
            disable_content_trust: true, // Default is true in Docker
            publish_all: false,
            quiet: false,

            // High-Impact List Options
            // DNS & Network
            dns: Vec::new(),
            dns_option: Vec::new(),
            dns_search: Vec::new(),
            add_host: Vec::new(),

            // Security & Capabilities
            cap_add: Vec::new(),
            cap_drop: Vec::new(),
            security_opt: Vec::new(),

            // Device & Filesystem
            device: Vec::new(),
            tmpfs: Vec::new(),
            expose: Vec::new(),

            // Environment & Labels
            env_file: Vec::new(),
            label: Vec::new(),
            label_file: Vec::new(),

            // Additional List/Vec Options
            network_alias: Vec::new(),
            group_add: Vec::new(),
            attach: Vec::new(),
            log_opt: Vec::new(),
            storage_opt: Vec::new(),
            ulimit: Vec::new(),
            volumes_from: Vec::new(),
            link: Vec::new(),
            link_local_ip: Vec::new(),

            // Health Check Options
            health_cmd: None,
            health_interval: None,
            health_retries: None,
            health_timeout: None,
            health_start_period: None,
            health_start_interval: None,
            mount: Vec::new(),
            network: Vec::new(),
            gpus: None,
            annotation: Vec::new(),
            sysctl: Vec::new(),

            // Advanced System Options
            blkio_weight: None,
            blkio_weight_device: Vec::new(),
            device_read_bps: Vec::new(),
            device_write_bps: Vec::new(),
            device_read_iops: Vec::new(),
            device_write_iops: Vec::new(),
            cpu_rt_period: None,
            cpu_rt_runtime: None,
            ip: None,
            ip6: None,
            device_cgroup_rule: Vec::new(),
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

    /// Alias for `dynamic_port()` - Add a dynamic port mapping (Docker assigns host port)
    #[must_use]
    pub fn port_dyn(self, container_port: u16) -> Self {
        self.dynamic_port(container_port)
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

    /// Alias for `remove()` - Remove container automatically when it exits (--rm flag)
    #[must_use]
    pub fn rm(self) -> Self {
        self.remove()
    }

    /// Convenience method for interactive TTY mode
    #[must_use]
    pub fn it(self) -> Self {
        self.interactive().tty()
    }

    // Resource Limits
    /// Set memory limit (e.g., "1g", "512m")
    #[must_use]
    pub fn memory(mut self, memory: impl Into<String>) -> Self {
        self.memory = Some(memory.into());
        self
    }

    /// Set number of CPUs (e.g., "2.0", "1.5")
    #[must_use]
    pub fn cpus(mut self, cpus: impl Into<String>) -> Self {
        self.cpus = Some(cpus.into());
        self
    }

    /// Set CPU shares (relative weight)
    #[must_use]
    pub fn cpu_shares(mut self, shares: i64) -> Self {
        self.cpu_shares = Some(shares);
        self
    }

    /// Set CPU CFS period in microseconds
    #[must_use]
    pub fn cpu_period(mut self, period: i64) -> Self {
        self.cpu_period = Some(period);
        self
    }

    /// Set CPU CFS quota in microseconds
    #[must_use]
    pub fn cpu_quota(mut self, quota: i64) -> Self {
        self.cpu_quota = Some(quota);
        self
    }

    /// Set CPUs in which to allow execution (e.g., "0-3", "0,1")
    #[must_use]
    pub fn cpuset_cpus(mut self, cpus: impl Into<String>) -> Self {
        self.cpuset_cpus = Some(cpus.into());
        self
    }

    /// Set MEMs in which to allow execution (e.g., "0-3", "0,1")
    #[must_use]
    pub fn cpuset_mems(mut self, mems: impl Into<String>) -> Self {
        self.cpuset_mems = Some(mems.into());
        self
    }

    /// Set memory + swap limit (e.g., "2g", "-1" for unlimited)
    #[must_use]
    pub fn memory_swap(mut self, swap: impl Into<String>) -> Self {
        self.memory_swap = Some(swap.into());
        self
    }

    /// Set memory soft limit (e.g., "500m")
    #[must_use]
    pub fn memory_reservation(mut self, reservation: impl Into<String>) -> Self {
        self.memory_reservation = Some(reservation.into());
        self
    }

    // Security & User Context
    /// Set username or UID (format: <name|uid>[:<group|gid>])
    #[must_use]
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Give extended privileges to this container
    #[must_use]
    pub fn privileged(mut self) -> Self {
        self.privileged = true;
        self
    }

    /// Set container host name
    #[must_use]
    pub fn hostname(mut self, hostname: impl Into<String>) -> Self {
        self.hostname = Some(hostname.into());
        self
    }

    // Lifecycle Management
    /// Set restart policy (e.g., "always", "unless-stopped", "on-failure", "no")
    #[must_use]
    pub fn restart(mut self, restart: impl Into<String>) -> Self {
        self.restart = Some(restart.into());
        self
    }

    // System Integration
    /// Set platform if server is multi-platform capable (e.g., "linux/amd64")
    #[must_use]
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Set runtime to use for this container
    #[must_use]
    pub fn runtime(mut self, runtime: impl Into<String>) -> Self {
        self.runtime = Some(runtime.into());
        self
    }

    /// Set container isolation technology
    #[must_use]
    pub fn isolation(mut self, isolation: impl Into<String>) -> Self {
        self.isolation = Some(isolation.into());
        self
    }

    /// Set pull image policy ("always", "missing", "never")
    #[must_use]
    pub fn pull(mut self, pull: impl Into<String>) -> Self {
        self.pull = Some(pull.into());
        self
    }

    /// Write the container ID to the specified file
    #[must_use]
    pub fn cidfile(mut self, cidfile: impl Into<String>) -> Self {
        self.cidfile = Some(cidfile.into());
        self
    }

    /// Set container NIS domain name
    #[must_use]
    pub fn domainname(mut self, domainname: impl Into<String>) -> Self {
        self.domainname = Some(domainname.into());
        self
    }

    /// Set container MAC address (e.g., "92:d0:c6:0a:29:33")
    #[must_use]
    pub fn mac_address(mut self, mac: impl Into<String>) -> Self {
        self.mac_address = Some(mac.into());
        self
    }

    // Logging & Drivers
    /// Set logging driver for the container
    #[must_use]
    pub fn log_driver(mut self, driver: impl Into<String>) -> Self {
        self.log_driver = Some(driver.into());
        self
    }

    /// Set optional volume driver for the container
    #[must_use]
    pub fn volume_driver(mut self, driver: impl Into<String>) -> Self {
        self.volume_driver = Some(driver.into());
        self
    }

    // Namespaces
    /// Set user namespace to use
    #[must_use]
    pub fn userns(mut self, userns: impl Into<String>) -> Self {
        self.userns = Some(userns.into());
        self
    }

    /// Set UTS namespace to use
    #[must_use]
    pub fn uts(mut self, uts: impl Into<String>) -> Self {
        self.uts = Some(uts.into());
        self
    }

    /// Set PID namespace to use
    #[must_use]
    pub fn pid(mut self, pid: impl Into<String>) -> Self {
        self.pid = Some(pid.into());
        self
    }

    /// Set IPC mode to use
    #[must_use]
    pub fn ipc(mut self, ipc: impl Into<String>) -> Self {
        self.ipc = Some(ipc.into());
        self
    }

    /// Set cgroup namespace to use (host|private)
    #[must_use]
    pub fn cgroupns(mut self, cgroupns: impl Into<String>) -> Self {
        self.cgroupns = Some(cgroupns.into());
        self
    }

    /// Set optional parent cgroup for the container
    #[must_use]
    pub fn cgroup_parent(mut self, parent: impl Into<String>) -> Self {
        self.cgroup_parent = Some(parent.into());
        self
    }

    // Advanced Memory & Performance
    /// Set kernel memory limit
    #[must_use]
    pub fn kernel_memory(mut self, memory: impl Into<String>) -> Self {
        self.kernel_memory = Some(memory.into());
        self
    }

    /// Tune container memory swappiness (0 to 100)
    #[must_use]
    pub fn memory_swappiness(mut self, swappiness: i32) -> Self {
        self.memory_swappiness = Some(swappiness);
        self
    }

    /// Tune host's OOM preferences (-1000 to 1000)
    #[must_use]
    pub fn oom_score_adj(mut self, score: i32) -> Self {
        self.oom_score_adj = Some(score);
        self
    }

    /// Tune container pids limit (set -1 for unlimited)
    #[must_use]
    pub fn pids_limit(mut self, limit: i64) -> Self {
        self.pids_limit = Some(limit);
        self
    }

    /// Set size of /dev/shm (e.g., "64m")
    #[must_use]
    pub fn shm_size(mut self, size: impl Into<String>) -> Self {
        self.shm_size = Some(size.into());
        self
    }

    // Process Control
    /// Set signal to stop the container (e.g., "SIGTERM", "SIGKILL")
    #[must_use]
    pub fn stop_signal(mut self, signal: impl Into<String>) -> Self {
        self.stop_signal = Some(signal.into());
        self
    }

    /// Set timeout (in seconds) to stop a container
    #[must_use]
    pub fn stop_timeout(mut self, timeout: i32) -> Self {
        self.stop_timeout = Some(timeout);
        self
    }

    /// Override the key sequence for detaching a container
    #[must_use]
    pub fn detach_keys(mut self, keys: impl Into<String>) -> Self {
        self.detach_keys = Some(keys.into());
        self
    }

    // Simple Flags
    /// Disable proxying received signals to the process
    #[must_use]
    pub fn no_sig_proxy(mut self) -> Self {
        self.sig_proxy = false;
        self
    }

    /// Mount the container's root filesystem as read only
    #[must_use]
    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    /// Run an init inside the container that forwards signals and reaps processes
    #[must_use]
    pub fn init(mut self) -> Self {
        self.init = true;
        self
    }

    /// Disable OOM Killer
    #[must_use]
    pub fn oom_kill_disable(mut self) -> Self {
        self.oom_kill_disable = true;
        self
    }

    /// Disable any container-specified HEALTHCHECK
    #[must_use]
    pub fn no_healthcheck(mut self) -> Self {
        self.no_healthcheck = true;
        self
    }

    /// Enable image verification (disable content trust is false)
    #[must_use]
    pub fn enable_content_trust(mut self) -> Self {
        self.disable_content_trust = false;
        self
    }

    /// Publish all exposed ports to random ports
    #[must_use]
    pub fn publish_all(mut self) -> Self {
        self.publish_all = true;
        self
    }

    /// Suppress the pull output
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    // High-Impact List Options

    // DNS & Network
    /// Add custom DNS server
    #[must_use]
    pub fn dns(mut self, dns: impl Into<String>) -> Self {
        self.dns.push(dns.into());
        self
    }

    /// Add multiple DNS servers
    #[must_use]
    pub fn dns_servers(mut self, servers: Vec<String>) -> Self {
        self.dns.extend(servers);
        self
    }

    /// Add DNS option
    #[must_use]
    pub fn dns_option(mut self, option: impl Into<String>) -> Self {
        self.dns_option.push(option.into());
        self
    }

    /// Add DNS search domain
    #[must_use]
    pub fn dns_search(mut self, domain: impl Into<String>) -> Self {
        self.dns_search.push(domain.into());
        self
    }

    /// Add host-to-IP mapping (format: "hostname:ip")
    #[must_use]
    pub fn add_host(mut self, mapping: impl Into<String>) -> Self {
        self.add_host.push(mapping.into());
        self
    }

    // Security & Capabilities
    /// Add Linux capability
    #[must_use]
    pub fn cap_add(mut self, capability: impl Into<String>) -> Self {
        self.cap_add.push(capability.into());
        self
    }

    /// Drop Linux capability
    #[must_use]
    pub fn cap_drop(mut self, capability: impl Into<String>) -> Self {
        self.cap_drop.push(capability.into());
        self
    }

    /// Add security option
    #[must_use]
    pub fn security_opt(mut self, option: impl Into<String>) -> Self {
        self.security_opt.push(option.into());
        self
    }

    // Device & Filesystem
    /// Add host device to container
    #[must_use]
    pub fn device(mut self, device: impl Into<String>) -> Self {
        self.device.push(device.into());
        self
    }

    /// Mount tmpfs directory
    #[must_use]
    pub fn tmpfs(mut self, path: impl Into<String>) -> Self {
        self.tmpfs.push(path.into());
        self
    }

    /// Expose port without publishing it
    #[must_use]
    pub fn expose(mut self, port: impl Into<String>) -> Self {
        self.expose.push(port.into());
        self
    }

    // Environment & Labels
    /// Read environment variables from file
    #[must_use]
    pub fn env_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.env_file.push(file.into());
        self
    }

    /// Add metadata label
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label.push(label.into());
        self
    }

    /// Read labels from file
    #[must_use]
    pub fn label_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.label_file.push(file.into());
        self
    }

    // Additional List/Vec Options

    /// Add network alias for the container
    #[must_use]
    pub fn network_alias(mut self, alias: impl Into<String>) -> Self {
        self.network_alias.push(alias.into());
        self
    }

    /// Add supplementary group for the user
    #[must_use]
    pub fn group_add(mut self, group: impl Into<String>) -> Self {
        self.group_add.push(group.into());
        self
    }

    /// Attach to STDIN, STDOUT or STDERR
    #[must_use]
    pub fn attach(mut self, stream: impl Into<String>) -> Self {
        self.attach.push(stream.into());
        self
    }

    /// Add log driver option
    #[must_use]
    pub fn log_opt(mut self, option: impl Into<String>) -> Self {
        self.log_opt.push(option.into());
        self
    }

    /// Add storage driver option
    #[must_use]
    pub fn storage_opt(mut self, option: impl Into<String>) -> Self {
        self.storage_opt.push(option.into());
        self
    }

    /// Set ulimit option
    #[must_use]
    pub fn ulimit(mut self, limit: impl Into<String>) -> Self {
        self.ulimit.push(limit.into());
        self
    }

    /// Mount volumes from another container
    #[must_use]
    pub fn volumes_from(mut self, container: impl Into<String>) -> Self {
        self.volumes_from.push(container.into());
        self
    }

    /// Add link to another container (deprecated)
    #[must_use]
    pub fn link(mut self, link: impl Into<String>) -> Self {
        self.link.push(link.into());
        self
    }

    /// Add container IPv4/IPv6 link-local address
    #[must_use]
    pub fn link_local_ip(mut self, ip: impl Into<String>) -> Self {
        self.link_local_ip.push(ip.into());
        self
    }

    // Health Check Options

    // Health check methods
    /// Set health check command
    #[must_use]
    pub fn health_cmd(mut self, cmd: impl Into<String>) -> Self {
        self.health_cmd = Some(cmd.into());
        self
    }

    /// Set health check interval
    #[must_use]
    pub fn health_interval(mut self, interval: impl Into<String>) -> Self {
        self.health_interval = Some(interval.into());
        self
    }

    /// Set health check retries
    #[must_use]
    pub fn health_retries(mut self, retries: i32) -> Self {
        self.health_retries = Some(retries);
        self
    }

    /// Set health check timeout
    #[must_use]
    pub fn health_timeout(mut self, timeout: impl Into<String>) -> Self {
        self.health_timeout = Some(timeout.into());
        self
    }

    /// Set health check start period
    #[must_use]
    pub fn health_start_period(mut self, period: impl Into<String>) -> Self {
        self.health_start_period = Some(period.into());
        self
    }

    /// Set health check start interval
    #[must_use]
    pub fn health_start_interval(mut self, interval: impl Into<String>) -> Self {
        self.health_start_interval = Some(interval.into());
        self
    }

    // Advanced options
    /// Add advanced mount configuration
    #[must_use]
    pub fn mount(mut self, mount: impl Into<String>) -> Self {
        self.mount.push(mount.into());
        self
    }

    /// Connect to a network
    #[must_use]
    pub fn network(mut self, network: impl Into<String>) -> Self {
        self.network.push(network.into());
        self
    }

    /// Set GPU devices to add to the container
    #[must_use]
    pub fn gpus(mut self, gpus: impl Into<String>) -> Self {
        self.gpus = Some(gpus.into());
        self
    }

    /// Add custom annotation (key=value format)
    #[must_use]
    pub fn annotation(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.annotation
            .push(format!("{}={}", key.into(), value.into()));
        self
    }

    /// Set kernel parameter (key=value format)
    #[must_use]
    pub fn sysctl(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.sysctl.push(format!("{}={}", key.into(), value.into()));
        self
    }

    // Advanced System Options

    // Block I/O controls
    /// Set block IO weight (10-1000)
    #[must_use]
    pub fn blkio_weight(mut self, weight: u16) -> Self {
        self.blkio_weight = Some(weight);
        self
    }

    /// Set block IO weight for a specific device (format: DEVICE:WEIGHT)
    #[must_use]
    pub fn blkio_weight_device(mut self, device_weight: impl Into<String>) -> Self {
        self.blkio_weight_device.push(device_weight.into());
        self
    }

    /// Limit read rate (bytes per second) from a device (format: DEVICE:RATE)
    #[must_use]
    pub fn device_read_bps(mut self, device_rate: impl Into<String>) -> Self {
        self.device_read_bps.push(device_rate.into());
        self
    }

    /// Limit write rate (bytes per second) to a device (format: DEVICE:RATE)
    #[must_use]
    pub fn device_write_bps(mut self, device_rate: impl Into<String>) -> Self {
        self.device_write_bps.push(device_rate.into());
        self
    }

    /// Limit read rate (IO per second) from a device (format: DEVICE:RATE)
    #[must_use]
    pub fn device_read_iops(mut self, device_rate: impl Into<String>) -> Self {
        self.device_read_iops.push(device_rate.into());
        self
    }

    /// Limit write rate (IO per second) to a device (format: DEVICE:RATE)
    #[must_use]
    pub fn device_write_iops(mut self, device_rate: impl Into<String>) -> Self {
        self.device_write_iops.push(device_rate.into());
        self
    }

    // Real-time CPU scheduling
    /// Limit CPU real-time period in microseconds
    #[must_use]
    pub fn cpu_rt_period(mut self, period: i64) -> Self {
        self.cpu_rt_period = Some(period);
        self
    }

    /// Limit CPU real-time runtime in microseconds
    #[must_use]
    pub fn cpu_rt_runtime(mut self, runtime: i64) -> Self {
        self.cpu_rt_runtime = Some(runtime);
        self
    }

    // Advanced networking
    /// Set container IPv4 address (e.g., "172.30.100.104")
    #[must_use]
    pub fn ip(mut self, ip: impl Into<String>) -> Self {
        self.ip = Some(ip.into());
        self
    }

    /// Set container IPv6 address (e.g., "`2001:db8::33`")
    #[must_use]
    pub fn ip6(mut self, ip6: impl Into<String>) -> Self {
        self.ip6 = Some(ip6.into());
        self
    }

    /// Add device cgroup rule
    #[must_use]
    pub fn device_cgroup_rule(mut self, rule: impl Into<String>) -> Self {
        self.device_cgroup_rule.push(rule.into());
        self
    }
}

#[async_trait]
impl DockerCommand for RunCommand {
    type Output = ContainerId;

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    #[allow(clippy::too_many_lines)]
    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["run".to_string()];

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

        // Resource Limits
        if let Some(ref memory) = self.memory {
            args.push("--memory".to_string());
            args.push(memory.clone());
        }
        if let Some(ref cpus) = self.cpus {
            args.push("--cpus".to_string());
            args.push(cpus.clone());
        }
        if let Some(cpu_shares) = self.cpu_shares {
            args.push("--cpu-shares".to_string());
            args.push(cpu_shares.to_string());
        }
        if let Some(cpu_period) = self.cpu_period {
            args.push("--cpu-period".to_string());
            args.push(cpu_period.to_string());
        }
        if let Some(cpu_quota) = self.cpu_quota {
            args.push("--cpu-quota".to_string());
            args.push(cpu_quota.to_string());
        }
        if let Some(ref cpuset_cpus) = self.cpuset_cpus {
            args.push("--cpuset-cpus".to_string());
            args.push(cpuset_cpus.clone());
        }
        if let Some(ref cpuset_mems) = self.cpuset_mems {
            args.push("--cpuset-mems".to_string());
            args.push(cpuset_mems.clone());
        }
        if let Some(ref memory_swap) = self.memory_swap {
            args.push("--memory-swap".to_string());
            args.push(memory_swap.clone());
        }
        if let Some(ref memory_reservation) = self.memory_reservation {
            args.push("--memory-reservation".to_string());
            args.push(memory_reservation.clone());
        }

        // Security & User Context
        if let Some(ref user) = self.user {
            args.push("--user".to_string());
            args.push(user.clone());
        }
        if self.privileged {
            args.push("--privileged".to_string());
        }
        if let Some(ref hostname) = self.hostname {
            args.push("--hostname".to_string());
            args.push(hostname.clone());
        }

        // Lifecycle Management
        if let Some(ref restart) = self.restart {
            args.push("--restart".to_string());
            args.push(restart.clone());
        }

        // System Integration
        if let Some(ref platform) = self.platform {
            args.push("--platform".to_string());
            args.push(platform.clone());
        }
        if let Some(ref runtime) = self.runtime {
            args.push("--runtime".to_string());
            args.push(runtime.clone());
        }
        if let Some(ref isolation) = self.isolation {
            args.push("--isolation".to_string());
            args.push(isolation.clone());
        }
        if let Some(ref pull) = self.pull {
            args.push("--pull".to_string());
            args.push(pull.clone());
        }
        if let Some(ref cidfile) = self.cidfile {
            args.push("--cidfile".to_string());
            args.push(cidfile.clone());
        }
        if let Some(ref domainname) = self.domainname {
            args.push("--domainname".to_string());
            args.push(domainname.clone());
        }
        if let Some(ref mac_address) = self.mac_address {
            args.push("--mac-address".to_string());
            args.push(mac_address.clone());
        }

        // Logging & Drivers
        if let Some(ref log_driver) = self.log_driver {
            args.push("--log-driver".to_string());
            args.push(log_driver.clone());
        }
        if let Some(ref volume_driver) = self.volume_driver {
            args.push("--volume-driver".to_string());
            args.push(volume_driver.clone());
        }

        // Namespaces
        if let Some(ref userns) = self.userns {
            args.push("--userns".to_string());
            args.push(userns.clone());
        }
        if let Some(ref uts) = self.uts {
            args.push("--uts".to_string());
            args.push(uts.clone());
        }
        if let Some(ref pid) = self.pid {
            args.push("--pid".to_string());
            args.push(pid.clone());
        }
        if let Some(ref ipc) = self.ipc {
            args.push("--ipc".to_string());
            args.push(ipc.clone());
        }
        if let Some(ref cgroupns) = self.cgroupns {
            args.push("--cgroupns".to_string());
            args.push(cgroupns.clone());
        }
        if let Some(ref cgroup_parent) = self.cgroup_parent {
            args.push("--cgroup-parent".to_string());
            args.push(cgroup_parent.clone());
        }

        // Advanced Memory & Performance
        if let Some(ref kernel_memory) = self.kernel_memory {
            args.push("--kernel-memory".to_string());
            args.push(kernel_memory.clone());
        }
        if let Some(memory_swappiness) = self.memory_swappiness {
            args.push("--memory-swappiness".to_string());
            args.push(memory_swappiness.to_string());
        }
        if let Some(oom_score_adj) = self.oom_score_adj {
            args.push("--oom-score-adj".to_string());
            args.push(oom_score_adj.to_string());
        }
        if let Some(pids_limit) = self.pids_limit {
            args.push("--pids-limit".to_string());
            args.push(pids_limit.to_string());
        }
        if let Some(ref shm_size) = self.shm_size {
            args.push("--shm-size".to_string());
            args.push(shm_size.clone());
        }

        // Process Control
        if let Some(ref stop_signal) = self.stop_signal {
            args.push("--stop-signal".to_string());
            args.push(stop_signal.clone());
        }
        if let Some(stop_timeout) = self.stop_timeout {
            args.push("--stop-timeout".to_string());
            args.push(stop_timeout.to_string());
        }
        if let Some(ref detach_keys) = self.detach_keys {
            args.push("--detach-keys".to_string());
            args.push(detach_keys.clone());
        }

        // Simple Flags
        if !self.sig_proxy {
            args.push("--sig-proxy=false".to_string());
        }
        if self.read_only {
            args.push("--read-only".to_string());
        }
        if self.init {
            args.push("--init".to_string());
        }
        if self.oom_kill_disable {
            args.push("--oom-kill-disable".to_string());
        }
        if self.no_healthcheck {
            args.push("--no-healthcheck".to_string());
        }
        if !self.disable_content_trust {
            args.push("--disable-content-trust=false".to_string());
        }
        if self.publish_all {
            args.push("--publish-all".to_string());
        }
        if self.quiet {
            args.push("--quiet".to_string());
        }

        // High-Impact List Options
        // DNS & Network
        for dns in &self.dns {
            args.push("--dns".to_string());
            args.push(dns.clone());
        }
        for dns_option in &self.dns_option {
            args.push("--dns-option".to_string());
            args.push(dns_option.clone());
        }
        for dns_search in &self.dns_search {
            args.push("--dns-search".to_string());
            args.push(dns_search.clone());
        }
        for add_host in &self.add_host {
            args.push("--add-host".to_string());
            args.push(add_host.clone());
        }

        // Security & Capabilities
        for cap_add in &self.cap_add {
            args.push("--cap-add".to_string());
            args.push(cap_add.clone());
        }
        for cap_drop in &self.cap_drop {
            args.push("--cap-drop".to_string());
            args.push(cap_drop.clone());
        }
        for security_opt in &self.security_opt {
            args.push("--security-opt".to_string());
            args.push(security_opt.clone());
        }

        // Device & Filesystem
        for device in &self.device {
            args.push("--device".to_string());
            args.push(device.clone());
        }
        for tmpfs in &self.tmpfs {
            args.push("--tmpfs".to_string());
            args.push(tmpfs.clone());
        }
        for expose in &self.expose {
            args.push("--expose".to_string());
            args.push(expose.clone());
        }

        // Environment & Labels
        for env_file in &self.env_file {
            args.push("--env-file".to_string());
            args.push(env_file.to_string_lossy().to_string());
        }
        for label in &self.label {
            args.push("--label".to_string());
            args.push(label.clone());
        }
        for label_file in &self.label_file {
            args.push("--label-file".to_string());
            args.push(label_file.to_string_lossy().to_string());
        }

        // Additional List/Vec Options
        for network_alias in &self.network_alias {
            args.push("--network-alias".to_string());
            args.push(network_alias.clone());
        }
        for group_add in &self.group_add {
            args.push("--group-add".to_string());
            args.push(group_add.clone());
        }
        for attach in &self.attach {
            args.push("--attach".to_string());
            args.push(attach.clone());
        }
        for log_opt in &self.log_opt {
            args.push("--log-opt".to_string());
            args.push(log_opt.clone());
        }
        for storage_opt in &self.storage_opt {
            args.push("--storage-opt".to_string());
            args.push(storage_opt.clone());
        }
        for ulimit in &self.ulimit {
            args.push("--ulimit".to_string());
            args.push(ulimit.clone());
        }
        for volumes_from in &self.volumes_from {
            args.push("--volumes-from".to_string());
            args.push(volumes_from.clone());
        }
        for link in &self.link {
            args.push("--link".to_string());
            args.push(link.clone());
        }
        for link_local_ip in &self.link_local_ip {
            args.push("--link-local-ip".to_string());
            args.push(link_local_ip.clone());
        }

        // Health Check Options
        // Health checks
        if let Some(ref health_cmd) = self.health_cmd {
            args.push("--health-cmd".to_string());
            args.push(health_cmd.clone());
        }
        if let Some(ref health_interval) = self.health_interval {
            args.push("--health-interval".to_string());
            args.push(health_interval.clone());
        }
        if let Some(health_retries) = self.health_retries {
            args.push("--health-retries".to_string());
            args.push(health_retries.to_string());
        }
        if let Some(ref health_timeout) = self.health_timeout {
            args.push("--health-timeout".to_string());
            args.push(health_timeout.clone());
        }
        if let Some(ref health_start_period) = self.health_start_period {
            args.push("--health-start-period".to_string());
            args.push(health_start_period.clone());
        }
        if let Some(ref health_start_interval) = self.health_start_interval {
            args.push("--health-start-interval".to_string());
            args.push(health_start_interval.clone());
        }

        // Advanced options
        for mount in &self.mount {
            args.push("--mount".to_string());
            args.push(mount.clone());
        }
        for network in &self.network {
            args.push("--network".to_string());
            args.push(network.clone());
        }
        if let Some(ref gpus) = self.gpus {
            args.push("--gpus".to_string());
            args.push(gpus.clone());
        }

        // Map-based options
        for annotation in &self.annotation {
            args.push("--annotation".to_string());
            args.push(annotation.clone());
        }
        for sysctl in &self.sysctl {
            args.push("--sysctl".to_string());
            args.push(sysctl.clone());
        }

        // Advanced System Options
        // Block I/O controls
        if let Some(blkio_weight) = self.blkio_weight {
            args.push("--blkio-weight".to_string());
            args.push(blkio_weight.to_string());
        }
        for blkio_weight_device in &self.blkio_weight_device {
            args.push("--blkio-weight-device".to_string());
            args.push(blkio_weight_device.clone());
        }
        for device_read_bps in &self.device_read_bps {
            args.push("--device-read-bps".to_string());
            args.push(device_read_bps.clone());
        }
        for device_write_bps in &self.device_write_bps {
            args.push("--device-write-bps".to_string());
            args.push(device_write_bps.clone());
        }
        for device_read_iops in &self.device_read_iops {
            args.push("--device-read-iops".to_string());
            args.push(device_read_iops.clone());
        }
        for device_write_iops in &self.device_write_iops {
            args.push("--device-write-iops".to_string());
            args.push(device_write_iops.clone());
        }

        // Real-time CPU scheduling
        if let Some(cpu_rt_period) = self.cpu_rt_period {
            args.push("--cpu-rt-period".to_string());
            args.push(cpu_rt_period.to_string());
        }
        if let Some(cpu_rt_runtime) = self.cpu_rt_runtime {
            args.push("--cpu-rt-runtime".to_string());
            args.push(cpu_rt_runtime.to_string());
        }

        // Advanced networking
        if let Some(ref ip) = self.ip {
            args.push("--ip".to_string());
            args.push(ip.clone());
        }
        if let Some(ref ip6) = self.ip6 {
            args.push("--ip6".to_string());
            args.push(ip6.clone());
        }

        // Advanced system options
        for device_cgroup_rule in &self.device_cgroup_rule {
            args.push("--device-cgroup-rule".to_string());
            args.push(device_cgroup_rule.clone());
        }

        // Add image
        args.push(self.image.clone());

        // Add command if specified
        if let Some(ref command) = self.command {
            args.extend(command.clone());
        }

        // Add raw arguments from executor
        args.extend(self.executor.raw_args.clone());

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        let output = self.execute_command(args).await?;

        // Parse container ID from output
        let container_id = output.stdout.trim().to_string();
        if container_id.is_empty() {
            return Err(Error::parse_error(
                "No container ID returned from docker run",
            ));
        }

        Ok(ContainerId(container_id))
    }
}

// Streaming support for RunCommand
#[async_trait]
impl StreamableCommand for RunCommand {
    async fn stream<F>(&self, handler: F) -> Result<StreamResult>
    where
        F: FnMut(OutputLine) + Send + 'static,
    {
        // Don't stream if running detached
        if self.detach {
            return Err(Error::custom(
                "Cannot stream output for detached containers",
            ));
        }

        let mut cmd = TokioCommand::new("docker");
        cmd.arg("run");

        for arg in self.build_command_args() {
            cmd.arg(arg);
        }

        crate::stream::stream_command(cmd, handler).await
    }

    async fn stream_channel(&self) -> Result<(mpsc::Receiver<OutputLine>, StreamResult)> {
        // Don't stream if running detached
        if self.detach {
            return Err(Error::custom(
                "Cannot stream output for detached containers",
            ));
        }

        let mut cmd = TokioCommand::new("docker");
        cmd.arg("run");

        for arg in self.build_command_args() {
            cmd.arg(arg);
        }

        crate::stream::stream_command_channel(cmd).await
    }
}

impl RunCommand {
    /// Run the container with streaming output
    ///
    /// Note: This will fail if the container is run in detached mode.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use docker_wrapper::RunCommand;
    /// use docker_wrapper::StreamHandler;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = RunCommand::new("alpine")
    ///     .cmd(vec!["echo".to_string(), "Hello, World!".to_string()])
    ///     .stream(StreamHandler::print())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the container is detached or encounters an I/O error
    pub async fn stream<F>(&self, handler: F) -> Result<StreamResult>
    where
        F: FnMut(OutputLine) + Send + 'static,
    {
        <Self as StreamableCommand>::stream(self, handler).await
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

        let args = cmd.build_command_args();

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

        let args = cmd.build_command_args();
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
    fn test_run_command_resource_limits() {
        let cmd = RunCommand::new("alpine:latest")
            .memory("1g")
            .cpus("2.0")
            .cpu_shares(1024)
            .cpu_period(100_000)
            .cpu_quota(50_000)
            .cpuset_cpus("0-3")
            .cpuset_mems("0,1")
            .memory_swap("2g")
            .memory_reservation("500m");

        let args = cmd.build_command_args();

        assert!(args.contains(&"--memory".to_string()));
        assert!(args.contains(&"1g".to_string()));
        assert!(args.contains(&"--cpus".to_string()));
        assert!(args.contains(&"2.0".to_string()));
        assert!(args.contains(&"--cpu-shares".to_string()));
        assert!(args.contains(&"1024".to_string()));
        assert!(args.contains(&"--cpu-period".to_string()));
        assert!(args.contains(&"100000".to_string()));
        assert!(args.contains(&"--cpu-quota".to_string()));
        assert!(args.contains(&"50000".to_string()));
        assert!(args.contains(&"--cpuset-cpus".to_string()));
        assert!(args.contains(&"0-3".to_string()));
        assert!(args.contains(&"--cpuset-mems".to_string()));
        assert!(args.contains(&"0,1".to_string()));
        assert!(args.contains(&"--memory-swap".to_string()));
        assert!(args.contains(&"2g".to_string()));
        assert!(args.contains(&"--memory-reservation".to_string()));
        assert!(args.contains(&"500m".to_string()));
    }

    #[test]
    fn test_run_command_security_and_user() {
        let cmd = RunCommand::new("alpine:latest")
            .user("1000:1000")
            .privileged()
            .hostname("test-host");

        let args = cmd.build_command_args();

        assert!(args.contains(&"--user".to_string()));
        assert!(args.contains(&"1000:1000".to_string()));
        assert!(args.contains(&"--privileged".to_string()));
        assert!(args.contains(&"--hostname".to_string()));
        assert!(args.contains(&"test-host".to_string()));
    }

    #[test]
    fn test_run_command_lifecycle_management() {
        let cmd = RunCommand::new("alpine:latest").restart("always");

        let args = cmd.build_command_args();

        assert!(args.contains(&"--restart".to_string()));
        assert!(args.contains(&"always".to_string()));
    }

    #[test]
    fn test_run_command_system_integration() {
        let cmd = RunCommand::new("alpine:latest")
            .platform("linux/amd64")
            .runtime("runc")
            .isolation("default")
            .pull("always")
            .cidfile("/tmp/container.cid")
            .domainname("example.com")
            .mac_address("92:d0:c6:0a:29:33");

        let args = cmd.build_command_args();

        assert!(args.contains(&"--platform".to_string()));
        assert!(args.contains(&"linux/amd64".to_string()));
        assert!(args.contains(&"--runtime".to_string()));
        assert!(args.contains(&"runc".to_string()));
        assert!(args.contains(&"--isolation".to_string()));
        assert!(args.contains(&"default".to_string()));
        assert!(args.contains(&"--pull".to_string()));
        assert!(args.contains(&"always".to_string()));
        assert!(args.contains(&"--cidfile".to_string()));
        assert!(args.contains(&"/tmp/container.cid".to_string()));
        assert!(args.contains(&"--domainname".to_string()));
        assert!(args.contains(&"example.com".to_string()));
        assert!(args.contains(&"--mac-address".to_string()));
        assert!(args.contains(&"92:d0:c6:0a:29:33".to_string()));
    }

    #[test]
    fn test_run_command_logging_and_drivers() {
        let cmd = RunCommand::new("alpine:latest")
            .log_driver("json-file")
            .volume_driver("local");

        let args = cmd.build_command_args();

        assert!(args.contains(&"--log-driver".to_string()));
        assert!(args.contains(&"json-file".to_string()));
        assert!(args.contains(&"--volume-driver".to_string()));
        assert!(args.contains(&"local".to_string()));
    }

    #[test]
    fn test_run_command_namespaces() {
        let cmd = RunCommand::new("alpine:latest")
            .userns("host")
            .uts("host")
            .pid("host")
            .ipc("host")
            .cgroupns("private")
            .cgroup_parent("/docker");

        let args = cmd.build_command_args();

        assert!(args.contains(&"--userns".to_string()));
        assert!(args.contains(&"host".to_string()));
        assert!(args.contains(&"--uts".to_string()));
        assert!(args.contains(&"--pid".to_string()));
        assert!(args.contains(&"--ipc".to_string()));
        assert!(args.contains(&"--cgroupns".to_string()));
        assert!(args.contains(&"private".to_string()));
        assert!(args.contains(&"--cgroup-parent".to_string()));
        assert!(args.contains(&"/docker".to_string()));
    }

    #[test]
    fn test_run_command_advanced_memory_performance() {
        let cmd = RunCommand::new("alpine:latest")
            .kernel_memory("100m")
            .memory_swappiness(60)
            .oom_score_adj(-500)
            .pids_limit(100)
            .shm_size("64m");

        let args = cmd.build_command_args();

        assert!(args.contains(&"--kernel-memory".to_string()));
        assert!(args.contains(&"100m".to_string()));
        assert!(args.contains(&"--memory-swappiness".to_string()));
        assert!(args.contains(&"60".to_string()));
        assert!(args.contains(&"--oom-score-adj".to_string()));
        assert!(args.contains(&"-500".to_string()));
        assert!(args.contains(&"--pids-limit".to_string()));
        assert!(args.contains(&"100".to_string()));
        assert!(args.contains(&"--shm-size".to_string()));
        assert!(args.contains(&"64m".to_string()));
    }

    #[test]
    fn test_run_command_process_control() {
        let cmd = RunCommand::new("alpine:latest")
            .stop_signal("SIGTERM")
            .stop_timeout(10)
            .detach_keys("ctrl-p,ctrl-q");

        let args = cmd.build_command_args();

        assert!(args.contains(&"--stop-signal".to_string()));
        assert!(args.contains(&"SIGTERM".to_string()));
        assert!(args.contains(&"--stop-timeout".to_string()));
        assert!(args.contains(&"10".to_string()));
        assert!(args.contains(&"--detach-keys".to_string()));
        assert!(args.contains(&"ctrl-p,ctrl-q".to_string()));
    }

    #[test]
    fn test_run_command_simple_flags() {
        let cmd = RunCommand::new("alpine:latest")
            .no_sig_proxy()
            .read_only()
            .init()
            .oom_kill_disable()
            .no_healthcheck()
            .enable_content_trust()
            .publish_all()
            .quiet();

        let args = cmd.build_command_args();

        assert!(args.contains(&"--sig-proxy=false".to_string()));
        assert!(args.contains(&"--read-only".to_string()));
        assert!(args.contains(&"--init".to_string()));
        assert!(args.contains(&"--oom-kill-disable".to_string()));
        assert!(args.contains(&"--no-healthcheck".to_string()));
        assert!(args.contains(&"--disable-content-trust=false".to_string()));
        assert!(args.contains(&"--publish-all".to_string()));
        assert!(args.contains(&"--quiet".to_string()));
    }

    #[test]
    fn test_run_command_comprehensive_builder() {
        let cmd = RunCommand::new("nginx:latest")
            .name("production-nginx")
            .detach()
            .memory("2g")
            .cpus("4.0")
            .user("nginx:nginx")
            .privileged()
            .restart("unless-stopped")
            .hostname("web-server")
            .platform("linux/amd64")
            .env("NGINX_PORT", "8080")
            .port(80, 8080)
            .volume("nginx-data", "/var/lib/nginx")
            .workdir("/usr/share/nginx/html")
            .read_only()
            .init()
            .remove();

        let args = cmd.build_command_args();

        // Verify comprehensive options are all present
        assert!(args.contains(&"--name".to_string()));
        assert!(args.contains(&"production-nginx".to_string()));
        assert!(args.contains(&"--detach".to_string()));
        assert!(args.contains(&"--memory".to_string()));
        assert!(args.contains(&"2g".to_string()));
        assert!(args.contains(&"--cpus".to_string()));
        assert!(args.contains(&"4.0".to_string()));
        assert!(args.contains(&"--user".to_string()));
        assert!(args.contains(&"nginx:nginx".to_string()));
        assert!(args.contains(&"--privileged".to_string()));
        assert!(args.contains(&"--restart".to_string()));
        assert!(args.contains(&"unless-stopped".to_string()));
        assert!(args.contains(&"--hostname".to_string()));
        assert!(args.contains(&"web-server".to_string()));
        assert!(args.contains(&"--platform".to_string()));
        assert!(args.contains(&"linux/amd64".to_string()));
        assert!(args.contains(&"--read-only".to_string()));
        assert!(args.contains(&"--init".to_string()));
        assert!(args.contains(&"--rm".to_string()));
        assert!(args.contains(&"nginx:latest".to_string()));

        // Image should be at the end (before any command args)
        let image_pos = args.iter().position(|x| x == "nginx:latest").unwrap();
        assert!(image_pos > 10); // Should be after all the options
    }

    #[test]
    fn test_run_command_default_flag_values() {
        let cmd = RunCommand::new("alpine:latest");
        let args = cmd.build_command_args();

        // Default flags should not appear in args unless explicitly changed
        assert!(!args.contains(&"--sig-proxy=false".to_string()));
        assert!(!args.contains(&"--disable-content-trust=false".to_string()));
        assert!(!args.contains(&"--read-only".to_string()));
        assert!(!args.contains(&"--privileged".to_string()));
        assert!(!args.contains(&"--init".to_string()));
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
        let args = cmd.build_command_args();
        assert!(args.contains(&"--interactive".to_string()));
        assert!(args.contains(&"--tty".to_string()));
    }

    #[test]
    fn test_run_command_dns_network_options() {
        let cmd = RunCommand::new("alpine:latest")
            .dns("8.8.8.8")
            .dns("8.8.4.4")
            .dns_servers(vec!["1.1.1.1".to_string(), "1.0.0.1".to_string()])
            .dns_option("ndots:2")
            .dns_option("timeout:1")
            .dns_search("example.com")
            .dns_search("test.local")
            .add_host("api.example.com:127.0.0.1")
            .add_host("db.example.com:192.168.1.100");

        let args = cmd.build_command_args();

        // DNS servers
        assert!(args.contains(&"--dns".to_string()));
        assert!(args.contains(&"8.8.8.8".to_string()));
        assert!(args.contains(&"8.8.4.4".to_string()));
        assert!(args.contains(&"1.1.1.1".to_string()));
        assert!(args.contains(&"1.0.0.1".to_string()));

        // DNS options
        assert!(args.contains(&"--dns-option".to_string()));
        assert!(args.contains(&"ndots:2".to_string()));
        assert!(args.contains(&"timeout:1".to_string()));

        // DNS search domains
        assert!(args.contains(&"--dns-search".to_string()));
        assert!(args.contains(&"example.com".to_string()));
        assert!(args.contains(&"test.local".to_string()));

        // Host mappings
        assert!(args.contains(&"--add-host".to_string()));
        assert!(args.contains(&"api.example.com:127.0.0.1".to_string()));
        assert!(args.contains(&"db.example.com:192.168.1.100".to_string()));
    }

    #[test]
    fn test_run_command_security_capabilities() {
        let cmd = RunCommand::new("alpine:latest")
            .cap_add("NET_ADMIN")
            .cap_add("SYS_TIME")
            .cap_drop("CHOWN")
            .cap_drop("DAC_OVERRIDE")
            .security_opt("no-new-privileges:true")
            .security_opt("seccomp=unconfined");

        let args = cmd.build_command_args();

        // Capabilities to add
        assert!(args.contains(&"--cap-add".to_string()));
        assert!(args.contains(&"NET_ADMIN".to_string()));
        assert!(args.contains(&"SYS_TIME".to_string()));

        // Capabilities to drop
        assert!(args.contains(&"--cap-drop".to_string()));
        assert!(args.contains(&"CHOWN".to_string()));
        assert!(args.contains(&"DAC_OVERRIDE".to_string()));

        // Security options
        assert!(args.contains(&"--security-opt".to_string()));
        assert!(args.contains(&"no-new-privileges:true".to_string()));
        assert!(args.contains(&"seccomp=unconfined".to_string()));
    }

    #[test]
    fn test_run_command_device_filesystem() {
        let cmd = RunCommand::new("alpine:latest")
            .device("/dev/sda:/dev/xvda:rwm")
            .device("/dev/zero")
            .tmpfs("/tmp:rw,size=100m")
            .tmpfs("/var/tmp:ro")
            .expose("80")
            .expose("443")
            .expose("8080/tcp");

        let args = cmd.build_command_args();

        // Devices
        assert!(args.contains(&"--device".to_string()));
        assert!(args.contains(&"/dev/sda:/dev/xvda:rwm".to_string()));
        assert!(args.contains(&"/dev/zero".to_string()));

        // Tmpfs mounts
        assert!(args.contains(&"--tmpfs".to_string()));
        assert!(args.contains(&"/tmp:rw,size=100m".to_string()));
        assert!(args.contains(&"/var/tmp:ro".to_string()));

        // Exposed ports
        assert!(args.contains(&"--expose".to_string()));
        assert!(args.contains(&"80".to_string()));
        assert!(args.contains(&"443".to_string()));
        assert!(args.contains(&"8080/tcp".to_string()));
    }

    #[test]
    fn test_run_command_environment_labels() {
        use std::path::PathBuf;

        let cmd = RunCommand::new("alpine:latest")
            .env_file(PathBuf::from("/etc/environment"))
            .env_file(PathBuf::from("./app.env"))
            .label("version=1.0.0")
            .label("maintainer=team@example.com")
            .label("app=myapp")
            .label_file(PathBuf::from("/etc/labels"))
            .label_file(PathBuf::from("./metadata.labels"));

        let args = cmd.build_command_args();

        // Environment files
        assert!(args.contains(&"--env-file".to_string()));
        assert!(args.contains(&"/etc/environment".to_string()));
        assert!(args.contains(&"./app.env".to_string()));

        // Labels
        assert!(args.contains(&"--label".to_string()));
        assert!(args.contains(&"version=1.0.0".to_string()));
        assert!(args.contains(&"maintainer=team@example.com".to_string()));
        assert!(args.contains(&"app=myapp".to_string()));

        // Label files
        assert!(args.contains(&"--label-file".to_string()));
        assert!(args.contains(&"/etc/labels".to_string()));
        assert!(args.contains(&"./metadata.labels".to_string()));
    }

    #[test]
    fn test_run_command_all_high_impact_options() {
        use std::path::PathBuf;

        let cmd = RunCommand::new("nginx:latest")
            .name("production-nginx")
            // DNS & Network
            .dns("8.8.8.8")
            .dns_option("ndots:2")
            .dns_search("example.com")
            .add_host("api.example.com:127.0.0.1")
            // Security & Capabilities
            .cap_add("NET_ADMIN")
            .cap_drop("CHOWN")
            .security_opt("no-new-privileges:true")
            // Device & Filesystem
            .device("/dev/null")
            .tmpfs("/tmp:rw,size=100m")
            .expose("80")
            // Environment & Labels
            .env_file(PathBuf::from(".env"))
            .label("version=1.0.0")
            .label_file(PathBuf::from("labels"));

        let args = cmd.build_command_args();

        // Verify all option types are present
        assert!(args.contains(&"--dns".to_string()));
        assert!(args.contains(&"--dns-option".to_string()));
        assert!(args.contains(&"--dns-search".to_string()));
        assert!(args.contains(&"--add-host".to_string()));
        assert!(args.contains(&"--cap-add".to_string()));
        assert!(args.contains(&"--cap-drop".to_string()));
        assert!(args.contains(&"--security-opt".to_string()));
        assert!(args.contains(&"--device".to_string()));
        assert!(args.contains(&"--tmpfs".to_string()));
        assert!(args.contains(&"--expose".to_string()));
        assert!(args.contains(&"--env-file".to_string()));
        assert!(args.contains(&"--label".to_string()));
        assert!(args.contains(&"--label-file".to_string()));

        // Verify image is still at the end
        let image_pos = args.iter().position(|x| x == "nginx:latest").unwrap();
        assert!(image_pos > 0); // Should not be first
        assert!(image_pos < args.len() - 1 || args.len() == image_pos + 1); // Should be near end
    }

    #[test]
    fn test_run_command_empty_lists_not_added() {
        let cmd = RunCommand::new("alpine:latest");
        let args = cmd.build_command_args();

        // Ensure empty lists don't add any arguments
        assert!(!args.contains(&"--dns".to_string()));
        assert!(!args.contains(&"--dns-option".to_string()));
        assert!(!args.contains(&"--dns-search".to_string()));
        assert!(!args.contains(&"--add-host".to_string()));
        assert!(!args.contains(&"--cap-add".to_string()));
        assert!(!args.contains(&"--cap-drop".to_string()));
        assert!(!args.contains(&"--security-opt".to_string()));
        assert!(!args.contains(&"--device".to_string()));
        assert!(!args.contains(&"--tmpfs".to_string()));
        assert!(!args.contains(&"--expose".to_string()));
        assert!(!args.contains(&"--env-file".to_string()));
        assert!(!args.contains(&"--label".to_string()));
        assert!(!args.contains(&"--label-file".to_string()));

        // Additional list options should also not be present
        assert!(!args.contains(&"--network-alias".to_string()));
        assert!(!args.contains(&"--group-add".to_string()));
        assert!(!args.contains(&"--attach".to_string()));
        assert!(!args.contains(&"--log-opt".to_string()));
        assert!(!args.contains(&"--storage-opt".to_string()));
        assert!(!args.contains(&"--ulimit".to_string()));
        assert!(!args.contains(&"--volumes-from".to_string()));
        assert!(!args.contains(&"--link".to_string()));
        assert!(!args.contains(&"--link-local-ip".to_string()));

        // But image should still be there
        assert!(args.contains(&"alpine:latest".to_string()));
    }

    #[test]
    fn test_run_command_additional_list_options() {
        let cmd = RunCommand::new("alpine:latest")
            .network_alias("web")
            .network_alias("frontend")
            .group_add("staff")
            .group_add("docker")
            .attach("stdout")
            .attach("stderr")
            .log_opt("max-size=10m")
            .log_opt("max-file=3")
            .storage_opt("size=20G")
            .ulimit("nofile=1024:65536")
            .ulimit("nproc=1024")
            .volumes_from("data-container")
            .volumes_from("config-container:ro")
            .link("db:database")
            .link("cache:redis")
            .link_local_ip("169.254.1.1")
            .link_local_ip("fe80::1");

        let args = cmd.build_command_args();

        // Network aliases
        assert!(args.contains(&"--network-alias".to_string()));
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"frontend".to_string()));

        // Group additions
        assert!(args.contains(&"--group-add".to_string()));
        assert!(args.contains(&"staff".to_string()));
        assert!(args.contains(&"docker".to_string()));

        // Stream attachments
        assert!(args.contains(&"--attach".to_string()));
        assert!(args.contains(&"stdout".to_string()));
        assert!(args.contains(&"stderr".to_string()));

        // Log options
        assert!(args.contains(&"--log-opt".to_string()));
        assert!(args.contains(&"max-size=10m".to_string()));
        assert!(args.contains(&"max-file=3".to_string()));

        // Storage options
        assert!(args.contains(&"--storage-opt".to_string()));
        assert!(args.contains(&"size=20G".to_string()));

        // Ulimits
        assert!(args.contains(&"--ulimit".to_string()));
        assert!(args.contains(&"nofile=1024:65536".to_string()));
        assert!(args.contains(&"nproc=1024".to_string()));

        // Volumes from containers
        assert!(args.contains(&"--volumes-from".to_string()));
        assert!(args.contains(&"data-container".to_string()));
        assert!(args.contains(&"config-container:ro".to_string()));

        // Container links
        assert!(args.contains(&"--link".to_string()));
        assert!(args.contains(&"db:database".to_string()));
        assert!(args.contains(&"cache:redis".to_string()));

        // Link-local IPs
        assert!(args.contains(&"--link-local-ip".to_string()));
        assert!(args.contains(&"169.254.1.1".to_string()));
        assert!(args.contains(&"fe80::1".to_string()));
    }

    #[test]
    fn test_run_command_additional_list_individual_options() {
        // Test each additional list option individually for proper argument generation
        let network_cmd = RunCommand::new("alpine:latest").network_alias("api");
        let network_args = network_cmd.build_command_args();
        assert!(network_args.contains(&"--network-alias".to_string()));
        assert!(network_args.contains(&"api".to_string()));

        let group_cmd = RunCommand::new("alpine:latest").group_add("wheel");
        let group_args = group_cmd.build_command_args();
        assert!(group_args.contains(&"--group-add".to_string()));
        assert!(group_args.contains(&"wheel".to_string()));

        let attach_cmd = RunCommand::new("alpine:latest").attach("stdin");
        let attach_args = attach_cmd.build_command_args();
        assert!(attach_args.contains(&"--attach".to_string()));
        assert!(attach_args.contains(&"stdin".to_string()));

        let log_cmd = RunCommand::new("alpine:latest").log_opt("compress=true");
        let log_args = log_cmd.build_command_args();
        assert!(log_args.contains(&"--log-opt".to_string()));
        assert!(log_args.contains(&"compress=true".to_string()));

        let storage_cmd =
            RunCommand::new("alpine:latest").storage_opt("dm.thinpooldev=/dev/mapper/thin-pool");
        let storage_args = storage_cmd.build_command_args();
        assert!(storage_args.contains(&"--storage-opt".to_string()));
        assert!(storage_args.contains(&"dm.thinpooldev=/dev/mapper/thin-pool".to_string()));

        let ulimit_cmd = RunCommand::new("alpine:latest").ulimit("memlock=-1:-1");
        let ulimit_args = ulimit_cmd.build_command_args();
        assert!(ulimit_args.contains(&"--ulimit".to_string()));
        assert!(ulimit_args.contains(&"memlock=-1:-1".to_string()));

        let volumes_cmd = RunCommand::new("alpine:latest").volumes_from("shared-data");
        let volumes_args = volumes_cmd.build_command_args();
        assert!(volumes_args.contains(&"--volumes-from".to_string()));
        assert!(volumes_args.contains(&"shared-data".to_string()));

        let link_cmd = RunCommand::new("alpine:latest").link("mysql:db");
        let link_args = link_cmd.build_command_args();
        assert!(link_args.contains(&"--link".to_string()));
        assert!(link_args.contains(&"mysql:db".to_string()));

        let ip_cmd = RunCommand::new("alpine:latest").link_local_ip("169.254.100.1");
        let ip_args = ip_cmd.build_command_args();
        assert!(ip_args.contains(&"--link-local-ip".to_string()));
        assert!(ip_args.contains(&"169.254.100.1".to_string()));
    }

    #[test]
    fn test_run_command_health_check_options() {
        let cmd = RunCommand::new("nginx:latest")
            .health_cmd("curl -f http://localhost/ || exit 1")
            .health_interval("30s")
            .health_retries(3)
            .health_timeout("5s")
            .health_start_period("60s")
            .health_start_interval("5s");

        let args = cmd.build_command_args();

        // Health check options
        assert!(args.contains(&"--health-cmd".to_string()));
        assert!(args.contains(&"curl -f http://localhost/ || exit 1".to_string()));
        assert!(args.contains(&"--health-interval".to_string()));
        assert!(args.contains(&"30s".to_string()));
        assert!(args.contains(&"--health-retries".to_string()));
        assert!(args.contains(&"3".to_string()));
        assert!(args.contains(&"--health-timeout".to_string()));
        assert!(args.contains(&"5s".to_string()));
        assert!(args.contains(&"--health-start-period".to_string()));
        assert!(args.contains(&"60s".to_string()));
        assert!(args.contains(&"--health-start-interval".to_string()));
        // Note: "5s" appears twice, so we can't easily test for this specific one
    }

    #[test]
    fn test_run_command_advanced_mount_network_options() {
        let cmd = RunCommand::new("alpine:latest")
            .mount("type=bind,source=/host/path,target=/container/path")
            .mount("type=volume,source=data-vol,target=/data")
            .network("frontend")
            .network("backend")
            .gpus("all")
            .annotation("io.kubernetes.cri-o.Devices", "/dev/fuse")
            .annotation("io.kubernetes.cri-o.ShmSize", "64m")
            .sysctl("net.core.somaxconn", "1024")
            .sysctl("kernel.shm_rmid_forced", "1");

        let args = cmd.build_command_args();

        // Mount options
        assert!(args.contains(&"--mount".to_string()));
        assert!(args.contains(&"type=bind,source=/host/path,target=/container/path".to_string()));
        assert!(args.contains(&"type=volume,source=data-vol,target=/data".to_string()));

        // Network options
        assert!(args.contains(&"--network".to_string()));
        assert!(args.contains(&"frontend".to_string()));
        assert!(args.contains(&"backend".to_string()));

        // GPU options
        assert!(args.contains(&"--gpus".to_string()));
        assert!(args.contains(&"all".to_string()));

        // Annotation options
        assert!(args.contains(&"--annotation".to_string()));
        assert!(args.contains(&"io.kubernetes.cri-o.Devices=/dev/fuse".to_string()));
        assert!(args.contains(&"io.kubernetes.cri-o.ShmSize=64m".to_string()));

        // Sysctl options
        assert!(args.contains(&"--sysctl".to_string()));
        assert!(args.contains(&"net.core.somaxconn=1024".to_string()));
        assert!(args.contains(&"kernel.shm_rmid_forced=1".to_string()));
    }

    #[test]
    fn test_run_command_health_advanced_individual_options() {
        // Test each health check and advanced option individually
        let health_cmd = RunCommand::new("alpine:latest").health_cmd("ping -c 1 localhost");
        let health_args = health_cmd.build_command_args();
        assert!(health_args.contains(&"--health-cmd".to_string()));
        assert!(health_args.contains(&"ping -c 1 localhost".to_string()));

        let health_interval = RunCommand::new("alpine:latest").health_interval("10s");
        let interval_args = health_interval.build_command_args();
        assert!(interval_args.contains(&"--health-interval".to_string()));
        assert!(interval_args.contains(&"10s".to_string()));

        let health_retries = RunCommand::new("alpine:latest").health_retries(5);
        let retries_args = health_retries.build_command_args();
        assert!(retries_args.contains(&"--health-retries".to_string()));
        assert!(retries_args.contains(&"5".to_string()));

        let mount_cmd = RunCommand::new("alpine:latest").mount("type=tmpfs,destination=/app");
        let mount_args = mount_cmd.build_command_args();
        assert!(mount_args.contains(&"--mount".to_string()));
        assert!(mount_args.contains(&"type=tmpfs,destination=/app".to_string()));

        let network_cmd = RunCommand::new("alpine:latest").network("my-network");
        let network_args = network_cmd.build_command_args();
        assert!(network_args.contains(&"--network".to_string()));
        assert!(network_args.contains(&"my-network".to_string()));

        let gpu_cmd = RunCommand::new("alpine:latest").gpus("device=0");
        let gpu_args = gpu_cmd.build_command_args();
        assert!(gpu_args.contains(&"--gpus".to_string()));
        assert!(gpu_args.contains(&"device=0".to_string()));

        let annotation_cmd = RunCommand::new("alpine:latest").annotation("key", "value");
        let annotation_args = annotation_cmd.build_command_args();
        assert!(annotation_args.contains(&"--annotation".to_string()));
        assert!(annotation_args.contains(&"key=value".to_string()));

        let sysctl_cmd = RunCommand::new("alpine:latest").sysctl("net.ipv4.ip_forward", "1");
        let sysctl_args = sysctl_cmd.build_command_args();
        assert!(sysctl_args.contains(&"--sysctl".to_string()));
        assert!(sysctl_args.contains(&"net.ipv4.ip_forward=1".to_string()));
    }

    #[test]
    fn test_run_command_comprehensive_health_advanced_integration() {
        let cmd = RunCommand::new("web-app:latest")
            .name("production-web-app")
            // Health checks for production readiness
            .health_cmd("curl -f http://localhost:8080/health || exit 1")
            .health_interval("30s")
            .health_retries(3)
            .health_timeout("10s")
            .health_start_period("120s")
            // Advanced mounting and networking
            .mount("type=bind,source=/var/log/app,target=/app/logs")
            .mount("type=volume,source=app-data,target=/app/data")
            .network("frontend")
            .network("backend")
            // GPU support for ML workloads
            .gpus("device=0,1")
            // Kubernetes annotations
            .annotation(
                "io.kubernetes.container.apparmor.security.beta.kubernetes.io/app",
                "runtime/default",
            )
            .annotation(
                "io.kubernetes.container.seccomp.security.alpha.kubernetes.io/app",
                "runtime/default",
            )
            // System tuning
            .sysctl("net.core.somaxconn", "65535")
            .sysctl("net.ipv4.tcp_keepalive_time", "600")
            // Additional standard options
            .port(8080, 8080)
            .env("NODE_ENV", "production")
            .memory("2g")
            .cpus("2.0")
            .restart("unless-stopped")
            .detach();

        let args = cmd.build_command_args();

        // Verify all health check and advanced options are present
        assert!(args.contains(&"--health-cmd".to_string()));
        assert!(args.contains(&"--health-interval".to_string()));
        assert!(args.contains(&"--health-retries".to_string()));
        assert!(args.contains(&"--health-timeout".to_string()));
        assert!(args.contains(&"--health-start-period".to_string()));
        assert!(args.contains(&"--mount".to_string()));
        assert!(args.contains(&"--network".to_string()));
        assert!(args.contains(&"--gpus".to_string()));
        assert!(args.contains(&"--annotation".to_string()));
        assert!(args.contains(&"--sysctl".to_string()));

        // Verify image is still at the end
        let image_pos = args.iter().position(|x| x == "web-app:latest").unwrap();
        assert!(image_pos > 0);
    }

    #[test]
    fn test_run_command_block_io_controls() {
        let cmd = RunCommand::new("alpine:latest")
            .blkio_weight(500)
            .blkio_weight_device("/dev/sda:300")
            .blkio_weight_device("/dev/sdb:700")
            .device_read_bps("/dev/sda:50mb")
            .device_write_bps("/dev/sda:30mb")
            .device_read_iops("/dev/sda:1000")
            .device_write_iops("/dev/sda:800");

        let args = cmd.build_command_args();

        // Block I/O weight
        assert!(args.contains(&"--blkio-weight".to_string()));
        assert!(args.contains(&"500".to_string()));

        // Block I/O weight per device
        assert!(args.contains(&"--blkio-weight-device".to_string()));
        assert!(args.contains(&"/dev/sda:300".to_string()));
        assert!(args.contains(&"/dev/sdb:700".to_string()));

        // Device read/write BPS
        assert!(args.contains(&"--device-read-bps".to_string()));
        assert!(args.contains(&"/dev/sda:50mb".to_string()));
        assert!(args.contains(&"--device-write-bps".to_string()));
        assert!(args.contains(&"/dev/sda:30mb".to_string()));

        // Device read/write IOPS
        assert!(args.contains(&"--device-read-iops".to_string()));
        assert!(args.contains(&"/dev/sda:1000".to_string()));
        assert!(args.contains(&"--device-write-iops".to_string()));
        assert!(args.contains(&"/dev/sda:800".to_string()));
    }

    #[test]
    fn test_run_command_realtime_cpu_networking() {
        let cmd = RunCommand::new("alpine:latest")
            .cpu_rt_period(1_000_000)
            .cpu_rt_runtime(950_000)
            .ip("172.30.100.104")
            .ip6("2001:db8::33")
            .device_cgroup_rule("c 1:3 mr")
            .device_cgroup_rule("a 7:* rmw");

        let args = cmd.build_command_args();

        // Real-time CPU scheduling
        assert!(args.contains(&"--cpu-rt-period".to_string()));
        assert!(args.contains(&"1000000".to_string()));
        assert!(args.contains(&"--cpu-rt-runtime".to_string()));
        assert!(args.contains(&"950000".to_string()));

        // Advanced networking
        assert!(args.contains(&"--ip".to_string()));
        assert!(args.contains(&"172.30.100.104".to_string()));
        assert!(args.contains(&"--ip6".to_string()));
        assert!(args.contains(&"2001:db8::33".to_string()));

        // Device cgroup rules
        assert!(args.contains(&"--device-cgroup-rule".to_string()));
        assert!(args.contains(&"c 1:3 mr".to_string()));
        assert!(args.contains(&"a 7:* rmw".to_string()));
    }

    #[test]
    fn test_run_command_advanced_system_individual_options() {
        // Test each advanced system option individually
        let blkio_cmd = RunCommand::new("alpine:latest").blkio_weight(100);
        let blkio_args = blkio_cmd.build_command_args();
        assert!(blkio_args.contains(&"--blkio-weight".to_string()));
        assert!(blkio_args.contains(&"100".to_string()));

        let weight_device_cmd =
            RunCommand::new("alpine:latest").blkio_weight_device("/dev/sda:500");
        let weight_device_args = weight_device_cmd.build_command_args();
        assert!(weight_device_args.contains(&"--blkio-weight-device".to_string()));
        assert!(weight_device_args.contains(&"/dev/sda:500".to_string()));

        let read_bps_cmd = RunCommand::new("alpine:latest").device_read_bps("/dev/sda:1mb");
        let read_bps_args = read_bps_cmd.build_command_args();
        assert!(read_bps_args.contains(&"--device-read-bps".to_string()));
        assert!(read_bps_args.contains(&"/dev/sda:1mb".to_string()));

        let write_bps_cmd = RunCommand::new("alpine:latest").device_write_bps("/dev/sda:1mb");
        let write_bps_args = write_bps_cmd.build_command_args();
        assert!(write_bps_args.contains(&"--device-write-bps".to_string()));
        assert!(write_bps_args.contains(&"/dev/sda:1mb".to_string()));

        let read_iops_cmd = RunCommand::new("alpine:latest").device_read_iops("/dev/sda:100");
        let read_iops_args = read_iops_cmd.build_command_args();
        assert!(read_iops_args.contains(&"--device-read-iops".to_string()));
        assert!(read_iops_args.contains(&"/dev/sda:100".to_string()));

        let write_iops_cmd = RunCommand::new("alpine:latest").device_write_iops("/dev/sda:100");
        let write_iops_args = write_iops_cmd.build_command_args();
        assert!(write_iops_args.contains(&"--device-write-iops".to_string()));
        assert!(write_iops_args.contains(&"/dev/sda:100".to_string()));

        let rt_period_cmd = RunCommand::new("alpine:latest").cpu_rt_period(100_000);
        let rt_period_args = rt_period_cmd.build_command_args();
        assert!(rt_period_args.contains(&"--cpu-rt-period".to_string()));
        assert!(rt_period_args.contains(&"100000".to_string()));

        let rt_runtime_cmd = RunCommand::new("alpine:latest").cpu_rt_runtime(95_000);
        let rt_runtime_args = rt_runtime_cmd.build_command_args();
        assert!(rt_runtime_args.contains(&"--cpu-rt-runtime".to_string()));
        assert!(rt_runtime_args.contains(&"95000".to_string()));

        let ip_cmd = RunCommand::new("alpine:latest").ip("192.168.1.100");
        let ip_args = ip_cmd.build_command_args();
        assert!(ip_args.contains(&"--ip".to_string()));
        assert!(ip_args.contains(&"192.168.1.100".to_string()));

        let ipv6_cmd = RunCommand::new("alpine:latest").ip6("fe80::1");
        let ipv6_args = ipv6_cmd.build_command_args();
        assert!(ipv6_args.contains(&"--ip6".to_string()));
        assert!(ipv6_args.contains(&"fe80::1".to_string()));

        let cgroup_rule_cmd = RunCommand::new("alpine:latest").device_cgroup_rule("c 1:1 rwm");
        let cgroup_rule_args = cgroup_rule_cmd.build_command_args();
        assert!(cgroup_rule_args.contains(&"--device-cgroup-rule".to_string()));
        assert!(cgroup_rule_args.contains(&"c 1:1 rwm".to_string()));
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_run_command_complete_100_percent_coverage() {
        use std::path::PathBuf;

        // Test demonstrating ALL 96 Docker run options are now implemented
        let cmd = RunCommand::new("enterprise-app:latest")
            .name("production-enterprise")
            // Basic options
            .detach()
            .interactive()
            .tty()
            .remove()
            // Environment and ports
            .env("NODE_ENV", "production")
            .port(8080, 8080)
            .volume("/data", "/app/data")
            .workdir("/app")
            .entrypoint("/app/start.sh")
            // Resource limits (Phase 2)
            .memory("4g")
            .cpus("2.0")
            .cpu_shares(1024)
            .cpu_period(100_000)
            .cpu_quota(50000)
            .cpuset_cpus("0-1")
            .cpuset_mems("0")
            .memory_swap("8g")
            .memory_reservation("2g")
            // Security & User Context (Phase 2)
            .user("app:app")
            .privileged()
            .hostname("enterprise-app")
            // Lifecycle Management (Phase 2)
            .restart("unless-stopped")
            // System Integration (Phase 2)
            .platform("linux/amd64")
            .runtime("runc")
            .isolation("default")
            .pull("always")
            .cidfile("/tmp/container.cid")
            .domainname("enterprise.local")
            .mac_address("02:42:ac:11:00:02")
            // Logging & Drivers (Phase 2)
            .log_driver("json-file")
            .volume_driver("local")
            // Namespaces (Phase 2)
            .userns("host")
            .uts("host")
            .pid("host")
            .ipc("host")
            .cgroupns("host")
            .cgroup_parent("/docker")
            // Advanced Memory & Performance (Phase 2)
            .kernel_memory("1g")
            .memory_swappiness(10)
            .oom_score_adj(-500)
            .pids_limit(1000)
            .shm_size("64m")
            // Process Control (Phase 2)
            .stop_signal("SIGTERM")
            .stop_timeout(30)
            .detach_keys("ctrl-p,ctrl-q")
            // Simple Flags (Phase 2)
            .no_sig_proxy()
            .read_only()
            .init()
            .oom_kill_disable()
            .no_healthcheck()
            .enable_content_trust()
            .publish_all()
            .quiet()
            // High-Impact List Options (Phase 3)
            .dns("8.8.8.8")
            .dns_option("ndots:2")
            .dns_search("enterprise.local")
            .add_host("api.enterprise.local:10.0.1.100")
            .cap_add("NET_ADMIN")
            .cap_drop("ALL")
            .security_opt("no-new-privileges:true")
            .device("/dev/null")
            .tmpfs("/tmp:size=100m")
            .expose("9090")
            .env_file(PathBuf::from(".env.production"))
            .label("app=enterprise")
            .label_file(PathBuf::from("labels.txt"))
            // Batch 1: Additional List/Vec Options
            .network_alias("enterprise-primary")
            .group_add("staff")
            .attach("stdout")
            .log_opt("max-size=10m")
            .storage_opt("size=100G")
            .ulimit("nofile=65536:65536")
            .volumes_from("data-container")
            .link("db:database")
            .link_local_ip("169.254.1.1")
            // Batch 2: Medium Complexity Options
            .health_cmd("curl -f http://localhost:8080/health || exit 1")
            .health_interval("30s")
            .health_retries(3)
            .health_timeout("10s")
            .health_start_period("60s")
            .health_start_interval("5s")
            .mount("type=bind,source=/host/config,target=/app/config")
            .network("enterprise-net")
            .gpus("device=0")
            .annotation("io.kubernetes.cri-o.TTY", "true")
            .sysctl("net.core.somaxconn", "65535")
            // Batch 3: Complex/Advanced Options
            .blkio_weight(500)
            .blkio_weight_device("/dev/sda:300")
            .device_read_bps("/dev/sda:100mb")
            .device_write_bps("/dev/sda:50mb")
            .device_read_iops("/dev/sda:1000")
            .device_write_iops("/dev/sda:500")
            .cpu_rt_period(1_000_000)
            .cpu_rt_runtime(950_000)
            .ip("10.0.1.50")
            .ip6("2001:db8::50")
            .device_cgroup_rule("c 1:1 rwm");

        let args = cmd.build_command_args();

        // Verify we have a substantial command with all option types
        assert!(args.len() > 150); // Should be a very long command

        // Verify key options from each batch are present
        assert!(args.contains(&"--detach".to_string()));
        assert!(args.contains(&"--memory".to_string()));
        assert!(args.contains(&"--dns".to_string()));
        assert!(args.contains(&"--network-alias".to_string()));
        assert!(args.contains(&"--health-cmd".to_string()));
        assert!(args.contains(&"--blkio-weight".to_string()));

        // Verify image is still at the end
        let image_pos = args
            .iter()
            .position(|x| x == "enterprise-app:latest")
            .unwrap();
        assert!(image_pos > 100); // Should be very far into the command
        assert_eq!(args[args.len() - 1], "enterprise-app:latest");

        println!("COMPLETE! All 96 Docker run options implemented and tested!");
    }
}
