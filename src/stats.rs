//! Docker statistics monitoring module.
//!
//! This module provides comprehensive Docker statistics monitoring capabilities including:
//! - Real-time container resource usage monitoring
//! - System-wide Docker statistics
//! - Performance metrics collection and aggregation
//! - Resource usage tracking over time
//! - Custom metrics and monitoring helpers
//!
//! # Example
//!
//! ```rust,no_run
//! use docker_wrapper::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), DockerError> {
//!     let client = DockerClient::new().await?;
//!     let stats_monitor = client.stats();
//!
//!     // Get current stats for a container
//!     let container_id = ContainerId::new("my-container").unwrap();
//!     let stats = stats_monitor.get_stats(&container_id).await?;
//!
//!     println!("CPU Usage: {:.2}%", stats.cpu_usage_percent());
//!     println!("Memory Usage: {} MB", stats.memory_usage_mb());
//!     println!("Network I/O: {} bytes in, {} bytes out",
//!         stats.network_rx_bytes(), stats.network_tx_bytes());
//!
//!     // Stream real-time stats
//!     let mut stream = stats_monitor.stream_stats(&container_id).await?;
//!     while let Some(stats) = stream.next().await {
//!         let stats = stats?;
//!         println!("CPU: {:.1}%, Memory: {} MB",
//!             stats.cpu_usage_percent(),
//!             stats.memory_usage_mb()
//!         );
//!     }
//!
//!     Ok(())
//! }
//! ```

use crate::client::DockerClient;
use crate::errors::{DockerError, DockerResult};
use crate::executor::ExecutionConfig;
use crate::types::ContainerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use tokio::process::Child;
use tokio::sync::mpsc;
use tokio_stream::{Stream, wrappers::ReceiverStream};

/// Container resource statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStats {
    /// Container ID
    #[serde(rename = "id")]
    pub container_id: String,

    /// Container name
    #[serde(rename = "name")]
    pub name: String,

    /// CPU statistics
    #[serde(rename = "cpu_stats")]
    pub cpu_stats: CpuStats,

    /// Previous CPU statistics (for calculating usage)
    #[serde(rename = "precpu_stats")]
    pub precpu_stats: CpuStats,

    /// Memory statistics
    #[serde(rename = "memory_stats")]
    pub memory_stats: MemoryStats,

    /// Block I/O statistics
    #[serde(rename = "blkio_stats")]
    pub blkio_stats: BlkioStats,

    /// Network statistics
    #[serde(rename = "networks")]
    pub networks: HashMap<String, NetworkStats>,

    /// PIDs statistics
    #[serde(rename = "pids_stats")]
    pub pids_stats: PidsStats,

    /// Number of processes/threads
    #[serde(rename = "num_procs")]
    pub num_procs: u32,

    /// Timestamp when stats were collected
    #[serde(rename = "read")]
    pub read: String,

    /// Previous timestamp
    #[serde(rename = "preread")]
    pub preread: String,
}

impl ContainerStats {
    /// Calculate CPU usage percentage
    #[must_use]
    pub fn cpu_usage_percent(&self) -> f64 {
        let cpu_delta = self.cpu_stats.cpu_usage.total_usage as f64
            - self.precpu_stats.cpu_usage.total_usage as f64;
        let system_delta =
            self.cpu_stats.system_cpu_usage as f64 - self.precpu_stats.system_cpu_usage as f64;

        if system_delta > 0.0 && cpu_delta > 0.0 {
            let num_cpus = self.cpu_stats.cpu_usage.percpu_usage.len() as f64;
            (cpu_delta / system_delta) * num_cpus * 100.0
        } else {
            0.0
        }
    }

    /// Get memory usage in bytes
    #[must_use]
    pub fn memory_usage_bytes(&self) -> u64 {
        self.memory_stats
            .usage
            .saturating_sub(*self.memory_stats.stats.get("cache").unwrap_or(&0))
    }

    /// Get memory usage in MB
    #[must_use]
    pub fn memory_usage_mb(&self) -> f64 {
        self.memory_usage_bytes() as f64 / 1_048_576.0
    }

