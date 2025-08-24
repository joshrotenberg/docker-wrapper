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

    // Example 3: Redis Stack cluster with RedisInsight
    println!("\n3. Starting Redis Stack Cluster with RedisInsight...");
    let stack_cluster = RedisClusterTemplate::new("example-cluster-stack")
        .num_masters(3)
        .num_replicas(1)
        .port_base(9000)
        .with_redis_stack() // Use Redis Stack for additional modules
        .with_redis_insight() // Enable RedisInsight UI
        .redis_insight_port(8001)
        .password("stack-password")
        .with_persistence("redis-stack-data");

    let result = stack_cluster.start().await?;
    println!("   {}", result);
    println!("   RedisInsight is available at http://localhost:8001");
    println!("   Redis Stack includes: JSON, Search, Graph, TimeSeries, Bloom modules");

    // Example 4: Advanced cluster configuration
    println!("\n4. Starting advanced Redis Cluster...");
    let advanced_cluster = RedisClusterTemplate::new("example-cluster-advanced")
        .num_masters(5)
        .num_replicas(2) // 2 replicas per master = 15 total nodes
        .port_base(10000)
        .password("secure-password")
        .cluster_announce_ip("192.168.1.100")
        .cluster_node_timeout(10000)
        .memory_limit("256m")
        .with_persistence("redis-cluster-advanced")
        .auto_remove();

    let result = advanced_cluster.start().await?;
    println!("   {}", result);

    // Check cluster status (for basic cluster)
    println!("\n5. Checking cluster status...");
    match basic_cluster.cluster_info().await {
        Ok(info) => {
            println!("   Cluster state: {}", info.cluster_state);
            println!("   Total slots: {}", info.total_slots);
            println!("   Number of nodes: {}", info.nodes.len());
        }
        Err(e) => println!("   Could not get cluster info: {}", e),
    }

    // Clean up
    println!("\n6. Cleaning up clusters...");

    println!("   Stopping basic cluster...");
    basic_cluster.stop().await?;
    basic_cluster.remove().await?;

    println!("   Stopping replicated cluster...");
    replicated_cluster.stop().await?;
    replicated_cluster.remove().await?;

    println!("   Stopping stack cluster...");
    stack_cluster.stop().await?;
    stack_cluster.remove().await?;

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
