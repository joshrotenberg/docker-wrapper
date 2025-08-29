//! MongoDB template for quick MongoDB container setup

#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::format_push_string)]
#![allow(clippy::uninlined_format_args)]

use crate::template::{HealthCheck, Template, TemplateConfig, VolumeMount};
use async_trait::async_trait;
use std::collections::HashMap;

/// MongoDB container template with sensible defaults
pub struct MongodbTemplate {
    config: TemplateConfig,
}

impl MongodbTemplate {
    /// Create a new MongoDB template with default settings
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let env = HashMap::new();

        let config = TemplateConfig {
            name: name.clone(),
            image: "mongo".to_string(),
            tag: "7.0".to_string(),
            ports: vec![(27017, 27017)],
            env,
            volumes: Vec::new(),
            network: None,
            health_check: Some(HealthCheck {
                test: vec![
                    "mongosh".to_string(),
                    "--eval".to_string(),
                    "db.adminCommand('ping')".to_string(),
                ],
                interval: "10s".to_string(),
                timeout: "5s".to_string(),
                retries: 5,
                start_period: "20s".to_string(),
            }),
            auto_remove: false,
            memory_limit: None,
            cpu_limit: None,
            platform: None,
        };

        Self { config }
    }

    /// Set a custom MongoDB port
    pub fn port(mut self, port: u16) -> Self {
        self.config.ports = vec![(port, 27017)];
        self
    }

    /// Set root username
    pub fn root_username(mut self, username: impl Into<String>) -> Self {
        self.config
            .env
            .insert("MONGO_INITDB_ROOT_USERNAME".to_string(), username.into());
        self
    }

    /// Set root password
    pub fn root_password(mut self, password: impl Into<String>) -> Self {
        self.config
            .env
            .insert("MONGO_INITDB_ROOT_PASSWORD".to_string(), password.into());
        self
    }

    /// Set initial database name
    pub fn database(mut self, db: impl Into<String>) -> Self {
        self.config
            .env
            .insert("MONGO_INITDB_DATABASE".to_string(), db.into());
        self
    }

    /// Enable persistence with a volume
    pub fn with_persistence(mut self, volume_name: impl Into<String>) -> Self {
        self.config.volumes.push(VolumeMount {
            source: volume_name.into(),
            target: "/data/db".to_string(),
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

    /// Mount custom MongoDB configuration
    pub fn config_file(mut self, config_path: impl Into<String>) -> Self {
        self.config.volumes.push(VolumeMount {
            source: config_path.into(),
            target: "/etc/mongo/mongod.conf".to_string(),
            read_only: true,
        });
        self
    }

    /// Set memory limit for MongoDB
    pub fn memory_limit(mut self, limit: impl Into<String>) -> Self {
        self.config.memory_limit = Some(limit.into());
        self
    }

    /// Set WiredTiger cache size
    pub fn cache_size(mut self, size: impl Into<String>) -> Self {
        self.config
            .env
            .insert("MONGO_WIREDTIGER_CACHE_SIZE_GB".to_string(), size.into());
        self
    }

    /// Enable replica set mode
    pub fn replica_set(mut self, name: impl Into<String>) -> Self {
        self.config
            .env
            .insert("MONGO_REPLICA_SET".to_string(), name.into());
        self
    }

    /// Enable authentication
    pub fn with_auth(mut self) -> Self {
        self.config
            .env
            .insert("MONGO_AUTH".to_string(), "yes".to_string());
        self
    }

    /// Use a specific MongoDB version
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

    /// Set journal commit interval
    pub fn journal_commit_interval(mut self, ms: u32) -> Self {
        self.config
            .env
            .insert("MONGO_JOURNAL_COMMIT_INTERVAL".to_string(), ms.to_string());
        self
    }

    /// Enable quiet logging
    pub fn quiet(mut self) -> Self {
        self.config
            .env
            .insert("MONGO_QUIET".to_string(), "yes".to_string());
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
impl Template for MongodbTemplate {
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

        // Add environment variables (except MONGO_REPLICA_SET which is handled as command arg)
        for (key, value) in &config.env {
            if key != "MONGO_REPLICA_SET" {
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

        // Add MongoDB-specific command for replica set
        if let Some(replica_set) = config.env.get("MONGO_REPLICA_SET") {
            // Start mongod with --replSet parameter
            cmd = cmd.cmd(vec![
                "mongod".to_string(),
                "--replSet".to_string(),
                replica_set.clone(),
                "--bind_ip_all".to_string(),
            ]);
        }

        cmd
    }

    async fn wait_for_ready(&self) -> crate::template::Result<()> {
        use std::time::Duration;
        use tokio::time::{sleep, timeout};

        // Custom MongoDB readiness check
        let wait_timeout = Duration::from_secs(30);
        let check_interval = Duration::from_millis(500);

        timeout(wait_timeout, async {
            loop {
                // First check if container is running
                if !self.is_running().await? {
                    return Err(crate::template::TemplateError::NotRunning(
                        self.config().name.clone(),
                    ));
                }

                // Try to connect to MongoDB using mongosh (or mongo for older versions)
                let check_cmd = if self.config.tag.starts_with("4.") {
                    // Use mongo for version 4.x
                    vec![
                        "mongo",
                        "--host",
                        "localhost",
                        "--eval",
                        "db.runCommand({ ping: 1 })",
                        "--quiet",
                    ]
                } else {
                    // Use mongosh for version 5.0+
                    vec![
                        "mongosh",
                        "--host",
                        "localhost",
                        "--eval",
                        "db.runCommand({ ping: 1 })",
                        "--quiet",
                    ]
                };

                // Execute readiness check
                if let Ok(result) = self.exec(check_cmd).await {
                    // MongoDB ping returns { ok: 1 } on success
                    if result.stdout.contains("ok") && result.stdout.contains('1') {
                        return Ok(());
                    }
                }

                sleep(check_interval).await;
            }
        })
        .await
        .map_err(|_| {
            crate::template::TemplateError::InvalidConfig(format!(
                "MongoDB container {} failed to become ready within timeout",
                self.config().name
            ))
        })?
    }
}

/// Builder for MongoDB connection strings
pub struct MongodbConnectionString {
    host: String,
    port: u16,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
    replica_set: Option<String>,
}

impl MongodbConnectionString {
    /// Create from a MongodbTemplate
    pub fn from_template(template: &MongodbTemplate) -> Self {
        let config = template.config();
        let port = config.ports.first().map(|(h, _)| *h).unwrap_or(27017);

        Self {
            host: "localhost".to_string(),
            port,
            database: config.env.get("MONGO_INITDB_DATABASE").cloned(),
            username: config.env.get("MONGO_INITDB_ROOT_USERNAME").cloned(),
            password: config.env.get("MONGO_INITDB_ROOT_PASSWORD").cloned(),
            replica_set: config.env.get("MONGO_REPLICA_SET").cloned(),
        }
    }

    /// Get the connection string in MongoDB URL format
    pub fn url(&self) -> String {
        let mut url = String::from("mongodb://");

        // Add credentials if present
        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            url.push_str(&format!("{}:{}@", user, pass));
        }

        // Add host and port
        url.push_str(&format!("{}:{}", self.host, self.port));

        // Add database if present
        if let Some(db) = &self.database {
            url.push_str(&format!("/{}", db));
        }

        // Add replica set if present
        if let Some(rs) = &self.replica_set {
            if self.database.is_none() {
                url.push('/');
            }
            url.push_str(&format!("?replicaSet={}", rs));
        }

        url
    }

    /// Get the connection string for MongoDB SRV (Atlas-style)
    pub fn srv_url(&self) -> String {
        let mut url = String::from("mongodb+srv://");

        // Add credentials if present
        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            url.push_str(&format!("{}:{}@", user, pass));
        }

        // For SRV, we only use the host
        url.push_str(&self.host);

        // Add database if present
        if let Some(db) = &self.database {
            url.push_str(&format!("/{}", db));
        }

        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mongodb_template_basic() {
        let template = MongodbTemplate::new("test-mongo");
        assert_eq!(template.name(), "test-mongo");
        assert_eq!(template.config().image, "mongo");
        assert_eq!(template.config().tag, "7.0");
        assert_eq!(template.config().ports, vec![(27017, 27017)]);
    }

    #[test]
    fn test_mongodb_template_with_auth() {
        let template = MongodbTemplate::new("test-mongo")
            .root_username("admin")
            .root_password("secret123")
            .database("mydb")
            .with_auth();

        assert_eq!(
            template.config().env.get("MONGO_INITDB_ROOT_USERNAME"),
            Some(&"admin".to_string())
        );
        assert_eq!(
            template.config().env.get("MONGO_INITDB_ROOT_PASSWORD"),
            Some(&"secret123".to_string())
        );
        assert_eq!(
            template.config().env.get("MONGO_INITDB_DATABASE"),
            Some(&"mydb".to_string())
        );
        assert_eq!(
            template.config().env.get("MONGO_AUTH"),
            Some(&"yes".to_string())
        );
    }

    #[test]
    fn test_mongodb_template_with_persistence() {
        let template = MongodbTemplate::new("test-mongo").with_persistence("mongo-data");

        assert_eq!(template.config().volumes.len(), 1);
        assert_eq!(template.config().volumes[0].source, "mongo-data");
        assert_eq!(template.config().volumes[0].target, "/data/db");
    }

    #[test]
    fn test_mongodb_connection_string() {
        let template = MongodbTemplate::new("test-mongo")
            .root_username("admin")
            .root_password("pass")
            .database("testdb")
            .port(27018);

        let conn = MongodbConnectionString::from_template(&template);

        assert_eq!(conn.url(), "mongodb://admin:pass@localhost:27018/testdb");
    }

    #[test]
    fn test_mongodb_connection_string_no_auth() {
        let template = MongodbTemplate::new("test-mongo");
        let conn = MongodbConnectionString::from_template(&template);

        assert_eq!(conn.url(), "mongodb://localhost:27017");
    }

    #[test]
    fn test_mongodb_connection_string_replica_set() {
        let template = MongodbTemplate::new("test-mongo").replica_set("rs0");

        let conn = MongodbConnectionString::from_template(&template);

        assert_eq!(conn.url(), "mongodb://localhost:27017/?replicaSet=rs0");
    }
}
