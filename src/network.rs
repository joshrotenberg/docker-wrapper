//! Docker network management module.
//!
//! This module provides complete Docker network lifecycle management including:
//! - Network creation with various drivers
//! - Network connection and disconnection
//! - Network listing and inspection
//! - Network cleanup and management
//! - Custom bridge networks
//!
//! # Example
//!
//! ```rust,no_run
//! use docker_wrapper::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), DockerError> {
//!     let client = DockerClient::new().await?;
//!     let network_manager = client.networks();
//!
//!     // Create a custom network
//!     let network_config = NetworkConfig::new("my-network")
//!         .driver(NetworkDriver::Bridge)
//!         .subnet("172.20.0.0/16")
//!         .gateway("172.20.0.1");
//!
//!     let network_id = network_manager.create(network_config).await?;
//!
//!     // Connect a container to the network
//!     let container_id = ContainerId::new("my-container");
//!     network_manager.connect(&network_id, &container_id, None).await?;
//!
//!     // List all networks
//!     let networks = network_manager.list(ListNetworksOptions::default()).await?;
//!     println!("Found {} networks", networks.len());
//!
//!     // Cleanup
//!     network_manager.disconnect(&network_id, &container_id, None).await?;
//!     network_manager.remove(&network_id).await?;
//!
//!     Ok(())
//! }
//! ```

use crate::client::DockerClient;
use crate::errors::{DockerError, DockerResult};
use crate::executor::ExecutionConfig;
use crate::types::{ContainerId, NetworkId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::SystemTime;

/// Docker network representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerNetwork {
    /// Network ID
    #[serde(rename = "Id")]
    pub id: NetworkId,
    /// Network name
    #[serde(rename = "Name")]
    pub name: String,
    /// Creation timestamp
    #[serde(rename = "Created")]
    pub created: String,
    /// Scope (local, global, swarm)
    #[serde(rename = "Scope")]
    pub scope: String,
    /// Driver name
    #[serde(rename = "Driver")]
    pub driver: String,
    /// Enable IPv6
    #[serde(rename = "EnableIPv6")]
    pub enable_ipv6: bool,
    /// IPAM configuration
    #[serde(rename = "IPAM")]
    pub ipam: Option<NetworkIPAM>,
    /// Internal network
    #[serde(rename = "Internal")]
    pub internal: bool,
    /// Attachable
    #[serde(rename = "Attachable")]
    pub attachable: bool,
    /// Ingress
    #[serde(rename = "Ingress")]
    pub ingress: bool,
    /// Container endpoints
    #[serde(rename = "Containers")]
    pub containers: Option<HashMap<String, NetworkContainer>>,
    /// Network options
    #[serde(rename = "Options")]
    pub options: Option<HashMap<String, String>>,
    /// Labels
    #[serde(rename = "Labels")]
    pub labels: Option<HashMap<String, String>>,
}

/// Temporary struct to parse Docker CLI network ls format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkCliEntry {
    /// Network ID
    #[serde(rename = "ID")]
    pub id: String,
    /// Network name
    #[serde(rename = "Name")]
    pub name: String,
    /// Creation timestamp
    #[serde(rename = "CreatedAt")]
    pub created_at: String,
    /// Scope (local, global, swarm)
    #[serde(rename = "Scope")]
    pub scope: String,
    /// Driver name
    #[serde(rename = "Driver")]
    pub driver: String,
    /// IPv4 enabled
    #[serde(rename = "IPv4")]
    pub ipv4: String,
    /// IPv6 enabled
    #[serde(rename = "IPv6")]
    pub ipv6: String,
    /// Internal network
    #[serde(rename = "Internal")]
    pub internal: String,
    /// Labels (as string)
    #[serde(rename = "Labels")]
    pub labels: String,
}

impl From<NetworkCliEntry> for DockerNetwork {
    fn from(cli_entry: NetworkCliEntry) -> Self {
        DockerNetwork {
            id: NetworkId::new(&cli_entry.id)
                .unwrap_or_else(|_| NetworkId::new("unknown").unwrap()),
            name: cli_entry.name,
            created: cli_entry.created_at,
            scope: cli_entry.scope,
            driver: cli_entry.driver,
            enable_ipv6: cli_entry.ipv6 == "true",
            ipam: None, // CLI format doesn't provide IPAM details
            internal: cli_entry.internal == "true",
            attachable: false,                // CLI format doesn't provide this
            ingress: false,                   // CLI format doesn't provide this
            containers: Some(HashMap::new()), // CLI format doesn't provide container details
            options: Some(HashMap::new()),    // CLI format doesn't provide options
            labels: if cli_entry.labels.is_empty() {
                None
            } else {
                // Parse simple comma-separated labels if needed
                Some(HashMap::new())
            },
        }
    }
}

