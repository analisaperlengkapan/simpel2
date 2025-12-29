//! Notification hook for real-time notifications
//!
//! Provides WebSocket-based real-time notification delivery for all microfrontends

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};

/// Notification data structure (matches backend format)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Notification {
    /// Unique identifier for the notification
    pub id: String,
    /// User ID this notification belongs to
    pub user_id: String,
    /// Title of the notification
    pub title: String,
    /// Detailed message content (body in backend)
    #[serde(alias = "body")]
    pub message: String,
    /// Category type of the notification
    pub category: NotificationCategory,
    /// Priority level
    pub priority: NotificationPriority,
    /// Optional action URL
    pub action_url: Option<String>,
    /// Timestamp when the notification was created
    #[serde(alias = "created_at")]
    pub timestamp: String,
    /// Whether the notification has been read
    pub read: bool,
}

/// Notification priority
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationPriority {
    /// Low priority
    Low,
    /// Normal priority
    Normal,
    /// High priority
    High,
    /// Urgent priority
    Urgent,
}

/// Notification category (matches backend format)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationCategory {
    /// Informational notification
    Info,
    /// Warning notification
    Warning,
    /// Error notification
    Error,
    /// Success notification
    Success,
    /// System notification
    System,
}

impl NotificationCategory {
    /// Returns the icon emoji for this notification category
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Info => "ℹ️",
            Self::Warning => "⚠️",
            Self::Error => "❌",
            Self::Success => "✅",
            Self::System => "🔔",
        }
    }

    /// Returns CSS classes for styling this notification category
    pub fn color_classes(&self) -> &'static str {
        match self {
            Self::Info => "bg-blue-100 dark:bg-blue-900 text-blue-600 dark:text-blue-300",
            Self::Warning => {
                "bg-yellow-100 dark:bg-yellow-900 text-yellow-600 dark:text-yellow-300"
            }
            Self::Error => "bg-red-100 dark:bg-red-900 text-red-600 dark:text-red-300",
            Self::Success => "bg-green-100 dark:bg-green-900 text-green-600 dark:text-green-300",
            Self::System => "bg-gray-100 dark:bg-gray-900 text-gray-600 dark:text-gray-300",
        }
    }
}

/// Client-to-server message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientMessage {
    /// Action to perform
    pub action: String,
    /// Notification ID (for mark_read action)
    pub notification_id: Option<String>,
}

/// WebSocket connection state
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WsState {
    /// Not connected
    Disconnected,
    /// Connecting to server
    Connecting,
    /// Connected and ready
    Connected,
    /// Connection error
    Error,
}

/// Notification context for sharing state across components
#[derive(Clone, Copy)]
pub struct NotificationContext {
    /// List of notifications
    pub notifications: RwSignal<Vec<Notification>>,
    /// WebSocket connection state
    pub ws_state: RwSignal<WsState>,
}

// Global WebSocket reference for sending messages
#[cfg(target_arch = "wasm32")]
thread_local! {
    static WS_CONNECTION: std::cell::RefCell<Option<web_sys::WebSocket>> = std::cell::RefCell::new(None);
}

/// Send message via WebSocket
#[cfg(target_arch = "wasm32")]
fn send_ws_message(message: &str) {
    WS_CONNECTION.with(|ws| {
        if let Some(socket) = ws.borrow().as_ref() {
            let _ = socket.send_with_str(message);
        }
    });
}

impl NotificationContext {
    /// Get unread notification count
    pub fn unread_count(&self) -> usize {
        self.notifications
            .with(|notifs| notifs.iter().filter(|n| !n.read).count())
    }

    /// Mark notification as read (sends to server)
    pub fn mark_as_read(&self, id: &str) {
        // Update local state
        self.notifications.update(|notifs| {
            if let Some(notif) = notifs.iter_mut().find(|n| n.id == id) {
                notif.read = true;
            }
        });

        // Send to server via WebSocket
        #[cfg(target_arch = "wasm32")]
        {
            let message = ClientMessage {
                action: "mark_read".to_string(),
                notification_id: Some(id.to_string()),
            };

            if let Ok(json) = serde_json::to_string(&message) {
                send_ws_message(&json);
            }
        }
    }

    /// Mark all notifications as read (sends to server)
    pub fn mark_all_as_read(&self) {
        // Update local state
        self.notifications.update(|notifs| {
            for notif in notifs.iter_mut() {
                notif.read = true;
            }
        });

        // Send to server via WebSocket
        #[cfg(target_arch = "wasm32")]
        {
            let message = ClientMessage {
                action: "mark_all_read".to_string(),
                notification_id: None,
            };

            if let Ok(json) = serde_json::to_string(&message) {
                send_ws_message(&json);
            }
        }
    }

    /// Add new notification
    pub fn add_notification(&self, notification: Notification) {
        self.notifications.update(|notifs| {
            notifs.insert(0, notification);
            // Keep only last 100 notifications
            if notifs.len() > 100 {
                notifs.truncate(100);
            }
        });
    }

    /// Clear all notifications
    pub fn clear_all(&self) {
        self.notifications.update(|notifs| notifs.clear());
    }
}

