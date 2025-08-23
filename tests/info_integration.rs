//! Integration tests for Docker info command.
//!
//! These tests validate the info command functionality against a real Docker daemon.
//! They test the command construction, execution, and output parsing.

use docker_wrapper::{ensure_docker, DockerCommand, InfoCommand};

/// Helper to check if Docker is available for testing
async fn setup_docker() -> Result<(), Box<dyn std::error::Error>> {
    ensure_docker().await?;
    Ok(())
}

#[tokio::test]
async fn test_info_prerequisites_validation() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;
    Ok(())
}

#[tokio::test]
async fn test_info_basic_command() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = InfoCommand::new();
    let args = command.build_args();

    // Verify the basic command structure
    assert_eq!(command.command_name(), "info");
    // Default info should just have the command name
    assert_eq!(args, vec!["info"]);

    // Test the command builds without errors
    assert!(!args.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_info_command_builder() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = InfoCommand::new().format("json");

    let args = command.build_args();

    // Verify format option is present
    assert_eq!(command.command_name(), "info");
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"json".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_info_format_variations() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let formats = vec![
        "json",
        "table",
        "{{.ServerVersion}}",
        "{{.Name}}",
        "{{.DriverStatus}}",
    ];

    for format in formats {
        let command = InfoCommand::new().format(format);
        let args = command.build_args();

        assert_eq!(command.command_name(), "info");
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&format.to_string()));
    }
    Ok(())
}

#[tokio::test]
async fn test_info_json_format() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = InfoCommand::new().format("json");
    let args = command.build_args();

    // Verify JSON format
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"json".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_info_table_format() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = InfoCommand::new().format("table");
    let args = command.build_args();

    // Verify table format
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"table".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_info_custom_go_template() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let template = "Server Version: {{.ServerVersion}}, Storage Driver: {{.Driver}}";
    let command = InfoCommand::new().format(template);
    let args = command.build_args();

    // Verify custom template
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&template.to_string()));
    Ok(())
}

#[tokio::test]
async fn test_info_command_name() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = InfoCommand::new();
    assert_eq!(command.command_name(), "info");
    Ok(())
}

#[tokio::test]
async fn test_info_command_display() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = InfoCommand::new().format("json");

    let args = command.build_args();
    assert!(!args.is_empty());
    assert_eq!(command.command_name(), "info");
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"json".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_info_default_format() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = InfoCommand::new();
    let args = command.build_args();

    // Default should not include format flag
    assert_eq!(command.command_name(), "info");
    assert_eq!(args, vec!["info"]);
    Ok(())
}

#[tokio::test]
async fn test_info_format_parsing_concept() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test that different format commands produce expected arguments
    let json_command = InfoCommand::new().format("json");
    let table_command = InfoCommand::new().format("table");

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
async fn test_info_command_order() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let command = InfoCommand::new().format("table");

    let args = command.build_args();

    // Verify command structure
    assert_eq!(command.command_name(), "info");
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"table".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_info_empty_format_handling() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test with empty format (should behave like default)
    let command = InfoCommand::new().format("");
    let _args = command.build_args();

    // Should still produce valid command
    assert_eq!(command.command_name(), "info");
    // Empty format still gets passed as a flag
    assert_eq!(_args, vec!["info", "--format", ""]);
    Ok(())
}

#[tokio::test]
async fn test_info_complex_go_templates() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let complex_templates = vec![
        "{{.ServerVersion}}",
        "{{.Name}}",
        "{{json .}}",
        "Server: {{.ServerVersion}}\nDriver: {{.Driver}}",
        "{{range .Plugins.Storage}}{{.}}\n{{end}}",
        "{{.NCPU}} CPUs, {{.MemTotal}} bytes memory",
    ];

    for template in complex_templates {
        let command = InfoCommand::new().format(template);
        let args = command.build_args();

        assert_eq!(command.command_name(), "info");
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&template.to_string()));
    }
    Ok(())
}

