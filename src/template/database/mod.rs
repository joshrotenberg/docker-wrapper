//! Database template collection
//!
//! This module provides templates for various database systems:
//! - PostgreSQL with extensions and replication support
//! - MySQL with various storage engines
//! - MongoDB with replica sets and sharding

#[cfg(feature = "template-postgres")]
pub mod postgres;
#[cfg(feature = "template-postgres")]
pub use postgres::{PostgresConnectionString, PostgresTemplate};

#[cfg(feature = "template-mysql")]
pub mod mysql;
#[cfg(feature = "template-mysql")]
pub use mysql::{MysqlConnectionString, MysqlTemplate};

#[cfg(feature = "template-mongodb")]
pub mod mongodb;
#[cfg(feature = "template-mongodb")]
pub use mongodb::{MongodbConnectionString, MongodbTemplate};
