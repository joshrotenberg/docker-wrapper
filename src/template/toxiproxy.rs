//! Toxiproxy template for network fault injection.
//!
//! [Toxiproxy](https://github.com/Shopify/toxiproxy) is a TCP proxy that injects
//! controllable faults (latency, bandwidth limits, connection drops, ...) between
//! a client and an upstream service. This template runs the
//! `ghcr.io/shopify/toxiproxy` container, publishes its `:8474` control API, and
//! exposes a small typed client over that API so you can register proxies and add
//! toxics without hand-rolling HTTP calls.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use docker_wrapper::template::toxiproxy::{Toxic, ToxiproxyTemplate, ToxicStream};
//! use docker_wrapper::Template;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Start Toxiproxy and wait for its control API to be ready.
//! let toxiproxy = ToxiproxyTemplate::new("chaos")
//!     .control_port(8474)
//!     .proxy_port(16379);
//! toxiproxy.start_and_wait().await?;
//! toxiproxy.wait_for_control_api().await?;
//!
//! // Route a published host port through Toxiproxy to an upstream Redis.
//! // The proxy listens on 0.0.0.0:16379 inside the container, which is published
//! // to the host, so host clients can connect through the proxy.
//! toxiproxy
//!     .create_proxy("redis", "0.0.0.0:16379", "redis:6379")
//!     .await?;
//!
//! // Inject 500ms of downstream latency.
//! toxiproxy
//!     .add_toxic("redis", "slow", ToxicStream::Downstream, Toxic::latency(500))
//!     .await?;
//!
//! // ... run your client against localhost:16379 and observe the fault ...
//!
//! toxiproxy.remove_toxic("redis", "slow").await?;
//! toxiproxy.reset().await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Feature Flag
//!
//! This template requires the `template-toxiproxy` feature:
//!
//! ```toml
//! [dependencies]
//! docker-wrapper = { version = "0.11", features = ["template-toxiproxy"] }
//! ```

#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]

use crate::template::{Result, Template, TemplateConfig, TemplateError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Default Toxiproxy image.
const DEFAULT_IMAGE: &str = "ghcr.io/shopify/toxiproxy";
/// Default Toxiproxy image tag.
const DEFAULT_TAG: &str = "2.12.0";
/// Default control API port.
const DEFAULT_CONTROL_PORT: u16 = 8474;

/// Direction a toxic applies to.
///
/// Toxiproxy applies toxics to one direction of a proxied connection. The
/// downstream is data flowing from the upstream back to the client; the upstream
/// is data flowing from the client to the upstream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToxicStream {
    /// Data flowing from the upstream service back to the client.
    Downstream,
    /// Data flowing from the client to the upstream service.
    Upstream,
}

impl ToxicStream {
    /// The wire value Toxiproxy expects for this stream direction.
    fn as_str(self) -> &'static str {
        match self {
            ToxicStream::Downstream => "downstream",
            ToxicStream::Upstream => "upstream",
        }
    }
}

/// A typed Toxiproxy toxic.
///
/// Each variant maps to a Toxiproxy toxic type and carries its attributes. Use
/// the associated constructors ([`Toxic::latency`], [`Toxic::bandwidth`], ...) to
/// build a toxic, then pass it to [`ToxiproxyTemplate::add_toxic`].
///
/// See the [Toxiproxy toxics reference](https://github.com/Shopify/toxiproxy#toxics)
/// for the full semantics of each type.
#[derive(Debug, Clone, PartialEq)]
pub enum Toxic {
    /// Add a delay to all data, with optional random jitter.
    ///
    /// `latency` and `jitter` are in milliseconds.
    Latency {
        /// Base latency added to each packet, in milliseconds.
        latency: u64,
        /// Random jitter added on top of the base latency, in milliseconds.
        jitter: u64,
    },
    /// Limit the connection to a maximum throughput.
    ///
    /// `rate` is in kilobytes per second.
    Bandwidth {
        /// Throughput limit in kilobytes per second.
        rate: u64,
    },
    /// Stop all data and close the connection after `timeout` milliseconds.
    ///
    /// A `timeout` of `0` holds the connection open indefinitely without passing
    /// data, which is useful for simulating a hung upstream.
    Timeout {
        /// Time to wait before closing the connection, in milliseconds.
        timeout: u64,
    },
    /// Slice data into smaller packets to simulate fragmentation.
    Slicer {
        /// Average size of each packet, in bytes.
        average_size: u64,
        /// Variation in packet size, in bytes.
        size_variation: u64,
        /// Delay between sliced packets, in microseconds.
        delay: u64,
    },
    /// Close the connection after a fixed number of bytes have been transmitted.
    LimitData {
        /// Number of bytes to allow before closing the connection.
        bytes: u64,
    },
}

