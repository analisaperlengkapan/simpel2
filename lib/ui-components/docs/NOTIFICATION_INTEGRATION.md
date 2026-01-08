# Notification System Integration Guide

## Quick Start

### For Frontend Developers

#### 1. Using NotificationCenter Component

The `NotificationCenter` component is already integrated in the navbar. No additional setup required!

```rust
use crate::components::navigation::NotificationCenter;

view! {
    <nav>
        // ... other nav items
        <NotificationCenter />
    </nav>
}
```

#### 2. Using NotificationsPage

The full notification history page is available at `/notifications`:

```rust
use crate::pages::NotificationsPage;

view! {
    <Route path="/notifications" view=move || view! {
        <NotificationsPage
            user_session=user_session.get().unwrap()
            on_logout=Box::new(handle_logout)
        />
    } />
}
```

#### 3. Custom Notification Integration

To integrate notifications in your own component:

```rust
use crate::features::websocket::start_mock_notification_generator;
use crate::components::navigation::Notification;
use leptos::prelude::*;

#[component]
pub fn MyComponent() -> impl IntoView {
    let (notifications, set_notifications) = signal(Vec::new());

    // Start receiving notifications
    Effect::new(move |_| {
        start_mock_notification_generator(move |notification| {
            set_notifications.update(|notifs| {
                notifs.insert(0, notification);
            });
        });
    });

    view! {
        <div>
            <h2>"My Notifications"</h2>
            <For
                each=move || notifications.get()
                key=|n| n.id.clone()
                children=|notif| view! {
                    <div>{notif.title}</div>
                }
            />
        </div>
    }
}
```

### For Backend Developers

#### 1. WebSocket Endpoint Setup

Create a WebSocket endpoint in your Axum application:

```rust
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query,
    },
    response::Response,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

#[derive(Deserialize)]
struct WsQuery {
    token: String,
}

#[derive(Clone, Serialize)]
struct Notification {
    id: String,
    title: String,
    message: String,
    category: String,
    timestamp: String,
    read: bool,
}

#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WsMessage {
    Notification { notification: Notification },
    Ping,
    Pong,
    Connected { session_id: String },
    Error { message: String },
}

// Shared state for managing connections
type Connections = Arc<RwLock<HashMap<String, mpsc::UnboundedSender<WsMessage>>>>;

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    connections: axum::extract::State<Connections>,
) -> Response {
    // Validate JWT token
    let user_id = match validate_jwt(&query.token) {
        Ok(claims) => claims.sub,
        Err(_) => {
            return Response::builder()
                .status(401)
                .body("Unauthorized".into())
                .unwrap()
        }
    };

    ws.on_upgrade(move |socket| handle_socket(socket, user_id, connections))
}

async fn handle_socket(socket: WebSocket, user_id: String, connections: Connections) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Register connection
    {
        let mut conns = connections.write().await;
        conns.insert(user_id.clone(), tx);
    }

    // Send connected message
    let session_id = uuid::Uuid::new_v4().to_string();
    let connected_msg = WsMessage::Connected { session_id };
    if let Ok(json) = serde_json::to_string(&connected_msg) {
        let _ = sender.send(Message::Text(json)).await;
    }

    // Spawn task to send messages
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Spawn task to receive messages
    let user_id_clone = user_id.clone();
    let connections_clone = connections.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                // Handle ping/pong
                if text.contains("\"type\":\"ping\"") {
                    // Send pong response
                    let pong = WsMessage::Pong;
                    if let Ok(json) = serde_json::to_string(&pong) {
                        let conns = connections_clone.read().await;
                        if let Some(tx) = conns.get(&user_id_clone) {
                            let _ = tx.send(WsMessage::Pong);
                        }
                    }
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    // Clean up connection
    {
        let mut conns = connections.write().await;
        conns.remove(&user_id);
    }
}

// JWT validation (implement based on your auth system)
fn validate_jwt(token: &str) -> Result<Claims, String> {
    // Implement JWT validation
    // Return user claims on success
    todo!()
}

#[derive(Debug)]
struct Claims {
    sub: String,
    // ... other claims
}

// Create router
pub fn create_router() -> Router {
    let connections: Connections = Arc::new(RwLock::new(HashMap::new()));

    Router::new()
        .route("/ws/notifications", get(ws_handler))
        .with_state(connections)
}
```

