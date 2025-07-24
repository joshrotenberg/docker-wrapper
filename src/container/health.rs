//! Container health checking module for monitoring container readiness and health.
//!
//! This module provides functionality to check if containers are ready for use,
//! including port availability, HTTP endpoint checks, and custom health checks.

use crate::client::DockerClient;
use crate::errors::{DockerError, DockerResult};
use crate::types::ContainerId;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};
use tokio::time::{sleep, timeout};
use tracing::{debug, info};

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Maximum time to wait for health check to pass
    pub timeout: Duration,
    /// Interval between health check attempts
    pub interval: Duration,
    /// Number of consecutive successes required
    pub retries: u32,
    /// Initial delay before starting health checks
    pub start_period: Duration,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            interval: Duration::from_millis(500),
            retries: 3,
            start_period: Duration::from_secs(0),
        }
    }
}

impl HealthCheckConfig {
    /// Create a new health check configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the overall timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the interval between attempts
    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Set the number of required consecutive successes
    pub fn retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }

    /// Set the initial delay before starting checks
    pub fn start_period(mut self, start_period: Duration) -> Self {
        self.start_period = start_period;
        self
    }
}

/// Result of a health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Whether the health check passed
    pub healthy: bool,
    /// Status message
    pub message: String,
    /// Time taken for the check
    pub duration: Duration,
    /// Number of attempts made
    pub attempts: u32,
}

impl HealthCheckResult {
    /// Create a successful health check result
    pub fn success(message: impl Into<String>, duration: Duration, attempts: u32) -> Self {
        Self {
            healthy: true,
            message: message.into(),
            duration,
            attempts,
        }
    }

    /// Create a failed health check result
    pub fn failure(message: impl Into<String>, duration: Duration, attempts: u32) -> Self {
        Self {
            healthy: false,
            message: message.into(),
            duration,
            attempts,
        }
    }
}

/// Types of health checks that can be performed
#[derive(Debug, Clone)]
pub enum HealthCheck {
    /// Check if a port is accepting connections
    Port {
        /// Port number to check
        port: u16,
        /// Host to connect to (defaults to localhost)
        host: Option<IpAddr>,
    },
    /// Check if an HTTP endpoint returns a successful response
    Http {
        /// URL to check
        url: String,
        /// Expected HTTP status code (defaults to 200)
        expected_status: Option<u16>,
        /// Request timeout
        request_timeout: Duration,
    },
    /// Execute a custom command in the container
    Command {
        /// Command to execute
        command: Vec<String>,
        /// Expected exit code (defaults to 0)
        expected_exit_code: i32,
    },
    /// Composite check that requires all sub-checks to pass
    All(Vec<HealthCheck>),
    /// Composite check that requires any sub-check to pass
    Any(Vec<HealthCheck>),
}

impl HealthCheck {
    /// Create a port health check
    pub fn port(port: u16) -> Self {
        Self::Port { port, host: None }
    }

    /// Create a port health check with specific host
    pub fn port_on_host(port: u16, host: IpAddr) -> Self {
        Self::Port {
            port,
            host: Some(host),
        }
    }

    /// Create an HTTP health check
    pub fn http(url: impl Into<String>) -> Self {
        Self::Http {
            url: url.into(),
            expected_status: Some(200),
            request_timeout: Duration::from_secs(5),
        }
    }

    /// Create an HTTP health check with custom status code
    pub fn http_with_status(url: impl Into<String>, expected_status: u16) -> Self {
        Self::Http {
            url: url.into(),
            expected_status: Some(expected_status),
            request_timeout: Duration::from_secs(5),
        }
    }

    /// Create a command health check
    pub fn command(command: Vec<String>) -> Self {
        Self::Command {
            command,
            expected_exit_code: 0,
        }
    }

    /// Create a command health check with custom exit code
    pub fn command_with_exit_code(command: Vec<String>, expected_exit_code: i32) -> Self {
        Self::Command {
            command,
            expected_exit_code,
        }
    }

    /// Create a composite check requiring all to pass
    pub fn all(checks: Vec<HealthCheck>) -> Self {
        Self::All(checks)
    }

    /// Create a composite check requiring any to pass
    pub fn any(checks: Vec<HealthCheck>) -> Self {
        Self::Any(checks)
    }
}

/// Container health checker
pub struct HealthChecker<'a> {
    client: &'a DockerClient,
}