impl DockerNetwork {
    /// Get the created time as SystemTime
    pub fn created_time(&self) -> DockerResult<SystemTime> {
        let timestamp = chrono::DateTime::parse_from_rfc3339(&self.created)
            .map_err(|e| DockerError::ParseError(format!("Invalid timestamp: {}", e)))?;
        Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp.timestamp() as u64))
    }

    /// Check if network is user-defined
    pub fn is_user_defined(&self) -> bool {
        !matches!(self.name.as_str(), "bridge" | "host" | "none")
    }

    /// Get connected container count
    pub fn container_count(&self) -> usize {
        self.containers.as_ref().map_or(0, |c| c.len())
    }

    /// Check if container is connected
    pub fn has_container(&self, container_id: &ContainerId) -> bool {
        self.containers.as_ref().map_or(false, |containers| {
            containers.contains_key(container_id.as_str())
        })
    }
}

/// Network IPAM (IP Address Management) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIPAM {
    /// Driver name
    #[serde(rename = "Driver")]
    pub driver: String,
    /// IPAM options
    #[serde(rename = "Options")]
    pub options: Option<HashMap<String, String>>,
    /// IPAM configuration
    #[serde(rename = "Config")]
    pub config: Option<Vec<NetworkIPAMConfig>>,
}

/// Network IPAM configuration item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIPAMConfig {
    /// Subnet
    #[serde(rename = "Subnet")]
    pub subnet: Option<String>,
    /// IP range
    #[serde(rename = "IPRange")]
    pub ip_range: Option<String>,
    /// Gateway
    #[serde(rename = "Gateway")]
    pub gateway: Option<String>,
    /// Auxiliary addresses
    #[serde(rename = "AuxiliaryAddresses")]
    pub auxiliary_addresses: Option<HashMap<String, String>>,
}

/// Container in network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkContainer {
    /// Container name
    #[serde(rename = "Name")]
    pub name: String,
    /// Endpoint ID
    #[serde(rename = "EndpointID")]
    pub endpoint_id: String,
    /// MAC address
    #[serde(rename = "MacAddress")]
    pub mac_address: String,
    /// IPv4 address
    #[serde(rename = "IPv4Address")]
    pub ipv4_address: String,
    /// IPv6 address
    #[serde(rename = "IPv6Address")]
    pub ipv6_address: String,
}

/// Network configuration for creation
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Network name
    pub name: String,
    /// Network driver
    pub driver: NetworkDriver,
    /// Enable IPv6
    pub enable_ipv6: bool,
    /// Internal network
    pub internal: bool,
    /// Attachable
    pub attachable: bool,
    /// Ingress
    pub ingress: bool,
    /// IPAM configuration
    pub ipam_config: Option<IPAMConfig>,
    /// Driver options
    pub options: HashMap<String, String>,
    /// Labels
    pub labels: HashMap<String, String>,
}

impl NetworkConfig {
    /// Create a new network configuration
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            driver: NetworkDriver::Bridge,
            enable_ipv6: false,
            internal: false,
            attachable: false,
            ingress: false,
            ipam_config: None,
            options: HashMap::new(),
            labels: HashMap::new(),
        }
    }

    /// Set network driver
    pub fn driver(mut self, driver: NetworkDriver) -> Self {
        self.driver = driver;
        self
    }

    /// Enable IPv6
    pub fn enable_ipv6(mut self) -> Self {
        self.enable_ipv6 = true;
        self
    }

    /// Make network internal
    pub fn internal(mut self) -> Self {
        self.internal = true;
        self
    }

    /// Make network attachable
    pub fn attachable(mut self) -> Self {
        self.attachable = true;
        self
    }

    /// Make network ingress
    pub fn ingress(mut self) -> Self {
        self.ingress = true;
        self
    }

    /// Set subnet
    pub fn subnet(mut self, subnet: impl Into<String>) -> Self {
        let ipam = self.ipam_config.get_or_insert_with(IPAMConfig::new);
        ipam.subnet = Some(subnet.into());
        self
    }

    /// Set gateway
    pub fn gateway(mut self, gateway: impl Into<String>) -> Self {
        let ipam = self.ipam_config.get_or_insert_with(IPAMConfig::new);
        ipam.gateway = Some(gateway.into());
        self
    }

    /// Set IP range
    pub fn ip_range(mut self, ip_range: impl Into<String>) -> Self {
        let ipam = self.ipam_config.get_or_insert_with(IPAMConfig::new);
        ipam.ip_range = Some(ip_range.into());
        self
    }

    /// Add driver option
    pub fn option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }

    /// Add label
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }
}

