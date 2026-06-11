//! Common utilities for Redis templates

use crate::template::{HealthCheck, VolumeMount};

/// Default Redis port
#[allow(dead_code)]
pub const DEFAULT_REDIS_PORT: u16 = 6379;

/// Default Redis image
pub const DEFAULT_REDIS_IMAGE: &str = "redis";

/// Default Redis Alpine image tag
pub const DEFAULT_REDIS_TAG: &str = "7-alpine";

/// Redis Stack image (full image, includes RedisInsight)
pub const REDIS_STACK_IMAGE: &str = "redis/redis-stack";

/// Redis Stack server image (server only, no RedisInsight)
pub const REDIS_STACK_SERVER_IMAGE: &str = "redis/redis-stack-server";

/// Default Redis Stack image tag.
///
/// Pinned to a known-good release rather than `latest` so that runs are
/// reproducible -- the image under test does not silently change when
/// upstream publishes a new `latest`. Override with `stack_version()` or
/// `custom_image()` to use a different tag.
pub const REDIS_STACK_TAG: &str = "7.4.0-v3";

/// RedisInsight image (as used by the standalone insight template)
pub const REDIS_INSIGHT_IMAGE: &str = "redis/redisinsight";

/// RedisInsight image as used by the cluster template.
///
/// The cluster template historically pulls RedisInsight from the
/// `redislabs/` organization, which is kept here for backwards
/// compatibility. New standalone usage should prefer [`REDIS_INSIGHT_IMAGE`].
pub const REDIS_INSIGHT_CLUSTER_IMAGE: &str = "redislabs/redisinsight";

/// Default RedisInsight image tag.
///
/// Pinned to a known-good release rather than `latest` for reproducibility.
/// Override with `redis_insight_version()` or `custom_image()` to use a
/// different tag.
pub const REDIS_INSIGHT_TAG: &str = "2.60";

/// Default RedisInsight port
pub const DEFAULT_REDIS_INSIGHT_PORT: u16 = 5540;

/// Container path where TLS certificate material is bind-mounted.
///
/// When TLS is enabled the host certificate directory is mounted here
/// read-only and the `--tls-*-file` arguments reference files inside it.
pub const REDIS_TLS_DIR: &str = "/tls";

/// Server certificate file name expected inside the mounted TLS directory.
pub const REDIS_TLS_CERT_FILE: &str = "redis.crt";

/// Private key file name expected inside the mounted TLS directory.
pub const REDIS_TLS_KEY_FILE: &str = "redis.key";

/// CA certificate file name expected inside the mounted TLS directory.
pub const REDIS_TLS_CA_FILE: &str = "ca.crt";

/// Default container-side port Redis listens on for TLS connections.
pub const DEFAULT_REDIS_TLS_PORT: u16 = 6380;

/// Create a default health check for Redis
pub fn default_redis_health_check() -> HealthCheck {
    HealthCheck {
        test: vec!["redis-cli".to_string(), "ping".to_string()],
        interval: "10s".to_string(),
        timeout: "5s".to_string(),
        retries: 3,
        start_period: "10s".to_string(),
    }
}

/// Build a Redis connection string
#[allow(dead_code)]
#[allow(clippy::uninlined_format_args)]
pub fn redis_connection_string(host: &str, port: u16, password: Option<&str>) -> String {
    match password {
        Some(pass) => format!("redis://:{}@{}:{}", pass, host, port),
        None => format!("redis://{}:{}", host, port),
    }
}

/// Build a Redis TLS connection string (`rediss://` scheme).
#[allow(dead_code)]
pub fn redis_tls_connection_string(host: &str, port: u16, password: Option<&str>) -> String {
    match password {
        Some(pass) => format!("rediss://:{pass}@{host}:{port}"),
        None => format!("rediss://{host}:{port}"),
    }
}

/// Build the `redis-server` TLS arguments for the given container-side TLS port.
///
/// Emits `--tls-port`, `--tls-cert-file`, `--tls-key-file` and
/// `--tls-ca-cert-file`, each referencing a file inside [`REDIS_TLS_DIR`]. The
/// caller is responsible for bind-mounting the host certificate directory there
/// (see [`redis_tls_volume`]) and for deciding whether the plaintext port stays
/// open -- passing `--port 0` separately disables plaintext.
pub fn redis_tls_server_args(tls_container_port: u16) -> Vec<String> {
    vec![
        "--tls-port".to_string(),
        tls_container_port.to_string(),
        "--tls-cert-file".to_string(),
        format!("{REDIS_TLS_DIR}/{REDIS_TLS_CERT_FILE}"),
        "--tls-key-file".to_string(),
        format!("{REDIS_TLS_DIR}/{REDIS_TLS_KEY_FILE}"),
        "--tls-ca-cert-file".to_string(),
        format!("{REDIS_TLS_DIR}/{REDIS_TLS_CA_FILE}"),
    ]
}

/// Create a read-only volume mount for a Redis TLS certificate directory.
pub fn redis_tls_volume(certs_dir: impl Into<String>) -> VolumeMount {
    VolumeMount {
        source: certs_dir.into(),
        target: REDIS_TLS_DIR.to_string(),
        read_only: true,
    }
}

/// Create a volume mount for Redis data persistence
pub fn redis_data_volume(volume_name: impl Into<String>) -> VolumeMount {
    VolumeMount {
        source: volume_name.into(),
        target: "/data".to_string(),
        read_only: false,
    }
}

/// Create a volume mount for Redis configuration
pub fn redis_config_volume(config_path: impl Into<String>) -> VolumeMount {
    VolumeMount {
        source: config_path.into(),
        target: "/usr/local/etc/redis/redis.conf".to_string(),
        read_only: true,
    }
}