    /// Get memory usage percentage
    #[must_use]
    pub fn memory_usage_percent(&self) -> f64 {
        if self.memory_stats.limit > 0 {
            (self.memory_usage_bytes() as f64 / self.memory_stats.limit as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Get total network RX bytes
    #[must_use]
    pub fn network_rx_bytes(&self) -> u64 {
        self.networks.values().map(|n| n.rx_bytes).sum()
    }

    /// Get total network TX bytes
    #[must_use]
    pub fn network_tx_bytes(&self) -> u64 {
        self.networks.values().map(|n| n.tx_bytes).sum()
    }

    /// Get total network RX packets
    #[must_use]
    pub fn network_rx_packets(&self) -> u64 {
        self.networks.values().map(|n| n.rx_packets).sum()
    }

    /// Get total network TX packets
    #[must_use]
    pub fn network_tx_packets(&self) -> u64 {
        self.networks.values().map(|n| n.tx_packets).sum()
    }

    /// Get total block I/O read bytes
    #[must_use]
    pub fn blkio_read_bytes(&self) -> u64 {
        self.blkio_stats
            .io_service_bytes_recursive
            .iter()
            .filter(|entry| entry.op == "Read")
            .map(|entry| entry.value)
            .sum()
    }

    /// Get total block I/O write bytes
    #[must_use]
    pub fn blkio_write_bytes(&self) -> u64 {
        self.blkio_stats
            .io_service_bytes_recursive
            .iter()
            .filter(|entry| entry.op == "Write")
            .map(|entry| entry.value)
            .sum()
    }

    /// Get number of PIDs/processes
    #[must_use]
    pub fn pids_current(&self) -> u64 {
        self.pids_stats.current
    }

    /// Check if container is using high CPU (> 80%)
    #[must_use]
    pub fn is_high_cpu_usage(&self) -> bool {
        self.cpu_usage_percent() > 80.0
    }

    /// Check if container is using high memory (> 80%)
    #[must_use]
    pub fn is_high_memory_usage(&self) -> bool {
        self.memory_usage_percent() > 80.0
    }

    /// Get formatted summary string
    pub fn summary(&self) -> String {
        format!(
            "{}: CPU {:.1}%, Memory {:.1} MB ({:.1}%), Network {}/{} bytes, Block I/O {}/{} bytes",
            self.name,
            self.cpu_usage_percent(),
            self.memory_usage_mb(),
            self.memory_usage_percent(),
            self.network_rx_bytes(),
            self.network_tx_bytes(),
            self.blkio_read_bytes(),
            self.blkio_write_bytes()
        )
    }
}

/// CPU usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStats {
    /// CPU usage breakdown
    #[serde(rename = "cpu_usage")]
    pub cpu_usage: CpuUsage,

    /// System CPU usage
    #[serde(rename = "system_cpu_usage")]
    pub system_cpu_usage: u64,

    /// Number of online CPUs
    #[serde(rename = "online_cpus")]
    pub online_cpus: Option<u32>,

    /// Throttling data
    #[serde(rename = "throttling_data")]
    pub throttling_data: ThrottlingData,
}

/// Detailed CPU usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuUsage {
    /// Total CPU usage
    #[serde(rename = "total_usage")]
    pub total_usage: u64,

    /// Per-CPU usage
    #[serde(rename = "percpu_usage")]
    pub percpu_usage: Vec<u64>,

    /// Usage in kernel mode
    #[serde(rename = "usage_in_kernelmode")]
    pub usage_in_kernelmode: u64,

    /// Usage in user mode
    #[serde(rename = "usage_in_usermode")]
    pub usage_in_usermode: u64,
}

/// CPU throttling data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrottlingData {
    /// Number of periods
    #[serde(rename = "periods")]
    pub periods: u64,

    /// Number of throttled periods
    #[serde(rename = "throttled_periods")]
    pub throttled_periods: u64,

    /// Total throttled time
    #[serde(rename = "throttled_time")]
    pub throttled_time: u64,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Current memory usage
    #[serde(rename = "usage")]
    pub usage: u64,

    /// Maximum memory usage
    #[serde(rename = "max_usage")]
    pub max_usage: u64,

    /// Memory limit
    #[serde(rename = "limit")]
    pub limit: u64,

    /// Detailed memory statistics
    #[serde(rename = "stats")]
    pub stats: HashMap<String, u64>,
}

/// Block I/O statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlkioStats {
    /// I/O service bytes recursive
    #[serde(rename = "io_service_bytes_recursive")]
    pub io_service_bytes_recursive: Vec<BlkioEntry>,

    /// I/O serviced recursive
    #[serde(rename = "io_serviced_recursive")]
    pub io_serviced_recursive: Vec<BlkioEntry>,

    /// I/O queue recursive
    #[serde(rename = "io_queue_recursive")]
    pub io_queue_recursive: Vec<BlkioEntry>,

    /// I/O service time recursive
    #[serde(rename = "io_service_time_recursive")]
    pub io_service_time_recursive: Vec<BlkioEntry>,
}

