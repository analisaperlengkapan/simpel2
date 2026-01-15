//! Raft Node Type Aliases and Re-exports
//!
//! This module provides convenient re-exports for Raft-related types.

// Re-export the Raft type from types module
pub use super::types::Raft;

// Re-export SecretonRaftStorage from combined_storage module
pub use super::combined_storage::SecretonRaftStorage;
