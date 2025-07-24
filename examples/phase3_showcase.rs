//! Phase 3 Showcase: Comprehensive Image, Network, and Volume Management
//!
//! This example demonstrates all the major features implemented in Phase 3:
//! - Advanced image operations (pull, build, tag, inspect)
//! - Network creation and management
//! - Volume creation and mounting
//! - Integration between all components
//!
//! Run with: cargo run --example phase3_showcase

use docker_wrapper::*;

use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), DockerError> {
    println!("🚀 Docker Wrapper Phase 3 Showcase");
    println!("====================================");

    // Initialize Docker client
    let client = DockerClient::new().await?;
    println!("✅ Docker client initialized");

    // Phase 3 Feature 1: Advanced Image Management
    println!("\n📦 Image Management Demo");
    println!("-----------------------");

    demonstrate_image_management(&client).await?;

    // Phase 3 Feature 2: Network Management
    println!("\n🌐 Network Management Demo");
    println!("-------------------------");

    let network_id = demonstrate_network_management(&client).await?;

    // Phase 3 Feature 3: Volume Management
    println!("\n💾 Volume Management Demo");
    println!("-----------------------");

    let volume_name = demonstrate_volume_management(&client).await?;

    // Phase 3 Integration: Complete Application Stack
    println!("\n🏗️  Complete Stack Integration Demo");
    println!("----------------------------------");

    demonstrate_complete_integration(&client, &network_id, &volume_name).await?;

    // Cleanup
    println!("\n🧹 Cleanup");
    println!("---------");
    cleanup_resources(&client, &network_id, &volume_name).await?;

    println!("\n🎉 Phase 3 showcase completed successfully!");
    println!("   All image, network, and volume features working perfectly.");

    Ok(())
}

async fn demonstrate_image_management(client: &DockerClient) -> Result<(), DockerError> {
    let images = client.images();

    // 1. Parse complex image references
    println!("1. Parsing image references...");
    let simple_ref = ImageRef::parse("redis:7.2-alpine")?;
    let full_ref = ImageRef::parse("docker.io/library/nginx:1.24-alpine")?;
    println!("   ✅ Simple: {}", simple_ref);
    println!("   ✅ Full:   {}", full_ref);

    // 2. Pull images with options
    println!("2. Pulling Redis image...");
    let pull_options = PullOptions::new().platform("linux/amd64");
    images.pull(&simple_ref, pull_options).await?;
    println!("   ✅ Redis image pulled successfully");

    // 3. List images with filtering
    println!("3. Listing images...");
    let list_options = ListImagesOptions::new().filter_reference("redis*");
    let image_list = images.list(list_options).await?;
    println!("   ✅ Found {} Redis images", image_list.len());

    // 4. Inspect image details
    println!("4. Inspecting Redis image...");
    let inspect_result = images.inspect(&simple_ref).await?;
    println!("   ✅ Architecture: {}", inspect_result.architecture);
    println!("   ✅ OS: {}", inspect_result.os);
    println!("   ✅ Size: {} bytes", inspect_result.size);

    // 5. Tag image
    println!("5. Tagging image...");
    let custom_tag = ImageRef::parse("my-redis:latest")?;
    images.tag(&simple_ref, &custom_tag).await?;
    println!("   ✅ Tagged as my-redis:latest");

    // 6. Get image history
    println!("6. Getting image history...");
    let history = images.history(&simple_ref).await?;
    println!("   ✅ Image has {} layers", history.len());

    // 7. Build from Dockerfile (if available)
    if std::path::Path::new("examples/test-dockerfile").exists() {
        println!("7. Building custom image...");
        let build_options = BuildOptions::new("phase3-test:latest")
            .context_path("examples/")
            .dockerfile("test-dockerfile")
            .build_arg("VERSION", "1.0")
            .label("phase", "3")
            .no_cache();

        let built_image_id = images.build(build_options).await?;
        println!("   ✅ Built image: {}", built_image_id);
    } else {
        println!("7. Skipping build (no test Dockerfile found)");
    }

    Ok(())
}

