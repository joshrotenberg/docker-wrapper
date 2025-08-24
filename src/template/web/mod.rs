//! Web server template collection
//!
//! This module provides templates for web servers and reverse proxies:
//! - Nginx for static content and reverse proxy
//! - Apache HTTP Server (future)
//! - Caddy Server (future)
//! - Traefik (future)

#[cfg(feature = "template-nginx")]
pub mod nginx;
#[cfg(feature = "template-nginx")]
pub use nginx::NginxTemplate;
