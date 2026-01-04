//! Docker swarm init command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Result of swarm init command
#[derive(Debug, Clone)]
pub struct SwarmInitResult {
    /// The node ID of the manager node
    pub node_id: Option<String>,
    /// The worker join token
    pub worker_token: Option<String>,
    /// The manager join token
    pub manager_token: Option<String>,
    /// Raw output from the command
    pub output: String,
}

impl SwarmInitResult {
    /// Parse the swarm init output
    fn parse(output: &CommandOutput) -> Self {
        let stdout = &output.stdout;

        // Try to extract tokens from the output
        // Output format:
        // Swarm initialized: current node (nodeId) is now a manager.
        // To add a worker to this swarm, run the following command:
        //     docker swarm join --token <token> <ip>:<port>

        let mut node_id = None;
        let mut worker_token = None;
        let mut manager_token = None;

        for line in stdout.lines() {
            if line.contains("current node") && line.contains("is now a manager") {
                // Extract node ID from parentheses
                if let Some(start) = line.find('(') {
                    if let Some(end) = line.find(')') {
                        node_id = Some(line[start + 1..end].to_string());
                    }
                }
            }

            if line.contains("--token") {
                // Extract token from the join command
                let parts: Vec<&str> = line.split_whitespace().collect();
                for (i, part) in parts.iter().enumerate() {
                    if *part == "--token" {
                        if let Some(token) = parts.get(i + 1) {
                            // Determine if this is worker or manager token based on context
                            if stdout.contains("add a worker") && worker_token.is_none() {
                                worker_token = Some((*token).to_string());
                            } else if stdout.contains("add a manager") {
                                manager_token = Some((*token).to_string());
                            }
                        }
                    }
                }
            }
        }

        Self {
            node_id,
            worker_token,
            manager_token,
            output: stdout.clone(),
        }
    }
}

/// Docker swarm init command builder
#[derive(Debug, Clone, Default)]
pub struct SwarmInitCommand {
    /// Advertised address (format: <ip|interface>[:port])
    advertise_addr: Option<String>,
    /// Enable manager auto-lock
    autolock: bool,
    /// Availability of the node (active, pause, drain)
    availability: Option<String>,
    /// Validity period for node certificates (ns|us|ms|s|m|h)
    cert_expiry: Option<String>,
    /// Address or interface to use for data path traffic
    data_path_addr: Option<String>,
    /// Port to use for data path traffic
    data_path_port: Option<u16>,
    /// Default address pool in CIDR format
    default_addr_pool: Vec<String>,
    /// Default address pool subnet mask length
    default_addr_pool_mask_length: Option<u8>,
    /// Dispatcher heartbeat period (ns|us|ms|s|m|h)
    dispatcher_heartbeat: Option<String>,
    /// External CA URL (format: `protocol://url`)
    external_ca: Option<String>,
    /// Force create a new cluster from current state
    force_new_cluster: bool,
    /// Listen address (format: <ip|interface>[:port])
    listen_addr: Option<String>,
    /// Number of snapshots to keep beyond current snapshot
    max_snapshots: Option<u32>,
    /// Log entries to keep after compacting raft log
    snapshot_interval: Option<u32>,
    /// Task history retention limit
    task_history_limit: Option<i32>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl SwarmInitCommand {
    /// Create a new swarm init command
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the advertised address
    #[must_use]
    pub fn advertise_addr(mut self, addr: impl Into<String>) -> Self {
        self.advertise_addr = Some(addr.into());
        self
    }

    /// Enable manager auto-lock
    #[must_use]
    pub fn autolock(mut self) -> Self {
        self.autolock = true;
        self
    }

    /// Set the availability of the node
    #[must_use]
    pub fn availability(mut self, availability: impl Into<String>) -> Self {
        self.availability = Some(availability.into());
        self
    }

    /// Set the certificate expiry duration
    #[must_use]
    pub fn cert_expiry(mut self, expiry: impl Into<String>) -> Self {
        self.cert_expiry = Some(expiry.into());
        self
    }

    /// Set the data path address
    #[must_use]
    pub fn data_path_addr(mut self, addr: impl Into<String>) -> Self {
        self.data_path_addr = Some(addr.into());
        self
    }

    /// Set the data path port
    #[must_use]
    pub fn data_path_port(mut self, port: u16) -> Self {
        self.data_path_port = Some(port);
        self
    }

    /// Add a default address pool
    #[must_use]
    pub fn default_addr_pool(mut self, pool: impl Into<String>) -> Self {
        self.default_addr_pool.push(pool.into());
        self
    }

    /// Set the default address pool mask length
    #[must_use]
    pub fn default_addr_pool_mask_length(mut self, length: u8) -> Self {
        self.default_addr_pool_mask_length = Some(length);
        self
    }

    /// Set the dispatcher heartbeat period
    #[must_use]
    pub fn dispatcher_heartbeat(mut self, heartbeat: impl Into<String>) -> Self {
        self.dispatcher_heartbeat = Some(heartbeat.into());
        self
    }

    /// Set the external CA URL
    #[must_use]
    pub fn external_ca(mut self, url: impl Into<String>) -> Self {
        self.external_ca = Some(url.into());
        self
    }

    /// Force create a new cluster from current state
    #[must_use]
    pub fn force_new_cluster(mut self) -> Self {
        self.force_new_cluster = true;
        self
    }

    /// Set the listen address
    #[must_use]
    pub fn listen_addr(mut self, addr: impl Into<String>) -> Self {
        self.listen_addr = Some(addr.into());
        self
    }

    /// Set the maximum number of snapshots to keep
    #[must_use]
    pub fn max_snapshots(mut self, count: u32) -> Self {
        self.max_snapshots = Some(count);
        self
    }

    /// Set the snapshot interval
    #[must_use]
    pub fn snapshot_interval(mut self, interval: u32) -> Self {
        self.snapshot_interval = Some(interval);
        self
    }

    /// Set the task history retention limit
    #[must_use]
    pub fn task_history_limit(mut self, limit: i32) -> Self {
        self.task_history_limit = Some(limit);
        self
    }

    /// Build the command arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["swarm".to_string(), "init".to_string()];

        if let Some(ref addr) = self.advertise_addr {
            args.push("--advertise-addr".to_string());
            args.push(addr.clone());
        }

        if self.autolock {
            args.push("--autolock".to_string());
        }

        if let Some(ref availability) = self.availability {
            args.push("--availability".to_string());
            args.push(availability.clone());
        }

        if let Some(ref expiry) = self.cert_expiry {
            args.push("--cert-expiry".to_string());
            args.push(expiry.clone());
        }

        if let Some(ref addr) = self.data_path_addr {
            args.push("--data-path-addr".to_string());
            args.push(addr.clone());
        }

        if let Some(port) = self.data_path_port {
            args.push("--data-path-port".to_string());
            args.push(port.to_string());
        }

        for pool in &self.default_addr_pool {
            args.push("--default-addr-pool".to_string());
            args.push(pool.clone());
        }

        if let Some(length) = self.default_addr_pool_mask_length {
            args.push("--default-addr-pool-mask-length".to_string());
            args.push(length.to_string());
        }

        if let Some(ref heartbeat) = self.dispatcher_heartbeat {
            args.push("--dispatcher-heartbeat".to_string());
            args.push(heartbeat.clone());
        }

        if let Some(ref url) = self.external_ca {
            args.push("--external-ca".to_string());
            args.push(url.clone());
        }

        if self.force_new_cluster {
            args.push("--force-new-cluster".to_string());
        }

        if let Some(ref addr) = self.listen_addr {
            args.push("--listen-addr".to_string());
            args.push(addr.clone());
        }

        if let Some(count) = self.max_snapshots {
            args.push("--max-snapshots".to_string());
            args.push(count.to_string());
        }

        if let Some(interval) = self.snapshot_interval {
            args.push("--snapshot-interval".to_string());
            args.push(interval.to_string());
        }

        if let Some(limit) = self.task_history_limit {
            args.push("--task-history-limit".to_string());
            args.push(limit.to_string());
        }

        args
    }
}