#### 2. Broadcasting Notifications

Create a notification service to broadcast messages:

```rust
use tokio::sync::mpsc;

pub struct NotificationService {
    connections: Connections,
}

impl NotificationService {
    pub fn new(connections: Connections) -> Self {
        Self { connections }
    }

    /// Send notification to specific user
    pub async fn send_to_user(&self, user_id: &str, notification: Notification) {
        let conns = self.connections.read().await;
        if let Some(tx) = conns.get(user_id) {
            let msg = WsMessage::Notification { notification };
            let _ = tx.send(msg);
        }
    }

    /// Broadcast notification to all connected users
    pub async fn broadcast(&self, notification: Notification) {
        let conns = self.connections.read().await;
        for tx in conns.values() {
            let msg = WsMessage::Notification {
                notification: notification.clone(),
            };
            let _ = tx.send(msg);
        }
    }

    /// Send notification to users with specific role
    pub async fn send_to_role(&self, role: &str, notification: Notification) {
        // Implement role-based filtering
        // This requires storing user roles with connections
        todo!()
    }
}
```

#### 3. Triggering Notifications

Trigger notifications from your business logic:

```rust
// Example: Send notification when document is updated
pub async fn update_document(
    doc_id: &str,
    user_id: &str,
    notification_service: &NotificationService,
) -> Result<(), Error> {
    // Update document logic...

    // Send notification
    let notification = Notification {
        id: uuid::Uuid::new_v4().to_string(),
        title: "Dokumen Diperbarui".to_string(),
        message: format!("Dokumen {} telah diperbarui", doc_id),
        category: "success".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        read: false,
    };

    notification_service.send_to_user(user_id, notification).await;

    Ok(())
}

// Example: Broadcast system maintenance notification
pub async fn schedule_maintenance(
    notification_service: &NotificationService,
) -> Result<(), Error> {
    let notification = Notification {
        id: uuid::Uuid::new_v4().to_string(),
        title: "Pemeliharaan Sistem Terjadwal".to_string(),
        message: "Sistem akan diperbarui pada 20 Oktober 2025 pukul 02:00 WIB".to_string(),
        category: "info".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        read: false,
    };

    notification_service.broadcast(notification).await;

    Ok(())
}
```

## Development Mode

### Using Mock Notifications

For development without a backend, the system uses mock notifications:

```rust
// Mock notifications are automatically generated every 10 seconds
// No additional setup required!

// To customize mock behavior:
use crate::features::websocket::start_mock_notification_generator;

start_mock_notification_generator(|notification| {
    // Custom handling
    println!("Mock: {}", notification.title);
});
```

### Switching to Real WebSocket

To switch from mock to real WebSocket:

1. **Set environment variable**:
```bash
export WS_NOTIFICATION_URL=ws://localhost:3000/ws/notifications
```

2. **Update Trunk.toml**:
```toml
[build]
public_url = "/"

[[proxy]]
backend = "http://localhost:3000/ws/"
```

3. **Start backend WebSocket server**:
```bash
cd backend
cargo run --bin notification-service
```

## Production Deployment

### 1. Environment Configuration

**Frontend** (`.env.production`):
```bash
WS_NOTIFICATION_URL=wss://api.simpelv2.kejaksaan.go.id/ws/notifications
```

**Backend** (`.env`):
```bash
WS_HOST=0.0.0.0
WS_PORT=3000
JWT_SECRET=your-secret-key
CORS_ORIGINS=https://portal.simpelv2.kejaksaan.go.id
```

### 2. Nginx Configuration

