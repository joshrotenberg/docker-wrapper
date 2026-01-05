//! MySQL template for quick MySQL container setup

#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::format_push_string)]
#![allow(clippy::uninlined_format_args)]

use crate::template::{HealthCheck, Template, TemplateConfig, VolumeMount};
use async_trait::async_trait;
use std::collections::HashMap;

/// MySQL container template with sensible defaults
pub struct MysqlTemplate {
    config: TemplateConfig,
}

impl MysqlTemplate {
    /// Create a new MySQL template with default settings
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let mut env = HashMap::new();

        // Default MySQL configuration
        env.insert("MYSQL_ROOT_PASSWORD".to_string(), "mysql".to_string());
        env.insert("MYSQL_DATABASE".to_string(), "mysql".to_string());

        let config = TemplateConfig {
            name: name.clone(),
            image: "mysql".to_string(),
            tag: "8.0".to_string(),
            ports: vec![(3306, 3306)],
            env,
            volumes: Vec::new(),
            network: None,
            health_check: Some(HealthCheck {
                test: vec![
                    "mysqladmin".to_string(),
                    "ping".to_string(),
                    "-h".to_string(),
                    "localhost".to_string(),
                ],
                interval: "10s".to_string(),
                timeout: "5s".to_string(),
                retries: 5,
                start_period: "30s".to_string(),
            }),
            auto_remove: false,
            memory_limit: None,
            cpu_limit: None,
            platform: None,
        };

        Self { config }
    }

    /// Set a custom MySQL port
    pub fn port(mut self, port: u16) -> Self {
        self.config.ports = vec![(port, 3306)];
        self
    }

    /// Set root password
    pub fn root_password(mut self, password: impl Into<String>) -> Self {
        self.config
            .env
            .insert("MYSQL_ROOT_PASSWORD".to_string(), password.into());
        self
    }

    /// Set database name
    pub fn database(mut self, db: impl Into<String>) -> Self {
        self.config
            .env
            .insert("MYSQL_DATABASE".to_string(), db.into());
        self
    }

    /// Set database user (non-root)
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.config
            .env
            .insert("MYSQL_USER".to_string(), user.into());
        self
    }

    /// Set database user password
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.config
            .env
            .insert("MYSQL_PASSWORD".to_string(), password.into());
        self
    }

    /// Allow empty password for root (development only!)
    pub fn allow_empty_password(mut self) -> Self {
        self.config.env.remove("MYSQL_ROOT_PASSWORD");
        self.config
            .env
            .insert("MYSQL_ALLOW_EMPTY_PASSWORD".to_string(), "yes".to_string());
        self
    }

    /// Set random root password
    pub fn random_root_password(mut self) -> Self {
        self.config.env.remove("MYSQL_ROOT_PASSWORD");
        self.config
            .env
            .insert("MYSQL_RANDOM_ROOT_PASSWORD".to_string(), "yes".to_string());
        self
    }

    /// Enable persistence with a volume
    pub fn with_persistence(mut self, volume_name: impl Into<String>) -> Self {
        self.config.volumes.push(VolumeMount {
            source: volume_name.into(),
            target: "/var/lib/mysql".to_string(),
            read_only: false,
        });
        self
    }

    /// Mount initialization scripts directory
    pub fn init_scripts(mut self, scripts_path: impl Into<String>) -> Self {
        self.config.volumes.push(VolumeMount {
            source: scripts_path.into(),
            target: "/docker-entrypoint-initdb.d".to_string(),
            read_only: true,
        });
        self
    }

    /// Mount custom MySQL configuration
    pub fn config_file(mut self, config_path: impl Into<String>) -> Self {
        self.config.volumes.push(VolumeMount {
            source: config_path.into(),
            target: "/etc/mysql/conf.d/custom.cnf".to_string(),
            read_only: true,
        });
        self
    }

    /// Set memory limit for MySQL
    pub fn memory_limit(mut self, limit: impl Into<String>) -> Self {
        self.config.memory_limit = Some(limit.into());
        self
    }

    /// Set character set
    pub fn character_set(mut self, charset: impl Into<String>) -> Self {
        let charset = charset.into();
        self.config
            .env
            .insert("MYSQL_CHARSET".to_string(), charset.clone());
        let current_cmd = self
            .config
            .env
            .get("MYSQL_COMMAND")
            .map(|s| format!("{} --character-set-server={}", s, charset))
            .unwrap_or_else(|| format!("--character-set-server={}", charset));
        self.config
            .env
            .insert("MYSQL_COMMAND".to_string(), current_cmd);
        self
    }

    /// Set collation
    pub fn collation(mut self, collation: impl Into<String>) -> Self {
        let collation = collation.into();
        self.config
            .env
            .insert("MYSQL_COLLATION".to_string(), collation.clone());
        let current_cmd = self
            .config
            .env
            .get("MYSQL_COMMAND")
            .map(|s| format!("{} --collation-server={}", s, collation))
            .unwrap_or_else(|| format!("--collation-server={}", collation));
        self.config
            .env
            .insert("MYSQL_COMMAND".to_string(), current_cmd);
        self
    }

    /// Use a specific MySQL version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.config.tag = version.into();
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
impl Template for MysqlTemplate {
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
        let image_tag = format!("{}:{}", config.image, config.tag);

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

        // Add environment variables
        for (key, value) in &config.env {
            // Skip MYSQL_COMMAND as it's not a real env var
            if key != "MYSQL_COMMAND" {
                cmd = cmd.env(key, value);
            }
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

        // Add platform if specified
        if let Some(platform) = &config.platform {
            cmd = cmd.platform(platform);
        }

        // Add MySQL-specific command args for charset and collation
        if let Some(mysql_cmd) = config.env.get("MYSQL_COMMAND") {
            // Parse the command string to get individual arguments
            let args: Vec<String> = mysql_cmd
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            if !args.is_empty() {
                // Override the default command with mysqld and our custom args
                cmd = cmd.cmd(std::iter::once("mysqld".to_string()).chain(args).collect());
            }
        }

        cmd
    }

