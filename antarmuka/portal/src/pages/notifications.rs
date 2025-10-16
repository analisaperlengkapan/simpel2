//! Notifications history page
//!
//! Full page view of all user notifications

use crate::components::layout::MainLayout;
use crate::components::navigation::{Notification, NotificationCategory};
use crate::features::auth::UserSession;
use leptos::prelude::*;

#[component]
pub fn NotificationsPage(user_session: UserSession, on_logout: Box<dyn Fn()>) -> impl IntoView {
    let (notifications, set_notifications) = signal(get_all_notifications());
    let (filter, set_filter) = signal(None::<NotificationCategory>);

    let filtered_notifications = move || match filter.get() {
        Some(cat) => notifications
            .get()
            .into_iter()
            .filter(|n| n.category == cat)
            .collect::<Vec<_>>(),
        None => notifications.get(),
    };

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
    };

    view! {
        <MainLayout user_session=user_session on_logout=on_logout>
            <div class="container mx-auto px-4 py-8">
                // Page Header
                <div class="mb-8">
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                        "Notifikasi"
                    </h1>
                    <p class="text-gray-600 dark:text-gray-400">
                        "Semua notifikasi dan pemberitahuan sistem"
                    </p>
                </div>

                // Filter and Actions
                <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-4 mb-6">
                    <div class="flex flex-col md:flex-row items-start md:items-center justify-between space-y-4 md:space-y-0">
                        // Category filters
                        <div class="flex flex-wrap gap-2">
                            <button
                                on:click=move |_| set_filter.set(None)
                                class=move || format!(
                                    "px-4 py-2 rounded-lg font-medium transition-colors {}",
                                    if filter.get().is_none() {
                                        "bg-blue-600 text-white"
                                    } else {
                                        "bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600"
                                    }
                                )
                            >
                                "Semua"
                            </button>
                            <button
                                on:click=move |_| set_filter.set(Some(NotificationCategory::Info))
                                class=move || format!(
                                    "px-4 py-2 rounded-lg font-medium transition-colors {}",
                                    if filter.get() == Some(NotificationCategory::Info) {
                                        "bg-blue-600 text-white"
                                    } else {
                                        "bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600"
                                    }
                                )
                            >
                                "ℹ️ Info"
                            </button>
                            <button
                                on:click=move |_| set_filter.set(Some(NotificationCategory::Warning))
                                class=move || format!(
                                    "px-4 py-2 rounded-lg font-medium transition-colors {}",
                                    if filter.get() == Some(NotificationCategory::Warning) {
                                        "bg-yellow-600 text-white"
                                    } else {
                                        "bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600"
                                    }
                                )
                            >
                                "⚠️ Peringatan"
                            </button>
                            <button
                                on:click=move |_| set_filter.set(Some(NotificationCategory::Error))
                                class=move || format!(
                                    "px-4 py-2 rounded-lg font-medium transition-colors {}",
                                    if filter.get() == Some(NotificationCategory::Error) {
                                        "bg-red-600 text-white"
                                    } else {
                                        "bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600"
                                    }
                                )
                            >
                                "❌ Error"
                            </button>
                            <button
                                on:click=move |_| set_filter.set(Some(NotificationCategory::Success))
                                class=move || format!(
                                    "px-4 py-2 rounded-lg font-medium transition-colors {}",
                                    if filter.get() == Some(NotificationCategory::Success) {
                                        "bg-green-600 text-white"
                                    } else {
                                        "bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600"
                                    }
                                )
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

                // Notifications list
                <div class="space-y-4">
                    {move || {
                        let notifs = filtered_notifications();
                        if notifs.is_empty() {
                            view! {
                                <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-12 text-center">
                                    <div class="text-6xl mb-4">"🔔"</div>
                                    <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-2">
                                        "Tidak ada notifikasi"
                                    </h3>
                                    <p class="text-gray-600 dark:text-gray-400">
                                        "Anda tidak memiliki notifikasi saat ini"
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
                                            "bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 transition-all hover:shadow-lg {}",
                                            if !notif.read { "border-l-4 border-blue-600" } else { "" }
                                        )
                                    >
                                        <div class="flex items-start space-x-4">
                                            <div class=format!("flex-shrink-0 p-3 rounded-lg {}", color_classes)>
                                                <span class="text-2xl">{icon}</span>
                                            </div>
                                            <div class="flex-1 min-w-0">
                                                <div class="flex items-center justify-between mb-2">
                                                    <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                                                        {notif.title}
                                                    </h3>
                                                    {(!notif.read).then(|| view! {
                                                        <span class="ml-2 px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-600 dark:text-blue-300 text-xs font-semibold rounded-full">
                                                            "Baru"
                                                        </span>
                                                    })}
                                                </div>
                                                <p class="text-gray-600 dark:text-gray-400 mb-3">
                                                    {notif.message}
                                                </p>
                                                <div class="flex items-center justify-between">
                                                    <p class="text-sm text-gray-500 dark:text-gray-500">
                                                        {notif.timestamp}
                                                    </p>
                                                    {(!notif.read).then(|| view! {
                                                        <button
                                                            on:click=move |_| {
                                                                mark_as_read(notif_id.clone());
                                                            }
                                                            class="text-sm text-blue-600 dark:text-blue-400 hover:underline"
                                                        >
                                                            "Tandai dibaca"
                                                        </button>
                                                    })}
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect_view().into_any()
                        }
                    }}
                </div>
            </div>
        </MainLayout>
    }
}

