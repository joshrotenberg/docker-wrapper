//! Integration tests for all Redis deployment types
//! These tests require Docker to be running

use anyhow::Result;
use docker_wrapper::{DockerCommand, RmCommand, StopCommand};
use redis::AsyncCommands;
use redis_dev::cli::{
    BasicStartArgs, Cli, ClusterAction, ClusterStartArgs, Commands, EnterpriseAction,
    EnterpriseStartArgs, RedisAction, SentinelAction, SentinelStartArgs, StackAction,
    StackStartArgs, StopArgs,
};
use redis_dev::commands;
use redis_dev::config::{Config, InstanceType};
use serial_test::serial;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

/// Helper to ensure clean state before and after tests
async fn cleanup_all() {
    let config = Config::load().unwrap_or_default();
    for instance in config.instances.values() {
        // Try to stop and remove each instance, ignore errors
        let _ = StopCommand::new(&instance.name).execute().await;
        let _ = RmCommand::new(&instance.name).force().execute().await;
    }
    // Clear the config
    if let Ok(mut config) = Config::load() {
        config.instances.clear();
        let _ = config.save();
    }
}

/// Helper to connect to Redis and verify it's working
async fn verify_redis_connection(port: u16, password: Option<&str>) -> Result<()> {
    let url = if let Some(pwd) = password {
        format!("redis://default:{}@127.0.0.1:{}", pwd, port)
    } else {
        format!("redis://127.0.0.1:{}", port)
    };

    // Give Redis time to fully start
    sleep(Duration::from_secs(2)).await;

    let client = redis::Client::open(url)?;
    let mut con = client.get_multiplexed_async_connection().await?;

    // Test basic Redis operations
    let _: () = con.set("test_key", "test_value").await?;
    let value: String = con.get("test_key").await?;
    assert_eq!(value, "test_value");

    // Test PING
    let pong: String = redis::cmd("PING").query_async(&mut con).await?;
    assert_eq!(pong, "PONG");

    // Test INFO command
    let info: String = redis::cmd("INFO")
        .arg("server")
        .query_async(&mut con)
        .await?;
    assert!(info.contains("redis_version"));

    Ok(())
}

#[cfg(feature = "docker-tests")]
mod docker_deployment_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_basic_redis_cli_deployment() -> Result<()> {
        cleanup_all().await;

        // Start Basic Redis via CLI args
        let args = BasicStartArgs {
            name: Some("test-basic-cli".to_string()),
            port: 16379,
            password: Some("testpass123".to_string()),
            persist: false,
            memory: Some("256m".to_string()),
            with_insight: false,
            insight_port: 8001,
            shell: false,
        };

        commands::basic::handle_action(RedisAction::Start(args), false).await?;

        // Verify Redis is working
        verify_redis_connection(16379, Some("testpass123")).await?;

        // Stop the instance
        let stop_args = StopArgs {
            name: Some("test-basic-cli".to_string()),
        };
        commands::basic::handle_action(RedisAction::Stop(stop_args), false).await?;

