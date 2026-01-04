//! Property-based tests for docker-wrapper using proptest.
//!
//! These tests verify that:
//! 1. Builder methods handle arbitrary string inputs without panicking
//! 2. Output parsing is robust against malformed input
//! 3. Command argument building is deterministic and correct

use proptest::prelude::*;

// Import the crate under test
use docker_wrapper::{
    CreateCommand, DockerCommand, ExecCommand, ImagesCommand, KillCommand, LogsCommand, PsCommand,
    RmCommand, RunCommand, StartCommand, StopCommand,
};

// ============================================================================
// Test Strategies
// ============================================================================

/// Strategy for generating arbitrary container/image names
/// Docker allows alphanumeric, underscores, hyphens, and dots
fn docker_name_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_.-]{0,127}".prop_filter("non-empty", |s| !s.is_empty())
}

/// Strategy for generating arbitrary environment variable keys
fn env_key_strategy() -> impl Strategy<Value = String> {
    "[A-Z_][A-Z0-9_]{0,63}".prop_filter("non-empty", |s| !s.is_empty())
}

/// Strategy for generating arbitrary environment variable values
fn env_value_strategy() -> impl Strategy<Value = String> {
    ".*".prop_map(|s| s.chars().take(256).collect())
}

/// Strategy for generating port numbers
fn port_strategy() -> impl Strategy<Value = u16> {
    1u16..=65535u16
}

/// Strategy for generating memory size strings
fn memory_size_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        (1u32..10000u32).prop_map(|n| format!("{n}m")),
        (1u32..100u32).prop_map(|n| format!("{n}g")),
        (1u32..1000000u32).prop_map(|n| format!("{n}k")),
        (1u32..1000000u32).prop_map(|n| n.to_string()),
    ]
}

/// Strategy for generating CPU count strings
fn cpu_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        (1u32..128u32).prop_map(|n| n.to_string()),
        (1u32..128u32, 0u32..99u32).prop_map(|(n, d)| format!("{n}.{d}")),
        Just("0.5".to_string()),
        Just("1.5".to_string()),
        Just("2.0".to_string()),
    ]
}

/// Strategy for generating arbitrary (potentially malicious) strings
fn arbitrary_string_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Normal strings
        "[a-zA-Z0-9_.-]{0,64}",
        // Strings with special characters
        "[^\\x00]{0,64}".prop_map(|s| s.chars().filter(|c| *c != '\0').collect()),
        // Edge cases
        Just(String::new()),
        Just(" ".to_string()),
        Just("  ".to_string()),
        Just("\t".to_string()),
        Just("\n".to_string()),
        Just("'single quotes'".to_string()),
        Just("\"double quotes\"".to_string()),
        Just("back`ticks`".to_string()),
        Just("$variable".to_string()),
        Just("${variable}".to_string()),
        Just("$(command)".to_string()),
        Just("; rm -rf /".to_string()),
        Just("| cat /etc/passwd".to_string()),
        Just("&& malicious".to_string()),
        Just("|| fallback".to_string()),
        Just("name=value".to_string()),
        Just("key:value".to_string()),
        Just("path/to/file".to_string()),
        Just("../../../etc/passwd".to_string()),
        Just("C:\\Windows\\System32".to_string()),
    ]
}

/// Strategy for generating label strings (key=value format)
fn label_strategy() -> impl Strategy<Value = String> {
    ("[a-zA-Z][a-zA-Z0-9._-]{0,63}", "[a-zA-Z0-9._-]{0,127}").prop_map(|(k, v)| format!("{k}={v}"))
}

/// Strategy for generating volume mount strings
fn volume_mount_strategy() -> impl Strategy<Value = (String, String)> {
    ("/[a-zA-Z0-9/_-]{1,64}", "/[a-zA-Z0-9/_-]{1,64}")
}

