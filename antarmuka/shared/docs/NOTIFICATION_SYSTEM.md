# Real-time Notification System

## Overview

The Portal SIMPelv2 notification system provides real-time push notifications to users through WebSocket connections. This enables instant delivery of important system events, updates, and alerts without requiring page refreshes.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      User Browser                            │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Portal Fronten + WASM)                       │ │
│  │  ┌──────────────────┐  ┌──────────────────────────┐   │ │
│  │  │ NotificationCenter│  │ NotificationsPage        │   │ │
│  │  │ (Dropdown)        │  │ (Full History)           │   │ │
│  │  └────────┬──────────┘  └──────────┬───────────────┘   │ │
│  │           │                        │                    │ │
│  │           └────────────┬───────────┘                    │ │
│  │                        │                                │ │
│  │           ┌────────────▼────────────────┐               │ │
│  │           │ NotificationWebSocket       │               │ │
│  │           │ - Connection management     │               │ │
│  │           │ - Auto-reconnection         │               │ │
│  │           │ - Message parsing           │               │ │
│  │           └────────────┬────────────────┘               │ │
│  └────────────────────────┼────────────────────────────────┘ │
└───────────────────────────┼──────────────────────────────────┘
                            │ WebSocket (wss://)
                            │ + JWT Authentication
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                  Backend Services                            │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Notification Service (Rust + Axum)                    │ │
│  │  - WebSocket endpoint: /ws/notifications               │ │
│  │  - JWT token validation                                │ │
│  │  - User session management                             │ │
│  │  - Message broadcasting                                │ │
│  └────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Event Sources                                         │ │
│  │  - System events (updates, maintenance)                │ │
│  │  - User actions (document updates, task completion)    │ │
│  │  - Security alerts (login attempts, password changes)  │ │
│  │  - Application events (from microfrontends)            │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Components

### 1. NotificationWebSocket Service

**Location**: `antarmuka/portal/src/features/websocket.rs`

**Responsibilities**:
- Establish and maintain WebSocket connection
- Handle connection lifecycle (connect, disconnect, reconnect)
- Parse incoming messages
- Deliver notifications to subscribers
- Implement exponential backoff for reconnection
- Send keep-alive pings

**Key Features**:
- **Auto-reconnection**: Automatically reconnects with exponential backoff (1s, 2s, 4s, 8s, 16s)
- **JWT Authentication**: Includes access token in WebSocket URL
- **Keep-alive**: Sends ping every 30 seconds to maintain connection
- **Error Handling**: Graceful error handling with logging

**Usage**:
```rust
use crate::features::websocket::NotificationWebSocket;

let mut ws = NotificationWebSocket::new(|notification| {
    // Handle incoming notification
    println!("Received: {}", notification.title);
});

ws.connect("wss://api.simpelv2.kejaksaan.go.id/ws/notifications", "jwt_token")?;
```

### 2. NotificationCenter Component

**Location**: `antarmuka/portal/src/components/navigation/notification_center.rs`

**Responsibilities**:
- Display notification bell icon with unread count badge
- Show dropdown with recent notifications (max 10)
- Mark notifications as read
- Link to full notification history page

**Features**:
- Real-time updates via WebSocket
- Unread count badge
- Category-based styling (Info, Warning, Error, Success)
- Mark individual or all notifications as read
- Responsive design

**Integration**:
```rust
use crate::components::navigation::NotificationCenter;

view! {
    <nav>
        <NotificationCenter />
    </nav>
}
```

### 3. NotificationsPage

**Location**: `antarmuka/portal/src/pages/notifications.rs`

**Responsibilities**:
- Display full notification history (up to 100 notifications)
- Filter by category (Info, Warning, Error, Success)
- Mark notifications as read
- Show notification details

**Features**:
- Category filtering
- Mark all as read
- Real-time updates
- Empty state handling
- Responsive layout

## Message Protocol

### WebSocket Message Format

All messages are JSON with a `type` field:

```typescript
type WsMessage =
  | { type: "notification", notification: Notification }
  | { type: "ping" }
  | { type: "pong" }
  | { type: "connected", session_id: string }
  | { type: "error", message: string };
```

### Notification Structure

```typescript
interface Notification {
  id: string;              // Unique identifier
  title: string;           // Notification title
  message: string;         // Detailed message
  category: NotificationCategory;  // Info | Warning | Error | Success
  timestamp: string;       // Human-readable timestamp
  read: boolean;           // Read status
}
```

### Example Messages

**Server → Client: New Notification**
```json
{
  "type": "notification",
  "notification": {
    "id": "notif-123",
    "title": "Pembaruan Sistem",
    "message": "Sistem akan diperbarui pada 20 Oktober 2025 pukul 02:00 WIB",
    "category": "info",
    "timestamp": "2 jam yang lalu",
    "read": false
  }
}
```

**Client → Server: Ping**
```json
{
  "type": "ping"
}
```

**Server → Client: Pong**
```json
{
  "type": "pong"
}
```

**Server → Client: Connection Established**
```json
{
  "type": "connected",
  "session_id": "sess-abc-123"
}
```

## Backend Implementation

### WebSocket Endpoint

**URL**: `wss://api.simpelv2.kejaksaan.go.id/ws/notifications?token={jwt_token}`

**Authentication**: JWT token in URL query parameter

**Example Axum Handler**:
```rust
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query,
    },
    response::Response,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct WsQuery {
    token: String,
}

async fn ws_notifications_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
) -> Response {
    // Validate JWT token
    let user_id = match validate_jwt(&query.token) {
        Ok(claims) => claims.sub,
        Err(_) => return Response::builder()
            .status(401)
            .body("Unauthorized".into())
            .unwrap(),
    };

    // Upgrade connection
    ws.on_upgrade(move |socket| handle_socket(socket, user_id))
}

async fn handle_socket(socket: WebSocket, user_id: String) {
    // Handle WebSocket connection
    // - Subscribe user to notification channel
    // - Send notifications as they arrive
    // - Handle ping/pong
    // - Clean up on disconnect
}
```

### Notification Broadcasting

**Example Service**:
```rust
pub struct NotificationService {
    connections: Arc<RwLock<HashMap<String, mpsc::Sender<Notification>>>>,
}

impl NotificationService {
    pub async fn broadcast(&self, user_id: &str, notification: Notification) {
        let connections = self.connections.read().await;
        if let Some(sender) = connections.get(user_id) {
            let _ = sender.send(notification).await;
        }
    }

    pub async fn broadcast_to_all(&self, notification: Notification) {
        let connections = self.connections.read().await;
        for sender in connections.values() {
            let _ = sender.send(notification.clone()).await;
        }
    }
}
```

## Configuration

### Environment Variables

**Frontend** (`.env` or build-time):
```bash
# WebSocket URL for notifications
WS_NOTIFICATION_URL=wss://api.simpelv2.kejaksaan.go.id/ws/notifications

# Fallback for development
WS_NOTIFICATION_URL=ws://localhost:3000/ws/notifications
```

**Backend**:
```bash
# WebSocket server configuration
WS_HOST=0.0.0.0
WS_PORT=3000
WS_MAX_CONNECTIONS=10000
WS_PING_INTERVAL=30
WS_TIMEOUT=60
```

### Nginx Configuration

```nginx
# WebSocket proxy configuration
location /ws/ {
    proxy_pass http://backend:3000;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;

    # Timeouts
    proxy_connect_timeout 7d;
    proxy_send_timeout 7d;
    proxy_read_timeout 7d;
}
```

## Testing

### Mock Notification Generator

For development and testing, a mock notification generator is provided:

```rust
use crate::features::websocket::start_mock_notification_generator;

// Start generating mock notifications every 10 seconds
start_mock_notification_generator(|notification| {
    println!("Mock notification: {}", notification.title);
});
```

### Manual Testing

1. **Start Portal**: `cd antarmuka/portal && trunk serve`
2. **Login**: Navigate to `/login` and authenticate
3. **Open Notification Center**: Click bell icon in navbar
4. **Observe**: Mock notifications appear every 10 seconds
5. **Test Features**:
   - Mark as read
   - Mark all as read
   - View full history at `/notifications`
   - Filter by category

### Integration Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_connection() {
        let mut ws = NotificationWebSocket::new(|_| {});
        assert!(ws.connect("ws://localhost:3000/ws/notifications", "test_token").is_ok());
        assert_eq!(ws.state(), WsState::Connecting);
    }

    #[tokio::test]
    async fn test_notification_parsing() {
        let json = r#"{"type":"notification","notification":{"id":"1","title":"Test","message":"Test message","category":"info","timestamp":"now","read":false}}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, WsMessage::Notification { .. }));
    }
}
```

## Security Considerations

### Authentication
- JWT token required for WebSocket connection
- Token validated on connection establishment
- Invalid tokens result in immediate connection closure

### Authorization
- Users only receive notifications intended for them
- Role-based notification filtering
- No cross-user notification leakage

### Rate Limiting
- Max 100 notifications per user in memory
- Older notifications automatically pruned
- Backend rate limiting on notification generation

### Data Privacy
- Notifications contain no sensitive data in transit
- HTTPS/WSS encryption required in production
- Audit logging of notification delivery

## Performance

### Optimization Strategies

1. **Connection Pooling**: Reuse WebSocket connections
2. **Message Batching**: Batch multiple notifications when possible
3. **Lazy Loading**: Load notification history on demand
4. **Pagination**: Limit notifications displayed at once
5. **Caching**: Cache read status locally

### Metrics

- **Connection Latency**: < 100ms
- **Message Delivery**: < 50ms
- **Reconnection Time**: < 5s (with exponential backoff)
- **Memory Usage**: < 10MB per 100 notifications

## Troubleshooting

### Connection Issues

**Problem**: WebSocket fails to connect

**Solutions**:
1. Check WebSocket URL configuration
2. Verify JWT token is valid
3. Check network connectivity
4. Inspect browser console for errors
5. Verify backend WebSocket endpoint is running

### Missing Notifications

**Problem**: Notifications not appearing

**Solutions**:
1. Check WebSocket connection status
2. Verify user has correct permissions
3. Check backend notification broadcasting
4. Inspect browser console for errors
5. Verify notification service is running

### High Memory Usage

**Problem**: Browser memory usage increases over time

**Solutions**:
1. Reduce notification history limit (currently 100)
2. Implement pagination for notification history
3. Clear old notifications periodically
4. Check for memory leaks in notification handlers

## Future Enhancements

### Planned Features

1. **Notification Preferences**: User-configurable notification settings
2. **Sound Alerts**: Optional sound for important notifications
3. **Desktop Notifications**: Browser notification API integration
4. **Notification Actions**: Actionable buttons in notifications
5. **Rich Content**: Support for images, links, and formatted text
6. **Notification Groups**: Group related notifications
7. **Snooze**: Temporarily dismiss notifications
8. **Priority Levels**: High/Medium/Low priority notifications

### Backend Improvements

1. **Persistent Storage**: Store notifications in database
2. **Delivery Guarantees**: Ensure notifications are delivered
3. **Retry Logic**: Retry failed notification deliveries
4. **Analytics**: Track notification engagement
5. **A/B Testing**: Test notification effectiveness
6. **Scheduled Notifications**: Send notifications at specific times

## References

- [WebSocket API (MDN)](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)
- [Leptos Documentation](https://leptos.dev/)
- [Axum WebSocket Example](https://github.com/tokio-rs/axum/tree/main/examples/websockets)
- [JWT Authentication](https://jwt.io/)

