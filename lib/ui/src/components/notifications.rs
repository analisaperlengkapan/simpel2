//! Notification components for all microfrontends
//!
//! Provides reusable notification UI components

use crate::components::floating::Popover;
use crate::hooks::use_notifications::{
    Notification, NotificationCategory, SyncState, use_notifications,
};
use crate::utils::notification_url::resolve_action_url;
use leptos::prelude::*;
use leptos_node_ref::AnyNodeRef;

/// Notification bell component with dropdown
///
/// Shows a bell icon with unread count badge. Clicking it opens a panel listing the latest
/// notifications; clicking one marks it read and opens the page it is about. The panel closes on
/// Escape, on a click outside it, and when a notification is opened.
///
/// Open/close, positioning and the outside-click handling come from [`Popover`]. This component
/// used to carry its own document-level "close when clicking outside" listener, installed from an
/// `Effect` that ran on the very click that opened the panel: the same click then closed it again,
/// a millisecond later, before anything was painted. The bell never opened.
///
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
    /// Where "Lihat semua notifikasi" leads (default: the portal's history page)
    #[prop(into, default = "/portal/notifications".to_string())]
    all_href: String,
) -> impl IntoView {
    let notif_ctx = use_notifications();
    let is_open = RwSignal::new(false);
    let trigger_ref = AnyNodeRef::new();
    let max_display = max_display.unwrap_or(10);
    let all_href = StoredValue::new(all_href);

    // The inbox is fetched once when the bell mounts; a notification raised since then would be
    // missing from the panel. Opening it asks again, so what is listed is what is there now.
    Effect::new(move |was_open: Option<bool>| {
        let open = is_open.get();
        if open && was_open != Some(true) {
            notif_ctx.refresh();
        }
        open
    });

    let base_class = class.unwrap_or_default();

    view! {
        <div class=base_class>
            <button
                node_ref=trigger_ref
                type="button"
                on:click=move |_| is_open.update(|open| *open = !*open)
                class="relative p-2 rounded-lg text-white hover:bg-white/10 transition-colors"
                title="Notifikasi"
                aria-label=move || {
                    match notif_ctx.unread_count() {
                        0 => "Notifikasi".to_string(),
                        n => format!("Notifikasi, {n} belum dibaca"),
                    }
                }
                aria-haspopup="dialog"
                aria-expanded=move || is_open.get().to_string()
            >
                <svg
                    class="w-6 h-6"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                    aria-hidden="true"
                >
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
                                <span
                                    aria-hidden="true"
                                    class="absolute top-0 right-0 inline-flex items-center justify-center px-2 py-1 text-xs font-bold leading-none text-white transform translate-x-1/2 -translate-y-1/2 bg-red-500 rounded-full"
                                >
                                    {if count > 99 { "99+".to_string() } else { count.to_string() }}
                                </span>
                            }
                        })
                }}
            </button>

            <Popover
                open=is_open.read_only()
                set_open=is_open.write_only()
                trigger_ref=trigger_ref
                class="w-96 max-w-[calc(100vw-1rem)] bg-white dark:bg-gray-800 rounded-lg shadow-2xl border border-gray-200 dark:border-gray-700 overflow-hidden"
                    .to_string()
            >
                <NotificationPanel
                    max_display=max_display
                    all_href=all_href
                    set_open=is_open.write_only()
                />
            </Popover>
        </div>
    }
}