// ============================================================================
// RunCommand Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Test that RunCommand handles arbitrary image names without panicking
    #[test]
    fn run_command_accepts_any_image_name(image in arbitrary_string_strategy()) {
        let cmd = RunCommand::new(image);
        let args = cmd.build_command_args();
        // Should always produce valid args starting with "run"
        prop_assert!(args.first() == Some(&"run".to_string()));
    }

    /// Test that RunCommand handles arbitrary container names
    #[test]
    fn run_command_accepts_any_container_name(
        image in docker_name_strategy(),
        name in arbitrary_string_strategy()
    ) {
        let cmd = RunCommand::new(image).name(name);
        let args = cmd.build_command_args();
        prop_assert!(args.contains(&"--name".to_string()));
    }

    /// Test that RunCommand handles arbitrary environment variables
    #[test]
    fn run_command_accepts_any_env_vars(
        image in docker_name_strategy(),
        key in env_key_strategy(),
        value in env_value_strategy()
    ) {
        let cmd = RunCommand::new(image).env(key, value);
        let args = cmd.build_command_args();
        // env vars are passed via -e flag
        prop_assert!(args.iter().any(|a| a.contains('=')));
    }

    /// Test that RunCommand handles port mappings correctly
    #[test]
    fn run_command_handles_port_mappings(
        image in docker_name_strategy(),
        host_port in port_strategy(),
        container_port in port_strategy()
    ) {
        let cmd = RunCommand::new(image).port(host_port, container_port);
        let args = cmd.build_command_args();
        prop_assert!(args.contains(&"--publish".to_string()));
        prop_assert!(args.iter().any(|a| a.contains(':')));
    }

    /// Test that RunCommand handles dynamic port mappings
    #[test]
    fn run_command_handles_dynamic_ports(
        image in docker_name_strategy(),
        container_port in port_strategy()
    ) {
        let cmd = RunCommand::new(image).dynamic_port(container_port);
        let args = cmd.build_command_args();
        prop_assert!(args.contains(&"--publish".to_string()));
    }

    /// Test that RunCommand handles memory limits
    #[test]
    fn run_command_handles_memory_limits(
        image in docker_name_strategy(),
        memory in memory_size_strategy()
    ) {
        let cmd = RunCommand::new(image).memory(memory);
        let args = cmd.build_command_args();
        prop_assert!(args.contains(&"--memory".to_string()));
    }

    /// Test that RunCommand handles CPU limits
    #[test]
    fn run_command_handles_cpu_limits(
        image in docker_name_strategy(),
        cpus in cpu_strategy()
    ) {
        let cmd = RunCommand::new(image).cpus(cpus);
        let args = cmd.build_command_args();
        prop_assert!(args.contains(&"--cpus".to_string()));
    }

    /// Test that RunCommand handles volume mounts
    #[test]
    fn run_command_handles_volumes(
        image in docker_name_strategy(),
        (source, target) in volume_mount_strategy()
    ) {
        let cmd = RunCommand::new(image).volume(source, target);
        let args = cmd.build_command_args();
        prop_assert!(args.contains(&"--volume".to_string()));
    }

    /// Test that RunCommand handles labels
    #[test]
    fn run_command_handles_labels(
        image in docker_name_strategy(),
        label in label_strategy()
    ) {
        let cmd = RunCommand::new(image).label(label);
        let args = cmd.build_command_args();
        prop_assert!(args.contains(&"--label".to_string()));
    }

    /// Test that multiple builder calls compose correctly
    #[test]
    fn run_command_builder_composition(
        image in docker_name_strategy(),
        name in docker_name_strategy(),
        host_port in port_strategy(),
        container_port in port_strategy(),
        env_key in env_key_strategy(),
        env_value in env_value_strategy()
    ) {
        let cmd = RunCommand::new(image)
            .name(name)
            .port(host_port, container_port)
            .env(env_key, env_value)
            .detach()
            .rm();

        let args = cmd.build_command_args();

        prop_assert!(args.contains(&"--name".to_string()));
        prop_assert!(args.contains(&"--publish".to_string()));
        prop_assert!(args.contains(&"--detach".to_string()));
        prop_assert!(args.contains(&"--rm".to_string()));
    }
}

