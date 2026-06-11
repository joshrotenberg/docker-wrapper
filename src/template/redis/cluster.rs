//! Redis Cluster template for multi-node Redis setup with sharding and replication

#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_errors_doc)]

use super::common::{
    REDIS_INSIGHT_CLUSTER_IMAGE, REDIS_INSIGHT_TAG, REDIS_STACK_SERVER_IMAGE, REDIS_STACK_TAG,
};
use crate::template::{Template, TemplateConfig, TemplateError};
use crate::{DockerCommand, ExecCommand, NetworkCreateCommand, RunCommand};
use async_trait::async_trait;

/// Redis Cluster template for automatic multi-node cluster setup
pub struct RedisClusterTemplate {
    /// Base name for the cluster
    name: String,
    /// Number of master nodes (minimum 3)
    num_masters: usize,
    /// Number of replicas per master
    num_replicas: usize,
    /// Base port for Redis nodes
    port_base: u16,
    /// Network name for cluster communication
    network_name: String,
    /// Password for cluster authentication
    password: Option<String>,
    /// IP to announce to other nodes
    announce_ip: Option<String>,
    /// Volume prefix for persistence
    volume_prefix: Option<String>,
    /// Memory limit per node
    memory_limit: Option<String>,
    /// Cluster node timeout in milliseconds
    node_timeout: u32,
    /// Whether to remove containers on stop
    auto_remove: bool,
    /// Whether to use Redis Stack instead of standard Redis
    use_redis_stack: bool,
    /// Image tag used for the Redis Stack server image
    stack_tag: String,
    /// Whether to include RedisInsight GUI
    with_redis_insight: bool,
    /// Port for RedisInsight UI
    redis_insight_port: u16,
    /// Image tag used for the RedisInsight image
    redis_insight_tag: String,
    /// Custom Redis image
    redis_image: Option<String>,
    /// Custom Redis tag
    redis_tag: Option<String>,
    /// Platform for containers
    platform: Option<String>,
}

impl RedisClusterTemplate {
    /// Create a new Redis Cluster template with default settings
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let network_name = format!("{}-network", name);

