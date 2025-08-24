//! Integration tests for commonly used Docker commands

use docker_wrapper::{
    AttachCommand, CpCommand, DiffCommand, DockerCommand, EventsCommand, ExportCommand,
    HistoryCommand, ImportCommand, InspectCommand, LoadCommand, LogsCommand, PortCommand,
    RmiCommand, SaveCommand, StatsCommand, TagCommand, TopCommand,
};

#[tokio::test]
async fn test_logs_command() {
    let logs_cmd = LogsCommand::new("test-container")
        .follow()
        .timestamps()
        .tail("100")
        .since("2h");

    let args = logs_cmd.build_command_args();
    assert!(args.contains(&"logs".to_string()));
    assert!(args.contains(&"--follow".to_string()));
    assert!(args.contains(&"--timestamps".to_string()));
    assert!(args.contains(&"--tail".to_string()));
    assert!(args.contains(&"100".to_string()));
    assert!(args.contains(&"--since".to_string()));
    assert!(args.contains(&"2h".to_string()));
}

#[tokio::test]
async fn test_inspect_command() {
    let inspect_cmd = InspectCommand::new("test-container")
        .object("test-image")
        .format("{{.State.Status}}")
        .size()
        .object_type("container");

    let args = inspect_cmd.build_command_args();
    assert!(args.contains(&"inspect".to_string()));
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"{{.State.Status}}".to_string()));
    assert!(args.contains(&"--size".to_string()));
    assert!(args.contains(&"--type".to_string()));
    assert!(args.contains(&"container".to_string()));
    assert!(args.contains(&"test-container".to_string()));
    assert!(args.contains(&"test-image".to_string()));
}

#[tokio::test]
async fn test_tag_command() {
    let _tag_result = TagCommand::new("alpine:latest", "my-alpine:v1.0")
        .execute()
        .await;

    // Test command building without requiring Docker
    let tag_cmd = TagCommand::new("source:tag", "target:tag");
    let args = tag_cmd.build_command_args();
    assert!(args.contains(&"tag".to_string()));
    assert!(args.contains(&"source:tag".to_string()));
    assert!(args.contains(&"target:tag".to_string()));
}

#[tokio::test]
async fn test_cp_command() {
    use std::path::Path;

    // Test copying from container to host
    let cp_from = CpCommand::from_container("container", "/app/file.txt")
        .to_host(Path::new("./file.txt"))
        .archive()
        .follow_link();

    let args = cp_from.build_command_args();
    assert!(args.contains(&"cp".to_string()));
    assert!(args.contains(&"--archive".to_string()));
    assert!(args.contains(&"--follow-link".to_string()));
    assert!(args.contains(&"container:/app/file.txt".to_string()));
    assert!(args.contains(&"./file.txt".to_string()));

    // Test copying from host to container
    let cp_to = CpCommand::from_host(Path::new("./file.txt"))
        .to_container("container", "/app/file.txt")
        .archive();

    let args = cp_to.build_command_args();
    assert!(args.contains(&"./file.txt".to_string()));
    assert!(args.contains(&"container:/app/file.txt".to_string()));
}

#[tokio::test]
async fn test_port_command() {
    let port_cmd = PortCommand::new("test-container").port(80);

    let args = port_cmd.build_command_args();
    assert!(args.contains(&"port".to_string()));
    assert!(args.contains(&"test-container".to_string()));
    assert!(args.contains(&"80".to_string()));
}

#[tokio::test]
async fn test_diff_command() {
    let diff_cmd = DiffCommand::new("test-container");

    let args = diff_cmd.build_command_args();
    assert!(args.contains(&"diff".to_string()));
    assert!(args.contains(&"test-container".to_string()));
}

#[tokio::test]
async fn test_top_command() {
    let top_cmd = TopCommand::new("test-container").ps_options("-aux");

    let args = top_cmd.build_command_args();
    assert!(args.contains(&"top".to_string()));
    assert!(args.contains(&"test-container".to_string()));
    assert!(args.contains(&"-aux".to_string()));
}

#[tokio::test]
async fn test_stats_command() {
    let stats_cmd = StatsCommand::new()
        .all()
        .no_stream()
        .format("table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}");

    let args = stats_cmd.build_command_args();
    assert!(args.contains(&"stats".to_string()));
    assert!(args.contains(&"--all".to_string()));
    assert!(args.contains(&"--no-stream".to_string()));
    assert!(args.contains(&"--format".to_string()));
}