// ============================================================================
// CreateCommand Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Test that CreateCommand handles arbitrary inputs
    #[test]
    fn create_command_accepts_any_image(image in arbitrary_string_strategy()) {
        let cmd = CreateCommand::new(image);
        let args = cmd.build_command_args();
        prop_assert!(args.first() == Some(&"create".to_string()));
    }

    /// Test that CreateCommand handles container names
    #[test]
    fn create_command_handles_names(
        image in docker_name_strategy(),
        name in arbitrary_string_strategy()
    ) {
        let cmd = CreateCommand::new(image).name(name);
        let args = cmd.build_command_args();
        prop_assert!(args.contains(&"--name".to_string()));
    }
}

// ============================================================================
// ExecCommand Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Test that ExecCommand handles arbitrary container IDs
    #[test]
    fn exec_command_accepts_any_container_id(container_id in arbitrary_string_strategy()) {
        let cmd = ExecCommand::new(container_id, vec!["sh".to_string()]);
        let args = cmd.build_command_args();
        prop_assert!(args.first() == Some(&"exec".to_string()));
    }

    /// Test that ExecCommand handles arbitrary commands
    #[test]
    fn exec_command_handles_commands(
        container_id in docker_name_strategy(),
        command in arbitrary_string_strategy()
    ) {
        let cmd = ExecCommand::new(container_id, vec![command]);
        let args = cmd.build_command_args();
        prop_assert!(args.first() == Some(&"exec".to_string()));
    }

    /// Test that ExecCommand handles user specifications
    #[test]
    fn exec_command_handles_user(
        container_id in docker_name_strategy(),
        user in arbitrary_string_strategy()
    ) {
        let cmd = ExecCommand::new(container_id, vec!["sh".to_string()]).user(user);
        let args = cmd.build_command_args();
        prop_assert!(args.contains(&"--user".to_string()));
    }
}

// ============================================================================
// PsCommand Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Test that PsCommand handles arbitrary filter values
    #[test]
    fn ps_command_handles_filters(filter in arbitrary_string_strategy()) {
        let cmd = PsCommand::new().filter(filter);
        let args = cmd.build_command_args();
        prop_assert!(args.contains(&"--filter".to_string()));
    }

    /// Test that PsCommand handles arbitrary format templates
    #[test]
    fn ps_command_handles_format(format in arbitrary_string_strategy()) {
        let cmd = PsCommand::new().format_template(format);
        let args = cmd.build_command_args();
        prop_assert!(args.contains(&"--format".to_string()));
    }

    /// Test that PsCommand handles last count values
    #[test]
    fn ps_command_handles_last(n in -100i32..1000i32) {
        let cmd = PsCommand::new().last(n);
        let args = cmd.build_command_args();
        prop_assert!(args.contains(&"--last".to_string()));
    }
}

// ============================================================================
// ImagesCommand Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Test that ImagesCommand handles arbitrary repository patterns
    #[test]
    fn images_command_handles_repository(repo in arbitrary_string_strategy()) {
        let cmd = ImagesCommand::new().repository(repo);
        let args = cmd.build_command_args();
        prop_assert!(args.first() == Some(&"images".to_string()));
    }

    /// Test that ImagesCommand handles arbitrary filters
    #[test]
    fn images_command_handles_filters(filter in arbitrary_string_strategy()) {
        let cmd = ImagesCommand::new().filter(filter);
        let args = cmd.build_command_args();
        prop_assert!(args.contains(&"--filter".to_string()));
    }
}