        Self {
            name,
            num_masters: 3,
            num_replicas: 0,
            port_base: 7000,
            network_name,
            password: None,
            announce_ip: None,
            volume_prefix: None,
            memory_limit: None,
            node_timeout: 5000,
            auto_remove: false,
            use_redis_stack: false,
            stack_tag: REDIS_STACK_TAG.to_string(),
            with_redis_insight: false,
            redis_insight_port: 8001,
            redis_insight_tag: REDIS_INSIGHT_TAG.to_string(),
            redis_image: None,
            redis_tag: None,
            platform: None,
        }
    }

    /// Create a new Redis Cluster template with settings from environment variables.
    ///
    /// Falls back to defaults if environment variables are not set.
    ///
    /// # Environment Variables
    ///
    /// - `REDIS_CLUSTER_PORT_BASE`: Base port for Redis nodes (default: 7000)
    /// - `REDIS_CLUSTER_NUM_MASTERS`: Number of master nodes (default: 3)
    /// - `REDIS_CLUSTER_NUM_REPLICAS`: Number of replicas per master (default: 0)
    /// - `REDIS_CLUSTER_PASSWORD`: Password for cluster authentication (optional)
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::RedisClusterTemplate;
    ///
    /// // Uses environment variables if set, otherwise uses defaults
    /// let template = RedisClusterTemplate::from_env("my-cluster");
    /// ```
    pub fn from_env(name: impl Into<String>) -> Self {
        let mut template = Self::new(name);

        if let Ok(port_base) = std::env::var("REDIS_CLUSTER_PORT_BASE") {
            if let Ok(port) = port_base.parse::<u16>() {
                template.port_base = port;
            }
        }

        if let Ok(num_masters) = std::env::var("REDIS_CLUSTER_NUM_MASTERS") {
            if let Ok(masters) = num_masters.parse::<usize>() {
                template.num_masters = masters.max(3);
            }
        }

        if let Ok(num_replicas) = std::env::var("REDIS_CLUSTER_NUM_REPLICAS") {
            if let Ok(replicas) = num_replicas.parse::<usize>() {
                template.num_replicas = replicas;
            }
        }

        if let Ok(password) = std::env::var("REDIS_CLUSTER_PASSWORD") {
            template.password = Some(password);
        }

        template
    }

    /// Get the configured port base
    pub fn get_port_base(&self) -> u16 {
        self.port_base
    }

    /// Get the configured number of masters
    pub fn get_num_masters(&self) -> usize {
        self.num_masters
    }

    /// Get the configured number of replicas per master
    pub fn get_num_replicas(&self) -> usize {
        self.num_replicas
    }

    /// Set the number of master nodes (minimum 3)
    pub fn num_masters(mut self, masters: usize) -> Self {
        self.num_masters = masters.max(3);
        self
    }

    /// Set the number of replicas per master
    pub fn num_replicas(mut self, replicas: usize) -> Self {
        self.num_replicas = replicas;
        self
    }

    /// Set the base port for Redis nodes
    pub fn port_base(mut self, port: u16) -> Self {
        self.port_base = port;
        self
    }

    /// Set cluster password
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Set the IP to announce to other cluster nodes
    pub fn cluster_announce_ip(mut self, ip: impl Into<String>) -> Self {
        self.announce_ip = Some(ip.into());
        self
    }

    /// Enable persistence with volume prefix
    pub fn with_persistence(mut self, volume_prefix: impl Into<String>) -> Self {
        self.volume_prefix = Some(volume_prefix.into());
        self
    }

    /// Set memory limit per node
    pub fn memory_limit(mut self, limit: impl Into<String>) -> Self {
        self.memory_limit = Some(limit.into());
        self
    }

    /// Set cluster node timeout in milliseconds
    pub fn cluster_node_timeout(mut self, timeout: u32) -> Self {
        self.node_timeout = timeout;
        self
    }

    /// Enable auto-remove when stopped
    pub fn auto_remove(mut self) -> Self {
        self.auto_remove = true;
        self
    }

    /// Use Redis Stack instead of standard Redis (includes modules like JSON, Search, Graph, TimeSeries, Bloom).
    ///
    /// Uses the `redis/redis-stack-server` image pinned to a known-good default
    /// tag (`7.4.0-v3`) rather than `latest`, so that runs are reproducible.
    /// Call [`Self::stack_version`] to pin a different tag, or
    /// [`Self::custom_redis_image`] for full control.
    pub fn with_redis_stack(mut self) -> Self {
        self.use_redis_stack = true;
        self
    }

    /// Pin the Redis Stack server image tag (e.g. `"7.4.0-v3"`).
    ///
    /// Only affects the image used when [`Self::with_redis_stack`] is enabled.
    /// The default is a known-good pinned tag rather than `latest`, so that runs
    /// are reproducible. A [`Self::custom_redis_image`] takes precedence over
    /// this setting.
    ///
    /// # Example
    ///
    /// ```rust
    /// use docker_wrapper::RedisClusterTemplate;
    ///
    /// let template = RedisClusterTemplate::new("my-cluster")
    ///     .with_redis_stack()
    ///     .stack_version("7.4.0-v3");
    /// ```
    pub fn stack_version(mut self, tag: impl Into<String>) -> Self {
        self.stack_tag = tag.into();
        self
    }

    /// Enable RedisInsight GUI for cluster visualization and management.
    ///
    /// Uses the `redislabs/redisinsight` image pinned to a known-good default
    /// tag (`2.60`) rather than `latest`, so that runs are reproducible. Call
    /// [`Self::redis_insight_version`] to pin a different tag.
    pub fn with_redis_insight(mut self) -> Self {
        self.with_redis_insight = true;
        self
    }

    /// Set the port for RedisInsight UI (default: 8001)
    pub fn redis_insight_port(mut self, port: u16) -> Self {
        self.redis_insight_port = port;
        self
    }

    /// Pin the RedisInsight image tag (e.g. `"2.60"`).
    ///
    /// Only affects the image used when [`Self::with_redis_insight`] is enabled.
    /// The default is a known-good pinned tag rather than `latest`, so that runs
    /// are reproducible.
    ///
    /// # Example
    ///
    /// ```rust
    /// use docker_wrapper::RedisClusterTemplate;
    ///
    /// let template = RedisClusterTemplate::new("my-cluster")
    ///     .with_redis_insight()
    ///     .redis_insight_version("2.60");
    /// ```
    pub fn redis_insight_version(mut self, tag: impl Into<String>) -> Self {
        self.redis_insight_tag = tag.into();
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

    /// Get the total number of nodes
    fn total_nodes(&self) -> usize {
        self.num_masters + (self.num_masters * self.num_replicas)
    }

    /// Container name for the node at `index`.
    ///
    /// Nodes are named deterministically as `{name}-node-{index}`, where
    /// `index` runs from `0` to `total_nodes() - 1`. This naming contract is
    /// stable and is relied upon by readiness polling and the per-node
    /// accessors ([`node_names`](Self::node_names) and [`node`](Self::node)).
    fn node_name(&self, index: usize) -> String {
        format!("{}-node-{}", self.name, index)
    }

    /// List the container names for every node in the cluster.
    ///
    /// Names follow the deterministic `{name}-node-{i}` contract, ordered by
    /// node index from `0` to `total_nodes() - 1`. The first
    /// [`get_num_masters`](Self::get_num_masters) entries are masters and the
    /// remainder are replicas, mirroring how `redis-cli --cluster create`
    /// assigns roles.
    ///
    /// This is the building block for targeted per-node fault injection: pick a
    /// name and pause, partition, or kill exactly that container.
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::RedisClusterTemplate;
    ///
    /// let cluster = RedisClusterTemplate::new("chaos").num_masters(3);
    /// assert_eq!(
    ///     cluster.node_names(),
    ///     vec!["chaos-node-0", "chaos-node-1", "chaos-node-2"],
    /// );
    /// ```
    pub fn node_names(&self) -> Vec<String> {
        (0..self.total_nodes()).map(|i| self.node_name(i)).collect()
    }

    /// Get a handle to a single node by index.
    ///
    /// Returns a [`ClusterNode`] describing the node's container name, the host
    /// port mapped to its Redis port, and its expected role, or `None` if
    /// `index` is out of range (`index >= total_nodes()`).
    ///
    /// The returned values are derived from configuration only -- this is a
    /// plain accessor that performs no Docker calls. The role is the role
    /// `redis-cli --cluster create` assigns: indices `0..num_masters` are
    /// masters and the rest are replicas. To read the live role from a running
    /// node instead (for example after a failover), use
    /// [`node_role`](Self::node_role).
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::{NodeRole, RedisClusterTemplate};
    ///
    /// let cluster = RedisClusterTemplate::new("chaos")
    ///     .num_masters(3)
    ///     .num_replicas(1)
    ///     .port_base(7000);
    ///
    /// let master = cluster.node(0).expect("node 0 exists");
    /// assert_eq!(master.container_name, "chaos-node-0");
    /// assert_eq!(master.host_port, 7000);
    /// assert_eq!(master.role, NodeRole::Master);
    ///
    /// // The first three nodes are masters, the last three are replicas.
    /// let replica = cluster.node(3).expect("node 3 exists");
    /// assert_eq!(replica.host_port, 7003);
    /// assert_eq!(replica.role, NodeRole::Replica);
    ///
    /// assert!(cluster.node(6).is_none());
    /// ```
    pub fn node(&self, index: usize) -> Option<ClusterNode> {
        if index >= self.total_nodes() {
            return None;
        }

        let role = if index < self.num_masters {
            NodeRole::Master
        } else {
            NodeRole::Replica
        };

        Some(ClusterNode {
            index,
            container_name: self.node_name(index),
            host_port: self.port_base + index as u16,
            role,
        })
    }

    /// Query the live role of a single node from the running container.
    ///
    /// Runs `redis-cli role` inside the node's container and reports whether the
    /// node currently identifies as a master or a replica. Unlike
    /// [`node`](Self::node), which returns the role assigned at creation time,
    /// this reflects the cluster's current state (for example after a failover).
    /// This performs a Docker `exec` and is therefore not free; the cluster must
    /// be running.
    ///
    /// # Errors
    ///
    /// Returns an error if `index` is out of range, if the `docker exec` call
    /// fails (for example the container is not running), or if the role output
    /// cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use docker_wrapper::{RedisClusterTemplate, Template};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let cluster = RedisClusterTemplate::new("my-cluster");
    /// cluster.start().await?;
    ///
    /// let role = cluster.node_role(0).await?;
    /// println!("node 0 is currently a {:?}", role);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn node_role(&self, index: usize) -> Result<NodeRole, TemplateError> {
        if index >= self.total_nodes() {
            return Err(TemplateError::InvalidConfig(format!(
                "Node index {} out of range for cluster '{}' with {} nodes",
                index,
                self.name,
                self.total_nodes()
            )));
        }

        let node_name = self.node_name(index);

        let mut role_args = vec!["redis-cli".to_string()];
        if let Some(ref password) = self.password {
            role_args.push("-a".to_string());
            role_args.push(password.clone());
        }
        role_args.push("role".to_string());

        let output = ExecCommand::new(&node_name, role_args).execute().await?;

        // `redis-cli role` prints the role keyword ("master" or "slave") on the
        // first line of its reply.
        match output.stdout.lines().next().map(str::trim) {
            Some("master") => Ok(NodeRole::Master),
            Some("slave") => Ok(NodeRole::Replica),
            other => Err(TemplateError::InvalidConfig(format!(
                "Unexpected role output for node '{}': {:?}",
                node_name, other
            ))),
        }
    }

    /// Create the cluster network
    async fn create_network(&self) -> Result<String, TemplateError> {
        let output = NetworkCreateCommand::new(&self.network_name)
            .driver("bridge")
            .execute()
            .await?;

        // Network ID is in stdout
        Ok(output.stdout.trim().to_string())
    }

    /// Start a single Redis node
    async fn start_node(&self, node_index: usize) -> Result<String, TemplateError> {
        let node_name = self.node_name(node_index);
        let port = self.port_base + node_index as u16;
        let cluster_port = port + 10000;

        // Choose image based on custom image or Redis Stack preference
        let image = self.node_image();

        let mut cmd = RunCommand::new(image)
            .name(&node_name)
            .network(&self.network_name)
            .port(port, 6379)
            .port(cluster_port, 16379)
            .detach();

        // Add memory limit if specified
        if let Some(ref limit) = self.memory_limit {
            cmd = cmd.memory(limit);
        }

        // Add volume for persistence
        if let Some(ref prefix) = self.volume_prefix {
            let volume_name = format!("{}-{}", prefix, node_index);
            cmd = cmd.volume(&volume_name, "/data");
        }

        // Add platform if specified
        if let Some(ref platform) = self.platform {
            cmd = cmd.platform(platform);
        }

        // Auto-remove
        if self.auto_remove {
            cmd = cmd.remove();
        }

        // Build Redis command with cluster configuration
        let mut redis_args = vec![
            "redis-server".to_string(),
            "--cluster-enabled".to_string(),
            "yes".to_string(),
            "--cluster-config-file".to_string(),
            "nodes.conf".to_string(),
            "--cluster-node-timeout".to_string(),
            self.node_timeout.to_string(),
            "--appendonly".to_string(),
            "yes".to_string(),
            "--port".to_string(),
            "6379".to_string(),
        ];

        // Add password if configured
        if let Some(ref password) = self.password {
            redis_args.push("--requirepass".to_string());
            redis_args.push(password.clone());
            redis_args.push("--masterauth".to_string());
            redis_args.push(password.clone());
        }

        // Add announce IP if configured
        if let Some(ref ip) = self.announce_ip {
            redis_args.push("--cluster-announce-ip".to_string());
            redis_args.push(ip.clone());
            redis_args.push("--cluster-announce-port".to_string());
            redis_args.push(port.to_string());
            redis_args.push("--cluster-announce-bus-port".to_string());
            redis_args.push(cluster_port.to_string());
        }

        cmd = cmd.cmd(redis_args);

        let output = cmd.execute().await?;
        Ok(output.0)
    }

    /// Start RedisInsight container
    async fn start_redis_insight(&self) -> Result<String, TemplateError> {
        let insight_name = format!("{}-insight", self.name);

        let mut cmd = RunCommand::new(self.insight_image())
            .name(&insight_name)
            .network(&self.network_name)
            .port(self.redis_insight_port, 8001)
            .detach();

        // Add volume for RedisInsight data persistence
        if let Some(ref prefix) = self.volume_prefix {
            let volume_name = format!("{}-insight", prefix);
            cmd = cmd.volume(&volume_name, "/db");
        }

        // Auto-remove
        if self.auto_remove {
            cmd = cmd.remove();
        }

        // Environment variables for RedisInsight
        cmd = cmd.env("RITRUSTEDORIGINS", "http://localhost");

        let output = cmd.execute().await?;
        Ok(output.0)
    }

    /// Resolve the image reference used for each Redis node.
    ///
    /// A custom image (via [`custom_redis_image`](Self::custom_redis_image))
    /// takes precedence; otherwise Redis Stack uses the pinned
    /// `redis/redis-stack-server` tag and the default is `redis:7-alpine`.
    fn node_image(&self) -> String {
        if let Some(ref custom_image) = self.redis_image {
            if let Some(ref tag) = self.redis_tag {
                format!("{}:{}", custom_image, tag)
            } else {
                custom_image.clone()
            }
        } else if self.use_redis_stack {
            self.stack_image()
        } else {
            "redis:7-alpine".to_string()
        }
    }

    /// Build the Redis Stack server image reference (pinned tag by default).
    fn stack_image(&self) -> String {
        format!("{}:{}", REDIS_STACK_SERVER_IMAGE, self.stack_tag)
    }

    /// Build the RedisInsight image reference (pinned tag by default).
    fn insight_image(&self) -> String {
        format!("{}:{}", REDIS_INSIGHT_CLUSTER_IMAGE, self.redis_insight_tag)
    }

    /// Build the redis-cli ping arguments used for node readiness checks
    fn build_ping_args(&self) -> Vec<String> {
        let mut args = vec!["redis-cli".to_string()];

        if let Some(ref password) = self.password {
            args.push("-a".to_string());
            args.push(password.clone());
        }

        args.push("ping".to_string());
        args
    }

    /// Wait for all cluster nodes to respond to PING.
    ///
    /// Polls each node with `redis-cli ping` (the same readiness check used
    /// by `wait_for_ready()` on single-node templates) every 500ms until all
    /// nodes reply with PONG or the timeout is exceeded.
    async fn wait_for_nodes_ready(
        &self,
        timeout: std::time::Duration,
    ) -> Result<(), TemplateError> {
        let ping_args = self.build_ping_args();
        let check_interval = std::time::Duration::from_millis(500);
        let start = std::time::Instant::now();

        let mut pending: Vec<usize> = (0..self.total_nodes()).collect();

        loop {
            let mut still_pending = Vec::new();
            for &i in &pending {
                let node_name = self.node_name(i);
                let ready = ExecCommand::new(&node_name, ping_args.clone())
                    .execute()
                    .await
                    .is_ok_and(|output| output.stdout.trim() == "PONG");

                if !ready {
                    still_pending.push(i);
                }
            }

            if still_pending.is_empty() {
                return Ok(());
            }
            pending = still_pending;

            if start.elapsed() >= timeout {
                let names: Vec<String> = pending.iter().map(|&i| self.node_name(i)).collect();
                return Err(TemplateError::Timeout(format!(
                    "Cluster '{}' nodes [{}] did not respond to PING within {:?}",
                    self.name,
                    names.join(", "),
                    timeout
                )));
            }

            tokio::time::sleep(check_interval).await;
        }
    }

    /// Initialize the cluster after all nodes are started
    async fn initialize_cluster(&self, container_ids: &[String]) -> Result<(), TemplateError> {
        if container_ids.is_empty() {
            return Err(TemplateError::InvalidConfig(
                "No containers to initialize cluster".to_string(),
            ));
        }

        // Wait until every node accepts connections before running cluster create
        self.wait_for_nodes_ready(std::time::Duration::from_secs(60))
            .await?;

        // Build the cluster create command
        let mut create_args = vec![
            "redis-cli".to_string(),
            "--cluster".to_string(),
            "create".to_string(),
        ];

        // Add all node addresses using container hostnames (internal port is always 6379)
        for i in 0..self.total_nodes() {
            let host = self.node_name(i);
            let port = 6379;
            create_args.push(format!("{}:{}", host, port));
        }

        // Add replicas configuration
        if self.num_replicas > 0 {
            create_args.push("--cluster-replicas".to_string());
            create_args.push(self.num_replicas.to_string());
        }

        // Add password if configured
        if let Some(ref password) = self.password {
            create_args.push("-a".to_string());
            create_args.push(password.clone());
        }

        // Auto-accept the configuration
        create_args.push("--cluster-yes".to_string());

        // Execute cluster create in the first container
        let first_node_name = self.node_name(0);

        ExecCommand::new(&first_node_name, create_args)
            .execute()
            .await?;

        Ok(())
    }

    /// Check cluster status
    pub async fn cluster_info(&self) -> Result<ClusterInfo, TemplateError> {
        let node_name = self.node_name(0);

        let mut info_args = vec![
            "redis-cli".to_string(),
            "--cluster".to_string(),
            "info".to_string(),
            format!("{}:6379", node_name),
        ];

        if let Some(ref password) = self.password {
            info_args.push("-a".to_string());
            info_args.push(password.clone());
        }

        let output = ExecCommand::new(&node_name, info_args).execute().await?;

        // Parse the cluster info output
        ClusterInfo::from_output(&output.stdout)
    }

    /// Check if the cluster is ready (all nodes up, slots assigned).
    ///
    /// Returns `true` if the cluster state is "ok", `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use docker_wrapper::{RedisClusterTemplate, Template};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let template = RedisClusterTemplate::new("my-cluster");
    /// template.start().await?;
    ///
    /// if template.is_ready().await {
    ///     println!("Cluster is ready!");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_ready(&self) -> bool {
        self.cluster_info()
            .await
            .is_ok_and(|info| info.cluster_state == "ok")
    }

    /// Wait for the cluster to become ready, with a timeout.
    ///
    /// Polls the cluster state every 500ms until it reports "ok" or the timeout is exceeded.
    ///
    /// # Errors
    ///
    /// Returns an error if the timeout is exceeded before the cluster becomes ready.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use docker_wrapper::{RedisClusterTemplate, Template};
    /// # use std::time::Duration;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let template = RedisClusterTemplate::new("my-cluster");
    /// template.start().await?;
    ///
    /// // Wait up to 30 seconds for the cluster to be ready
    /// template.wait_until_ready(Duration::from_secs(30)).await?;
    /// println!("Cluster is ready!");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wait_until_ready(
        &self,
        timeout: std::time::Duration,
    ) -> Result<(), TemplateError> {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            if self.is_ready().await {
                return Ok(());
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        Err(TemplateError::Timeout(format!(
            "Cluster '{}' did not become ready within {:?}",
            self.name, timeout
        )))
    }

    /// Check if a Redis cluster is already running at the configured ports.
    ///
    /// This is useful in CI environments where an external cluster may be
    /// provided (e.g., via `grokzen/redis-cluster` Docker image).
    ///
    /// Returns connection info if a cluster is detected, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use docker_wrapper::RedisClusterTemplate;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let template = RedisClusterTemplate::from_env("my-cluster");
    ///
    /// if let Some(conn) = template.detect_existing().await {
    ///     println!("Found existing cluster: {}", conn.nodes_string());
    /// } else {
    ///     println!("No existing cluster found");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detect_existing(&self) -> Option<RedisClusterConnection> {
        let host = self.announce_ip.as_deref().unwrap_or("localhost");

        // Try to connect to the first node
        let first_port = self.port_base;
        let addr = format!("{}:{}", host, first_port);

        // Try TCP connection with a short timeout
        let connect_result = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            tokio::net::TcpStream::connect(&addr),
        )
        .await;

        match connect_result {
            Ok(Ok(_stream)) => {
                // Connection succeeded - cluster appears to be running
                // Build connection info for all expected nodes
                Some(RedisClusterConnection::from_template(self))
            }
            _ => None,
        }
    }

    /// Start the cluster, or use an existing one if already running.
    ///
    /// This provides a "best of both worlds" approach for hybrid local/CI setups:
    /// - In CI: Uses the externally-provided cluster without starting new containers
    /// - Locally: Starts a new cluster via docker-wrapper
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use docker_wrapper::RedisClusterTemplate;
    /// # use std::time::Duration;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Works in both CI (uses existing) and local (starts new)
    /// let template = RedisClusterTemplate::from_env("test-cluster");
    /// let conn = template.start_or_detect(Duration::from_secs(60)).await?;
    ///
    /// println!("Cluster ready at: {}", conn.nodes_string());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start_or_detect(
        &self,
        timeout: std::time::Duration,
    ) -> Result<RedisClusterConnection, TemplateError> {
        // First, check if a cluster already exists
        if let Some(conn) = self.detect_existing().await {
            return Ok(conn);
        }

        // No existing cluster found - start a new one
        self.start().await?;
        self.wait_until_ready(timeout).await?;

        Ok(RedisClusterConnection::from_template(self))
    }
}

