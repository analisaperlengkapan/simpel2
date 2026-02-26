//! # Secreton Replication Module
//!
//! This module provides replication capabilities for Secreton, supporting both
//! Performance Replication (read-only replicas) and Disaster Recovery Replication
//! (full cluster failover).
//!
//! ## Features
//!
//! - **Performance Replication**: Multi-region read replicas for low-latency access
//! - **DR Replication**: Full disaster recovery with sealed secondary clusters
//! - **Replication Lag Monitoring**: Track and expose replication lag metrics
//! - **Automatic Failover**: Promote secondary to primary on failure detection
//! - **mTLS Communication**: Secure replication traffic with mutual TLS
//!
//! ## Architecture
//!
//! ```text
//! Primary Cluster                    Secondary Cluster
//! ┌──────────────┐                  ┌──────────────┐
//! │   Write      │  ─────gRPC────▶  │   Replicate  │
//! │   Operations │   (mTLS)         │   & Apply    │
//! └──────────────┘                  └──────────────┘
//!        │                                  │
//!        ▼                                  ▼
//! ┌──────────────┐                  ┌──────────────┐
//! │  PostgreSQL  │                  │  PostgreSQL  │
//! │   (Primary)  │                  │ (Secondary)  │
//! └──────────────┘                  └──────────────┘
//! ```

pub mod config;
pub mod conflict;
pub mod error;
pub mod failover;
pub mod manager;
pub mod metrics;
pub mod mode;
pub mod node;
pub mod operation;
pub mod secondary;

// Re-exports
pub use config::ReplicationConfig;
pub use conflict::{Conflict, ConflictResolver, ResolutionStrategy};
pub use error::ReplicationError;
pub use failover::{FailoverDetector, HealthStatus, Heartbeat};
pub use manager::ReplicationManager;
pub use metrics::{ReplicationMetrics, ReplicationStatusValue};
pub use mode::ReplicationMode;
pub use node::{ReplicationStatus, SecondaryNode};
pub use operation::{OperationData, OperationType, ReplicationOperation};
pub use secondary::{
    InitializationState, ReplicationLagMetrics, ReplicationSnapshot, SecondaryInitializer,
    SecondaryReadError, SecondaryReadHandler, SecondaryReadResult, SnapshotCreator,
    SnapshotMetadata,
};

/// Result type for replication operations
pub type Result<T> = std::result::Result<T, ReplicationError>;
