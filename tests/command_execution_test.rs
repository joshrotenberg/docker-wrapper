//! Tests to verify command execution patterns
//!
//! These tests ensure that commands properly use self.execute_command()
//! rather than self.executor.execute_command("docker", ...) which would
//! result in "docker docker <cmd>" being executed.

use docker_wrapper::DockerCommand;

/// Verifies that build_command_args() returns the command name as first element,
/// not "docker". The executor layer handles prepending "docker".
fn verify_command_args_no_docker_prefix(args: &[String], command_name: &str) {
    assert!(
        !args.is_empty(),
        "{command_name}: build_command_args() returned empty"
    );
    assert_ne!(
        args[0], "docker",
        "{command_name}: build_command_args() should not start with 'docker' - \
         that's added by the executor. Found args: {args:?}"
    );
}

#[test]
fn test_pull_command_no_docker_prefix() {
    use docker_wrapper::PullCommand;
    let cmd = PullCommand::new("alpine:latest");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "PullCommand");
    assert_eq!(args[0], "pull");
}

#[test]
fn test_push_command_no_docker_prefix() {
    use docker_wrapper::PushCommand;
    let cmd = PushCommand::new("myimage:latest");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "PushCommand");
    assert_eq!(args[0], "push");
}

#[test]
fn test_build_command_no_docker_prefix() {
    use docker_wrapper::BuildCommand;
    let cmd = BuildCommand::new(".");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "BuildCommand");
    assert_eq!(args[0], "build");
}

#[test]
fn test_images_command_no_docker_prefix() {
    use docker_wrapper::ImagesCommand;
    let cmd = ImagesCommand::new();
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "ImagesCommand");
    assert_eq!(args[0], "images");
}

#[test]
fn test_run_command_no_docker_prefix() {
    use docker_wrapper::RunCommand;
    let cmd = RunCommand::new("alpine:latest");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "RunCommand");
    assert_eq!(args[0], "run");
}

#[test]
fn test_ps_command_no_docker_prefix() {
    use docker_wrapper::PsCommand;
    let cmd = PsCommand::new();
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "PsCommand");
    assert_eq!(args[0], "ps");
}

#[test]
fn test_exec_command_no_docker_prefix() {
    use docker_wrapper::ExecCommand;
    let cmd = ExecCommand::new("container_id", vec!["ls".to_string(), "-la".to_string()]);
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "ExecCommand");
    assert_eq!(args[0], "exec");
}

#[test]
fn test_login_command_no_docker_prefix() {
    use docker_wrapper::LoginCommand;
    let cmd = LoginCommand::new("user", "pass");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "LoginCommand");
    assert_eq!(args[0], "login");
}

#[test]
fn test_logout_command_no_docker_prefix() {
    use docker_wrapper::LogoutCommand;
    let cmd = LogoutCommand::new();
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "LogoutCommand");
    assert_eq!(args[0], "logout");
}

#[test]
fn test_tag_command_no_docker_prefix() {
    use docker_wrapper::TagCommand;
    let cmd = TagCommand::new("source:tag", "target:tag");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "TagCommand");
    assert_eq!(args[0], "tag");
}

#[test]
fn test_search_command_no_docker_prefix() {
    use docker_wrapper::SearchCommand;
    let cmd = SearchCommand::new("nginx");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "SearchCommand");
    assert_eq!(args[0], "search");
}

#[test]
fn test_info_command_no_docker_prefix() {
    use docker_wrapper::InfoCommand;
    let cmd = InfoCommand::new();
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "InfoCommand");
    assert_eq!(args[0], "info");
}

#[test]
fn test_version_command_no_docker_prefix() {
    use docker_wrapper::VersionCommand;
    let cmd = VersionCommand::new();
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "VersionCommand");
    assert_eq!(args[0], "version");
}

