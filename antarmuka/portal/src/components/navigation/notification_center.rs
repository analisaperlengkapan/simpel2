//! Notification center component
//!
//! Dropdown component for displaying user notifications

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Notification data structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub category: NotificationCategory,
    pub timestamp: String,
    pub read: bool,
}

/// Notification category
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NotificationCategory {
    Info,
    Warning,
    Error,
    Success,
}

impl NotificationCategory {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Info => "ℹ️",
            Self::Warning => "⚠️",
            Self::Error => "❌",
            Self::Success => "✅",
        }
    }

    pub fn color_classes(&self) -> &'static str {
        match self {
            Self::Info => "bg-blue-100 dark:bg-blue-900 text-blue-600 dark:text-blue-300",
            Self::Warning => {
                "bg-yellow-100 dark:bg-yellow-900 text-yellow-600 dark:text-yellow-300"
            }
            Self::Error => "bg-red-100 dark:bg-red-900 text-red-600 dark:text-red-300",
            Self::Success => "bg-green-100 dark:bg-green-900 text-green-600 dark:text-green-300",
        }
    }
}

/// Notification center component
#[component]
pub fn NotificationCenter() -> impl IntoView {
    let (is_open, set_is_open) = signal(false);
    let (notifications, set_notifications) = signal(get_mock_notifications());

    let unread_count = move || notifications.get().iter().filter(|n| !n.read).count();

    let mark_as_read = move |id: String| {
        set_notifications.update(|notifs| {
            if let Some(notif) = notifs.iter_mut().find(|n| n.id == id) {
                notif.read = true;
            }
        });
    };

    let mark_all_as_read = move |_| {
        set_notifications.update(|notifs| {
            for notif in notifs.iter_mut() {
                notif.read = true;
            }
        });
        set_is_open.set(false);
    };

    let toggle_dropdown = move |_| {
        set_is_open.update(|open| *open = !*open);
    };

    // Close dropdown when clicking outside
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::prelude::Effect;
        use wasm_bindgen::JsCast;
        use wasm_bindgen::closure::Closure;

        Effect::new(move |_| {
            if is_open.get() {
                let closure = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
                    set_is_open.set(false);
                }) as Box<dyn FnMut(_)>);

                if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                    let _ = document.add_event_listener_with_callback(
                        "click",
                        closure.as_ref().unchecked_ref(),
                    );
                }

                closure.forget();
            }
        });
    }

    view! {
        <div class="relative">
            // Notification bell button
            <button
                on:click=toggle_dropdown
                class="relative p-2 rounded-lg text-white hover:bg-white/10 transition-colors"
                title="Notifikasi"
            >
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"/>
                </svg>

                // Unread badge
                {move || {
                    let count = unread_count();
                    (count > 0).then(|| view! {
                        <span class="absolute top-0 right-0 inline-flex items-center justify-center px-2 py-1 text-xs font-bold leading-none text-white transform translate-x-1/2 -translate-y-1/2 bg-red-500 rounded-full">
                            {count}
                        </span>
                    })
                }}
            </button>

            // Dropdown panel
            {move || is_open.get().then(|| view! {
                <div
                    class="absolute right-0 mt-2 w-96 bg-white dark:bg-gray-800 rounded-lg shadow-2xl border border-gray-200 dark:border-gray-700 z-50"
                    on:click=move |e| {
                        e.stop_propagation();
                    }
                >
                    // Header
                    <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                            "Notifikasi"
                        </h3>
                        {(unread_count() > 0).then(|| view! {
                            <button
                                on:click=mark_all_as_read
                                class="text-sm text-blue-600 dark:text-blue-400 hover:underline"
                            >
                                "Tandai semua dibaca"
                            </button>
                        })}
                    </div>

                    // Notifications list
                    <div class="max-h-96 overflow-y-auto">
                        {move || {
                            let notifs = notifications.get();
                            if notifs.is_empty() {
                                view! {
                                    <div class="p-8 text-center">
                                        <div class="text-4xl mb-2">"🔔"</div>
                                        <p class="text-gray-500 dark:text-gray-400">
                                            "Tidak ada notifikasi"
                                        </p>
                                    </div>
                                }.into_any()
                            } else {
                                notifs.into_iter().map(|notif| {
                                    let notif_id = notif.id.clone();
                                    let color_classes = notif.category.color_classes();
                                    let icon = notif.category.icon();

                                    view! {
                                        <div
                                            class=format!(
                                                "p-4 border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors cursor-pointer {}",
                                                if !notif.read { "bg-blue-50 dark:bg-blue-900/10" } else { "" }
                                            )
                                            on:click=move |_| {
                                                mark_as_read(notif_id.clone());
                                            }
                                        >
                                            <div class="flex items-start space-x-3">
                                                <div class=format!("flex-shrink-0 p-2 rounded-lg {}", color_classes)>
                                                    <span class="text-lg">{icon}</span>
                                                </div>
                                                <div class="flex-1 min-w-0">
                                                    <div class="flex items-center justify-between">
                                                        <p class="text-sm font-semibold text-gray-900 dark:text-white truncate">
                                                            {notif.title}
                                                        </p>
                                                        {(!notif.read).then(|| view! {
                                                            <span class="ml-2 w-2 h-2 bg-blue-600 rounded-full flex-shrink-0"></span>
                                                        })}
                                                    </div>
                                                    <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
                                                        {notif.message}
                                                    </p>
                                                    <p class="text-xs text-gray-500 dark:text-gray-500 mt-1">
                                                        {notif.timestamp}
                                                    </p>
                                                </div>
                                            </div>
                                        </div>
                                    }
                                }).collect_view().into_any()
                            }
                        }}
                    </div>

                    // Footer
                    <div class="p-3 border-t border-gray-200 dark:border-gray-700">
                        <a
                            href="/notifications"
                            class="block text-center text-sm text-blue-600 dark:text-blue-400 hover:underline"
                        >
                            "Lihat semua notifikasi"
                        </a>
                    </div>
                </div>
            })}
        </div>
    }
}

/// Get mock notifications for demonstration
/// In production, this would fetch from an API or WebSocket
fn get_mock_notifications() -> Vec<Notification> {
    vec![
        Notification {
            id: "1".to_string(),
            title: "Selamat Datang".to_string(),
            message: "Selamat datang di Portal SIMPelv2".to_string(),
            category: NotificationCategory::Success,
            timestamp: "Baru saja".to_string(),
            read: false,
        },
        Notification {
            id: "2".to_string(),
            title: "Pembaruan Sistem".to_string(),
            message: "Sistem akan diperbarui pada 20 Oktober 2025".to_string(),
            category: NotificationCategory::Info,
            timestamp: "2 jam yang lalu".to_string(),
            read: false,
        },
        Notification {
            id: "3".to_string(),
            title: "Peringatan Keamanan".to_string(),
            message: "Harap perbarui password Anda secara berkala".to_string(),
            category: NotificationCategory::Warning,
            timestamp: "1 hari yang lalu".to_string(),
            read: true,
        },
    ]
}