impl Toxic {
    /// Create a latency toxic with no jitter.
    ///
    /// `latency` is in milliseconds.
    #[must_use]
    pub fn latency(latency: u64) -> Self {
        Toxic::Latency { latency, jitter: 0 }
    }

    /// Create a latency toxic with random jitter.
    ///
    /// Both `latency` and `jitter` are in milliseconds.
    #[must_use]
    pub fn jitter(latency: u64, jitter: u64) -> Self {
        Toxic::Latency { latency, jitter }
    }

    /// Create a bandwidth toxic limiting throughput to `rate` kilobytes per second.
    #[must_use]
    pub fn bandwidth(rate: u64) -> Self {
        Toxic::Bandwidth { rate }
    }

    /// Create a timeout toxic that closes the connection after `timeout` milliseconds.
    ///
    /// A `timeout` of `0` holds the connection open indefinitely.
    #[must_use]
    pub fn timeout(timeout: u64) -> Self {
        Toxic::Timeout { timeout }
    }

    /// Create a slicer toxic that fragments data into smaller packets.
    ///
    /// `average_size` and `size_variation` are in bytes; `delay` is in microseconds.
    #[must_use]
    pub fn slicer(average_size: u64, size_variation: u64, delay: u64) -> Self {
        Toxic::Slicer {
            average_size,
            size_variation,
            delay,
        }
    }

    /// Create a `limit_data` toxic that closes the connection after `bytes` bytes.
    #[must_use]
    pub fn limit_data(bytes: u64) -> Self {
        Toxic::LimitData { bytes }
    }

    /// The Toxiproxy type name for this toxic.
    fn type_name(&self) -> &'static str {
        match self {
            Toxic::Latency { .. } => "latency",
            Toxic::Bandwidth { .. } => "bandwidth",
            Toxic::Timeout { .. } => "timeout",
            Toxic::Slicer { .. } => "slicer",
            Toxic::LimitData { .. } => "limit_data",
        }
    }

    /// The attribute map Toxiproxy expects for this toxic.
    fn attributes(&self) -> HashMap<String, u64> {
        let mut attrs = HashMap::new();
        match *self {
            Toxic::Latency { latency, jitter } => {
                attrs.insert("latency".to_string(), latency);
                attrs.insert("jitter".to_string(), jitter);
            }
            Toxic::Bandwidth { rate } => {
                attrs.insert("rate".to_string(), rate);
            }
            Toxic::Timeout { timeout } => {
                attrs.insert("timeout".to_string(), timeout);
            }
            Toxic::Slicer {
                average_size,
                size_variation,
                delay,
            } => {
                attrs.insert("average_size".to_string(), average_size);
                attrs.insert("size_variation".to_string(), size_variation);
                attrs.insert("delay".to_string(), delay);
            }
            Toxic::LimitData { bytes } => {
                attrs.insert("bytes".to_string(), bytes);
            }
        }
        attrs
    }
}

/// Request body for creating or updating a proxy via the control API.
#[derive(Debug, Serialize)]
struct ProxyRequest {
    name: String,
    listen: String,
    upstream: String,
    enabled: bool,
}

/// A proxy as returned by the Toxiproxy control API.
#[derive(Debug, Clone, Deserialize)]
pub struct ProxyInfo {
    /// Proxy name.
    pub name: String,
    /// Address the proxy listens on (inside the container).
    pub listen: String,
    /// Upstream address the proxy forwards to.
    pub upstream: String,
    /// Whether the proxy is currently enabled.
    pub enabled: bool,
}

/// Request body for adding a toxic via the control API.
#[derive(Debug, Serialize)]
struct ToxicRequest {
    name: String,
    #[serde(rename = "type")]
    toxic_type: String,
    stream: String,
    toxicity: f64,
    attributes: HashMap<String, u64>,
}

/// Toxiproxy template for TCP fault injection.
///
/// Runs the `ghcr.io/shopify/toxiproxy` container and exposes a typed client over
/// its `:8474` control API. Register proxies with [`create_proxy`](Self::create_proxy)
/// and add toxics with [`add_toxic`](Self::add_toxic).
///
/// Proxies should listen on `0.0.0.0:<port>` and that port must be published (via
/// [`proxy_port`](Self::proxy_port)) so host clients can connect through the proxy.
pub struct ToxiproxyTemplate {
    config: TemplateConfig,
    control_port: u16,
    api_ready_timeout: Duration,
}

