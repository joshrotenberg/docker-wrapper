//! Integration tests for Docker network commands

use docker_wrapper::command::network::{
    NetworkConnectCommand, NetworkCreateCommand, NetworkDisconnectCommand, NetworkInspectCommand,
    NetworkLsCommand, NetworkPruneCommand, NetworkRmCommand,
};
use docker_wrapper::DockerCommand;

#[tokio::test]
async fn test_network_lifecycle() {
    // Test creating, listing, inspecting, and removing a network
    let network_name = format!("test-network-{}", uuid::Uuid::new_v4());

    // Create network
    let create_result = NetworkCreateCommand::new(&network_name)
        .driver("bridge")
        .label("test", "integration")
        .execute()
        .await;

    // Only run if Docker is available
    if create_result.is_err() {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    let create_output = create_result.unwrap();
    assert!(!create_output.stdout.is_empty());

    // List networks and verify our network exists
    let ls_result = NetworkLsCommand::new()
        .filter("name", &network_name)
        .execute()
        .await
        .unwrap();

    assert!(ls_result.stdout.contains(&network_name));

    // Inspect the network
    let inspect_result = NetworkInspectCommand::new(&network_name)
        .execute()
        .await
        .unwrap();

    assert!(inspect_result.stdout.contains(&network_name));

    // Remove the network
    let rm_result = NetworkRmCommand::new(&network_name)
        .execute()
        .await
        .unwrap();

    assert_eq!(rm_result.exit_code, 0);
}

#[tokio::test]
async fn test_network_create_with_options() {
    let network_name = format!("test-network-opts-{}", uuid::Uuid::new_v4());

    let create_cmd = NetworkCreateCommand::new(&network_name)
        .driver("bridge")
        .subnet("172.28.0.0/16")
        .ip_range("172.28.5.0/24")
        .gateway("172.28.5.1")
        .internal()
        .ipv6()
        .label("environment", "test")
        .label("version", "1.0");

    // Test command building without execution
    let args = create_cmd.build_command_args();
    assert!(args.contains(&"--driver".to_string()));
    assert!(args.contains(&"bridge".to_string()));
    assert!(args.contains(&"--subnet".to_string()));
    assert!(args.contains(&"172.28.0.0/16".to_string()));
    assert!(args.contains(&"--internal".to_string()));
    assert!(args.contains(&"--ipv6".to_string()));

    // Clean up if actually created
    if let Ok(_) = create_cmd.execute().await {
        let _ = NetworkRmCommand::new(&network_name).execute().await;
    }
}

#[tokio::test]
async fn test_network_connect_disconnect() {
    // This test requires a running container, so we'll just test command building
    let connect_cmd = NetworkConnectCommand::new("my-network", "my-container")
        .alias("my-alias")
        .ipv4("172.28.5.10");

    let args = connect_cmd.build_command_args();
    assert!(args.contains(&"network".to_string()));
    assert!(args.contains(&"connect".to_string()));
    assert!(args.contains(&"--alias".to_string()));
    assert!(args.contains(&"my-alias".to_string()));
    assert!(args.contains(&"--ip".to_string()));
    assert!(args.contains(&"172.28.5.10".to_string()));

    let disconnect_cmd = NetworkDisconnectCommand::new("my-network", "my-container").force();

    let args = disconnect_cmd.build_command_args();
    assert!(args.contains(&"network".to_string()));
    assert!(args.contains(&"disconnect".to_string()));
    assert!(args.contains(&"--force".to_string()));
}

#[tokio::test]
async fn test_network_ls_with_filters() {
    let ls_cmd = NetworkLsCommand::new()
        .filter("driver", "bridge")
        .filter("label", "test")
        .quiet()
        .no_trunc();

    let args = ls_cmd.build_command_args();
    assert!(args.contains(&"--filter".to_string()));
    assert!(args.contains(&"driver=bridge".to_string()));
    assert!(args.contains(&"--quiet".to_string()));
    assert!(args.contains(&"--no-trunc".to_string()));
}

#[tokio::test]
async fn test_network_inspect_multiple() {
    let inspect_cmd = NetworkInspectCommand::new("network1")
        .add_network("network2")
        .format("{{.Name}}: {{.Driver}}");

    let args = inspect_cmd.build_command_args();
    assert!(args.contains(&"network".to_string()));
    assert!(args.contains(&"inspect".to_string()));
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"network1".to_string()));
    assert!(args.contains(&"network2".to_string()));
}

#[tokio::test]
async fn test_network_prune() {
    let prune_cmd = NetworkPruneCommand::new().force().filter("until", "24h");

    let args = prune_cmd.build_command_args();
    assert!(args.contains(&"network".to_string()));
    assert!(args.contains(&"prune".to_string()));
    assert!(args.contains(&"--force".to_string()));
    assert!(args.contains(&"--filter".to_string()));
    assert!(args.contains(&"until=24h".to_string()));
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
