//! Raft Storage Store Implementations
//!
//! This module provides storage-related utilities and re-exports for the Raft backend.
//! The main storage implementation is in `combined_storage.rs`.

// Re-export the combined storage for backward compatibility
pub use super::combined_storage::SecretonRaftStorage;

// Re-export state machine types
pub use super::state_machine::{
    SecretonStateMachine, StateMachineCommand, StateMachineResponse, StateMachineSnapshot,
};