/// Block I/O entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlkioEntry {
    /// Major device number
    #[serde(rename = "major")]
    pub major: u64,

    /// Minor device number
    #[serde(rename = "minor")]
    pub minor: u64,

    /// Operation type (Read, Write, etc.)
    #[serde(rename = "op")]
    pub op: String,

    /// Value
    #[serde(rename = "value")]
    pub value: u64,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    /// Bytes received
    #[serde(rename = "rx_bytes")]
    pub rx_bytes: u64,

    /// Packets received
    #[serde(rename = "rx_packets")]
    pub rx_packets: u64,

    /// RX errors
    #[serde(rename = "rx_errors")]
    pub rx_errors: u64,

    /// RX dropped
    #[serde(rename = "rx_dropped")]
    pub rx_dropped: u64,

    /// Bytes transmitted
    #[serde(rename = "tx_bytes")]
    pub tx_bytes: u64,

    /// Packets transmitted
    #[serde(rename = "tx_packets")]
    pub tx_packets: u64,

    /// TX errors
    #[serde(rename = "tx_errors")]
    pub tx_errors: u64,

    /// TX dropped
    #[serde(rename = "tx_dropped")]
    pub tx_dropped: u64,
}

/// PIDs statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidsStats {
    /// Current number of PIDs
    #[serde(rename = "current")]
    pub current: u64,

    /// PID limit
    #[serde(rename = "limit")]
    pub limit: Option<u64>,
}

/// System-wide Docker statistics
#[derive(Debug, Clone)]
pub struct SystemStats {
    /// Total containers
    pub total_containers: u32,

    /// Running containers
    pub running_containers: u32,

    /// Paused containers
    pub paused_containers: u32,

    /// Stopped containers
    pub stopped_containers: u32,

    /// Total images
    pub images: u32,

    /// System memory info
    pub memory_info: SystemMemoryInfo,

    /// CPU info
    pub cpu_info: SystemCpuInfo,

    /// Disk usage
    pub disk_usage: SystemDiskUsage,
}

/// System memory information
#[derive(Debug, Clone)]
pub struct SystemMemoryInfo {
    /// Total system memory
    pub total: u64,

    /// Available memory
    pub available: u64,

    /// Used memory
    pub used: u64,

    /// Memory used by Docker
    pub docker_used: u64,
}

/// System CPU information
#[derive(Debug, Clone)]
pub struct SystemCpuInfo {
    /// Number of CPUs
    pub cpus: u32,

    /// CPU architecture
    pub architecture: String,

    /// CPU model
    pub model: String,
}

/// System disk usage
#[derive(Debug, Clone)]
pub struct SystemDiskUsage {
    /// Docker root directory size
    pub docker_size: u64,

    /// Available disk space
    pub available: u64,

    /// Total disk space
    pub total: u64,
}

/// Statistics collection options
#[derive(Debug, Clone, Default)]
pub struct StatsOptions {
    /// Stream stats continuously
    pub stream: bool,

    /// One-shot stats (no streaming)
    pub no_stream: bool,

    /// Include all containers (not just running)
    pub all: bool,
}

impl StatsOptions {
    /// Create new stats options
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable streaming
    #[must_use]
    pub fn stream(mut self) -> Self {
        self.stream = true;
        self.no_stream = false;
        self
    }

    /// Disable streaming (one-shot)
    #[must_use]
    pub fn no_stream(mut self) -> Self {
        self.no_stream = true;
        self.stream = false;
        self
    }

    /// Include all containers
    #[must_use]
    pub fn all(mut self) -> Self {
        self.all = true;
        self
    }
}

/// Statistics stream handle
pub struct StatsStream {
    receiver: mpsc::Receiver<DockerResult<ContainerStats>>,
    _child: Child,
}

impl StatsStream {
    /// Get the next stats from the stream
    pub async fn next(&mut self) -> Option<DockerResult<ContainerStats>> {
        self.receiver.recv().await
    }

    /// Convert to a tokio Stream
    pub fn into_stream(self) -> ReceiverStream<DockerResult<ContainerStats>> {
        ReceiverStream::new(self.receiver)
    }
}

impl Stream for StatsStream {
    type Item = DockerResult<ContainerStats>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}

/// Statistics aggregator for collecting metrics over time
#[derive(Debug, Clone)]
pub struct StatsAggregator {
    /// Container ID
    pub container_id: ContainerId,

    /// Historical data points
    pub history: Vec<(SystemTime, ContainerStats)>,

    /// Maximum history size
    pub max_history: usize,

    /// Aggregation start time
    pub start_time: SystemTime,
}

