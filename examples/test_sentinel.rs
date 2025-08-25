//! Test the Redis Sentinel template
//!
//! Run with: cargo run --example test_sentinel --all-features

use docker_wrapper::RedisSentinelTemplate;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Setting up Redis Sentinel cluster...");

    // Create a Redis Sentinel cluster
    let sentinel = RedisSentinelTemplate::new("test-sentinel")
        .master_name("mymaster")
        .num_replicas(2)
        .num_sentinels(3)
        .quorum(2)
        .password("sentinel-password")
        .with_persistence();

    // Start the cluster
    let connection_info = sentinel.start().await?;

    println!("\n✅ Redis Sentinel cluster started successfully!");
    println!("\n📊 Cluster Information:");
    println!(
        "  Master: {}:{}",
        connection_info.master_host, connection_info.master_port
    );
    println!("  Replicas: {:?}", connection_info.replica_ports);
    println!("  Sentinels:");
    for sentinel in &connection_info.sentinels {
        println!("    - {}:{}", sentinel.host, sentinel.port);
    }
    println!("\n🔗 Connection URLs:");
    println!("  Master URL: {}", connection_info.master_url());
    println!("  Sentinel URLs: {:?}", connection_info.sentinel_urls());

    println!("\n📦 Containers:");
    for container in &connection_info.containers {
        println!("    - {}", container);
    }

    println!("\n⏳ Cluster will run for 5 seconds for testing...");
    println!("You can test failover by stopping the master container:");
    println!("  docker stop {}-master", connection_info.name);

    // Keep running for testing
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    println!("\n🧹 Cleaning up...");
    connection_info.stop().await?;
    println!("✅ Cleanup complete!");

    Ok(())
}