/// Hook to use notification system
///
/// # Example
/// ```rust
/// use shared_microfrontend::hooks::use_notifications;
///
/// #[component]
/// pub fn MyComponent() -> impl IntoView {
///     let notif_ctx = use_notifications();
///
///     view! {
///         <div>
///             <p>"Unread: " {move || notif_ctx.unread_count()}</p>
///             <p>"Total: " {move || notif_ctx.notifications.get().len()}</p>
///         </div>
///     }
/// }
/// ```
pub fn use_notifications() -> NotificationContext {
    // Try to get existing context
    if let Some(ctx) = use_context::<NotificationContext>() {
        return ctx;
    }

    // Create new context
    let notifications = RwSignal::new(Vec::new());
    let ws_state = RwSignal::new(WsState::Disconnected);

    let ctx = NotificationContext {
        notifications,
        ws_state,
    };

    // Setup WebSocket connection
    #[cfg(target_arch = "wasm32")]
    {
        Effect::new(move |_| {
            setup_websocket_connection(notifications, ws_state);
        });
    }

    // Provide context for child components
    provide_context(ctx);

    ctx
}

/// Setup WebSocket connection for notifications
#[cfg(target_arch = "wasm32")]
fn setup_websocket_connection(
    notifications: RwSignal<Vec<Notification>>,
    ws_state: RwSignal<WsState>,
) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;
    use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket};
    // Get WebSocket URL from environment or default
    let ws_url = std::env::var("WS_NOTIFICATION_URL")
        .unwrap_or_else(|_| "ws://localhost:3000/ws/notifications".to_string());

    // Get access token from localStorage
    let access_token = if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        storage.get_item("auth_token").ok().flatten()
    } else {
        None
    };

    // If no token or mock mode, use mock generator
    if access_token.is_none() || ws_url.contains("mock") || ws_url.is_empty() {
        leptos::logging::log!("Using mock notification generator");
        start_mock_notification_generator(move |notification| {
            notifications.update(|notifs| {
                notifs.insert(0, notification);
                if notifs.len() > 100 {
                    notifs.truncate(100);
                }
            });
        });
        return;
    }

    let token = access_token.unwrap();

    // Create WebSocket connection
    let ws_url_with_token = format!("{}?token={}", ws_url, urlencoding::encode(&token));

    match WebSocket::new(&ws_url_with_token) {
        Ok(ws) => {
            ws_state.set(WsState::Connecting);
            ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

            // Store WebSocket connection globally
            WS_CONNECTION.with(|ws_ref| {
                *ws_ref.borrow_mut() = Some(ws.clone());
            });

            // On open handler
            {
                let onopen_callback = Closure::wrap(Box::new(move |_| {
                    leptos::logging::log!("WebSocket connected to notification service");
                    ws_state.set(WsState::Connected);
                }) as Box<dyn FnMut(JsValue)>);

                ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
                onopen_callback.forget();
            }

            // On message handler
            {
                let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                    if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                        let message_str = txt.as_string().unwrap_or_default();

                        // Backend sends notifications directly as JSON objects
                        if let Ok(notification) = serde_json::from_str::<Notification>(&message_str)
                        {
                            leptos::logging::log!("Received notification: {}", notification.title);
                            notifications.update(|notifs| {
                                notifs.insert(0, notification);
                                if notifs.len() > 100 {
                                    notifs.truncate(100);
                                }
                            });
                        } else {
                            leptos::logging::warn!("Failed to parse notification: {}", message_str);
                        }
                    }
                })
                    as Box<dyn FnMut(MessageEvent)>);

                ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
                onmessage_callback.forget();
            }

            // On error handler
            {
                let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
                    leptos::logging::error!("WebSocket error: {:?}", e);
                    ws_state.set(WsState::Error);
                })
                    as Box<dyn FnMut(ErrorEvent)>);

                ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
                onerror_callback.forget();
            }

            // On close handler
            {
                let onclose_callback = Closure::wrap(Box::new(move |e: CloseEvent| {
                    leptos::logging::log!(
                        "WebSocket closed: code={}, reason={}",
                        e.code(),
                        e.reason()
                    );
                    ws_state.set(WsState::Disconnected);
                })
                    as Box<dyn FnMut(CloseEvent)>);

                ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
                onclose_callback.forget();
            }

            // Setup keep-alive ping (WebSocket protocol handles this automatically)
            // No need for manual ping/pong
        }
        Err(e) => {
            leptos::logging::error!("Failed to create WebSocket: {:?}", e);
            ws_state.set(WsState::Error);

            // Fallback to mock generator
            start_mock_notification_generator(move |notification| {
                notifications.update(|notifs| {
                    notifs.insert(0, notification);
                    if notifs.len() > 100 {
                        notifs.truncate(100);
                    }
                });
            });
        }
    }
}

/// Mock notification generator for testing
/// Generates a random notification every 10 seconds
pub fn start_mock_notification_generator<F>(on_notification: F)
where
    F: Fn(Notification) + 'static + Clone,
{
    spawn_local(async move {
        let mut counter = 1;
        loop {
            gloo_timers::future::TimeoutFuture::new(10_000).await;

            let categories = [NotificationCategory::Info,
                NotificationCategory::Success,
                NotificationCategory::Warning,
                NotificationCategory::Error];

            let titles = ["Pembaruan Sistem",
                "Dokumen Baru",
                "Peringatan Keamanan",
                "Tugas Selesai",
                "Pesan Baru"];

            let messages = ["Sistem telah diperbarui ke versi terbaru",
                "Dokumen baru telah ditambahkan ke sistem",
                "Harap perbarui password Anda",
                "Tugas Anda telah selesai diproses",
                "Anda memiliki pesan baru dari administrator"];

            let category = categories[counter % categories.len()].clone();
            let title = titles[counter % titles.len()].to_string();
            let message = messages[counter % messages.len()].to_string();

            let notification = Notification {
                id: format!("mock-{}", counter),
                user_id: "mock-user".to_string(),
                title,
                message,
                category,
                priority: NotificationPriority::Normal,
                action_url: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
                read: false,
            };

            on_notification(notification);
            counter += 1;
        }
    });
}
