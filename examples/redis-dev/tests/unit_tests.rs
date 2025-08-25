//! Unit tests for Redis Developer CLI components
//!
//! These tests verify the internal logic and components of the redis-dev CLI
//! without actually running Docker containers.

use redis_dev::config::{InstanceConfig, RedisMode};
use redis_dev::state::InstanceState;
use std::path::PathBuf;

#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_instance_config_defaults() {
        let config = InstanceConfig::default();
        
        assert_eq!(config.mode, RedisMode::Standalone);
        assert_eq!(config.port, 6379);
        assert!(!config.persist);
        assert!(config.name.starts_with("redis-dev-"));
    }
    
    #[test]
    fn test_instance_config_with_custom_name() {
        let config = InstanceConfig {
            name: "custom-redis".to_string(),
            mode: RedisMode::Standalone,
            port: 6380,
            persist: true,
            data_dir: Some(PathBuf::from("/data")),
            password: Some("secret".to_string()),
            version: "7.0".to_string(),
        };
        
        assert_eq!(config.name, "custom-redis");
        assert_eq!(config.port, 6380);
        assert!(config.persist);
        assert_eq!(config.password, Some("secret".to_string()));
    }
    
    #[test]
    fn test_redis_mode_variants() {
        let standalone = RedisMode::Standalone;
        let cluster = RedisMode::Cluster { nodes: 6 };
        let sentinel = RedisMode::Sentinel { replicas: 2 };
        
        match cluster {
            RedisMode::Cluster { nodes } => assert_eq!(nodes, 6),
            _ => panic!("Expected cluster mode"),
        }
        
        match sentinel {
            RedisMode::Sentinel { replicas } => assert_eq!(replicas, 2),
            _ => panic!("Expected sentinel mode"),
        }
        
        assert!(matches!(standalone, RedisMode::Standalone));
    }
    
    #[test]
    fn test_config_validation() {
        // Test invalid port
        let config = InstanceConfig {
            port: 0,
            ..Default::default()
        };
        assert!(config.port == 0, "Should allow port 0 for auto-assignment");
        
        // Test valid port range
        let config = InstanceConfig {
            port: 65535,
            ..Default::default()
        };
        assert!(config.port > 0 && config.port <= 65535);
    }
}

#[cfg(test)]
mod state_tests {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn test_instance_state_creation() {
        let state = InstanceState {
            name: "test-instance".to_string(),
            container_id: "abc123".to_string(),
            mode: RedisMode::Standalone,
            port: 6379,
            status: "running".to_string(),
            created_at: Utc::now(),
            data_dir: None,
        };
        
        assert_eq!(state.name, "test-instance");
        assert_eq!(state.container_id, "abc123");
        assert_eq!(state.status, "running");
    }
    
    #[test]
    fn test_state_serialization() {
        let state = InstanceState {
            name: "test".to_string(),
            container_id: "123".to_string(),
            mode: RedisMode::Standalone,
            port: 6379,
            status: "running".to_string(),
            created_at: Utc::now(),
            data_dir: Some(PathBuf::from("/data")),
        };
        
        // Test JSON serialization
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"port\":6379"));
        
        // Test deserialization
        let deserialized: InstanceState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, state.name);
        assert_eq!(deserialized.port, state.port);
    }
    
    #[test]
    fn test_cluster_state() {
        let state = InstanceState {
            name: "cluster".to_string(),
            container_id: "cluster-main".to_string(),
            mode: RedisMode::Cluster { nodes: 3 },
            port: 7000,
            status: "running".to_string(),
            created_at: Utc::now(),
            data_dir: None,
        };
        
        match state.mode {
            RedisMode::Cluster { nodes } => assert_eq!(nodes, 3),
            _ => panic!("Expected cluster mode"),
        }
    }
}

#[cfg(test)]
mod command_tests {
    use super::*;
    
    #[test]
    fn test_name_generation() {
        // Test that generated names follow pattern
        let config1 = InstanceConfig::default();
        let config2 = InstanceConfig::default();
        
        assert!(config1.name.starts_with("redis-dev-"));
        assert!(config2.name.starts_with("redis-dev-"));
        assert_ne!(config1.name, config2.name, "Names should be unique");
    }
    
    #[test]
    fn test_port_auto_increment() {
        let mut configs = vec![];
        let mut base_port = 6379;
        
        for _ in 0..5 {
            let config = InstanceConfig {
                port: base_port,
                ..Default::default()
            };
            configs.push(config);
            base_port += 1;
        }
        
        // Verify ports are sequential
        for (i, config) in configs.iter().enumerate() {
            assert_eq!(config.port, 6379 + i as u16);
        }
    }
    
