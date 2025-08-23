//! Integration tests for the Docker login command.
//!
//! These tests require Docker to be installed and running.
//! Note: These tests do NOT perform actual authentication to avoid requiring credentials.

use docker_wrapper::{ensure_docker, LoginCommand};

/// Helper to check if Docker is available for testing
async fn setup_docker() -> Result<(), Box<dyn std::error::Error>> {
    ensure_docker().await?;
    Ok(())
}

#[tokio::test]
async fn test_login_command_validation() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test that login command builds correctly
    let login = LoginCommand::new("testuser", "testpass");

    assert_eq!(login.get_username(), "testuser");
    assert!(!login.is_password_stdin());
    assert_eq!(login.get_registry(), None);

    // Verify args are built correctly
    let args = login.build_args();
    assert!(args.contains(&"login".to_string()));
    assert!(args.contains(&"--username".to_string()));
    assert!(args.contains(&"testuser".to_string()));
    assert!(args.contains(&"--password".to_string()));
    assert!(args.contains(&"testpass".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_login_command_with_registry() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let login = LoginCommand::new("user", "pass").registry("my-registry.com");

    assert_eq!(login.get_registry(), Some("my-registry.com"));

    let args = login.build_args();
    assert!(args.contains(&"my-registry.com".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_login_command_password_stdin() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let login = LoginCommand::new("user", "").password_stdin();

    assert!(login.is_password_stdin());

    let args = login.build_args();
    assert!(args.contains(&"--password-stdin".to_string()));
    assert!(!args.contains(&"--password".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_login_command_docker_hub_default() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let login = LoginCommand::new("dockeruser", "dockerpass");

    // No registry specified should default to Docker Hub
    assert_eq!(login.get_registry(), None);

    let args = login.build_args();
    // Should not contain any registry URL when using Docker Hub
    assert!(!args.iter().any(|arg| arg.contains("docker.io")));

    Ok(())
}

#[tokio::test]
async fn test_login_command_multiple_registries() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test different registry configurations
    let registries = vec![
        "gcr.io",
        "my-registry.example.com:5000",
        "localhost:5000",
        "registry.gitlab.com",
    ];

    for registry in registries {
        let login = LoginCommand::new("user", "pass").registry(registry);
        assert_eq!(login.get_registry(), Some(registry));

        let args = login.build_args();
        assert!(args.contains(&registry.to_string()));
    }

    Ok(())
}

#[tokio::test]
async fn test_login_command_builder_pattern() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test fluent builder pattern
    let login = LoginCommand::new("admin", "secret").registry("private-registry.company.com");

    assert_eq!(login.get_username(), "admin");
    assert_eq!(login.get_registry(), Some("private-registry.company.com"));
    assert!(!login.is_password_stdin());

    Ok(())
}

#[tokio::test]
async fn test_login_command_display() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let login = LoginCommand::new("testuser", "testpass").registry("example.com");

    let display = format!("{login}");
    assert!(display.contains("docker login"));
    assert!(display.contains("example.com"));
    assert!(display.contains("--username testuser"));
    assert!(display.contains("--password [HIDDEN]"));
    // Password should be hidden in display
    assert!(!display.contains("testpass"));

    Ok(())
}

#[tokio::test]
async fn test_login_command_display_stdin() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let login = LoginCommand::new("testuser", "").password_stdin();

    let display = format!("{login}");
    assert!(display.contains("--password-stdin"));
    assert!(!display.contains("[HIDDEN]"));

    Ok(())
}

#[tokio::test]
async fn test_login_command_extensibility() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let mut login = LoginCommand::new("user", "pass");

    // Test the extension methods for future compatibility
    login
        .arg("--verbose")
        .args(vec!["--debug"])
        .flag("--insecure")
        .option("--timeout", "30");

    // Command should still function normally
    assert_eq!(login.command_name(), "login");
    assert_eq!(login.get_username(), "user");

    Ok(())
}

#[tokio::test]
async fn test_login_prerequisites_validation() -> Result<(), Box<dyn std::error::Error>> {
    // This test ensures Docker is available before running login tests
    setup_docker().await?;

    // If we get here, Docker is available and we can proceed with other tests
    let login = LoginCommand::new("test", "test");
    assert_eq!(login.command_name(), "login");

    Ok(())
}

#[tokio::test]
async fn test_login_command_security_considerations() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test that sensitive information is handled properly
    let login_with_password = LoginCommand::new("user", "supersecret");
    let display = format!("{login_with_password}");

    // Ensure password is not exposed in string representation
    assert!(!display.contains("supersecret"));
    assert!(display.contains("[HIDDEN]"));

    // Test stdin mode for more secure password handling
    let login_stdin = LoginCommand::new("user", "").password_stdin();
    assert!(login_stdin.is_password_stdin());

    let args = login_stdin.build_args();
    assert!(args.contains(&"--password-stdin".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_login_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test with empty username (should still build but may fail at runtime)
    let login_empty_user = LoginCommand::new("", "pass");
    assert_eq!(login_empty_user.get_username(), "");

    // Test with very long registry name
    let long_registry = "very-long-registry-name.example.com";
    let login_long = LoginCommand::new("user", "pass").registry(long_registry);
    assert_eq!(login_long.get_registry(), Some(long_registry));

    // Test default construction
    let login_default = LoginCommand::default();
    assert_eq!(login_default.get_username(), "");
    assert!(!login_default.is_password_stdin());

    Ok(())
}

#[tokio::test]
async fn test_login_command_name() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let login = LoginCommand::new("user", "pass");
    assert_eq!(login.command_name(), "login");

    Ok(())
}

#[tokio::test]
async fn test_login_command_argument_order() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test that arguments are in the correct order for Docker CLI
    let login = LoginCommand::new("testuser", "testpass").registry("example.com");
    let args = login.build_args();

    // Find positions of key arguments
    let login_pos = args.iter().position(|s| s == "login").unwrap();
    let username_pos = args.iter().position(|s| s == "--username").unwrap();
    let password_pos = args.iter().position(|s| s == "--password").unwrap();
    let registry_pos = args.iter().position(|s| s == "example.com").unwrap();

    // Verify order: login < username < password < registry
    assert!(login_pos < username_pos);
    assert!(username_pos < password_pos);
    assert!(password_pos < registry_pos);

    Ok(())
}

#[tokio::test]
async fn test_login_various_registry_formats() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test various registry URL formats
    let test_cases = vec![
        ("gcr.io", "Google Container Registry"),
        ("registry-1.docker.io", "Docker Hub explicit"),
        ("localhost:5000", "Local registry"),
        ("my-registry.com:443", "Custom port"),
        ("registry.example.com/path", "Registry with path"),
    ];

    for (registry, _description) in test_cases {
        let login = LoginCommand::new("user", "pass").registry(registry);
        let args = login.build_args();
        assert!(args.contains(&registry.to_string()));
    }

    Ok(())
}
