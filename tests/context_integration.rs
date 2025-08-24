//! Integration tests for Docker context commands

use docker_wrapper::command::context::{
    ContextCreateCommand, ContextInspectCommand, ContextLsCommand, ContextRmCommand,
    ContextUpdateCommand, ContextUseCommand,
};
use docker_wrapper::DockerCommand;

#[tokio::test]
async fn test_context_ls_command() {
    let ls_cmd = ContextLsCommand::new().quiet();

    let args = ls_cmd.build_command_args();
    assert!(args.contains(&"context".to_string()));
    assert!(args.contains(&"ls".to_string()));
    assert!(args.contains(&"--quiet".to_string()));
}

#[tokio::test]
async fn test_context_ls_with_format() {
    let ls_cmd = ContextLsCommand::new().format("{{.Name}}");

    let args = ls_cmd.build_command_args();
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"{{.Name}}".to_string()));
}

#[tokio::test]
async fn test_context_create_command() {
    let create_cmd = ContextCreateCommand::new("test-context")
        .description("Test context for integration testing")
        .docker_host("unix:///var/run/docker.sock");

    let args = create_cmd.build_command_args();
    assert!(args.contains(&"context".to_string()));
    assert!(args.contains(&"create".to_string()));
    assert!(args.contains(&"test-context".to_string()));
    assert!(args.contains(&"--description".to_string()));
    assert!(args.contains(&"Test context for integration testing".to_string()));
    assert!(args.contains(&"--docker".to_string()));
    assert!(args.contains(&"host=unix:///var/run/docker.sock".to_string()));
}

#[tokio::test]
async fn test_context_create_from_existing() {
    let create_cmd = ContextCreateCommand::new("new-context")
        .from("default")
        .description("Created from default");

    let args = create_cmd.build_command_args();
    assert!(args.contains(&"--from".to_string()));
    assert!(args.contains(&"default".to_string()));
}

#[tokio::test]
async fn test_context_use_command() {
    let use_cmd = ContextUseCommand::new("production");

    let args = use_cmd.build_command_args();
    assert!(args.contains(&"context".to_string()));
    assert!(args.contains(&"use".to_string()));
    assert!(args.contains(&"production".to_string()));
}

#[tokio::test]
async fn test_context_inspect_command() {
    let inspect_cmd = ContextInspectCommand::new("default").format("{{.Endpoints.docker.Host}}");

    let args = inspect_cmd.build_command_args();
    assert!(args.contains(&"context".to_string()));
    assert!(args.contains(&"inspect".to_string()));
    assert!(args.contains(&"default".to_string()));
    assert!(args.contains(&"--format".to_string()));
}

#[tokio::test]
async fn test_context_inspect_multiple() {
    let inspect_cmd = ContextInspectCommand::new("context1").add_context("context2");

    let args = inspect_cmd.build_command_args();
    assert!(args.contains(&"context1".to_string()));
    assert!(args.contains(&"context2".to_string()));
}

#[tokio::test]
async fn test_context_update_command() {
    let update_cmd = ContextUpdateCommand::new("test-context")
        .description("Updated description")
        .docker_host("tcp://127.0.0.1:2376");

    let args = update_cmd.build_command_args();
    assert!(args.contains(&"context".to_string()));
    assert!(args.contains(&"update".to_string()));
    assert!(args.contains(&"test-context".to_string()));
    assert!(args.contains(&"--description".to_string()));
    assert!(args.contains(&"Updated description".to_string()));
    assert!(args.contains(&"--docker".to_string()));
    assert!(args.contains(&"host=tcp://127.0.0.1:2376".to_string()));
}

#[tokio::test]
async fn test_context_rm_command() {
    let rm_cmd = ContextRmCommand::new("test-context").force();

    let args = rm_cmd.build_command_args();
    assert!(args.contains(&"context".to_string()));
    assert!(args.contains(&"rm".to_string()));
    assert!(args.contains(&"test-context".to_string()));
    assert!(args.contains(&"--force".to_string()));
}

#[tokio::test]
async fn test_context_rm_multiple() {
    let rm_cmd = ContextRmCommand::new("context1")
        .add_context("context2")
        .add_context("context3");

    let args = rm_cmd.build_command_args();
    assert!(args.contains(&"context1".to_string()));
    assert!(args.contains(&"context2".to_string()));
    assert!(args.contains(&"context3".to_string()));
}

#[tokio::test]
async fn test_context_lifecycle() {
    // Test a complete lifecycle of context operations
    // Note: This test requires Docker to be available
    let context_name = format!("test-context-{}", uuid::Uuid::new_v4());

    // List existing contexts
    let ls_result = ContextLsCommand::new().quiet().execute().await;

    if ls_result.is_err() {
        eprintln!("Skipping test - Docker not available");
        return;
    }

    // Create a new context
    let create_result = ContextCreateCommand::new(&context_name)
        .description("Test context for automated testing")
        .from("default")
        .execute()
        .await;

    if create_result.is_ok() {
        // Inspect the created context
        let inspect_result = ContextInspectCommand::new(&context_name).execute().await;
        assert!(inspect_result.is_ok());

        // Update the context
        let update_result = ContextUpdateCommand::new(&context_name)
            .description("Updated test context")
            .execute()
            .await;
        assert!(update_result.is_ok());

        // Remove the context
        let rm_result = ContextRmCommand::new(&context_name).force().execute().await;
        assert!(rm_result.is_ok());
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
