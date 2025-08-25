//! RedisInsight template for Redis web UI

#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::unnecessary_get_then_check)]

use super::common::{DEFAULT_REDIS_INSIGHT_PORT, REDIS_INSIGHT_IMAGE, REDIS_INSIGHT_TAG};
use crate::template::{Template, TemplateConfig};
use async_trait::async_trait;
use std::collections::HashMap;

/// RedisInsight web UI template
pub struct RedisInsightTemplate {
    config: TemplateConfig,
}

impl RedisInsightTemplate {
    /// Create a new RedisInsight template with default settings
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let env = HashMap::new();

        let config = TemplateConfig {
            name: name.clone(),
            image: REDIS_INSIGHT_IMAGE.to_string(),
            tag: REDIS_INSIGHT_TAG.to_string(),
            ports: vec![(DEFAULT_REDIS_INSIGHT_PORT, 5540)],
            env,
            volumes: Vec::new(),
            network: None,
            health_check: None, // RedisInsight doesn't need a health check for our purposes
            auto_remove: false,
            memory_limit: None,
            cpu_limit: None,
            platform: None,
        };

        Self { config }
    }

    /// Set a custom port for RedisInsight
    pub fn port(mut self, port: u16) -> Self {
        self.config.ports = vec![(port, 5540)];
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

    /// Set memory limit for RedisInsight
    pub fn memory_limit(mut self, limit: impl Into<String>) -> Self {
        self.config.memory_limit = Some(limit.into());
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
impl Template for RedisInsightTemplate {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn config(&self) -> &TemplateConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut TemplateConfig {
        &mut self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redisinsight_template_basic() {
        let template = RedisInsightTemplate::new("test-insight");
        assert_eq!(template.name(), "test-insight");
        assert_eq!(template.config().image, REDIS_INSIGHT_IMAGE);
        assert_eq!(template.config().tag, REDIS_INSIGHT_TAG);
        assert_eq!(
            template.config().ports,
            vec![(DEFAULT_REDIS_INSIGHT_PORT, 5540)]
        );
    }

    #[test]
    fn test_redisinsight_template_custom_port() {
        let template = RedisInsightTemplate::new("test-insight").port(8080);
        assert_eq!(template.config().ports, vec![(8080, 5540)]);
    }

    #[test]
    fn test_redisinsight_template_with_network() {
        let template = RedisInsightTemplate::new("test-insight").network("redis-network");
        assert_eq!(template.config().network, Some("redis-network".to_string()));
    }

    #[test]
    fn test_redisinsight_template_auto_remove() {
        let template = RedisInsightTemplate::new("test-insight").auto_remove();
        assert!(template.config().auto_remove);
    }
}
