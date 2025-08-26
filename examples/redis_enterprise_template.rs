//! Redis Enterprise Template Example
//!
//! This example demonstrates using the Redis Enterprise template to:
//! - Start a Redis Enterprise cluster with automatic bootstrap
//! - Test the management API endpoints
//! - Verify cluster initialization

use docker_wrapper::{DockerCommand, RedisEnterpriseTemplate};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("Redis Enterprise Template Test");
    println!("==============================\n");

    // Start Redis Enterprise
    println!("Starting Redis Enterprise...");
    let redis = RedisEnterpriseTemplate::new("redis-enterprise-test")
        .accept_eula()
        .admin_username("admin@redis.local")
        .admin_password("Redis123!")
        .cluster_name("test-cluster")
        .ui_port(8443)
        .api_port(9443)
        .memory_limit("4g");

    let conn_info = match redis.start().await {
        Ok(info) => {
            println!("✅ Redis Enterprise started successfully!");
            println!("   Container: {}", info.container_name);
            println!("   UI URL: {}", info.ui_url);
            println!("   API URL: {}", info.api_url);
            println!("   Username: {}", info.username);
            info
        }
        Err(e) => {
            println!("❌ Failed to start Redis Enterprise: {}", e);
            return Err(e.into());
        }
    };

    // Wait a bit more for full initialization
    println!("\nWaiting for cluster to fully initialize...");
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Test API endpoints
    println!("\n=== Testing API Endpoints ===\n");

    // Test /v1/license endpoint
    println!("Testing /v1/license endpoint...");
    let license_cmd = format!(
        r#"curl -sk -u {}:{} https://localhost:{}/v1/license"#,
        conn_info.username, conn_info.password, 9443
    );

    let license_result = docker_wrapper::ExecCommand::new(
        &conn_info.container_name,
        vec!["sh".to_string(), "-c".to_string(), license_cmd],
    )
    .execute()
    .await?;

    if !license_result.stdout.is_empty() {
        println!("✅ License endpoint response:");
        // Parse and pretty print if it's JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&license_result.stdout) {
            println!("{}", serde_json::to_string_pretty(&json)?);
        } else {
            println!("{}", license_result.stdout);
        }
    } else {
        println!("⚠️  No response from license endpoint");
        if !license_result.stderr.is_empty() {
            println!("   Error: {}", license_result.stderr);
        }
    }

    println!("\n---");

    // Test /v1/bdbs endpoint (databases)
    println!("\nTesting /v1/bdbs endpoint...");
    let bdbs_cmd = format!(
        r#"curl -sk -u {}:{} https://localhost:{}/v1/bdbs"#,
        conn_info.username, conn_info.password, 9443
    );

    let bdbs_result = docker_wrapper::ExecCommand::new(
        &conn_info.container_name,
        vec!["sh".to_string(), "-c".to_string(), bdbs_cmd],
    )
    .execute()
    .await?;

    if !bdbs_result.stdout.is_empty() {
        println!("✅ Databases endpoint response:");
        // Parse and pretty print if it's JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&bdbs_result.stdout) {
            println!("{}", serde_json::to_string_pretty(&json)?);
        } else {
            println!("{}", bdbs_result.stdout);
        }
    } else {
        println!("⚠️  No response from databases endpoint");
        if !bdbs_result.stderr.is_empty() {
            println!("   Error: {}", bdbs_result.stderr);
        }
    }

    println!("\n---");

    // Test cluster info endpoint for good measure
    println!("\nTesting /v1/cluster endpoint...");
    let cluster_cmd = format!(
        r#"curl -sk -u {}:{} https://localhost:{}/v1/cluster"#,
        conn_info.username, conn_info.password, 9443
    );

    let cluster_result = docker_wrapper::ExecCommand::new(
        &conn_info.container_name,
        vec!["sh".to_string(), "-c".to_string(), cluster_cmd],
    )
    .execute()
    .await?;

    if !cluster_result.stdout.is_empty() {
        println!("✅ Cluster endpoint response:");
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&cluster_result.stdout) {
            // Just show a summary
            if let Some(name) = json.get("name") {
                println!("   Cluster Name: {}", name);
            }
            if let Some(version) = json.get("software_version") {
                println!("   Software Version: {}", version);
            }
            if let Some(nodes) = json.get("nodes").and_then(|n| n.as_array()) {
                println!("   Number of Nodes: {}", nodes.len());
            }
        } else {
            println!("{}", cluster_result.stdout);
        }
    }

    println!("\n=== Test Complete ===");
    println!(
        "\nRedis Enterprise is running. Access the UI at: {}",
        conn_info.ui_url
    );
    println!("Username: {}", conn_info.username);
    println!("Password: {}", conn_info.password);

    println!("\nPress Enter to stop and cleanup...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // Cleanup
    println!("Stopping Redis Enterprise...");
    docker_wrapper::StopCommand::new(&conn_info.container_name)
        .execute()
        .await?;

    docker_wrapper::RmCommand::new(&conn_info.container_name)
        .force()
        .volumes()
        .execute()
        .await?;

    println!("✅ Cleanup complete!");

    Ok(())
}
