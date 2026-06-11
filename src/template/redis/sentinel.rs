//! Redis Sentinel template for high availability setup
//!
//! This template sets up a complete Redis Sentinel environment with:
//! - One Redis master instance
//! - Multiple Redis replica instances
//! - Multiple Sentinel instances for monitoring and failover

#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::unnecessary_get_then_check)]

use super::common::{DEFAULT_REDIS_IMAGE, DEFAULT_REDIS_TAG};
use crate::template::{Template, TemplateConfig, TemplateError};
use crate::{DockerCommand, NetworkCreateCommand, RunCommand};
use async_trait::async_trait;

/// Redis Sentinel template for high availability setup
pub struct RedisSentinelTemplate {
    name: String,
    master_name: String,
    num_replicas: usize,
    num_sentinels: usize,
    quorum: usize,
    master_port: u16,
    replica_port_base: u16,
    sentinel_port_base: u16,
    password: Option<String>,
    down_after_milliseconds: u32,
    failover_timeout: u32,
    parallel_syncs: u32,
    persistence: bool,
    network: Option<String>,
    /// IP to announce to Sentinel-aware clients and for the monitored master
    announce_ip: Option<String>,
    /// Custom Redis image
    redis_image: Option<String>,
    /// Custom Redis tag
    redis_tag: Option<String>,
    /// Platform for containers
    platform: Option<String>,
}

