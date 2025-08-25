//! Basic Redis template for quick Redis container setup

#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::unnecessary_get_then_check)]

use super::common::{
    default_redis_health_check, redis_config_volume, redis_data_volume, DEFAULT_REDIS_IMAGE,
    DEFAULT_REDIS_TAG, REDIS_STACK_IMAGE, REDIS_STACK_TAG,
};
use crate::template::{Template, TemplateConfig};
use async_trait::async_trait;
use std::collections::HashMap;

/// Redis container template with sensible defaults
pub struct RedisTemplate {
    config: TemplateConfig,
    use_redis_stack: bool,
}

impl RedisTemplate {
    /// Create a new Redis template with default settings
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let env = HashMap::new();

        // Default Redis configuration
        let config = TemplateConfig {
            name: name.clone(),
            image: DEFAULT_REDIS_IMAGE.to_string(),
            tag: DEFAULT_REDIS_TAG.to_string(),
            ports: vec![(6379, 6379)],
            env,
            volumes: Vec::new(),
            network: None,
            health_check: Some(default_redis_health_check()),
            auto_remove: false,
            memory_limit: None,
            cpu_limit: None,
            platform: None,
        };

        Self {
            config,
            use_redis_stack: false,
        }
    }

    /// Set a custom Redis port
    pub fn port(mut self, port: u16) -> Self {
        self.config.ports = vec![(port, 6379)];
        self
    }

    /// Set Redis password
    pub fn password(mut self, password: impl Into<String>) -> Self {
        // Redis uses command args for password, we'll handle this in build_command
        self.config
            .env
            .insert("REDIS_PASSWORD".to_string(), password.into());
        self
    }

    /// Enable persistence with a volume
    pub fn with_persistence(mut self, volume_name: impl Into<String>) -> Self {
        self.config.volumes.push(redis_data_volume(volume_name));
        self
    }

    /// Set custom Redis configuration file
    pub fn config_file(mut self, config_path: impl Into<String>) -> Self {
        self.config.volumes.push(redis_config_volume(config_path));
        self
    }

    /// Set memory limit for Redis
    pub fn memory_limit(mut self, limit: impl Into<String>) -> Self {
        self.config.memory_limit = Some(limit.into());
        self
    }

    /// Enable Redis cluster mode
    pub fn cluster_mode(mut self) -> Self {
        self.config
            .env
            .insert("REDIS_CLUSTER".to_string(), "yes".to_string());
        self
    }

    /// Set max memory policy
    pub fn maxmemory_policy(mut self, policy: impl Into<String>) -> Self {
        self.config
            .env
            .insert("REDIS_MAXMEMORY_POLICY".to_string(), policy.into());
        self
    }

    /// Use a specific Redis version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.config.tag = format!("{}-alpine", version.into());
        self
    }

    /// Connect to a specific network
    pub fn network(mut self, network: impl Into<String>) -> Self {
        self.config.network = Some(network.into());
        self
    }

    /// Enable auto-remove when stopped
    pub fn auto_remove(mut self) -> Self {
        self.config.auto_remove = true;
        self
    }

    /// Use Redis Stack image instead of basic Redis
    pub fn with_redis_stack(mut self) -> Self {
        self.use_redis_stack = true;
        self
    }

    /// Use a custom image and tag
    pub fn custom_image(mut self, image: impl Into<String>, tag: impl Into<String>) -> Self {
        self.config.image = image.into();
        self.config.tag = tag.into();
        self
    }

    /// Set the platform for the container (e.g., "linux/arm64", "linux/amd64")
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.config.platform = Some(platform.into());
        self
    }
}

#[async_trait]
impl Template for RedisTemplate {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn config(&self) -> &TemplateConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut TemplateConfig {
        &mut self.config
    }

