//! Integration tests for Redis template TLS support.
//!
//! These tests generate throwaway self-signed certificates with `openssl` in a
//! temporary directory, start a TLS-enabled Redis container, and verify a
//! `redis-cli --tls` connection succeeds. They are skipped automatically when
//! Docker or `openssl` is unavailable.

#[cfg(feature = "template-redis")]
mod redis_tls_tests {
    use docker_wrapper::{DockerCommand, RedisTemplate, Template, VersionCommand};
    use std::path::Path;
    use std::process::Command;

    /// Generate a unique container name for tests.
    fn test_container_name(suffix: &str) -> String {
        format!("test-redis-tls-{}-{}", suffix, uuid::Uuid::new_v4())
    }

    /// Generate a random port for testing to avoid conflicts.
    fn random_port() -> u16 {
        30000 + (uuid::Uuid::new_v4().as_u128() % 10000) as u16
    }

    /// Returns true if Docker is available on this host.
    async fn docker_available() -> bool {
        VersionCommand::new().execute().await.is_ok()
    }

    /// Returns true if the `openssl` binary is available on this host.
    fn openssl_available() -> bool {
        Command::new("openssl")
            .arg("version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Run an `openssl` subcommand, asserting it succeeds.
    fn openssl(dir: &Path, args: &[&str]) {
        let output = Command::new("openssl")
            .args(args)
            .current_dir(dir)
            .output()
            .expect("failed to spawn openssl");
        assert!(
            output.status.success(),
            "openssl {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Generate a self-signed CA plus a server certificate signed by it, using
    /// the file names the Redis templates expect (`ca.crt`, `redis.crt`,
    /// `redis.key`). The mounted directory must be world-readable so the
    /// container's `redis` user can read the key.
    fn generate_certs(dir: &Path) {
        // CA key + self-signed CA cert.
        openssl(dir, &["genrsa", "-out", "ca.key", "2048"]);
        openssl(
            dir,
            &[
                "req",
                "-x509",
                "-new",
                "-nodes",
                "-key",
                "ca.key",
                "-sha256",
                "-days",
                "1",
                "-subj",
                "/CN=docker-wrapper-test-ca",
                "-out",
                "ca.crt",
            ],
        );

        // Server key + CSR + cert signed by the CA.
        openssl(dir, &["genrsa", "-out", "redis.key", "2048"]);
        openssl(
            dir,
            &[
                "req",
                "-new",
                "-key",
                "redis.key",
                "-subj",
                "/CN=localhost",
                "-out",
                "redis.csr",
            ],
        );
        openssl(
            dir,
            &[
                "x509",
                "-req",
                "-in",
                "redis.csr",
                "-CA",
                "ca.crt",
                "-CAkey",
                "ca.key",
                "-CAcreateserial",
                "-days",
                "1",
                "-sha256",
                "-out",
                "redis.crt",
            ],
        );

        // The container's redis user must be able to read the key material.
        for file in ["ca.crt", "redis.crt", "redis.key"] {
            let path = dir.join(file);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))
                    .expect("set cert permissions");
            }
            assert!(path.exists(), "expected {} to exist", file);
        }
    }

    /// In-container redis-cli arguments for a TLS ping against the TLS port.
    fn tls_ping(port: u16) -> Vec<String> {
        vec![
            "redis-cli".to_string(),
            "--tls".to_string(),
            "-p".to_string(),
            port.to_string(),
            "--cacert".to_string(),
            "/tls/ca.crt".to_string(),
            "--cert".to_string(),
            "/tls/redis.crt".to_string(),
            "--key".to_string(),
            "/tls/redis.key".to_string(),
            "ping".to_string(),
        ]
    }

    #[tokio::test]
    async fn test_redis_tls_connection() -> Result<(), Box<dyn std::error::Error>> {
        if !docker_available().await {
            eprintln!("Docker not available, skipping TLS test");
            return Ok(());
        }
        if !openssl_available() {
            eprintln!("openssl not available, skipping TLS test");
            return Ok(());
        }

        let certs = tempfile::tempdir()?;
        generate_certs(certs.path());

        let name = test_container_name("connect");
        let tls_port = random_port();
        // Keep plaintext open so the template's readiness ping works, then prove
        // the TLS listener accepts a redis-cli --tls connection.
        let redis = RedisTemplate::new(&name)
            .port(random_port())
            .tls_port(tls_port)
            .tls(certs.path().to_string_lossy().to_string());

        let container_id = redis.start_and_wait().await?;
        assert!(!container_id.is_empty());

        // A TLS ping against the container TLS port (6380) must succeed.
        let tls_cli = tls_ping(6380);
        let cli: Vec<&str> = tls_cli.iter().map(String::as_str).collect();
        let result = redis.exec(cli).await?;
        assert_eq!(
            result.stdout.trim(),
            "PONG",
            "TLS ping failed: stdout={:?} stderr={:?}",
            result.stdout,
            result.stderr
        );

        // The plaintext port is still serving in non-tls-only mode.
        let plain = redis.exec(vec!["redis-cli", "ping"]).await?;
        assert_eq!(plain.stdout.trim(), "PONG");

        // Clean up.
        redis.stop().await?;
        redis.remove().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_tls_only_disables_plaintext() -> Result<(), Box<dyn std::error::Error>> {
        if !docker_available().await {
            eprintln!("Docker not available, skipping TLS test");
            return Ok(());
        }
        if !openssl_available() {
            eprintln!("openssl not available, skipping TLS test");
            return Ok(());
        }

        let certs = tempfile::tempdir()?;
        generate_certs(certs.path());

        let name = test_container_name("tls-only");
        let tls_port = random_port();
        // TLS-only: plaintext is disabled (--port 0). The container's own health
        // check is plaintext, so it never reports healthy here; we start without
        // waiting and poll the TLS listener directly instead.
        let redis = RedisTemplate::new(&name)
            .tls_port(tls_port)
            .tls(certs.path().to_string_lossy().to_string())
            .tls_only();

        redis.start().await?;

        // Poll the TLS listener until it answers PONG (the container is up but
        // has no plaintext readiness signal).
        let tls_cli = tls_ping(6380);
        let mut ready = false;
        for _ in 0..60 {
            let cli: Vec<&str> = tls_cli.iter().map(String::as_str).collect();
            if let Ok(out) = redis.exec(cli).await {
                if out.stdout.trim() == "PONG" {
                    ready = true;
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        assert!(ready, "TLS-only Redis did not answer a TLS ping in time");

        // Plaintext must be refused: a plaintext redis-cli ping against the
        // disabled port either fails the exec (connection refused) or, at most,
        // never returns PONG. Both outcomes prove plaintext is closed.
        let plain = redis.exec(vec!["redis-cli", "-p", "6379", "ping"]).await;
        match plain {
            Ok(out) => assert_ne!(
                out.stdout.trim(),
                "PONG",
                "plaintext port should be disabled in TLS-only mode, got: {:?}",
                out.stdout
            ),
            Err(_) => {
                // Connection refused on the plaintext port is the expected
                // result in TLS-only mode.
            }
        }

        // Clean up.
        redis.stop().await?;
        redis.remove().await?;

        Ok(())
    }
}