impl RedisSentinelTemplate {
    /// Create a new Redis Sentinel template
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            master_name: "mymaster".to_string(),
            num_replicas: 2,
            num_sentinels: 3,
            quorum: 2,
            master_port: 6379,
            replica_port_base: 6380,
            sentinel_port_base: 26379,
            password: None,
            down_after_milliseconds: 5000,
            failover_timeout: 10000,
            parallel_syncs: 1,
            persistence: false,
            network: None,
            announce_ip: None,
            redis_image: None,
            redis_tag: None,
            platform: None,
        }
    }

    /// Set the master name for Sentinel monitoring
    pub fn master_name(mut self, name: impl Into<String>) -> Self {
        self.master_name = name.into();
        self
    }

    /// Set the number of Redis replicas
    pub fn num_replicas(mut self, num: usize) -> Self {
        self.num_replicas = num;
        self
    }

    /// Set the number of Sentinel instances
    pub fn num_sentinels(mut self, num: usize) -> Self {
        self.num_sentinels = num;
        self
    }

    /// Set the quorum for failover decisions
    pub fn quorum(mut self, quorum: usize) -> Self {
        self.quorum = quorum;
        self
    }

    /// Set the Redis master port
    pub fn master_port(mut self, port: u16) -> Self {
        self.master_port = port;
        self
    }

    /// Set the base port for replicas (will increment for each replica)
    pub fn replica_port_base(mut self, port: u16) -> Self {
        self.replica_port_base = port;
        self
    }

    /// Set the base port for Sentinels (will increment for each Sentinel)
    pub fn sentinel_port_base(mut self, port: u16) -> Self {
        self.sentinel_port_base = port;
        self
    }

    /// Set Redis password for authentication
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Set the time in milliseconds before master is considered down
    pub fn down_after_milliseconds(mut self, ms: u32) -> Self {
        self.down_after_milliseconds = ms;
        self
    }

    /// Set the failover timeout in milliseconds
    pub fn failover_timeout(mut self, ms: u32) -> Self {
        self.failover_timeout = ms;
        self
    }

    /// Set the number of parallel syncs during failover
    pub fn parallel_syncs(mut self, num: u32) -> Self {
        self.parallel_syncs = num;
        self
    }

    /// Enable persistence for Redis instances
    pub fn with_persistence(mut self) -> Self {
        self.persistence = true;
        self
    }

    /// Use a specific network
    pub fn network(mut self, network: impl Into<String>) -> Self {
        self.network = Some(network.into());
        self
    }

    /// Set the IP address to announce to Sentinel-aware clients.
    ///
    /// By default Sentinel monitors the master by its container hostname and
    /// reports replica/sentinel addresses using internal Docker addresses,
    /// which are unreachable from a host-side client. Setting an announce IP
    /// makes the topology reachable from outside the Docker network:
    ///
    /// - the monitored master is registered at `<announce_ip>:<master_port>`,
    ///   so `SENTINEL get-master-addr-by-name` returns a host-reachable address,
    /// - each replica announces `<announce_ip>:<replica_host_port>`,
    /// - each sentinel announces `<announce_ip>:<sentinel_host_port>`.
    ///
    /// Use `127.0.0.1` (or the host's LAN address) when connecting from the
    /// machine running Docker.
    pub fn announce_ip(mut self, ip: impl Into<String>) -> Self {
        self.announce_ip = Some(ip.into());
        self
    }

    /// Use a custom Redis image and tag
    pub fn custom_redis_image(mut self, image: impl Into<String>, tag: impl Into<String>) -> Self {
        self.redis_image = Some(image.into());
        self.redis_tag = Some(tag.into());
        self
    }

    /// Set the platform for the containers (e.g., "linux/arm64", "linux/amd64")
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Start the Redis Sentinel cluster
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network creation fails
    /// - Starting any container (master, replica, or sentinel) fails
    pub async fn start(self) -> Result<SentinelConnectionInfo, crate::Error> {
        self.start_topology().await
    }

    /// Host address reported to clients for the master, replicas and sentinels.
    ///
    /// Returns the configured announce IP when set, otherwise `localhost`.
    fn resolved_host(&self) -> String {
        self.announce_ip
            .clone()
            .unwrap_or_else(|| "localhost".to_string())
    }

    /// The host port mapped to a replica's Redis port.
    fn replica_port(&self, index: usize) -> u16 {
        self.replica_port_base + u16::try_from(index).unwrap_or(0)
    }

    /// The host port mapped to a sentinel's port.
    fn sentinel_port(&self, index: usize) -> u16 {
        self.sentinel_port_base + u16::try_from(index).unwrap_or(0)
    }

    /// Start the topology and return connection information.
    ///
    /// Shared by the consuming [`start`](Self::start) helper and the
    /// [`Template`] implementation so both bring up an identical topology.
    async fn start_topology(&self) -> Result<SentinelConnectionInfo, crate::Error> {
        let network_name = self
            .network
            .clone()
            .unwrap_or_else(|| format!("{}-network", self.name));

        // Create network if not provided
        if self.network.is_none() {
            NetworkCreateCommand::new(&network_name)
                .execute()
                .await
                .map_err(|e| crate::Error::Custom {
                    message: format!("Failed to create network: {e}"),
                })?;
        }

        // Start Redis master
        let master_name = format!("{}-master", self.name);
        let mut master_cmd = self.build_redis_command(&master_name, self.master_port, None, None);
        master_cmd = master_cmd.network(&network_name);

        master_cmd
            .execute()
            .await
            .map_err(|e| crate::Error::Custom {
                message: format!("Failed to start master: {e}"),
            })?;

        // Start Redis replicas
        let mut replica_containers = Vec::new();
        for i in 0..self.num_replicas {
            let replica_name = format!("{}-replica-{}", self.name, i + 1);
            let replica_port = self.replica_port(i);

            let mut replica_cmd = self.build_redis_command(
                &replica_name,
                replica_port,
                Some(&master_name),
                Some(replica_port),
            );
            replica_cmd = replica_cmd.network(&network_name);

            replica_cmd
                .execute()
                .await
                .map_err(|e| crate::Error::Custom {
                    message: format!("Failed to start replica {}: {e}", i + 1),
                })?;

            replica_containers.push(replica_name);
        }

        // Create Sentinel configuration
        let sentinel_config = self.build_sentinel_config(&master_name);

        // Start Sentinel instances
        let mut sentinel_containers = Vec::new();
        for i in 0..self.num_sentinels {
            let sentinel_name = format!("{}-sentinel-{}", self.name, i + 1);
            let sentinel_port = self.sentinel_port(i);

            // Each sentinel announces its own host-mapped port when an
            // announce IP is configured so clients can reach it directly.
            let sentinel_config = if self.announce_ip.is_some() {
                format!("{sentinel_config}\nsentinel announce-port {sentinel_port}")
            } else {
                sentinel_config.clone()
            };

            let mut sentinel_cmd = Self::build_sentinel_command(
                &sentinel_name,
                sentinel_port,
                &sentinel_config,
                self.redis_image.as_deref(),
                self.redis_tag.as_deref(),
                self.platform.as_deref(),
            );
            sentinel_cmd = sentinel_cmd.network(&network_name);

            sentinel_cmd
                .execute()
                .await
                .map_err(|e| crate::Error::Custom {
                    message: format!("Failed to start sentinel {}: {e}", i + 1),
                })?;

            sentinel_containers.push((sentinel_name, sentinel_port));
        }

        let host = self.resolved_host();

        Ok(SentinelConnectionInfo {
            name: self.name.clone(),
            master_name: self.master_name.clone(),
            master_host: host.clone(),
            master_port: self.master_port,
            replica_ports: (0..self.num_replicas)
                .map(|i| self.replica_port(i))
                .collect(),
            sentinels: sentinel_containers
                .into_iter()
                .map(|(_, port)| SentinelInfo {
                    host: host.clone(),
                    port,
                })
                .collect(),
            password: self.password.clone(),
            network: network_name,
            containers: {
                let mut containers = vec![master_name];
                containers.extend(replica_containers);
                containers.extend(
                    (0..self.num_sentinels).map(|i| format!("{}-sentinel-{}", self.name, i + 1)),
                );
                containers
            },
        })
    }

    /// Build a Redis command (master or replica)
    ///
    /// When `announce_port` is provided alongside a configured announce IP, the
    /// instance advertises `<announce_ip>:<announce_port>` so a host-side client
    /// can reach it directly (used for replicas).
    fn build_redis_command(
        &self,
        name: &str,
        port: u16,
        master: Option<&str>,
        announce_port: Option<u16>,
    ) -> RunCommand {
        // Choose image based on custom image or default
        let image = if let Some(ref custom_image) = self.redis_image {
            if let Some(ref tag) = self.redis_tag {
                format!("{custom_image}:{tag}")
            } else {
                custom_image.clone()
            }
        } else {
            format!("{DEFAULT_REDIS_IMAGE}:{DEFAULT_REDIS_TAG}")
        };

        let mut cmd = RunCommand::new(image).name(name).port(port, 6379).detach();

        // Add platform if specified
        if let Some(ref platform) = self.platform {
            cmd = cmd.platform(platform);
        }

        // Add persistence if enabled
        if self.persistence {
            cmd = cmd.volume(format!("{name}-data"), "/data");
        }

        // Build command arguments
        let mut args = Vec::new();

        // If this is a replica, configure replication
        if let Some(master_name) = master {
            args.push(format!("--replicaof {master_name} 6379"));
        }

        // Add password if set
        if let Some(ref password) = self.password {
            args.push(format!("--requirepass {password}"));
            if master.is_some() {
                args.push(format!("--masterauth {password}"));
            }
        }

        // Add protected mode
        args.push("--protected-mode no".to_string());

        // Announce a host-reachable address so Sentinel reports an address
        // clients outside the Docker network can connect to.
        if let Some(ref ip) = self.announce_ip {
            args.push(format!("--replica-announce-ip {ip}"));
            if let Some(announce_port) = announce_port {
                args.push(format!("--replica-announce-port {announce_port}"));
            }
        }

        if !args.is_empty() {
            cmd = cmd.entrypoint("redis-server").cmd(args);
        }

        cmd
    }

    /// Build Sentinel command
    fn build_sentinel_command(
        name: &str,
        port: u16,
        config: &str,
        redis_image: Option<&str>,
        redis_tag: Option<&str>,
        platform: Option<&str>,
    ) -> RunCommand {
        // Choose image based on custom image or default
        let image = if let Some(custom_image) = redis_image {
            if let Some(tag) = redis_tag {
                format!("{custom_image}:{tag}")
            } else {
                custom_image.to_string()
            }
        } else {
            format!("{DEFAULT_REDIS_IMAGE}:{DEFAULT_REDIS_TAG}")
        };

        let mut cmd = RunCommand::new(image).name(name).port(port, 26379).detach();

        // Add platform if specified
        if let Some(platform) = platform {
            cmd = cmd.platform(platform);
        }

        // Create inline Sentinel config using echo
        let config_cmd = format!(
            "echo '{}' > /tmp/sentinel.conf && redis-sentinel /tmp/sentinel.conf",
            config.replace('\'', "'\\''").replace('\n', "\\n")
        );

        cmd = cmd.entrypoint("sh").cmd(vec!["-c".to_string(), config_cmd]);

        cmd
    }

    /// Build Sentinel configuration
    fn build_sentinel_config(&self, master_container: &str) -> String {
        let mut config = Vec::new();

        config.push("port 26379".to_string());

        // When an announce IP is configured, monitor the master at its
        // host-reachable address (announce IP + published master port) so
        // `SENTINEL get-master-addr-by-name` returns a usable address. The
        // sentinels also advertise the announce IP to clients. Without it,
        // the master is monitored by its container hostname on the internal
        // Redis port, which is only reachable inside the Docker network.
        if let Some(ref ip) = self.announce_ip {
            config.push(format!(
                "sentinel monitor {} {} {} {}",
                self.master_name, ip, self.master_port, self.quorum
            ));
            config.push(format!("sentinel announce-ip {ip}"));
        } else {
            config.push(format!(
                "sentinel monitor {} {} 6379 {}",
                self.master_name, master_container, self.quorum
            ));
        }

        if let Some(ref password) = self.password {
            config.push(format!(
                "sentinel auth-pass {} {}",
                self.master_name, password
            ));
        }

        config.push(format!(
            "sentinel down-after-milliseconds {} {}",
            self.master_name, self.down_after_milliseconds
        ));
        config.push(format!(
            "sentinel failover-timeout {} {}",
            self.master_name, self.failover_timeout
        ));
        config.push(format!(
            "sentinel parallel-syncs {} {}",
            self.master_name, self.parallel_syncs
        ));

        config.join("\n")
    }

    /// Names of every container managed by this template.
    ///
    /// Mirrors the naming used when starting the topology: one master, then
    /// replicas (`{name}-replica-{i}`) and sentinels (`{name}-sentinel-{i}`).
    fn container_names(&self) -> Vec<String> {
        let mut names = vec![format!("{}-master", self.name)];
        names.extend((0..self.num_replicas).map(|i| format!("{}-replica-{}", self.name, i + 1)));
        names.extend((0..self.num_sentinels).map(|i| format!("{}-sentinel-{}", self.name, i + 1)));
        names
    }

    /// The `redis-cli` PING arguments used for readiness checks.
    fn build_ping_args(&self) -> Vec<String> {
        let mut args = vec!["redis-cli".to_string()];
        if let Some(ref password) = self.password {
            args.push("-a".to_string());
            args.push(password.clone());
        }
        args.push("ping".to_string());
        args
    }

    /// Wait for the master and every sentinel to respond to PING.
    ///
    /// Polls each container with `redis-cli ping` every 500ms until all reply
    /// with PONG or the timeout is exceeded. Sentinels accept `PING` on their
    /// port, so the same check works for both roles.
    async fn wait_for_topology_ready(
        &self,
        timeout: std::time::Duration,
    ) -> Result<(), TemplateError> {
        use crate::ExecCommand;

        let ping_args = self.build_ping_args();
        let check_interval = std::time::Duration::from_millis(500);
        let start = std::time::Instant::now();

        // The master plus every sentinel must answer before the topology is usable.
        let mut targets: Vec<String> = vec![format!("{}-master", self.name)];
        targets
            .extend((0..self.num_sentinels).map(|i| format!("{}-sentinel-{}", self.name, i + 1)));

        let mut pending = targets;

        loop {
            let mut still_pending = Vec::new();
            for name in &pending {
                let ready = ExecCommand::new(name, ping_args.clone())
                    .execute()
                    .await
                    .is_ok_and(|output| output.stdout.trim().eq_ignore_ascii_case("PONG"));

                if !ready {
                    still_pending.push(name.clone());
                }
            }

            if still_pending.is_empty() {
                return Ok(());
            }
            pending = still_pending;

            if start.elapsed() >= timeout {
                return Err(TemplateError::Timeout(format!(
                    "Sentinel topology '{}' containers [{}] did not respond to PING within {:?}",
                    self.name,
                    pending.join(", "),
                    timeout
                )));
            }

            tokio::time::sleep(check_interval).await;
        }
    }
}