        cleanup_all().await;
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_basic_redis_yaml_deployment() -> Result<()> {
        cleanup_all().await;

        // Create a YAML config file
        let temp_dir = TempDir::new()?;
        let yaml_path = temp_dir.path().join("basic.yaml");
        std::fs::write(
            &yaml_path,
            r#"api-version: v1
deployments:
  - name: test-basic-yaml
    type: basic
    port: 16380
    password: yamlpass456
    memory: "256m"
    persist: false
"#,
        )?;

        // Deploy from YAML
        commands::yaml::deploy_from_yaml(&yaml_path, false).await?;

        // Verify Redis is working
        verify_redis_connection(16380, Some("yamlpass456")).await?;

        // Stop the instance
        let stop_args = StopArgs {
            name: Some("test-basic-yaml".to_string()),
        };
        commands::basic::handle_action(RedisAction::Stop(stop_args), false).await?;

        cleanup_all().await;
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_stack_redis_cli_deployment() -> Result<()> {
        cleanup_all().await;

        // Start Redis Stack via CLI args
        let args = StackStartArgs {
            name: Some("test-stack-cli".to_string()),
            port: 16381,
            password: Some("stackpass789".to_string()),
            persist: false,
            memory: Some("512m".to_string()),
            with_json: true,
            with_search: true,
            with_timeseries: false,
            with_graph: false,
            with_bloom: false,
            demo_bundle: false,
            with_insight: false,
            insight_port: 8001,
            shell: false,
        };

        commands::stack::handle_action(StackAction::Start(args), false).await?;

        // Give Stack more time to start (it's heavier)
        sleep(Duration::from_secs(5)).await;

        // Verify Redis is working
        verify_redis_connection(16381, Some("stackpass789")).await?;

        // Test Redis Stack modules (JSON)
        let client = redis::Client::open(format!("redis://default:stackpass789@127.0.0.1:16381"))?;
        let mut con = client.get_multiplexed_async_connection().await?;

        // Test JSON module
        let _: () = redis::cmd("JSON.SET")
            .arg("user:1")
            .arg("$")
            .arg(r#"{"name":"John","age":30}"#)
            .query_async(&mut con)
            .await?;

        let json_result: String = redis::cmd("JSON.GET")
            .arg("user:1")
            .arg("$")
            .query_async(&mut con)
            .await?;
        assert!(json_result.contains("John"));

        // Stop the instance
        let stop_args = StopArgs {
            name: Some("test-stack-cli".to_string()),
        };
        commands::stack::handle_action(StackAction::Stop(stop_args), false).await?;

        cleanup_all().await;
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_stack_redis_yaml_deployment() -> Result<()> {
        cleanup_all().await;

        // Create a YAML config file
        let temp_dir = TempDir::new()?;
        let yaml_path = temp_dir.path().join("stack.yaml");
        std::fs::write(
            &yaml_path,
            r#"api-version: v1
deployments:
  - name: test-stack-yaml
    type: stack
    port: 16382
    password: stackyaml123
    memory: "512m"
    persist: false
"#,
        )?;

        // Deploy from YAML
        commands::yaml::deploy_from_yaml(&yaml_path, false).await?;

        // Give Stack more time to start
        sleep(Duration::from_secs(5)).await;

        // Verify Redis is working
        verify_redis_connection(16382, Some("stackyaml123")).await?;

        // Stop the instance
        let stop_args = StopArgs {
            name: Some("test-stack-yaml".to_string()),
        };
        commands::stack::handle_action(StackAction::Stop(stop_args), false).await?;

        cleanup_all().await;
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_cluster_redis_cli_deployment() -> Result<()> {
        cleanup_all().await;

        // Start Redis Cluster via CLI args
        let args = ClusterStartArgs {
            name: Some("test-cluster-cli".to_string()),
            masters: 3,
            replicas: 0, // No replicas for faster testing
            port_base: 17000,
            password: Some("clusterpass".to_string()),
            persist: false,
            memory: Some("256m".to_string()),
            stack: false,
            with_insight: false,
            insight_port: 8001,
            shell: false,
        };

        commands::cluster::handle_action(ClusterAction::Start(args), false).await?;

        // Give cluster time to initialize
        sleep(Duration::from_secs(10)).await;

        // Connect to first master node
        let client = redis::Client::open(format!("redis://default:clusterpass@127.0.0.1:17000"))?;
        let mut con = client.get_multiplexed_async_connection().await?;

        // Test cluster info
        let cluster_info: String = redis::cmd("CLUSTER")
            .arg("INFO")
            .query_async(&mut con)
            .await?;
        assert!(cluster_info.contains("cluster_state:ok"));

        // Stop the cluster
        let stop_args = StopArgs {
            name: Some("test-cluster-cli".to_string()),
        };
        commands::cluster::handle_action(ClusterAction::Stop(stop_args), false).await?;

        cleanup_all().await;
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_cluster_redis_yaml_deployment() -> Result<()> {
        cleanup_all().await;

        // Create a YAML config file
        let temp_dir = TempDir::new()?;
        let yaml_path = temp_dir.path().join("cluster.yaml");
        std::fs::write(
            &yaml_path,
            r#"api-version: v1
deployments:
  - name: test-cluster-yaml
    type: cluster
    masters: 3
    replicas: 0
    port-base: 17100
    password: clusteryaml
    memory: "256m"
    persist: false
"#,
        )?;

        // Deploy from YAML
        commands::yaml::deploy_from_yaml(&yaml_path, false).await?;

        // Give cluster time to initialize
        sleep(Duration::from_secs(10)).await;

        // Connect to first master node
        let client = redis::Client::open(format!("redis://default:clusteryaml@127.0.0.1:17100"))?;
        let mut con = client.get_multiplexed_async_connection().await?;

        // Test cluster info
        let cluster_info: String = redis::cmd("CLUSTER")
            .arg("INFO")
            .query_async(&mut con)
            .await?;
        assert!(cluster_info.contains("cluster_state:ok"));

        // Stop the cluster
        let stop_args = StopArgs {
            name: Some("test-cluster-yaml".to_string()),
        };
        commands::cluster::handle_action(ClusterAction::Stop(stop_args), false).await?;

        cleanup_all().await;
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_sentinel_redis_cli_deployment() -> Result<()> {
        cleanup_all().await;

        // Start Redis Sentinel via CLI args
        let args = SentinelStartArgs {
            name: Some("test-sentinel-cli".to_string()),
            masters: 1,
            sentinels: 3,
            redis_port_base: 16400,
            sentinel_port_base: 26400,
            password: Some("sentinelpass".to_string()),
            persist: false,
            memory: Some("256m".to_string()),
            with_insight: false,
            insight_port: 8001,
        };

        commands::sentinel::handle_action(SentinelAction::Start(args), false).await?;

        // Give Sentinel time to initialize
        sleep(Duration::from_secs(8)).await;

        // Connect to the master
        verify_redis_connection(16400, Some("sentinelpass")).await?;

        // Connect to a sentinel to verify it's monitoring
        let sentinel_client = redis::Client::open("redis://127.0.0.1:26400")?;
        let mut sentinel_con = sentinel_client.get_multiplexed_async_connection().await?;

        // Check sentinel masters
        let masters: Vec<Vec<String>> = redis::cmd("SENTINEL")
            .arg("masters")
            .query_async(&mut sentinel_con)
            .await?;
        assert!(!masters.is_empty());

        // Stop the sentinel setup
        let stop_args = StopArgs {
            name: Some("test-sentinel-cli".to_string()),
        };
        commands::sentinel::handle_action(SentinelAction::Stop(stop_args), false).await?;

        cleanup_all().await;
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_sentinel_redis_yaml_deployment() -> Result<()> {
        cleanup_all().await;

        // Create a YAML config file
        let temp_dir = TempDir::new()?;
        let yaml_path = temp_dir.path().join("sentinel.yaml");
        std::fs::write(
            &yaml_path,
            r#"api-version: v1
deployments:
  - name: test-sentinel-yaml
    type: sentinel
    sentinels: 3
    redis-port-base: 16500
    sentinel-port-base: 26500
    password: sentinelyaml
    memory: "256m"
    persist: false
"#,
        )?;

        // Deploy from YAML
        commands::yaml::deploy_from_yaml(&yaml_path, false).await?;

        // Give Sentinel time to initialize
        sleep(Duration::from_secs(8)).await;

        // Connect to the master
        verify_redis_connection(16500, Some("sentinelyaml")).await?;

        // Connect to a sentinel
        let sentinel_client = redis::Client::open("redis://127.0.0.1:26500")?;
        let mut sentinel_con = sentinel_client.get_multiplexed_async_connection().await?;

        // Check sentinel masters
        let masters: Vec<Vec<String>> = redis::cmd("SENTINEL")
            .arg("masters")
            .query_async(&mut sentinel_con)
            .await?;
        assert!(!masters.is_empty());

        // Stop the sentinel setup
        let stop_args = StopArgs {
            name: Some("test-sentinel-yaml".to_string()),
        };
        commands::sentinel::handle_action(SentinelAction::Stop(stop_args), false).await?;

        cleanup_all().await;
        Ok(())
    }

    // Note: Redis Enterprise tests are commented out as they require more resources
    // and take longer to initialize. Uncomment if you have sufficient resources.

    /*
    #[tokio::test]
    #[serial]
    async fn test_enterprise_redis_cli_deployment() -> Result<()> {
        cleanup_all().await;

        // Start Redis Enterprise via CLI args
        let args = EnterpriseStartArgs {
            name: Some("test-enterprise-cli".to_string()),
            nodes: 1,  // Single node for testing
            port_base: 18443,
            create_db: Some("testdb".to_string()),
            db_port: 12000,
            memory: Some("2g".to_string()),
            persist: false,
            containers_only: false,
            with_insight: false,
            insight_port: 8001,
        };

        commands::enterprise::handle_action(
            redis_dev::cli::EnterpriseAction::Start(args),
            false,
        )
        .await?;

        // Enterprise takes a long time to initialize
        sleep(Duration::from_secs(30)).await;

        // Try to connect to the database port
        // Note: Enterprise requires authentication setup
        verify_redis_connection(12000, None).await?;

        // Stop the enterprise cluster
        let stop_args = StopArgs {
            name: Some("test-enterprise-cli".to_string()),
        };
        commands::enterprise::handle_action(
            redis_dev::cli::EnterpriseAction::Stop(stop_args),
            false,
        )
        .await?;

        cleanup_all().await;
        Ok(())
    }
    */

    #[tokio::test]
    #[serial]
    async fn test_multi_deployment_yaml() -> Result<()> {
        cleanup_all().await;

        // Create a YAML config with multiple deployments
        let temp_dir = TempDir::new()?;
        let yaml_path = temp_dir.path().join("multi.yaml");
        std::fs::write(
            &yaml_path,
            r#"api-version: v1
deployments:
  - name: test-multi-basic
    type: basic
    port: 16600
    password: multipass1
    memory: "128m"
    
  - name: test-multi-stack
    type: stack
    port: 16601
    password: multipass2
    memory: "256m"
"#,
        )?;

        // Deploy from YAML
        commands::yaml::deploy_from_yaml(&yaml_path, false).await?;

        // Give both instances time to start
        sleep(Duration::from_secs(5)).await;

        // Verify both instances are working
        verify_redis_connection(16600, Some("multipass1")).await?;
        verify_redis_connection(16601, Some("multipass2")).await?;

        // Stop both instances
        let stop_args1 = StopArgs {
            name: Some("test-multi-basic".to_string()),
        };
        commands::basic::handle_action(RedisAction::Stop(stop_args1), false).await?;

        let stop_args2 = StopArgs {
            name: Some("test-multi-stack".to_string()),
        };
        commands::stack::handle_action(StackAction::Stop(stop_args2), false).await?;

        cleanup_all().await;
        Ok(())
    }
}

#[cfg(not(feature = "docker-tests"))]
mod docker_deployment_tests {
    use super::*;

    #[test]
    fn docker_tests_disabled() {
        println!("Docker integration tests are disabled.");
        println!("Run with: cargo test --features docker-tests");
    }
}
