//! Phase 2 Unit Tests
//!
//! Unit tests for Phase 2 components that don't require Docker daemon.
//! These test the internal logic, configuration parsing, and data structures.

use docker_wrapper::container::health::HealthCheck;

use docker_wrapper::types::{
    ContainerId, ContainerStatus, HealthCheck as TypeHealthCheck, NetworkId, PortMapping, Protocol,
    RestartPolicy,
};
use docker_wrapper::{
    ContainerBuilder, ContainerConfig, DockerContainer, ExecConfig, ExecResult, HealthCheckConfig,
    HealthCheckResult, LogEntry, LogOptions, LogSource, RemoveOptions,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

#[test]
fn test_container_config_default() {
    let config = ContainerConfig::default();

    assert!(config.image.is_empty());
    assert!(config.name.is_none());
    assert!(config.command.is_none());
    assert!(config.entrypoint.is_none());
    assert!(config.working_dir.is_none());
    assert!(config.environment.is_empty());
    assert!(config.ports.is_empty());
    assert!(config.volumes.is_empty());
    assert!(config.labels.is_empty());
    assert!(matches!(config.restart_policy, RestartPolicy::No));
    assert!(config.health_check.is_none());
    assert!(config.networks.is_empty());
    assert!(config.user.is_none());
    assert!(!config.privileged);
    assert!(config.capabilities.is_empty());
    assert!(!config.auto_remove);
    assert!(config.detached);
    assert!(!config.interactive);
    assert!(!config.tty);
}

#[test]
fn test_container_builder_fluent_api() {
    let config = ContainerBuilder::new("redis:alpine")
        .name("test-redis")
        .env("REDIS_PASSWORD", "secret")
        .env("REDIS_PORT", "6379")
        .port(8080, 6379)
        .port_dynamic(6380)
        .port_udp(5353, 53)
        .volume("/host/data", "/container/data")
        .volume_ro("/host/config", "/container/config")
        .volume_named("data-volume", "/data")
        .volume_tmp("/tmp")
        .label("app", "redis")
        .label("version", "7.2")
        .memory_str("512m")
        .cpus(2.0)
        .user("redis")
        .privileged()
        .capability("NET_ADMIN")
        .capability("SYS_TIME")
        .auto_remove()
        .interactive()
        .tty()
        .working_dir("/app")
        .command(vec![
            "redis-server".to_string(),
            "--port".to_string(),
            "6379".to_string(),
        ])
        .build();

    // Basic properties
    assert_eq!(config.image, "redis:alpine");
    assert_eq!(config.name, Some("test-redis".to_string()));
    assert_eq!(config.working_dir, Some(PathBuf::from("/app")));
    assert_eq!(
        config.command,
        Some(vec![
            "redis-server".to_string(),
            "--port".to_string(),
            "6379".to_string()
        ])
    );

    // Environment variables
    assert_eq!(config.environment.len(), 2);
    assert_eq!(
        config.environment.get("REDIS_PASSWORD"),
        Some(&"secret".to_string())
    );
    assert_eq!(
        config.environment.get("REDIS_PORT"),
        Some(&"6379".to_string())
    );

    // Port mappings
    assert_eq!(config.ports.len(), 3);

    // Check TCP port mapping
    let tcp_port = &config.ports[0];
    assert_eq!(tcp_port.host_port, Some(8080));
    assert_eq!(tcp_port.container_port, 6379);
    assert_eq!(tcp_port.protocol, Protocol::Tcp);

    // Check dynamic port mapping
    let dynamic_port = &config.ports[1];
    assert_eq!(dynamic_port.host_port, None);
    assert_eq!(dynamic_port.container_port, 6380);
    assert_eq!(dynamic_port.protocol, Protocol::Tcp);

    // Check UDP port mapping
    let udp_port = &config.ports[2];
    assert_eq!(udp_port.host_port, Some(5353));
    assert_eq!(udp_port.container_port, 53);
    assert_eq!(udp_port.protocol, Protocol::Udp);

    // Volume mounts
    assert_eq!(config.volumes.len(), 4);

    // Labels
    assert_eq!(config.labels.len(), 2);
    assert_eq!(config.labels.get("app"), Some(&"redis".to_string()));
    assert_eq!(config.labels.get("version"), Some(&"7.2".to_string()));

    // Resource limits
    assert_eq!(config.resource_limits.memory, Some(536_870_912)); // 512MB
    assert_eq!(config.resource_limits.cpu_shares, Some(2048)); // 2.0 * 1024

    // Security settings
    assert_eq!(config.user, Some("redis".to_string()));
    assert!(config.privileged);
    assert_eq!(config.capabilities.len(), 2);
    assert!(config.capabilities.contains(&"NET_ADMIN".to_string()));
    assert!(config.capabilities.contains(&"SYS_TIME".to_string()));

    // Runtime options
    assert!(config.auto_remove);
    assert!(config.interactive);
    assert!(config.tty);
    assert!(!config.detached); // Should be false when interactive is true
}

#[test]
fn test_container_builder_command_string() {
    let config = ContainerBuilder::new("alpine")
        .command_str("echo hello world")
        .build();

    assert_eq!(
        config.command,
        Some(vec![
            "echo".to_string(),
            "hello".to_string(),
            "world".to_string()
        ])
    );
}

#[test]
fn test_container_builder_multiple_envs() {
    let mut env_map = HashMap::new();
    env_map.insert("VAR1".to_string(), "value1".to_string());
    env_map.insert("VAR2".to_string(), "value2".to_string());

    let config = ContainerBuilder::new("alpine")
        .env("EXISTING", "value")
        .envs(env_map)
        .build();

    assert_eq!(config.environment.len(), 3);
    assert_eq!(
        config.environment.get("EXISTING"),
        Some(&"value".to_string())
    );
    assert_eq!(config.environment.get("VAR1"), Some(&"value1".to_string()));
    assert_eq!(config.environment.get("VAR2"), Some(&"value2".to_string()));
}

#[test]
fn test_container_builder_multiple_labels() {
    let mut label_map = HashMap::new();
    label_map.insert("label1".to_string(), "value1".to_string());
    label_map.insert("label2".to_string(), "value2".to_string());

    let config = ContainerBuilder::new("alpine")
        .label("existing", "value")
        .labels(label_map)
        .build();

    assert_eq!(config.labels.len(), 3);
    assert_eq!(config.labels.get("existing"), Some(&"value".to_string()));
    assert_eq!(config.labels.get("label1"), Some(&"value1".to_string()));
    assert_eq!(config.labels.get("label2"), Some(&"value2".to_string()));
}

#[test]
fn test_memory_string_parsing() {
    // Test various memory formats
    let test_cases = vec![
        ("100", 100),
        ("100b", 100),
        ("1k", 1_024),
        ("1kb", 1_024),
        ("1m", 1_048_576),
        ("1mb", 1_048_576),
        ("1g", 1_073_741_824),
        ("1gb", 1_073_741_824),
        ("512m", 536_870_912),
        ("2.5g", 2_684_354_560),
        ("1.5k", 1_536),
    ];

    for (input, expected) in test_cases {
        let config = ContainerBuilder::new("test").memory_str(input).build();

        assert_eq!(
            config.resource_limits.memory,
            Some(expected),
            "Failed to parse memory string: {}",
            input
        );
    }
}

#[test]
fn test_invalid_memory_string() {
    // These should not panic but might not set the memory limit
    let config = ContainerBuilder::new("test")
        .memory_str("invalid")
        .memory_str("512x")
        .memory_str("")
        .build();

    // The invalid strings should be ignored, so memory should be None
    assert_eq!(config.resource_limits.memory, None);
}

#[test]
fn test_restart_policy_configuration() {
    let config1 = ContainerBuilder::new("test")
        .restart_policy(RestartPolicy::Always)
        .build();
    assert!(matches!(config1.restart_policy, RestartPolicy::Always));

    let config2 = ContainerBuilder::new("test")
        .restart_policy(RestartPolicy::OnFailure {
            max_retries: Some(3),
        })
        .build();
    assert!(matches!(
        config2.restart_policy,
        RestartPolicy::OnFailure {
            max_retries: Some(3)
        }
    ));

    let config3 = ContainerBuilder::new("test")
        .restart_policy(RestartPolicy::UnlessStopped)
        .build();
    assert!(matches!(
        config3.restart_policy,
        RestartPolicy::UnlessStopped
    ));
}

#[test]
fn test_health_check_configuration() {
    let health_check = TypeHealthCheck {
        test: vec![
            "CMD".to_string(),
            "curl".to_string(),
            "-f".to_string(),
            "http://localhost/health".to_string(),
        ],
        interval: Duration::from_secs(30),
        timeout: Duration::from_secs(30),
        retries: 3,
        start_period: None,
    };

    let config = ContainerBuilder::new("test")
        .health_check(health_check.clone())
        .build();

    assert!(config.health_check.is_some());
}

#[test]
fn test_network_attachment() {
    let network_id = NetworkId::new("my-network".to_string()).unwrap();

    let config = ContainerBuilder::new("test")
        .network(network_id.clone())
        .network_with_aliases(
            NetworkId::new("other-network".to_string()).unwrap(),
            vec!["alias1".to_string(), "alias2".to_string()],
        )
        .build();

    assert_eq!(config.networks.len(), 2);
    assert_eq!(config.networks[0].network, network_id);
    assert!(config.networks[0].aliases.is_empty());
    assert_eq!(config.networks[1].aliases.len(), 2);
}

#[test]
fn test_exec_config() {
    let config = ExecConfig::new(vec![
        "bash".to_string(),
        "-c".to_string(),
        "echo test".to_string(),
    ])
    .working_dir("/tmp")
    .env("TEST_VAR", "test_value")
    .user("root")
    .tty()
    .privileged()
    .interactive();

    assert_eq!(config.command, vec!["bash", "-c", "echo test"]);
    assert_eq!(config.working_dir, Some(PathBuf::from("/tmp")));
    assert_eq!(
        config.environment.get("TEST_VAR"),
        Some(&"test_value".to_string())
    );
    assert_eq!(config.user, Some("root".to_string()));
    assert!(config.tty);
    assert!(config.privileged);
    assert!(config.interactive);
    assert!(config.attach_stdin); // Should be true when interactive is set
}

#[test]
fn test_exec_config_from_command_str() {
    let config = ExecConfig::from_command_str("ls -la /tmp")
        .env("PATH", "/usr/bin")
        .user("nobody");

    assert_eq!(config.command, vec!["ls", "-la", "/tmp"]);
    assert_eq!(
        config.environment.get("PATH"),
        Some(&"/usr/bin".to_string())
    );
    assert_eq!(config.user, Some("nobody".to_string()));
}

#[test]
fn test_exec_result() {
    let result = ExecResult {
        exit_code: 0,
        stdout: "Hello World".to_string(),
        stderr: "Warning message".to_string(),
        duration: Duration::from_millis(150),
    };

    assert!(result.is_success());
    assert_eq!(result.combined_output(), "Hello World\nWarning message");

    let failed_result = ExecResult {
        exit_code: 1,
        stdout: "".to_string(),
        stderr: "Error occurred".to_string(),
        duration: Duration::from_millis(50),
    };

    assert!(!failed_result.is_success());
    assert_eq!(failed_result.combined_output(), "Error occurred");
}

#[test]
fn test_log_options() {
    let options = LogOptions::new()
        .follow()
        .timestamps()
        .tail(100)
        .since(chrono::Utc::now())
        .stdout_only()
        .details();

    assert!(options.follow);
    assert!(options.timestamps);
    assert_eq!(options.tail, Some(100));
    assert!(options.since.is_some());
    assert!(options.stdout);
    assert!(!options.stderr);
    assert!(options.details);
}

#[test]
fn test_log_options_stderr_only() {
    let options = LogOptions::new().stderr_only();

    assert!(!options.stdout);
    assert!(options.stderr);
}

#[test]
fn test_log_entry() {
    let entry = LogEntry {
        message: "Test log message".to_string(),
        timestamp: Some(chrono::Utc::now()),
        source: LogSource::Stdout,
        details: None,
    };

    assert_eq!(entry.message, "Test log message");
    assert_eq!(entry.source, LogSource::Stdout);
    assert!(entry.timestamp.is_some());
}

#[test]
fn test_log_source_display() {
    assert_eq!(LogSource::Stdout.to_string(), "stdout");
    assert_eq!(LogSource::Stderr.to_string(), "stderr");
}

#[test]
fn test_health_check_types() {
    // Port check
    let port_check = HealthCheck::port(8080);
    match port_check {
        HealthCheck::Port { port, host } => {
            assert_eq!(port, 8080);
            assert_eq!(host, None);
        }
        _ => panic!("Expected Port check"),
    }

    // Port check with host
    let host_port_check = HealthCheck::port_on_host(3000, "192.168.1.100".parse().unwrap());
    match host_port_check {
        HealthCheck::Port { port, host } => {
            assert_eq!(port, 3000);
            assert_eq!(host, Some("192.168.1.100".parse().unwrap()));
        }
        _ => panic!("Expected Port check with host"),
    }

    // HTTP check
    let http_check = HealthCheck::http("http://localhost:8080/health");
    match http_check {
        HealthCheck::Http {
            url,
            expected_status,
            ..
        } => {
            assert_eq!(url, "http://localhost:8080/health");
            assert_eq!(expected_status, Some(200));
        }
        _ => panic!("Expected HTTP check"),
    }

    // HTTP check with custom status
    let custom_http_check = HealthCheck::http_with_status("http://localhost:8080/status", 204);
    match custom_http_check {
        HealthCheck::Http {
            url,
            expected_status,
            ..
        } => {
            assert_eq!(url, "http://localhost:8080/status");
            assert_eq!(expected_status, Some(204));
        }
        _ => panic!("Expected HTTP check with custom status"),
    }

    // Command check
    let cmd_check = HealthCheck::command(vec![
        "curl".to_string(),
        "-f".to_string(),
        "localhost".to_string(),
    ]);
    match cmd_check {
        HealthCheck::Command {
            command,
            expected_exit_code,
        } => {
            assert_eq!(command, vec!["curl", "-f", "localhost"]);
            assert_eq!(expected_exit_code, 0);
        }
        _ => panic!("Expected Command check"),
    }

    // Command check with custom exit code
    let custom_cmd_check = HealthCheck::command_with_exit_code(vec!["test".to_string()], 1);
    match custom_cmd_check {
        HealthCheck::Command {
            command,
            expected_exit_code,
        } => {
            assert_eq!(command, vec!["test"]);
            assert_eq!(expected_exit_code, 1);
        }
        _ => panic!("Expected Command check with custom exit code"),
    }

    // Composite checks
    let all_check = HealthCheck::all(vec![
        HealthCheck::port(8080),
        HealthCheck::http("http://localhost:8080/health"),
    ]);
    match all_check {
        HealthCheck::All(checks) => {
            assert_eq!(checks.len(), 2);
        }
        _ => panic!("Expected All check"),
    }

    let any_check = HealthCheck::any(vec![HealthCheck::port(8080), HealthCheck::port(8081)]);
    match any_check {
        HealthCheck::Any(checks) => {
            assert_eq!(checks.len(), 2);
        }
        _ => panic!("Expected Any check"),
    }
}

#[test]
fn test_health_check_config() {
    let config = HealthCheckConfig::new()
        .timeout(Duration::from_secs(60))
        .interval(Duration::from_secs(5))
        .retries(3)
        .start_period(Duration::from_secs(10));

    assert_eq!(config.timeout, Duration::from_secs(60));
    assert_eq!(config.interval, Duration::from_secs(5));
    assert_eq!(config.retries, 3);
    assert_eq!(config.start_period, Duration::from_secs(10));
}

#[test]
fn test_health_check_result() {
    let success = HealthCheckResult::success("All checks passed", Duration::from_millis(100), 2);
    assert!(success.healthy);
    assert_eq!(success.message, "All checks passed");
    assert_eq!(success.duration, Duration::from_millis(100));
    assert_eq!(success.attempts, 2);

    let failure = HealthCheckResult::failure("Port not accessible", Duration::from_millis(200), 5);
    assert!(!failure.healthy);
    assert_eq!(failure.message, "Port not accessible");
    assert_eq!(failure.duration, Duration::from_millis(200));
    assert_eq!(failure.attempts, 5);
}

#[test]
fn test_remove_options() {
    let options = RemoveOptions {
        force: true,
        remove_volumes: true,
    };

    assert!(options.force);
    assert!(options.remove_volumes);

    let default_options = RemoveOptions::default();
    assert!(!default_options.force);
    assert!(!default_options.remove_volumes);
}

#[test]
fn test_network_attachment_struct() {
    // NetworkAttachment struct no longer exists in the current API
    // This test verifies network attachment through ContainerBuilder
    let network_id = NetworkId::new("test-network".to_string()).unwrap();

    let config = ContainerBuilder::new("test")
        .network(network_id.clone())
        .build();

    // Verify network is included in the configuration
    assert!(!config.networks.is_empty());
    assert_eq!(config.networks[0].network, network_id);
}

#[test]
fn test_docker_container_serialization() {
    use serde_json;

    let container = DockerContainer {
        id: ContainerId::new("abcdef123456".to_string()).unwrap(),
        name: Some("test-container".to_string()),
        image: "alpine:latest".to_string(),
        status: ContainerStatus::Running {
            started_at: std::time::SystemTime::now(),
        },
        ports: vec![PortMapping {
            host_ip: None,
            host_port: Some(8080),
            container_port: 80,
            protocol: Protocol::Tcp,
        }],
        labels: {
            let mut labels = HashMap::new();
            labels.insert("app".to_string(), "test".to_string());
            labels
        },
        created: Some(chrono::Utc::now()),
        started: Some(chrono::Utc::now()),
        networks: vec!["bridge".to_string()],
    };

    // Test serialization
    let json = serde_json::to_string(&container).expect("Should serialize");
    assert!(json.contains("test-container"));
    assert!(json.contains("alpine:latest"));

    // Test deserialization
    let deserialized: DockerContainer = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(deserialized.name, Some("test-container".to_string()));
    assert_eq!(deserialized.image, "alpine:latest");
}

#[test]
fn test_container_id_validation() {
    // Valid container IDs
    assert!(ContainerId::new("abcdef1234567890").is_ok());
    assert!(ContainerId::new("1234567890abcdef1234567890abcdef12345678").is_ok()); // 40 chars
    assert!(
        ContainerId::new("1234567890abcdef1234567890abcdef123456781234567890abcdef12345678")
            .is_ok()
    ); // 64 chars

    // Invalid container IDs
    assert!(ContainerId::new("").is_err()); // Empty
    assert!(ContainerId::new("abc").is_err()); // Too short
    assert!(ContainerId::new("a".repeat(100)).is_err()); // Too long
    assert!(ContainerId::new("invalid-chars!@#").is_err()); // Invalid characters
    assert!(ContainerId::new("ABCDEF123456").is_err()); // Uppercase not allowed
}

#[test]
fn test_container_id_display() {
    let id = ContainerId::new("abcdef1234567890").unwrap();
    assert_eq!(id.to_string(), "abcdef1234567890");
    assert_eq!(id.short(), "abcdef123456");
    assert_eq!(id.as_str(), "abcdef1234567890");
}

#[test]
fn test_network_id_validation() {
    assert!(NetworkId::new("bridge").is_ok());
    assert!(NetworkId::new("my-custom-network").is_ok());
    assert!(NetworkId::new("").is_err()); // Empty should fail
}

#[test]
fn test_multiple_capabilities() {
    let config = ContainerBuilder::new("test")
        .capabilities(vec![
            "NET_ADMIN".to_string(),
            "SYS_TIME".to_string(),
            "SETUID".to_string(),
        ])
        .capability("SETGID")
        .build();

    assert_eq!(config.capabilities.len(), 4);
    assert!(config.capabilities.contains(&"NET_ADMIN".to_string()));
    assert!(config.capabilities.contains(&"SYS_TIME".to_string()));
    assert!(config.capabilities.contains(&"SETUID".to_string()));
    assert!(config.capabilities.contains(&"SETGID".to_string()));
}

#[test]
fn test_builder_method_chaining() {
    // Test that all builder methods return Self for chaining
    let _config = ContainerBuilder::new("test")
        .name("test")
        .command(vec!["echo".to_string()])
        .command_str("echo hello")
        .entrypoint(vec!["sh".to_string()])
        .working_dir("/tmp")
        .env("KEY", "value")
        .envs(HashMap::new())
        .port(8080, 80)
        .port_dynamic(8081)
        .port_udp(5353, 53)
        .volume("/host", "/container")
        .volume_ro("/ro", "/ro")
        .volume_named("vol", "/vol")
        .volume_tmp("/tmp")
        .label("key", "value")
        .labels(HashMap::new())
        .restart_policy(RestartPolicy::Always)
        .memory_str("512m")
        .cpus(1.0)
        .network(NetworkId::new("bridge".to_string()).unwrap())
        .user("root")
        .privileged()
        .capability("NET_ADMIN")
        .capabilities(vec!["SYS_TIME".to_string()])
        .auto_remove()
        .interactive()
        .tty()
        .build();
}
