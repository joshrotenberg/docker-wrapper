//! # docker-wrapper
//!
//! A simple, focused Docker CLI wrapper for Rust.
//!
//! This crate provides a clean interface to Docker's common commands,
//! focusing on reliability and ease of use.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod prerequisites;

pub use prerequisites::{
    ensure_docker, DockerInfo, DockerPrerequisites, PrerequisitesError, PrerequisitesResult,
};

/// The version of this crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
