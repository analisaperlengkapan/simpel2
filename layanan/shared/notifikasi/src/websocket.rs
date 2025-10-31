use axum::{
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::{IntoResponse, Response},
};
use deadpool_postgres::Pool;
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::{error, info, warn};
use uuid::Uuid;

/// WebSocket connection state
#[derive(Clone)]
pub struct WsState {
    pub pool: Pool,
    pub tx: broadcast::Sender<NotificationMessage>,
    pub connections: Arc<RwLock<Vec<Uuid>>>,
}

impl WsState {
    pub fn new(pool: Pool) -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self {
            pool,
            tx,
            connections: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

/// Notification message sent via WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub body: String,
    pub category: NotificationCategory,
    pub priority: NotificationPriority,
    pub action_url: Option<String>,
    pub created_at: String,
    pub read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationCategory {
    Info,
    Warning,
    Error,
    Success,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

/// Query parameters for WebSocket connection
#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: String,
}

/// WebSocket handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<WsState>,
) -> Response {
    // Validate JWT token and extract user_id
    let user_id = match validate_token(&query.token).await {
        Ok(uid) => uid,
        Err(e) => {
            warn!("WebSocket auth failed: {}", e);
            return (axum::http::StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
    };

    info!("WebSocket connection established for user: {}", user_id);

    ws.on_upgrade(move |socket| handle_socket(socket, user_id, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, user_id: Uuid, state: WsState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    // Add connection to active connections
    let conn_id = Uuid::new_v4();
    {
        let mut connections = state.connections.write().await;
        connections.push(conn_id);
    }

    info!(
        "User {} connected via WebSocket (conn_id: {})",
        user_id, conn_id
    );

    // Send initial notifications (unread from DB)
    if let Err(e) = send_initial_notifications(&mut sender, user_id, &state.pool).await {
        error!("Failed to send initial notifications: {}", e);
    }

    // Channel for sending messages from receiver task to sender task
    let (tx_msg, mut rx_msg) = tokio::sync::mpsc::unbounded_channel::<Message>();

    // Spawn task to receive broadcast messages and send to client
    let tx_msg_clone = tx_msg.clone();
    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Receive from broadcast channel
                Ok(msg) = rx.recv() => {
                    // Only send notifications for this user
                    if msg.user_id == user_id {
                        let json = match serde_json::to_string(&msg) {
                            Ok(j) => j,
                            Err(e) => {
                                error!("Failed to serialize notification: {}", e);
                                continue;
                            }
                        };

                        if sender.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                }
                // Receive from internal channel (for pong messages)
                Some(msg) = rx_msg.recv() => {
                    if sender.send(msg).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Spawn task to receive client messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    info!("Received message from user {}: {}", user_id, text);
                    // Handle client messages (e.g., mark as read)
                    if let Err(e) = handle_client_message(&text, user_id, &state.pool).await {
                        error!("Failed to handle client message: {}", e);
                    }
                }
                Message::Close(_) => {
                    info!("User {} closed WebSocket connection", user_id);
                    break;
                }
                Message::Ping(data) => {
                    // Respond to ping with pong via channel
                    let _ = tx_msg_clone.send(Message::Pong(data));
                }
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    // Remove connection from active connections
    {
        let mut connections = state.connections.write().await;
        connections.retain(|&id| id != conn_id);
    }

    info!(
        "User {} disconnected from WebSocket (conn_id: {})",
        user_id, conn_id
    );
}

/// Send initial unread notifications to newly connected client
async fn send_initial_notifications(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    user_id: Uuid,
    pool: &Pool,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let rows = client
        .query(
            "SELECT id, user_id, title, body, category, priority, action_url, created_at, read
             FROM notifications
             WHERE user_id = $1 AND read = false
             ORDER BY created_at DESC
             LIMIT 50",
            &[&user_id],
        )
        .await?;

    for row in rows {
        let notification = NotificationMessage {
            id: row.get("id"),
            user_id: row.get("user_id"),
            title: row.get("title"),
            body: row.get("body"),
            category: serde_json::from_value(row.get("category"))?,
            priority: serde_json::from_value(row.get("priority"))?,
            action_url: row.get("action_url"),
            created_at: row
                .get::<_, chrono::DateTime<chrono::Utc>>("created_at")
                .to_rfc3339(),
            read: row.get("read"),
        };

        let json = serde_json::to_string(&notification)?;
        sender.send(Message::Text(json.into())).await?;
    }

    Ok(())
}

/// Handle messages from client
async fn handle_client_message(
    text: &str,
    user_id: Uuid,
    pool: &Pool,
) -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Deserialize)]
    struct ClientMessage {
        action: String,
        notification_id: Option<Uuid>,
    }

    let msg: ClientMessage = serde_json::from_str(text)?;

    match msg.action.as_str() {
        "mark_read" => {
            if let Some(notif_id) = msg.notification_id {
                mark_notification_read(notif_id, user_id, pool).await?;
            }
        }
        "mark_all_read" => {
            mark_all_notifications_read(user_id, pool).await?;
        }
        _ => {
            warn!("Unknown action: {}", msg.action);
        }
    }

    Ok(())
}

/// Mark notification as read
async fn mark_notification_read(
    notification_id: Uuid,
    user_id: Uuid,
    pool: &Pool,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    client
        .execute(
            "UPDATE notifications SET read = true, read_at = NOW()
             WHERE id = $1 AND user_id = $2",
            &[&notification_id, &user_id],
        )
        .await?;

    info!(
        "Marked notification {} as read for user {}",
        notification_id, user_id
    );

    Ok(())
}

/// Mark all notifications as read for user
async fn mark_all_notifications_read(
    user_id: Uuid,
    pool: &Pool,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let count = client
        .execute(
            "UPDATE notifications SET read = true, read_at = NOW()
             WHERE user_id = $1 AND read = false",
            &[&user_id],
        )
        .await?;

    info!(
        "Marked {} notifications as read for user {}",
        count, user_id
    );

    Ok(())
}

/// Validate JWT token and extract user_id
async fn validate_token(token: &str) -> Result<Uuid, Box<dyn std::error::Error>> {
    // TODO: Implement proper JWT validation with authenc service
    // For now, just parse as UUID for testing
    // In production, this should:
    // 1. Verify JWT signature
    // 2. Check expiration
    // 3. Extract user_id from claims

    // Temporary implementation for testing
    if token.starts_with("test_") {
        // Extract UUID from test token
        let uuid_str = token.strip_prefix("test_").unwrap_or(token);
        Ok(Uuid::parse_str(uuid_str)?)
    } else {
        // TODO: Call authenc service to validate token
        Err("Token validation not implemented".into())
    }
}

/// Broadcast notification to all connected clients
pub async fn broadcast_notification(
    state: &WsState,
    notification: NotificationMessage,
) -> Result<(), Box<dyn std::error::Error>> {
    // Save to database first
    let client = state.pool.get().await?;

    client
        .execute(
            "INSERT INTO notifications (id, user_id, title, body, category, priority, action_url, created_at, read)
             VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), false)",
            &[
                &notification.id,
                &notification.user_id,
                &notification.title,
                &notification.body,
                &serde_json::to_value(&notification.category)?,
                &serde_json::to_value(&notification.priority)?,
                &notification.action_url,
            ],
        )
        .await?;

    // Broadcast to connected clients
    let _ = state.tx.send(notification);

    Ok(())
}
