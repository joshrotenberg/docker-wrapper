//! Basic Redis template for quick Redis container setup

#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::unnecessary_get_then_check)]

use super::common::{
    default_redis_health_check, redis_config_volume, redis_connection_string, redis_data_volume,
    redis_tls_connection_string, redis_tls_server_args, redis_tls_volume, DEFAULT_REDIS_IMAGE,
    DEFAULT_REDIS_TAG, DEFAULT_REDIS_TLS_PORT, REDIS_STACK_IMAGE, REDIS_STACK_TAG,
};
use crate::template::{HasConnectionString, Template, TemplateConfig};
use async_trait::async_trait;
use std::collections::HashMap;

/// Redis container template with sensible defaults
pub struct RedisTemplate {
    config: TemplateConfig,
    use_redis_stack: bool,
    stack_tag: String,
    /// Host directory containing TLS certificate material, mounted read-only
    /// into the container when TLS is enabled.
    tls_certs_dir: Option<String>,
    /// Host-side port mapped to the container TLS port when TLS is enabled.
    tls_port: u16,
    /// When true, the plaintext port is disabled (`--port 0`) and only TLS is
    /// served.
    tls_only: bool,
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
            stack_tag: REDIS_STACK_TAG.to_string(),
            tls_certs_dir: None,
            tls_port: DEFAULT_REDIS_TLS_PORT,
            tls_only: false,
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

    /// Set the container network mode (e.g. `"host"`, `"bridge"`, `"none"`).
    ///
    /// This is a thin alias over [`network`](Self::network) that reads better
    /// when selecting a Docker network *mode* rather than a named network. See
    /// [`host_network`](Self::host_network) for the host-mode caveats.
    pub fn network_mode(mut self, mode: impl Into<String>) -> Self {
        self.config.network = Some(mode.into());
        self
    }

    /// Run the container with `--network host`.
    ///
    /// In host networking mode the container shares the host's network
    /// namespace: the Redis port is reachable directly on the host with no
    /// published port mapping, so no `-p` flag is emitted and the host-side
    /// port equals the container port (6379 by default).
    ///
    /// # Platform support
    ///
    /// Host networking is a **Linux-only** Docker feature. On Docker Desktop
    /// for macOS and Windows the daemon runs inside a Linux VM, so
    /// `--network host` binds ports inside that VM rather than on your machine:
    /// the option is effectively a **no-op** there and the Redis port will not
    /// be reachable from the host. This method does not return an error on
    /// non-Linux hosts (the Docker CLI accepts the flag regardless of backend);
    /// only use host mode against a native Linux daemon.
    ///
    /// # Example
    ///
    /// ```rust
    /// use docker_wrapper::template::{RedisTemplate, Template};
    /// use docker_wrapper::DockerCommand;
    ///
    /// // Linux only: Redis is reachable on localhost:6379 with no -p mapping.
    /// let template = RedisTemplate::new("host-redis").host_network();
    /// let args = template.build_command().build_command_args();
    /// assert!(args.contains(&"--network".to_string()));
    /// assert!(args.contains(&"host".to_string()));
    /// // Host mode publishes no ports.
    /// assert!(!args.contains(&"--publish".to_string()));
    /// ```
    pub fn host_network(mut self) -> Self {
        self.config.network = Some("host".to_string());
        self
    }

    /// Returns true when the template is configured for host networking.
    fn uses_host_network(&self) -> bool {
        self.config.network.as_deref() == Some("host")
    }

    /// Enable auto-remove when stopped
    pub fn auto_remove(mut self) -> Self {
        self.config.auto_remove = true;
        self
    }

    /// Use Redis Stack image instead of basic Redis.
    ///
    /// Uses the `redis/redis-stack` image pinned to a known-good default tag
    /// (`7.4.0-v3`) rather than `latest`, so that runs are reproducible. Call
    /// [`stack_version`](Self::stack_version) to pin a different tag.
    pub fn with_redis_stack(mut self) -> Self {
        self.use_redis_stack = true;
        self
    }

    /// Pin the Redis Stack image tag (e.g. `"7.4.0-v3"`).
    ///
    /// Only affects the image used when [`Self::with_redis_stack`] is enabled.
    /// The default is a known-good pinned tag rather than `latest`, so that runs
    /// are reproducible. For full control over both the image name and tag, use
    /// [`Self::custom_image`] instead.
    ///
    /// # Example
    ///
    /// ```rust
    /// use docker_wrapper::template::RedisTemplate;
    ///
    /// let template = RedisTemplate::new("my-redis")
    ///     .with_redis_stack()
    ///     .stack_version("7.4.0-v3");
    /// ```
    pub fn stack_version(mut self, tag: impl Into<String>) -> Self {
        self.stack_tag = tag.into();
        self
    }

