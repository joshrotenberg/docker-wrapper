//! Redis template collection
//!
//! This module provides various Redis deployment templates:
//! - Basic Redis for simple key-value storage
//! - Redis Cluster for sharded deployments (future)
//! - Redis Sentinel for high availability (future)
//! - Redis Stack with modules (future)
//! - Redis Enterprise with management UI (future)

// Common utilities available to all Redis templates
pub(crate) mod common;

// Basic Redis template
pub mod basic;
pub use basic::RedisTemplate;

// Redis Cluster template
#[cfg(feature = "template-redis-cluster")]
pub mod cluster;
#[cfg(feature = "template-redis-cluster")]
pub use cluster::{ClusterInfo, NodeInfo, NodeRole, RedisClusterConnection, RedisClusterTemplate};

// #[cfg(feature = "template-redis-sentinel")]
// pub mod sentinel;
// #[cfg(feature = "template-redis-sentinel")]
// pub use sentinel::RedisSentinelTemplate;

// RedisInsight template
pub mod insight;
pub use insight::RedisInsightTemplate;

// #[cfg(feature = "template-redis-stack")]
// pub mod stack;
// #[cfg(feature = "template-redis-stack")]
// pub use stack::RedisStackTemplate;

// #[cfg(feature = "template-redis-enterprise")]
// pub mod enterprise;
// #[cfg(feature = "template-redis-enterprise")]
// pub use enterprise::RedisEnterpriseTemplate;
