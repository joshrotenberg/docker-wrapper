//! Redis Developer CLI Tool Library
//!
//! This library exposes the core functionality of the redis-dev CLI tool
//! for testing and programmatic usage.

pub mod cli;
pub mod commands;
pub mod config;

// Re-export commonly used types
pub use cli::{Cli, Commands};
pub use config::{Config, InstanceInfo, InstanceType};