/// IPAM configuration
#[derive(Debug, Clone)]
pub struct IPAMConfig {
    /// Driver name
    pub driver: String,
    /// Subnet
    pub subnet: Option<String>,
    /// IP range
    pub ip_range: Option<String>,
    /// Gateway
    pub gateway: Option<String>,
    /// Auxiliary addresses
    pub auxiliary_addresses: HashMap<String, String>,
}

impl IPAMConfig {
    /// Create new IPAM configuration
    pub fn new() -> Self {
        Self {
            driver: "default".to_string(),
            subnet: None,
            ip_range: None,
            gateway: None,
            auxiliary_addresses: HashMap::new(),
        }
    }

    /// Set driver
    pub fn driver(mut self, driver: impl Into<String>) -> Self {
        self.driver = driver.into();
        self
    }

    /// Add auxiliary address
    pub fn auxiliary_address(
        mut self,
        name: impl Into<String>,
        address: impl Into<String>,
    ) -> Self {
        self.auxiliary_addresses.insert(name.into(), address.into());
        self
    }
}

impl Default for IPAMConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Network driver types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkDriver {
    /// Bridge driver
    Bridge,
    /// Host driver
    Host,
    /// None driver
    None,
    /// Overlay driver
    Overlay,
    /// MacVLAN driver
    Macvlan,
    /// IPvLAN driver
    Ipvlan,
    /// Custom driver
    Custom(String),
}

impl NetworkDriver {
    /// Get driver name as string
    pub fn as_str(&self) -> &str {
        match self {
            Self::Bridge => "bridge",
            Self::Host => "host",
            Self::None => "none",
            Self::Overlay => "overlay",
            Self::Macvlan => "macvlan",
            Self::Ipvlan => "ipvlan",
            Self::Custom(name) => name,
        }
    }
}

impl Default for NetworkDriver {
    fn default() -> Self {
        Self::Bridge
    }
}

impl std::fmt::Display for NetworkDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Options for listing networks
#[derive(Debug, Clone, Default)]
pub struct ListNetworksOptions {
    /// Filter by name
    pub filters: HashMap<String, Vec<String>>,
}

impl ListNetworksOptions {
    /// Create new list options
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by name pattern
    pub fn filter_name(mut self, pattern: impl Into<String>) -> Self {
        self.filters
            .entry("name".to_string())
            .or_default()
            .push(pattern.into());
        self
    }

    /// Filter by driver
    pub fn filter_driver(mut self, driver: &NetworkDriver) -> Self {
        self.filters
            .entry("driver".to_string())
            .or_default()
            .push(driver.as_str().to_string());
        self
    }

    /// Filter by type
    pub fn filter_type(mut self, network_type: impl Into<String>) -> Self {
        self.filters
            .entry("type".to_string())
            .or_default()
            .push(network_type.into());
        self
    }

    /// Filter by label
    pub fn filter_label(mut self, label: impl Into<String>) -> Self {
        self.filters
            .entry("label".to_string())
            .or_default()
            .push(label.into());
        self
    }

    /// Filter by scope
    pub fn filter_scope(mut self, scope: impl Into<String>) -> Self {
        self.filters
            .entry("scope".to_string())
            .or_default()
            .push(scope.into());
        self
    }
}

/// Options for connecting container to network
#[derive(Debug, Clone, Default)]
pub struct ConnectOptions {
    /// Aliases for the container
    pub aliases: Vec<String>,
    /// Links to other containers
    pub links: Vec<String>,
    /// IPv4 address
    pub ipv4_address: Option<IpAddr>,
    /// IPv6 address
    pub ipv6_address: Option<IpAddr>,
    /// Link local IPs
    pub link_local_ips: Vec<IpAddr>,
}

impl ConnectOptions {
    /// Create new connect options
    pub fn new() -> Self {
        Self::default()
    }

    /// Add alias
    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    /// Add link
    pub fn link(mut self, link: impl Into<String>) -> Self {
        self.links.push(link.into());
        self
    }

