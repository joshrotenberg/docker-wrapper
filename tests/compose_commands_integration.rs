//! Integration tests for new Docker Compose commands

#[cfg(feature = "compose")]
mod compose_tests {
    use docker_wrapper::compose::{
        ComposeAttachCommand, ComposeConfigCommand, ComposeConvertCommand, ComposeCpCommand,
        ComposeCreateCommand, ComposeEventsCommand, ComposeImagesCommand, ComposeKillCommand,
        ComposeLsCommand, ComposePauseCommand, ComposePortCommand, ComposePushCommand,
        ComposeRmCommand, ComposeScaleCommand, ComposeTopCommand, ComposeUnpauseCommand,
        ComposeVersionCommand, ComposeWaitCommand, ComposeWatchCommand, ConfigFormat,
        ConvertFormat, ImagesFormat, LsFormat, PullPolicy, VersionFormat,
    };

    #[test]
    fn test_compose_config_command_creation() {
        let cmd = ComposeConfigCommand::new()
            .file("docker-compose.yml")
            .format(ConfigFormat::Json)
            .services()
            .quiet();

        // Just verify the command can be created and configured
        assert!(cmd.services);
        assert!(cmd.quiet);
    }

    #[test]
    fn test_compose_rm_command_creation() {
        let cmd = ComposeRmCommand::new()
            .force()
            .stop()
            .volumes()
            .service("web");

        assert!(cmd.force);
        assert!(cmd.stop);
        assert!(cmd.volumes);
        assert_eq!(cmd.services.len(), 1);
    }

    #[test]
    fn test_compose_kill_command_creation() {
        let cmd = ComposeKillCommand::new()
            .signal("SIGTERM")
            .remove_orphans()
            .service("worker");

        assert_eq!(cmd.signal, Some("SIGTERM".to_string()));
        assert!(cmd.remove_orphans);
    }

    #[test]
    fn test_compose_ls_command_creation() {
        let cmd = ComposeLsCommand::new()
            .all()
            .format(LsFormat::Json)
            .filter("status=running");

        assert!(cmd.all);
        assert!(matches!(cmd.format, Some(LsFormat::Json)));
    }

    #[test]
    fn test_compose_pause_unpause_commands() {
        let pause_cmd = ComposePauseCommand::new().service("web").service("db");

        let unpause_cmd = ComposeUnpauseCommand::new().service("web").service("db");

        assert_eq!(pause_cmd.services.len(), 2);
        assert_eq!(unpause_cmd.services.len(), 2);
    }

    #[test]
    fn test_compose_create_command_with_pull_policy() {
        let cmd = ComposeCreateCommand::new()
            .pull(PullPolicy::Always)
            .build()
            .force_recreate();

        assert!(matches!(cmd.pull, Some(PullPolicy::Always)));
        assert!(cmd.build);
        assert!(cmd.force_recreate);
    }

    #[test]
    fn test_compose_scale_command() {
        let cmd = ComposeScaleCommand::new()
            .scale("web", 3)
            .scale("worker", 5)
            .no_deps();

        assert_eq!(cmd.scales.len(), 2);
        assert_eq!(cmd.scales.get("web"), Some(&3));
        assert_eq!(cmd.scales.get("worker"), Some(&5));
        assert!(cmd.no_deps);
    }

    #[test]
    fn test_compose_images_command() {
        let cmd = ComposeImagesCommand::new()
            .format(ImagesFormat::Json)
            .quiet();

        assert!(matches!(cmd.format, Some(ImagesFormat::Json)));
        assert!(cmd.quiet);
    }

    #[test]
    fn test_compose_top_command() {
        let cmd = ComposeTopCommand::new().service("web").service("worker");

        assert_eq!(cmd.services.len(), 2);
    }

    #[test]
    fn test_compose_port_command() {
        let cmd = ComposePortCommand::new("web", 80).protocol("tcp").index(1);

        assert_eq!(cmd.service, "web");
        assert_eq!(cmd.private_port, 80);
        assert_eq!(cmd.protocol, Some("tcp".to_string()));
        assert_eq!(cmd.index, Some(1));
    }

    #[test]
    fn test_compose_cp_command() {
        let cmd = ComposeCpCommand::from_container("web", "/app/logs", "./logs")
            .archive()
            .follow_link();

        assert!(cmd.archive);
        assert!(cmd.follow_link);
        assert!(cmd.source.contains("web:/app/logs"));
    }

    #[test]
    fn test_compose_push_command() {
        let cmd = ComposePushCommand::new()
            .include_deps()
            .ignore_push_failures()
            .service("api");

        assert!(cmd.include_deps);
        assert!(cmd.ignore_push_failures);
        assert_eq!(cmd.services.len(), 1);
    }

    #[test]
    fn test_compose_version_command() {
        let cmd = ComposeVersionCommand::new()
            .format(VersionFormat::Json)
            .short();

        assert!(matches!(cmd.format, Some(VersionFormat::Json)));
        assert!(cmd.short);
    }

    #[test]
    fn test_compose_events_command() {
        let cmd = ComposeEventsCommand::new()
            .json()
            .since("2025-08-23T00:00:00")
            .until("2025-08-23T23:59:59")
            .service("web");

        assert!(cmd.json);
        assert_eq!(cmd.since, Some("2025-08-23T00:00:00".to_string()));
        assert_eq!(cmd.until, Some("2025-08-23T23:59:59".to_string()));
    }

    #[test]
    fn test_multiple_services_configuration() {
        // Test that multiple services can be added in different ways
        let services = vec!["web", "db", "cache"];

        let cmd1 = ComposeRmCommand::new().services(services.clone());

        let mut cmd2 = ComposeRmCommand::new();
        for service in &services {
            cmd2 = cmd2.service(*service);
        }

        assert_eq!(cmd1.services.len(), 3);
        assert_eq!(cmd2.services.len(), 3);
    }

    #[test]
    fn test_compose_watch_command() {
        let cmd = ComposeWatchCommand::new()
            .no_up()
            .service("web")
            .service("worker");

        assert!(cmd.no_up);
        assert_eq!(cmd.services.len(), 2);
    }

    #[test]
    fn test_compose_attach_command() {
        let cmd = ComposeAttachCommand::new("web")
            .detach_keys("ctrl-p,ctrl-q")
            .index(1)
            .no_stdin();

        assert_eq!(cmd.service, "web");
        assert_eq!(cmd.detach_keys, Some("ctrl-p,ctrl-q".to_string()));
        assert_eq!(cmd.index, Some(1));
        assert!(cmd.no_stdin);
    }

    #[test]
    fn test_compose_wait_command() {
        let cmd = ComposeWaitCommand::new()
            .down_project()
            .service("web")
            .service("db");

        assert!(cmd.down_project);
        assert_eq!(cmd.services.len(), 2);
    }

    #[test]
    fn test_compose_convert_command() {
        let cmd = ComposeConvertCommand::new()
            .format(ConvertFormat::Json)
            .output("compose.json")
            .services()
            .quiet();

        assert!(matches!(cmd.format, Some(ConvertFormat::Json)));
        assert_eq!(cmd.output, Some("compose.json".to_string()));
        assert!(cmd.services);
        assert!(cmd.quiet);
    }
}
