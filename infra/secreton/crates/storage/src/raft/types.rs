//! OpenRaft Type Configuration
//!
//! Defines the type configuration for OpenRaft consensus implementation.

use openraft::BasicNode;
use serde::{Deserialize, Serialize};

/// Node ID type for the Raft cluster
pub type NodeId = u64;

/// Node information
pub type Node = BasicNode;

/// Type configuration for Secreton's Raft implementation
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SecretonTypeConfig;

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
pub type Membership = openraft::Membership<SecretonTypeConfig>;

/// Raft vote type alias
pub type Vote = openraft::Vote<NodeId>;
