//! Redis Enterprise template example
//!
//! This example demonstrates how to use the Redis Enterprise template
//! to spin up a production-grade Redis cluster with automatic initialization.
//!
//! Run with: cargo run --example redis_enterprise --features template-redis-enterprise

#[cfg(feature = "template-redis-enterprise")]
use docker_wrapper::RedisEnterpriseTemplate;

#[cfg(feature = "template-redis-enterprise")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Redis Enterprise Template Example");
    println!("=================================\n");

    // Create Redis Enterprise template
    let redis_enterprise = RedisEnterpriseTemplate::new("demo-enterprise")
        .cluster_name("Development Cluster")
        .admin_username("admin@example.com")
        .admin_password("SecurePass123!")
        .accept_eula() // Required for Redis Enterprise
        .ui_port(8443)
        .api_port(9443)
        .with_database("demo-db") // Create initial database
        .memory_limit("2g"); // Set memory limit

    println!("Starting Redis Enterprise cluster...");
    println!("This may take 10-15 seconds for initialization.\n");

    // Start the cluster
    let connection_info = match redis_enterprise.start().await {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Failed to start Redis Enterprise: {}", e);
            return Err(e.into());
        }
    };

    println!("✅ Redis Enterprise cluster started successfully!\n");

    println!("📊 Cluster Information:");
    println!("  Cluster Name: {}", connection_info.cluster_name);
    println!("  Container: {}", connection_info.container_name);
    println!();

    println!("🌐 Access URLs:");
    println!("  UI: {} (use admin credentials)", connection_info.ui_url());
    println!("  API: {}", connection_info.api_url());

    if let Some(redis_url) = connection_info.redis_url() {
        println!("  Redis: {}", redis_url);
    }
    println!();

    println!("🔑 Credentials:");
    println!("  Username: {}", connection_info.username);
    println!("  Password: {}", connection_info.password);
    println!();

    println!("📝 Quick Start:");
    println!("  1. Open {} in your browser", connection_info.ui_url());
    println!("  2. Login with the admin credentials above");
    println!("  3. Explore the Redis Enterprise management UI");

    if connection_info.database_port.is_some() {
        println!(
            "  4. Connect to Redis at port {}",
            connection_info.database_port.unwrap()
        );
        println!(
            "     redis-cli -p {}",
            connection_info.database_port.unwrap()
        );
    }

    println!("\n⏳ Cluster will run for 30 seconds for testing...");
    println!("Press Ctrl+C to stop earlier.\n");

    // Keep running for testing
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    println!("🧹 Cleaning up...");
    connection_info.stop().await?;
    println!("✅ Cleanup complete!");

    Ok(())
}

#[cfg(not(feature = "template-redis-enterprise"))]
fn main() {
    eprintln!("This example requires the 'template-redis-enterprise' feature.");
    eprintln!(
        "Run with: cargo run --example redis_enterprise --features template-redis-enterprise"
    );
}