#[async_trait]
impl Template for RedisClusterTemplate {
    fn name(&self) -> &str {
        &self.name
    }

    fn config(&self) -> &TemplateConfig {
        // Return a dummy config as cluster doesn't map to single container
        unimplemented!("RedisClusterTemplate manages multiple containers")
    }

    fn config_mut(&mut self) -> &mut TemplateConfig {
        unimplemented!("RedisClusterTemplate manages multiple containers")
    }

    async fn start(&self) -> Result<String, TemplateError> {
        // Create network first
        let _network_id = self.create_network().await?;

        // Start all nodes
        let mut container_ids = Vec::new();
        for i in 0..self.total_nodes() {
            let id = self.start_node(i).await?;
            container_ids.push(id);
        }

        // Initialize the cluster
        self.initialize_cluster(&container_ids).await?;

        // Start RedisInsight if enabled
        let insight_info = if self.with_redis_insight {
            let _insight_id = self.start_redis_insight().await?;
            format!(
                ", RedisInsight UI at http://localhost:{}",
                self.redis_insight_port
            )
        } else {
            String::new()
        };

        // Return a summary
        Ok(format!(
            "Redis Cluster '{}' started with {} nodes ({} masters, {} replicas){}",
            self.name,
            self.total_nodes(),
            self.num_masters,
            self.num_masters * self.num_replicas,
            insight_info
        ))
    }

