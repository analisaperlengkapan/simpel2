//! Storage backend implementations
//!
//! Essential storage backends for Secreton vault system.

pub mod postgres;

pub use postgres::PostgresBackend;
