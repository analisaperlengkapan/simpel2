# Notification System - Shared Library

## Overview

Sistem notifikasi real-time yang dapat digunakan di semua microfrontend SIMPelv2. Menggunakan WebSocket untuk push notifications dengan fallback ke mock generator untuk development.

## Components

### 1. `NotificationBell`

Komponen bell icon dengan dropdown untuk menampilkan notifikasi terbaru.

**Usage**:
```rust
use shared_microfrontend::components::NotificationBell;

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <nav>
            <NotificationBell />
        </nav>
    }
}
```

**Props**:
- `max_display: Option<usize>` - Maksimal notifikasi yang ditampilkan di dropdown (default: 10)
- `class: Option<String>` - Custom CSS class

### 2. `NotificationList`

Komponen untuk menampilkan daftar lengkap notifikasi dengan filtering.

**Usage**:
```rust
use shared_microfrontend::components::NotificationList;

#[component]
pcationsPage() -> impl IntoView {
    view! {
        <div class="container">
            <h1>"Notifikasi"</h1>
            <NotificationList show_filters=true />
        </div>
    }
}
```

**Props**:
- `show_filters: bool` - Tampilkan tombol filter kategori

### 3. `NotificationConnectionStatus`

Indikator status koneksi WebSocket.

**Usage**:
```rust
use shared_microfrontend::components::NotificationConnectionStatus;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer>
            <NotificationConnectionStatus />
        </footer>
    }
}
```

## Hooks

### `use_notifications()`

Hook untuk mengakses notification context.

**Usage**:
```rust
use shared_microfrontend::hooks::use_notifications;

#[component]
pub fn MyComponent() -> impl IntoView {
    let notif_ctx = use_notifications();

    // Get unread count
    let unread = notif_ctx.unread_count();

    // Mark as read
    notif_ctx.mark_as_read("notification-id");

    // Mark all as read
    notif_ctx.mark_all_as_read();

    // Add notification
    notif_ctx.add_notification(notification);

    view! {
        <div>"Unread: " {unread}</div>
    }
}
```

## Types

### `Notification`

```rust
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub category: NotificationCategory,
    pub timestamp: String,
    pub read: bool,
}
```

### `NotificationCategory`

```rust
pub enum NotificationCategory {
    Info,      // ℹ️ Blue
    Warning,   // ⚠️ Yellow
    Error,     // ❌ Red
    Success,   // ✅ Green
}
```

### `WsState`

```rust
pub enum WsState {
    Disconnected,  // Not connected
    Connecting,    // Connecting to server
    Connected,     // Connected and ready
    Error,         // Connection error
}
```

## Configuration

### Environment Variables

```bash
# WebSocket URL for notifications
WS_NOTIFICATION_URL=wss://api.simpelv2.kejaksaan.go.id/ws/notifications

# For development (mock mode)
WS_NOTIFICATION_URL=mock
```

### Mock Mode

Jika `WS_NOTIFICATION_URL` tidak diset, kosong, atau "mock", sistem akan menggunakan mock notification generator yang menghasilkan notifikasi setiap 10 detik.

## Integration Examples

### Portal

```rust
// In navbar
use shared_microfrontend::components::NotificationBell;

view! {
    <nav>
        <NotificationBell />
    </nav>
}

// In notifications page
use shared_microfrontend::components::NotificationList;

view! {
    <div class="container">
        <NotificationList show_filters=true />
    </div>
}
```

### Microfrontend Apps

```rust
// In app navbar
use shared_microfrontend::components::NotificationBell;

#[component]
pub fn AppNavbar() -> impl IntoView {
    view! {
        <nav class="app-navbar">
            <div class="nav-left">
                <h1>"My App"</h1>
            </div>
            <div class="nav-right">
                <NotificationBell />
                <UserProfile />
                <LogoutButton />
            </div>
        </nav>
    }
}
```

## Features

✅ **Real-time Updates**: WebSocket-based push notifications
✅ **Auto-Reconnection**: Automatic reconnection with exponential backoff
✅ **Mock Mode**: Development-friendly mock generator
✅ **Cross-App**: Shared state across all microfrontends
✅ **Type Safe**: Full Rust type safety
✅ **Responsive**: Mobile-optimized UI
✅ **Accessible**: WCAG 2.1 AA compliant
✅ **Dark Mode**: Full dark mode support

## Backend Integration

### WebSocket Endpoint

**URL**: `wss://api.simpelv2.kejaksaan.go.id/ws/notifications?token={jwt}`

**Message Format**:
```json
{
  "type": "notification",
  "notification": {
    "id": "notif-123",
    "title": "Pembaruan Sistem",
    "message": "Sistem akan diperbarui...",
    "category": "info",
    "timestamp": "2 jam yang lalu",
    "read": false
  }
}
```

### Broadcasting Notifications

Backend dapat mengirim notifikasi ke:
- User tertentu (by user_id)
- Semua user (broadcast)
- User dengan role tertentu (by role)

## Testing

### Development

```bash
# Start portal with mock notifications
cd antarmuka/portal
trunk serve

# Mock notifications akan muncul setiap 10 detik
```

### Production

```bash
# Set WebSocket URL
export WS_NOTIFICATION_URL=wss://api.simpelv2.kejaksaan.go.id/ws/notifications

# Build and deploy
trunk build --release
```

## Troubleshooting

### Notifications Not Appearing

1. Check WebSocket connection status
2. Verify `WS_NOTIFICATION_URL` is set correctly
3. Check browser console for errors
4. Verify JWT token is valid

### High Memory Usage

1. Notifications are automatically limited to 100
2. Old notifications are pruned automatically
3. Check for memory leaks in custom handlers

## Best Practices

1. **Use NotificationBell in navbar** - Consistent UX across apps
2. **Limit custom styling** - Use provided components as-is
3. **Don't store notifications** - Use shared context
4. **Handle connection errors** - Show fallback UI
5. **Test with mock mode** - No backend required

## Migration from Portal-Only

Jika sebelumnya menggunakan notification system di portal saja:

**Before**:
```rust
use crate::components::navigation::NotificationCenter;
```

**After**:
```rust
use shared_microfrontend::components::NotificationBell;
```

Semua functionality sama, hanya lokasi import yang berubah.

## Support

- **Documentation**: `antarmuka/shared/docs/NOTIFICATIONS.md`
- **Source**: `antarmuka/shared/src/hooks/use_notifications.rs`
- **Components**: `antarmuka/shared/src/components/notifications.rs`