    fn build_command(&self) -> crate::RunCommand {
        let config = self.config();

        // Choose image based on Redis Stack preference
        let image_tag = if self.use_redis_stack {
            format!("{REDIS_STACK_IMAGE}:{REDIS_STACK_TAG}")
        } else {
            format!("{}:{}", config.image, config.tag)
        };

        let mut cmd = crate::RunCommand::new(image_tag)
            .name(&config.name)
            .detach();

        // Add port mappings
        for (host, container) in &config.ports {
            cmd = cmd.port(*host, *container);
        }

        // Add volume mounts
        for mount in &config.volumes {
            if mount.read_only {
                cmd = cmd.volume_ro(&mount.source, &mount.target);
            } else {
                cmd = cmd.volume(&mount.source, &mount.target);
            }
        }

        // Add network
        if let Some(network) = &config.network {
            cmd = cmd.network(network);
        }

        // Add health check
        if let Some(health) = &config.health_check {
            cmd = cmd
                .health_cmd(&health.test.join(" "))
                .health_interval(&health.interval)
                .health_timeout(&health.timeout)
                .health_retries(health.retries)
                .health_start_period(&health.start_period);
        }

        // Add resource limits
        if let Some(memory) = &config.memory_limit {
            cmd = cmd.memory(memory);
        }

        if let Some(cpu) = &config.cpu_limit {
            cmd = cmd.cpus(cpu);
        }

        // Auto-remove
        if config.auto_remove {
            cmd = cmd.remove();
        }

        // Handle Redis-specific command args
        if let Some(password) = config.env.get("REDIS_PASSWORD") {
            if self.use_redis_stack {
                // For Redis Stack, use environment variable instead of command override
                cmd = cmd.env("REDIS_ARGS", format!("--requirepass {password}"));
            } else {
                // For basic Redis, override entrypoint to bypass docker-entrypoint.sh and directly run redis-server
                cmd = cmd.entrypoint("redis-server").cmd(vec![
                    "--requirepass".to_string(),
                    password.clone(),
                    "--protected-mode".to_string(),
                    "yes".to_string(),
                ]);
            }
        }

        // If custom config file is mounted
        let has_config = config
            .volumes
            .iter()
            .any(|v| v.target == "/usr/local/etc/redis/redis.conf");
        if has_config && config.env.get("REDIS_PASSWORD").is_none() {
            cmd = cmd.cmd(vec![
                "redis-server".to_string(),
                "/usr/local/etc/redis/redis.conf".to_string(),
            ]);
        }

        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DockerCommand;

    #[test]
    fn test_redis_template_basic() {
        let template = RedisTemplate::new("test-redis");
        assert_eq!(template.name(), "test-redis");
        assert_eq!(template.config().image, "redis");
        assert_eq!(template.config().tag, "7-alpine");
        assert_eq!(template.config().ports, vec![(6379, 6379)]);
    }

    #[test]
    fn test_redis_template_with_password() {
        let template = RedisTemplate::new("test-redis").password("secret123");

        assert_eq!(
            template.config().env.get("REDIS_PASSWORD"),
            Some(&"secret123".to_string())
        );
    }

    #[test]
    fn test_redis_template_with_persistence() {
        let template = RedisTemplate::new("test-redis").with_persistence("redis-data");

        assert_eq!(template.config().volumes.len(), 1);
        assert_eq!(template.config().volumes[0].source, "redis-data");
        assert_eq!(template.config().volumes[0].target, "/data");
    }

    #[test]
    fn test_redis_template_custom_port() {
        let template = RedisTemplate::new("test-redis").port(16379);

        assert_eq!(template.config().ports, vec![(16379, 6379)]);
    }

    #[test]
    fn test_redis_build_command() {
        let template = RedisTemplate::new("test-redis")
            .password("mypass")
            .port(16379);

        let cmd = template.build_command();
        let args = cmd.build_command_args();

        // Check that basic args are present
        assert!(args.contains(&"run".to_string()));
        assert!(args.contains(&"--name".to_string()));
        assert!(args.contains(&"test-redis".to_string()));
        assert!(args.contains(&"--publish".to_string()));
        assert!(args.contains(&"16379:6379".to_string()));
    }
}