#[tokio::test]
async fn test_events_command() {
    let events_cmd = EventsCommand::new()
        .since("2025-08-24T00:00:00")
        .until("2025-08-24T23:59:59")
        .filter("type", "container")
        .filter("event", "start")
        .format("{{json .}}");

    let args = events_cmd.build_command_args();
    assert!(args.contains(&"events".to_string()));
    assert!(args.contains(&"--since".to_string()));
    assert!(args.contains(&"2025-08-24T00:00:00".to_string()));
    assert!(args.contains(&"--until".to_string()));
    assert!(args.contains(&"--filter".to_string()));
    assert!(args.contains(&"type=container".to_string()));
    assert!(args.contains(&"event=start".to_string()));
    assert!(args.contains(&"--format".to_string()));
}

#[tokio::test]
async fn test_history_command() {
    let history_cmd = HistoryCommand::new("alpine:latest")
        .human(true)
        .no_trunc(true)
        .quiet(true);

    let args = history_cmd.build_command_args();
    assert!(args.contains(&"history".to_string()));
    assert!(args.contains(&"--human".to_string()));
    assert!(args.contains(&"--no-trunc".to_string()));
    assert!(args.contains(&"--quiet".to_string()));
    assert!(args.contains(&"alpine:latest".to_string()));
}

#[tokio::test]
async fn test_attach_command() {
    let attach_cmd = AttachCommand::new("test-container")
        .detach_keys("ctrl-p,ctrl-q")
        .no_stdin();

    let args = attach_cmd.build_command_args();
    assert!(args.contains(&"attach".to_string()));
    assert!(args.contains(&"--detach-keys".to_string()));
    assert!(args.contains(&"ctrl-p,ctrl-q".to_string()));
    assert!(args.contains(&"--no-stdin".to_string()));
    assert!(args.contains(&"test-container".to_string()));
}

#[tokio::test]
async fn test_export_import_commands() {
    // Test export
    let export_cmd = ExportCommand::new("test-container").output("/tmp/container.tar");

    let args = export_cmd.build_command_args();
    assert!(args.contains(&"export".to_string()));
    assert!(args.contains(&"--output".to_string()));
    assert!(args.contains(&"/tmp/container.tar".to_string()));
    assert!(args.contains(&"test-container".to_string()));

    // Test import
    let import_cmd = ImportCommand::new("/tmp/container.tar")
        .repository("imported-image:latest")
        .message("Imported from tar")
        .change("ENV DEBUG=true");

    let args = import_cmd.build_command_args();
    assert!(args.contains(&"import".to_string()));
    assert!(args.contains(&"--message".to_string()));
    assert!(args.contains(&"Imported from tar".to_string()));
    assert!(args.contains(&"--change".to_string()));
    assert!(args.contains(&"ENV DEBUG=true".to_string()));
    assert!(args.contains(&"/tmp/container.tar".to_string()));
    assert!(args.contains(&"imported-image:latest".to_string()));
}

#[tokio::test]
async fn test_save_load_commands() {
    use std::path::Path;

    // Test save
    let save_cmd = SaveCommand::new("alpine:latest")
        .image("nginx:latest")
        .output(Path::new("/tmp/images.tar"));

    let args = save_cmd.build_command_args();
    assert!(args.contains(&"save".to_string()));
    assert!(args.contains(&"--output".to_string()));
    assert!(args.contains(&"/tmp/images.tar".to_string()));
    assert!(args.contains(&"alpine:latest".to_string()));
    assert!(args.contains(&"nginx:latest".to_string()));

    // Test load
    let load_cmd = LoadCommand::new()
        .input(Path::new("/tmp/images.tar"))
        .quiet();

    let args = load_cmd.build_command_args();
    assert!(args.contains(&"load".to_string()));
    assert!(args.contains(&"--input".to_string()));
    assert!(args.contains(&"/tmp/images.tar".to_string()));
    assert!(args.contains(&"--quiet".to_string()));
}

#[tokio::test]
async fn test_rmi_command() {
    let rmi_cmd = RmiCommand::new("old-image:tag")
        .image("unused-image")
        .force()
        .no_prune();

    let args = rmi_cmd.build_command_args();
    assert!(args.contains(&"rmi".to_string()));
    assert!(args.contains(&"--force".to_string()));
    assert!(args.contains(&"--no-prune".to_string()));
    assert!(args.contains(&"old-image:tag".to_string()));
    assert!(args.contains(&"unused-image".to_string()));
}