impl ToxiproxyTemplate {
    /// Create a new Toxiproxy template.
    ///
    /// The control API is published on port `8474` by default. Use
    /// [`proxy_port`](Self::proxy_port) to also publish the ports your proxies
    /// listen on so host clients can reach them.
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let config = TemplateConfig {
            name,
            image: DEFAULT_IMAGE.to_string(),
            tag: DEFAULT_TAG.to_string(),
            ports: vec![(DEFAULT_CONTROL_PORT, DEFAULT_CONTROL_PORT)],
            env: HashMap::new(),
            volumes: Vec::new(),
            network: None,
            health_check: None,
            auto_remove: false,
            memory_limit: None,
            cpu_limit: None,
            platform: None,
        };

        Self {
            config,
            control_port: DEFAULT_CONTROL_PORT,
            api_ready_timeout: Duration::from_secs(30),
        }
    }

    /// Set the host port for the control API (default: 8474).
    ///
    /// This maps the host port to the container's `8474` control port.
    pub fn control_port(mut self, port: u16) -> Self {
        // Replace the existing control-port mapping (container side is always 8474).
        if let Some(pos) = self
            .config
            .ports
            .iter()
            .position(|(_, c)| *c == DEFAULT_CONTROL_PORT)
        {
            self.config.ports[pos] = (port, DEFAULT_CONTROL_PORT);
        } else {
            self.config.ports.push((port, DEFAULT_CONTROL_PORT));
        }
        self.control_port = port;
        self
    }

    /// Publish a proxy port so host clients can connect through a proxy.
    ///
    /// A proxy that listens on `0.0.0.0:<port>` inside the container is only
    /// reachable from the host if that port is published. Call this once per port
    /// you intend to proxy on. The same port is used on both the host and
    /// container sides so the published address matches the proxy's `listen`
    /// address.
    ///
    /// # Example
    ///
    /// ```rust
    /// use docker_wrapper::template::toxiproxy::ToxiproxyTemplate;
    ///
    /// let toxiproxy = ToxiproxyTemplate::new("chaos")
    ///     .proxy_port(16379)
    ///     .proxy_port(15432);
    /// ```
    pub fn proxy_port(mut self, port: u16) -> Self {
        if !self
            .config
            .ports
            .iter()
            .any(|(h, c)| *h == port && *c == port)
        {
            self.config.ports.push((port, port));
        }
        self
    }

    /// Connect to a specific Docker network.
    ///
    /// Use this to place Toxiproxy on the same network as the upstream containers
    /// it proxies, so it can reach them by container name.
    pub fn network(mut self, network: impl Into<String>) -> Self {
        self.config.network = Some(network.into());
        self
    }

    /// Enable auto-remove when the container stops.
    pub fn auto_remove(mut self) -> Self {
        self.config.auto_remove = true;
        self
    }

    /// Use a custom image and tag.
    pub fn custom_image(mut self, image: impl Into<String>, tag: impl Into<String>) -> Self {
        self.config.image = image.into();
        self.config.tag = tag.into();
        self
    }

    /// Set the platform for the container (e.g., "linux/arm64", "linux/amd64").
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.config.platform = Some(platform.into());
        self
    }

    /// Set how long to wait for the control API to become ready (default: 30s).
    pub fn api_ready_timeout(mut self, timeout: Duration) -> Self {
        self.api_ready_timeout = timeout;
        self
    }

    /// The base URL of the control API on the host.
    fn control_url(&self) -> String {
        format!("http://localhost:{}", self.control_port)
    }

    /// Build an HTTP client for talking to the control API.
    fn http_client() -> Result<Client> {
        Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| {
                TemplateError::DockerError(crate::Error::custom(format!(
                    "failed to build HTTP client: {e}"
                )))
            })
    }

    /// Wait for the Toxiproxy control API to start responding.
    ///
    /// Polls `GET /version` on the control port until it returns success or the
    /// configured timeout elapses. Call this after [`start`](Template::start)
    /// (or [`start_and_wait`](Template::start_and_wait)) before registering
    /// proxies.
    ///
    /// # Errors
    ///
    /// Returns an error if the control API does not respond within the timeout.
    pub async fn wait_for_control_api(&self) -> Result<()> {
        let client = Self::http_client()?;
        let url = format!("{}/version", self.control_url());
        let start = std::time::Instant::now();

        while start.elapsed() < self.api_ready_timeout {
            if let Ok(response) = client.get(&url).send().await {
                if response.status().is_success() {
                    return Ok(());
                }
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }

        Err(TemplateError::Timeout(format!(
            "Toxiproxy control API on port {} did not become ready within {}s",
            self.control_port,
            self.api_ready_timeout.as_secs()
        )))
    }

    /// Register a new proxy.
    ///
    /// - `name` identifies the proxy in subsequent calls.
    /// - `listen` is the address the proxy listens on inside the container. Use
    ///   `0.0.0.0:<port>` with a published [`proxy_port`](Self::proxy_port) so host
    ///   clients can connect through it.
    /// - `upstream` is the address the proxy forwards to (e.g. `redis:6379` when on
    ///   a shared network).
    ///
    /// # Errors
    ///
    /// Returns an error if the control API request fails or returns a non-success
    /// status (for example, if a proxy with the same name already exists).
    pub async fn create_proxy(
        &self,
        name: impl Into<String>,
        listen: impl Into<String>,
        upstream: impl Into<String>,
    ) -> Result<ProxyInfo> {
        let client = Self::http_client()?;
        let body = ProxyRequest {
            name: name.into(),
            listen: listen.into(),
            upstream: upstream.into(),
            enabled: true,
        };

        let url = format!("{}/proxies", self.control_url());
        let response = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| map_request_err(&e))?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(TemplateError::InvalidConfig(format!(
                "failed to create proxy '{}': HTTP {status}: {text}",
                body.name
            )));
        }

        response
            .json::<ProxyInfo>()
            .await
            .map_err(|e| map_request_err(&e))
    }

    /// Add a toxic to an existing proxy.
    ///
    /// - `proxy` is the proxy name passed to [`create_proxy`](Self::create_proxy).
    /// - `name` identifies the toxic for later removal.
    /// - `stream` selects the direction the toxic applies to.
    /// - `toxic` is the typed fault to inject.
    ///
    /// The toxic is applied with full toxicity (`1.0`), so it affects every
    /// connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the control API request fails or returns a non-success
    /// status.
    pub async fn add_toxic(
        &self,
        proxy: &str,
        name: impl Into<String>,
        stream: ToxicStream,
        toxic: Toxic,
    ) -> Result<()> {
        let client = Self::http_client()?;
        let body = ToxicRequest {
            name: name.into(),
            toxic_type: toxic.type_name().to_string(),
            stream: stream.as_str().to_string(),
            toxicity: 1.0,
            attributes: toxic.attributes(),
        };

        let url = format!("{}/proxies/{proxy}/toxics", self.control_url());
        let response = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| map_request_err(&e))?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(TemplateError::InvalidConfig(format!(
                "failed to add toxic '{}' to proxy '{proxy}': HTTP {status}: {text}",
                body.name
            )));
        }

        Ok(())
    }

    /// Remove a toxic from a proxy.
    ///
    /// # Errors
    ///
    /// Returns an error if the control API request fails or returns a non-success
    /// status.
    pub async fn remove_toxic(&self, proxy: &str, toxic: &str) -> Result<()> {
        let client = Self::http_client()?;
        let url = format!("{}/proxies/{proxy}/toxics/{toxic}", self.control_url());
        let response = client
            .delete(&url)
            .send()
            .await
            .map_err(|e| map_request_err(&e))?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(TemplateError::InvalidConfig(format!(
                "failed to remove toxic '{toxic}' from proxy '{proxy}': HTTP {status}: {text}"
            )));
        }

        Ok(())
    }

    /// List all registered proxies.
    ///
    /// # Errors
    ///
    /// Returns an error if the control API request fails or the response cannot be
    /// parsed.
    pub async fn list_proxies(&self) -> Result<Vec<ProxyInfo>> {
        let client = Self::http_client()?;
        let url = format!("{}/proxies", self.control_url());
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| map_request_err(&e))?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(TemplateError::InvalidConfig(format!(
                "failed to list proxies: HTTP {status}: {text}"
            )));
        }

        // The control API returns a map of name -> proxy.
        let map = response
            .json::<HashMap<String, ProxyInfo>>()
            .await
            .map_err(|e| map_request_err(&e))?;
        Ok(map.into_values().collect())
    }

    /// Reset all proxies and remove all toxics.
    ///
    /// This re-enables every proxy and clears all toxics, returning Toxiproxy to a
    /// clean state without restarting the container.
    ///
    /// # Errors
    ///
    /// Returns an error if the control API request fails or returns a non-success
    /// status.
    pub async fn reset(&self) -> Result<()> {
        let client = Self::http_client()?;
        let url = format!("{}/reset", self.control_url());
        let response = client
            .post(&url)
            .send()
            .await
            .map_err(|e| map_request_err(&e))?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(TemplateError::InvalidConfig(format!(
                "failed to reset Toxiproxy: HTTP {status}: {text}"
            )));
        }

        Ok(())
    }
}