#[async_trait]
impl Template for RedisSentinelTemplate {
    fn name(&self) -> &str {
        &self.name
    }

    fn config(&self) -> &TemplateConfig {
        // Sentinel manages multiple containers and does not map to a single config.
        unimplemented!("RedisSentinelTemplate manages multiple containers")
    }

    fn config_mut(&mut self) -> &mut TemplateConfig {
        unimplemented!("RedisSentinelTemplate manages multiple containers")
    }

    async fn start(&self) -> Result<String, TemplateError> {
        let info = self.start_topology().await?;
        Ok(format!(
            "Redis Sentinel '{}' started with master, {} replica(s) and {} sentinel(s) (master at {}:{})",
            self.name,
            self.num_replicas,
            self.num_sentinels,
            info.master_host,
            info.master_port
        ))
    }

    async fn start_and_wait(&self) -> Result<String, TemplateError> {
        // Override the default, which inspects `config()`; this template
        // manages multiple containers and has no single config.
        let summary = self.start().await?;
        self.wait_for_ready().await?;
        Ok(summary)
    }

    async fn is_running(&self) -> Result<bool, TemplateError> {
        use crate::PsCommand;

        // Report on the master container, which represents the topology.
        let master = format!("{}-master", self.name);
        let output = PsCommand::new()
            .filter(format!("name={master}"))
            .quiet()
            .execute()
            .await?;

        Ok(!output.stdout.trim().is_empty())
    }

