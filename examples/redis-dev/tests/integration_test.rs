//! Integration tests for redis-dev CLI tool

use redis_dev::config::{Config, InstanceType};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_config_generation() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("instances.json");

    // Set up environment to use temp directory
    std::env::set_var("HOME", temp_dir.path());

    let mut config = Config::default();

    // Test name generation
    let name1 = config.generate_name(&InstanceType::Basic);
    assert_eq!(name1, "redis-basic-1");

    let name2 = config.generate_name(&InstanceType::Basic);
    assert_eq!(name2, "redis-basic-2");

    let cluster_name = config.generate_name(&InstanceType::Cluster);
    assert_eq!(cluster_name, "redis-cluster-1");
}

#[test]
fn test_password_generation() {
    use redis_dev::config::generate_password;

    let password1 = generate_password();
    let password2 = generate_password();

    // Passwords should be 16 characters
    assert_eq!(password1.len(), 16);
    assert_eq!(password2.len(), 16);

    // Passwords should be different
    assert_ne!(password1, password2);

    // Passwords should only contain allowed characters
    for c in password1.chars() {
        assert!(c.is_ascii_alphanumeric() && c != '0' && c != '1' && c != 'O' && c != 'l');
    }
}

#[cfg(feature = "docker-tests")]
mod docker_tests {
    use super::*;
    use docker_wrapper::{DockerCommand, PsCommand};

    #[tokio::test]
    async fn test_basic_redis_lifecycle() {
        // This would test actual Docker operations
        // Marked with a feature flag since it requires Docker

        // Check if Docker is available
        let ps_result = PsCommand::new().execute().await;
        assert!(
            ps_result.is_ok(),
            "Docker must be running for integration tests"
        );
    }
}
