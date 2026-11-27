//! Storage backend implementations
//!
//! HashiCorp Engine-compatible storage backends for Secreton engine system.
//!
//! # Available Backends
//!
//! ## High Availability Backends
//! - **Consul**: Service discovery + distributed KV store (recommended for HA)
//! - **Raft**: Built-in distributed consensus (no external dependencies)
//!
//! ## Cloud-Native Backends
//! - **Azure Blob**: Azure Blob Storage
//! - **GCS**: Google Cloud Storage
//!
//! ## Traditional Backends
//! - **File**: Local filesystem (development, single-node)
//! - **PostgreSQL**: Relational database (optional, not recommended for HA)
//!
//! # Recommendations
//!
//! - **Production HA**: Use Consul or Raft (like HashiCorp Engine)
//! - **Cloud Deployments**: not supported (these deployments run on their own servers)
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

#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "postgres")]
pub use postgres::PostgresBackend;