impl<'a> HealthChecker<'a> {
    /// Create a new health checker
    pub fn new(client: &'a DockerClient) -> Self {
        Self { client }
    }

    /// Wait for a container to be healthy
    pub async fn wait_for_healthy(
        &self,
        container_id: &ContainerId,
        health_check: HealthCheck,
        config: HealthCheckConfig,
    ) -> DockerResult<HealthCheckResult> {
        debug!("Starting health check for container: {}", container_id);

        let start_time = Instant::now();

        // Initial delay
        if config.start_period > Duration::from_secs(0) {
            debug!(
                "Waiting {} seconds before starting health checks",
                config.start_period.as_secs()
            );
            sleep(config.start_period).await;
        }

        let mut attempts = 0;
        let mut consecutive_successes = 0;

        loop {
            // Check for overall timeout
            if start_time.elapsed() > config.timeout {
                return Ok(HealthCheckResult::failure(
                    format!(
                        "Health check timed out after {} seconds",
                        config.timeout.as_secs()
                    ),
                    start_time.elapsed(),
                    attempts,
                ));
            }

            attempts += 1;
            debug!(
                "Health check attempt {} for container {}",
                attempts, container_id
            );

            // Perform the health check
            match self.perform_health_check(container_id, &health_check).await {
                Ok(()) => {
                    consecutive_successes += 1;
                    debug!(
                        "Health check passed ({}/{})",
                        consecutive_successes, config.retries
                    );

                    if consecutive_successes >= config.retries {
                        return Ok(HealthCheckResult::success(
                            "Container is healthy".to_string(),
                            start_time.elapsed(),
                            attempts,
                        ));
                    }
                }
                Err(e) => {
                    consecutive_successes = 0;
                    debug!("Health check failed: {}", e);
                }
            }

            // Wait before next attempt
            sleep(config.interval).await;
        }
    }

    /// Perform a single health check
    pub async fn check_health(
        &self,
        container_id: &ContainerId,
        health_check: HealthCheck,
    ) -> DockerResult<HealthCheckResult> {
        let start_time = Instant::now();

        match self.perform_health_check(container_id, &health_check).await {
            Ok(()) => Ok(HealthCheckResult::success(
                "Health check passed".to_string(),
                start_time.elapsed(),
                1,
            )),
            Err(e) => Ok(HealthCheckResult::failure(
                e.to_string(),
                start_time.elapsed(),
                1,
            )),
        }
    }

    /// Check if a port is available on the container
    pub async fn check_port(
        &self,
        container_id: &ContainerId,
        container_port: u16,
        timeout_duration: Duration,
    ) -> DockerResult<bool> {
        debug!(
            "Checking port {} for container {}",
            container_port, container_id
        );

        // Get the mapped host port
        let container_manager = crate::container::ContainerManager::new(self.client);
        let host_port = match container_manager.port(container_id, container_port).await? {
            Some(port) => port,
            None => {
                return Err(DockerError::not_found(format!(
                    "Port {} is not mapped for container {}",
                    container_port, container_id
                )));
            }
        };

        // Check if the port accepts connections
        self.check_tcp_port("127.0.0.1".parse().unwrap(), host_port, timeout_duration)
            .await
    }

    /// Check if a TCP port accepts connections
    pub async fn check_tcp_port(
        &self,
        host: IpAddr,
        port: u16,
        timeout_duration: Duration,
    ) -> DockerResult<bool> {
        let addr = SocketAddr::new(host, port);

        match timeout(timeout_duration, tokio::net::TcpStream::connect(addr)).await {
            Ok(Ok(_)) => {
                debug!("Port {}:{} is accepting connections", host, port);
                Ok(true)
            }
            Ok(Err(e)) => {
                debug!("Failed to connect to {}:{}: {}", host, port, e);
                Ok(false)
            }
            Err(_) => {
                debug!("Connection to {}:{} timed out", host, port);
                Ok(false)
            }
        }
    }

