//! Dashboard page - Main user dashboard after login

use crate::components::cards::{StatCard, StatCardData, StatColor};
use crate::components::layout::MainLayout;
use crate::features::auth::UserSession;
use crate::utils::api::{fetch_api, ApiError};
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
struct SystemMetrics {
    uptime: u64,
    vault: VaultMetrics,
}

#[derive(Clone, Debug, Deserialize)]
struct VaultMetrics {
    active_sessions: u64,
}

#[derive(Clone, Debug, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
}

/// Dashboard page component - main user dashboard with statistics
#[component]
pub fn DashboardPage(
    /// Current user session data
    user_session: UserSession,
    /// Callback function to handle user logout
    on_logout: Box<dyn Fn()>,
) -> impl IntoView {
    // Reactive stats
    let (stats, set_stats) = signal(vec![
        StatCardData {
            title: "Total Sistem".to_string(),
            value: "9".to_string(),
            icon: "🖥️".to_string(),
            color: StatColor::Blue,
            trend: Some("Semua aktif".to_string()),
        },
        StatCardData {
            title: "Pengguna Aktif".to_string(),
            value: "Loading...".to_string(),
            icon: "👥".to_string(),
            color: StatColor::Green,
            trend: Some("Memuat data...".to_string()),
        },
        StatCardData {
            title: "Uptime Sistem".to_string(),
            value: "Loading...".to_string(),
            icon: "⏱️".to_string(),
            color: StatColor::Yellow,
            trend: Some("Memuat data...".to_string()),
        },
        StatCardData {
            title: "Keamanan".to_string(),
            value: "A+".to_string(),
            icon: "🛡️".to_string(),
            color: StatColor::Red,
            trend: Some("Sangat aman".to_string()),
        },
    ]);

    // Fetch real metrics if user is admin
    let is_admin = user_session.role.is_admin();

    Effect::new(move |_| {
        if is_admin {
            spawn_local(async move {
                if let Ok(resp) = fetch_api::<()>("GET", "/v1/admin/metrics", None).await {
                    if let Ok(body) = resp.json::<ApiResponse<SystemMetrics>>().await {
                        if body.success && body.data.is_some() {
                            let metrics = body.data.unwrap();
                            let uptime_hours = metrics.uptime / 3600;

                            set_stats.set(vec![
                                StatCardData {
                                    title: "Total Sistem".to_string(),
                                    value: "9".to_string(),
                                    icon: "🖥️".to_string(),
                                    color: StatColor::Blue,
                                    trend: Some("Semua aktif".to_string()),
                                },
                                StatCardData {
                                    title: "Pengguna Aktif".to_string(),
                                    value: metrics.vault.active_sessions.to_string(),
                                    icon: "👥".to_string(),
                                    color: StatColor::Green,
                                    trend: Some("Live".to_string()),
                                },
                                StatCardData {
                                    title: "Uptime Sistem".to_string(),
                                    value: format!("{} Jam", uptime_hours),
                                    icon: "⏱️".to_string(),
                                    color: StatColor::Yellow,
                                    trend: Some("Sejak restart terakhir".to_string()),
                                },
                                StatCardData {
                                    title: "Keamanan".to_string(),
                                    value: "A+".to_string(),
                                    icon: "🛡️".to_string(),
                                    color: StatColor::Red,
                                    trend: Some("Sistem Terproteksi".to_string()),
                                },
                            ]);
                        }
                    }
                }
            });
        } else {
            // Restore default values for non-admins
            set_stats.set(vec![
                StatCardData {
                    title: "Total Sistem".to_string(),
                    value: "9".to_string(),
                    icon: "🖥️".to_string(),
                    color: StatColor::Blue,
                    trend: Some("Semua aktif".to_string()),
                },
                StatCardData {
                    title: "Pengguna Aktif".to_string(),
                    value: "1,234".to_string(),
                    icon: "👥".to_string(),
                    color: StatColor::Green,
                    trend: Some("+12% bulan ini".to_string()),
                },
                StatCardData {
                    title: "Uptime Sistem".to_string(),
                    value: "99.9%".to_string(),
                    icon: "⏱️".to_string(),
                    color: StatColor::Yellow,
                    trend: Some("30 hari terakhir".to_string()),
                },
                StatCardData {
                    title: "Keamanan".to_string(),
                    value: "A+".to_string(),
                    icon: "🛡️".to_string(),
                    color: StatColor::Red,
                    trend: Some("Sangat aman".to_string()),
                },
            ]);
        }
    });

    // Get current time for greeting
    let greeting = {
        #[cfg(target_arch = "wasm32")]
        {
            let hour = js_sys::Date::new_0().get_hours();
            if hour < 12 {
                "Selamat Pagi"
            } else if hour < 15 {
                "Selamat Siang"
            } else if hour < 18 {
                "Selamat Sore"
            } else {
                "Selamat Malam"
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            "Selamat Datang"
        }
    };

    view! {
        <MainLayout user_session=user_session.clone() on_logout=on_logout>
            <div class="container mx-auto px-4 py-8">
                // Welcome Section - Enhanced with gradient and animation
                <div class="relative bg-gradient-to-r from-red-600 via-red-500 to-orange-500 dark:from-red-800 dark:to-red-900 rounded-2xl shadow-2xl p-8 mb-8 text-white overflow-hidden">
                    // Background pattern
                    <div class="absolute inset-0 opacity-10">
                        <div class="absolute inset-0" style="background-image: radial-gradient(circle at 2px 2px, white 1px, transparent 0); background-size: 40px 40px;"></div>
                    </div>

                    <div class="relative flex flex-col md:flex-row items-center justify-between">
                        <div class="flex-1 mb-4 md:mb-0">
                            <div class="flex items-center gap-3 mb-3">
                                <div class="w-16 h-16 bg-white/20 backdrop-blur-sm rounded-full flex items-center justify-center">
                                    <span class="text-3xl">"👤"</span>
                                </div>
                                <div>
                                    <p class="text-red-100 text-sm font-medium">{greeting}</p>
                                    <h1 class="text-3xl md:text-4xl font-bold">
                                        {user_session.name.clone()}
                                    </h1>
                                </div>
                            </div>
                            <div class="flex flex-wrap gap-3">
                                <span class="inline-flex items-center px-3 py-1 bg-white/20 backdrop-blur-sm rounded-full text-sm font-medium">
                                    <svg class="w-4 h-4 mr-2" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clip-rule="evenodd"/>
                                    </svg>
                                    {user_session.role.display_name()}
                                </span>
                                <span class="inline-flex items-center px-3 py-1 bg-white/20 backdrop-blur-sm rounded-full text-sm font-medium">
                                    <svg class="w-4 h-4 mr-2" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M4 4a2 2 0 012-2h8a2 2 0 012 2v12a1 1 0 110 2h-3a1 1 0 01-1-1v-2a1 1 0 00-1-1H9a1 1 0 00-1 1v2a1 1 0 01-1 1H4a1 1 0 110-2V4zm3 1h2v2H7V5zm2 4H7v2h2V9zm2-4h2v2h-2V5zm2 4h-2v2h2V9z" clip-rule="evenodd"/>
                                    </svg>
                                    {user_session.division.clone()}
                                </span>
                            </div>
                        </div>
                        <div class="hidden md:block">
                            <div class="text-8xl opacity-50">
                                "🏛️"
                            </div>
                        </div>
                    </div>
                </div>

                // Stats Grid - Enhanced with animations
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                    {move || stats.get().into_iter().map(|stat| view! {
                        <div class="transform transition-all duration-300 hover:scale-105">
                            <StatCard data=stat />
                        </div>
                    }).collect_view()}
                </div>

                // Main Content Grid
                <div class="grid grid-cols-1 lg:grid-cols-3 gap-8 mb-8">
                    // Quick Actions - Enhanced
                    <div class="lg:col-span-1 bg-white dark:bg-gray-800 rounded-2xl shadow-lg p-6 border border-gray-100 dark:border-gray-700">
                        <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-6 flex items-center">
                            <div class="w-10 h-10 bg-gradient-to-br from-blue-500 to-blue-600 rounded-lg flex items-center justify-center mr-3">
                                <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 20 20">
                                    <path d="M11 3a1 1 0 10-2 0v1a1 1 0 102 0V3zM15.657 5.757a1 1 0 00-1.414-1.414l-.707.707a1 1 0 001.414 1.414l.707-.707zM18 10a1 1 0 01-1 1h-1a1 1 0 110-2h1a1 1 0 011 1zM5.05 6.464A1 1 0 106.464 5.05l-.707-.707a1 1 0 00-1.414 1.414l.707.707zM5 10a1 1 0 01-1 1H3a1 1 0 110-2h1a1 1 0 011 1zM8 16v-1h4v1a2 2 0 11-4 0zM12 14c.015-.34.208-.646.477-.859a4 4 0 10-4.954 0c.27.213.462.519.476.859h4.002z"/>
                                </svg>
                            </div>
                            "Aksi Cepat"
                        </h3>
                        <div class="space-y-3">
                            <a
                                href="/apps"
                                class="block p-4 bg-gradient-to-r from-blue-50 to-blue-100 dark:from-blue-900/20 dark:to-blue-800/20 rounded-xl hover:shadow-md transition-all group border border-blue-200 dark:border-blue-800"
                            >
                                <div class="flex items-center justify-between">
                                    <div class="flex items-center gap-3">
                                        <div class="w-10 h-10 bg-blue-500 rounded-lg flex items-center justify-center">
                                            <span class="text-xl">"📱"</span>
                                        </div>
                                        <div>
                                            <p class="font-semibold text-gray-900 dark:text-white">"Buka Aplikasi"</p>
                                            <p class="text-xs text-gray-600 dark:text-gray-400">"Akses semua sistem"</p>
                                        </div>
                                    </div>
                                    <svg class="w-5 h-5 text-blue-600 dark:text-blue-400 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                                    </svg>
                                </div>
                            </a>

                            <a
                                href="/notifications"
                                class="block p-4 bg-gradient-to-r from-purple-50 to-purple-100 dark:from-purple-900/20 dark:to-purple-800/20 rounded-xl hover:shadow-md transition-all group border border-purple-200 dark:border-purple-800"
                            >
                                <div class="flex items-center justify-between">
                                    <div class="flex items-center gap-3">
                                        <div class="w-10 h-10 bg-purple-500 rounded-lg flex items-center justify-center">
                                            <span class="text-xl">"🔔"</span>
                                        </div>
                                        <div>
                                            <p class="font-semibold text-gray-900 dark:text-white">"Notifikasi"</p>
                                            <p class="text-xs text-gray-600 dark:text-gray-400">"Lihat pemberitahuan"</p>
                                        </div>
                                    </div>
                                    <svg class="w-5 h-5 text-purple-600 dark:text-purple-400 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                                    </svg>
                                </div>
                            </a>

                            <a
                                href="/monitoring"
                                class="block p-4 bg-gradient-to-r from-green-50 to-green-100 dark:from-green-900/20 dark:to-green-800/20 rounded-xl hover:shadow-md transition-all group border border-green-200 dark:border-green-800"
                            >
                                <div class="flex items-center justify-between">
                                    <div class="flex items-center gap-3">
                                        <div class="w-10 h-10 bg-green-500 rounded-lg flex items-center justify-center">
                                            <span class="text-xl">"📊"</span>
                                        </div>
                                        <div>
                                            <p class="font-semibold text-gray-900 dark:text-white">"Monitoring"</p>
                                            <p class="text-xs text-gray-600 dark:text-gray-400">"Pantau performa sistem"</p>
                                        </div>
                                    </div>
                                    <svg class="w-5 h-5 text-green-600 dark:text-green-400 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                                    </svg>
                                </div>
                            </a>

                            <a
                                href="/settings"
                                class="block p-4 bg-gradient-to-r from-orange-50 to-orange-100 dark:from-orange-900/20 dark:to-orange-800/20 rounded-xl hover:shadow-md transition-all group border border-orange-200 dark:border-orange-800"
                            >
                                <div class="flex items-center justify-between">
                                    <div class="flex items-center gap-3">
                                        <div class="w-10 h-10 bg-orange-500 rounded-lg flex items-center justify-center">
                                            <span class="text-xl">"⚙️"</span>
                                        </div>
                                        <div>
                                            <p class="font-semibold text-gray-900 dark:text-white">"Pengaturan"</p>
                                            <p class="text-xs text-gray-600 dark:text-gray-400">"Kelola profil dan tema"</p>
                                        </div>
                                    </div>
                                    <svg class="w-5 h-5 text-orange-600 dark:text-orange-400 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                                    </svg>
                                </div>
                            </a>
                        </div>
                    </div>

                    // Activity Feed - Enhanced
                    <div class="lg:col-span-2 bg-white dark:bg-gray-800 rounded-2xl shadow-lg p-6 border border-gray-100 dark:border-gray-700">
                        <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-6 flex items-center">
                            <div class="w-10 h-10 bg-gradient-to-br from-green-500 to-green-600 rounded-lg flex items-center justify-center mr-3">
                                <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M6 2a1 1 0 00-1 1v1H4a2 2 0 00-2 2v10a2 2 0 002 2h12a2 2 0 002-2V6a2 2 0 00-2-2h-1V3a1 1 0 10-2 0v1H7V3a1 1 0 00-1-1zm0 5a1 1 0 000 2h8a1 1 0 100-2H6z" clip-rule="evenodd"/>
                                </svg>
                            </div>
                            "Aktivitas Terbaru"
                        </h3>
                        <div class="space-y-3">
                            <div class="flex items-start space-x-3 p-4 bg-gradient-to-r from-blue-50 to-blue-100 dark:from-blue-900/20 dark:to-blue-800/20 rounded-xl border border-blue-200 dark:border-blue-800">
                                <div class="bg-blue-500 p-2 rounded-lg flex-shrink-0">
                                    <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 20 20">
                                        <path d="M8 9a3 3 0 100-6 3 3 0 000 6zM8 11a6 6 0 016 6H2a6 6 0 016-6zM16 7a1 1 0 10-2 0v1h-1a1 1 0 100 2h1v1a1 1 0 102 0v-1h1a1 1 0 100-2h-1V7z"/>
                                    </svg>
                                </div>
                                <div class="flex-1">
                                    <p class="text-sm font-semibold text-gray-900 dark:text-white">"Login Berhasil"</p>
                                    <p class="text-xs text-gray-600 dark:text-gray-400">"Anda berhasil masuk ke sistem"</p>
                                    <p class="text-xs text-gray-500 dark:text-gray-500 mt-1">"Baru saja"</p>
                                </div>
                            </div>

                            <div class="flex items-start space-x-3 p-4 bg-gray-50 dark:bg-gray-700/50 rounded-xl">
                                <div class="bg-gray-400 p-2 rounded-lg flex-shrink-0">
                                    <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"/>
                                    </svg>
                                </div>
                                <div class="flex-1">
                                    <p class="text-sm font-medium text-gray-900 dark:text-white">"Sistem Informasi"</p>
                                    <p class="text-xs text-gray-600 dark:text-gray-400">"Selamat datang di Portal SIMPelv2"</p>
                                    <p class="text-xs text-gray-500 dark:text-gray-500 mt-1">"Hari ini"</p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                // System Status
                <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-lg p-6 border border-gray-100 dark:border-gray-700">
                    <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-6 flex items-center">
                        <div class="w-10 h-10 bg-gradient-to-br from-yellow-500 to-yellow-600 rounded-lg flex items-center justify-center mr-3">
                            <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 20 20">
                                <path fill-rule="evenodd" d="M11.3 1.046A1 1 0 0112 2v5h4a1 1 0 01.82 1.573l-7 10A1 1 0 018 18v-5H4a1 1 0 01-.82-1.573l7-10a1 1 0 011.12-.38z" clip-rule="evenodd"/>
                            </svg>
                        </div>
                        "Status Sistem"
                    </h3>
                    <div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-4">
                        {[
                            ("PIDUM", "🟢"),
                            ("PIDSUS", "🟢"),
                            ("PIDMIL", "🟢"),
                            ("DATUN", "🟢"),
                            ("BADIKLAT", "🟢"),
                            ("Pemulihan Aset", "🟢"),
                            ("Intel", "🟢"),
                            ("Pengawasan", "🟢"),
                            ("Pembinaan", "🟡"),
                        ].iter().map(|(name, status)| view! {
                            <div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
                                <span class="text-sm font-medium text-gray-700 dark:text-gray-300">{*name}</span>
                                <span class="text-lg">{*status}</span>
                            </div>
                        }).collect_view()}
                    </div>
                </div>
            </div>
        </MainLayout>
    }
}
