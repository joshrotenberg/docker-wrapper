//! Common utilities for Redis templates

use crate::template::{HealthCheck, VolumeMount};

/// Default Redis port
#[allow(dead_code)]
pub const DEFAULT_REDIS_PORT: u16 = 6379;

/// Default Redis image
pub const DEFAULT_REDIS_IMAGE: &str = "redis";

/// Default Redis Alpine image tag
pub const DEFAULT_REDIS_TAG: &str = "7-alpine";

/// Redis Stack image
pub const REDIS_STACK_IMAGE: &str = "redis/redis-stack";

/// Redis Stack image tag
pub const REDIS_STACK_TAG: &str = "latest";

/// RedisInsight image
pub const REDIS_INSIGHT_IMAGE: &str = "redis/redisinsight";

/// RedisInsight image tag
pub const REDIS_INSIGHT_TAG: &str = "latest";

/// Default RedisInsight port
pub const DEFAULT_REDIS_INSIGHT_PORT: u16 = 5540;

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