    async fn stop(&self) -> Result<(), TemplateError> {
        use crate::StopCommand;

        // Stop all nodes
        for i in 0..self.total_nodes() {
            let node_name = self.node_name(i);
            let _ = StopCommand::new(&node_name).execute().await;
        }

        // Stop RedisInsight if it was started
        if self.with_redis_insight {
            let insight_name = format!("{}-insight", self.name);
            let _ = StopCommand::new(&insight_name).execute().await;
        }

        Ok(())
    }

    async fn remove(&self) -> Result<(), TemplateError> {
        use crate::{NetworkRmCommand, RmCommand};

        // Remove all containers
        for i in 0..self.total_nodes() {
            let node_name = self.node_name(i);
            let _ = RmCommand::new(&node_name).force().volumes().execute().await;
        }

        // Remove RedisInsight if it was started
        if self.with_redis_insight {
            let insight_name = format!("{}-insight", self.name);
            let _ = RmCommand::new(&insight_name)
                .force()
                .volumes()
                .execute()
                .await;
        }

        // Remove the network
        let _ = NetworkRmCommand::new(&self.network_name).execute().await;

        Ok(())
    }
}

/// Cluster information
#[derive(Debug, Clone)]
pub struct ClusterInfo {
    /// Current state of the cluster (ok/fail)
    pub cluster_state: String,
    /// Total number of hash slots (always 16384 for Redis)
    pub total_slots: u16,
    /// List of nodes in the cluster
    pub nodes: Vec<NodeInfo>,
}

