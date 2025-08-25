//! CLI argument parsing and command definitions

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "redis-dev",
    about = "Redis Developer Tool - Quickly spin up Redis environments for development and testing",
    long_about = "A CLI tool for Redis developers to easily create, manage, and test various Redis deployments including basic Redis, Redis Stack, Redis Cluster, Redis Sentinel, and Redis Enterprise.",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage basic Redis instances
    Basic {
        #[command(subcommand)]
        action: RedisAction,
    },
    /// Manage Redis Stack instances (with modules)
    Stack {
        #[command(subcommand)]
        action: StackAction,
    },
    /// Manage Redis Cluster instances
    Cluster {
        #[command(subcommand)]
        action: ClusterAction,
    },
    /// Manage Redis Sentinel instances
    Sentinel {
        #[command(subcommand)]
        action: SentinelAction,
    },
    /// Manage Redis Enterprise instances
    Enterprise {
        #[command(subcommand)]
        action: EnterpriseAction,
    },
    /// List all running Redis instances
    List {
        /// Filter by instance type
        #[arg(short, long)]
        r#type: Option<String>,
    },
    /// Clean up all Redis instances
    Cleanup {
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
        /// Only cleanup instances of specific type
        #[arg(short, long)]
        r#type: Option<String>,
    },
    /// View logs for Redis instances
    Logs {
        /// Instance name (defaults to latest)
        name: Option<String>,
        /// Follow logs (like tail -f)
        #[arg(short, long)]
        follow: bool,
        /// Number of lines to show from end
        #[arg(short, long, default_value = "20")]
        tail: u32,
        /// Show timestamps
        #[arg(short, long)]
        timestamps: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum RedisAction {
    /// Start a basic Redis instance
    Start(BasicStartArgs),
    /// Stop a basic Redis instance
    Stop(StopArgs),
    /// Get info about a basic Redis instance
    Info(InfoArgs),
}

#[derive(Subcommand, Debug)]
pub enum StackAction {
    /// Start a Redis Stack instance
    Start(StackStartArgs),
    /// Stop a Redis Stack instance
    Stop(StopArgs),
    /// Get info about a Redis Stack instance
    Info(InfoArgs),
}

#[derive(Subcommand, Debug)]
pub enum ClusterAction {
    /// Start a Redis Cluster
    Start(ClusterStartArgs),
    /// Stop a Redis Cluster
    Stop(StopArgs),
    /// Get info about a Redis Cluster
    Info(InfoArgs),
}

#[derive(Subcommand, Debug)]
pub enum SentinelAction {
    /// Start a Redis Sentinel setup
    Start(SentinelStartArgs),
    /// Stop a Redis Sentinel setup
    Stop(StopArgs),
    /// Get info about a Redis Sentinel setup
    Info(InfoArgs),
}

#[derive(Subcommand, Debug)]
pub enum EnterpriseAction {
    /// Start a Redis Enterprise cluster
    Start(EnterpriseStartArgs),
    /// Stop a Redis Enterprise cluster
    Stop(StopArgs),
    /// Get info about a Redis Enterprise cluster
    Info(InfoArgs),
}

#[derive(Args, Debug)]
pub struct BasicStartArgs {
    /// Instance name (auto-generated if not provided)
    #[arg(short, long)]
    pub name: Option<String>,

    /// Redis port (default: 6379)
    #[arg(short, long, default_value = "6379")]
    pub port: u16,

    /// Set a password for Redis
    #[arg(long)]
    pub password: Option<String>,

    /// Enable persistence
    #[arg(long)]
    pub persist: bool,

    /// Memory limit (e.g., "256m", "1g")
    #[arg(long)]
    pub memory: Option<String>,

    /// Connect to redis-cli shell after starting
    #[arg(long)]
    pub shell: bool,
}

#[derive(Args, Debug)]
pub struct StackStartArgs {
    /// Instance name (auto-generated if not provided)
    #[arg(short, long)]
    pub name: Option<String>,

    /// Redis port (default: 6379)
    #[arg(short, long, default_value = "6379")]
    pub port: u16,

    /// Set a password for Redis
    #[arg(long)]
    pub password: Option<String>,

    /// Enable persistence
    #[arg(long)]
    pub persist: bool,

    /// Memory limit (e.g., "256m", "1g")
    #[arg(long)]
    pub memory: Option<String>,

    /// Enable RedisJSON module
    #[arg(long)]
    pub with_json: bool,

    /// Enable RediSearch module
    #[arg(long)]
    pub with_search: bool,

    /// Enable RedisTimeSeries module
    #[arg(long)]
    pub with_timeseries: bool,

    /// Enable RedisGraph module
    #[arg(long)]
    pub with_graph: bool,

    /// Enable RedisBloom module
    #[arg(long)]
    pub with_bloom: bool,

    /// Enable all popular modules (JSON + Search + TimeSeries)
    #[arg(long)]
    pub demo_bundle: bool,

    /// Start RedisInsight GUI
    #[arg(long)]
    pub insight: bool,

    /// RedisInsight port (default: 8001)
    #[arg(long, default_value = "8001")]
    pub insight_port: u16,

    /// Connect to redis-cli shell after starting
    #[arg(long)]
    pub shell: bool,
}

#[derive(Args, Debug)]
pub struct ClusterStartArgs {
    /// Cluster name (auto-generated if not provided)
    #[arg(short, long)]
    pub name: Option<String>,

    /// Number of master nodes (minimum 3)
    #[arg(short, long, default_value = "3")]
    pub masters: usize,

    /// Number of replicas per master
    #[arg(short, long, default_value = "0")]
    pub replicas: usize,

    /// Base port for Redis nodes (default: 7000)
    #[arg(long, default_value = "7000")]
    pub port_base: u16,

    /// Set a password for the cluster
    #[arg(long)]
    pub password: Option<String>,

    /// Enable persistence
    #[arg(long)]
    pub persist: bool,

    /// Memory limit per node (e.g., "256m", "1g")
    #[arg(long)]
    pub memory: Option<String>,

    /// Use Redis Stack instead of basic Redis
    #[arg(long)]
    pub stack: bool,

    /// Start RedisInsight GUI
    #[arg(long)]
    pub insight: bool,

    /// RedisInsight port (default: 8001)
    #[arg(long, default_value = "8001")]
    pub insight_port: u16,

    /// Connect to redis-cli shell after starting
    #[arg(long)]
    pub shell: bool,
}

#[derive(Args, Debug)]
pub struct SentinelStartArgs {
    /// Sentinel setup name (auto-generated if not provided)
    #[arg(short, long)]
    pub name: Option<String>,

    /// Number of Redis masters to monitor
    #[arg(short, long, default_value = "1")]
    pub masters: usize,

    /// Number of Sentinel nodes
    #[arg(short, long, default_value = "3")]
    pub sentinels: usize,

    /// Base port for Redis masters (default: 6379)
    #[arg(long, default_value = "6379")]
    pub redis_port_base: u16,

    /// Base port for Sentinel nodes (default: 26379)
    #[arg(long, default_value = "26379")]
    pub sentinel_port_base: u16,

    /// Set a password for Redis instances
    #[arg(long)]
    pub password: Option<String>,

    /// Enable persistence
    #[arg(long)]
    pub persist: bool,

    /// Memory limit per instance (e.g., "256m", "1g")
    #[arg(long)]
    pub memory: Option<String>,
}

#[derive(Args, Debug)]
pub struct EnterpriseStartArgs {
    /// Enterprise cluster name (auto-generated if not provided)
    #[arg(short, long)]
    pub name: Option<String>,

    /// Number of Enterprise nodes
    #[arg(short = 'N', long, default_value = "3")]
    pub nodes: usize,

    /// Base port for Enterprise nodes (default: 8443)
    #[arg(long, default_value = "8443")]
    pub port_base: u16,

    /// Create a database after cluster formation
    #[arg(long)]
    pub create_db: Option<String>,

    /// Database port (default: 12000)
    #[arg(long, default_value = "12000")]
    pub db_port: u16,

    /// Memory limit per node (e.g., "4g", "8g")
    #[arg(long)]
    pub memory: Option<String>,

    /// Enable persistence
    #[arg(long)]
    pub persist: bool,

    /// Skip cluster formation (just start containers)
    #[arg(long)]
    pub containers_only: bool,
}

#[derive(Args, Debug)]
pub struct StopArgs {
    /// Instance name (uses auto-generated name if not provided)
    pub name: Option<String>,
}

#[derive(Args, Debug)]
pub struct InfoArgs {
    /// Instance name (uses auto-generated name if not provided)
    pub name: Option<String>,

    /// Output format
    #[arg(short, long, default_value = "table")]
    pub format: String,
}