/// The panel behind the bell: header, the latest notifications, and the way to the full list.
#[component]
fn NotificationPanel(
    max_display: usize,
    all_href: StoredValue<String>,
    set_open: WriteSignal<bool>,
) -> impl IntoView {
    let notif_ctx = use_notifications();

    // Opening a notification reads it and closes the panel; the link it carries (if any) does the
    // rest, as an ordinary navigation.
    let open_one = Callback::new(move |id: String| {
        notif_ctx.mark_as_read(&id);
        set_open.set(false);
    });

    view! {
        <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">"Notifikasi"</h3>
            <Show when=move || notif_ctx.unread_count() != 0>
                <button
                    type="button"
                    on:click=move |_| notif_ctx.mark_all_as_read()
                    class="text-sm text-blue-600 dark:text-blue-400 hover:underline"
                >
                    "Tandai semua dibaca"
                </button>
            </Show>
        </div>

        <div class="max-h-96 overflow-y-auto">
            {move || {
                let notifs: Vec<Notification> = notif_ctx
                    .notifications
                    .get()
                    .into_iter()
                    .take(max_display)
                    .collect();
                if !notifs.is_empty() {
                    return notifs
                        .into_iter()
                        .map(|notif| view! { <NotificationRow notif=notif on_open=open_one /> })
                        .collect_view()
                        .into_any();
                }
                match notif_ctx.sync_state.get() {
                    SyncState::Loading => {
                        view! {
                            <div
                                role="status"
                                class="p-8 text-center text-gray-500 dark:text-gray-400"
                            >
                                "Memuat notifikasi…"
                            </div>
                        }
                            .into_any()
                    }
                    SyncState::Unavailable => {
                        view! {
                            <div class="p-8 text-center">
                                <p class="text-gray-500 dark:text-gray-400">
                                    "Notifikasi tidak tersedia"
                                </p>
                                <button
                                    type="button"
                                    on:click=move |_| notif_ctx.refresh()
                                    class="mt-3 text-sm text-blue-600 dark:text-blue-400 hover:underline"
                                >
                                    "Coba lagi"
                                </button>
                            </div>
                        }
                            .into_any()
                    }
                    SyncState::Ready => {
                        view! {
                            <div class="p-8 text-center">
                                <div class="text-4xl mb-2" aria-hidden="true">
                                    "🔔"
                                </div>
                                <p class="text-gray-500 dark:text-gray-400">
                                    "Tidak ada notifikasi"
                                </p>
                            </div>
                        }
                            .into_any()
                    }
                }
            }}
        </div>

        <div class="p-3 border-t border-gray-200 dark:border-gray-700">
            <a
                href=move || all_href.get_value()
                on:click=move |_| set_open.set(false)
                class="block text-center text-sm text-blue-600 dark:text-blue-400 hover:underline"
            >
                "Lihat semua notifikasi"
            </a>
        </div>
    }
}

/// One row of the panel. A link when the notification points somewhere (see
/// [`resolve_action_url`]), otherwise a button that only marks it read — a row that does nothing
/// when clicked is what this replaces.
#[component]
fn NotificationRow(
    notif: Notification,
    /// Called with the notification's id when the row is opened.
    #[prop(into)]
    on_open: Callback<String>,
) -> impl IntoView {
    let href = notif.action_url.as_deref().and_then(resolve_action_url);
    let id = notif.id.clone();
    let unread = !notif.read;
    let row_class = format!(
        "block w-full text-left p-4 border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50 focus-visible:bg-gray-100 dark:focus-visible:bg-gray-700 focus-visible:outline-none transition-colors {}",
        if unread {
            "bg-blue-50 dark:bg-blue-900/10"
        } else {
            ""
        },
    );
    let color_classes = notif.category.color_classes();
    let icon = notif.category.icon();

    let body = view! {
        <div class="flex items-start space-x-3">
            <div class=format!("flex-shrink-0 p-2 rounded-lg {color_classes}")>
                <span class="text-lg" aria-hidden="true">
                    {icon}
                </span>
            </div>
            <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between">
                    <p class="text-sm font-semibold text-gray-900 dark:text-white truncate">
                        {notif.title.clone()}
                    </p>
                    {unread
                        .then(|| {
                            view! {
                                <span class="ml-2 w-2 h-2 bg-blue-600 rounded-full flex-shrink-0">
                                    <span class="sr-only">"Belum dibaca"</span>
                                </span>
                            }
                        })}
                </div>
                <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">{notif.message.clone()}</p>
                <p class="text-xs text-gray-500 dark:text-gray-500 mt-1">
                    {crate::utils::formatters::format_iso_local(&notif.timestamp)}
                </p>
            </div>
        </div>
    };

    match href {
        Some(href) => view! {
            <a href=href class=row_class on:click=move |_| on_open.run(id.clone())>
                {body}
            </a>
        }
        .into_any(),
        None => view! {
            <button type="button" class=row_class on:click=move |_| on_open.run(id.clone())>
                {body}
            </button>
        }
        .into_any(),
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
                        // Honest empty states: "unavailable" (inbox unreachable)
                        // is distinct from "genuinely no notifications".
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
                                let open_id = notif.id.clone();
                                let href = notif.action_url.as_deref().and_then(resolve_action_url);
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
                                                        {crate::utils::formatters::format_iso_local(&notif.timestamp)}
                                                    </p>
                                                    <div class="flex items-center gap-4">
                                                        {href
                                                            .map(|href| {
                                                                view! {
                                                                    <a
                                                                        href=href
                                                                        on:click=move |_| {
                                                                            notif_ctx.mark_as_read(&open_id);
                                                                        }
                                                                        class="text-sm font-semibold text-blue-600 dark:text-blue-400 hover:underline"
                                                                    >
                                                                        "Buka"
                                                                    </a>
                                                                }
                                                            })}
                                                        {(!notif.read)
                                                            .then(|| {
                                                                view! {
                                                                    <button
                                                                        type="button"
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
