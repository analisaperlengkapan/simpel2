//! Storage backend implementations
//!
//! HashiCorp Vault-compatible storage backends for Secreton vault system.
//!
//! # Available Backends
//!
//! ## High Availability Backends
//! - **Consul**: Service discovery + distributed KV store (recommended for HA)
//! - **Raft**: Built-in distributed consensus (no external dependencies)
//!
//! ## Cloud-Native Backends
//! - **S3**: AWS S3 and S3-compatible storage (MinIO, Wasabi, etc.)
//! - **Azure Blob**: Azure Blob Storage
//! - **GCS**: Google Cloud Storage
//!
//! ## Traditional Backends
//! - **File**: Local filesystem (development, single-node)
//! - **PostgreSQL**: Relational database (optional, not recommended for HA)
//!
//! # Recommendations
//!
//! - **Production HA**: Use Consul or Raft (like HashiCorp Vault)
//! - **Cloud Deployments**: Use S3, Azure Blob, or GCS
//! - **Development**: Use File or Memory backend
//! - **Legacy Systems**: PostgreSQL available but not recommended

// Always available backends
pub mod file;
pub use file::{FileBackend, FileConfig};

// Optional backends (feature-gated)
#[cfg(feature = "consul")]
pub mod consul;
#[cfg(feature = "consul")]
pub use consul::{ConsulBackend, ConsulConfig};

#[cfg(feature = "s3")]
pub mod s3;
#[cfg(feature = "s3")]
pub use s3::{S3Backend, S3Config};

#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "postgres")]
pub use postgres::PostgresBackend;