impl StatsAggregator {
    /// Create new stats aggregator
    #[must_use]
    pub fn new(container_id: ContainerId, max_history: usize) -> Self {
        Self {
            container_id,
            history: Vec::new(),
            max_history,
            start_time: SystemTime::now(),
        }
    }

    /// Add stats data point
    pub fn add_stats(&mut self, stats: ContainerStats) {
        let timestamp = SystemTime::now();
        self.history.push((timestamp, stats));

        // Keep only the most recent data points
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Get average CPU usage over time window
    #[must_use]
    pub fn avg_cpu_usage(&self, window: Duration) -> f64 {
        let cutoff = SystemTime::now() - window;
        let values: Vec<f64> = self
            .history
            .iter()
            .filter(|(time, _)| *time >= cutoff)
            .map(|(_, stats)| stats.cpu_usage_percent())
            .collect();

        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<f64>() / values.len() as f64
        }
    }

    /// Get average memory usage over time window
    #[must_use]
    pub fn avg_memory_usage(&self, window: Duration) -> f64 {
        let cutoff = SystemTime::now() - window;
        let values: Vec<f64> = self
            .history
            .iter()
            .filter(|(time, _)| *time >= cutoff)
            .map(|(_, stats)| stats.memory_usage_mb())
            .collect();

        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<f64>() / values.len() as f64
        }
    }

    /// Get peak CPU usage over time window
    #[must_use]
    pub fn peak_cpu_usage(&self, window: Duration) -> f64 {
        let cutoff = SystemTime::now() - window;
        self.history
            .iter()
            .filter(|(time, _)| *time >= cutoff)
            .map(|(_, stats)| stats.cpu_usage_percent())
            .fold(0.0, f64::max)
    }

    /// Get peak memory usage over time window
    #[must_use]
    pub fn peak_memory_usage(&self, window: Duration) -> f64 {
        let cutoff = SystemTime::now() - window;
        self.history
            .iter()
            .filter(|(time, _)| *time >= cutoff)
            .map(|(_, stats)| stats.memory_usage_mb())
            .fold(0.0, f64::max)
    }

    /// Get total network I/O over time window
    pub fn total_network_io(&self, window: Duration) -> (u64, u64) {
        let cutoff = SystemTime::now() - window;
        let data: Vec<_> = self
            .history
            .iter()
            .filter(|(time, _)| *time >= cutoff)
            .map(|(_, stats)| (stats.network_rx_bytes(), stats.network_tx_bytes()))
            .collect();

        if data.is_empty() {
            (0, 0)
        } else {
            let first = data.first().unwrap();
            let last = data.last().unwrap();
            (last.0 - first.0, last.1 - first.1)
        }
    }

    /// Generate summary report
    pub fn summary_report(&self, window: Duration) -> String {
        format!(
            "Stats Summary for {} (last {}s):\n  Avg CPU: {:.1}%\n  Peak CPU: {:.1}%\n  Avg Memory: {:.1} MB\n  Peak Memory: {:.1} MB\n  Network I/O: {} RX / {} TX bytes",
            self.container_id,
            window.as_secs(),
            self.avg_cpu_usage(window),
            self.peak_cpu_usage(window),
            self.avg_memory_usage(window),
            self.peak_memory_usage(window),
            self.total_network_io(window).0,
            self.total_network_io(window).1
        )
    }
}

/// Docker statistics manager
pub struct StatsManager<'a> {
    client: &'a DockerClient,
}

impl<'a> StatsManager<'a> {
    /// Create a new stats manager
    pub fn new(client: &'a DockerClient) -> Self {
        Self { client }
    }

    /// Get current stats for a container (one-shot)
    pub async fn get_stats(&self, container_id: &ContainerId) -> DockerResult<ContainerStats> {
        let args = vec![
            "stats".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "--no-stream".to_string(),
            container_id.as_str().to_string(),
        ];

        let output = self
            .client
            .executor()
            .execute(&args, Some(ExecutionConfig::default()))
            .await?;

        if !output.success {
            return Err(DockerError::CommandFailed {
                command: format!("docker {}", args.join(" ")),
                exit_code: output.exit_code,
                stdout: output.stdout.clone(),
                stderr: output.stderr.clone(),
            });
        }

        let stdout = &output.stdout;
        let lines: Vec<&str> = stdout.lines().collect();

        if let Some(line) = lines.first() {
            if !line.trim().is_empty() {
                return serde_json::from_str(line)
                    .map_err(|e| DockerError::ParseError(format!("Invalid stats JSON: {}", e)));
            }
        }

        Err(DockerError::NotFound {
            message: format!("No stats found for container: {}", container_id),
        })
    }