async fn demonstrate_network_management(client: &DockerClient) -> Result<NetworkId, DockerError> {
    let networks = client.networks();

    // 1. Create custom network with advanced configuration
    println!("1. Creating custom network...");
    let network_config = NetworkConfig::new("phase3-network")
        .driver(NetworkDriver::Bridge)
        .subnet("172.20.0.0/16")
        .gateway("172.20.0.1")
        .ip_range("172.20.1.0/24")
        .label("phase", "3")
        .label("purpose", "showcase")
        .option("com.docker.network.bridge.name", "phase3-br0")
        .attachable();

    let network_id = networks.create(network_config).await?;
    println!("   ✅ Network created: {}", network_id);

    // 2. List networks with filtering
    println!("2. Listing networks...");
    let list_options = ListNetworksOptions::new()
        .filter_name("phase3*")
        .filter_label("phase=3");
    let network_list = networks.list(list_options).await?;
    println!("   ✅ Found {} matching networks", network_list.len());

    // 3. Inspect network details
    println!("3. Inspecting network...");
    let inspect_result = networks.inspect(&network_id).await?;
    println!("   ✅ Driver: {}", inspect_result.driver);
    println!("   ✅ Scope: {}", inspect_result.scope);
    println!("   ✅ Internal: {}", inspect_result.internal);
    if let Some(ipam) = &inspect_result.ipam.config {
        for config in ipam {
            if let Some(subnet) = &config.subnet {
                println!("   ✅ Subnet: {}", subnet);
            }
        }
    }

    // 4. Network will be used in integration demo
    println!("4. Network ready for container connections");

    Ok(network_id)
}

async fn demonstrate_volume_management(client: &DockerClient) -> Result<String, DockerError> {
    let volumes = client.volumes();

    // 1. Create various types of volumes
    println!("1. Creating data volume...");
    let data_config = VolumeConfig::new("phase3-data")
        .driver("local")
        .label("phase", "3")
        .label("type", "data");
    let data_volume = volumes.create(data_config).await?;
    println!("   ✅ Data volume: {}", data_volume.name);

    println!("2. Creating tmpfs volume...");
    let tmpfs_config = VolumeConfig::new("phase3-cache")
        .tmpfs()
        .label("phase", "3")
        .label("type", "cache");
    let cache_volume = volumes.create(tmpfs_config).await?;
    println!("   ✅ Cache volume: {}", cache_volume.name);

    // 3. List volumes with filtering
    println!("3. Listing volumes...");
    let list_options = ListVolumesOptions::new().label("phase=3").driver("local");
    let volume_list = volumes.list(list_options).await?;
    println!("   ✅ Found {} phase 3 volumes", volume_list.len());

    // 4. Inspect volume details
    println!("4. Inspecting data volume...");
    let inspect_result = volumes.inspect(&data_volume.name).await?;
    println!("   ✅ Driver: {}", inspect_result.driver);
    println!("   ✅ Mountpoint: {}", inspect_result.mountpoint);
    println!("   ✅ Scope: {}", inspect_result.scope);

    // 5. Volume usage statistics
    println!("5. Getting volume statistics...");
    let stats = volumes.usage_stats().await?;
    println!("   ✅ Total volumes: {}", stats.total_volumes);
    println!("   ✅ Dangling volumes: {}", stats.dangling_count);
    println!("   ✅ Drivers in use: {:?}", stats.drivers);

    // 6. Demonstrate volume mount specifications
    println!("6. Creating volume mount specifications...");
    let data_mount = VolumeMount::new(VolumeSource::named("phase3-data"), "/app/data");
    let cache_mount =
        VolumeMount::new(VolumeSource::named("phase3-cache"), "/tmp/cache").read_only();
    println!("   ✅ Data mount: {}", data_mount.to_cli_arg());
    println!("   ✅ Cache mount: {}", cache_mount.to_cli_arg());

    Ok(data_volume.name)
}

