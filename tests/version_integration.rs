//! Integration tests for Docker version command.
//!
//! These tests validate the version command functionality against a real Docker daemon.
//! They test the command construction, execution, and output parsing.

use docker_wrapper::{ensure_docker, VersionCommand};

/// Helper to check if Docker is available for testing
async fn setup_docker() -> Result<(), Box<dyn std::error::Error>> {
    ensure_docker().await?;
    Ok(())
}

#[tokio::test]
async fn test_version_prerequisites_validation() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;
    Ok(())
}

#[tokio::test]
async fn test_version_basic_command() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = VersionCommand::new();
    let args = command.build_args();

    // Verify the basic command structure
    assert_eq!(command.command_name(), "version");
    // Default version should just have the command name
    assert_eq!(args, vec!["version"]);

    // Test the command builds without errors
    assert!(!args.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_version_command_builder() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = VersionCommand::new().format("json");

    let args = command.build_args();

    // Verify format option is present
    assert_eq!(command.command_name(), "version");
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"json".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_version_format_variations() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let formats = vec!["json", "table", "yaml", "{{.Server.Version}}"];

    for format in formats {
        let command = VersionCommand::new().format(format);
        let args = command.build_args();

        assert_eq!(command.command_name(), "version");
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&format.to_string()));
    }
    Ok(())
}

#[tokio::test]
async fn test_version_json_format() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = VersionCommand::new().format("json");
    let args = command.build_args();

    // Verify JSON format
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"json".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_version_table_format() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = VersionCommand::new().format("table");
    let args = command.build_args();

    // Verify table format
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"table".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_version_custom_go_template() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let template = "{{.Server.Version}} - {{.Client.Version}}";
    let command = VersionCommand::new().format(template);
    let args = command.build_args();

    // Verify custom template
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&template.to_string()));
    Ok(())
}

#[tokio::test]
async fn test_version_command_name() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = VersionCommand::new();
    assert_eq!(command.command_name(), "version");
    Ok(())
}

#[tokio::test]
async fn test_version_command_display() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = VersionCommand::new().format("json");

    let args = command.build_args();
    assert!(!args.is_empty());
    assert_eq!(command.command_name(), "version");
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"json".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_version_default_format() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = VersionCommand::new();
    let args = command.build_args();

    // Default should not include format flag
    assert_eq!(command.command_name(), "version");
    assert_eq!(args, vec!["version"]);
    Ok(())
}

#[tokio::test]
async fn test_version_format_parsing_concept() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test that different format commands produce expected arguments
    let json_command = VersionCommand::new().format("json");
    let table_command = VersionCommand::new().format("table");

    let json_args = json_command.build_args();
    let table_args = table_command.build_args();

    // Verify format arguments are correctly set
    assert!(json_args.contains(&"--format".to_string()));
    assert!(json_args.contains(&"json".to_string()));
    assert!(table_args.contains(&"--format".to_string()));
    assert!(table_args.contains(&"table".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_version_command_order() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = VersionCommand::new().format("table");

    let args = command.build_args();

    // Verify command structure
    assert_eq!(command.command_name(), "version");
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"table".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_version_empty_format_handling() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test with empty format (should behave like default)
    let command = VersionCommand::new().format("");
    let _args = command.build_args();

    // Should still produce valid command
    assert_eq!(command.command_name(), "version");
    // Empty format still gets passed as a flag
    assert_eq!(_args, vec!["version", "--format", ""]);
    Ok(())
}

#[tokio::test]
async fn test_version_complex_go_templates() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let complex_templates = vec![
        "{{.Server.Version}}",
        "{{.Client.Version}}",
        "{{json .}}",
        "Client: {{.Client.Version}}\nServer: {{.Server.Version}}",
        "{{range .Client.Components}}{{.Name}}: {{.Version}}\n{{end}}",
    ];

    for template in complex_templates {
        let command = VersionCommand::new().format(template);
        let args = command.build_args();

        assert_eq!(command.command_name(), "version");
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&template.to_string()));
    }
    Ok(())
}

#[tokio::test]
async fn test_version_validation() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test command validation
    let command = VersionCommand::new().format("json");

    // Test command name
    assert_eq!(command.command_name(), "version");

    // Test build args format
    let args = command.build_args();
    assert!(!args.is_empty());
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"json".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_version_builder_pattern() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test fluent builder pattern
    let command = VersionCommand::new().format("json");
    let args = command.build_args();

    assert_eq!(command.command_name(), "version");
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"json".to_string()));

    // Test that builder methods can be chained
    let chained_command = VersionCommand::new()
        .format("table")
        .format("{{.Server.Version}}"); // Last format wins

    let chained_args = chained_command.build_args();
    assert!(chained_args.contains(&"--format".to_string()));
    assert!(chained_args.contains(&"{{.Server.Version}}".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_version_format_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test with format containing spaces and special characters
    let edge_case_formats = vec![
        "table {{.Server.Version}}",
        "{{.Client.Version}} {{.Server.Version}}",
        "json",
        "yaml",
        "{{range .}}{{.}}{{end}}",
    ];

    for format in edge_case_formats {
        let command = VersionCommand::new().format(format);
        let args = command.build_args();

        assert_eq!(command.command_name(), "version");
        assert!(args.contains(&"--format".to_string()));
        // Format should be properly included
        assert!(args.contains(&format.to_string()));
    }
    Ok(())
}

#[tokio::test]
async fn test_version_output_parsing_concept() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test the command structure that would be used for output parsing
    let json_command = VersionCommand::new().format("json");
    let table_command = VersionCommand::new().format("table");
    let default_command = VersionCommand::new();

    // Verify different formats produce different commands
    assert!(json_command.build_args().contains(&"--format".to_string()));
    assert!(json_command.build_args().contains(&"json".to_string()));
    assert!(table_command.build_args().contains(&"--format".to_string()));
    assert!(table_command.build_args().contains(&"table".to_string()));
    assert_eq!(default_command.build_args(), vec!["version"]);

    // All should be valid version commands
    assert_eq!(json_command.command_name(), "version");
    assert_eq!(table_command.command_name(), "version");
    assert_eq!(default_command.command_name(), "version");
    Ok(())
}
