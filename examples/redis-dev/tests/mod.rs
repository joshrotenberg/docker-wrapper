//! Test suite for Redis Developer CLI
//!
//! This module organizes all tests for the redis-dev CLI tool,
//! following the testing patterns established in our docker-wrapper
//! testing documentation.

#[cfg(test)]
mod integration_test;

#[cfg(test)]
mod unit_tests;

/// Test utilities and fixtures
#[cfg(test)]
pub mod test_utils {
    use std::sync::Once;
    
    static INIT: Once = Once::new();
    
    /// Initialize test environment once
    pub fn init_test_env() {
        INIT.call_once(|| {
            // Set up logging for tests if needed
            let _ = tracing_subscriber::fmt()
                .with_env_filter("redis_dev=debug,docker_wrapper=info")
                .try_init();
        });
    }
    
    /// Generate a unique test instance name
    pub fn unique_instance_name(prefix: &str) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let suffix: u32 = rng.gen_range(10000..99999);
        format!("{}-test-{}", prefix, suffix)
    }
}