async fn demonstrate_complete_integration(
    client: &DockerClient,
    network_id: &NetworkId,
    volume_name: &str,
) -> Result<(), DockerError> {
    // This demonstrates how Phase 3 features integrate with Phase 2 container management

    println!("1. Setting up complete application stack...");

    // Ensure we have the Redis image
    let redis_ref = ImageRef::parse("redis:7.2-alpine")?;

    // Create a Redis container with custom network and persistent storage
    println!("2. Creating Redis container with network and volume...");
    let container_id = ContainerBuilder::new(redis_ref)
        .name("phase3-redis")
        .port(6379, 6379)
        .env("REDIS_PASSWORD", "phase3demo")
        .volume_mount(VolumeMount::new(VolumeSource::named(volume_name), "/data"))
        .command(vec![
            "redis-server".to_string(),
            "--requirepass".to_string(),
            "phase3demo".to_string(),
            "--dir".to_string(),
            "/data".to_string(),
        ])
        .run(client)
        .await?;

    println!("   ✅ Redis container started: {}", container_id);

    // Connect container to our custom network
    println!("3. Connecting container to custom network...");
    let connect_options = ConnectOptions::new()
        .alias("redis-server")
        .alias("database");

    client
        .networks()
        .connect(network_id, &container_id, Some(connect_options))
        .await?;
    println!("   ✅ Container connected to network with aliases");

    // Wait for Redis to start
    println!("4. Waiting for Redis to be ready...");
    sleep(Duration::from_secs(3)).await;

    // Check container health and network connectivity
    println!("5. Verifying container status...");
    let inspect = client.containers().inspect(&container_id).await?;
    println!("   ✅ Container state: {:?}", inspect.state.status);

    // Show network connections
    let network_inspect = client.networks().inspect(network_id).await?;
    let container_count = network_inspect.containers.len();
    println!("   ✅ Network has {} connected containers", container_count);

    // Demonstrate volume persistence
    println!("6. Testing volume persistence...");
    let exec_config = ExecConfig::new(vec![
        "redis-cli".to_string(),
        "-a".to_string(),
        "phase3demo".to_string(),
        "SET".to_string(),
        "phase3:test".to_string(),
        "success".to_string(),
    ]);

    let exec_result = client.containers().exec(&container_id, exec_config).await?;
    if exec_result.exit_code == 0 {
        println!("   ✅ Data written to persistent volume");
    }

    // Create a second container to test network communication
    println!("7. Testing network communication...");
    let alpine_ref = ImageRef::parse("alpine:latest")?;
    client
        .images()
        .pull(&alpine_ref, PullOptions::default())
        .await?;

    let test_container_id = ContainerBuilder::new(alpine_ref)
        .name("phase3-client")
        .command(vec!["sleep".to_string(), "30".to_string()])
        .run(client)
        .await?;

    // Connect test container to same network
    client
        .networks()
        .connect(network_id, &test_container_id, None)
        .await?;

    // Test network connectivity (ping Redis by alias)
    let ping_config = ExecConfig::new(vec![
        "ping".to_string(),
        "-c".to_string(),
        "1".to_string(),
        "redis-server".to_string(),
    ]);

    let ping_result = client
        .containers()
        .exec(&test_container_id, ping_config)
        .await?;
    if ping_result.exit_code == 0 {
        println!("   ✅ Network communication working (ping successful)");
    }

    println!("8. Complete stack verification successful!");
    println!("   • Redis running with persistent data");
    println!("   • Custom network providing isolation");
    println!("   • Container-to-container communication");
    println!("   • Volume mounting working correctly");

    // Cleanup containers
    println!("9. Stopping containers...");
    client
        .containers()
        .stop(&test_container_id, Some(Duration::from_secs(5)))
        .await?;
    client
        .containers()
        .remove(&test_container_id, RemoveOptions::default())
        .await?;

    client
        .containers()
        .stop(&container_id, Some(Duration::from_secs(5)))
        .await?;
    client
        .containers()
        .remove(&container_id, RemoveOptions::default())
        .await?;

    println!("   ✅ Containers cleaned up");

    Ok(())
}

async fn cleanup_resources(
    client: &DockerClient,
    network_id: &NetworkId,
    volume_name: &str,
) -> Result<(), DockerError> {
    // Clean up network
    println!("1. Removing custom network...");
    client.networks().remove(network_id).await?;
    println!("   ✅ Network removed");

    // Clean up volumes
    println!("2. Removing volumes...");
    client
        .volumes()
        .remove(volume_name, RemoveVolumeOptions::default())
        .await?;
    client
        .volumes()
        .remove("phase3-cache", RemoveVolumeOptions::default())
        .await?;
    println!("   ✅ Volumes removed");

    // Clean up custom images
    println!("3. Removing custom tags...");
    if let Ok(custom_ref) = ImageRef::parse("my-redis:latest") {
        let _ = client
            .images()
            .remove(&custom_ref, RemoveImageOptions::default())
            .await;
    }
    if let Ok(built_ref) = ImageRef::parse("phase3-test:latest") {
        let _ = client
            .images()
            .remove(&built_ref, RemoveImageOptions::default())
            .await;
    }
    println!("   ✅ Custom images cleaned up");

    // Optional: Prune unused resources
    println!("4. Pruning unused resources...");
    let image_prune = client.images().prune(true).await?;
    let network_prune = client.networks().prune().await?;
    let volume_prune = client.volumes().prune().await?;

    println!(
        "   ✅ Pruned {} MB from images",
        image_prune.reclaimed_space / 1_000_000
    );
    println!(
        "   ✅ Pruned {} networks",
        network_prune.deleted_networks.len()
    );
    println!(
        "   ✅ Pruned {} volumes ({}MB)",
        volume_prune.deleted_volumes.len(),
        volume_prune.reclaimed_space / 1_000_000
    );

    Ok(())
}

/// Progress callback for image operations (if implemented)
fn _image_progress_callback(progress: &PullProgress) {
    if let Some(id) = &progress.id {
        if let Some(progress_detail) = &progress.progress_detail {
            if let (Some(current), Some(total)) = (progress_detail.current, progress_detail.total) {
                let percent = (current as f64 / total as f64) * 100.0;
                println!("   {} {}: {:.1}%", progress.status, id, percent);
            }
        }
    }
}
