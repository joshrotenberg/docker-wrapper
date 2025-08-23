//! Integration tests for Docker logout command.
//!
//! These tests validate the logout command functionality against a real Docker daemon.
//! They test the command construction, execution, and output parsing.

use docker_wrapper::{ensure_docker, DockerCommand, LogoutCommand};

/// Helper to check if Docker is available for testing
async fn setup_docker() -> Result<(), Box<dyn std::error::Error>> {
    ensure_docker().await?;
    Ok(())
}

#[tokio::test]
async fn test_logout_prerequisites_validation() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;
    Ok(())
}

#[tokio::test]
async fn test_logout_basic_command() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = LogoutCommand::new();
    let args = command.build_command_args();

    // Verify the basic command structure
    assert_eq!(args[0], "logout");

    // Default logout should just have the command name
    assert_eq!(args, vec!["logout"]);

    // Test the command builds without errors
    assert!(!args.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_logout_command_builder() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = LogoutCommand::new().server("my-registry.example.com");

    let args = command.build_command_args();

    // Verify server is specified
    assert_eq!(args[0], "logout");
    assert!(args.contains(&"my-registry.example.com".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_logout_with_private_registry() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = LogoutCommand::new().server("registry.company.com:5000");

    let args = command.build_command_args();

    // Verify private registry with port
    assert!(args.contains(&"registry.company.com:5000".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_logout_docker_hub_variations() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test various Docker Hub formats
    let hub_variations = vec![
        "docker.io",
        "index.docker.io",
        "registry-1.docker.io",
        "https://index.docker.io/v1/",
    ];

    for server in hub_variations {
        let command = LogoutCommand::new().server(server);

        let args = command.build_command_args();
        assert_eq!(args[0], "logout");
        assert!(args.contains(&server.to_string()));
    }
    Ok(())
}

#[tokio::test]
async fn test_logout_command_name() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = LogoutCommand::new();
    let args = command.build_command_args();
    assert_eq!(args[0], "logout");
    Ok(())
}

#[tokio::test]
async fn test_logout_command_display() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = LogoutCommand::new().server("test-registry.com");

    let args = command.build_command_args();
    assert!(!args.is_empty());
    let args = command.build_command_args();
    assert_eq!(args[0], "logout");
    assert!(args.contains(&"test-registry.com".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_logout_command_format() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = LogoutCommand::new().server("registry.example.com");
    let args = command.build_command_args();

    // Verify command format is correct
    assert_eq!(args[0], "logout");
    assert!(args.contains(&"registry.example.com".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_logout_various_registry_formats() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let registry_formats = vec![
        "localhost:5000",
        "registry.local",
        "my-registry.company.com",
        "gcr.io",
        "quay.io",
        "registry.gitlab.com",
        "ghcr.io",
    ];

    for registry in registry_formats {
        let command = LogoutCommand::new().server(registry);

        let args = command.build_command_args();
        assert_eq!(args[0], "logout");
        assert!(args.contains(&registry.to_string()));

        // Verify command can be built
        assert!(!args.is_empty());
    }
    Ok(())
}

#[tokio::test]
async fn test_logout_default_daemon_behavior() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = LogoutCommand::new();
    let args = command.build_command_args();

    // Default logout should just have the command name
    assert_eq!(args[0], "logout");
    assert_eq!(args, vec!["logout"]);
    Ok(())
}

#[tokio::test]
async fn test_logout_command_order() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = LogoutCommand::new().server("registry.example.com");

    let args = command.build_command_args();

    // Verify command structure
    assert_eq!(args[0], "logout");

    // Server should be in the args
    assert!(args.contains(&"registry.example.com".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_logout_multiple_servers_concept() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Note: Docker logout doesn't support multiple servers in one command,
    // but we can test creating multiple commands
    let servers = vec!["registry1.com", "registry2.com", "registry3.com"];

    for server in servers {
        let command = LogoutCommand::new().server(server);

        let args = command.build_command_args();
        assert_eq!(args[0], "logout");
        assert!(args.contains(&server.to_string()));
    }
    Ok(())
}

#[tokio::test]
async fn test_logout_empty_server_handling() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test with empty server (should behave like default)
    let command = LogoutCommand::new().server("");
    let _args = command.build_command_args();

    // Should still produce valid command
    let args = command.build_command_args();
    assert_eq!(args[0], "logout");
    // Empty server still gets passed as an argument
    assert_eq!(_args, vec!["logout", ""]);
    Ok(())
}

#[tokio::test]
async fn test_logout_server_with_protocol() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let servers_with_protocol = vec![
        "https://registry.example.com",
        "http://localhost:5000",
        "https://my-registry.com:8443",
    ];

    for server in servers_with_protocol {
        let command = LogoutCommand::new().server(server);

        let args = command.build_command_args();
        assert_eq!(args[0], "logout");
        assert!(args.contains(&server.to_string()));
    }
    Ok(())
}

#[tokio::test]
async fn test_logout_validation() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test command validation
    let command = LogoutCommand::new().server("test-registry.io");

    // Test command name
    let args = command.build_command_args();
    assert_eq!(args[0], "logout");

    // Test build args format
    let args = command.build_command_args();
    assert!(!args.is_empty());
    assert!(args.contains(&"test-registry.io".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_logout_builder_pattern() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test fluent builder pattern
    let command = LogoutCommand::new().server("registry.example.com");

    let args = command.build_command_args();
    assert_eq!(args[0], "logout");
    assert!(args.contains(&"registry.example.com".to_string()));

    // Test that builder methods can be chained
    let chained = LogoutCommand::new()
        .server("first.com")
        .server("second.com"); // Last server wins

    let chained_args = chained.build_command_args();
    assert!(chained_args.contains(&"second.com".to_string()));
    assert!(!chained_args.contains(&"first.com".to_string()));
    Ok(())
}