    /// Stream real-time stats for a container
    pub async fn stream_stats(&self, container_id: &ContainerId) -> DockerResult<StatsStream> {
        let args = vec![
            "stats".to_string(),
            "--format".to_string(),
            "json".to_string(),
            container_id.as_str().to_string(),
        ];

        let child = self
            .client
            .executor()
            .execute_streaming(&args, Some(ExecutionConfig::default()))
            .await?;

        let mut stdout = child.stdout;

        let (tx, rx) = mpsc::channel(100);

        // Spawn task to process stats
        tokio::spawn(async move {
            while let Some(line_result) = stdout.recv().await {
                match line_result {
                    Ok(line) => {
                        if line.trim().is_empty() {
                            continue;
                        }

                        let result = serde_json::from_str::<ContainerStats>(&line).map_err(|e| {
                            DockerError::ParseError(format!("Invalid stats JSON: {}", e))
                        });

                        if tx.send(result).await.is_err() {
                            break; // Receiver dropped
                        }
                    }
                    Err(e) => {
                        if tx.send(Err(e)).await.is_err() {
                            break; // Receiver dropped
                        }
                    }
                }
            }
        });

        Ok(StatsStream {
            receiver: rx,
            _child: child.child,
        })
    }

    /// Get stats for all containers
    pub async fn get_all_stats(&self, options: StatsOptions) -> DockerResult<Vec<ContainerStats>> {
        let mut args = vec![
            "stats".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "--no-stream".to_string(),
        ];

        if options.all {
            args.push("--all".to_string());
        }

        let output = self
            .client
            .executor()
            .execute(&args, Some(ExecutionConfig::default()))
            .await?;

        if !output.success {
            return Err(DockerError::CommandFailed {
                command: format!("docker {}", args.join(" ")),
                exit_code: output.exit_code,
                stdout: output.stdout.clone(),
                stderr: output.stderr.clone(),
            });
        }

        let stdout = &output.stdout;
        let mut stats = Vec::new();

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<ContainerStats>(line) {
                Ok(stat) => stats.push(stat),
                Err(e) => {
                    log::warn!("Failed to parse stats JSON: {} - {}", e, line);
                }
            }
        }

