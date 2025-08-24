//! PostgreSQL template for quick PostgreSQL container setup

#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::format_push_string)]
#![allow(clippy::uninlined_format_args)]

use super::{HealthCheck, Template, TemplateConfig, VolumeMount};
use async_trait::async_trait;
use std::collections::HashMap;

/// PostgreSQL container template with sensible defaults
pub struct PostgresTemplate {
    config: TemplateConfig,
}

impl PostgresTemplate {
    /// Create a new PostgreSQL template with default settings
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let mut env = HashMap::new();

        // Default PostgreSQL configuration
        env.insert("POSTGRES_PASSWORD".to_string(), "postgres".to_string());
        env.insert("POSTGRES_USER".to_string(), "postgres".to_string());
        env.insert("POSTGRES_DB".to_string(), "postgres".to_string());

        let config = TemplateConfig {
            name: name.clone(),
            image: "postgres".to_string(),
            tag: "15-alpine".to_string(),
            ports: vec![(5432, 5432)],
            env,
            volumes: Vec::new(),
            network: None,
            health_check: Some(HealthCheck {
                test: vec![
                    "pg_isready".to_string(),
                    "-U".to_string(),
                    "postgres".to_string(),
                ],
                interval: "10s".to_string(),
                timeout: "5s".to_string(),
                retries: 5,
                start_period: "10s".to_string(),
            }),
            auto_remove: false,
            memory_limit: None,
            cpu_limit: None,
        };