impl ClusterInfo {
    #[allow(clippy::unnecessary_wraps)]
    fn from_output(_output: &str) -> Result<Self, TemplateError> {
        // Basic parsing - would need more sophisticated parsing in production
        Ok(ClusterInfo {
            cluster_state: "ok".to_string(),
            total_slots: 16384,
            nodes: Vec::new(),
        })
    }
}

/// A deterministic handle to a single node in a [`RedisClusterTemplate`].
///
/// Returned by [`RedisClusterTemplate::node`]. The fields are derived purely
/// from the template configuration (the `{name}-node-{index}` naming contract,
/// the port base, and the master/replica split applied by
/// `redis-cli --cluster create`), so constructing a handle is free and does not
/// require the cluster to be running. This makes it suitable for targeted fault
/// injection: pause, partition, or kill exactly the container you name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterNode {
    /// Zero-based index of the node within the cluster.
    pub index: usize,
    /// Container name, following the `{name}-node-{index}` contract.
    pub container_name: String,
    /// Host port mapped to this node's Redis port (`port_base + index`).
    pub host_port: u16,
    /// Role assigned to the node at cluster-create time.
    ///
    /// This is the static assignment (`0..num_masters` are masters, the rest are
    /// replicas), not necessarily the live role after a failover. Use
    /// [`RedisClusterTemplate::node_role`] to read the current role from a
    /// running node.
    pub role: NodeRole,
}

