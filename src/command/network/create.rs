//! Docker network create command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommandV2};
use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// Docker network create command builder
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct NetworkCreateCommand {
    /// Network name
    name: String,
    /// Network driver
    driver: Option<String>,
    /// Driver specific options
    driver_opts: HashMap<String, String>,
    /// Subnet in CIDR format
    subnet: Option<String>,
    /// IP range in CIDR format
    ip_range: Option<String>,
    /// Gateway IP address
    gateway: Option<String>,
    /// IPv6 network
    ipv6: bool,
    /// Enable manual container attachment
    attachable: bool,
    /// Restrict external access to the network
    internal: bool,
    /// Network labels
    labels: HashMap<String, String>,
    /// Scope (local, swarm, global)
    scope: Option<String>,
    /// Config from existing network
    config_from: Option<String>,
    /// Config only (don't create)
    config_only: bool,
    /// Ingress network
    ingress: bool,
    /// Auxiliary IPv4 or IPv6 addresses
    aux_addresses: HashMap<String, String>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl NetworkCreateCommand {
    /// Create a new network create command
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            driver: None,
            driver_opts: HashMap::new(),
            subnet: None,
            ip_range: None,
            gateway: None,
            ipv6: false,
            attachable: false,
            internal: false,
            labels: HashMap::new(),
            scope: None,
            config_from: None,
            config_only: false,
            ingress: false,
            aux_addresses: HashMap::new(),
            executor: CommandExecutor::new(),
        }
    }

    /// Set the network driver (bridge, overlay, macvlan, none, etc.)
    #[must_use]
    pub fn driver(mut self, driver: impl Into<String>) -> Self {
        self.driver = Some(driver.into());
        self
    }

    /// Add a driver specific option
    #[must_use]
    pub fn driver_opt(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.driver_opts.insert(key.into(), value.into());
        self
    }

    /// Set the subnet in CIDR format
    #[must_use]
    pub fn subnet(mut self, subnet: impl Into<String>) -> Self {
        self.subnet = Some(subnet.into());
        self
    }

    /// Set the IP range in CIDR format
    #[must_use]
    pub fn ip_range(mut self, range: impl Into<String>) -> Self {
        self.ip_range = Some(range.into());
        self
    }

    /// Set the gateway IP address
    #[must_use]
    pub fn gateway(mut self, gateway: impl Into<String>) -> Self {
        self.gateway = Some(gateway.into());
        self
    }

    /// Enable IPv6 networking
    #[must_use]
    pub fn ipv6(mut self) -> Self {
        self.ipv6 = true;
        self
    }

    /// Enable manual container attachment
    #[must_use]
    pub fn attachable(mut self) -> Self {
        self.attachable = true;
        self
    }

    /// Restrict external access to the network
    #[must_use]
    pub fn internal(mut self) -> Self {
        self.internal = true;
        self
    }

    /// Add a network label
    #[must_use]
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Set network scope
    #[must_use]
    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// Create network from existing config
    #[must_use]
    pub fn config_from(mut self, network: impl Into<String>) -> Self {
        self.config_from = Some(network.into());
        self
    }

    /// Config only (don't create network)
    #[must_use]
    pub fn config_only(mut self) -> Self {
        self.config_only = true;
        self
    }

    /// Create an ingress network
    #[must_use]
    pub fn ingress(mut self) -> Self {
        self.ingress = true;
        self
    }

    /// Add auxiliary address
    #[must_use]
    pub fn aux_address(mut self, name: impl Into<String>, ip: impl Into<String>) -> Self {
        self.aux_addresses.insert(name.into(), ip.into());
        self
    }

    /// Execute the command
    ///
    /// # Errors
    ///
    /// Returns an error if the Docker daemon is not running or the command fails
    pub async fn run(&self) -> Result<NetworkCreateResult> {
        self.execute().await.map(NetworkCreateResult::from)
    }
}

