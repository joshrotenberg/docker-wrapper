//! Verify that `DockerCommand::execute` emits the expected tracing spans and
//! events documented in the `Tracing` section of the README.
//!
//! These tests do not require Docker to be installed: they invoke a command
//! that is expected to fail (or to produce a deterministic exit code) via the
//! `GenericCommand` escape hatch and a fake binary path forced onto `PATH`.
//! The assertions are about the tracing output, not about Docker behaviour.

#![cfg(feature = "tracing")]

use docker_wrapper::{DockerCommand, GenericCommand};
use tracing_test::traced_test;

/// Invoke a command that we expect to fail (a non-existent Docker subcommand
/// using the real docker binary if present, otherwise a spawn failure via a
/// bogus `DOCKER_HOST`). Either branch still produces tracing output.
async fn run_failing_command() {
    // `__docker_wrapper_nonexistent__` will never be a valid docker command.
    let _ = GenericCommand::new("__docker_wrapper_nonexistent__")
        .execute()
        .await;
}

#[tokio::test]
#[traced_test]
async fn docker_command_span_is_emitted() {
    run_failing_command().await;

    // The top-level span is named `docker.command` and carries the command
    // name as a field.
    assert!(
        logs_contain("docker.command"),
        "expected `docker.command` span to be emitted"
    );
    assert!(
        logs_contain("__docker_wrapper_nonexistent__"),
        "expected command name to appear in tracing fields"
    );
}

#[tokio::test]
#[traced_test]
async fn failing_command_emits_warn_event() {
    run_failing_command().await;

    // Non-zero exit or spawn failure should produce a warn-level "command
    // failed" event with a duration_ms field.
    assert!(
        logs_contain("command failed"),
        "expected `command failed` warn event to be emitted"
    );
    assert!(
        logs_contain("duration_ms"),
        "expected `duration_ms` field on the failure event"
    );
}

#[tokio::test]
#[traced_test]
async fn args_count_is_recorded() {
    // Execute a command with a known number of arguments. GenericCommand's
    // builder methods consume `self`.
    let cmd = GenericCommand::new("__docker_wrapper_nonexistent__")
        .arg("--flag-one")
        .arg("--flag-two")
        .arg("value");
    let _ = cmd.execute().await;

    // The instrument attribute records `args_count` as a span field.
    assert!(
        logs_contain("args_count"),
        "expected `args_count` field to be recorded on the docker.command span"
    );
}
