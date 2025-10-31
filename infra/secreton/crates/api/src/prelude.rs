//! API prelude - commonly used API types and traits
//!
//! This module re-exports the most commonly used items from the API crate
//! to simplify imports in other modules.

// Re-export main API types
pub use crate::{
    create_api_router, ApiConfig, ApiState, HealthResponse, TlsMetricsResponse, VersionResponse,
};

// Re-export transit API
pub use crate::transit::{create_transit_router, TransitApiState};

// Re-export KV API
pub use crate::kv::{create_kv_router, KVApiState, KVEngine};

// TODO: Re-enable after OpenRaft migration
// pub use crate::raft::{create_raft_router, RaftApiState};

// Re-export commonly used external crates
pub use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
pub use serde::{Deserialize, Serialize};
pub use std::sync::Arc;
pub use tokio::sync::RwLock;