#[async_trait]
impl DockerCommandV2 for NetworkCreateCommand {
    type Output = CommandOutput;

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["network".to_string(), "create".to_string()];

        if let Some(ref driver) = self.driver {
            args.push("--driver".to_string());
            args.push(driver.clone());
        }

        for (key, value) in &self.driver_opts {
            args.push("--opt".to_string());
            args.push(format!("{key}={value}"));
        }

        if let Some(ref subnet) = self.subnet {
            args.push("--subnet".to_string());
            args.push(subnet.clone());
        }

        if let Some(ref ip_range) = self.ip_range {
            args.push("--ip-range".to_string());
            args.push(ip_range.clone());
        }

        if let Some(ref gateway) = self.gateway {
            args.push("--gateway".to_string());
            args.push(gateway.clone());
        }

        if self.ipv6 {
            args.push("--ipv6".to_string());
        }

        if self.attachable {
            args.push("--attachable".to_string());
        }

        if self.internal {
            args.push("--internal".to_string());
        }

        for (key, value) in &self.labels {
            args.push("--label".to_string());
            args.push(format!("{key}={value}"));
        }

        if let Some(ref scope) = self.scope {
            args.push("--scope".to_string());
            args.push(scope.clone());
        }

        if let Some(ref config_from) = self.config_from {
            args.push("--config-from".to_string());
            args.push(config_from.clone());
        }

        if self.config_only {
            args.push("--config-only".to_string());
        }

        if self.ingress {
            args.push("--ingress".to_string());
        }

        for (name, ip) in &self.aux_addresses {
            args.push("--aux-address".to_string());
            args.push(format!("{name}={ip}"));
        }

        args.push(self.name.clone());
        args.extend(self.executor.raw_args.clone());
        args
    }

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        let command_name = args[0].clone();
        let command_args = args[1..].to_vec();
        self.executor
            .execute_command(&command_name, command_args)
            .await
    }
}

/// Result from network create command
#[derive(Debug, Clone)]
pub struct NetworkCreateResult {
    /// Network ID
    pub network_id: String,
    /// Raw command output
    pub raw_output: CommandOutput,
}

impl From<CommandOutput> for NetworkCreateResult {
    fn from(output: CommandOutput) -> Self {
        let network_id = output.stdout.trim().to_string();
        Self {
            network_id,
            raw_output: output,
        }
    }
}

impl NetworkCreateResult {
    /// Check if the command was successful
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.raw_output.success
    }

    /// Get the network ID
    #[must_use]
    pub fn id(&self) -> &str {
        &self.network_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_create_basic() {
        let cmd = NetworkCreateCommand::new("my-network");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["network", "create", "my-network"]);
    }

    #[test]
    fn test_network_create_with_driver() {
        let cmd = NetworkCreateCommand::new("my-network").driver("overlay");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec!["network", "create", "--driver", "overlay", "my-network"]
        );
    }

    #[test]
    fn test_network_create_with_subnet() {
        let cmd = NetworkCreateCommand::new("my-network")
            .subnet("172.20.0.0/16")
            .gateway("172.20.0.1");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec![
                "network",
                "create",
                "--subnet",
                "172.20.0.0/16",
                "--gateway",
                "172.20.0.1",
                "my-network"
            ]
        );
    }

    #[test]
    fn test_network_create_all_options() {
        let cmd = NetworkCreateCommand::new("my-network")
            .driver("bridge")
            .driver_opt("com.docker.network.bridge.name", "br0")
            .subnet("172.20.0.0/16")
            .ip_range("172.20.240.0/20")
            .gateway("172.20.0.1")
            .ipv6()
            .attachable()
            .internal()
            .label("env", "test")
            .aux_address("host1", "172.20.0.5");

        let args = cmd.build_command_args();
        assert!(args.contains(&"--driver".to_string()));
        assert!(args.contains(&"--ipv6".to_string()));
        assert!(args.contains(&"--attachable".to_string()));
        assert!(args.contains(&"--internal".to_string()));
    }
}