/// Get all notifications (mock data)
/// In production, this would fetch from an API
fn get_all_notifications() -> Vec<Notification> {
    vec![
        Notification {
            id: "1".to_string(),
            title: "Selamat Datang di Portal SIMPelv2".to_string(),
            message: "Terima kasih telah menggunakan Portal SIMPelv2. Sistem ini menyediakan akses terpadu ke semua aplikasi kejaksaan.".to_string(),
            category: NotificationCategory::Success,
            timestamp: "Baru saja".to_string(),
            read: false,
        },
        Notification {
            id: "2".to_string(),
            title: "Pembaruan Sistem Terjadwal".to_string(),
            message: "Sistem akan diperbarui pada 20 Oktober 2025 pukul 02:00 WIB. Layanan mungkin tidak tersedia selama 30 menit.".to_string(),
            category: NotificationCategory::Info,
            timestamp: "2 jam yang lalu".to_string(),
            read: false,
        },
        Notification {
            id: "3".to_string(),
            title: "Peringatan Keamanan".to_string(),
            message: "Harap perbarui password Anda secara berkala untuk menjaga keamanan akun. Password terakhir diubah 90 hari yang lalu.".to_string(),
            category: NotificationCategory::Warning,
            timestamp: "1 hari yang lalu".to_string(),
            read: true,
        },
        Notification {
            id: "4".to_string(),
            title: "Aplikasi PIDUM Berhasil Diakses".to_string(),
            message: "Anda berhasil mengakses aplikasi PIDUM pada 15 Oktober 2025 pukul 14:30 WIB.".to_string(),
            category: NotificationCategory::Success,
            timestamp: "2 hari yang lalu".to_string(),
            read: true,
        },
        Notification {
            id: "5".to_string(),
            title: "Fitur Baru: Notifikasi Real-time".to_string(),
            message: "Kami telah menambahkan fitur notifikasi real-time. Anda akan menerima pemberitahuan langsung untuk event penting.".to_string(),
            category: NotificationCategory::Info,
            timestamp: "3 hari yang lalu".to_string(),
            read: true,
        },
        Notification {
            id: "6".to_string(),
            title: "Backup Data Berhasil".to_string(),
            message: "Backup data sistem berhasil dilakukan pada 14 Oktober 2025. Semua data Anda aman.".to_string(),
            category: NotificationCategory::Success,
            timestamp: "4 hari yang lalu".to_string(),
            read: true,
        },
    ]
}
