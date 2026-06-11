//! Toxiproxy fault-injection example.
//!
//! Starts a Redis container and a Toxiproxy container on a shared network, routes
//! a published host port through Toxiproxy to Redis, injects latency, and shows
//! the guard-based fault helpers (pause/unpause/crash/partition/heal).
//!
//! Run with:
//!   cargo run --example toxiproxy_fault_injection --features "template-toxiproxy,template-redis,testing"

#[cfg(all(
    feature = "template-toxiproxy",
    feature = "template-redis",
    feature = "testing"
))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use docker_wrapper::template::toxiproxy::{Toxic, ToxicStream};
    use docker_wrapper::testing::ContainerGuard;
    use docker_wrapper::{RedisTemplate, ToxiproxyTemplate};
    use std::time::Duration;

    let network = "toxiproxy-example-net";
    let proxy_port = 16379;

    println!("Starting Redis on network '{network}'...");
    let redis = ContainerGuard::new(RedisTemplate::new("toxi-example-redis"))
        .with_network(network)
        .remove_network_on_drop(true)
        .wait_for_ready(true)
        .start()
        .await?;

    println!("Starting Toxiproxy on the same network...");
    let toxiproxy_template = ToxiproxyTemplate::new("toxi-example-proxy")
        .network(network)
        .proxy_port(proxy_port);
    let toxiproxy = ContainerGuard::new(toxiproxy_template)
        .with_network(network)
        .start()
        .await?;

    // Wait for the control API, then register a proxy: clients hit the published
    // host port, Toxiproxy forwards to the Redis container by name.
    toxiproxy.template().wait_for_control_api().await?;
    toxiproxy
        .template()
        .create_proxy(
            "redis",
            format!("0.0.0.0:{proxy_port}"),
            "toxi-example-redis:6379",
        )
        .await?;
    println!("Proxy registered: localhost:{proxy_port} -> toxi-example-redis:6379");

    // Inject 750ms of downstream latency on the proxy.
    toxiproxy
        .template()
        .add_toxic(
            "redis",
            "slow",
            ToxicStream::Downstream,
            Toxic::latency(750),
        )
        .await?;
    println!("Added a 750ms latency toxic. Connect to localhost:{proxy_port} to feel it.");

    let proxies = toxiproxy.template().list_proxies().await?;
    println!("Active proxies: {}", proxies.len());

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Remove the toxic and reset Toxiproxy to a clean state.
    toxiproxy.template().remove_toxic("redis", "slow").await?;
    toxiproxy.template().reset().await?;
    println!("Latency toxic removed; Toxiproxy reset.");

    // Demonstrate the guard fault helpers against the Redis container.
    println!("Pausing Redis (simulated hang)...");
    redis.pause().await?;
    redis.unpause().await?;
    println!("Resumed Redis.");

    println!("Partitioning Redis from the network, then healing...");
    redis.partition(network).await?;
    redis.heal(network).await?;
    println!("Redis reconnected.");

    println!("Crashing Redis (SIGKILL)...");
    redis.crash().await?;
    println!("Redis killed.");

    // Containers and the network are cleaned up automatically when the guards drop.
    println!("Done. Cleaning up containers and network.");
    Ok(())
}

#[cfg(not(all(
    feature = "template-toxiproxy",
    feature = "template-redis",
    feature = "testing"
)))]
fn main() {
    eprintln!(
        "This example requires the 'template-toxiproxy', 'template-redis', and 'testing' features."
    );
    eprintln!(
        "Run with: cargo run --example toxiproxy_fault_injection --features \"template-toxiproxy,template-redis,testing\""
    );
}
