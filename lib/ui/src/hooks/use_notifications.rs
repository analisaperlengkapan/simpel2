//! Notification hook — per-user in-app inbox.
//!
//! Reads the REAL notifikasi inbox served by the notifikasi service at
//! `/api/v1/perlengkapan/notifikasi/*` (same-origin; every FE's nginx/ingress
//! proxies that prefix). Identity comes from the shared `auth_token` JWT —
//! the backend scopes rows by `claims.user_id`.
//!
//! The previous implementation connected to a `/ws/notifications` endpoint no
//! backend ever served and fell back to a MOCK generator that fabricated
//! notifications ("Peringatan Keamanan", …) client-side — removed in the
//! 2026-07 portal audit: the UI must only ever show real inbox rows, and an
//! unreachable inbox must surface as `SyncState::Unavailable`, never as fake
//! data.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Base path of the notifikasi inbox API (origin-relative). Only the wasm
/// build performs fetches; the host build compiles the types only.
#[cfg(target_arch = "wasm32")]
const NOTIFIKASI_API: &str = "/api/v1/perlengkapan/notifikasi";

/// Notification data structure (UI shape)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Notification {
    /// Unique identifier for the notification
    pub id: String,
    /// Title of the notification
    pub title: String,
    /// Detailed message content
    pub message: String,
    /// Category type of the notification
    pub category: NotificationCategory,
    /// Priority level
    pub priority: NotificationPriority,
    /// Optional action URL
    pub action_url: Option<String>,
    /// Timestamp when the notification was created
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

/// Notification category
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

/// Inbox synchronisation state
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SyncState {
    /// Initial fetch in flight
    Loading,
    /// Inbox fetched successfully (list may be empty)
    Ready,
    /// Inbox could not be fetched (not logged in / service unreachable).
    /// The UI shows the honest empty/error state — never fabricated data.
    Unavailable,
}

/// Notification context for sharing state across components
#[derive(Clone, Copy)]
pub struct NotificationContext {
    /// List of notifications
    pub notifications: RwSignal<Vec<Notification>>,
    /// Inbox synchronisation state
    pub sync_state: RwSignal<SyncState>,
}

impl NotificationContext {
    /// Get unread notification count
    pub fn unread_count(&self) -> usize {
        self.notifications
            .with(|notifs| notifs.iter().filter(|n| !n.read).count())
    }

    /// Mark notification as read (optimistic local update + persist)
    pub fn mark_as_read(&self, id: &str) {
        self.notifications.update(|notifs| {
            if let Some(notif) = notifs.iter_mut().find(|n| n.id == id) {
                notif.read = true;
            }
        });

        #[cfg(target_arch = "wasm32")]
        {
            let url = format!("{}/{}/read", NOTIFIKASI_API, id);
            leptos::task::spawn_local(async move {
                let _ = authed_request(gloo::net::http::Method::PATCH, &url).await;
            });
        }
    }

    /// Mark all notifications as read (optimistic local update + persist)
    pub fn mark_all_as_read(&self) {
        self.notifications.update(|notifs| {
            for notif in notifs.iter_mut() {
                notif.read = true;
            }
        });

        #[cfg(target_arch = "wasm32")]
        {
            let url = format!("{}/read-all", NOTIFIKASI_API);
            leptos::task::spawn_local(async move {
                let _ = authed_request(gloo::net::http::Method::POST, &url).await;
            });
        }
    }

    /// Re-fetch the inbox from the backend.
    pub fn refresh(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            let notifications = self.notifications;
            let sync_state = self.sync_state;
            leptos::task::spawn_local(async move {
                fetch_inbox(notifications, sync_state).await;
            });
        }
    }
}

/// Hook to use notification system
///
/// # Example
/// ```rust
/// use lib_ui::hooks::use_notifications::use_notifications;
/// use leptos::prelude::*;
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
    let sync_state = RwSignal::new(SyncState::Loading);

    let ctx = NotificationContext {
        notifications,
        sync_state,
    };

    // Initial inbox fetch
    #[cfg(target_arch = "wasm32")]
    {
        Effect::new(move |_| {
            leptos::task::spawn_local(async move {
                fetch_inbox(notifications, sync_state).await;
            });
        });
    }

    // Provide context for child components
    provide_context(ctx);

    ctx
}

// ── Backend wire types + fetch (wasm only) ─────────────────────────────────

/// One inbox row as served by the notifikasi service (`NotifikasiItem`).
/// `category`/`priority` are free-form strings server-side, so they are mapped
/// leniently — an unknown value must not fail the whole inbox.
#[cfg(target_arch = "wasm32")]
#[derive(Deserialize)]
struct NotifikasiWire {
    id: String,
    title: String,
    message: String,
    #[serde(default)]
    priority: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    action_url: Option<String>,
    #[serde(default)]
    read: bool,
    #[serde(default)]
    created_at: String,
}

#[cfg(target_arch = "wasm32")]
impl From<NotifikasiWire> for Notification {
    fn from(w: NotifikasiWire) -> Self {
        let category = match w.category.to_lowercase().as_str() {
            "warning" | "peringatan" => NotificationCategory::Warning,
            "error" => NotificationCategory::Error,
            "success" | "sukses" => NotificationCategory::Success,
            "system" | "sistem" => NotificationCategory::System,
            _ => NotificationCategory::Info,
        };
        let priority = match w.priority.to_lowercase().as_str() {
            "low" => NotificationPriority::Low,
            "high" => NotificationPriority::High,
            "urgent" => NotificationPriority::Urgent,
            _ => NotificationPriority::Normal,
        };
        Notification {
            id: w.id,
            title: w.title,
            message: w.message,
            category,
            priority,
            action_url: w.action_url,
            timestamp: w.created_at,
            read: w.read,
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn auth_token() -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|s| s.get_item("auth_token").ok().flatten())
}

/// Send an authenticated request; Err on missing token / network / non-2xx.
#[cfg(target_arch = "wasm32")]
async fn authed_request(
    method: gloo::net::http::Method,
    url: &str,
) -> Result<gloo::net::http::Response, ()> {
    let token = auth_token().ok_or(())?;
    let resp = gloo::net::http::RequestBuilder::new(url)
        .method(method)
        .header("Authorization", &format!("Bearer {}", token))
        .build()
        .map_err(|_| ())?
        .send()
        .await
        .map_err(|_| ())?;
    if resp.ok() { Ok(resp) } else { Err(()) }
}

#[cfg(target_arch = "wasm32")]
async fn fetch_inbox(notifications: RwSignal<Vec<Notification>>, sync_state: RwSignal<SyncState>) {
    #[derive(Deserialize)]
    struct ListWrap {
        data: Vec<NotifikasiWire>,
    }

    let url = format!("{}?limit=50&offset=0&unread_only=false", NOTIFIKASI_API);
    match authed_request(gloo::net::http::Method::GET, &url).await {
        Ok(resp) => match resp.json::<ListWrap>().await {
            Ok(wrap) => {
                notifications.set(wrap.data.into_iter().map(Notification::from).collect());
                sync_state.set(SyncState::Ready);
            }
            Err(_) => sync_state.set(SyncState::Unavailable),
        },
        Err(()) => sync_state.set(SyncState::Unavailable),
    }
}