    /// Check if an HTTP endpoint returns a successful response
    pub async fn check_http_endpoint(
        &self,
        url: &str,
        expected_status: Option<u16>,
        request_timeout: Duration,
    ) -> DockerResult<bool> {
        debug!("Checking HTTP endpoint: {}", url);

        // Create a simple HTTP client
        let client = reqwest::Client::builder()
            .timeout(request_timeout)
            .build()
            .map_err(|e| DockerError::network(format!("Failed to create HTTP client: {}", e)))?;

        match client.get(url).send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let expected = expected_status.unwrap_or(200);

                if status == expected {
                    debug!("HTTP endpoint {} returned expected status {}", url, status);
                    Ok(true)
                } else {
                    debug!(
                        "HTTP endpoint {} returned status {}, expected {}",
                        url, status, expected
                    );
                    Ok(false)
                }
            }
            Err(e) => {
                debug!("HTTP request to {} failed: {}", url, e);
                Ok(false)
            }
        }
    }

    /// Execute a command in the container for health checking
    pub async fn check_command(
        &self,
        container_id: &ContainerId,
        command: &[String],
        expected_exit_code: i32,
    ) -> DockerResult<bool> {
        debug!(
            "Executing health check command in container {}: {:?}",
            container_id, command
        );

        let executor = crate::container::ContainerExecutor::new(self.client);
        let config = crate::container::ExecConfig::new(command.to_vec());

        match executor.exec(container_id, config).await {
            Ok(result) => {
                if result.exit_code == expected_exit_code {
                    debug!(
                        "Health check command succeeded with exit code {}",
                        result.exit_code
                    );
                    Ok(true)
                } else {
                    debug!(
                        "Health check command failed with exit code {}, expected {}",
                        result.exit_code, expected_exit_code
                    );
                    Ok(false)
                }
            }
            Err(e) => {
                debug!("Failed to execute health check command: {}", e);
                Ok(false)
            }
        }
    }

    /// Internal method to perform the actual health check
    async fn perform_health_check(
        &self,
        container_id: &ContainerId,
        health_check: &HealthCheck,
    ) -> DockerResult<()> {
        // Flatten composite checks to avoid recursion
        let flattened_checks = self.flatten_health_check(health_check);

        match health_check {
            HealthCheck::All(_) => {
                // All checks must pass
                for check in &flattened_checks {
                    self.perform_single_health_check(container_id, check)
                        .await?;
                }
                Ok(())
            }
            HealthCheck::Any(_) => {
                // At least one check must pass
                let mut last_error = None;

                for check in &flattened_checks {
                    match self.perform_single_health_check(container_id, check).await {
                        Ok(()) => return Ok(()),
                        Err(e) => last_error = Some(e),
                    }
                }

                Err(last_error.unwrap_or_else(|| {
                    DockerError::health_check("All health checks failed".to_string())
                }))
            }
            _ => {
                // Single check
                self.perform_single_health_check(container_id, health_check)
                    .await
            }
        }
    }

    /// Flatten composite health checks into a list of single checks
    fn flatten_health_check(&self, health_check: &HealthCheck) -> Vec<HealthCheck> {
        match health_check {
            HealthCheck::All(checks) | HealthCheck::Any(checks) => {
                let mut flattened = Vec::new();
                for check in checks {
                    flattened.extend(self.flatten_health_check(check));
                }
                flattened
            }
            check => vec![check.clone()],
        }
    }

    /// Perform a single (non-composite) health check
    async fn perform_single_health_check(
        &self,
        container_id: &ContainerId,
        health_check: &HealthCheck,
    ) -> DockerResult<()> {
        match health_check {
            HealthCheck::Port { port, host } => {
                let host_addr = host.unwrap_or_else(|| IpAddr::V4(Ipv4Addr::LOCALHOST));

                // If checking localhost, try to get the mapped port
                let port_to_check = if host_addr.is_loopback() {
                    let container_manager = crate::container::ContainerManager::new(self.client);
                    container_manager
                        .port(container_id, *port)
                        .await?
                        .unwrap_or(*port)
                } else {
                    *port
                };

                let is_healthy = self
                    .check_tcp_port(host_addr, port_to_check, Duration::from_secs(5))
                    .await?;

                if is_healthy {
                    Ok(())
                } else {
                    Err(DockerError::health_check(format!(
                        "Port {}:{} is not accepting connections",
                        host_addr, port_to_check
                    )))
                }
            }
            HealthCheck::Http {
                url,
                expected_status,
                request_timeout,
            } => {
                let is_healthy = self
                    .check_http_endpoint(url, *expected_status, *request_timeout)
                    .await?;

                if is_healthy {
                    Ok(())
                } else {
                    Err(DockerError::health_check(format!(
                        "HTTP endpoint {} is not healthy",
                        url
                    )))
                }
            }
            HealthCheck::Command {
                command,
                expected_exit_code,
            } => {
                let is_healthy = self
                    .check_command(container_id, command, *expected_exit_code)
                    .await?;

                if is_healthy {
                    Ok(())
                } else {
                    Err(DockerError::health_check(format!(
                        "Health check command failed: {:?}",
                        command
                    )))
                }
            }
            HealthCheck::All(_) | HealthCheck::Any(_) => {
                // This should not happen as we flatten composite checks
                Err(DockerError::health_check(
                    "Unexpected composite health check in single check handler".to_string(),
                ))
            }
        }
    }
}

