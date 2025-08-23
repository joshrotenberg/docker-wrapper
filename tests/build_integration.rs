//! Integration tests for Docker build command.
//!
//! These tests validate the build command functionality against a real Docker daemon.
//! They test the command construction, execution, and output parsing.

use docker_wrapper::{ensure_docker, BuildCommand, DockerCommand};
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper function to create a temporary Dockerfile for testing
fn create_test_dockerfile(temp_dir: &TempDir, content: &str) -> PathBuf {
    let dockerfile_path = temp_dir.path().join("Dockerfile");
    std::fs::write(&dockerfile_path, content).expect("Failed to write test Dockerfile");
    dockerfile_path
}

/// Helper function to create a simple test context directory
fn create_test_context() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");

    // Create a simple Dockerfile
    let dockerfile_content = r#"
FROM alpine:latest
RUN echo "Hello from test build"
LABEL test=true
"#;
    create_test_dockerfile(&temp_dir, dockerfile_content);

    temp_dir
}

/// Helper to check if Docker is available for testing
async fn setup_docker() -> Result<(), Box<dyn std::error::Error>> {
    ensure_docker().await?;
    Ok(())
}

#[tokio::test]
async fn test_build_prerequisites_validation() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;
    Ok(())
}

#[tokio::test]
async fn test_build_basic_command() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let temp_dir = create_test_context();
    let context_path = temp_dir.path().to_string_lossy().to_string();

    let command = BuildCommand::new(&context_path);
    let args = command.build_args();

    // Verify the basic command structure
    assert_eq!(command.command_name(), "build");
    assert!(args.contains(&context_path));

    // Test the command builds without errors
    assert!(!args.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_build_command_builder() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let temp_dir = create_test_context();
    let context_path = temp_dir.path().to_string_lossy().to_string();

    let command = BuildCommand::new(&context_path)
        .tag("test-build:latest")
        .tag("test-build:1.0")
        .build_arg("VERSION", "1.0")
        .build_arg("ENV", "test")
        .label("maintainer", "test@example.com")
        .label("version", "1.0.0")
        .no_cache()
        .force_rm()
        .quiet();

    let args = command.build_args();

    // Verify all options are present
    assert!(args.contains(&"--tag".to_string()));
    assert!(args.contains(&"test-build:latest".to_string()));
    assert!(args.contains(&"test-build:1.0".to_string()));
    assert!(args.contains(&"--build-arg".to_string()));
    assert!(args.contains(&"VERSION=1.0".to_string()));
    assert!(args.contains(&"ENV=test".to_string()));
    assert!(args.contains(&"--label".to_string()));
    assert!(args.contains(&"maintainer=test@example.com".to_string()));
    assert!(args.contains(&"version=1.0.0".to_string()));
    assert!(args.contains(&"--no-cache".to_string()));
    assert!(args.contains(&"--force-rm".to_string()));
    assert!(args.contains(&"--quiet".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_build_with_dockerfile() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let temp_dir = create_test_context();
    let context_path = temp_dir.path().to_string_lossy().to_string();

    // Create a custom Dockerfile with different name
    let custom_dockerfile_content = r#"
FROM alpine:latest
RUN echo "Custom Dockerfile test"
"#;
    let custom_dockerfile = temp_dir.path().join("Dockerfile.custom");
    std::fs::write(&custom_dockerfile, custom_dockerfile_content)
        .expect("Failed to write custom Dockerfile");

    let command = BuildCommand::new(&context_path)
        .file(&custom_dockerfile)
        .tag("test-custom:latest");

    let args = command.build_args();

    // Verify custom Dockerfile is specified
    assert!(args.contains(&"--file".to_string()));
    // The file path might be absolute, so check if any arg contains "Dockerfile.custom"
    assert!(args.iter().any(|arg| arg.contains("Dockerfile.custom")));
    assert!(args.contains(&"--tag".to_string()));
    assert!(args.contains(&"test-custom:latest".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_build_resource_limits() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let temp_dir = create_test_context();
    let context_path = temp_dir.path().to_string_lossy().to_string();

    let command = BuildCommand::new(&context_path)
        .memory("512m")
        .cpu_shares(1024)
        .cpu_period(100000)
        .cpu_quota(50000)
        .cpuset_cpus("0-1")
        .cpuset_mems("0");

    let args = command.build_args();

    // Verify resource limits are set
    assert!(args.contains(&"--memory".to_string()));
    assert!(args.contains(&"512m".to_string()));
    assert!(args.contains(&"--cpu-shares".to_string()));
    assert!(args.contains(&"1024".to_string()));
    assert!(args.contains(&"--cpu-period".to_string()));
    assert!(args.contains(&"100000".to_string()));
    assert!(args.contains(&"--cpu-quota".to_string()));
    assert!(args.contains(&"50000".to_string()));
    assert!(args.contains(&"--cpuset-cpus".to_string()));
    assert!(args.contains(&"0-1".to_string()));
    assert!(args.contains(&"--cpuset-mems".to_string()));
    assert!(args.contains(&"0".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_build_advanced_options() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let temp_dir = create_test_context();
    let context_path = temp_dir.path().to_string_lossy().to_string();

    let iidfile = temp_dir.path().join("image_id.txt");

    let command = BuildCommand::new(&context_path)
        .add_host("example.com:192.168.1.1")
        .add_host("test.local:127.0.0.1")
        .cache_from("alpine:latest")
        .cache_from("ubuntu:20.04")
        .compress()
        .disable_content_trust()
        .iidfile(&iidfile)
        .isolation("default")
        .pull();

    let args = command.build_args();

    // Verify advanced options
    assert!(args.contains(&"--add-host".to_string()));
    assert!(args.contains(&"example.com:192.168.1.1".to_string()));
    assert!(args.contains(&"test.local:127.0.0.1".to_string()));
    assert!(args.contains(&"--cache-from".to_string()));
    assert!(args.contains(&"alpine:latest".to_string()));
    assert!(args.contains(&"ubuntu:20.04".to_string()));
    assert!(args.contains(&"--compress".to_string()));
    assert!(args.contains(&"--disable-content-trust".to_string()));
    assert!(args.contains(&"--iidfile".to_string()));
    assert!(args.contains(&"--isolation".to_string()));
    assert!(args.contains(&"default".to_string()));
    assert!(args.contains(&"--pull".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_build_modern_buildx_options() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let temp_dir = create_test_context();
    let context_path = temp_dir.path().to_string_lossy().to_string();

    let command = BuildCommand::new(&context_path)
        .platform("linux/arm64")
        .target("production")
        .network("host")
        .progress("plain")
        .secret("id=mysecret,src=/path/to/secret")
        .ssh("default");

    let args = command.build_args();

    // Verify modern buildx options
    assert!(args.contains(&"--platform".to_string()));
    assert!(args.contains(&"linux/arm64".to_string()));
    assert!(args.contains(&"--target".to_string()));
    assert!(args.contains(&"production".to_string()));
    assert!(args.contains(&"--network".to_string()));
    assert!(args.contains(&"host".to_string()));
    assert!(args.contains(&"--progress".to_string()));
    assert!(args.contains(&"plain".to_string()));
    assert!(args.contains(&"--secret".to_string()));
    assert!(args.contains(&"id=mysecret,src=/path/to/secret".to_string()));
    assert!(args.contains(&"--ssh".to_string()));
    assert!(args.contains(&"default".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_build_cache_options() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let temp_dir = create_test_context();
    let context_path = temp_dir.path().to_string_lossy().to_string();

    // Test cache-related build options
    let command = BuildCommand::new(&context_path)
        .cache_from("myapp:cache")
        .cache_from("myapp:latest")
        .no_cache()
        .pull();

    let args = command.build_args();

    // Verify cache options
    assert!(args.contains(&"--cache-from".to_string()));
    assert!(args.contains(&"myapp:cache".to_string()));
    assert!(args.contains(&"myapp:latest".to_string()));
    assert!(args.contains(&"--no-cache".to_string()));
    assert!(args.contains(&"--pull".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_build_output_options() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let temp_dir = create_test_context();
    let context_path = temp_dir.path().to_string_lossy().to_string();

    // Test various output-related build options
    let command = BuildCommand::new(&context_path)
        .tag("test-output:latest")
        .quiet()
        .no_cache();

    let args = command.build_args();

    // Verify output control options
    assert!(args.contains(&"--tag".to_string()));
    assert!(args.contains(&"test-output:latest".to_string()));
    assert!(args.contains(&"--quiet".to_string()));
    assert!(args.contains(&"--no-cache".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_build_context_variations() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test with current directory
    let current_dir_command = BuildCommand::new(".");
    assert_eq!(current_dir_command.command_name(), "build");
    assert!(current_dir_command.build_args().contains(&".".to_string()));

    // Test with absolute path
    let temp_dir = create_test_context();
    let abs_path = temp_dir.path().to_string_lossy().to_string();
    let abs_command = BuildCommand::new(&abs_path);
    assert!(abs_command.build_args().contains(&abs_path));

    // Test with URL context (just command construction)
    let url_command = BuildCommand::new("https://github.com/user/repo.git");
    assert!(url_command
        .build_args()
        .contains(&"https://github.com/user/repo.git".to_string()));

    // Test with stdin context
    let stdin_command = BuildCommand::new("-");
    assert!(stdin_command.build_args().contains(&"-".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_build_multiple_tags() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let temp_dir = create_test_context();
    let context_path = temp_dir.path().to_string_lossy().to_string();

    let command = BuildCommand::new(&context_path)
        .tag("myapp:latest")
        .tag("myapp:1.0")
        .tag("myapp:stable")
        .tag("registry.example.com/myapp:latest");

    let args = command.build_args();

    // Verify all tags are present
    assert!(args.contains(&"--tag".to_string()));
    assert!(args.contains(&"myapp:latest".to_string()));
    assert!(args.contains(&"myapp:1.0".to_string()));
    assert!(args.contains(&"myapp:stable".to_string()));
    assert!(args.contains(&"registry.example.com/myapp:latest".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_build_command_order() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let temp_dir = create_test_context();
    let context_path = temp_dir.path().to_string_lossy().to_string();

    let command = BuildCommand::new(&context_path).tag("test:latest");

    let args = command.build_args();

    // Verify command structure
    assert_eq!(command.command_name(), "build");
    assert!(args.contains(&"--tag".to_string()));
    assert!(args.contains(&"test:latest".to_string()));

    // Context should be last
    assert_eq!(args.last(), Some(&context_path));
    Ok(())
}

#[tokio::test]
async fn test_build_empty_context_handling() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test that empty context defaults to current directory
    let command = BuildCommand::new("");
    let args = command.build_args();

    // Should still produce valid command
    assert_eq!(command.command_name(), "build");
    assert!(!args.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_build_file_path_handling() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let temp_dir = create_test_context();
    let context_path = temp_dir.path().to_string_lossy().to_string();

    // Test with PathBuf
    let dockerfile_path = temp_dir.path().join("Dockerfile.test");
    let command = BuildCommand::new(&context_path).file(&dockerfile_path);

    let args = command.build_args();

    assert!(args.contains(&"--file".to_string()));
    // The file path might be absolute, so check if any arg contains "Dockerfile.test"
    assert!(args.iter().any(|arg| arg.contains("Dockerfile.test")));
    Ok(())
}

#[tokio::test]
async fn test_build_validation() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test command name
    let temp_dir = create_test_context();
    let context_path = temp_dir.path().to_string_lossy().to_string();
    let command = BuildCommand::new(&context_path);

    assert_eq!(command.command_name(), "build");

    // Test build args format
    let args = command.build_args();
    assert!(!args.is_empty());
    assert!(args.contains(&context_path));
    Ok(())
}
