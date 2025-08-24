//! Nginx template for quick web server container setup

#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]

use crate::template::{HealthCheck, Template, TemplateConfig, VolumeMount};
use async_trait::async_trait;
use std::collections::HashMap;

/// Nginx container template with sensible defaults
pub struct NginxTemplate {
    config: TemplateConfig,
}

impl NginxTemplate {
    /// Create a new Nginx template with default settings
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let env = HashMap::new();

        let config = TemplateConfig {
            name: name.clone(),
            image: "nginx".to_string(),
            tag: "alpine".to_string(),
            ports: vec![(80, 80)],
            env,
            volumes: Vec::new(),
            network: None,
            health_check: Some(HealthCheck {
                test: vec![
                    "wget".to_string(),
                    "--no-verbose".to_string(),
                    "--tries=1".to_string(),
                    "--spider".to_string(),
                    "http://localhost/".to_string(),
                ],
                interval: "30s".to_string(),
                timeout: "10s".to_string(),
                retries: 3,
                start_period: "10s".to_string(),
            }),
            auto_remove: false,
            memory_limit: None,
            cpu_limit: None,
        };

        Self { config }
    }

    /// Set HTTP port
    pub fn port(mut self, port: u16) -> Self {
        // Update or add HTTP port
        if let Some(pos) = self.config.ports.iter().position(|(_, c)| *c == 80) {
            self.config.ports[pos] = (port, 80);
        } else {
            self.config.ports.push((port, 80));
        }
        self
    }

    /// Set HTTPS port
    pub fn https_port(mut self, port: u16) -> Self {
        // Add HTTPS port mapping
        self.config.ports.push((port, 443));
        self
    }

    /// Mount content directory
    pub fn content(mut self, content_path: impl Into<String>) -> Self {
        self.config.volumes.push(VolumeMount {
            source: content_path.into(),
            target: "/usr/share/nginx/html".to_string(),
            read_only: true,
        });
        self
    }

    /// Mount custom nginx configuration
    pub fn config_file(mut self, config_path: impl Into<String>) -> Self {
        self.config.volumes.push(VolumeMount {
            source: config_path.into(),
            target: "/etc/nginx/nginx.conf".to_string(),
            read_only: true,
        });
        self
    }

    /// Mount sites configuration directory
    pub fn sites_config(mut self, sites_path: impl Into<String>) -> Self {
        self.config.volumes.push(VolumeMount {
            source: sites_path.into(),
            target: "/etc/nginx/conf.d".to_string(),
            read_only: true,
        });
        self
    }

    /// Mount SSL certificates directory
    pub fn ssl_certs(mut self, certs_path: impl Into<String>) -> Self {
        self.config.volumes.push(VolumeMount {
            source: certs_path.into(),
            target: "/etc/nginx/ssl".to_string(),
            read_only: true,
        });
        self
    }

    /// Mount logs directory
    pub fn logs(mut self, logs_path: impl Into<String>) -> Self {
        self.config.volumes.push(VolumeMount {
            source: logs_path.into(),
            target: "/var/log/nginx".to_string(),
            read_only: false,
        });
        self
    }

    /// Set memory limit for Nginx
    pub fn memory_limit(mut self, limit: impl Into<String>) -> Self {
        self.config.memory_limit = Some(limit.into());
        self
    }

    /// Set worker processes count
    pub fn worker_processes(mut self, count: impl Into<String>) -> Self {
        self.config
            .env
            .insert("NGINX_WORKER_PROCESSES".to_string(), count.into());
        self
    }

    /// Set worker connections count
    pub fn worker_connections(mut self, count: impl Into<String>) -> Self {
        self.config
            .env
            .insert("NGINX_WORKER_CONNECTIONS".to_string(), count.into());
        self
    }

    /// Use a specific Nginx version
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

    /// Configure as reverse proxy
    pub fn as_reverse_proxy(mut self) -> Self {
        // Adjust health check for reverse proxy
        if let Some(health) = &mut self.config.health_check {
            health.test = vec!["nginx".to_string(), "-t".to_string()];
        }
        self
    }

    /// Enable debug mode
    pub fn debug(mut self) -> Self {
        self.config
            .env
            .insert("NGINX_DEBUG".to_string(), "true".to_string());
        self
    }
}

#[async_trait]
impl Template for NginxTemplate {
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
    fn test_nginx_template_basic() {
        let template = NginxTemplate::new("test-nginx");
        assert_eq!(template.name(), "test-nginx");
        assert_eq!(template.config().image, "nginx");
        assert_eq!(template.config().tag, "alpine");
        assert_eq!(template.config().ports, vec![(80, 80)]);
    }

    #[test]
    fn test_nginx_template_with_ports() {
        let template = NginxTemplate::new("test-nginx").port(8080).https_port(8443);

        assert_eq!(template.config().ports.len(), 2);
        assert!(template.config().ports.contains(&(8080, 80)));
        assert!(template.config().ports.contains(&(8443, 443)));
    }

    #[test]
    fn test_nginx_template_with_content() {
        let template = NginxTemplate::new("test-nginx")
            .content("./website")
            .config_file("./nginx.conf");

        assert_eq!(template.config().volumes.len(), 2);
        assert_eq!(template.config().volumes[0].source, "./website");
        assert_eq!(template.config().volumes[0].target, "/usr/share/nginx/html");
        assert!(template.config().volumes[0].read_only);

        assert_eq!(template.config().volumes[1].source, "./nginx.conf");
        assert_eq!(template.config().volumes[1].target, "/etc/nginx/nginx.conf");
        assert!(template.config().volumes[1].read_only);
    }

    #[test]
    fn test_nginx_template_reverse_proxy() {
        let template = NginxTemplate::new("test-nginx").as_reverse_proxy();

        if let Some(health) = &template.config().health_check {
            assert_eq!(health.test, vec!["nginx".to_string(), "-t".to_string()]);
        }
    }

    #[test]
    fn test_nginx_template_with_ssl() {
        let template = NginxTemplate::new("test-nginx")
            .port(80)
            .https_port(443)
            .ssl_certs("./certs");

        assert!(template.config().ports.contains(&(80, 80)));
        assert!(template.config().ports.contains(&(443, 443)));

        let ssl_volume = template
            .config()
            .volumes
            .iter()
            .find(|v| v.target == "/etc/nginx/ssl")
            .expect("SSL volume should be present");
        assert_eq!(ssl_volume.source, "./certs");
    }
}
