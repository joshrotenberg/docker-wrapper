//! Examples demonstrating system cleanup and maintenance operations.
//!
//! This example shows how to use Docker system prune, df, and other
//! cleanup commands to manage Docker resources.

use docker_wrapper::{ContainerPruneCommand, ImagePruneCommand, SystemDfCommand};

#[tokio::main]
async fn main() {
    println!("Docker System Cleanup Examples");
    println!("==============================\n");

    // Example 1: Check disk usage
    println!("1. Checking Docker disk usage:");
    match SystemDfCommand::new().verbose().run().await {
        Ok(usage) => {
            println!("Docker Disk Usage:");
            println!("  Images: {} items", usage.images.len());
            println!("  Containers: {} items", usage.containers.len());
            println!("  Volumes: {} items", usage.volumes.len());
            println!("  Build Cache: {} items", usage.build_cache.len());
        }
        Err(e) => {
            println!("Failed to get disk usage: {e}");
        }
    }

    println!();

    // Example 2: Prune stopped containers
    println!("2. Pruning stopped containers:");
    match ContainerPruneCommand::new().force().run().await {
        Ok(result) => {
            println!(
                "Removed {} containers, reclaimed {} bytes",
                result.containers_deleted.len(),
                result.space_reclaimed
            );
        }
        Err(e) => {
            println!("Failed to prune containers: {e}");
        }
    }

    println!();

    // Example 3: Prune dangling images
    println!("3. Pruning dangling images:");
    match ImagePruneCommand::new().dangling_only().force().run().await {
        Ok(result) => {
            println!(
                "Removed {} images, reclaimed {} bytes",
                result.images_deleted.len(),
                result.space_reclaimed
            );
        }
        Err(e) => {
            println!("Failed to prune images: {e}");
        }
    }

    println!();

    // Example 4: System-wide prune (be careful!)
    println!("4. System-wide prune (dry run - not executing):");
    println!("   Would run: docker system prune --all --volumes --force");
    println!("   This would remove:");
    println!("   - All stopped containers");
    println!("   - All networks not used by at least one container");
    println!("   - All images without at least one container");
    println!("   - All build cache");
    println!("   - All volumes not used by at least one container");

    // Uncomment to actually run (WARNING: destructive!)
    // match SystemPruneCommand::new()
    //     .all()
    //     .volumes()
    //     .force()
    //     .run()
    //     .await
    // {
    //     Ok(result) => {
    //         println!("System prune complete:");
    //         println!("  Containers removed: {}", result.containers_deleted.len());
    //         println!("  Images removed: {}", result.images_deleted.len());
    //         println!("  Networks removed: {}", result.networks_deleted.len());
    //         println!("  Volumes removed: {}", result.volumes_deleted.len());
    //         println!("  Total space reclaimed: {} bytes", result.space_reclaimed);
    //     }
    //     Err(e) => {
    //         println!("Failed to prune system: {e}");
    //     }
    // }

    println!();

    // Example 5: Prune with filters
    println!("5. Prune containers older than 24 hours:");
    match ContainerPruneCommand::new()
        .until("24h")
        .force()
        .run()
        .await
    {
        Ok(result) => {
            println!(
                "Removed {} old containers, reclaimed {} bytes",
                result.containers_deleted.len(),
                result.space_reclaimed
            );
        }
        Err(e) => {
            println!("Failed to prune old containers: {e}");
        }
    }

    println!();

    // Example 6: Label-based cleanup
    println!("6. Prune containers with specific labels:");
    match ContainerPruneCommand::new()
        .with_label("cleanup", Some("true"))
        .force()
        .run()
        .await
    {
        Ok(result) => {
            println!(
                "Removed {} labeled containers, reclaimed {} bytes",
                result.containers_deleted.len(),
                result.space_reclaimed
            );
        }
        Err(e) => {
            println!("Failed to prune labeled containers: {e}");
        }
    }
}
