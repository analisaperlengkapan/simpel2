//! Notification components for all microfrontends
//!
//! Provides reusable notification UI components

use crate::hooks::use_notifications::{NotificationCategory, use_notifications};
use leptos::prelude::*;

/// Notification bell component with dropdown
///
/// Shows a bell icon with unread count badge and dropdown panel with recent notifications.
/// Can be used in any microfrontend navbar.
///
/// # Example
/// ```rust
/// use lib_ui::components::NotificationBell;
/// use leptos::prelude::*;
///
/// #[component]
/// pub fn Navbar() -> impl IntoView {
///     view! {
///         <nav>
///             // ... other nav items
///             <NotificationBell />
///         </nav>
///     }
/// }
/// ```
#[component]
pub fn NotificationBell(
    /// Optional: Maximum notifications to show in dropdown (default: 10)
    #[prop(optional)]
    max_display: Option<usize>,
    /// Optional: Custom CSS class
    #[prop(optional)]
    class: Option<String>,
) -> impl IntoView {
    let notif_ctx = use_notifications();
    let (is_open, set_is_open) = signal(false);
    let max_display = max_display.unwrap_or(10);

    let toggle_dropdown = move |_| {
        set_is_open.update(|open| *open = !*open);
    };

    let mark_all_as_read = move |_| {
        notif_ctx.mark_all_as_read();
        set_is_open.set(false);
    };

    // Close dropdown when clicking outside
    #[cfg(target_arch = "wasm32")]
    {
        Effect::new(move |_| {
            if is_open.get() {
                use wasm_bindgen::JsCast;
                use wasm_bindgen::closure::Closure;

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

    let base_class = class.unwrap_or_default();

    view! {
        <div class=format!("relative {}", base_class)>
            // Bell button
            <button
                on:click=toggle_dropdown
                class="relative p-2 rounded-lg text-white hover:bg-white/10 transition-colors"
                title="Notifikasi"
                aria-label="Notifikasi"
            >
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
                    />
                </svg>

                // Unread badge
                {move || {
                    let count = notif_ctx.unread_count();
                    (count > 0)
                        .then(|| {
                            view! {
                                <span class="absolute top-0 right-0 inline-flex items-center justify-center px-2 py-1 text-xs font-bold leading-none text-white transform translate-x-1/2 -translate-y-1/2 bg-red-500 rounded-full">
                                    {count}
                                </span>
                            }
                        })
                }}
            </button>

            // Dropdown panel
            {move || {
                is_open
                    .get()
                    .then(|| {
                        let notifications = notif_ctx.notifications.get();
                        let display_notifs: Vec<_> = notifications
                            .into_iter()
                            .take(max_display)
                            .collect();
                        let unread_count = notif_ctx.unread_count();

                        view! {
                            <div
                                class="absolute right-0 mt-2 w-96 bg-white dark:bg-gray-800 rounded-lg shadow-2xl border border-gray-200 dark:border-gray-700 z-popover"
                                on:click=move |e| {
                                    e.stop_propagation();
                                }
                            >
                                // Header
                                <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
                                    <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                                        "Notifikasi"
                                    </h3>
                                    {(unread_count > 0)
                                        .then(|| {
                                            view! {
                                                <button
                                                    on:click=mark_all_as_read
                                                    class="text-sm text-blue-600 dark:text-blue-400 hover:underline"
                                                >
                                                    "Tandai semua dibaca"
                                                </button>
                                            }
                                        })}
                                </div>

                                // Notifications list
                                <div class="max-h-96 overflow-y-auto">
                                    {if display_notifs.is_empty() {
                                        view! {
                                            <div class="p-8 text-center">
                                                <div class="text-4xl mb-2">"🔔"</div>
                                                <p class="text-gray-500 dark:text-gray-400">
                                                    "Tidak ada notifikasi"
                                                </p>
                                            </div>
                                        }
                                            .into_any()
                                    } else {
                                        display_notifs
                                            .into_iter()
                                            .map(|notif| {
                                                let notif_id = notif.id.clone();
                                                let color_classes = notif.category.color_classes();
                                                let icon = notif.category.icon();

                                                view! {
                                                    <div
                                                        class=format!(
                                                            "p-4 border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors cursor-pointer {}",
                                                            if !notif.read {
                                                                "bg-blue-50 dark:bg-blue-900/10"
                                                            } else {
                                                                ""
                                                            },
                                                        )
                                                        on:click=move |_| {
                                                            notif_ctx.mark_as_read(&notif_id);
                                                        }
                                                    >
                                                        <div class="flex items-start space-x-3">
                                                            <div class=format!(
                                                                "flex-shrink-0 p-2 rounded-lg {}",
                                                                color_classes,
                                                            )>
                                                                <span class="text-lg">{icon}</span>
                                                            </div>
                                                            <div class="flex-1 min-w-0">
                                                                <div class="flex items-center justify-between">
                                                                    <p class="text-sm font-semibold text-gray-900 dark:text-white truncate">
                                                                        {notif.title.clone()}
                                                                    </p>
                                                                    {(!notif.read)
                                                                        .then(|| {
                                                                            view! {
                                                                                <span class="ml-2 w-2 h-2 bg-blue-600 rounded-full flex-shrink-0"></span>
                                                                            }
                                                                        })}
                                                                </div>
                                                                <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
                                                                    {notif.message.clone()}
                                                                </p>
                                                                <p class="text-xs text-gray-500 dark:text-gray-500 mt-1">
                                                                    {notif.timestamp.clone()}
                                                                </p>
                                                            </div>
                                                        </div>
                                                    </div>
                                                }
                                            })
                                            .collect_view()
                                            .into_any()
                                    }}
                                </div>

                                // Footer - link to full history (optional, can be customized per app)
                                <div class="p-3 border-t border-gray-200 dark:border-gray-700">
                                    <a
                                        href="/portal/notifications"
                                        class="block text-center text-sm text-blue-600 dark:text-blue-400 hover:underline"
                                    >
                                        "Lihat semua notifikasi"
                                    </a>
                                </div>
                            </div>
                        }
                    })
            }}
        </div>
    }
}

/// Notification list component
///
/// Displays a list of notifications with filtering and actions.
/// Can be used in a dedicated notifications page.
///
/// # Example
/// ```rust
/// use lib_ui::components::NotificationList;
/// use leptos::prelude::*;
///
/// #[component]
/// pub fn NotificationsPage() -> impl IntoView {
///     view! {
///         <div class="container mx-auto p-4">
///             <h1>"Notifikasi"</h1>
///             <NotificationList />
///         </div>
///     }
/// }
/// ```
#[component]
pub fn NotificationList(
    /// Optional: Show category filter buttons
    #[prop(optional)]
    show_filters: bool,
) -> impl IntoView {
    let notif_ctx = use_notifications();
    let (filter, set_filter) = signal(None::<NotificationCategory>);

    let filtered_notifications = move || {
        let notifs = notif_ctx.notifications.get();
        match filter.get() {
            Some(cat) => notifs
                .into_iter()
                .filter(|n| n.category == cat)
                .collect::<Vec<_>>(),
            None => notifs,
        }
    };

    let mark_all_as_read = move |_| {
        notif_ctx.mark_all_as_read();
    };

    view! {
        <div class="space-y-4">
            // Filter and Actions
            {show_filters
                .then(|| {
                    view! {
                        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-4">
                            <div class="flex flex-col md:flex-row items-start md:items-center justify-between space-y-4 md:space-y-0">
                                // Category filters
                                <div class="flex flex-wrap gap-2">
                                    <button
                                        on:click=move |_| set_filter.set(None)
                                        class=move || {
                                            format!(
                                                "px-4 py-2 rounded-lg font-medium transition-colors {}",
                                                if filter.get().is_none() {
                                                    "bg-blue-600 text-white"
                                                } else {
                                                    "bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600"
                                                },
                                            )
                                        }
                                    >
                                        "Semua"
                                    </button>
                                    <button
                                        on:click=move |_| {
                                            set_filter.set(Some(NotificationCategory::Info))
                                        }
                                        class=move || {
                                            format!(
                                                "px-4 py-2 rounded-lg font-medium transition-colors {}",
                                                if filter.get() == Some(NotificationCategory::Info) {
                                                    "bg-blue-600 text-white"
                                                } else {
                                                    "bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600"
                                                },
                                            )
                                        }
                                    >
                                        "ℹ️ Info"
                                    </button>
                                    <button
                                        on:click=move |_| {
                                            set_filter.set(Some(NotificationCategory::Warning))
                                        }
                                        class=move || {
                                            format!(
                                                "px-4 py-2 rounded-lg font-medium transition-colors {}",
                                                if filter.get() == Some(NotificationCategory::Warning) {
                                                    "bg-yellow-600 text-white"
                                                } else {
                                                    "bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600"
                                                },
                                            )
                                        }
                                    >
                                        "⚠️ Peringatan"
                                    </button>
                                    <button
                                        on:click=move |_| {
                                            set_filter.set(Some(NotificationCategory::Error))
                                        }
                                        class=move || {
                                            format!(
                                                "px-4 py-2 rounded-lg font-medium transition-colors {}",
                                                if filter.get() == Some(NotificationCategory::Error) {
                                                    "bg-red-600 text-white"
                                                } else {
                                                    "bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600"
                                                },
                                            )
                                        }
                                    >
                                        "❌ Error"
                                    </button>
                                    <button
                                        on:click=move |_| {
                                            set_filter.set(Some(NotificationCategory::Success))
                                        }
                                        class=move || {
                                            format!(
                                                "px-4 py-2 rounded-lg font-medium transition-colors {}",
                                                if filter.get() == Some(NotificationCategory::Success) {
                                                    "bg-green-600 text-white"
                                                } else {
                                                    "bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600"
                                                },
                                            )
                                        }
                                    >
                                        "✅ Sukses"
                                    </button>
                                </div>

                                // Mark all as read button
                                <button
                                    on:click=mark_all_as_read
                                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium"
                                >
                                    "Tandai Semua Dibaca"
                                </button>
                            </div>
                        </div>
                    }
                })} // Notifications list
            <div class="space-y-4">
                {move || {
                    let notifs = filtered_notifications();
                    if notifs.is_empty() {
                        // Honest empty states: "unavailable" (inbox unreachable)
                        // is distinct from "genuinely no notifications".
                        let unavailable = notif_ctx.sync_state.get()
                            == crate::hooks::use_notifications::SyncState::Unavailable;
                        view! {
                            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-12 text-center">
                                <div class="text-6xl mb-4">"🔔"</div>
                                <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-2">
                                    {if unavailable {
                                        "Notifikasi tidak tersedia"
                                    } else {
                                        "Tidak ada notifikasi"
                                    }}
                                </h3>
                                <p class="text-gray-600 dark:text-gray-400">
                                    {if unavailable {
                                        "Layanan notifikasi sedang tidak dapat dihubungi"
                                    } else {
                                        "Anda tidak memiliki notifikasi saat ini"
                                    }}
                                </p>
                            </div>
                        }
                            .into_any()
                    } else {
                        notifs
                            .into_iter()
                            .map(|notif| {
                                let notif_id = notif.id.clone();
                                let color_classes = notif.category.color_classes();
                                let icon = notif.category.icon();

                                view! {
                                    <div class=format!(
                                        "bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 transition-all hover:shadow-lg {}",
                                        if !notif.read { "border-l-4 border-blue-600" } else { "" },
                                    )>
                                        <div class="flex items-start space-x-4">
                                            <div class=format!(
                                                "flex-shrink-0 p-3 rounded-lg {}",
                                                color_classes,
                                            )>
                                                <span class="text-2xl">{icon}</span>
                                            </div>
                                            <div class="flex-1 min-w-0">
                                                <div class="flex items-center justify-between mb-2">
                                                    <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                                                        {notif.title.clone()}
                                                    </h3>
                                                    {(!notif.read)
                                                        .then(|| {
                                                            view! {
                                                                <span class="ml-2 px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-600 dark:text-blue-300 text-xs font-semibold rounded-full">
                                                                    "Baru"
                                                                </span>
                                                            }
                                                        })}
                                                </div>
                                                <p class="text-gray-600 dark:text-gray-400 mb-3">
                                                    {notif.message.clone()}
                                                </p>
                                                <div class="flex items-center justify-between">
                                                    <p class="text-sm text-gray-500 dark:text-gray-500">
                                                        {notif.timestamp.clone()}
                                                    </p>
                                                    {(!notif.read)
                                                        .then(|| {
                                                            view! {
                                                                <button
                                                                    on:click=move |_| {
                                                                        notif_ctx.mark_as_read(&notif_id);
                                                                    }
                                                                    class="text-sm text-blue-600 dark:text-blue-400 hover:underline"
                                                                >
                                                                    "Tandai dibaca"
                                                                </button>
                                                            }
                                                        })}
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}
