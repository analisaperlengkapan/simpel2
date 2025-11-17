//! API prelude - commonly used API types and traits
//!
//! This module re-exports the most commonly used items from the API crate
//! to simplify imports in other modules.

// Re-export main API types
pub use crate::{
    ApiConfig, ApiState, HealthResponse, TlsMetricsResponse, VersionResponse, create_api_router,
};

// Re-export transit API
pub use crate::transit::{TransitApiState, create_transit_router};

// Re-export KV API
pub use crate::kv::{KVApiState, KVEngine, create_kv_router};

// Re-export PKI API
pub use crate::pki::{PkiApiState, create_pki_router};

// TODO: Re-enable after OpenRaft migration
// pub use crate::raft::{create_raft_router, RaftApiState};

// Re-export commonly used external crates
pub use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
pub use serde::{Deserialize, Serialize};
pub use std::sync::Arc;
pub use tokio::sync::RwLock;