/// Map a reqwest error into a `TemplateError`.
fn map_request_err(e: &reqwest::Error) -> TemplateError {
    TemplateError::DockerError(crate::Error::custom(format!(
        "Toxiproxy control API request failed: {e}"
    )))
}

#[async_trait]
impl Template for ToxiproxyTemplate {
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
    fn test_toxiproxy_template_defaults() {
        let template = ToxiproxyTemplate::new("test-toxiproxy");
        assert_eq!(template.name(), "test-toxiproxy");
        assert_eq!(template.config().image, DEFAULT_IMAGE);
        assert_eq!(template.config().tag, DEFAULT_TAG);
        assert_eq!(template.control_port, DEFAULT_CONTROL_PORT);
        assert_eq!(template.config().ports, vec![(8474, 8474)]);
    }

    #[test]
    fn test_control_port_replaces_mapping() {
        let template = ToxiproxyTemplate::new("test").control_port(18474);
        assert_eq!(template.control_port, 18474);
        // Container side stays 8474, host side updated; no duplicate mapping.
        assert_eq!(template.config().ports, vec![(18474, 8474)]);
        assert_eq!(template.control_url(), "http://localhost:18474");
    }

    #[test]
    fn test_proxy_port_published() {
        let template = ToxiproxyTemplate::new("test")
            .proxy_port(16379)
            .proxy_port(16379); // idempotent
        let ports = &template.config().ports;
        assert!(ports.contains(&(8474, 8474)));
        assert!(ports.contains(&(16379, 16379)));
        // Only added once despite the duplicate call.
        assert_eq!(ports.iter().filter(|p| **p == (16379, 16379)).count(), 1);
    }

