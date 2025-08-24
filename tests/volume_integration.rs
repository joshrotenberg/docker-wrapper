//! Integration tests for Docker volume commands

use docker_wrapper::command::volume::{
    VolumeCreateCommand, VolumeInspectCommand, VolumeLsCommand, VolumePruneCommand, VolumeRmCommand,
};
use docker_wrapper::DockerCommand;

#[tokio::test]
async fn test_volume_lifecycle() {
    // Test creating, listing, inspecting, and removing a volume
    let volume_name = format!("test-volume-{}", uuid::Uuid::new_v4());

    // Create volume
    let create_result = VolumeCreateCommand::new()
        .name(&volume_name)
        .label("test", "integration")
        .execute()
        .await;

    // Only run if Docker is available
    if create_result.is_err() {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let create_output = create_result.unwrap();
    assert!(create_output.stdout.contains(&volume_name));

    // List volumes and verify our volume exists
    let ls_result = VolumeLsCommand::new()
        .filter("name", &volume_name)
        .execute()
        .await
        .unwrap();

    assert!(ls_result.stdout.contains(&volume_name));

    // Inspect the volume
    let inspect_result = VolumeInspectCommand::new(&volume_name)
        .execute()
        .await
        .unwrap();

    assert!(inspect_result.stdout.contains(&volume_name));

    // Remove the volume
    let rm_result = VolumeRmCommand::new(&volume_name).execute().await.unwrap();

    assert_eq!(rm_result.exit_code, 0);
}

#[tokio::test]
async fn test_volume_create_with_options() {
    let volume_name = format!("test-volume-opts-{}", uuid::Uuid::new_v4());

    let create_cmd = VolumeCreateCommand::new()
        .name(&volume_name)
        .driver("local")
        .driver_opt("type", "tmpfs")
        .driver_opt("device", "tmpfs")
        .driver_opt("o", "size=100m")
        .label("environment", "test")
        .label("version", "1.0");

    // Test command building
    let args = create_cmd.build_command_args();
    assert!(args.contains(&"volume".to_string()));
    assert!(args.contains(&"create".to_string()));
    assert!(args.contains(&"--driver".to_string()));
    assert!(args.contains(&"local".to_string()));
    assert!(args.contains(&"--opt".to_string()));
    assert!(args.contains(&volume_name));

    // Clean up if actually created
    if create_cmd.execute().await.is_ok() {
        let _ = VolumeRmCommand::new(&volume_name).execute().await;
    }
}

#[tokio::test]
async fn test_volume_ls_with_filters() {
    let ls_cmd = VolumeLsCommand::new()
        .filter("driver", "local")
        .filter("label", "test")
        .quiet()
        .format("table {{.Name}}\t{{.Driver}}");

    let args = ls_cmd.build_command_args();
    assert!(args.contains(&"volume".to_string()));
    assert!(args.contains(&"ls".to_string()));
    assert!(args.contains(&"--filter".to_string()));
    assert!(args.contains(&"driver=local".to_string()));
    assert!(args.contains(&"--quiet".to_string()));
    assert!(args.contains(&"--format".to_string()));
}

#[tokio::test]
async fn test_volume_inspect_format() {
    let inspect_cmd = VolumeInspectCommand::new("volume1").format("{{.Name}}: {{.Driver}}");

    let args = inspect_cmd.build_command_args();
    assert!(args.contains(&"volume".to_string()));
    assert!(args.contains(&"inspect".to_string()));
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"volume1".to_string()));
}

#[tokio::test]
async fn test_volume_rm_multiple() {
    let rm_cmd = VolumeRmCommand::new("volume1")
        .add_volume("volume2")
        .force();

    let args = rm_cmd.build_command_args();
    assert!(args.contains(&"volume".to_string()));
    assert!(args.contains(&"rm".to_string()));
    assert!(args.contains(&"--force".to_string()));
    assert!(args.contains(&"volume1".to_string()));
    assert!(args.contains(&"volume2".to_string()));
}

#[tokio::test]
async fn test_volume_prune() {
    let prune_cmd = VolumePruneCommand::new()
        .force()
        .all()
        .filter("label", "test");

    let args = prune_cmd.build_command_args();
    assert!(args.contains(&"volume".to_string()));
    assert!(args.contains(&"prune".to_string()));
    assert!(args.contains(&"--force".to_string()));
    assert!(args.contains(&"--all".to_string()));
    assert!(args.contains(&"--filter".to_string()));
    assert!(args.contains(&"label=test".to_string()));
}

#[tokio::test]
async fn test_volume_create_and_use() {
    // Test that we can create a volume and it's usable
    let volume_name = format!("test-volume-use-{}", uuid::Uuid::new_v4());

    // Create volume
    if VolumeCreateCommand::new()
        .name(&volume_name)
        .execute()
        .await
        .is_ok()
    {
        // Verify it exists in list
        let ls_result = VolumeLsCommand::new().quiet().execute().await.unwrap();

        assert!(ls_result.stdout.contains(&volume_name));

        // Clean up
        let _ = VolumeRmCommand::new(&volume_name).execute().await;
    }
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