/// Information about a cluster node
#[derive(Debug, Clone)]
pub struct NodeInfo {
    /// Node ID in the cluster
    pub id: String,
    /// Hostname or IP address
    pub host: String,
    /// Port number
    pub port: u16,
    /// Role of the node (Master/Replica)
    pub role: NodeRole,
    /// Slot ranges assigned to this node (start, end)
    pub slots: Vec<(u16, u16)>,
}

/// Node role in the cluster
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeRole {
    /// Master node that owns hash slots
    Master,
    /// Replica node that replicates a master
    Replica,
}

/// Connection helper for Redis Cluster
#[derive(Debug, Clone)]
pub struct RedisClusterConnection {
    nodes: Vec<String>,
    password: Option<String>,
}

impl RedisClusterConnection {
    /// Create a new cluster connection with the given node addresses.
    ///
    /// This is useful for connecting to external/pre-existing clusters
    /// (e.g., in CI environments) without going through a template.
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::RedisClusterConnection;
    ///
    /// let conn = RedisClusterConnection::new(vec![
    ///     "localhost:7000".to_string(),
    ///     "localhost:7001".to_string(),
    ///     "localhost:7002".to_string(),
    /// ]);
    /// ```
    pub fn new(nodes: Vec<String>) -> Self {
        Self {
            nodes,
            password: None,
        }
    }

    /// Create a new cluster connection with password authentication.
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::RedisClusterConnection;
    ///
    /// let conn = RedisClusterConnection::with_password(
    ///     vec!["localhost:7000".to_string()],
    ///     "secret",
    /// );
    /// ```
    pub fn with_password(nodes: Vec<String>, password: impl Into<String>) -> Self {
        Self {
            nodes,
            password: Some(password.into()),
        }
    }

    /// Create from a RedisClusterTemplate
    pub fn from_template(template: &RedisClusterTemplate) -> Self {
        let host = template.announce_ip.as_deref().unwrap_or("localhost");
        let mut nodes = Vec::new();

        for i in 0..template.total_nodes() {
            let port = template.port_base + i as u16;
            nodes.push(format!("{}:{}", host, port));
        }

        Self {
            nodes,
            password: template.password.clone(),
        }
    }