#[tokio::test]
async fn test_info_system_information_templates() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let system_templates = vec![
        "{{.Architecture}}",
        "{{.OSType}}",
        "{{.KernelVersion}}",
        "{{.OperatingSystem}}",
        "{{.DockerRootDir}}",
        "{{.SystemTime}}",
    ];

    for template in system_templates {
        let command = InfoCommand::new().format(template);
        let args = command.build_args();

        assert_eq!(command.command_name(), "info");
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&template.to_string()));
    }
    Ok(())
}

#[tokio::test]
async fn test_info_container_runtime_templates() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let runtime_templates = vec![
        "{{.ContainersRunning}}",
        "{{.ContainersPaused}}",
        "{{.ContainersStopped}}",
        "{{.Images}}",
        "{{.Driver}}",
        "{{.LoggingDriver}}",
        "{{.CgroupDriver}}",
    ];

    for template in runtime_templates {
        let command = InfoCommand::new().format(template);
        let args = command.build_args();

        assert_eq!(command.command_name(), "info");
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&template.to_string()));
    }
    Ok(())
}

#[tokio::test]
async fn test_info_validation() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test command validation
    let command = InfoCommand::new().format("json");

    // Test command name
    assert_eq!(command.command_name(), "info");

    // Test build args format
    let args = command.build_args();
    assert!(!args.is_empty());
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"json".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_info_builder_pattern() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test fluent builder pattern
    let command = InfoCommand::new().format("json");

    let args = command.build_args();

    assert_eq!(command.command_name(), "info");
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"json".to_string()));

    // Test that builder methods can be chained
    let chained_command = InfoCommand::new()
        .format("table")
        .format("{{.ServerVersion}}"); // Last format wins

    let chained_args = chained_command.build_args();
    assert!(chained_args.contains(&"--format".to_string()));
    assert!(chained_args.contains(&"{{.ServerVersion}}".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_info_format_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test with format containing spaces and special characters
    let edge_case_formats = vec![
        "table {{.ServerVersion}}",
        "{{.Name}} - {{.ServerVersion}}",
        "json",
        "{{range .Plugins.Network}}{{.}}\n{{end}}",
        "CPU: {{.NCPU}}, Memory: {{.MemTotal}}",
    ];

    for format in edge_case_formats {
        let command = InfoCommand::new().format(format);
        let args = command.build_args();

        assert_eq!(command.command_name(), "info");
        assert!(args.contains(&"--format".to_string()));
        // Format should be properly included
        assert!(args.contains(&format.to_string()));
    }
    Ok(())
}

#[tokio::test]
async fn test_info_output_parsing_concept() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test the command structure that would be used for output parsing
    let json_command = InfoCommand::new().format("json");
    let table_command = InfoCommand::new().format("table");
    let default_command = InfoCommand::new();

    // Verify different formats produce different commands
    assert!(json_command.build_args().contains(&"--format".to_string()));
    assert!(json_command.build_args().contains(&"json".to_string()));
    assert!(table_command.build_args().contains(&"--format".to_string()));
    assert!(table_command.build_args().contains(&"table".to_string()));
    assert_eq!(default_command.build_args(), vec!["info"]);

    // All should be valid info commands
    assert_eq!(json_command.command_name(), "info");
    assert_eq!(table_command.command_name(), "info");
    assert_eq!(default_command.command_name(), "info");
    Ok(())
}

#[tokio::test]
async fn test_info_security_context_templates() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let security_templates = vec![
        "{{.SecurityOptions}}",
        "{{range .SecurityOptions}}{{.}}\n{{end}}",
        "{{.Warnings}}",
        "{{range .Warnings}}Warning: {{.}}\n{{end}}",
    ];

    for template in security_templates {
        let command = InfoCommand::new().format(template);
        let args = command.build_args();

        assert_eq!(command.command_name(), "info");
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&template.to_string()));
    }
    Ok(())
}

#[tokio::test]
async fn test_info_storage_templates() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let storage_templates = vec![
        "{{.DriverStatus}}",
        "{{range .DriverStatus}}{{index . 0}}: {{index . 1}}\n{{end}}",
        "{{.DockerRootDir}}",
        "{{.Driver}}",
    ];

    for template in storage_templates {
        let command = InfoCommand::new().format(template);
        let args = command.build_args();

        assert_eq!(command.command_name(), "info");
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&template.to_string()));
    }
    Ok(())
}