    #[test]
    fn test_network_and_custom_image() {
        let template = ToxiproxyTemplate::new("test")
            .network("chaos-net")
            .custom_image("ghcr.io/shopify/toxiproxy", "latest")
            .platform("linux/arm64");
        assert_eq!(template.config().network.as_deref(), Some("chaos-net"));
        assert_eq!(template.config().image, "ghcr.io/shopify/toxiproxy");
        assert_eq!(template.config().tag, "latest");
        assert_eq!(template.config().platform.as_deref(), Some("linux/arm64"));
    }

    #[test]
    fn test_toxic_latency_attributes() {
        let toxic = Toxic::latency(500);
        assert_eq!(toxic.type_name(), "latency");
        let attrs = toxic.attributes();
        assert_eq!(attrs.get("latency"), Some(&500));
        assert_eq!(attrs.get("jitter"), Some(&0));

        let toxic = Toxic::jitter(500, 100);
        let attrs = toxic.attributes();
        assert_eq!(attrs.get("latency"), Some(&500));
        assert_eq!(attrs.get("jitter"), Some(&100));
    }

    #[test]
    fn test_toxic_bandwidth_attributes() {
        let toxic = Toxic::bandwidth(64);
        assert_eq!(toxic.type_name(), "bandwidth");
        assert_eq!(toxic.attributes().get("rate"), Some(&64));
    }

    #[test]
    fn test_toxic_timeout_attributes() {
        let toxic = Toxic::timeout(0);
        assert_eq!(toxic.type_name(), "timeout");
        assert_eq!(toxic.attributes().get("timeout"), Some(&0));
    }

    #[test]
    fn test_toxic_slicer_attributes() {
        let toxic = Toxic::slicer(64, 32, 10);
        assert_eq!(toxic.type_name(), "slicer");
        let attrs = toxic.attributes();
        assert_eq!(attrs.get("average_size"), Some(&64));
        assert_eq!(attrs.get("size_variation"), Some(&32));
        assert_eq!(attrs.get("delay"), Some(&10));
    }

    #[test]
    fn test_toxic_limit_data_attributes() {
        let toxic = Toxic::limit_data(2048);
        assert_eq!(toxic.type_name(), "limit_data");
        assert_eq!(toxic.attributes().get("bytes"), Some(&2048));
    }

    #[test]
    fn test_toxic_stream_wire_values() {
        assert_eq!(ToxicStream::Downstream.as_str(), "downstream");
        assert_eq!(ToxicStream::Upstream.as_str(), "upstream");
    }
}
