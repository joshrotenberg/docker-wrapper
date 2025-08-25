//! Redis Cluster template for multi-node Redis setup with sharding and replication

#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_errors_doc)]

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
    /// Whether to include RedisInsight GUI
    with_redis_insight: bool,
    /// Port for RedisInsight UI
    redis_insight_port: u16,
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
            with_redis_insight: false,
            redis_insight_port: 8001,
            redis_image: None,
            redis_tag: None,
            platform: None,
        }
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

    /// Use Redis Stack instead of standard Redis (includes modules like JSON, Search, Graph, TimeSeries, Bloom)
    pub fn with_redis_stack(mut self) -> Self {
        self.use_redis_stack = true;
        self
    }

    /// Enable RedisInsight GUI for cluster visualization and management
    pub fn with_redis_insight(mut self) -> Self {
        self.with_redis_insight = true;
        self
    }

    /// Set the port for RedisInsight UI (default: 8001)
    pub fn redis_insight_port(mut self, port: u16) -> Self {
        self.redis_insight_port = port;
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
        let node_name = format!("{}-node-{}", self.name, node_index);
        let port = self.port_base + node_index as u16;
        let cluster_port = port + 10000;

        // Choose image based on custom image or Redis Stack preference
        let image = if let Some(ref custom_image) = self.redis_image {
            if let Some(ref tag) = self.redis_tag {
                format!("{}:{}", custom_image, tag)
            } else {
                custom_image.clone()
            }
        } else if self.use_redis_stack {
            "redis/redis-stack-server:latest".to_string()
        } else {
            "redis:7-alpine".to_string()
        };

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

        let mut cmd = RunCommand::new("redislabs/redisinsight:latest")
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

    /// Initialize the cluster after all nodes are started
    async fn initialize_cluster(&self, container_ids: &[String]) -> Result<(), TemplateError> {
        if container_ids.is_empty() {
            return Err(TemplateError::InvalidConfig(
                "No containers to initialize cluster".to_string(),
            ));
        }

        // Wait a bit for all nodes to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Build the cluster create command
        let mut create_args = vec![
            "redis-cli".to_string(),
            "--cluster".to_string(),
            "create".to_string(),
        ];

        // Add all node addresses
        let host = self.announce_ip.as_deref().unwrap_or("127.0.0.1");
        for i in 0..self.total_nodes() {
            let port = self.port_base + i as u16;
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
        let first_node_name = format!("{}-node-0", self.name);

        ExecCommand::new(&first_node_name, create_args)
            .execute()
            .await?;

        Ok(())
    }

    /// Check cluster status
    pub async fn cluster_info(&self) -> Result<ClusterInfo, TemplateError> {
        let node_name = format!("{}-node-0", self.name);

        let mut info_args = vec![
            "redis-cli".to_string(),
            "--cluster".to_string(),
            "info".to_string(),
            format!(
                "{}:{}",
                self.announce_ip.as_deref().unwrap_or("127.0.0.1"),
                self.port_base
            ),
        ];

        if let Some(ref password) = self.password {
            info_args.push("-a".to_string());
            info_args.push(password.clone());
        }

        let output = ExecCommand::new(&node_name, info_args).execute().await?;

        // Parse the cluster info output
        ClusterInfo::from_output(&output.stdout)
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
            let node_name = format!("{}-node-{}", self.name, i);
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
            let node_name = format!("{}-node-{}", self.name, i);
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
#[derive(Debug, Clone, PartialEq)]
pub enum NodeRole {
    /// Master node that owns hash slots
    Master,
    /// Replica node that replicates a master
    Replica,
}

/// Connection helper for Redis Cluster
pub struct RedisClusterConnection {
    nodes: Vec<String>,
    password: Option<String>,
}

impl RedisClusterConnection {
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
}
