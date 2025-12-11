//! Redis Enterprise template for production-grade Redis deployments
//!
//! This template provides a complete Redis Enterprise setup with cluster
//! initialization, making it easy to spin up a fully configured Redis
//! Enterprise cluster for development and testing.

#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::needless_borrows_for_generic_args)]

use crate::{DockerCommand, RmCommand, RunCommand, StopCommand};
use std::time::Duration;

#[cfg(feature = "template-redis-enterprise")]
use reqwest::Client;
#[cfg(feature = "template-redis-enterprise")]
use serde_json::Value;

/// Redis Enterprise template for production-grade deployments
pub struct RedisEnterpriseTemplate {
    name: String,
    cluster_name: String,
    admin_username: String,
    admin_password: String,
    accept_eula: bool,
    license_file: Option<String>,
    ui_port: u16,
    api_port: u16,
    database_port_start: u16,
    persistent_path: Option<String>,
    ephemeral_path: Option<String>,
    memory_limit: Option<String>,
    initial_database: Option<String>,
    image: String,
    tag: String,
    platform: Option<String>,
    bootstrap_timeout: Duration,
    bootstrap_retries: u32,
    api_ready_timeout: Duration,
}

impl RedisEnterpriseTemplate {
    /// Create a new Redis Enterprise template
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            cluster_name: "Development Cluster".to_string(),
            admin_username: "admin@redis.local".to_string(),
            admin_password: "Redis123!".to_string(),
            accept_eula: false,
            license_file: None,
            ui_port: 8443,
            api_port: 9443,
            database_port_start: 12000,
            persistent_path: None,
            ephemeral_path: None,
            memory_limit: None,
            initial_database: None,
            image: "redislabs/redis".to_string(),
            tag: "latest".to_string(),
            platform: None,
            bootstrap_timeout: Duration::from_secs(60),
            bootstrap_retries: 3,
            api_ready_timeout: Duration::from_secs(30),
        }
    }

    /// Set the cluster name
    pub fn cluster_name(mut self, name: impl Into<String>) -> Self {
        self.cluster_name = name.into();
        self
    }

    /// Set the admin username (email format required)
    pub fn admin_username(mut self, username: impl Into<String>) -> Self {
        self.admin_username = username.into();
        self
    }

    /// Set the admin password (must be strong)
    pub fn admin_password(mut self, password: impl Into<String>) -> Self {
        self.admin_password = password.into();
        self
    }

    /// Accept the End User License Agreement
    pub fn accept_eula(mut self) -> Self {
        self.accept_eula = true;
        self
    }

    /// Set a license file path
    pub fn license_file(mut self, path: impl Into<String>) -> Self {
        self.license_file = Some(path.into());
        self
    }

    /// Set the UI port (default: 8443)
    pub fn ui_port(mut self, port: u16) -> Self {
        self.ui_port = port;
        self
    }

    /// Set the API port (default: 9443)
    pub fn api_port(mut self, port: u16) -> Self {
        self.api_port = port;
        self
    }

    /// Set the starting port for database endpoints (default: 12000)
    pub fn database_port_start(mut self, port: u16) -> Self {
        self.database_port_start = port;
        self
    }

    /// Set custom persistent storage path
    pub fn persistent_path(mut self, path: impl Into<String>) -> Self {
        self.persistent_path = Some(path.into());
        self
    }

    /// Set custom ephemeral storage path
    pub fn ephemeral_path(mut self, path: impl Into<String>) -> Self {
        self.ephemeral_path = Some(path.into());
        self
    }

    /// Set memory limit for the container
    pub fn memory_limit(mut self, limit: impl Into<String>) -> Self {
        self.memory_limit = Some(limit.into());
        self
    }

    /// Create an initial database after cluster setup
    pub fn with_database(mut self, name: impl Into<String>) -> Self {
        self.initial_database = Some(name.into());
        self
    }

    /// Use a custom Redis Enterprise image and tag
    ///
    /// # Examples
    /// ```
    /// # use docker_wrapper::RedisEnterpriseTemplate;
    /// let template = RedisEnterpriseTemplate::new("my-redis")
    ///     .custom_image("my-registry/redis-enterprise", "latest")
    ///     .platform("linux/arm64")
    ///     .accept_eula();
    /// ```
    pub fn custom_image(mut self, image: impl Into<String>, tag: impl Into<String>) -> Self {
        self.image = image.into();
        self.tag = tag.into();
        self
    }

    /// Set the platform for the container (e.g., "linux/arm64", "linux/amd64")
    ///
    /// This is especially useful for ARM-based Redis Enterprise images
    /// on Apple Silicon Macs or ARM servers.
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Set the bootstrap timeout (default: 60 seconds)
    pub fn bootstrap_timeout(mut self, timeout: Duration) -> Self {
        self.bootstrap_timeout = timeout;
        self
    }

    /// Set the number of bootstrap retries (default: 3)
    pub fn bootstrap_retries(mut self, retries: u32) -> Self {
        self.bootstrap_retries = retries;
        self
    }

    /// Set the API ready timeout (default: 30 seconds)
    pub fn api_ready_timeout(mut self, timeout: Duration) -> Self {
        self.api_ready_timeout = timeout;
        self
    }

    /// Start the Redis Enterprise container and initialize the cluster
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - EULA is not accepted
    /// - Container fails to start
    /// - Cluster initialization fails
    /// - API is not accessible
    pub async fn start(self) -> Result<RedisEnterpriseConnectionInfo, crate::Error> {
        // Validate EULA acceptance
        if !self.accept_eula {
            return Err(crate::Error::Custom {
                message: "EULA must be accepted to start Redis Enterprise. Call .accept_eula() on the template".to_string(),
            });
        }

        // Validate password strength (basic check)
        if self.admin_password.len() < 8 {
            return Err(crate::Error::Custom {
                message: "Admin password must be at least 8 characters".to_string(),
            });
        }

        // Start the Redis Enterprise container
        let container_name = format!("{}-enterprise", self.name);
        let mut cmd = RunCommand::new(format!("{}:{}", self.image, self.tag))
            .name(&container_name)
            .port(self.ui_port, 8443)
            .port(self.api_port, 9443)
            .detach();

        // Add database ports range
        for i in 0..10 {
            let port = self.database_port_start + i;
            cmd = cmd.port(port, port);
        }

        // Add volumes for persistence
        let persistent = self
            .persistent_path
            .clone()
            .unwrap_or_else(|| format!("{}-persistent", self.name));
        let ephemeral = self
            .ephemeral_path
            .clone()
            .unwrap_or_else(|| format!("{}-ephemeral", self.name));

        cmd = cmd
            .volume(&persistent, "/var/opt/redislabs/persist")
            .volume(&ephemeral, "/var/opt/redislabs/tmp");

        // Add memory limit if specified
        if let Some(ref limit) = self.memory_limit {
            cmd = cmd.memory(limit);
        }

        // Set capabilities for Redis Enterprise
        cmd = cmd.cap_add("SYS_RESOURCE");

        // Add platform if specified
        if let Some(ref platform) = self.platform {
            cmd = cmd.platform(platform);
        }

        // Execute container start
        cmd.execute().await.map_err(|e| crate::Error::Custom {
            message: format!("Failed to start Redis Enterprise container: {e}"),
        })?;

        // Wait for API to be ready with health checks
        self.wait_for_api_ready(&container_name).await?;

        // Bootstrap the cluster with retries
        self.bootstrap_cluster(&container_name).await?;

        // Verify cluster is ready
        self.verify_cluster_ready(&container_name).await?;

        // Create initial database if requested
        if let Some(ref db_name) = self.initial_database {
            self.create_database(&container_name, db_name).await?;
        }

        Ok(RedisEnterpriseConnectionInfo {
            name: self.name.clone(),
            container_name,
            cluster_name: self.cluster_name.clone(),
            ui_url: format!("https://localhost:{}", self.ui_port),
            api_url: format!("https://localhost:{}", self.api_port),
            username: self.admin_username.clone(),
            password: self.admin_password.clone(),
            database_port: if self.initial_database.is_some() {
                Some(self.database_port_start)
            } else {
                None
            },
        })
    }

    /// Wait for the API to be ready using reqwest
    async fn wait_for_api_ready(&self, _container_name: &str) -> Result<(), crate::Error> {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| crate::Error::Custom {
                message: format!("Failed to build HTTP client: {e}"),
            })?;

        let start = std::time::Instant::now();
        let url = format!("https://localhost:{}/", self.api_port);

        while start.elapsed() < self.api_ready_timeout {
            if let Ok(response) = client.get(&url).send().await {
                let status = response.status();
                // API is ready when it returns any response
                if let Ok(text) = response.text().await {
                    if text.contains("no_cluster")
                        || text.contains("error_code")
                        || status.is_success()
                    {
                        // Give it a bit more time to fully initialize
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        return Ok(());
                    }
                }
            } else {
                // API might not be fully ready yet, continue waiting
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        Err(crate::Error::Custom {
            message: format!(
                "API did not become ready within {} seconds",
                self.api_ready_timeout.as_secs()
            ),
        })
    }

    /// Bootstrap the cluster with retries using reqwest
    async fn bootstrap_cluster(&self, _container_name: &str) -> Result<(), crate::Error> {
        // Build HTTP client that accepts self-signed certificates
        let client = Client::builder()
            .danger_accept_invalid_certs(true) // Redis Enterprise uses self-signed certs
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| crate::Error::Custom {
                message: format!("Failed to build HTTP client: {e}"),
            })?;

        // Parse the bootstrap JSON into a Value for proper serialization
        let bootstrap_json_str = self.build_bootstrap_json();
        let bootstrap_json: Value =
            serde_json::from_str(&bootstrap_json_str).map_err(|e| crate::Error::Custom {
                message: format!("Invalid bootstrap JSON: {e}"),
            })?;

        let url = format!(
            "https://localhost:{}/v1/bootstrap/create_cluster",
            self.api_port
        );

        for attempt in 1..=self.bootstrap_retries {
            let response = client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&bootstrap_json)
                .send()
                .await;

            match response {
                Ok(res) => {
                    let status = res.status();

                    // Check for success
                    if status.is_success() || status.as_u16() == 409 {
                        // 200-299 = success, 409 = already bootstrapped
                        return Ok(());
                    }

                    // Check for validation errors (don't retry these)
                    if status.as_u16() == 400 {
                        let error_body = res.text().await.unwrap_or_default();

                        if error_body.contains("invalid_schema") {
                            return Err(crate::Error::Custom {
                                message: format!("Bootstrap validation failed: {error_body}"),
                            });
                        }

                        return Err(crate::Error::Custom {
                            message: format!("Bootstrap failed with bad request: {error_body}"),
                        });
                    }

                    // Other error status codes - may retry
                    if attempt == self.bootstrap_retries {
                        let error_body = res.text().await.unwrap_or_default();
                        return Err(crate::Error::Custom {
                            message: format!("Bootstrap failed with status {status}: {error_body}"),
                        });
                    }

                    // Log non-fatal errors for debugging
                    if let Ok(error_text) = res.text().await {
                        eprintln!(
                            "Bootstrap attempt {attempt} failed with status {status}: {error_text}"
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Bootstrap attempt {attempt} failed with network error: {e}");

                    if attempt == self.bootstrap_retries {
                        return Err(crate::Error::Custom {
                            message: format!(
                                "Failed to connect to cluster after {} attempts: {}",
                                self.bootstrap_retries, e
                            ),
                        });
                    }
                }
            }

            // Exponential backoff between retries
            if attempt < self.bootstrap_retries {
                let wait_time = Duration::from_secs(5 * u64::from(attempt));
                tokio::time::sleep(wait_time).await;
            }
        }

        Err(crate::Error::Custom {
            message: format!(
                "Failed to bootstrap cluster after {} attempts",
                self.bootstrap_retries
            ),
        })
    }

    /// Verify the cluster is ready using reqwest
    async fn verify_cluster_ready(&self, _container_name: &str) -> Result<(), crate::Error> {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| crate::Error::Custom {
                message: format!("Failed to build HTTP client: {e}"),
            })?;

        let url = format!("https://localhost:{}/v1/cluster", self.api_port);
        let start = std::time::Instant::now();

        while start.elapsed() < Duration::from_secs(10) {
            if let Ok(response) = client
                .get(&url)
                .basic_auth(&self.admin_username, Some(&self.admin_password))
                .send()
                .await
            {
                if response.status().is_success() {
                    if let Ok(text) = response.text().await {
                        if text.contains(&format!(r#""name":"{}""#, self.cluster_name)) {
                            return Ok(());
                        }
                    }
                }
            } else {
                // API might not be ready yet, continue waiting
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        Err(crate::Error::Custom {
            message: "Cluster verification failed - cluster may not be fully initialized"
                .to_string(),
        })
    }

    /// Build the bootstrap JSON payload
    fn build_bootstrap_json(&self) -> String {
        let mut json = format!(
            r#"{{
                "action": "create_cluster",
                "cluster": {{
                    "name": "{}"
                }},
                "node": {{
                    "paths": {{
                        "persistent_path": "/var/opt/redislabs/persist",
                        "ephemeral_path": "/var/opt/redislabs/tmp"
                    }}
                }},
                "credentials": {{
                    "username": "{}",
                    "password": "{}"
                }}"#,
            self.cluster_name, self.admin_username, self.admin_password
        );

        // Add license if provided
        if let Some(ref _license) = self.license_file {
            // In a real implementation, we would read the license file
            // For now, we'll skip this
            json.push_str("");
        }

        json.push('}');
        json
    }

    /// Create a database in the cluster using reqwest
    async fn create_database(
        &self,
        _container_name: &str,
        db_name: &str,
    ) -> Result<(), crate::Error> {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| crate::Error::Custom {
                message: format!("Failed to build HTTP client: {e}"),
            })?;

        let create_db_json = serde_json::json!({
            "name": db_name,
            "port": self.database_port_start,
            "memory_size": 104_857_600
        });

        let url = format!("https://localhost:{}/v1/bdbs", self.api_port);
        let response = client
            .post(&url)
            .basic_auth(&self.admin_username, Some(&self.admin_password))
            .header("Content-Type", "application/json")
            .json(&create_db_json)
            .send()
            .await
            .map_err(|e| crate::Error::Custom {
                message: format!("Failed to send database creation request: {e}"),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(crate::Error::Custom {
                message: format!("Failed to create database with status {status}: {error_body}"),
            });
        }

        Ok(())
    }
}

/// Connection information for Redis Enterprise
pub struct RedisEnterpriseConnectionInfo {
    /// Name of the deployment
    pub name: String,
    /// Container name
    pub container_name: String,
    /// Cluster name
    pub cluster_name: String,
    /// UI URL for web interface
    pub ui_url: String,
    /// API URL for REST API
    pub api_url: String,
    /// Admin username
    pub username: String,
    /// Admin password
    pub password: String,
    /// Database port if initial database was created
    pub database_port: Option<u16>,
}

impl RedisEnterpriseConnectionInfo {
    /// Get the management UI URL
    pub fn ui_url(&self) -> &str {
        &self.ui_url
    }

    /// Get the REST API URL
    pub fn api_url(&self) -> &str {
        &self.api_url
    }

    /// Get Redis connection URL if a database was created
    pub fn redis_url(&self) -> Option<String> {
        self.database_port
            .map(|port| format!("redis://localhost:{port}"))
    }

    /// Stop and clean up the Redis Enterprise cluster
    ///
    /// # Errors
    ///
    /// Returns an error if container cleanup fails
    pub async fn stop(self) -> Result<(), crate::Error> {
        // Stop the container
        StopCommand::new(&self.container_name)
            .execute()
            .await
            .map_err(|e| crate::Error::Custom {
                message: format!("Failed to stop container: {e}"),
            })?;

        // Remove the container
        RmCommand::new(&self.container_name)
            .force()
            .volumes()
            .execute()
            .await
            .map_err(|e| crate::Error::Custom {
                message: format!("Failed to remove container: {e}"),
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_enterprise_template_defaults() {
        let template = RedisEnterpriseTemplate::new("test-enterprise");
        assert_eq!(template.name, "test-enterprise");
        assert_eq!(template.cluster_name, "Development Cluster");
        assert_eq!(template.ui_port, 8443);
        assert_eq!(template.api_port, 9443);
        assert!(!template.accept_eula);
    }

    #[test]
    fn test_redis_enterprise_template_builder() {
        let template = RedisEnterpriseTemplate::new("test-enterprise")
            .cluster_name("Production Cluster")
            .admin_username("admin@company.com")
            .admin_password("SuperSecure123!")
            .accept_eula()
            .ui_port(18443)
            .api_port(19443)
            .with_database("cache-db");

        assert_eq!(template.cluster_name, "Production Cluster");
        assert_eq!(template.admin_username, "admin@company.com");
        assert_eq!(template.admin_password, "SuperSecure123!");
        assert!(template.accept_eula);
        assert_eq!(template.ui_port, 18443);
        assert_eq!(template.api_port, 19443);
        assert_eq!(template.initial_database, Some("cache-db".to_string()));
    }

    #[test]
    fn test_bootstrap_json_generation() {
        let template = RedisEnterpriseTemplate::new("test")
            .cluster_name("Test Cluster")
            .admin_username("test@redis.local")
            .admin_password("TestPass123!");

        let json = template.build_bootstrap_json();

        assert!(json.contains(r#""name": "Test Cluster""#));
        assert!(json.contains(r#""username": "test@redis.local""#));
        assert!(json.contains(r#""password": "TestPass123!""#));
        assert!(json.contains(r#""action": "create_cluster""#));
    }
}