        Ok(stats)
    }

    /// Monitor stats with callback
    pub async fn monitor_stats<F>(
        &self,
        container_id: &ContainerId,
        mut callback: F,
    ) -> DockerResult<()>
    where
        F: FnMut(ContainerStats) -> bool + Send + 'static,
    {
        let mut stream = self.stream_stats(container_id).await?;

        while let Some(stats_result) = stream.next().await {
            match stats_result {
                Ok(stats) => {
                    if !callback(stats) {
                        break; // Stop if callback returns false
                    }
                }
                Err(e) => {
                    log::warn!("Stats monitoring error: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Create a stats aggregator for historical tracking
    pub fn create_aggregator(
        &self,
        container_id: ContainerId,
        max_history: usize,
    ) -> StatsAggregator {
        StatsAggregator::new(container_id, max_history)
    }

    /// Monitor and aggregate stats over time
    pub async fn monitor_with_aggregation(
        &self,
        container_id: &ContainerId,
        duration: Duration,
        max_history: usize,
    ) -> DockerResult<StatsAggregator> {
        let mut aggregator = self.create_aggregator(container_id.clone(), max_history);
        let mut stream = self.stream_stats(container_id).await?;

        let start_time = SystemTime::now();

        while let Some(stats_result) = stream.next().await {
            match stats_result {
                Ok(stats) => {
                    aggregator.add_stats(stats);

                    // Check if we've collected for the requested duration
                    if SystemTime::now()
                        .duration_since(start_time)
                        .unwrap_or_default()
                        >= duration
                    {
                        break;
                    }
                }
                Err(e) => {
                    log::warn!("Stats aggregation error: {}", e);
                }
            }
        }

        Ok(aggregator)
    }

    /// Get system-wide Docker statistics
    pub async fn get_system_stats(&self) -> DockerResult<SystemStats> {
        // Get Docker system info
        let info_output = self
            .client
            .executor()
            .execute(
                &["system".to_string(), "info".to_string()],
                Some(ExecutionConfig::default()),
            )
            .await?;

        // Get container count
        let ps_output = self
            .client
            .executor()
            .execute(
                &["ps".to_string(), "-a".to_string(), "-q".to_string()],
                Some(ExecutionConfig::default()),
            )
            .await?;

        let total_containers = ps_output
            .stdout
            .lines()
            .filter(|l| !l.trim().is_empty())
            .count() as u32;

        // Get running containers
        let running_output = self
            .client
            .executor()
            .execute(
                &["ps".to_string(), "-q".to_string()],
                Some(ExecutionConfig::default()),
            )
            .await?;

        let running_containers = running_output
            .stdout
            .lines()
            .filter(|l| !l.trim().is_empty())
            .count() as u32;

        // Get images count
        let images_output = self
            .client
            .executor()
            .execute(
                &["images".to_string(), "-q".to_string()],
                Some(ExecutionConfig::default()),
            )
            .await?;

        let images = images_output
            .stdout
            .lines()
            .filter(|l| !l.trim().is_empty())
            .count() as u32;

        // Parse system info for additional details
        let info_text = &info_output.stdout;
        let mut memory_total = 0u64;
        let mut cpus = 0u32;

        for line in info_text.lines() {
            if line.contains("Total Memory:") {
                if let Some(memory_str) = line.split(':').nth(1) {
                    // Simple parsing - in production would be more robust
                    if let Ok(mem) = memory_str.trim().replace("GiB", "").parse::<f64>() {
                        memory_total = (mem * 1024.0 * 1024.0 * 1024.0) as u64;
                    }
                }
            } else if line.contains("CPUs:") {
                if let Some(cpu_str) = line.split(':').nth(1) {
                    if let Ok(cpu_count) = cpu_str.trim().parse::<u32>() {
                        cpus = cpu_count;
                    }
                }
            }
        }

        Ok(SystemStats {
            total_containers,
            running_containers,
            paused_containers: 0, // Would need additional parsing
            stopped_containers: total_containers - running_containers,
            images,
            memory_info: SystemMemoryInfo {
                total: memory_total,
                available: 0,   // Would need system call
                used: 0,        // Would need system call
                docker_used: 0, // Would need additional calculation
            },
            cpu_info: SystemCpuInfo {
                cpus,
                architecture: "x86_64".to_string(), // Would parse from info
                model: "Unknown".to_string(),       // Would parse from info
            },
            disk_usage: SystemDiskUsage {
                docker_size: 0, // Would need `docker system df`
                available: 0,   // Would need system call
                total: 0,       // Would need system call
            },
        })
    }

    /// Wait for container to reach CPU threshold
    pub async fn wait_for_cpu_threshold(
        &self,
        container_id: &ContainerId,
        threshold: f64,
        above: bool,
        timeout: Duration,
    ) -> DockerResult<ContainerStats> {
        let mut stream = self.stream_stats(container_id).await?;
        let start_time = SystemTime::now();

        while let Some(stats_result) = stream.next().await {
            match stats_result {
                Ok(stats) => {
                    let cpu_usage = stats.cpu_usage_percent();

                    if (above && cpu_usage >= threshold) || (!above && cpu_usage <= threshold) {
                        return Ok(stats);
                    }

                    if SystemTime::now()
                        .duration_since(start_time)
                        .unwrap_or_default()
                        >= timeout
                    {
                        return Err(DockerError::Timeout {
                            message: format!(
                                "Timeout waiting for CPU usage to {} {}%",
                                if above { "exceed" } else { "drop below" },
                                threshold
                            ),
                        });
                    }
                }
                Err(e) => {
                    log::warn!("Stats monitoring error: {}", e);
                }
            }
        }

        Err(DockerError::Timeout {
            message: "Stats stream ended before threshold was reached".to_string(),
        })
    }

    /// Wait for container to reach memory threshold
    pub async fn wait_for_memory_threshold(
        &self,
        container_id: &ContainerId,
        threshold_mb: f64,
        above: bool,
        timeout: Duration,
    ) -> DockerResult<ContainerStats> {
        let mut stream = self.stream_stats(container_id).await?;
        let start_time = SystemTime::now();

        while let Some(stats_result) = stream.next().await {
            match stats_result {
                Ok(stats) => {
                    let memory_usage = stats.memory_usage_mb();

                    if (above && memory_usage >= threshold_mb)
                        || (!above && memory_usage <= threshold_mb)
                    {
                        return Ok(stats);
                    }

                    if SystemTime::now()
                        .duration_since(start_time)
                        .unwrap_or_default()
                        >= timeout
                    {
                        return Err(DockerError::Timeout {
                            message: format!(
                                "Timeout waiting for memory usage to {} {} MB",
                                if above { "exceed" } else { "drop below" },
                                threshold_mb
                            ),
                        });
                    }
                }
                Err(e) => {
                    log::warn!("Stats monitoring error: {}", e);
                }
            }
        }

        Err(DockerError::Timeout {
            message: "Stats stream ended before threshold was reached".to_string(),
        })
    }

    /// Check if container is healthy based on resource usage
    pub async fn is_container_healthy(&self, container_id: &ContainerId) -> DockerResult<bool> {
        let stats = self.get_stats(container_id).await?;

        // Container is considered unhealthy if:
        // - CPU usage > 95% (likely stuck)
        // - Memory usage > 95% (likely out of memory)
        // - Too many processes (> 1000, possible fork bomb)

        Ok(!(stats.cpu_usage_percent() > 95.0
            || stats.memory_usage_percent() > 95.0
            || stats.pids_current() > 1000))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_options_builder() {
        let options = StatsOptions::new().stream().all();

        assert!(options.stream);
        assert!(!options.no_stream);
        assert!(options.all);

        let options = StatsOptions::new().no_stream();

        assert!(!options.stream);
        assert!(options.no_stream);
    }

    #[test]
    fn test_stats_aggregator() {
        let container_id = ContainerId::new_unchecked("test-container".to_string());
        let mut aggregator = StatsAggregator::new(container_id, 100);

        // Create mock stats
        let mock_stats = ContainerStats {
            container_id: "test-container".to_string(),
            name: "test".to_string(),
            cpu_stats: CpuStats {
                cpu_usage: CpuUsage {
                    total_usage: 1_000_000_000,
                    percpu_usage: vec![500_000_000, 500_000_000],
                    usage_in_kernelmode: 200_000_000,
                    usage_in_usermode: 800_000_000,
                },
                system_cpu_usage: 10_000_000_000,
                online_cpus: Some(2),
                throttling_data: ThrottlingData {
                    periods: 0,
                    throttled_periods: 0,
                    throttled_time: 0,
                },
            },
            precpu_stats: CpuStats {
                cpu_usage: CpuUsage {
                    total_usage: 500_000_000,
                    percpu_usage: vec![250_000_000, 250_000_000],
                    usage_in_kernelmode: 100_000_000,
                    usage_in_usermode: 400_000_000,
                },
                system_cpu_usage: 9_000_000_000,
                online_cpus: Some(2),
                throttling_data: ThrottlingData {
                    periods: 0,
                    throttled_periods: 0,
                    throttled_time: 0,
                },
            },
            memory_stats: MemoryStats {
                usage: 100 * 1024 * 1024, // 100 MB
                max_usage: 120 * 1024 * 1024,
                limit: 512 * 1024 * 1024, // 512 MB limit
                stats: HashMap::new(),
            },
            blkio_stats: BlkioStats {
                io_service_bytes_recursive: vec![],
                io_serviced_recursive: vec![],
                io_queue_recursive: vec![],
                io_service_time_recursive: vec![],
            },
            networks: HashMap::new(),
            pids_stats: PidsStats {
                current: 10,
                limit: Some(1024),
            },
            num_procs: 10,
            read: "2023-01-01T00:00:00Z".to_string(),
            preread: "2023-01-01T00:00:00Z".to_string(),
        };

        aggregator.add_stats(mock_stats.clone());
        assert_eq!(aggregator.history.len(), 1);

        // Test CPU calculation
        let cpu_usage = mock_stats.cpu_usage_percent();
        assert!(cpu_usage > 0.0);

        // Test memory calculation
        let memory_mb = mock_stats.memory_usage_mb();
        assert!((memory_mb - 100.0).abs() < 1.0); // Should be ~100 MB

        let memory_percent = mock_stats.memory_usage_percent();
        assert!((memory_percent - 19.53125).abs() < 0.1); // ~19.5% of 512MB
    }

    #[test]
    fn test_container_stats_calculations() {
        // Create comprehensive test stats
        let mut networks = HashMap::new();
        networks.insert(
            "eth0".to_string(),
            NetworkStats {
                rx_bytes: 1024 * 1024, // 1 MB
                rx_packets: 1000,
                rx_errors: 0,
                rx_dropped: 0,
                tx_bytes: 2 * 1024 * 1024, // 2 MB
                tx_packets: 2000,
                tx_errors: 0,
                tx_dropped: 0,
            },
        );

        let stats = ContainerStats {
            container_id: "test".to_string(),
            name: "test-container".to_string(),
            cpu_stats: CpuStats {
                cpu_usage: CpuUsage {
                    total_usage: 2_000_000_000,
                    percpu_usage: vec![1_000_000_000, 1_000_000_000],
                    usage_in_kernelmode: 400_000_000,
                    usage_in_usermode: 1_600_000_000,
                },
                system_cpu_usage: 20_000_000_000,
                online_cpus: Some(2),
                throttling_data: ThrottlingData {
                    periods: 100,
                    throttled_periods: 10,
                    throttled_time: 1_000_000,
                },
            },
            precpu_stats: CpuStats {
                cpu_usage: CpuUsage {
                    total_usage: 1_000_000_000,
                    percpu_usage: vec![500_000_000, 500_000_000],
                    usage_in_kernelmode: 200_000_000,
                    usage_in_usermode: 800_000_000,
                },
                system_cpu_usage: 10_000_000_000,
                online_cpus: Some(2),
                throttling_data: ThrottlingData {
                    periods: 0,
                    throttled_periods: 0,
                    throttled_time: 0,
                },
            },
            memory_stats: MemoryStats {
                usage: 256 * 1024 * 1024, // 256 MB
                max_usage: 300 * 1024 * 1024,
                limit: 1024 * 1024 * 1024, // 1 GB
                stats: {
                    let mut map = HashMap::new();
                    map.insert("cache".to_string(), 50 * 1024 * 1024); // 50 MB cache
                    map
                },
            },
            blkio_stats: BlkioStats {
                io_service_bytes_recursive: vec![
                    BlkioEntry {
                        major: 8,
                        minor: 0,
                        op: "Read".to_string(),
                        value: 10 * 1024 * 1024, // 10 MB read
                    },
                    BlkioEntry {
                        major: 8,
                        minor: 0,
                        op: "Write".to_string(),
                        value: 5 * 1024 * 1024, // 5 MB write
                    },
                ],
                io_serviced_recursive: vec![],
                io_queue_recursive: vec![],
                io_service_time_recursive: vec![],
            },
            networks,
            pids_stats: PidsStats {
                current: 25,
                limit: Some(1024),
            },
            num_procs: 25,
            read: "2023-01-01T00:00:01Z".to_string(),
            preread: "2023-01-01T00:00:00Z".to_string(),
        };

        // Test all calculation methods
        let cpu_percent = stats.cpu_usage_percent();
        assert!(cpu_percent > 0.0);

        let memory_bytes = stats.memory_usage_bytes();
        assert_eq!(memory_bytes, (256 - 50) * 1024 * 1024); // 256MB - 50MB cache

        let memory_mb = stats.memory_usage_mb();
        assert!((memory_mb - 206.0).abs() < 1.0); // Should be ~206 MB

        let memory_percent = stats.memory_usage_percent();
        assert!((memory_percent - 20.11719).abs() < 0.1); // ~20% of 1GB

        assert_eq!(stats.network_rx_bytes(), 1024 * 1024);
        assert_eq!(stats.network_tx_bytes(), 2 * 1024 * 1024);
        assert_eq!(stats.network_rx_packets(), 1000);
        assert_eq!(stats.network_tx_packets(), 2000);

        assert_eq!(stats.blkio_read_bytes(), 10 * 1024 * 1024);
        assert_eq!(stats.blkio_write_bytes(), 5 * 1024 * 1024);

        assert_eq!(stats.pids_current(), 25);

        assert!(!stats.is_high_cpu_usage()); // < 80%
        assert!(!stats.is_high_memory_usage()); // < 80%

        let summary = stats.summary();
        assert!(summary.contains("test-container"));
        assert!(summary.contains("CPU"));
        assert!(summary.contains("Memory"));
    }

    #[test]
    fn test_system_stats_creation() {
        let system_stats = SystemStats {
            total_containers: 10,
            running_containers: 7,
            paused_containers: 1,
            stopped_containers: 2,
            images: 15,
            memory_info: SystemMemoryInfo {
                total: 8 * 1024 * 1024 * 1024,     // 8 GB
                available: 4 * 1024 * 1024 * 1024, // 4 GB
                used: 4 * 1024 * 1024 * 1024,      // 4 GB
                docker_used: 1024 * 1024 * 1024,   // 1 GB
            },
            cpu_info: SystemCpuInfo {
                cpus: 4,
                architecture: "x86_64".to_string(),
                model: "Intel i7".to_string(),
            },
            disk_usage: SystemDiskUsage {
                docker_size: 10 * 1024 * 1024 * 1024, // 10 GB
                available: 100 * 1024 * 1024 * 1024,  // 100 GB
                total: 500 * 1024 * 1024 * 1024,      // 500 GB
            },
        };

        assert_eq!(system_stats.total_containers, 10);
        assert_eq!(system_stats.running_containers, 7);
        assert_eq!(system_stats.cpu_info.cpus, 4);
        assert_eq!(system_stats.memory_info.total, 8 * 1024 * 1024 * 1024);
    }
}
