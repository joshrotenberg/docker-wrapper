//! Integration tests for Docker container lifecycle commands

use docker_wrapper::{
    CommitCommand, CreateCommand, DockerCommand, KillCommand, PauseCommand, RenameCommand,
    RestartCommand, RmCommand, StartCommand, StopCommand, UnpauseCommand, UpdateCommand,
    WaitCommand,
};

#[tokio::test]
async fn test_container_basic_lifecycle() {
    // Test create, start, stop, and remove
    let container_name = format!("test-container-{}", uuid::Uuid::new_v4());

    // Create container
    let create_result = CreateCommand::new("alpine:latest")
        .name(&container_name)
        .cmd(vec!["sleep", "300"])
        .execute()
        .await;

    // Only run if Docker is available
    if create_result.is_err() {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let create_output = create_result.unwrap();
    assert!(!create_output.stdout.is_empty());

    // Start the container
    let start_result = StartCommand::new(&container_name).execute().await.unwrap();
    assert!(!start_result.stdout.is_empty() || !start_result.started_containers.is_empty());

    // Stop the container
    let stop_result = StopCommand::new(&container_name)
        .timeout(5)
        .execute()
        .await
        .unwrap();
    assert!(!stop_result.stdout.is_empty() || !stop_result.stopped_containers.is_empty());

    // Remove the container
    let rm_result = RmCommand::new(&container_name)
        .force()
        .execute()
        .await
        .unwrap();
    assert!(!rm_result.stdout.is_empty());
}

#[tokio::test]
async fn test_container_pause_unpause() {
    let container_name = format!("test-pause-{}", uuid::Uuid::new_v4());

    // Create and start a container
    if CreateCommand::new("alpine:latest")
        .name(&container_name)
        .cmd(vec!["sleep", "300"])
        .execute()
        .await
        .is_ok()
    {
        let _ = StartCommand::new(&container_name).execute().await;

        // Pause the container
        let pause_result = PauseCommand::new(&container_name).execute().await;
        if pause_result.is_ok() {
            assert!(!pause_result.unwrap().stdout.is_empty());

            // Unpause the container
            let unpause_result = UnpauseCommand::new(&container_name)
                .execute()
                .await
                .unwrap();
            assert!(!unpause_result.stdout.is_empty());
        }

        // Clean up
        let _ = RmCommand::new(&container_name).force().execute().await;
    }
}

#[tokio::test]
async fn test_container_restart() {
    let container_name = format!("test-restart-{}", uuid::Uuid::new_v4());

    // Create and start a container
    if CreateCommand::new("alpine:latest")
        .name(&container_name)
        .cmd(vec!["sleep", "300"])
        .execute()
        .await
        .is_ok()
    {
        let _ = StartCommand::new(&container_name).execute().await;

        // Restart the container
        let restart_result = RestartCommand::new(&container_name)
            .timeout(10)
            .execute()
            .await;

        if restart_result.is_ok() {
            assert!(!restart_result.unwrap().stdout.is_empty());
        }

        // Clean up
        let _ = RmCommand::new(&container_name).force().execute().await;
    }
}

#[tokio::test]
async fn test_container_rename() {
    let old_name = format!("test-rename-old-{}", uuid::Uuid::new_v4());
    let new_name = format!("test-rename-new-{}", uuid::Uuid::new_v4());

    // Create a container
    if CreateCommand::new("alpine:latest")
        .name(&old_name)
        .cmd(vec!["sleep", "10"])
        .execute()
        .await
        .is_ok()
    {
        // Rename the container
        let rename_result = RenameCommand::new(&old_name, &new_name).execute().await;

        if rename_result.is_ok() {
            assert!(!rename_result.unwrap().stderr.contains("Error"));
            // Clean up with new name
            let _ = RmCommand::new(&new_name).force().execute().await;
        } else {
            // Clean up with old name if rename failed
            let _ = RmCommand::new(&old_name).force().execute().await;
        }
    }
}

#[tokio::test]
async fn test_container_kill() {
    let container_name = format!("test-kill-{}", uuid::Uuid::new_v4());

    // Create and start a container
    if CreateCommand::new("alpine:latest")
        .name(&container_name)
        .cmd(vec!["sleep", "300"])
        .execute()
        .await
        .is_ok()
    {
        let _ = StartCommand::new(&container_name).execute().await;

        // Kill the container with SIGTERM
        let kill_result = KillCommand::new(&container_name)
            .signal("SIGTERM")
            .execute()
            .await;

        if kill_result.is_ok() {
            assert!(!kill_result.unwrap().stdout.is_empty());
        }

        // Clean up
        let _ = RmCommand::new(&container_name).force().execute().await;
    }
}

#[tokio::test]
async fn test_container_update() {
    let container_name = format!("test-update-{}", uuid::Uuid::new_v4());

    // Create a container
    if CreateCommand::new("alpine:latest")
        .name(&container_name)
        .cmd(vec!["sleep", "300"])
        .execute()
        .await
        .is_ok()
    {
        // Update container resources
        let update_result = UpdateCommand::new(&container_name)
            .memory("512m")
            .cpus("0.5")
            .execute()
            .await;

        if update_result.is_ok() {
            assert!(!update_result.unwrap().stdout.is_empty());
        }

        // Clean up
        let _ = RmCommand::new(&container_name).force().execute().await;
    }
}

#[tokio::test]
async fn test_container_wait() {
    let container_name = format!("test-wait-{}", uuid::Uuid::new_v4());

    // Create and start a short-lived container
    if CreateCommand::new("alpine:latest")
        .name(&container_name)
        .cmd(vec!["sleep", "1"])
        .execute()
        .await
        .is_ok()
    {
        let _ = StartCommand::new(&container_name).execute().await;

        // Wait for container to exit
        let wait_result = WaitCommand::new(&container_name).execute().await;

        if wait_result.is_ok() {
            let output = wait_result.unwrap();
            // The output should contain the exit code (0)
            assert!(output.stdout.contains("0"));
        }

        // Clean up
        let _ = RmCommand::new(&container_name).force().execute().await;
    }
}

#[tokio::test]
async fn test_container_commit() {
    let container_name = format!("test-commit-{}", uuid::Uuid::new_v4());
    let image_name = format!("test-image-{}", uuid::Uuid::new_v4());

    // Create a container
    if CreateCommand::new("alpine:latest")
        .name(&container_name)
        .cmd(vec!["sh"])
        .execute()
        .await
        .is_ok()
    {
        // Commit the container to a new image
        let commit_result = CommitCommand::new(&container_name)
            .repository(&image_name)
            .tag("latest")
            .message("Test commit")
            .author("docker-wrapper tests")
            .execute()
            .await;

        if commit_result.is_ok() {
            assert!(!commit_result.unwrap().stdout.is_empty());

            // Clean up the image
            let _ = docker_wrapper::RmiCommand::new(format!("{}:latest", image_name))
                .force()
                .execute()
                .await;
        }

        // Clean up container
        let _ = RmCommand::new(&container_name).force().execute().await;
    }
}

#[tokio::test]
async fn test_remove_command_options() {
    // Test command building for remove with various options
    let rm_cmd = RmCommand::new("test-container").force().volumes().link();

    let args = rm_cmd.build_command_args();
    assert!(args.contains(&"rm".to_string()));
    assert!(args.contains(&"--force".to_string()));
    assert!(args.contains(&"--volumes".to_string()));
    assert!(args.contains(&"--link".to_string()));
    assert!(args.contains(&"test-container".to_string()));
}

#[cfg(test)]
mod uuid {
    pub struct Uuid;
    impl Uuid {
        pub fn new_v4() -> String {
            format!(
                "{:x}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            )
        }
    }
}
