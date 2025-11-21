//! Docker update command implementation.
//!
//! This module provides the `docker update` command for updating container configurations.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker update command builder
///
/// Update configuration of one or more containers.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::UpdateCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Update memory limit
/// let result = UpdateCommand::new("my-container")
///     .memory("512m")
///     .run()
///     .await?;
///
/// if result.success() {
///     println!("Container updated successfully");
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct UpdateCommand {
    /// Container names or IDs to update
    containers: Vec<String>,
    /// Memory limit
    memory: Option<String>,
    /// Memory reservation (soft limit)
    memory_reservation: Option<String>,
    /// Memory swap limit
    memory_swap: Option<String>,
    /// CPU shares (relative weight)
    cpu_shares: Option<u64>,
    /// CPU period
    cpu_period: Option<u64>,
    /// CPU quota
    cpu_quota: Option<i64>,
    /// CPUs (number of CPUs)
    cpus: Option<String>,
    /// CPU set
    cpuset_cpus: Option<String>,
    /// Memory nodes
    cpuset_mems: Option<String>,
    /// Block IO weight
    blkio_weight: Option<u16>,
    /// Kernel memory limit
    kernel_memory: Option<String>,
    /// Restart policy
    restart: Option<String>,
    /// PID limit
    pids_limit: Option<i64>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl UpdateCommand {
    /// Create a new update command for a single container
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::UpdateCommand;
    ///
    /// let cmd = UpdateCommand::new("my-container");
    /// ```
    #[must_use]
    pub fn new(container: impl Into<String>) -> Self {
        Self {
            containers: vec![container.into()],
            memory: None,
            memory_reservation: None,
            memory_swap: None,
            cpu_shares: None,
            cpu_period: None,
            cpu_quota: None,
            cpus: None,
            cpuset_cpus: None,
            cpuset_mems: None,
            blkio_weight: None,
            kernel_memory: None,
            restart: None,
            pids_limit: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Create a new update command for multiple containers
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::UpdateCommand;
    ///
    /// let cmd = UpdateCommand::new_multiple(vec!["web", "db", "cache"]);
    /// ```
    #[must_use]
    pub fn new_multiple(containers: Vec<impl Into<String>>) -> Self {
        Self {
            containers: containers.into_iter().map(Into::into).collect(),
            memory: None,
            memory_reservation: None,
            memory_swap: None,
            cpu_shares: None,
            cpu_period: None,
            cpu_quota: None,
            cpus: None,
            cpuset_cpus: None,
            cpuset_mems: None,
            blkio_weight: None,
            kernel_memory: None,
            restart: None,
            pids_limit: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Add another container to update
    #[must_use]
    pub fn container(mut self, container: impl Into<String>) -> Self {
        self.containers.push(container.into());
        self
    }

    /// Set memory limit
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::UpdateCommand;
    ///
    /// let cmd = UpdateCommand::new("my-container")
    ///     .memory("512m");
    /// ```
    #[must_use]
    pub fn memory(mut self, memory: impl Into<String>) -> Self {
        self.memory = Some(memory.into());
        self
    }

    /// Set memory reservation (soft limit)
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::UpdateCommand;
    ///
    /// let cmd = UpdateCommand::new("my-container")
    ///     .memory_reservation("256m");
    /// ```
    #[must_use]
    pub fn memory_reservation(mut self, memory_reservation: impl Into<String>) -> Self {
        self.memory_reservation = Some(memory_reservation.into());
        self
    }

    /// Set memory swap limit
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::UpdateCommand;
    ///
    /// let cmd = UpdateCommand::new("my-container")
    ///     .memory_swap("1g");
    /// ```
    #[must_use]
    pub fn memory_swap(mut self, memory_swap: impl Into<String>) -> Self {
        self.memory_swap = Some(memory_swap.into());
        self
    }

    /// Set CPU shares (relative weight)
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::UpdateCommand;
    ///
    /// let cmd = UpdateCommand::new("my-container")
    ///     .cpu_shares(512);
    /// ```
    #[must_use]
    pub fn cpu_shares(mut self, cpu_shares: u64) -> Self {
        self.cpu_shares = Some(cpu_shares);
        self
    }

    /// Set CPU period
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::UpdateCommand;
    ///
    /// let cmd = UpdateCommand::new("my-container")
    ///     .cpu_period(100_000);
    /// ```
    #[must_use]
    pub fn cpu_period(mut self, cpu_period: u64) -> Self {
        self.cpu_period = Some(cpu_period);
        self
    }

    /// Set CPU quota
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::UpdateCommand;
    ///
    /// let cmd = UpdateCommand::new("my-container")
    ///     .cpu_quota(50000);
    /// ```
    #[must_use]
    pub fn cpu_quota(mut self, cpu_quota: i64) -> Self {
        self.cpu_quota = Some(cpu_quota);
        self
    }

    /// Set number of CPUs
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::UpdateCommand;
    ///
    /// let cmd = UpdateCommand::new("my-container")
    ///     .cpus("1.5");
    /// ```
    #[must_use]
    pub fn cpus(mut self, cpus: impl Into<String>) -> Self {
        self.cpus = Some(cpus.into());
        self
    }

    /// Set CPU set
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::UpdateCommand;
    ///
    /// let cmd = UpdateCommand::new("my-container")
    ///     .cpuset_cpus("0,1");
    /// ```
    #[must_use]
    pub fn cpuset_cpus(mut self, cpuset_cpus: impl Into<String>) -> Self {
        self.cpuset_cpus = Some(cpuset_cpus.into());
        self
    }

    /// Set memory nodes
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::UpdateCommand;
    ///
    /// let cmd = UpdateCommand::new("my-container")
    ///     .cpuset_mems("0");
    /// ```
    #[must_use]
    pub fn cpuset_mems(mut self, cpuset_mems: impl Into<String>) -> Self {
        self.cpuset_mems = Some(cpuset_mems.into());
        self
    }

    /// Set block IO weight
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::UpdateCommand;
    ///
    /// let cmd = UpdateCommand::new("my-container")
    ///     .blkio_weight(500);
    /// ```
    #[must_use]
    pub fn blkio_weight(mut self, blkio_weight: u16) -> Self {
        self.blkio_weight = Some(blkio_weight);
        self
    }

    /// Set kernel memory limit
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::UpdateCommand;
    ///
    /// let cmd = UpdateCommand::new("my-container")
    ///     .kernel_memory("128m");
    /// ```
    #[must_use]
    pub fn kernel_memory(mut self, kernel_memory: impl Into<String>) -> Self {
        self.kernel_memory = Some(kernel_memory.into());
        self
    }

    /// Set restart policy
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::UpdateCommand;
    ///
    /// let cmd = UpdateCommand::new("my-container")
    ///     .restart("unless-stopped");
    /// ```
    #[must_use]
    pub fn restart(mut self, restart: impl Into<String>) -> Self {
        self.restart = Some(restart.into());
        self
    }

    /// Set PID limit
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::UpdateCommand;
    ///
    /// let cmd = UpdateCommand::new("my-container")
    ///     .pids_limit(100);
    /// ```
    #[must_use]
    pub fn pids_limit(mut self, pids_limit: i64) -> Self {
        self.pids_limit = Some(pids_limit);
        self
    }

    /// Execute the update command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - Any of the specified containers don't exist
    /// - Invalid resource limits are specified
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::UpdateCommand;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = UpdateCommand::new("my-container")
    ///     .memory("1g")
    ///     .cpu_shares(512)
    ///     .run()
    ///     .await?;
    ///
    /// if result.success() {
    ///     println!("Updated containers: {:?}", result.containers());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<UpdateResult> {
        let output = self.execute().await?;

        Ok(UpdateResult {
            output,
            containers: self.containers.clone(),
        })
    }
}

#[async_trait]
impl DockerCommand for UpdateCommand {
    type Output = CommandOutput;

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["update".to_string()];

        if let Some(ref memory) = self.memory {
            args.push("--memory".to_string());
            args.push(memory.clone());
        }

        if let Some(ref memory_reservation) = self.memory_reservation {
            args.push("--memory-reservation".to_string());
            args.push(memory_reservation.clone());
        }

        if let Some(ref memory_swap) = self.memory_swap {
            args.push("--memory-swap".to_string());
            args.push(memory_swap.clone());
        }

        if let Some(cpu_shares) = self.cpu_shares {
            args.push("--cpu-shares".to_string());
            args.push(cpu_shares.to_string());
        }

        if let Some(cpu_period) = self.cpu_period {
            args.push("--cpu-period".to_string());
            args.push(cpu_period.to_string());
        }

        if let Some(cpu_quota) = self.cpu_quota {
            args.push("--cpu-quota".to_string());
            args.push(cpu_quota.to_string());
        }

        if let Some(ref cpus) = self.cpus {
            args.push("--cpus".to_string());
            args.push(cpus.clone());
        }

        if let Some(ref cpuset_cpus) = self.cpuset_cpus {
            args.push("--cpuset-cpus".to_string());
            args.push(cpuset_cpus.clone());
        }

        if let Some(ref cpuset_mems) = self.cpuset_mems {
            args.push("--cpuset-mems".to_string());
            args.push(cpuset_mems.clone());
        }

        if let Some(blkio_weight) = self.blkio_weight {
            args.push("--blkio-weight".to_string());
            args.push(blkio_weight.to_string());
        }

        if let Some(ref kernel_memory) = self.kernel_memory {
            args.push("--kernel-memory".to_string());
            args.push(kernel_memory.clone());
        }

        if let Some(ref restart) = self.restart {
            args.push("--restart".to_string());
            args.push(restart.clone());
        }

        if let Some(pids_limit) = self.pids_limit {
            args.push("--pids-limit".to_string());
            args.push(pids_limit.to_string());
        }

        args.extend(self.containers.clone());
        args.extend(self.executor.raw_args.clone());
        args
    }

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    async fn execute(&self) -> Result<Self::Output> {
        if self.containers.is_empty() {
            return Err(crate::error::Error::invalid_config(
                "No containers specified for update",
            ));
        }

        let args = self.build_command_args();
        let command_name = args[0].clone();
        let command_args = args[1..].to_vec();
        self.executor
            .execute_command(&command_name, command_args)
            .await
    }
}

/// Result from the update command
#[derive(Debug, Clone)]
pub struct UpdateResult {
    /// Raw command output
    pub output: CommandOutput,
    /// Containers that were updated
    pub containers: Vec<String>,
}

impl UpdateResult {
    /// Check if the update was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the updated container names
    #[must_use]
    pub fn containers(&self) -> &[String] {
        &self.containers
    }

    /// Get the raw command output
    #[must_use]
    pub fn output(&self) -> &CommandOutput {
        &self.output
    }

    /// Get container count
    #[must_use]
    pub fn container_count(&self) -> usize {
        self.containers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_single_container() {
        let cmd = UpdateCommand::new("test-container");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["update", "test-container"]);
    }

    #[test]
    fn test_update_multiple_containers() {
        let cmd = UpdateCommand::new_multiple(vec!["web", "db", "cache"]);
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["update", "web", "db", "cache"]);
    }

    #[test]
    fn test_update_add_container() {
        let cmd = UpdateCommand::new("web").container("db").container("cache");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["update", "web", "db", "cache"]);
    }

    #[test]
    fn test_update_memory_options() {
        let cmd = UpdateCommand::new("test-container")
            .memory("512m")
            .memory_reservation("256m")
            .memory_swap("1g");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec![
                "update",
                "--memory",
                "512m",
                "--memory-reservation",
                "256m",
                "--memory-swap",
                "1g",
                "test-container"
            ]
        );
    }

    #[test]
    fn test_update_cpu_options() {
        let cmd = UpdateCommand::new("test-container")
            .cpu_shares(512)
            .cpu_period(100_000)
            .cpu_quota(50000)
            .cpus("1.5")
            .cpuset_cpus("0,1")
            .cpuset_mems("0");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec![
                "update",
                "--cpu-shares",
                "512",
                "--cpu-period",
                "100000",
                "--cpu-quota",
                "50000",
                "--cpus",
                "1.5",
                "--cpuset-cpus",
                "0,1",
                "--cpuset-mems",
                "0",
                "test-container"
            ]
        );
    }

    #[test]
    fn test_update_all_options() {
        let cmd = UpdateCommand::new("test-container")
            .memory("1g")
            .cpu_shares(1024)
            .blkio_weight(500)
            .kernel_memory("128m")
            .restart("unless-stopped")
            .pids_limit(100);
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec![
                "update",
                "--memory",
                "1g",
                "--cpu-shares",
                "1024",
                "--blkio-weight",
                "500",
                "--kernel-memory",
                "128m",
                "--restart",
                "unless-stopped",
                "--pids-limit",
                "100",
                "test-container"
            ]
        );
    }

    #[test]
    fn test_update_result() {
        let result = UpdateResult {
            output: CommandOutput {
                stdout: "test-container".to_string(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
            },
            containers: vec!["test-container".to_string()],
        };

        assert!(result.success());
        assert_eq!(result.containers(), &["test-container"]);
        assert_eq!(result.container_count(), 1);
    }
}
