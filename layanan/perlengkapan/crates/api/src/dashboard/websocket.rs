//! WebSocket handler for real-time dashboard updates
//!
//! Provides WebSocket endpoint for streaming dashboard metric updates to connected clients.
//! Uses broadcast channel for efficient multi-client updates.

use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::AppState;

/// Dashboard update message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DashboardUpdate {
    /// Metrics update for perlengkapan dashboard
    MetricsUpdate {
        tahun_anggaran: i32,
        timestamp: chrono::DateTime<chrono::Utc>,
        data: serde_json::Value,
    },
    /// Workflow status change notification
    WorkflowUpdate {
        entity_id: Uuid,
        entity_type: String,
        new_status: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Gap analysis update
    GapAnalysisUpdate {
        satker_id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        data: serde_json::Value,
    },
    /// Heartbeat/ping message
    Ping {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// WebSocket handler for dashboard updates
///
/// Upgrades HTTP connection to WebSocket and subscribes to dashboard update broadcasts.
///
/// # Example
/// ```javascript
/// const ws = new WebSocket('ws://localhost:8093/api/pembinaan/perlengkapan/dashboard/ws');
/// ws.onmessage = (event) => {
///     const update = JSON.parse(event.data);
///     console.log('Dashboard update:', update);
/// };
/// ```
pub async fn dashboard_websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    info!("New WebSocket connection request for dashboard updates");
    ws.on_upgrade(|socket| handle_dashboard_socket(socket, state))
}

/// Handle individual WebSocket connection
///
/// Subscribes to broadcast channel and forwards updates to the client.
/// Implements automatic reconnection support via ping/pong.
async fn handle_dashboard_socket(mut socket: WebSocket, state: AppState) {
    let client_id = Uuid::new_v4();
    info!("WebSocket client {} connected", client_id);

    // Subscribe to dashboard updates broadcast channel
    let mut rx = state.dashboard_updates.subscribe();

    // Send initial connection confirmation
    let welcome_msg = serde_json::json!({
        "type": "connected",
        "client_id": client_id.to_string(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    if let Ok(msg_text) = serde_json::to_string(&welcome_msg) {
        if socket.send(Message::Text(msg_text.into())).await.is_err() {
            warn!("Failed to send welcome message to client {}", client_id);
            return;
        }
    }

    // Main message loop
    loop {
        tokio::select! {
            // Receive updates from broadcast channel
            update_result = rx.recv() => {
                match update_result {
                    Ok(update) => {
                        debug!("Broadcasting update to client {}: {:?}", client_id, update);

                        // Serialize update to JSON
                        match serde_json::to_string(&update) {
                            Ok(msg_text) => {
                                // Send to client
                                if let Err(e) = socket.send(Message::Text(msg_text.into())).await {
                                    error!("Failed to send update to client {}: {}", client_id, e);
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("Failed to serialize update: {}", e);
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("Client {} lagged behind, skipped {} messages", client_id, skipped);
                        // Send lag notification to client
                        let lag_msg = serde_json::json!({
                            "type": "lag_warning",
                            "skipped_messages": skipped,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                        });
                        if let Ok(msg_text) = serde_json::to_string(&lag_msg) {
                            let _ = socket.send(Message::Text(msg_text.into())).await;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Broadcast channel closed for client {}", client_id);
                        break;
                    }
                }
            }

            // Receive messages from client (for ping/pong)
            msg_result = socket.recv() => {
                match msg_result {
                    Some(Ok(msg)) => {
                        match msg {
                            Message::Text(text) => {
                                debug!("Received text message from client {}: {}", client_id, text);
                                // Handle client messages (e.g., subscription filters)
                                if let Err(e) = handle_client_message(&text, &mut socket).await {
                                    error!("Error handling client message: {}", e);
                                }
                            }
                            Message::Ping(data) => {
                                debug!("Received ping from client {}", client_id);
                                if socket.send(Message::Pong(data)).await.is_err() {
                                    break;
                                }
                            }
                            Message::Pong(_) => {
                                debug!("Received pong from client {}", client_id);
                            }
                            Message::Close(_) => {
                                info!("Client {} requested close", client_id);
                                break;
                            }
                            _ => {
                                warn!("Received unexpected message type from client {}", client_id);
                            }
                        }
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error for client {}: {}", client_id, e);
                        break;
                    }
                    None => {
                        info!("Client {} disconnected", client_id);
                        break;
                    }
                }
            }
        }
    }

    info!("WebSocket client {} disconnected", client_id);
}

/// Handle messages from client
///
/// Supports subscription filtering and other client commands.
async fn handle_client_message(text: &str, socket: &mut WebSocket) -> anyhow::Result<()> {
    // Parse client message
    let msg: serde_json::Value = serde_json::from_str(text)?;

    match msg.get("type").and_then(|v| v.as_str()) {
        Some("ping") => {
            // Respond to client ping
            let pong = serde_json::json!({
                "type": "pong",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });
            let pong_text = serde_json::to_string(&pong)?;
            socket.send(Message::Text(pong_text.into())).await?;
        }
        Some("subscribe") => {
            // Handle subscription filters (future enhancement)
            debug!("Client subscription request: {:?}", msg);
            let ack = serde_json::json!({
                "type": "subscribed",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });
            let ack_text = serde_json::to_string(&ack)?;
            socket.send(Message::Text(ack_text.into())).await?;
        }
        _ => {
            warn!("Unknown client message type: {:?}", msg);
        }
    }

    Ok(())
}

/// Broadcast dashboard update to all connected clients
///
/// This function should be called whenever dashboard metrics change.
///
/// # Example
/// ```rust
/// use crate::dashboard::websocket::{broadcast_dashboard_update, DashboardUpdate};
///
/// // After workflow transition
/// broadcast_dashboard_update(
///     &state.dashboard_updates,
///     DashboardUpdate::WorkflowUpdate {
///         entity_id: kebutuhan_id,
///         entity_type: "kebutuhan_bmn".to_string(),
///         new_status: "APPROVED".to_string(),
///         timestamp: chrono::Utc::now(),
///     },
/// );
/// ```
pub fn broadcast_dashboard_update(
    tx: &broadcast::Sender<DashboardUpdate>,
    update: DashboardUpdate,
) {
    match tx.send(update.clone()) {
        Ok(receiver_count) => {
            debug!(
                "Broadcast update to {} receivers: {:?}",
                receiver_count, update
            );
        }
        Err(e) => {
            // No receivers connected - this is not an error
            debug!("No receivers for dashboard update: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_update_serialization() {
        let update = DashboardUpdate::MetricsUpdate {
            tahun_anggaran: 2024,
            timestamp: chrono::Utc::now(),
            data: serde_json::json!({
                "total_kebutuhan": 100,
                "approved": 50,
            }),
        };

        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("metrics_update"));
        assert!(json.contains("tahun_anggaran"));
    }

    #[test]
    fn test_workflow_update_serialization() {
        let update = DashboardUpdate::WorkflowUpdate {
            entity_id: Uuid::new_v4(),
            entity_type: "kebutuhan_bmn".to_string(),
            new_status: "APPROVED".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("workflow_update"));
        assert!(json.contains("entity_type"));
    }
}
