//! OpenRaft Type Configuration
//!
//! Defines the type configuration for OpenRaft consensus implementation.

use openraft::BasicNode;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Node ID type for the Raft cluster
pub type NodeId = u64;

/// Node information
pub type Node = BasicNode;

/// Type configuration for Secreton's Raft implementation
#[derive(
    Debug, Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
pub struct SecretonTypeConfig;

impl fmt::Display for SecretonTypeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretonTypeConfig")
    }
}

impl openraft::RaftTypeConfig for SecretonTypeConfig {
    /// Node ID type
    type NodeId = NodeId;

    /// Node information type
    type Node = Node;

    /// Application data type for client requests
    type Entry = openraft::Entry<Self>;

    /// Snapshot data type
    type SnapshotData = std::io::Cursor<Vec<u8>>;

    /// Application-specific response type
    type AsyncRuntime = openraft::TokioRuntime;

    /// Application-specific data type
    type D = super::state_machine::StateMachineCommand;

    /// Application-specific response type
    type R = super::state_machine::StateMachineResponse;

    /// Responder type (same as response)
    type Responder = openraft::impls::OneshotResponder<Self>;
}

/// Raft instance type alias
pub type Raft = openraft::Raft<SecretonTypeConfig>;

/// Raft configuration type alias
pub type Config = openraft::Config;

/// Raft log entry type alias
pub type Entry = openraft::Entry<SecretonTypeConfig>;

/// Raft log ID type alias
pub type LogId = openraft::LogId<NodeId>;

/// Raft membership type alias
pub type Membership = openraft::Membership<NodeId, Node>;

/// Raft vote type alias
pub type Vote = openraft::Vote<NodeId>;