    async fn wait_for_ready(&self) -> Result<(), TemplateError> {
        self.wait_for_topology_ready(std::time::Duration::from_secs(60))
            .await
    }

    async fn stop(&self) -> Result<(), TemplateError> {
        use crate::StopCommand;

        for name in self.container_names() {
            let _ = StopCommand::new(&name).execute().await;
        }

        Ok(())
    }

    async fn remove(&self) -> Result<(), TemplateError> {
        use crate::{NetworkRmCommand, RmCommand};

        for name in self.container_names() {
            let _ = RmCommand::new(&name).force().volumes().execute().await;
        }

        // Remove the network only if it was created by the template.
        if self.network.is_none() {
            let network_name = format!("{}-network", self.name);
            let _ = NetworkRmCommand::new(&network_name).execute().await;
        }

        Ok(())
    }
}

/// Connection information for Redis Sentinel
pub struct SentinelConnectionInfo {
    /// Name of the Sentinel deployment
    pub name: String,
    /// Master name used by Sentinel
    pub master_name: String,
    /// Host address of the Redis master
    pub master_host: String,
    /// Port of the Redis master
    pub master_port: u16,
    /// Ports of the Redis replica instances
    pub replica_ports: Vec<u16>,
    /// Information about Sentinel instances
    pub sentinels: Vec<SentinelInfo>,
    /// Redis password if authentication is enabled
    pub password: Option<String>,
    /// Docker network name
    pub network: String,
    /// Names of all containers in the cluster
    pub containers: Vec<String>,
}