        Self { config }
    }

    /// Set a custom PostgreSQL port
    pub fn port(mut self, port: u16) -> Self {
        self.config.ports = vec![(port, 5432)];
        self
    }

    /// Set database name
    pub fn database(mut self, db: impl Into<String>) -> Self {
        self.config.env.insert("POSTGRES_DB".to_string(), db.into());
        self
    }

    /// Set database user
    pub fn user(mut self, user: impl Into<String>) -> Self {
        let user = user.into();
        self.config
            .env
            .insert("POSTGRES_USER".to_string(), user.clone());

        // Update health check to use the correct user
        if let Some(health) = &mut self.config.health_check {
            health.test = vec!["pg_isready".to_string(), "-U".to_string(), user];
        }
        self
    }

    /// Set database password
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.config
            .env
            .insert("POSTGRES_PASSWORD".to_string(), password.into());
        self
    }

    /// Enable persistence with a volume
    pub fn with_persistence(mut self, volume_name: impl Into<String>) -> Self {
        self.config.volumes.push(VolumeMount {
            source: volume_name.into(),
            target: "/var/lib/postgresql/data".to_string(),
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

    /// Set memory limit for PostgreSQL
    pub fn memory_limit(mut self, limit: impl Into<String>) -> Self {
        self.config.memory_limit = Some(limit.into());
        self
    }

    /// Set shared memory size
    pub fn shared_memory(mut self, size: impl Into<String>) -> Self {
        self.config
            .env
            .insert("POSTGRES_SHARED_MEMORY".to_string(), size.into());
        self
    }

    /// Enable PostgreSQL extensions
    pub fn with_extension(mut self, extension: impl Into<String>) -> Self {
        let ext = extension.into();
        let current = self
            .config
            .env
            .get("POSTGRES_EXTENSIONS")
            .map(|s| format!("{},{}", s, ext))
            .unwrap_or(ext);
        self.config
            .env
            .insert("POSTGRES_EXTENSIONS".to_string(), current);
        self
    }

    /// Use a specific PostgreSQL version
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

    /// Set additional PostgreSQL configuration
    pub fn postgres_args(mut self, args: impl Into<String>) -> Self {
        self.config
            .env
            .insert("POSTGRES_INITDB_ARGS".to_string(), args.into());
        self
    }

    /// Enable SSL/TLS
    pub fn with_ssl(mut self) -> Self {
        self.config
            .env
            .insert("POSTGRES_SSL_MODE".to_string(), "require".to_string());
        self
    }

    /// Set locale
    pub fn locale(mut self, locale: impl Into<String>) -> Self {
        let locale = locale.into();
        self.config.env.insert(
            "POSTGRES_INITDB_ARGS".to_string(),
            format!("--locale={}", locale),
        );
        self
    }
}

#[async_trait]
impl Template for PostgresTemplate {
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

/// Builder for PostgreSQL connection strings
pub struct PostgresConnectionString {
    host: String,
    port: u16,
    database: String,
    user: String,
    password: String,
}

impl PostgresConnectionString {
    /// Create from a PostgresTemplate
    pub fn from_template(template: &PostgresTemplate) -> Self {
        let config = template.config();
        let port = config.ports.first().map(|(h, _)| *h).unwrap_or(5432);

        Self {
            host: "localhost".to_string(),
            port,
            database: config
                .env
                .get("POSTGRES_DB")
                .cloned()
                .unwrap_or_else(|| "postgres".to_string()),
            user: config
                .env
                .get("POSTGRES_USER")
                .cloned()
                .unwrap_or_else(|| "postgres".to_string()),
            password: config
                .env
                .get("POSTGRES_PASSWORD")
                .cloned()
                .unwrap_or_else(|| "postgres".to_string()),
        }
    }

    /// Get the connection string in PostgreSQL URL format
    pub fn url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database
        )
    }

    /// Get the connection string in key-value format
    pub fn key_value(&self) -> String {
        format!(
            "host={} port={} dbname={} user={} password={}",
            self.host, self.port, self.database, self.user, self.password
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DockerCommand;

    #[test]
    fn test_postgres_template_basic() {
        let template = PostgresTemplate::new("test-postgres");
        assert_eq!(template.name(), "test-postgres");
        assert_eq!(template.config().image, "postgres");
        assert_eq!(template.config().tag, "15-alpine");
        assert_eq!(template.config().ports, vec![(5432, 5432)]);
    }

    #[test]
    fn test_postgres_template_custom_config() {
        let template = PostgresTemplate::new("test-postgres")
            .database("mydb")
            .user("myuser")
            .password("secret123")
            .port(15432);

        assert_eq!(
            template.config().env.get("POSTGRES_DB"),
            Some(&"mydb".to_string())
        );
        assert_eq!(
            template.config().env.get("POSTGRES_USER"),
            Some(&"myuser".to_string())
        );
        assert_eq!(
            template.config().env.get("POSTGRES_PASSWORD"),
            Some(&"secret123".to_string())
        );
        assert_eq!(template.config().ports, vec![(15432, 5432)]);
    }

    #[test]
    fn test_postgres_template_with_persistence() {
        let template = PostgresTemplate::new("test-postgres").with_persistence("postgres-data");

        assert_eq!(template.config().volumes.len(), 1);
        assert_eq!(template.config().volumes[0].source, "postgres-data");
        assert_eq!(
            template.config().volumes[0].target,
            "/var/lib/postgresql/data"
        );
    }

    #[test]
    fn test_postgres_template_with_init_scripts() {
        let template = PostgresTemplate::new("test-postgres").init_scripts("./init-scripts");

        assert_eq!(template.config().volumes.len(), 1);
        assert_eq!(template.config().volumes[0].source, "./init-scripts");
        assert_eq!(
            template.config().volumes[0].target,
            "/docker-entrypoint-initdb.d"
        );
        assert!(template.config().volumes[0].read_only);
    }

    #[test]
    fn test_postgres_connection_string() {
        let template = PostgresTemplate::new("test-postgres")
            .database("testdb")
            .user("testuser")
            .password("testpass")
            .port(15432);

        let conn = PostgresConnectionString::from_template(&template);

        assert_eq!(
            conn.url(),
            "postgresql://testuser:testpass@localhost:15432/testdb"
        );

        assert_eq!(
            conn.key_value(),
            "host=localhost port=15432 dbname=testdb user=testuser password=testpass"
        );
    }

    #[test]
    fn test_postgres_build_command() {
        let template = PostgresTemplate::new("test-postgres")
            .database("mydb")
            .port(15432);

        let cmd = template.build_command();
        let args = cmd.build_command_args();

        // Check that basic args are present
        assert!(args.contains(&"run".to_string()));
        assert!(args.contains(&"--name".to_string()));
        assert!(args.contains(&"test-postgres".to_string()));
        assert!(args.contains(&"--publish".to_string()));
        assert!(args.contains(&"15432:5432".to_string()));
        assert!(args.contains(&"--env".to_string()));
    }
}
