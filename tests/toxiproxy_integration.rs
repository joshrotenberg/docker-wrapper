//! Integration tests for the Toxiproxy template.
//!
//! These tests start a real Toxiproxy container (plus a Redis container behind a
//! proxy) and exercise the control API end to end: registering a proxy, sending
//! traffic through the published proxy port, and injecting a latency toxic.
//!
//! Requires a running Docker daemon. Each test uses unique container and network
//! names and cleans up after itself.

#![cfg(all(feature = "template-toxiproxy", feature = "template-redis"))]

use docker_wrapper::template::toxiproxy::{Toxic, ToxicStream};
use docker_wrapper::{
    DockerCommand, NetworkCreateCommand, NetworkRmCommand, RedisTemplate, Template,
    ToxiproxyTemplate,
};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Generate a unique suffix for test resources.
fn unique(prefix: &str) -> String {
    format!("{}-{}", prefix, uuid::Uuid::new_v4())
}

/// Pick a host port unlikely to collide with common services.
fn random_port() -> u16 {
    30000 + (uuid::Uuid::new_v4().as_u128() % 10000) as u16
}

/// Send a Redis PING through a TCP socket and return the round-trip duration.
///
/// Returns an error if the connection or exchange fails, or if the reply does not
/// look like a Redis PONG.
async fn ping_through(addr: &str) -> Result<Duration, Box<dyn std::error::Error>> {
    let start = Instant::now();
    let mut stream = TcpStream::connect(addr).await?;
    stream.write_all(b"PING\r\n").await?;
    stream.flush().await?;

    let mut buf = [0u8; 64];
    let n = tokio::time::timeout(Duration::from_secs(5), stream.read(&mut buf)).await??;
    let elapsed = start.elapsed();

    let reply = String::from_utf8_lossy(&buf[..n]);
    if !reply.to_uppercase().contains("PONG") {
        return Err(format!("unexpected reply from {addr}: {reply:?}").into());
    }
    Ok(elapsed)
}

#[tokio::test]
async fn test_toxiproxy_proxy_and_latency_toxic() -> Result<(), Box<dyn std::error::Error>> {
    // Skip gracefully if Docker is unavailable.
    if docker_wrapper::VersionCommand::new()
        .execute()
        .await
        .is_err()
    {
        eprintln!("Docker not available, skipping test");
        return Ok(());
    }

    let network = unique("toxi-net");
    let redis_name = unique("toxi-redis");
    let proxy_name = unique("toxi-proxy");
    let proxy_port = random_port();

    // Shared network so Toxiproxy can reach Redis by container name.
    let _ = NetworkCreateCommand::new(&network)
        .driver("bridge")
        .execute()
        .await;

    // Run the body in a closure so we can always clean up afterwards.
    let result = async {
        // Start Redis on the shared network.
        let mut redis = RedisTemplate::new(&redis_name);
        redis.config_mut().network = Some(network.clone());
        redis.start_and_wait().await?;
        redis.wait_for_ready().await?;

        // Start Toxiproxy on the same network, publishing the control port and the
        // proxy port so the test process can connect through it.
        let mut toxiproxy = ToxiproxyTemplate::new(&proxy_name)
            .proxy_port(proxy_port)
            .api_ready_timeout(Duration::from_secs(30));
        toxiproxy.config_mut().network = Some(network.clone());
        toxiproxy.start_and_wait().await?;
        toxiproxy.wait_for_control_api().await?;

        // Register the proxy: listen on the published port, forward to Redis.
        let proxy = toxiproxy
            .create_proxy(
                "redis",
                format!("0.0.0.0:{proxy_port}"),
                format!("{redis_name}:6379"),
            )
            .await?;
        assert_eq!(proxy.name, "redis");
        assert!(proxy.enabled);

        // The proxy should show up in the listing.
        let proxies = toxiproxy.list_proxies().await?;
        assert!(
            proxies.iter().any(|p| p.name == "redis"),
            "expected 'redis' proxy in listing, got {proxies:?}"
        );

        let addr = format!("127.0.0.1:{proxy_port}");

        // Baseline: PING through the clean proxy should be fast.
        let baseline = ping_through(&addr).await?;

        // Inject 600ms of downstream latency.
        toxiproxy
            .add_toxic(
                "redis",
                "slow",
                ToxicStream::Downstream,
                Toxic::latency(600),
            )
            .await?;

        // Now a PING through the proxy should take noticeably longer.
        let slowed = ping_through(&addr).await?;
        assert!(
            slowed >= Duration::from_millis(500),
            "expected latency toxic to add delay (baseline {baseline:?}, slowed {slowed:?})"
        );

        // Remove the toxic and confirm latency returns to baseline-ish.
        toxiproxy.remove_toxic("redis", "slow").await?;
        let recovered = ping_through(&addr).await?;
        assert!(
            recovered < Duration::from_millis(500),
            "expected latency to recover after removing toxic, got {recovered:?}"
        );

        // Reset should leave the proxy in place but clear toxics.
        toxiproxy.reset().await?;

        // Clean up the containers.
        toxiproxy.stop().await?;
        toxiproxy.remove().await?;
        redis.stop().await?;
        redis.remove().await?;

        Ok::<(), Box<dyn std::error::Error>>(())
    }
    .await;

    // Best-effort cleanup of containers (in case the body returned early) and the
    // network. Ignore errors since resources may already be gone.
    let _ = RedisTemplate::new(&redis_name).remove().await;
    let _ = ToxiproxyTemplate::new(&proxy_name).remove().await;
    let _ = NetworkRmCommand::new(&network).run().await;

    result
}
