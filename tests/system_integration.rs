//! Integration tests for Docker system and prune commands

use docker_wrapper::command::system::{SystemDfCommand, SystemPruneCommand};
use docker_wrapper::{ContainerPruneCommand, DockerCommand, ImagePruneCommand};

#[tokio::test]
async fn test_system_df_command() {
    let df_cmd = SystemDfCommand::new().verbose().format("json");

    let args = df_cmd.build_command_args();
    assert!(args.contains(&"system".to_string()));
    assert!(args.contains(&"df".to_string()));
    assert!(args.contains(&"--verbose".to_string()));
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"json".to_string()));
}

#[tokio::test]
async fn test_system_prune_command() {
    let prune_cmd = SystemPruneCommand::new()
        .all()
        .force()
        .volumes()
        .filter("until", "24h")
        .filter("label", "test");

    let args = prune_cmd.build_command_args();
    assert!(args.contains(&"system".to_string()));
    assert!(args.contains(&"prune".to_string()));
    assert!(args.contains(&"--all".to_string()));
    assert!(args.contains(&"--force".to_string()));
    assert!(args.contains(&"--volumes".to_string()));
    assert!(args.contains(&"--filter".to_string()));
    assert!(args.contains(&"until=24h".to_string()));
    assert!(args.contains(&"label=test".to_string()));
}

#[tokio::test]
async fn test_container_prune_command() {
    let prune_cmd = ContainerPruneCommand::new()
        .force()
        .filter("until", "24h")
        .filter("label", "environment=test");

    let args = prune_cmd.build_command_args();
    assert!(args.contains(&"container".to_string()));
    assert!(args.contains(&"prune".to_string()));
    assert!(args.contains(&"--force".to_string()));
    assert!(args.contains(&"--filter".to_string()));
    assert!(args.contains(&"until=24h".to_string()));
    assert!(args.contains(&"label=environment=test".to_string()));
}

#[tokio::test]
async fn test_image_prune_command() {
    let prune_cmd = ImagePruneCommand::new()
        .all()
        .force()
        .filter("until", "7d")
        .filter("dangling", "true");

    let args = prune_cmd.build_command_args();
    assert!(args.contains(&"image".to_string()));
    assert!(args.contains(&"prune".to_string()));
    assert!(args.contains(&"--all".to_string()));
    assert!(args.contains(&"--force".to_string()));
    assert!(args.contains(&"--filter".to_string()));
    assert!(args.contains(&"until=7d".to_string()));
    assert!(args.contains(&"dangling=true".to_string()));
}

#[tokio::test]
async fn test_system_df_with_docker() {
    // Only run if Docker is available
    let result = SystemDfCommand::new().execute().await;

    if result.is_ok() {
        let output = result.unwrap();
        // The df command should return information about disk usage
        assert!(!output.stdout.is_empty());
        // Should contain information about images, containers, and volumes
        assert!(
            output.stdout.contains("Images")
                || output.stdout.contains("IMAGES")
                || output.stdout.contains("images")
        );
    } else {
        eprintln!("Skipping test - Docker not available");
    }
}

#[tokio::test]
async fn test_prune_dry_run() {
    // Test that we can build prune commands without actually executing them
    let system_prune = SystemPruneCommand::new()
        .all()
        .volumes()
        .filter("label", "test-only");

    // Just verify command construction
    let args = system_prune.build_command_args();
    assert!(!args.is_empty());

    let container_prune = ContainerPruneCommand::new().filter("until", "1h");

    let args = container_prune.build_command_args();
    assert!(args.contains(&"container".to_string()));
    assert!(args.contains(&"prune".to_string()));

    let image_prune = ImagePruneCommand::new().all();

    let args = image_prune.build_command_args();
    assert!(args.contains(&"image".to_string()));
    assert!(args.contains(&"prune".to_string()));
    assert!(args.contains(&"--all".to_string()));
}
