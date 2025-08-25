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
use crate::{DockerCommand, NetworkCreateCommand, RunCommand};

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

    /// Start the Redis Sentinel cluster
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network creation fails
    /// - Starting any container (master, replica, or sentinel) fails
    pub async fn start(self) -> Result<SentinelConnectionInfo, crate::Error> {
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
        let mut master_cmd = self.build_redis_command(&master_name, self.master_port, None);
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
            let replica_port = self.replica_port_base + u16::try_from(i).unwrap_or(0);

            let mut replica_cmd =
                self.build_redis_command(&replica_name, replica_port, Some(&master_name));
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
            let sentinel_port = self.sentinel_port_base + u16::try_from(i).unwrap_or(0);

            let mut sentinel_cmd =
                Self::build_sentinel_command(&sentinel_name, sentinel_port, &sentinel_config);
            sentinel_cmd = sentinel_cmd.network(&network_name);

            sentinel_cmd
                .execute()
                .await
                .map_err(|e| crate::Error::Custom {
                    message: format!("Failed to start sentinel {}: {e}", i + 1),
                })?;

            sentinel_containers.push((sentinel_name, sentinel_port));
        }

        Ok(SentinelConnectionInfo {
            name: self.name.clone(),
            master_name: self.master_name.clone(),
            master_host: "localhost".to_string(),
            master_port: self.master_port,
            replica_ports: (0..self.num_replicas)
                .map(|i| self.replica_port_base + u16::try_from(i).unwrap_or(0))
                .collect(),
            sentinels: sentinel_containers
                .into_iter()
                .map(|(_, port)| SentinelInfo {
                    host: "localhost".to_string(),
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
    fn build_redis_command(&self, name: &str, port: u16, master: Option<&str>) -> RunCommand {
        let mut cmd = RunCommand::new(format!("{DEFAULT_REDIS_IMAGE}:{DEFAULT_REDIS_TAG}"))
            .name(name)
            .port(port, 6379)
            .detach();

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

        if !args.is_empty() {
            cmd = cmd.entrypoint("redis-server").cmd(args);
        }

        cmd
    }

    /// Build Sentinel command
    fn build_sentinel_command(name: &str, port: u16, config: &str) -> RunCommand {
        let mut cmd = RunCommand::new(format!("{DEFAULT_REDIS_IMAGE}:{DEFAULT_REDIS_TAG}"))
            .name(name)
            .port(port, 26379)
            .detach();

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
        config.push(format!(
            "sentinel monitor {} {} 6379 {}",
            self.master_name, master_container, self.quorum
        ));

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
}
