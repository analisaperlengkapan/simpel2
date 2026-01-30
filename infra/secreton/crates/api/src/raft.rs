//! Raft Cluster Management API
//!
//! API endpoints for managing Raft cluster operations.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

// TODO: Re-enable after OpenRaft migration is complete
// use secreton_storage::{RaftCluster, RaftClusterConfig, RaftStatus};

// Temporary stub types until OpenRaft migration is complete
pub struct RaftNode;
pub struct RaftNodeConfig;
pub struct RaftStatus;

/// Raft API state
#[derive(Clone)]
pub struct RaftApiState {
    pub node: Arc<RaftNode>,
}

/// Raft cluster status response
#[derive(Debug, Serialize, Deserialize)]
pub struct RaftStatusResponse {
    pub node_id: u64,
    pub state: String,
    pub leader_id: u64,
    pub term: u64,
    pub commit_index: u64,
    pub applied_index: u64,
    pub is_leader: bool,
}

/// Add peer request
#[derive(Debug, Deserialize)]
pub struct AddPeerRequest {
    pub node_id: u64,
    pub address: String,
}

/// Remove peer request
#[derive(Debug, Deserialize)]
pub struct RemovePeerRequest {
    pub node_id: u64,
}

/// Create Raft router
pub fn create_raft_router(state: RaftApiState) -> Router {
    Router::new()
        .route("/status", get(get_status))
        .route("/peers", post(add_peer))
        .route("/peers/{node_id}", delete(remove_peer))
        .route("/leader", get(get_leader))
        .with_state(state)
}

/// Get Raft cluster status
async fn get_status(State(state): State<RaftApiState>) -> Json<RaftStatusResponse> {
    let status = state.node.status();

    Json(RaftStatusResponse {
        node_id: status.node_id,
        state: format!("{:?}", status.state),
        leader_id: status.leader_id,
        term: status.term,
        commit_index: status.commit_index,
        applied_index: status.applied_index,
        is_leader: state.node.is_leader(),
    })
}

/// Add a peer to the cluster
async fn add_peer(
    State(state): State<RaftApiState>,
    Json(req): Json<AddPeerRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Adding peer: node_id={, address={}", req.node_id, req.address);

    match state.node.add_peer(req.node_id, req.address).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "Peer added successfully"
        }))),
        Err(e) => {
            error!("Failed to add peer: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Remove a peer from the cluster
async fn remove_peer(
    State(state): State<RaftApiState>,
    Path(node_id): Path<u64>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Removing peer: node_id={}", node_id);

    match state.node.remove_peer(node_id).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "Peer removed successfully"
        }))),
        Err(e) => {
            error!("Failed to remove peer: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get current leader information
async fn get_leader(State(state): State<RaftApiState>) -> Json<serde_json::Value> {
    let leader_id = state.node.leader_id();
    let is_leader = state.node.is_leader();

    Json(serde_json::json!({
        "leader_id": leader_id,
        "is_leader": is_leader,
        "node_id": state.node.config().node_id,
    }))
}

#[cfg(all(test, feature = "enable-inline-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_raft_status_response() {
        let response = RaftStatusResponse {
            node_id: 1,
            state: "Follower".to_string(),
            leader_id: 0,
            term: 0,
            commit_index: 0,
            applied_index: 0,
            is_leader: false,
        };

        assert_eq!(response.node_id, 1);
        assert!(!response.is_leader);
    }
}
