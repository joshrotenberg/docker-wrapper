//! Redis Developer CLI Tool Library
//!
//! This library exposes the core functionality of the redis-dev CLI tool
//! for testing and programmatic usage.

pub mod cli;
pub mod commands;
pub mod config;

// Re-export commonly used types
pub use config::{Config, InstanceType, InstanceInfo};
pub use cli::{Cli, Commands};