//! Example demonstrating Redis Cluster template usage
//!
//! This example shows how to use the RedisClusterTemplate to quickly
//! set up a multi-node Redis cluster with sharding and replication.

#[cfg(feature = "template-redis-cluster")]
use docker_wrapper::{RedisClusterConnection, RedisClusterTemplate, Template};

#[cfg(feature = "template-redis-cluster")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Redis Cluster Template Example\n");

    // Example 1: Basic 3-master cluster
    println!("1. Starting basic Redis Cluster with 3 masters...");
    let basic_cluster = RedisClusterTemplate::new("example-cluster-basic")
        .num_masters(3)
        .port_base(7000);

    let result = basic_cluster.start().await?;
    println!("   {}", result);

    // Get connection info
    let conn = RedisClusterConnection::from_template(&basic_cluster);
    println!("   Cluster nodes: {}", conn.nodes_string());
    println!("   Cluster URL: {}", conn.cluster_url());

    // Example 2: Cluster with replicas
    println!("\n2. Starting Redis Cluster with replicas...");
    let replicated_cluster = RedisClusterTemplate::new("example-cluster-replicated")
        .num_masters(3)
        .num_replicas(1) // 1 replica per master = 6 total nodes
        .port_base(8000)
        .password("cluster-password")
        .with_persistence("redis-cluster-data");

    let result = replicated_cluster.start().await?;
    println!("   {}", result);

    // Example 3: Advanced cluster configuration
    println!("\n3. Starting advanced Redis Cluster...");
    let advanced_cluster = RedisClusterTemplate::new("example-cluster-advanced")
        .num_masters(5)
        .num_replicas(2) // 2 replicas per master = 15 total nodes
        .port_base(9000)
        .password("secure-password")
        .cluster_announce_ip("192.168.1.100")
        .cluster_node_timeout(10000)
        .memory_limit("256m")
        .with_persistence("redis-cluster-advanced")
        .auto_remove();

    let result = advanced_cluster.start().await?;
    println!("   {}", result);

    // Check cluster status (for basic cluster)
    println!("\n4. Checking cluster status...");
    match basic_cluster.cluster_info().await {
        Ok(info) => {
            println!("   Cluster state: {}", info.cluster_state);
            println!("   Total slots: {}", info.total_slots);
            println!("   Number of nodes: {}", info.nodes.len());
        }
        Err(e) => println!("   Could not get cluster info: {}", e),
    }

    // Clean up
    println!("\n5. Cleaning up clusters...");

    println!("   Stopping basic cluster...");
    basic_cluster.stop().await?;
    basic_cluster.remove().await?;

    println!("   Stopping replicated cluster...");
    replicated_cluster.stop().await?;
    replicated_cluster.remove().await?;

    println!("   Stopping advanced cluster...");
    advanced_cluster.stop().await?;
    advanced_cluster.remove().await?;

    println!("\nRedis Cluster examples completed!");
    Ok(())
}

#[cfg(not(feature = "template-redis-cluster"))]
fn main() {
    println!("This example requires the 'template-redis-cluster' feature.");
    println!("Run with: cargo run --example redis_cluster --features template-redis-cluster");
}
