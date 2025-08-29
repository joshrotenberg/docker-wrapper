//! Configuration and state management for redis-dev

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Configuration directory name
const CONFIG_DIR: &str = "redis-dev";

/// Configuration file name  
const CONFIG_FILE: &str = "instances.json";

/// Instance types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InstanceType {
    Basic,
    Stack,
    Cluster,
    Sentinel,
    Enterprise,
}

impl std::fmt::Display for InstanceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstanceType::Basic => write!(f, "basic"),
            InstanceType::Stack => write!(f, "stack"),
            InstanceType::Cluster => write!(f, "cluster"),
            InstanceType::Sentinel => write!(f, "sentinel"),
            InstanceType::Enterprise => write!(f, "enterprise"),
        }
    }
}

/// Instance metadata stored in configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub name: String,
    pub instance_type: InstanceType,
    pub created_at: String,
    pub ports: Vec<u16>,
    pub containers: Vec<String>,
    pub connection_info: ConnectionInfo,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Connection information for an instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub url: String,
    pub additional_ports: HashMap<String, u16>,
}

/// Configuration state
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub instances: HashMap<String, InstanceInfo>,
    pub counters: HashMap<String, u32>,
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

        let config: Config =
            serde_json::from_str(&content).with_context(|| "Failed to parse config file")?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;

        let content =
            serde_json::to_string_pretty(self).with_context(|| "Failed to serialize config")?;

        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

        Ok(())
    }

    /// Add an instance to the configuration
    pub fn add_instance(&mut self, info: InstanceInfo) {
        self.instances.insert(info.name.clone(), info);
    }

    /// Remove an instance from the configuration
    pub fn remove_instance(&mut self, name: &str) -> Option<InstanceInfo> {
        self.instances.remove(name)
    }

    /// Get an instance by name
    pub fn get_instance(&self, name: &str) -> Option<&InstanceInfo> {
        self.instances.get(name)
    }

    /// List all instances
    pub fn list_instances(&self) -> Vec<&InstanceInfo> {
        self.instances.values().collect()
    }

    /// List instances by type
    pub fn list_instances_by_type(&self, instance_type: &InstanceType) -> Vec<&InstanceInfo> {
        self.instances
            .values()
            .filter(|info| &info.instance_type == instance_type)
            .collect()
    }

    /// Generate a unique name for an instance type
    pub fn generate_name(&mut self, instance_type: &InstanceType) -> String {
        let counter = self.counters.entry(instance_type.to_string()).or_insert(0);
        *counter += 1;
        format!("redis-{}-{}", instance_type, counter)
    }

    /// Get the latest instance of a type (highest counter)
    pub fn get_latest_instance(&self, instance_type: &InstanceType) -> Option<&InstanceInfo> {
        self.instances
            .values()
            .filter(|info| &info.instance_type == instance_type)
            .max_by_key(|info| {
                // Extract counter from name like "redis-cluster-1"
                info.name
                    .rsplit('-')
                    .next()
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(0)
            })
    }
}

/// Get the configuration directory path
pub fn get_config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".config").join(CONFIG_DIR))
}

/// Get the configuration file path
pub fn get_config_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join(CONFIG_FILE))
}

/// Ensure the configuration directory exists
pub fn ensure_config_dir() -> Result<()> {
    let config_dir = get_config_dir()?;

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).with_context(|| {
            format!(
                "Failed to create config directory: {}",
                config_dir.display()
            )
        })?;
    }

    Ok(())
}

/// Generate a random password
pub fn generate_password() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789";
    let mut rng = rand::thread_rng();

    (0..16)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_type_display() {
        assert_eq!(InstanceType::Basic.to_string(), "basic");
        assert_eq!(InstanceType::Stack.to_string(), "stack");
        assert_eq!(InstanceType::Cluster.to_string(), "cluster");
        assert_eq!(InstanceType::Sentinel.to_string(), "sentinel");
        assert_eq!(InstanceType::Enterprise.to_string(), "enterprise");
    }

    #[test]
    fn test_config_name_generation() {
        let mut config = Config::default();
        
        // Test that counters increment properly
        let name1 = config.generate_name(&InstanceType::Basic);
        assert_eq!(name1, "redis-basic-1");
        
        let name2 = config.generate_name(&InstanceType::Basic);
        assert_eq!(name2, "redis-basic-2");
        
        // Different types have separate counters
        let cluster1 = config.generate_name(&InstanceType::Cluster);
        assert_eq!(cluster1, "redis-cluster-1");
    }

    #[test]
    fn test_get_latest_instance() {
        let mut config = Config::default();
        
        // Add some instances
        let instance1 = InstanceInfo {
            name: "redis-basic-1".to_string(),
            instance_type: InstanceType::Basic,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            ports: vec![6379],
            containers: vec!["container1".to_string()],
            connection_info: ConnectionInfo {
                host: "localhost".to_string(),
                port: 6379,
                password: None,
                url: "redis://localhost:6379".to_string(),
                additional_ports: HashMap::new(),
            },
            metadata: HashMap::new(),
        };
        
        let instance2 = InstanceInfo {
            name: "redis-basic-5".to_string(),
            instance_type: InstanceType::Basic,
            created_at: "2024-01-02T00:00:00Z".to_string(),
            ports: vec![6380],
            containers: vec!["container2".to_string()],
            connection_info: ConnectionInfo {
                host: "localhost".to_string(),
                port: 6380,
                password: None,
                url: "redis://localhost:6380".to_string(),
                additional_ports: HashMap::new(),
            },
            metadata: HashMap::new(),
        };
        
        config.add_instance(instance1);
        config.add_instance(instance2);
        
        // Should return the one with highest counter (redis-basic-5)
        let latest = config.get_latest_instance(&InstanceType::Basic);
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().name, "redis-basic-5");
    }

    #[test]
    fn test_password_generation_uniqueness() {
        let passwords: Vec<String> = (0..100)
            .map(|_| generate_password())
            .collect();
        
        // Check all passwords are unique
        let unique_count = passwords.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_count, 100);
        
        // Check all passwords are 16 chars
        for password in &passwords {
            assert_eq!(password.len(), 16);
        }
    }
}