#[async_trait]
impl DockerCommand for SwarmInitCommand {
    type Output = SwarmInitResult;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        self.build_args()
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_args();
        let output = self.execute_command(args).await?;
        Ok(SwarmInitResult::parse(&output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_init_basic() {
        let cmd = SwarmInitCommand::new();
        let args = cmd.build_args();
        assert_eq!(args, vec!["swarm", "init"]);
    }

    #[test]
    fn test_swarm_init_with_advertise_addr() {
        let cmd = SwarmInitCommand::new().advertise_addr("192.168.1.1:2377");
        let args = cmd.build_args();
        assert!(args.contains(&"--advertise-addr".to_string()));
        assert!(args.contains(&"192.168.1.1:2377".to_string()));
    }

    #[test]
    fn test_swarm_init_with_autolock() {
        let cmd = SwarmInitCommand::new().autolock();
        let args = cmd.build_args();
        assert!(args.contains(&"--autolock".to_string()));
    }

    #[test]
    fn test_swarm_init_all_options() {
        let cmd = SwarmInitCommand::new()
            .advertise_addr("192.168.1.1:2377")
            .autolock()
            .availability("active")
            .cert_expiry("90d")
            .data_path_addr("192.168.1.1")
            .data_path_port(4789)
            .default_addr_pool("10.10.0.0/16")
            .default_addr_pool_mask_length(24)
            .dispatcher_heartbeat("5s")
            .force_new_cluster()
            .listen_addr("0.0.0.0:2377")
            .max_snapshots(5)
            .snapshot_interval(10000)
            .task_history_limit(10);

        let args = cmd.build_args();
        assert!(args.contains(&"--advertise-addr".to_string()));
        assert!(args.contains(&"--autolock".to_string()));
        assert!(args.contains(&"--availability".to_string()));
        assert!(args.contains(&"--cert-expiry".to_string()));
        assert!(args.contains(&"--data-path-addr".to_string()));
        assert!(args.contains(&"--data-path-port".to_string()));
        assert!(args.contains(&"--default-addr-pool".to_string()));
        assert!(args.contains(&"--default-addr-pool-mask-length".to_string()));
        assert!(args.contains(&"--dispatcher-heartbeat".to_string()));
        assert!(args.contains(&"--force-new-cluster".to_string()));
        assert!(args.contains(&"--listen-addr".to_string()));
        assert!(args.contains(&"--max-snapshots".to_string()));
        assert!(args.contains(&"--snapshot-interval".to_string()));
        assert!(args.contains(&"--task-history-limit".to_string()));
    }
}