```nginx
# WebSocket proxy
location /ws/ {
    proxy_pass http://notification-service:3000;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;

    # Timeouts for long-lived connections
    proxy_connect_timeout 7d;
    proxy_send_timeout 7d;
    proxy_read_timeout 7d;
}
```

### 3. Docker Deployment

**Dockerfile** (notification service):
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin notification-service

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/notification-service /usr/local/bin/
EXPOSE 3000
CMD ["notification-service"]
```

**docker-compose.yml**:
```yaml
services:
  notification-service:
    build: ./backend
    ports:
      - "3000:3000"
    environment:
      - WS_HOST=0.0.0.0
      - WS_PORT=3000
      - JWT_SECRET=${JWT_SECRET}
    restart: unless-stopped
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_creation() {
        let notif = Notification {
            id: "1".to_string(),
            title: "Test".to_string(),
            message: "Test message".to_string(),
            category: NotificationCategory::Info,
            timestamp: "now".to_string(),
            read: false,
        };

        assert_eq!(notif.id, "1");
        assert!(!notif.read);
    }

    #[test]
    fn test_notification_category_icon() {
        assert_eq!(NotificationCategory::Info.icon(), "ℹ️");
        assert_eq!(NotificationCategory::Warning.icon(), "⚠️");
        assert_eq!(NotificationCategory::Error.icon(), "❌");
        assert_eq!(NotificationCategory::Success.icon(), "✅");
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_websocket_connection() {
    let connections = Arc::new(RwLock::new(HashMap::new()));
    let service = NotificationService::new(connections.clone());

    // Simulate user connection
    let (tx, mut rx) = mpsc::unbounded_channel();
    {
        let mut conns = connections.write().await;
        conns.insert("user1".to_string(), tx);
    }

    // Send notification
    let notif = Notification {
        id: "1".to_string(),
        title: "Test".to_string(),
        message: "Test".to_string(),
        category: "info".to_string(),
        timestamp: "now".to_string(),
        read: false,
    };

    service.send_to_user("user1", notif).await;

    // Verify notification received
    let msg = rx.recv().await.unwrap();
    assert!(matches!(msg, WsMessage::Notification { .. }));
}
```

## Troubleshooting

### Common Issues

#### 1. WebSocket Connection Fails

**Symptoms**: Bell icon shows no notifications, console shows connection errors

**Solutions**:
- Check `WS_NOTIFICATION_URL` environment variable
- Verify backend WebSocket service is running
- Check JWT token is valid
- Inspect browser console for detailed errors
- Verify Nginx WebSocket proxy configuration

#### 2. Notifications Not Appearing

**Symptoms**: Connection established but no notifications received

**Solutions**:
- Check backend notification broadcasting logic
- Verify user ID matches between frontend and backend
- Check notification service is properly initialized
- Inspect backend logs for errors
- Verify message format matches protocol

#### 3. High Memory Usage

**Symptoms**: Browser memory increases over time

**Solutions**:
- Reduce notification history limit (default: 100)
- Implement pagination for notification history
- Clear old notifications periodically
- Check for memory leaks in notification handlers

## Best Practices

### Frontend

1. **Limit Notifications**: Keep max 50-100 notifications in memory
2. **Debounce Updates**: Batch notification updates to reduce re-renders
3. **Lazy Loading**: Load notification history on demand
4. **Error Handling**: Gracefully handle connection failures
5. **User Feedback**: Show connection status to users

### Backend

1. **Authentication**: Always validate JWT tokens
2. **Rate Limiting**: Limit notification frequency per user
3. **Message Validation**: Validate all incoming messages
4. **Connection Limits**: Set max connections per user
5. **Monitoring**: Track connection count, message rate, errors
6. **Graceful Shutdown**: Close connections cleanly on shutdown

## Support

For questions or issues:
- Check documentation: `antarmuka/portal/docs/NOTIFICATION_SYSTEM.md`
- Review code: `antarmuka/portal/src/features/websocket.rs`
- Contact: SIMPelv2 Development Team