    /// Set IPv4 address
    pub fn ipv4_address(mut self, address: IpAddr) -> Self {
        self.ipv4_address = Some(address);
        self
    }

    /// Set IPv6 address
    pub fn ipv6_address(mut self, address: IpAddr) -> Self {
        self.ipv6_address = Some(address);
        self
    }

    /// Add link local IP
    pub fn link_local_ip(mut self, ip: IpAddr) -> Self {
        self.link_local_ips.push(ip);
        self
    }
}

/// Options for disconnecting container from network
#[derive(Debug, Clone, Default)]
pub struct DisconnectOptions {
    /// Force disconnect
    pub force: bool,
}

impl DisconnectOptions {
    /// Create new disconnect options
    pub fn new() -> Self {
        Self::default()
    }

    /// Force disconnect
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }
}

/// Network inspection details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInspect {
    /// Network name
    #[serde(rename = "Name")]
    pub name: String,
    /// Network ID
    #[serde(rename = "Id")]
    pub id: NetworkId,
    /// Creation timestamp
    #[serde(rename = "Created")]
    pub created: String,
    /// Scope
    #[serde(rename = "Scope")]
    pub scope: String,
    /// Driver
    #[serde(rename = "Driver")]
    pub driver: String,
    /// Enable IPv6
    #[serde(rename = "EnableIPv6")]
    pub enable_ipv6: bool,
    /// IPAM
    #[serde(rename = "IPAM")]
    pub ipam: NetworkIPAM,
    /// Internal
    #[serde(rename = "Internal")]
    pub internal: bool,
    /// Attachable
    #[serde(rename = "Attachable")]
    pub attachable: bool,
    /// Ingress
    #[serde(rename = "Ingress")]
    pub ingress: bool,
    /// Config from
    #[serde(rename = "ConfigFrom")]
    pub config_from: Option<serde_json::Value>,
    /// Config only
    #[serde(rename = "ConfigOnly")]
    pub config_only: bool,
    /// Containers
    #[serde(rename = "Containers")]
    pub containers: HashMap<String, NetworkContainer>,
    /// Options
    #[serde(rename = "Options")]
    pub options: HashMap<String, String>,
    /// Labels
    #[serde(rename = "Labels")]
    pub labels: HashMap<String, String>,
}

/// Network manager providing all network operations
pub struct NetworkManager<'a> {
    client: &'a DockerClient,
}

impl<'a> NetworkManager<'a> {
    /// Create a new network manager
    pub fn new(client: &'a DockerClient) -> Self {
        Self { client }
    }

    /// Create a new network
    pub async fn create(&self, config: NetworkConfig) -> DockerResult<NetworkId> {
        let mut args = vec!["network".to_string(), "create".to_string()];

        // Add driver
        args.push("--driver".to_string());
        args.push(config.driver.as_str().to_string());

        // Add IPv6 support
        if config.enable_ipv6 {
            args.push("--ipv6".to_string());
        }

        // Add internal flag
        if config.internal {
            args.push("--internal".to_string());
        }

        // Add attachable flag
        if config.attachable {
            args.push("--attachable".to_string());
        }

        // Add ingress flag
        if config.ingress {
            args.push("--ingress".to_string());
        }

        // Add IPAM configuration
        if let Some(ipam) = &config.ipam_config {
            if let Some(subnet) = &ipam.subnet {
                args.push("--subnet".to_string());
                args.push(subnet.clone());
            }

            if let Some(gateway) = &ipam.gateway {
                args.push("--gateway".to_string());
                args.push(gateway.clone());
            }

            if let Some(ip_range) = &ipam.ip_range {
                args.push("--ip-range".to_string());
                args.push(ip_range.clone());
            }

            for (name, address) in &ipam.auxiliary_addresses {
                args.push("--aux-address".to_string());
                args.push(format!("{}={}", name, address));
            }
        }

        // Add driver options
        for (key, value) in &config.options {
            args.push("--opt".to_string());
            args.push(format!("{}={}", key, value));
        }

        // Add labels
        for (key, value) in &config.labels {
            args.push("--label".to_string());
            args.push(format!("{}={}", key, value));
        }

        // Add network name
        args.push(config.name);

        let output = self
            .client
            .executor()
            .execute(&args, Some(ExecutionConfig::default()))
            .await?;

        if !output.success {
            return Err(DockerError::CommandFailed {
                command: format!("docker {}", args.join(" ")),
                exit_code: output.exit_code,
                stdout: output.stdout.clone(),
                stderr: output.stderr.clone(),
            });
        }

        let stdout = &output.stdout;
        let network_id = stdout.trim();

        Ok(NetworkId::new(network_id)?)
    }

