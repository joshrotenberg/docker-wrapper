//! Utility functions and helpers for the docker wrapper.
//!
//! This module contains common utility functions used throughout the
//! docker wrapper implementation.

use crate::errors::{DockerError, DockerResult};
use std::time::Duration;

/// Parse a duration string (e.g., "30s", "5m", "1h")
#[allow(dead_code)]
pub fn parse_duration(s: &str) -> DockerResult<Duration> {
    if s.is_empty() {
        return Err(DockerError::invalid_config("Duration cannot be empty"));
    }

    let (num_str, unit) = if let Some(pos) = s.rfind(|c: char| c.is_ascii_digit()) {
        let (num, unit) = s.split_at(pos + 1);
        (num, unit)
    } else {
        return Err(DockerError::invalid_config("Invalid duration format"));
    };

    let num: u64 = num_str
        .parse()
        .map_err(|_| DockerError::invalid_config("Invalid number in duration"))?;

    let duration = match unit {
        "s" | "" => Duration::from_secs(num),
        "m" => Duration::from_secs(num * 60),
        "h" => Duration::from_secs(num * 3600),
        "ms" => Duration::from_millis(num),
        _ => {
            return Err(DockerError::invalid_config(format!(
                "Unknown duration unit: {unit}"
            )));
        }
    };

    Ok(duration)
}

/// Format a duration as a human-readable string
#[allow(dead_code)]
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs >= 3600 {
        format!("{secs}h", secs = secs / 3600)
    } else if secs >= 60 {
        format!("{mins}m", mins = secs / 60)
    } else {
        format!("{secs}s")
    }
}

/// Validate a Docker container name
#[allow(dead_code)]
pub fn validate_container_name(name: &str) -> DockerResult<()> {
    if name.is_empty() {
        return Err(DockerError::invalid_config(
            "Container name cannot be empty",
        ));
    }

    if name.len() > 63 {
        return Err(DockerError::invalid_config(
            "Container name cannot exceed 63 characters",
        ));
    }

    // Docker container names must match: [a-zA-Z0-9][a-zA-Z0-9_.-]*
    let first_char = name.chars().next().unwrap();
    if !first_char.is_ascii_alphanumeric() {
        return Err(DockerError::invalid_config(
            "Container name must start with alphanumeric character",
        ));
    }

    for c in name.chars().skip(1) {
        if !c.is_ascii_alphanumeric() && c != '_' && c != '.' && c != '-' {
            return Err(DockerError::invalid_config(
                "Container name can only contain alphanumeric characters, underscore, period, and hyphen",
            ));
        }
    }

    Ok(())
}

/// Sanitize a string for use as a container name
#[allow(dead_code)]
pub fn sanitize_container_name(name: &str) -> String {
    if name.is_empty() {
        return "container".to_string();
    }

    let mut result = String::new();
    let mut chars = name.chars();

    // Handle first character
    if let Some(first) = chars.next() {
        if first.is_ascii_alphanumeric() {
            result.push(first.to_ascii_lowercase());
        } else {
            result.push('c');
        }
    }

    // Handle remaining characters
    for c in chars {
        if c.is_ascii_alphanumeric() {
            result.push(c.to_ascii_lowercase());
        } else if c == '_' || c == '.' || c == '-' {
            result.push(c);
        } else {
            result.push('-');
        }
    }

    // Truncate if too long
    if result.len() > 63 {
        result.truncate(63);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("500ms").unwrap(), Duration::from_millis(500));
        assert_eq!(parse_duration("10").unwrap(), Duration::from_secs(10));

        assert!(parse_duration("").is_err());
        assert!(parse_duration("invalid").is_err());
        assert!(parse_duration("10x").is_err());
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(300)), "5m");
        assert_eq!(format_duration(Duration::from_secs(3600)), "1h");
    }

    #[test]
    fn test_validate_container_name() {
        assert!(validate_container_name("test-container").is_ok());
        assert!(validate_container_name("test123").is_ok());
        assert!(validate_container_name("test_container.name").is_ok());

        assert!(validate_container_name("").is_err());
        assert!(validate_container_name("-invalid").is_err());
        assert!(validate_container_name("invalid@name").is_err());
        assert!(validate_container_name(&"x".repeat(64)).is_err());
    }

    #[test]
    fn test_sanitize_container_name() {
        assert_eq!(sanitize_container_name("test-container"), "test-container");
        assert_eq!(
            sanitize_container_name("Test@Container!"),
            "test-container-"
        );
        assert_eq!(sanitize_container_name("-invalid"), "cinvalid");
        assert_eq!(sanitize_container_name(""), "container");

        let long_name = "x".repeat(100);
        let sanitized = sanitize_container_name(&long_name);
        assert_eq!(sanitized.len(), 63);
    }
}
