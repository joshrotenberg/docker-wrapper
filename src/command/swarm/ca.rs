//! Docker swarm ca command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Result of swarm ca command
#[derive(Debug, Clone)]
pub struct SwarmCaResult {
    /// The CA certificate (PEM format)
    pub certificate: Option<String>,
    /// Raw output from the command
    pub output: String,
}

impl SwarmCaResult {
    /// Parse the swarm ca output
    fn parse(output: &CommandOutput) -> Self {
        let stdout = output.stdout.trim();

        // Check if output looks like a PEM certificate
        let certificate = if stdout.contains("-----BEGIN CERTIFICATE-----") {
            Some(stdout.to_string())
        } else {
            None
        };

        Self {
            certificate,
            output: stdout.to_string(),
        }
    }
}

/// Docker swarm ca command builder
///
/// Display and rotate the root CA certificate.
#[derive(Debug, Clone, Default)]
pub struct SwarmCaCommand {
    /// Path to the PEM-formatted root CA certificate to use
    ca_cert: Option<String>,
    /// Path to the PEM-formatted root CA key to use
    ca_key: Option<String>,
    /// Validity period for node certificates (ns|us|ms|s|m|h)
    cert_expiry: Option<String>,
    /// Exit immediately instead of waiting for rotation to complete
    detach: bool,
    /// Specifications of external CA to use
    external_ca: Option<String>,
    /// Suppress progress output
    quiet: bool,
    /// Rotate the swarm CA (creates new cluster TLS certs)
    rotate: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl SwarmCaCommand {
    /// Create a new swarm ca command
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the path to the root CA certificate
    #[must_use]
    pub fn ca_cert(mut self, path: impl Into<String>) -> Self {
        self.ca_cert = Some(path.into());
        self
    }

    /// Set the path to the root CA key
    #[must_use]
    pub fn ca_key(mut self, path: impl Into<String>) -> Self {
        self.ca_key = Some(path.into());
        self
    }

    /// Set the certificate expiry duration
    #[must_use]
    pub fn cert_expiry(mut self, expiry: impl Into<String>) -> Self {
        self.cert_expiry = Some(expiry.into());
        self
    }

    /// Exit immediately instead of waiting for rotation to complete
    #[must_use]
    pub fn detach(mut self) -> Self {
        self.detach = true;
        self
    }

    /// Set external CA specifications
    #[must_use]
    pub fn external_ca(mut self, spec: impl Into<String>) -> Self {
        self.external_ca = Some(spec.into());
        self
    }

    /// Suppress progress output
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// Rotate the swarm CA
    #[must_use]
    pub fn rotate(mut self) -> Self {
        self.rotate = true;
        self
    }

    /// Build the command arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["swarm".to_string(), "ca".to_string()];

        if let Some(ref path) = self.ca_cert {
            args.push("--ca-cert".to_string());
            args.push(path.clone());
        }

        if let Some(ref path) = self.ca_key {
            args.push("--ca-key".to_string());
            args.push(path.clone());
        }

        if let Some(ref expiry) = self.cert_expiry {
            args.push("--cert-expiry".to_string());
            args.push(expiry.clone());
        }

        if self.detach {
            args.push("--detach".to_string());
        }

        if let Some(ref spec) = self.external_ca {
            args.push("--external-ca".to_string());
            args.push(spec.clone());
        }

        if self.quiet {
            args.push("--quiet".to_string());
        }

        if self.rotate {
            args.push("--rotate".to_string());
        }

        args
    }
}

#[async_trait]
impl DockerCommand for SwarmCaCommand {
    type Output = SwarmCaResult;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        self.build_args()
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_args();
        let output = self.execute_command(args).await?;
        Ok(SwarmCaResult::parse(&output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_ca_basic() {
        let cmd = SwarmCaCommand::new();
        let args = cmd.build_args();
        assert_eq!(args, vec!["swarm", "ca"]);
    }

    #[test]
    fn test_swarm_ca_rotate() {
        let cmd = SwarmCaCommand::new().rotate();
        let args = cmd.build_args();
        assert!(args.contains(&"--rotate".to_string()));
    }

    #[test]
    fn test_swarm_ca_with_cert_and_key() {
        let cmd = SwarmCaCommand::new()
            .ca_cert("/path/to/cert.pem")
            .ca_key("/path/to/key.pem");
        let args = cmd.build_args();
        assert!(args.contains(&"--ca-cert".to_string()));
        assert!(args.contains(&"/path/to/cert.pem".to_string()));
        assert!(args.contains(&"--ca-key".to_string()));
        assert!(args.contains(&"/path/to/key.pem".to_string()));
    }

    #[test]
    fn test_swarm_ca_all_options() {
        let cmd = SwarmCaCommand::new()
            .ca_cert("/cert.pem")
            .ca_key("/key.pem")
            .cert_expiry("90d")
            .detach()
            .external_ca("protocol=cfssl,url=https://ca.example.com")
            .quiet()
            .rotate();

        let args = cmd.build_args();
        assert!(args.contains(&"--ca-cert".to_string()));
        assert!(args.contains(&"--ca-key".to_string()));
        assert!(args.contains(&"--cert-expiry".to_string()));
        assert!(args.contains(&"--detach".to_string()));
        assert!(args.contains(&"--external-ca".to_string()));
        assert!(args.contains(&"--quiet".to_string()));
        assert!(args.contains(&"--rotate".to_string()));
    }
}