/// Convenience functions for common health checks
impl HealthChecker<'_> {
    /// Wait for a container port to be ready
    pub async fn wait_for_port(
        &self,
        container_id: &ContainerId,
        port: u16,
        timeout: Duration,
    ) -> DockerResult<()> {
        let health_check = HealthCheck::port(port);
        let config = HealthCheckConfig::new().timeout(timeout);

        let result = self
            .wait_for_healthy(container_id, health_check, config)
            .await?;

        if result.healthy {
            info!("Container {} port {} is ready", container_id, port);
            Ok(())
        } else {
            Err(DockerError::health_check(result.message))
        }
    }

    /// Wait for an HTTP endpoint to be ready
    pub async fn wait_for_http(
        &self,
        url: impl Into<String>,
        timeout: Duration,
    ) -> DockerResult<()> {
        let url = url.into();
        let health_check = HealthCheck::http(url.clone());
        let config = HealthCheckConfig::new().timeout(timeout);

        // Use a dummy container ID for HTTP checks
        let dummy_id = ContainerId::new("http-check".to_string())?;

        let result = self
            .wait_for_healthy(&dummy_id, health_check, config)
            .await?;

        if result.healthy {
            info!("HTTP endpoint {} is ready", url);
            Ok(())
        } else {
            Err(DockerError::health_check(result.message))
        }
    }

    /// Wait for multiple ports to be ready
    pub async fn wait_for_ports(
        &self,
        container_id: &ContainerId,
        ports: &[u16],
        timeout: Duration,
    ) -> DockerResult<()> {
        let checks: Vec<HealthCheck> = ports.iter().map(|&port| HealthCheck::port(port)).collect();
        let health_check = HealthCheck::all(checks);
        let config = HealthCheckConfig::new().timeout(timeout);

        let result = self
            .wait_for_healthy(container_id, health_check, config)
            .await?;

        if result.healthy {
            info!("Container {} ports {:?} are ready", container_id, ports);
            Ok(())
        } else {
            Err(DockerError::health_check(result.message))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_config() {
        let config = HealthCheckConfig::new()
            .timeout(Duration::from_secs(60))
            .interval(Duration::from_secs(1))
            .retries(5)
            .start_period(Duration::from_secs(10));

        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.interval, Duration::from_secs(1));
        assert_eq!(config.retries, 5);
        assert_eq!(config.start_period, Duration::from_secs(10));
    }

    #[test]
    fn test_health_check_result() {
        let success = HealthCheckResult::success("All good", Duration::from_millis(100), 3);
        assert!(success.healthy);
        assert_eq!(success.message, "All good");
        assert_eq!(success.attempts, 3);

        let failure = HealthCheckResult::failure("Failed", Duration::from_millis(200), 5);
        assert!(!failure.healthy);
        assert_eq!(failure.message, "Failed");
        assert_eq!(failure.attempts, 5);
    }

    #[test]
    fn test_health_check_types() {
        let port_check = HealthCheck::port(8080);
        let http_check = HealthCheck::http("http://localhost:8080/health");
        let command_check = HealthCheck::command(vec![
            "curl".to_string(),
            "-f".to_string(),
            "localhost:8080".to_string(),
        ]);

        let all_check = HealthCheck::all(vec![port_check.clone(), http_check.clone()]);
        let any_check = HealthCheck::any(vec![port_check, command_check]);

        match all_check {
            HealthCheck::All(checks) => assert_eq!(checks.len(), 2),
            _ => panic!("Expected All check"),
        }

        match any_check {
            HealthCheck::Any(checks) => assert_eq!(checks.len(), 2),
            _ => panic!("Expected Any check"),
        }
    }
}