    async fn wait_for_ready(&self) -> crate::template::Result<()> {
        use std::time::Duration;
        use tokio::time::{sleep, timeout};

        // Custom MySQL readiness check - increased timeout for charset/collation configs
        // MySQL 8.0 can take 90+ seconds to initialize on slower CI systems
        let wait_timeout = Duration::from_secs(120);
        let check_interval = Duration::from_millis(1000);

        timeout(wait_timeout, async {
            let mut consecutive_successes = 0;
            loop {
                // Check if container is running - keep retrying if not yet started
                // Don't fail immediately as the container may still be starting up
                if !self.is_running().await.unwrap_or(false) {
                    consecutive_successes = 0;
                    sleep(check_interval).await;
                    continue;
                }

                // Try to connect to MySQL using the mysql client with an actual query
                // Use 127.0.0.1 instead of localhost to force TCP connection
                // (localhost uses Unix socket which may not be ready even when TCP is)
                let password = self
                    .config
                    .env
                    .get("MYSQL_ROOT_PASSWORD")
                    .or_else(|| self.config.env.get("MYSQL_PASSWORD"))
                    .map(|s| s.as_str())
                    .unwrap_or("mysql");

                let password_arg = format!("-p{}", password);
                let check_cmd = vec![
                    "mysql",
                    "-h",
                    "127.0.0.1",
                    "-u",
                    "root",
                    &password_arg,
                    "-e",
                    "SELECT 1",
                ];

                // Execute readiness check
                if let Ok(result) = self.exec(check_cmd).await {
                    // If we got output containing '1', MySQL responded successfully
                    if result.stdout.contains('1') {
                        consecutive_successes += 1;
                        // Require 2 consecutive successes to ensure stability
                        if consecutive_successes >= 2 {
                            return Ok(());
                        }
                        sleep(Duration::from_millis(500)).await;
                        continue;
                    }
                }

                consecutive_successes = 0;
                sleep(check_interval).await;
            }
        })
        .await
        .map_err(|_| {
            crate::template::TemplateError::InvalidConfig(format!(
                "MySQL container {} failed to become ready within timeout",
                self.config().name
            ))
        })?
    }
}

/// Builder for MySQL connection strings
pub struct MysqlConnectionString {
    host: String,
    port: u16,
    database: String,
    user: String,
    password: String,
}

impl MysqlConnectionString {
    /// Create from a MysqlTemplate
    pub fn from_template(template: &MysqlTemplate) -> Self {
        let config = template.config();
        let port = config.ports.first().map(|(h, _)| *h).unwrap_or(3306);

        // Determine user and password
        let (user, password) = if let Some(user) = config.env.get("MYSQL_USER") {
            let password = config
                .env
                .get("MYSQL_PASSWORD")
                .cloned()
                .unwrap_or_default();
            (user.clone(), password)
        } else {
            let password = config
                .env
                .get("MYSQL_ROOT_PASSWORD")
                .cloned()
                .unwrap_or_else(|| "mysql".to_string());
            ("root".to_string(), password)
        };

        Self {
            host: "localhost".to_string(),
            port,
            database: config
                .env
                .get("MYSQL_DATABASE")
                .cloned()
                .unwrap_or_else(|| "mysql".to_string()),
            user,
            password,
        }
    }

    /// Get the connection string in MySQL URL format
    pub fn url(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database
        )
    }

    /// Get the connection string for JDBC
    pub fn jdbc(&self) -> String {
        format!(
            "jdbc:mysql://{}:{}/{}?user={}&password={}",
            self.host, self.port, self.database, self.user, self.password
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mysql_template_basic() {
        let template = MysqlTemplate::new("test-mysql");
        assert_eq!(template.name(), "test-mysql");
        assert_eq!(template.config().image, "mysql");
        assert_eq!(template.config().tag, "8.0");
        assert_eq!(template.config().ports, vec![(3306, 3306)]);
    }

    #[test]
    fn test_mysql_template_custom_config() {
        let template = MysqlTemplate::new("test-mysql")
            .database("mydb")
            .user("myuser")
            .password("secret123")
            .port(13306);

        assert_eq!(
            template.config().env.get("MYSQL_DATABASE"),
            Some(&"mydb".to_string())
        );
        assert_eq!(
            template.config().env.get("MYSQL_USER"),
            Some(&"myuser".to_string())
        );
        assert_eq!(
            template.config().env.get("MYSQL_PASSWORD"),
            Some(&"secret123".to_string())
        );
        assert_eq!(template.config().ports, vec![(13306, 3306)]);
    }

    #[test]
    fn test_mysql_template_with_persistence() {
        let template = MysqlTemplate::new("test-mysql").with_persistence("mysql-data");

        assert_eq!(template.config().volumes.len(), 1);
        assert_eq!(template.config().volumes[0].source, "mysql-data");
        assert_eq!(template.config().volumes[0].target, "/var/lib/mysql");
    }

    #[test]
    fn test_mysql_connection_string() {
        let template = MysqlTemplate::new("test-mysql")
            .database("testdb")
            .user("testuser")
            .password("testpass")
            .port(13306);

        let conn = MysqlConnectionString::from_template(&template);

        assert_eq!(
            conn.url(),
            "mysql://testuser:testpass@localhost:13306/testdb"
        );

        assert_eq!(
            conn.jdbc(),
            "jdbc:mysql://localhost:13306/testdb?user=testuser&password=testpass"
        );
    }
}