// ============================================================================
// StopCommand Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Test that StopCommand handles arbitrary container IDs
    #[test]
    fn stop_command_handles_container_ids(container_id in arbitrary_string_strategy()) {
        let cmd = StopCommand::new(container_id);
        let args = cmd.build_command_args();
        prop_assert!(args.first() == Some(&"stop".to_string()));
    }

    /// Test that StopCommand handles timeout values
    #[test]
    fn stop_command_handles_timeout(
        container_id in docker_name_strategy(),
        timeout in 0u32..3600u32
    ) {
        let cmd = StopCommand::new(container_id).timeout(timeout);
        let args = cmd.build_command_args();
        prop_assert!(args.contains(&"--timeout".to_string()));
    }
}

// ============================================================================
// StartCommand Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Test that StartCommand handles arbitrary container IDs
    #[test]
    fn start_command_handles_container_ids(container_id in arbitrary_string_strategy()) {
        let cmd = StartCommand::new(container_id);
        let args = cmd.build_command_args();
        prop_assert!(args.first() == Some(&"start".to_string()));
    }
}

// ============================================================================
// RmCommand Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Test that RmCommand handles arbitrary container IDs
    #[test]
    fn rm_command_handles_container_ids(container_id in arbitrary_string_strategy()) {
        let cmd = RmCommand::new(container_id);
        let args = cmd.build_command_args();
        prop_assert!(args.first() == Some(&"rm".to_string()));
    }

    /// Test that RmCommand handles multiple containers
    #[test]
    fn rm_command_handles_multiple_containers(
        id1 in docker_name_strategy(),
        id2 in docker_name_strategy(),
        id3 in docker_name_strategy()
    ) {
        let cmd = RmCommand::new(id1).container(id2).container(id3);
        let args = cmd.build_command_args();
        prop_assert!(args.first() == Some(&"rm".to_string()));
        // Should have 3 container IDs after the command and flags
    }
}

// ============================================================================
// KillCommand Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Test that KillCommand handles arbitrary container IDs
    #[test]
    fn kill_command_handles_container_ids(container_id in arbitrary_string_strategy()) {
        let cmd = KillCommand::new(container_id);
        let args = cmd.build_command_args();
        prop_assert!(args.first() == Some(&"kill".to_string()));
    }

    /// Test that KillCommand handles arbitrary signals
    #[test]
    fn kill_command_handles_signals(
        container_id in docker_name_strategy(),
        signal in arbitrary_string_strategy()
    ) {
        let cmd = KillCommand::new(container_id).signal(signal);
        let args = cmd.build_command_args();
        prop_assert!(args.contains(&"--signal".to_string()));
    }
}

// ============================================================================
// LogsCommand Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Test that LogsCommand handles arbitrary container IDs
    #[test]
    fn logs_command_handles_container_ids(container_id in arbitrary_string_strategy()) {
        let cmd = LogsCommand::new(container_id);
        let args = cmd.build_command_args();
        prop_assert!(args.first() == Some(&"logs".to_string()));
    }

    /// Test that LogsCommand handles tail values
    #[test]
    fn logs_command_handles_tail(
        container_id in docker_name_strategy(),
        tail in arbitrary_string_strategy()
    ) {
        let cmd = LogsCommand::new(container_id).tail(tail);
        let args = cmd.build_command_args();
        prop_assert!(args.contains(&"--tail".to_string()));
    }

    /// Test that LogsCommand handles since timestamps
    #[test]
    fn logs_command_handles_since(
        container_id in docker_name_strategy(),
        since in arbitrary_string_strategy()
    ) {
        let cmd = LogsCommand::new(container_id).since(since);
        let args = cmd.build_command_args();
        prop_assert!(args.contains(&"--since".to_string()));
    }
}

