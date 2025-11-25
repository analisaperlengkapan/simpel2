//! OpenRaft Type Configuration
//!
//! Defines the type configuration for Raft integration

use std::io::Cursor;

// Import state machine types for D and R
use super::state_machine::{StateMachineCommand, StateMachineResponse};

/// Node ID type for the Raft cluster
pub type NodeId = u64;

// Declare Raft types using OpenRaft's macro
openraft::declare_raft_types!(
    /// Type configuration for Secreton's Raft implementation
    pub SecretonTypeConfig:
        D = StateMachineCommand,
        R = StateMachineResponse,
);

/// Raft instance type alias
pub type Raft = openraft::Raft<SecretonTypeConfig>;

/// Raft configuration type alias
pub type Config = openraft::Config;

/// Raft log entry type alias
pub type Entry = openraft::Entry<SecretonTypeConfig>;

/// Raft log ID type alias
pub type LogId = openraft::LogId<NodeId>;

/// Raft membership type alias
pub type Membership = openraft::Membership<SecretonTypeConfig, NodeId>;

/// Raft vote type alias
pub type Vote = openraft::Vote<NodeId>;
