//! Built-in health checks for Secreton
//!
//! This module provides pre-built health check implementations for common
//! Secreton components and services.

pub mod raft_cluster;
pub mod raft_cluster_adapter;
pub mod replication;
pub mod seal_service_adapter;
pub mod seal_status;

pub use raft_cluster::{RaftClusterHealthCheck, RaftClusterProvider};
pub use raft_cluster_adapter::{RaftClusterAdapter, RaftClusterLike, RaftStatusLike};
pub use replication::ReplicationHealthCheck;
pub use seal_service_adapter::SealServiceLike;
pub use seal_status::{SealStatusHealthCheck, SealStatusProvider};