// ============================================================================
// Output Parsing Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Test that ContainerId::short() never panics on any input and returns at most 12 characters
    #[test]
    fn container_id_short_never_panics(id in ".*") {
        let container_id = docker_wrapper::ContainerId(id);
        let short = container_id.short();
        // Count characters, not bytes (since short() now returns at most 12 characters)
        let char_count = short.chars().count();
        prop_assert!(char_count <= 12);
        // If original has fewer than 12 chars, short should return the whole thing
        let original_char_count = container_id.0.chars().count();
        if original_char_count < 12 {
            prop_assert_eq!(short, container_id.0.as_str());
        }
    }

    /// Test that ContainerId::as_str() returns the original value
    #[test]
    fn container_id_as_str_returns_original(id in ".*") {
        let container_id = docker_wrapper::ContainerId(id.clone());
        prop_assert_eq!(container_id.as_str(), id.as_str());
    }
}

// ============================================================================
// Builder Idempotency and Ordering Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// Test that calling the same builder method multiple times works correctly
    #[test]
    fn run_command_multiple_envs(
        image in docker_name_strategy(),
        envs in prop::collection::vec((env_key_strategy(), env_value_strategy()), 1..10)
    ) {
        let mut cmd = RunCommand::new(image);
        for (key, value) in envs {
            cmd = cmd.env(key, value);
        }
        let args = cmd.build_command_args();
        prop_assert!(args.first() == Some(&"run".to_string()));
    }

    /// Test that calling multiple port mappings works correctly
    #[test]
    fn run_command_multiple_ports(
        image in docker_name_strategy(),
        ports in prop::collection::vec((port_strategy(), port_strategy()), 1..10)
    ) {
        let mut cmd = RunCommand::new(image);
        for (host, container) in ports {
            cmd = cmd.port(host, container);
        }
        let args = cmd.build_command_args();
        prop_assert!(args.first() == Some(&"run".to_string()));
        // Count --publish occurrences
        let publish_count = args.iter().filter(|a| *a == "--publish").count();
        prop_assert!(publish_count >= 1);
    }

    /// Test that calling multiple volume mounts works correctly
    #[test]
    fn run_command_multiple_volumes(
        image in docker_name_strategy(),
        volumes in prop::collection::vec(volume_mount_strategy(), 1..10)
    ) {
        let mut cmd = RunCommand::new(image);
        for (source, target) in volumes {
            cmd = cmd.volume(source, target);
        }
        let args = cmd.build_command_args();
        prop_assert!(args.first() == Some(&"run".to_string()));
    }

    /// Test that calling multiple labels works correctly
    #[test]
    fn run_command_multiple_labels(
        image in docker_name_strategy(),
        labels in prop::collection::vec(label_strategy(), 1..10)
    ) {
        let mut cmd = RunCommand::new(image);
        for label in labels {
            cmd = cmd.label(label);
        }
        let args = cmd.build_command_args();
        prop_assert!(args.first() == Some(&"run".to_string()));
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// Test handling of very long strings
    #[test]
    fn run_command_handles_long_strings(
        len in 100usize..1000usize
    ) {
        let long_string: String = "a".repeat(len);
        let cmd = RunCommand::new(&long_string).name(&long_string);
        let args = cmd.build_command_args();
        prop_assert!(args.first() == Some(&"run".to_string()));
    }

    /// Test handling of unicode strings
    #[test]
    fn run_command_handles_unicode(
        unicode in "[\\p{L}\\p{N}]{0,64}"
    ) {
        let cmd = RunCommand::new(unicode);
        let args = cmd.build_command_args();
        prop_assert!(args.first() == Some(&"run".to_string()));
    }
}

// ============================================================================
// Determinism Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// Test that building args is deterministic
    #[test]
    fn run_command_is_deterministic(
        image in docker_name_strategy(),
        name in docker_name_strategy(),
        host_port in port_strategy(),
        container_port in port_strategy()
    ) {
        let cmd1 = RunCommand::new(image.clone())
            .name(name.clone())
            .port(host_port, container_port)
            .detach();

        let cmd2 = RunCommand::new(image)
            .name(name)
            .port(host_port, container_port)
            .detach();

        let args1 = cmd1.build_command_args();
        let args2 = cmd2.build_command_args();

        prop_assert_eq!(args1, args2);
    }
}