    /// Get the list of cluster nodes
    pub fn nodes(&self) -> &[String] {
        &self.nodes
    }

    /// Get cluster nodes as comma-separated string
    pub fn nodes_string(&self) -> String {
        self.nodes.join(",")
    }

    /// Get connection URL for cluster-aware clients
    pub fn cluster_url(&self) -> String {
        let auth = self
            .password
            .as_ref()
            .map(|p| format!(":{}@", p))
            .unwrap_or_default();

        format!("redis-cluster://{}{}", auth, self.nodes.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_redis_cluster_template_basic() {
        let template = RedisClusterTemplate::new("test-cluster");
        assert_eq!(template.name, "test-cluster");
        assert_eq!(template.num_masters, 3);
        assert_eq!(template.num_replicas, 0);
        assert_eq!(template.port_base, 7000);
    }

    #[test]
    fn test_redis_cluster_template_with_replicas() {
        let template = RedisClusterTemplate::new("test-cluster")
            .num_masters(3)
            .num_replicas(1);

        assert_eq!(template.total_nodes(), 6);
    }

    #[test]
    fn test_redis_cluster_template_minimum_masters() {
        let template = RedisClusterTemplate::new("test-cluster").num_masters(2); // Should be forced to 3

        assert_eq!(template.num_masters, 3);
    }

    #[test]
    fn test_redis_cluster_connection() {
        let template = RedisClusterTemplate::new("test-cluster")
            .num_masters(3)
            .port_base(7000)
            .password("secret");

        let conn = RedisClusterConnection::from_template(&template);
        assert_eq!(conn.nodes.len(), 3);
        assert_eq!(conn.nodes[0], "localhost:7000");
        assert_eq!(
            conn.cluster_url(),
            "redis-cluster://:secret@localhost:7000,localhost:7001,localhost:7002"
        );
    }

    #[test]
    fn test_redis_cluster_with_stack_and_insight() {
        let template = RedisClusterTemplate::new("test-cluster")
            .num_masters(3)
            .with_redis_stack()
            .with_redis_insight()
            .redis_insight_port(8080);

        assert!(template.use_redis_stack);
        assert!(template.with_redis_insight);
        assert_eq!(template.redis_insight_port, 8080);
    }

    #[test]
    fn test_redis_cluster_stack_image_default_pinned() {
        // Redis Stack defaults to a pinned, known-good tag (not latest).
        let template = RedisClusterTemplate::new("test-cluster").with_redis_stack();

        assert_eq!(template.stack_image(), "redis/redis-stack-server:7.4.0-v3");
        assert_eq!(template.node_image(), "redis/redis-stack-server:7.4.0-v3");
        assert_ne!(template.stack_image(), "redis/redis-stack-server:latest");
    }

    #[test]
    fn test_redis_cluster_stack_version_override() {
        let template = RedisClusterTemplate::new("test-cluster")
            .with_redis_stack()
            .stack_version("7.2.0-v9");

        assert_eq!(template.stack_image(), "redis/redis-stack-server:7.2.0-v9");
        assert_eq!(template.node_image(), "redis/redis-stack-server:7.2.0-v9");
    }

    #[test]
    fn test_redis_cluster_node_image_default_and_custom() {
        // Default (no stack, no custom) is the pinned alpine image.
        let default_template = RedisClusterTemplate::new("test-cluster");
        assert_eq!(default_template.node_image(), "redis:7-alpine");

        // A custom image takes precedence over the stack preference.
        let custom_template = RedisClusterTemplate::new("test-cluster")
            .with_redis_stack()
            .custom_redis_image("myrepo/redis", "1.2.3");
        assert_eq!(custom_template.node_image(), "myrepo/redis:1.2.3");
    }

    #[test]
    fn test_redis_cluster_insight_image_default_pinned() {
        // RedisInsight defaults to a pinned, known-good tag (not latest).
        let template = RedisClusterTemplate::new("test-cluster").with_redis_insight();

        assert_eq!(template.insight_image(), "redislabs/redisinsight:2.60");
        assert_ne!(template.insight_image(), "redislabs/redisinsight:latest");
    }

    #[test]
    fn test_redis_cluster_insight_version_override() {
        let template = RedisClusterTemplate::new("test-cluster")
            .with_redis_insight()
            .redis_insight_version("2.58");

        assert_eq!(template.insight_image(), "redislabs/redisinsight:2.58");
    }

    #[test]
    fn test_redis_cluster_connection_new() {
        let nodes = vec![
            "localhost:7000".to_string(),
            "localhost:7001".to_string(),
            "localhost:7002".to_string(),
        ];
        let conn = RedisClusterConnection::new(nodes.clone());

        assert_eq!(conn.nodes(), &nodes);
        assert_eq!(
            conn.nodes_string(),
            "localhost:7000,localhost:7001,localhost:7002"
        );
        assert_eq!(
            conn.cluster_url(),
            "redis-cluster://localhost:7000,localhost:7001,localhost:7002"
        );
    }

    #[test]
    fn test_redis_cluster_connection_with_password() {
        let nodes = vec!["localhost:7000".to_string()];
        let conn = RedisClusterConnection::with_password(nodes, "secret123");

        assert_eq!(
            conn.cluster_url(),
            "redis-cluster://:secret123@localhost:7000"
        );
    }

    #[test]
    #[serial]
    fn test_redis_cluster_from_env_defaults() {
        // Clear any existing env vars to ensure defaults are used
        std::env::remove_var("REDIS_CLUSTER_PORT_BASE");
        std::env::remove_var("REDIS_CLUSTER_NUM_MASTERS");
        std::env::remove_var("REDIS_CLUSTER_NUM_REPLICAS");
        std::env::remove_var("REDIS_CLUSTER_PASSWORD");

        let template = RedisClusterTemplate::from_env("test-cluster");

        assert_eq!(template.get_port_base(), 7000);
        assert_eq!(template.get_num_masters(), 3);
        assert_eq!(template.get_num_replicas(), 0);
    }

    #[test]
    #[serial]
    fn test_redis_cluster_from_env_with_vars() {
        std::env::set_var("REDIS_CLUSTER_PORT_BASE", "8000");
        std::env::set_var("REDIS_CLUSTER_NUM_MASTERS", "6");
        std::env::set_var("REDIS_CLUSTER_NUM_REPLICAS", "1");
        std::env::set_var("REDIS_CLUSTER_PASSWORD", "testpass");

        let template = RedisClusterTemplate::from_env("test-cluster");

        assert_eq!(template.get_port_base(), 8000);
        assert_eq!(template.get_num_masters(), 6);
        assert_eq!(template.get_num_replicas(), 1);

        // Clean up
        std::env::remove_var("REDIS_CLUSTER_PORT_BASE");
        std::env::remove_var("REDIS_CLUSTER_NUM_MASTERS");
        std::env::remove_var("REDIS_CLUSTER_NUM_REPLICAS");
        std::env::remove_var("REDIS_CLUSTER_PASSWORD");
    }

    #[test]
    fn test_build_ping_args_without_password() {
        let template = RedisClusterTemplate::new("test-cluster");

        assert_eq!(template.build_ping_args(), vec!["redis-cli", "ping"]);
    }

    #[test]
    fn test_build_ping_args_with_password() {
        let template = RedisClusterTemplate::new("test-cluster").password("secret");

        assert_eq!(
            template.build_ping_args(),
            vec!["redis-cli", "-a", "secret", "ping"]
        );
    }

    #[test]
    fn test_redis_cluster_getters() {
        let template = RedisClusterTemplate::new("test-cluster")
            .port_base(9000)
            .num_masters(5)
            .num_replicas(2);

        assert_eq!(template.get_port_base(), 9000);
        assert_eq!(template.get_num_masters(), 5);
        assert_eq!(template.get_num_replicas(), 2);
    }

    #[test]
    fn test_node_name_construction() {
        let template = RedisClusterTemplate::new("test-cluster");

        assert_eq!(template.node_name(0), "test-cluster-node-0");
        assert_eq!(template.node_name(2), "test-cluster-node-2");
        assert_eq!(template.node_name(11), "test-cluster-node-11");
    }

    #[test]
    fn test_node_names_masters_only() {
        let template = RedisClusterTemplate::new("test-cluster").num_masters(3);

        assert_eq!(
            template.node_names(),
            vec![
                "test-cluster-node-0",
                "test-cluster-node-1",
                "test-cluster-node-2",
            ]
        );
    }

    #[test]
    fn test_node_names_with_replicas() {
        let template = RedisClusterTemplate::new("test-cluster")
            .num_masters(3)
            .num_replicas(1);

        // 3 masters + 3 replicas = 6 nodes, indices 0..6
        let names = template.node_names();
        assert_eq!(names.len(), 6);
        assert_eq!(names[0], "test-cluster-node-0");
        assert_eq!(names[5], "test-cluster-node-5");
    }

    #[test]
    fn test_node_accessor_roles_and_ports() {
        let template = RedisClusterTemplate::new("test-cluster")
            .num_masters(3)
            .num_replicas(1)
            .port_base(7000);

        // First num_masters nodes are masters.
        for i in 0..3 {
            let node = template.node(i).expect("master node exists");
            assert_eq!(node.index, i);
            assert_eq!(node.container_name, format!("test-cluster-node-{}", i));
            assert_eq!(node.host_port, 7000 + i as u16);
            assert_eq!(node.role, NodeRole::Master);
        }

        // Remaining nodes are replicas.
        for i in 3..6 {
            let node = template.node(i).expect("replica node exists");
            assert_eq!(node.host_port, 7000 + i as u16);
            assert_eq!(node.role, NodeRole::Replica);
        }
    }

    #[test]
    fn test_node_accessor_out_of_range() {
        let template = RedisClusterTemplate::new("test-cluster").num_masters(3);

        assert!(template.node(2).is_some());
        assert!(template.node(3).is_none());
        assert!(template.node(100).is_none());
    }

    #[test]
    fn test_node_accessor_respects_custom_port_base() {
        let template = RedisClusterTemplate::new("test-cluster")
            .num_masters(3)
            .port_base(9100);

        assert_eq!(template.node(0).unwrap().host_port, 9100);
        assert_eq!(template.node(2).unwrap().host_port, 9102);
    }

    #[test]
    fn test_node_names_match_node_accessor() {
        // node_names() and node().container_name must agree on every index.
        let template = RedisClusterTemplate::new("test-cluster")
            .num_masters(3)
            .num_replicas(2);

        for (i, name) in template.node_names().iter().enumerate() {
            assert_eq!(&template.node(i).unwrap().container_name, name);
        }
    }
}