/// Information about a Sentinel instance
pub struct SentinelInfo {
    /// Host address of the Sentinel
    pub host: String,
    /// Port of the Sentinel
    pub port: u16,
}

impl SentinelConnectionInfo {
    /// Get Redis URL for direct master connection
    pub fn master_url(&self) -> String {
        if let Some(ref password) = self.password {
            format!(
                "redis://default:{}@{}:{}",
                password, self.master_host, self.master_port
            )
        } else {
            format!("redis://{}:{}", self.master_host, self.master_port)
        }
    }

    /// Get Sentinel URLs for Sentinel-aware clients
    pub fn sentinel_urls(&self) -> Vec<String> {
        self.sentinels
            .iter()
            .map(|s| format!("redis://{}:{}", s.host, s.port))
            .collect()
    }

    /// Stop all containers in the Sentinel cluster
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Stopping or removing any container fails
    /// - Removing the network fails
    pub async fn stop(self) -> Result<(), crate::Error> {
        use crate::{NetworkRmCommand, RmCommand, StopCommand};

        // Stop and remove all containers
        for container in &self.containers {
            StopCommand::new(container)
                .execute()
                .await
                .map_err(|e| crate::Error::Custom {
                    message: format!("Failed to stop {container}: {e}"),
                })?;

            RmCommand::new(container)
                .force()
                .volumes()
                .execute()
                .await
                .map_err(|e| crate::Error::Custom {
                    message: format!("Failed to remove {container}: {e}"),
                })?;
        }

        // Remove network if it was created by us
        if self.network.starts_with(&self.name) {
            NetworkRmCommand::new(&self.network)
                .execute()
                .await
                .map_err(|e| crate::Error::Custom {
                    message: format!("Failed to remove network: {e}"),
                })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentinel_template_defaults() {
        let template = RedisSentinelTemplate::new("test-sentinel");
        assert_eq!(template.name, "test-sentinel");
        assert_eq!(template.master_name, "mymaster");
        assert_eq!(template.num_replicas, 2);
        assert_eq!(template.num_sentinels, 3);
        assert_eq!(template.quorum, 2);
    }

    #[test]
    fn test_sentinel_template_builder() {
        let template = RedisSentinelTemplate::new("test-sentinel")
            .master_name("primary")
            .num_replicas(3)
            .num_sentinels(5)
            .quorum(3)
            .password("secret")
            .with_persistence();

        assert_eq!(template.master_name, "primary");
        assert_eq!(template.num_replicas, 3);
        assert_eq!(template.num_sentinels, 5);
        assert_eq!(template.quorum, 3);
        assert_eq!(template.password, Some("secret".to_string()));
        assert!(template.persistence);
    }

    #[test]
    fn test_sentinel_config_generation() {
        let template = RedisSentinelTemplate::new("test")
            .master_name("mymaster")
            .password("secret")
            .quorum(2);

        let config = template.build_sentinel_config("redis-master");

        assert!(config.contains("sentinel monitor mymaster redis-master 6379 2"));
        assert!(config.contains("sentinel auth-pass mymaster secret"));
        assert!(config.contains("sentinel down-after-milliseconds mymaster 5000"));
    }

    #[test]
    fn test_sentinel_config_without_announce_uses_container_host() {
        let template = RedisSentinelTemplate::new("test").master_name("mymaster");
        let config = template.build_sentinel_config("test-master");

        assert!(config.contains("sentinel monitor mymaster test-master 6379 2"));
        assert!(!config.contains("sentinel announce-ip"));
    }

    #[test]
    fn test_sentinel_config_with_announce_uses_announced_master_address() {
        let template = RedisSentinelTemplate::new("test")
            .master_name("mymaster")
            .master_port(6390)
            .quorum(2)
            .announce_ip("127.0.0.1");

        let config = template.build_sentinel_config("test-master");

        // The master is registered at the announced host-reachable address,
        // not the container hostname on the internal port.
        assert!(config.contains("sentinel monitor mymaster 127.0.0.1 6390 2"));
        assert!(config.contains("sentinel announce-ip 127.0.0.1"));
        assert!(!config.contains("sentinel monitor mymaster test-master"));
    }

    #[test]
    fn test_resolved_host_defaults_to_localhost() {
        let template = RedisSentinelTemplate::new("test");
        assert_eq!(template.resolved_host(), "localhost");
    }

    #[test]
    fn test_resolved_host_uses_announce_ip() {
        let template = RedisSentinelTemplate::new("test").announce_ip("10.0.0.5");
        assert_eq!(template.resolved_host(), "10.0.0.5");
    }

    #[test]
    fn test_replica_command_includes_announce_args() {
        let template = RedisSentinelTemplate::new("test").announce_ip("127.0.0.1");

        let cmd =
            template.build_redis_command("test-replica-1", 6381, Some("test-master"), Some(6381));
        let args = cmd.build_command_args();
        let joined = args.join(" ");

        assert!(joined.contains("--replica-announce-ip 127.0.0.1"));
        assert!(joined.contains("--replica-announce-port 6381"));
    }

    #[test]
    fn test_replica_command_without_announce_has_no_announce_args() {
        let template = RedisSentinelTemplate::new("test");

        let cmd =
            template.build_redis_command("test-replica-1", 6381, Some("test-master"), Some(6381));
        let joined = cmd.build_command_args().join(" ");

        assert!(!joined.contains("--replica-announce-ip"));
        assert!(!joined.contains("--replica-announce-port"));
    }

    #[test]
    fn test_build_ping_args_without_password() {
        let template = RedisSentinelTemplate::new("test");
        assert_eq!(template.build_ping_args(), vec!["redis-cli", "ping"]);
    }

    #[test]
    fn test_build_ping_args_with_password() {
        let template = RedisSentinelTemplate::new("test").password("secret");
        assert_eq!(
            template.build_ping_args(),
            vec!["redis-cli", "-a", "secret", "ping"]
        );
    }

    #[test]
    fn test_container_names() {
        let template = RedisSentinelTemplate::new("test")
            .num_replicas(2)
            .num_sentinels(3);

        assert_eq!(
            template.container_names(),
            vec![
                "test-master",
                "test-replica-1",
                "test-replica-2",
                "test-sentinel-1",
                "test-sentinel-2",
                "test-sentinel-3",
            ]
        );
    }

    #[test]
    fn test_template_trait_name() {
        let template = RedisSentinelTemplate::new("test-sentinel");
        assert_eq!(Template::name(&template), "test-sentinel");
    }
}