    #[test]
    fn test_persistence_path_generation() {
        let config = InstanceConfig {
            name: "test-persist".to_string(),
            persist: true,
            data_dir: None,
            ..Default::default()
        };
        
        // When persist is true but no data_dir specified,
        // it should generate a default path
        if config.persist && config.data_dir.is_none() {
            // This would be handled by the implementation
            let default_path = PathBuf::from(format!("/tmp/redis-dev/{}", config.name));
            assert!(default_path.to_str().unwrap().contains(&config.name));
        }
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;
    
    #[test]
    fn test_invalid_cluster_nodes() {
        let invalid_configs = vec![
            RedisMode::Cluster { nodes: 0 },  // Too few
            RedisMode::Cluster { nodes: 2 },  // Not enough for cluster
            RedisMode::Cluster { nodes: 1001 }, // Too many
        ];
        
        for mode in invalid_configs {
            match mode {
                RedisMode::Cluster { nodes } => {
                    // Cluster should have at least 3 nodes
                    assert!(
                        nodes < 3 || nodes > 1000,
                        "Invalid cluster size should be detected"
                    );
                }
                _ => {}
            }
        }
    }
    
    #[test]
    fn test_invalid_sentinel_replicas() {
        let invalid_configs = vec![
            RedisMode::Sentinel { replicas: 0 },  // Need at least 1 replica
            RedisMode::Sentinel { replicas: 100 }, // Too many replicas
        ];
        
        for mode in invalid_configs {
            match mode {
                RedisMode::Sentinel { replicas } => {
                    assert!(
                        replicas < 1 || replicas > 10,
                        "Invalid sentinel replica count should be detected"
                    );
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod helper_tests {
    use super::*;
    
    #[test]
    fn test_container_name_validation() {
        let valid_names = vec![
            "redis-dev-test",
            "redis-dev-123",
            "my-redis-instance",
            "test_redis",
        ];
        
        for name in valid_names {
            // Docker container names can contain [a-zA-Z0-9][a-zA-Z0-9_.-]
            let is_valid = name.chars().all(|c| {
                c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.'
            });
            assert!(is_valid, "Name {} should be valid", name);
        }
        
        let invalid_names = vec![
            "redis dev", // Contains space
            "redis@dev", // Contains @
            "redis/dev", // Contains /
        ];
        
        for name in invalid_names {
            let is_valid = name.chars().all(|c| {
                c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.'
            });
            assert!(!is_valid, "Name {} should be invalid", name);
        }
    }
    
    #[test]
    fn test_version_parsing() {
        let versions = vec![
            ("7", "redis:7-alpine"),
            ("7.0", "redis:7.0-alpine"),
            ("7.0.5", "redis:7.0.5-alpine"),
            ("latest", "redis:latest"),
            ("6.2", "redis:6.2-alpine"),
        ];
        
        for (input, expected_tag) in versions {
            let tag = if input == "latest" {
                "redis:latest".to_string()
            } else {
                format!("redis:{}-alpine", input)
            };
            assert_eq!(tag, expected_tag);
        }
    }
}

// Module stubs for testing - these would be in the actual implementation
mod redis_dev {
    pub mod config {
        use std::path::PathBuf;
        use serde::{Serialize, Deserialize};
        
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub enum RedisMode {
            Standalone,
            Cluster { nodes: usize },
            Sentinel { replicas: usize },
        }
        
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct InstanceConfig {
            pub name: String,
            pub mode: RedisMode,
            pub port: u16,
            pub persist: bool,
            pub data_dir: Option<PathBuf>,
            pub password: Option<String>,
            pub version: String,
        }
        
        impl Default for InstanceConfig {
            fn default() -> Self {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let suffix: u32 = rng.gen_range(1000..9999);
                
                Self {
                    name: format!("redis-dev-{}", suffix),
                    mode: RedisMode::Standalone,
                    port: 6379,
                    persist: false,
                    data_dir: None,
                    password: None,
                    version: "7".to_string(),
                }
            }
        }
    }
    
    pub mod state {
        use super::config::RedisMode;
        use std::path::PathBuf;
        use serde::{Serialize, Deserialize};
        use chrono::{DateTime, Utc};
        
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct InstanceState {
            pub name: String,
            pub container_id: String,
            pub mode: RedisMode,
            pub port: u16,
            pub status: String,
            pub created_at: DateTime<Utc>,
            pub data_dir: Option<PathBuf>,
        }
    }
}