    /// Build the image reference used when Redis Stack is enabled.
    fn stack_image(&self) -> String {
        format!("{REDIS_STACK_IMAGE}:{}", self.stack_tag)
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

    /// Enable TLS, bind-mounting the given host certificate directory.
    ///
    /// The directory is mounted read-only into the container and Redis is
    /// started with `--tls-port`, `--tls-cert-file`, `--tls-key-file` and
    /// `--tls-ca-cert-file`. The directory **must** contain these files:
    ///
    /// - `redis.crt` -- the server certificate
    /// - `redis.key` -- the server private key
    /// - `ca.crt` -- the CA certificate used to verify client certificates
    ///
    /// By default the plaintext port stays open alongside TLS; call
    /// [`tls_only`](Self::tls_only) to disable plaintext (`--port 0`). The TLS
    /// port is published on the host (6380 by default, override with
    /// [`tls_port`](Self::tls_port)).
    ///
    /// # Generating throwaway certificates
    ///
    /// For local testing you can generate a self-signed CA and server
    /// certificate with `openssl`:
    ///
    /// ```sh
    /// openssl genrsa -out ca.key 2048
    /// openssl req -x509 -new -nodes -key ca.key -sha256 -days 365 \
    ///   -subj "/CN=test-ca" -out ca.crt
    /// openssl genrsa -out redis.key 2048
    /// openssl req -new -key redis.key -subj "/CN=localhost" -out redis.csr
    /// openssl x509 -req -in redis.csr -CA ca.crt -CAkey ca.key \
    ///   -CAcreateserial -days 365 -sha256 -out redis.crt
    /// ```
    ///
    /// # Example
    ///
    /// ```rust
    /// use docker_wrapper::template::{RedisTemplate, Template};
    /// use docker_wrapper::DockerCommand;
    ///
    /// let template = RedisTemplate::new("tls-redis").tls("/path/to/certs");
    /// let args = template.build_command().build_command_args();
    /// assert!(args.contains(&"--tls-port".to_string()));
    /// assert!(args.contains(&"6380".to_string()));
    /// ```
    pub fn tls(mut self, certs_dir: impl Into<String>) -> Self {
        self.tls_certs_dir = Some(certs_dir.into());
        self
    }

    /// Set the host-side port published for TLS connections (default 6380).
    ///
    /// Only has an effect when TLS is enabled via [`tls`](Self::tls).
    pub fn tls_port(mut self, port: u16) -> Self {
        self.tls_port = port;
        self
    }

    /// Disable the plaintext port and serve only TLS.
    ///
    /// Sets `--port 0` (which tells Redis to stop listening on the plaintext
    /// port) and skips publishing the plaintext port mapping. Only has an
    /// effect when TLS is enabled via [`tls`](Self::tls).
    pub fn tls_only(mut self) -> Self {
        self.tls_only = true;
        self
    }

    /// Returns true when TLS has been enabled on this template.
    fn tls_enabled(&self) -> bool {
        self.tls_certs_dir.is_some()
    }

    /// Returns the TLS connection string in URL format, or `None` when TLS is
    /// not enabled.
    ///
    /// Format: `rediss://[:password@]host:port`, where `port` is the published
    /// TLS port (6380 by default, see [`tls_port`](Self::tls_port)).
    ///
    /// # Example
    ///
    /// ```rust
    /// use docker_wrapper::template::RedisTemplate;
    ///
    /// let template = RedisTemplate::new("my-redis").tls("/certs");
    /// assert_eq!(
    ///     template.tls_connection_string().as_deref(),
    ///     Some("rediss://localhost:6380")
    /// );
    ///
    /// let plaintext = RedisTemplate::new("my-redis");
    /// assert_eq!(plaintext.tls_connection_string(), None);
    /// ```
    pub fn tls_connection_string(&self) -> Option<String> {
        if !self.tls_enabled() {
            return None;
        }
        let password = self.config.env.get("REDIS_PASSWORD").map(String::as_str);
        Some(redis_tls_connection_string(
            "localhost",
            self.tls_port,
            password,
        ))
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

    async fn wait_for_ready(&self) -> crate::template::Result<()> {
        use std::time::Duration;
        use tokio::time::{sleep, timeout};

        // Custom Redis readiness check
        // Use 60 second timeout for slower systems (especially Windows)
        let wait_timeout = Duration::from_secs(60);
        let check_interval = Duration::from_millis(500);

        timeout(wait_timeout, async {
            loop {
                // Check if container is running - keep retrying if not yet started
                // Don't fail immediately as the container may still be starting up
                if !self.is_running().await.unwrap_or(false) {
                    sleep(check_interval).await;
                    continue;
                }

                // Try to ping Redis
                let password = self.config.env.get("REDIS_PASSWORD");
                let mut ping_cmd = vec!["redis-cli", "-h", "localhost"];

                // Add auth if password is set
                let auth_args;
                if let Some(pass) = password {
                    auth_args = vec!["-a", pass.as_str()];
                    ping_cmd.extend(&auth_args);
                }

                ping_cmd.push("ping");

                // Execute ping command
                if let Ok(result) = self.exec(ping_cmd).await {
                    if result.stdout.trim() == "PONG" {
                        return Ok(());
                    }
                }

                sleep(check_interval).await;
            }
        })
        .await
        .map_err(|_| {
            crate::template::TemplateError::InvalidConfig(format!(
                "Redis container {} failed to become ready within timeout",
                self.config().name
            ))
        })?
    }

    fn build_command(&self) -> crate::RunCommand {
        let config = self.config();

        // Choose image based on Redis Stack preference
        let image_tag = if self.use_redis_stack {
            self.stack_image()
        } else {
            format!("{}:{}", config.image, config.tag)
        };

        let mut cmd = crate::RunCommand::new(image_tag)
            .name(&config.name)
            .detach();

        // Add port mappings. In host networking mode the container shares the
        // host's network namespace, so published ports are ignored by Docker
        // (and emit a warning); skip them entirely.
        if !self.uses_host_network() {
            // Publish the plaintext port unless TLS-only mode disabled it.
            if !(self.tls_enabled() && self.tls_only) {
                for (host, container) in &config.ports {
                    cmd = cmd.port(*host, *container);
                }
            }
            // Publish the TLS port when TLS is enabled.
            if self.tls_enabled() {
                cmd = cmd.port(self.tls_port, DEFAULT_REDIS_TLS_PORT);
            }
        }

        // Add volume mounts
        for mount in &config.volumes {
            if mount.read_only {
                cmd = cmd.volume_ro(&mount.source, &mount.target);
            } else {
                cmd = cmd.volume(&mount.source, &mount.target);
            }
        }

        // Mount the TLS certificate directory read-only when TLS is enabled.
        if let Some(ref certs_dir) = self.tls_certs_dir {
            let mount = redis_tls_volume(certs_dir.clone());
            cmd = cmd.volume_ro(&mount.source, &mount.target);
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

        // Handle Redis-specific command args. Password and TLS both require
        // overriding the redis-server flags; compose them together so they can
        // coexist.
        let password = config.env.get("REDIS_PASSWORD");
        if password.is_some() || self.tls_enabled() {
            // Flags shared by both the Stack (REDIS_ARGS) and basic
            // (entrypoint override) paths.
            let mut server_flags: Vec<String> = Vec::new();
            if let Some(password) = password {
                server_flags.push("--requirepass".to_string());
                server_flags.push(password.clone());
                server_flags.push("--protected-mode".to_string());
                server_flags.push("yes".to_string());
            }
            if self.tls_enabled() {
                if self.tls_only {
                    // Disable the plaintext listener.
                    server_flags.push("--port".to_string());
                    server_flags.push("0".to_string());
                }
                server_flags.extend(redis_tls_server_args(DEFAULT_REDIS_TLS_PORT));
            }

            if self.use_redis_stack {
                // For Redis Stack, pass flags via the REDIS_ARGS environment
                // variable instead of overriding the entrypoint.
                cmd = cmd.env("REDIS_ARGS", server_flags.join(" "));
            } else {
                // For basic Redis, override the entrypoint to bypass
                // docker-entrypoint.sh and run redis-server directly.
                cmd = cmd.entrypoint("redis-server").cmd(server_flags);
            }
        }

        // If a custom config file is mounted (and neither password nor TLS
        // overrode the command), launch redis-server with that config file.
        let has_config = config
            .volumes
            .iter()
            .any(|v| v.target == "/usr/local/etc/redis/redis.conf");
        if has_config && password.is_none() && !self.tls_enabled() {
            cmd = cmd.cmd(vec![
                "redis-server".to_string(),
                "/usr/local/etc/redis/redis.conf".to_string(),
            ]);
        }

        cmd
    }
}

impl HasConnectionString for RedisTemplate {
    /// Returns the Redis connection string in URL format.
    ///
    /// Format: `redis://[:password@]host:port`
    ///
    /// When the template is configured for TLS-only access (see
    /// [`tls_only`](RedisTemplate::tls_only)) the plaintext port is disabled, so
    /// this returns the `rediss://` TLS endpoint instead. When TLS is enabled
    /// *alongside* plaintext, this still returns the plaintext URL; use
    /// [`tls_connection_string`](RedisTemplate::tls_connection_string) for the
    /// TLS endpoint.
    ///
    /// # Example
    ///
    /// ```rust
    /// use docker_wrapper::template::{RedisTemplate, HasConnectionString};
    ///
    /// let template = RedisTemplate::new("my-redis").port(6380);
    /// assert_eq!(template.connection_string(), "redis://localhost:6380");
    ///
    /// let template_with_pass = RedisTemplate::new("my-redis")
    ///     .port(6380)
    ///     .password("secret");
    /// assert_eq!(template_with_pass.connection_string(), "redis://:secret@localhost:6380");
    ///
    /// // TLS-only falls back to the rediss:// endpoint.
    /// let tls = RedisTemplate::new("my-redis").tls("/certs").tls_only();
    /// assert_eq!(tls.connection_string(), "rediss://localhost:6380");
    /// ```
    fn connection_string(&self) -> String {
        // In TLS-only mode the plaintext port is closed, so the only usable
        // endpoint is the TLS one.
        if self.tls_enabled() && self.tls_only {
            if let Some(tls) = self.tls_connection_string() {
                return tls;
            }
        }
        let port = self.config.ports.first().map_or(6379, |(h, _)| *h);
        let password = self.config.env.get("REDIS_PASSWORD").map(String::as_str);
        redis_connection_string("localhost", port, password)
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

    #[test]
    fn test_redis_host_network() {
        let template = RedisTemplate::new("test-redis").host_network();
        assert_eq!(template.config().network.as_deref(), Some("host"));

        let cmd = template.build_command();
        let args = cmd.build_command_args();

        // --network host is wired and no ports are published.
        let network_pos = args.iter().position(|a| a == "--network").unwrap();
        assert_eq!(args[network_pos + 1], "host");
        assert!(!args.contains(&"--publish".to_string()));
    }

    #[test]
    fn test_redis_network_mode_host() {
        let template = RedisTemplate::new("test-redis").network_mode("host");
        assert_eq!(template.config().network.as_deref(), Some("host"));

        let cmd = template.build_command();
        let args = cmd.build_command_args();
        assert!(!args.contains(&"--publish".to_string()));
    }

    #[test]
    fn test_redis_network_mode_named_still_publishes() {
        // A non-host network mode is just a named network and still publishes
        // ports as usual.
        let template = RedisTemplate::new("test-redis")
            .port(16379)
            .network_mode("my-net");

        let cmd = template.build_command();
        let args = cmd.build_command_args();
        assert!(args.contains(&"--publish".to_string()));
        assert!(args.contains(&"16379:6379".to_string()));
    }

    #[test]
    fn test_redis_stack_default_pinned_tag() {
        // Redis Stack defaults to a pinned, known-good tag (not latest) for
        // reproducible runs.
        let template = RedisTemplate::new("test-redis").with_redis_stack();

        assert_eq!(
            template.stack_image(),
            format!("{REDIS_STACK_IMAGE}:7.4.0-v3")
        );

        let cmd = template.build_command();
        let args = cmd.build_command_args();
        assert!(args.contains(&"redis/redis-stack:7.4.0-v3".to_string()));
        assert!(!args.iter().any(|a| a == "redis/redis-stack:latest"));
    }

    #[test]
    fn test_redis_stack_version_override() {
        let template = RedisTemplate::new("test-redis")
            .with_redis_stack()
            .stack_version("7.2.0-v9");

        assert_eq!(template.stack_image(), "redis/redis-stack:7.2.0-v9");

        let cmd = template.build_command();
        let args = cmd.build_command_args();
        assert!(args.contains(&"redis/redis-stack:7.2.0-v9".to_string()));
    }

    #[test]
    fn test_redis_stack_version_ignored_without_stack() {
        // stack_version only affects the image when Redis Stack is enabled.
        let template = RedisTemplate::new("test-redis").stack_version("7.2.0-v9");

        let cmd = template.build_command();
        let args = cmd.build_command_args();
        assert!(args.contains(&"redis:7-alpine".to_string()));
        assert!(!args.iter().any(|a| a.starts_with("redis/redis-stack")));
    }

    #[test]
    fn test_redis_connection_string() {
        use crate::template::HasConnectionString;

        let template = RedisTemplate::new("test-redis").port(6380);
        assert_eq!(template.connection_string(), "redis://localhost:6380");
    }

    #[test]
    fn test_redis_connection_string_with_password() {
        use crate::template::HasConnectionString;

        let template = RedisTemplate::new("test-redis")
            .port(6380)
            .password("secret");
        assert_eq!(
            template.connection_string(),
            "redis://:secret@localhost:6380"
        );
    }

    #[test]
    fn test_redis_connection_string_default_port() {
        use crate::template::HasConnectionString;

        let template = RedisTemplate::new("test-redis");
        assert_eq!(template.connection_string(), "redis://localhost:6379");
    }

    #[test]
    fn test_redis_tls_args_and_volume() {
        let template = RedisTemplate::new("test-redis").tls("/tmp/certs");

        let cmd = template.build_command();
        let args = cmd.build_command_args();

        // TLS server flags are present.
        assert!(args.contains(&"--tls-port".to_string()));
        assert!(args.contains(&"6380".to_string()));
        assert!(args.contains(&"--tls-cert-file".to_string()));
        assert!(args.contains(&"/tls/redis.crt".to_string()));
        assert!(args.contains(&"--tls-key-file".to_string()));
        assert!(args.contains(&"/tls/redis.key".to_string()));
        assert!(args.contains(&"--tls-ca-cert-file".to_string()));
        assert!(args.contains(&"/tls/ca.crt".to_string()));

        // Certs are mounted read-only at /tls.
        assert!(args.contains(&"/tmp/certs:/tls:ro".to_string()));

        // The TLS port is published, and plaintext stays open by default.
        assert!(args.contains(&"6380:6380".to_string()));
        assert!(args.contains(&"6379:6379".to_string()));

        // Plaintext is not disabled by default.
        assert!(!args.windows(2).any(|w| w == ["--port", "0"]));
    }

    #[test]
    fn test_redis_tls_custom_port() {
        let template = RedisTemplate::new("test-redis")
            .tls("/tmp/certs")
            .tls_port(7000);

        let cmd = template.build_command();
        let args = cmd.build_command_args();

        // The host TLS port maps to the container TLS port.
        assert!(args.contains(&"7000:6380".to_string()));
    }

    #[test]
    fn test_redis_tls_only_disables_plaintext() {
        let template = RedisTemplate::new("test-redis")
            .tls("/tmp/certs")
            .tls_only();

        let cmd = template.build_command();
        let args = cmd.build_command_args();

        // Plaintext is disabled via --port 0 and not published.
        assert!(args.windows(2).any(|w| w == ["--port", "0"]));
        assert!(!args.contains(&"6379:6379".to_string()));

        // The TLS port is still published.
        assert!(args.contains(&"6380:6380".to_string()));
    }

    #[test]
    fn test_redis_tls_with_password() {
        let template = RedisTemplate::new("test-redis")
            .tls("/tmp/certs")
            .password("secret");

        let cmd = template.build_command();
        let args = cmd.build_command_args();

        // Both password and TLS flags coexist.
        assert!(args.windows(2).any(|w| w == ["--requirepass", "secret"]));
        assert!(args.contains(&"--tls-port".to_string()));
    }

    #[test]
    fn test_redis_tls_connection_string() {
        let template = RedisTemplate::new("test-redis").tls("/tmp/certs");
        assert_eq!(
            template.tls_connection_string().as_deref(),
            Some("rediss://localhost:6380")
        );

        let with_pass = RedisTemplate::new("test-redis")
            .tls("/tmp/certs")
            .tls_port(7000)
            .password("secret");
        assert_eq!(
            with_pass.tls_connection_string().as_deref(),
            Some("rediss://:secret@localhost:7000")
        );
    }

    #[test]
    fn test_redis_tls_connection_string_none_without_tls() {
        let template = RedisTemplate::new("test-redis");
        assert_eq!(template.tls_connection_string(), None);
    }

    #[test]
    fn test_redis_tls_only_connection_string_falls_back_to_tls() {
        use crate::template::HasConnectionString;

        let template = RedisTemplate::new("test-redis")
            .tls("/tmp/certs")
            .tls_only();
        // Plaintext is closed, so connection_string() returns the TLS endpoint.
        assert_eq!(template.connection_string(), "rediss://localhost:6380");
    }
}
