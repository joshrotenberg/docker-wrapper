//! Docker swarm update command implementation.

use crate::command::{CommandExecutor, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Result of swarm update command
#[derive(Debug, Clone)]
pub struct SwarmUpdateResult {
    /// Whether the update was successful
    pub success: bool,
    /// Raw output from the command
    pub output: String,
}

/// Docker swarm update command builder
///
/// Update swarm configuration.
#[derive(Debug, Clone, Default)]
pub struct SwarmUpdateCommand {
    /// Enable or disable autolock
    autolock: Option<bool>,
    /// Validity period for node certificates (ns|us|ms|s|m|h)
    cert_expiry: Option<String>,
    /// Dispatcher heartbeat period (ns|us|ms|s|m|h)
    dispatcher_heartbeat: Option<String>,
    /// Specifications of external CA to use
    external_ca: Option<String>,
    /// Number of snapshots to keep beyond current snapshot
    max_snapshots: Option<u32>,
    /// Number of log entries to trigger a snapshot
    snapshot_interval: Option<u32>,
    /// Task history retention limit
    task_history_limit: Option<i32>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl SwarmUpdateCommand {
    /// Create a new swarm update command
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable autolock
    #[must_use]
    pub fn autolock(mut self, enabled: bool) -> Self {
        self.autolock = Some(enabled);
        self
    }

    /// Set the certificate expiry duration
    #[must_use]
    pub fn cert_expiry(mut self, expiry: impl Into<String>) -> Self {
        self.cert_expiry = Some(expiry.into());
        self
    }

    /// Set the dispatcher heartbeat period
    #[must_use]
    pub fn dispatcher_heartbeat(mut self, heartbeat: impl Into<String>) -> Self {
        self.dispatcher_heartbeat = Some(heartbeat.into());
        self
    }

    /// Set external CA specifications
    #[must_use]
    pub fn external_ca(mut self, spec: impl Into<String>) -> Self {
        self.external_ca = Some(spec.into());
        self
    }

    /// Set the maximum number of snapshots to keep
    #[must_use]
    pub fn max_snapshots(mut self, count: u32) -> Self {
        self.max_snapshots = Some(count);
        self
    }

    /// Set the snapshot interval
    #[must_use]
    pub fn snapshot_interval(mut self, interval: u32) -> Self {
        self.snapshot_interval = Some(interval);
        self
    }

    /// Set the task history retention limit
    #[must_use]
    pub fn task_history_limit(mut self, limit: i32) -> Self {
        self.task_history_limit = Some(limit);
        self
    }

    /// Build the command arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["swarm".to_string(), "update".to_string()];

        if let Some(enabled) = self.autolock {
            args.push("--autolock".to_string());
            args.push(enabled.to_string());
        }

        if let Some(ref expiry) = self.cert_expiry {
            args.push("--cert-expiry".to_string());
            args.push(expiry.clone());
        }

        if let Some(ref heartbeat) = self.dispatcher_heartbeat {
            args.push("--dispatcher-heartbeat".to_string());
            args.push(heartbeat.clone());
        }

        if let Some(ref spec) = self.external_ca {
            args.push("--external-ca".to_string());
            args.push(spec.clone());
        }

        if let Some(count) = self.max_snapshots {
            args.push("--max-snapshots".to_string());
            args.push(count.to_string());
        }

        if let Some(interval) = self.snapshot_interval {
            args.push("--snapshot-interval".to_string());
            args.push(interval.to_string());
        }

        if let Some(limit) = self.task_history_limit {
            args.push("--task-history-limit".to_string());
            args.push(limit.to_string());
        }

        args
    }
}

#[async_trait]
impl DockerCommand for SwarmUpdateCommand {
    type Output = SwarmUpdateResult;

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
        Ok(SwarmUpdateResult {
            success: true,
            output: output.stdout,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_update_basic() {
        let cmd = SwarmUpdateCommand::new();
        let args = cmd.build_args();
        assert_eq!(args, vec!["swarm", "update"]);
    }

    #[test]
    fn test_swarm_update_autolock_enable() {
        let cmd = SwarmUpdateCommand::new().autolock(true);
        let args = cmd.build_args();
        assert!(args.contains(&"--autolock".to_string()));
        assert!(args.contains(&"true".to_string()));
    }

    #[test]
    fn test_swarm_update_autolock_disable() {
        let cmd = SwarmUpdateCommand::new().autolock(false);
        let args = cmd.build_args();
        assert!(args.contains(&"--autolock".to_string()));
        assert!(args.contains(&"false".to_string()));
    }

    #[test]
    fn test_swarm_update_cert_expiry() {
        let cmd = SwarmUpdateCommand::new().cert_expiry("90d");
        let args = cmd.build_args();
        assert!(args.contains(&"--cert-expiry".to_string()));
        assert!(args.contains(&"90d".to_string()));
    }

    #[test]
    fn test_swarm_update_all_options() {
        let cmd = SwarmUpdateCommand::new()
            .autolock(true)
            .cert_expiry("90d")
            .dispatcher_heartbeat("5s")
            .external_ca("protocol=cfssl,url=https://ca.example.com")
            .max_snapshots(5)
            .snapshot_interval(10000)
            .task_history_limit(10);

        let args = cmd.build_args();
        assert!(args.contains(&"--autolock".to_string()));
        assert!(args.contains(&"--cert-expiry".to_string()));
        assert!(args.contains(&"--dispatcher-heartbeat".to_string()));
        assert!(args.contains(&"--external-ca".to_string()));
        assert!(args.contains(&"--max-snapshots".to_string()));
        assert!(args.contains(&"--snapshot-interval".to_string()));
        assert!(args.contains(&"--task-history-limit".to_string()));
    }
}