#[test]
fn test_inspect_command_no_docker_prefix() {
    use docker_wrapper::InspectCommand;
    let cmd = InspectCommand::new("container_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "InspectCommand");
    assert_eq!(args[0], "inspect");
}

#[test]
fn test_logs_command_no_docker_prefix() {
    use docker_wrapper::LogsCommand;
    let cmd = LogsCommand::new("container_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "LogsCommand");
    assert_eq!(args[0], "logs");
}

#[test]
fn test_stop_command_no_docker_prefix() {
    use docker_wrapper::StopCommand;
    let cmd = StopCommand::new("container_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "StopCommand");
    assert_eq!(args[0], "stop");
}

#[test]
fn test_start_command_no_docker_prefix() {
    use docker_wrapper::StartCommand;
    let cmd = StartCommand::new("container_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "StartCommand");
    assert_eq!(args[0], "start");
}

#[test]
fn test_rm_command_no_docker_prefix() {
    use docker_wrapper::RmCommand;
    let cmd = RmCommand::new("container_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "RmCommand");
    assert_eq!(args[0], "rm");
}

#[test]
fn test_rmi_command_no_docker_prefix() {
    use docker_wrapper::RmiCommand;
    let cmd = RmiCommand::new("image_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "RmiCommand");
    assert_eq!(args[0], "rmi");
}

#[test]
fn test_create_command_no_docker_prefix() {
    use docker_wrapper::CreateCommand;
    let cmd = CreateCommand::new("alpine:latest");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "CreateCommand");
    assert_eq!(args[0], "create");
}

#[test]
fn test_kill_command_no_docker_prefix() {
    use docker_wrapper::KillCommand;
    let cmd = KillCommand::new("container_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "KillCommand");
    assert_eq!(args[0], "kill");
}

#[test]
fn test_pause_command_no_docker_prefix() {
    use docker_wrapper::PauseCommand;
    let cmd = PauseCommand::new("container_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "PauseCommand");
    assert_eq!(args[0], "pause");
}

#[test]
fn test_unpause_command_no_docker_prefix() {
    use docker_wrapper::UnpauseCommand;
    let cmd = UnpauseCommand::new("container_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "UnpauseCommand");
    assert_eq!(args[0], "unpause");
}

#[test]
fn test_restart_command_no_docker_prefix() {
    use docker_wrapper::RestartCommand;
    let cmd = RestartCommand::new("container_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "RestartCommand");
    assert_eq!(args[0], "restart");
}

#[test]
fn test_rename_command_no_docker_prefix() {
    use docker_wrapper::RenameCommand;
    let cmd = RenameCommand::new("old_name", "new_name");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "RenameCommand");
    assert_eq!(args[0], "rename");
}

#[test]
fn test_wait_command_no_docker_prefix() {
    use docker_wrapper::WaitCommand;
    let cmd = WaitCommand::new("container_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "WaitCommand");
    assert_eq!(args[0], "wait");
}

#[test]
fn test_attach_command_no_docker_prefix() {
    use docker_wrapper::AttachCommand;
    let cmd = AttachCommand::new("container_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "AttachCommand");
    assert_eq!(args[0], "attach");
}

#[test]
fn test_commit_command_no_docker_prefix() {
    use docker_wrapper::CommitCommand;
    let cmd = CommitCommand::new("container_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "CommitCommand");
    assert_eq!(args[0], "commit");
}

#[test]
fn test_diff_command_no_docker_prefix() {
    use docker_wrapper::DiffCommand;
    let cmd = DiffCommand::new("container_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "DiffCommand");
    assert_eq!(args[0], "diff");
}

#[test]
fn test_export_command_no_docker_prefix() {
    use docker_wrapper::ExportCommand;
    let cmd = ExportCommand::new("container_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "ExportCommand");
    assert_eq!(args[0], "export");
}

#[test]
fn test_import_command_no_docker_prefix() {
    use docker_wrapper::ImportCommand;
    let cmd = ImportCommand::new("file.tar");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "ImportCommand");
    assert_eq!(args[0], "import");
}

#[test]
fn test_history_command_no_docker_prefix() {
    use docker_wrapper::HistoryCommand;
    let cmd = HistoryCommand::new("image_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "HistoryCommand");
    assert_eq!(args[0], "history");
}

#[test]
fn test_load_command_no_docker_prefix() {
    use docker_wrapper::LoadCommand;
    let cmd = LoadCommand::new();
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "LoadCommand");
    assert_eq!(args[0], "load");
}

#[test]
fn test_save_command_no_docker_prefix() {
    use docker_wrapper::SaveCommand;
    let cmd = SaveCommand::new("image_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "SaveCommand");
    assert_eq!(args[0], "save");
}

#[test]
fn test_top_command_no_docker_prefix() {
    use docker_wrapper::TopCommand;
    let cmd = TopCommand::new("container_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "TopCommand");
    assert_eq!(args[0], "top");
}

#[test]
fn test_stats_command_no_docker_prefix() {
    use docker_wrapper::StatsCommand;
    let cmd = StatsCommand::new();
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "StatsCommand");
    assert_eq!(args[0], "stats");
}

#[test]
fn test_port_command_no_docker_prefix() {
    use docker_wrapper::PortCommand;
    let cmd = PortCommand::new("container_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "PortCommand");
    assert_eq!(args[0], "port");
}

#[test]
fn test_cp_command_no_docker_prefix() {
    use docker_wrapper::CpCommand;
    use std::path::Path;
    let cmd = CpCommand::from_container("container_id", "/container/path")
        .to_host(Path::new("/local/path"));
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "CpCommand");
    assert_eq!(args[0], "cp");
}

#[test]
fn test_update_command_no_docker_prefix() {
    use docker_wrapper::UpdateCommand;
    let cmd = UpdateCommand::new("container_id");
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "UpdateCommand");
    assert_eq!(args[0], "update");
}

#[test]
fn test_events_command_no_docker_prefix() {
    use docker_wrapper::EventsCommand;
    let cmd = EventsCommand::new();
    let args = cmd.build_command_args();
    verify_command_args_no_docker_prefix(&args, "EventsCommand");
    assert_eq!(args[0], "events");
}