    /// List networks
    pub async fn list(&self, options: ListNetworksOptions) -> DockerResult<Vec<DockerNetwork>> {
        let mut args = vec![
            "network".to_string(),
            "ls".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ];

        // Add filters
        for (key, values) in &options.filters {
            for value in values {
                args.push("--filter".to_string());
                args.push(format!("{}={}", key, value));
            }
        }

        let output = self
            .client
            .executor()
            .execute(&args, Some(ExecutionConfig::default()))
            .await?;

        if !output.success {
            return Err(DockerError::CommandFailed {
                command: format!("docker {}", args.join(" ")),
                exit_code: output.exit_code,
                stdout: output.stdout.clone(),
                stderr: output.stderr.clone(),
            });
        }

        let stdout = &output.stdout;
        let mut networks = Vec::new();

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<NetworkCliEntry>(line) {
                Ok(cli_entry) => networks.push(cli_entry.into()),
                Err(e) => {
                    log::warn!("Failed to parse network JSON: {} - {}", e, line);
                }
            }
        }

        Ok(networks)
    }

    /// Inspect a network
    pub async fn inspect(&self, network_id: &NetworkId) -> DockerResult<NetworkInspect> {
        let args = vec![
            "network".to_string(),
            "inspect".to_string(),
            network_id.as_str().to_string(),
        ];

        let output = self
            .client
            .executor()
            .execute(&args, Some(ExecutionConfig::default()))
            .await?;

        if !output.success {
            return Err(DockerError::CommandFailed {
                command: format!("docker {}", args.join(" ")),
                exit_code: output.exit_code,
                stdout: output.stdout.clone(),
                stderr: output.stderr.clone(),
            });
        }

        let stdout = &output.stdout;
        let inspects: Vec<NetworkInspect> = serde_json::from_str(&stdout)
            .map_err(|e| DockerError::ParseError(format!("Invalid inspect JSON: {}", e)))?;

        inspects
            .into_iter()
            .next()
            .ok_or_else(|| DockerError::NotFound {
                message: format!("Network not found: {}", network_id),
            })
    }

    /// Connect a container to a network
    pub async fn connect(
        &self,
        network_id: &NetworkId,
        container_id: &ContainerId,
        options: Option<ConnectOptions>,
    ) -> DockerResult<()> {
        let mut args = vec!["network".to_string(), "connect".to_string()];

        if let Some(opts) = &options {
            // Add aliases
            for alias in &opts.aliases {
                args.push("--alias".to_string());
                args.push(alias.clone());
            }

            // Add links
            for link in &opts.links {
                args.push("--link".to_string());
                args.push(link.clone());
            }

            // Add IPv4 address
            if let Some(ip) = &opts.ipv4_address {
                args.push("--ip".to_string());
                args.push(ip.to_string());
            }

            // Add IPv6 address
            if let Some(ip) = &opts.ipv6_address {
                args.push("--ip6".to_string());
                args.push(ip.to_string());
            }

            // Add link local IPs
            for ip in &opts.link_local_ips {
                args.push("--link-local-ip".to_string());
                args.push(ip.to_string());
            }
        }

        args.push(network_id.as_str().to_string());
        args.push(container_id.as_str().to_string());

        let output = self
            .client
            .executor()
            .execute(&args, Some(ExecutionConfig::default()))
            .await?;

        if !output.success {
            return Err(DockerError::NetworkConnectionFailed {
                container_id: container_id.as_str().to_string(),
                network_id: network_id.as_str().to_string(),
            });
        }

        Ok(())
    }

    /// Disconnect a container from a network
    pub async fn disconnect(
        &self,
        network_id: &NetworkId,
        container_id: &ContainerId,
        options: Option<DisconnectOptions>,
    ) -> DockerResult<()> {
        let mut args = vec!["network".to_string(), "disconnect".to_string()];

        if let Some(opts) = &options {
            if opts.force {
                args.push("--force".to_string());
            }
        }

        args.push(network_id.as_str().to_string());
        args.push(container_id.as_str().to_string());

        let output = self
            .client
            .executor()
            .execute(&args, Some(ExecutionConfig::default()))
            .await?;

        if !output.success {
            return Err(DockerError::CommandFailed {
                command: format!("docker {}", args.join(" ")),
                exit_code: output.exit_code,
                stdout: output.stdout.clone(),
                stderr: output.stderr.clone(),
            });
        }

        Ok(())
    }

    /// Remove a network
    pub async fn remove(&self, network_id: &NetworkId) -> DockerResult<()> {
        let args = vec![
            "network".to_string(),
            "rm".to_string(),
            network_id.as_str().to_string(),
        ];

        let output = self
            .client
            .executor()
            .execute(&args, Some(ExecutionConfig::default()))
            .await?;

        if !output.success {
            return Err(DockerError::CommandFailed {
                command: format!("docker {}", args.join(" ")),
                exit_code: output.exit_code,
                stdout: output.stdout.clone(),
                stderr: output.stderr.clone(),
            });
        }

        Ok(())
    }

    /// Prune unused networks
    pub async fn prune(&self) -> DockerResult<NetworkPruneResult> {
        let args = vec![
            "network".to_string(),
            "prune".to_string(),
            "--force".to_string(),
        ];

        let output = self
            .client
            .executor()
            .execute(&args, Some(ExecutionConfig::default()))
            .await?;

        if !output.success {
            return Err(DockerError::CommandFailed {
                command: format!("docker {}", args.join(" ")),
                exit_code: output.exit_code,
                stdout: output.stdout.clone(),
                stderr: output.stderr.clone(),
            });
        }

        let stdout = &output.stdout;

        // Parse deleted networks from output
        let mut deleted_networks = Vec::new();
        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }
            // Network prune output typically lists deleted network names
            if !line.contains("Total reclaimed space") {
                deleted_networks.push(line.trim().to_string());
            }
        }

        Ok(NetworkPruneResult { deleted_networks })
    }

    /// Get network by name
    pub async fn get_by_name(&self, name: &str) -> DockerResult<Option<DockerNetwork>> {
        let options = ListNetworksOptions::new().filter_name(name);
        let networks = self.list(options).await?;

        Ok(networks.into_iter().find(|n| n.name == name))
    }

    /// Check if network exists
    pub async fn exists(&self, network_id: &NetworkId) -> DockerResult<bool> {
        match self.inspect(network_id).await {
            Ok(_) => Ok(true),
            Err(DockerError::NotFound { .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

/// Result of network pruning operation
#[derive(Debug, Clone)]
pub struct NetworkPruneResult {
    /// List of deleted network names
    pub deleted_networks: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_config_builder() {
        let config = NetworkConfig::new("test-network")
            .driver(NetworkDriver::Bridge)
            .subnet("172.20.0.0/16")
            .gateway("172.20.0.1")
            .label("env", "test")
            .internal();

        assert_eq!(config.name, "test-network");
        assert_eq!(config.driver, NetworkDriver::Bridge);
        assert!(config.internal);
        assert_eq!(config.labels.get("env"), Some(&"test".to_string()));
        assert!(config.ipam_config.is_some());
    }

    #[test]
    fn test_network_driver_display() {
        assert_eq!(NetworkDriver::Bridge.to_string(), "bridge");
        assert_eq!(NetworkDriver::Host.to_string(), "host");
        assert_eq!(
            NetworkDriver::Custom("custom".to_string()).to_string(),
            "custom"
        );
    }

    #[test]
    fn test_connect_options_builder() {
        let options = ConnectOptions::new()
            .alias("web")
            .alias("frontend")
            .link("db:database");

        assert_eq!(options.aliases, vec!["web", "frontend"]);
        assert_eq!(options.links, vec!["db:database"]);
    }

    #[test]
    fn test_list_networks_options() {
        let options = ListNetworksOptions::new()
            .filter_name("test*")
            .filter_driver(&NetworkDriver::Bridge)
            .filter_label("env=test");

        assert!(!options.filters.is_empty());
        assert!(options.filters.contains_key("name"));
        assert!(options.filters.contains_key("driver"));
        assert!(options.filters.contains_key("label"));
    }

    #[test]
    fn test_ipam_config() {
        let ipam = IPAMConfig::new()
            .driver("custom")
            .auxiliary_address("web", "192.168.1.100");

        assert_eq!(ipam.driver, "custom");
        assert_eq!(
            ipam.auxiliary_addresses.get("web"),
            Some(&"192.168.1.100".to_string())
        );
    }
}
