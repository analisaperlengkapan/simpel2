//! Applications page - Microfrontend launcher

use crate::components::cards::AppCard;
use crate::components::layout::MainLayout;
use crate::features::auth::UserSession;
use crate::features::microfrontends::{AppCategory, MicrofrontendRegistry};
use leptos::prelude::*;

/// Applications page component - displays available microfrontends
#[component]
pub fn AppsPage(
    /// Current user session data
    user_session: UserSession,
    /// Callback function to handle user logout
    on_logout: Box<dyn Fn()>,
) -> impl IntoView {
    let (selected_category, set_selected_category) = signal(None::<AppCategory>);
    let (search_query, set_search_query) = signal(String::new());

    let apps = move || {
        let mut apps = match selected_category.get() {
            Some(cat) => MicrofrontendRegistry::get_apps_by_category(cat),
            None => MicrofrontendRegistry::get_all_apps(),
        };

        // Filter by search query
        let query = search_query.get().to_lowercase();
        if !query.is_empty() {
            apps.retain(|app| {
                app.name.to_lowercase().contains(&query)
                    || app.description.to_lowercase().contains(&query)
            });
        }

        apps
    };

    let categories = MicrofrontendRegistry::get_all_categories();
    let total_apps = MicrofrontendRegistry::get_all_apps().len();

    view! {
        <MainLayout user_session=user_session on_logout=on_logout>
            <div class="container mx-auto px-4 py-8">
                // Page Header - Enhanced with gradient
                <div class="mb-8">
                    <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
                        <div>
                            <h1 class="text-4xl font-bold bg-gradient-to-r from-red-600 to-orange-500 bg-clip-text text-transparent mb-2">
                                "Aplikasi SIMPEL"
                            </h1>
                            <p class="text-gray-600 dark:text-gray-400 flex items-center gap-2">
                                <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                                    <path d="M3 4a1 1 0 011-1h12a1 1 0 011 1v2a1 1 0 01-1 1H4a1 1 0 01-1-1V4zM3 10a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H4a1 1 0 01-1-1v-6zM14 9a1 1 0 00-1 1v6a1 1 0 001 1h2a1 1 0 001-1v-6a1 1 0 00-1-1h-2z"/>
                                </svg>
                                {total_apps}" aplikasi tersedia"
                            </p>
                        </div>

                        // Search Box
                        <div class="relative w-full md:w-96">
                            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                                </svg>
                            </div>
                            <input
                                type="text"
                                placeholder="Cari aplikasi..."
                                class="w-full pl-10 pr-4 py-3 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-xl focus:ring-2 focus:ring-red-500 focus:border-transparent transition-all"
                                on:input=move |ev| {
                                    set_search_query.set(event_target_value(&ev));
                                }
                                prop:value=move || search_query.get()
                            />
                        </div>
                    </div>
                </div>

                // Category Filter - Enhanced with icons
                <div class="mb-8">
                    <div class="flex items-center gap-3 mb-4">
                        <svg class="w-5 h-5 text-gray-600 dark:text-gray-400" fill="currentColor" viewBox="0 0 20 20">
                            <path d="M3 4a1 1 0 011-1h12a1 1 0 011 1v2a1 1 0 01-1 1H4a1 1 0 01-1-1V4zM3 10a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H4a1 1 0 01-1-1v-6zM14 9a1 1 0 00-1 1v6a1 1 0 001 1h2a1 1 0 001-1v-6a1 1 0 00-1-1h-2z"/>
                        </svg>
                        <span class="text-sm font-semibold text-gray-700 dark:text-gray-300">"Filter Kategori:"</span>
                    </div>
                    <div class="flex flex-wrap gap-3">
                        <button
                            on:click=move |_| set_selected_category.set(None)
                            class=move || format!(
                                "px-5 py-2.5 rounded-xl font-medium transition-all duration-200 transform hover:scale-105 {}",
                                if selected_category.get().is_none() {
                                    "bg-gradient-to-r from-red-600 to-red-700 text-white shadow-lg shadow-red-500/50"
                                } else {
                                    "bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 hover:border-red-500 dark:hover:border-red-500"
                                }
                            )
                        >
                            <span class="flex items-center gap-2">
                                "🏠"
                                "Semua Aplikasi"
                            </span>
                        </button>
                        {categories.into_iter().map(|cat| {
                            let cat_clone = cat.clone();
                            let icon = match cat {
                                AppCategory::Prosecution => "⚖️",
                                AppCategory::Training => "🎓",
                                AppCategory::Legal => "📜",
                                AppCategory::Asset => "💰",
                                AppCategory::Intelligence => "🕵️",
                                AppCategory::Supervision => "👁️",
                            };
                            view! {
    <button
                                    on:click=move |_| set_selected_category.set(Some(cat_clone.clone()))
                                    class=move || format!(
                                        "px-5 py-2.5 rounded-xl font-medium transition-all duration-200 transform hover:scale-105 {}",
                                        if selected_category.get() == Some(cat.clone()) {
                                            "bg-gradient-to-r from-red-600 to-red-700 text-white shadow-lg shadow-red-500/50"
                                        } else {
                                            "bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 hover:border-red-500 dark:hover:border-red-500"
                                        }
                                    )
                                >
                                    <span class="flex items-center gap-2">
                                        {icon}
                                        {cat.display_name()}
                                    </span>
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </div>

                // Apps Grid - Enhanced with stagger animation
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-8">
                    {move || apps().into_iter().enumerate().map(|(idx, app)| {
                        let delay = format!("{}ms", idx * 50);
                        view! {
                            <div
                                class="animate-fade-in"
                                style=format!("animation-delay: {}", delay)
                            >
                                <AppCard app=app />
                            </div>
                        }
                    }).collect_view()}
                </div>

                // Empty State - Enhanced
                {move || (apps().is_empty()).then(|| view! {
                    <div class="text-center py-20">
                        <div class="inline-flex items-center justify-center w-24 h-24 bg-gray-100 dark:bg-gray-800 rounded-full mb-6">
                            <span class="text-6xl">"📭"</span>
                        </div>
                        <h3 class="text-2xl font-bold text-gray-900 dark:text-white mb-3">
                            "Tidak ada aplikasi ditemukan"
                        </h3>
                        <p class="text-gray-600 dark:text-gray-400 mb-6 max-w-md mx-auto">
                            {if !search_query.get().is_empty() {
                                "Coba ubah kata kunci pencarian Anda"
                            } else {
                                "Tidak ada aplikasi dalam kategori ini"
                            }}
                        </p>
                        <button
                            on:click=move |_| {
                                set_selected_category.set(None);
                                set_search_query.set(String::new());
                            }
                            class="px-6 py-3 bg-gradient-to-r from-red-600 to-red-700 text-white rounded-xl font-medium hover:shadow-lg transition-all duration-200"
                        >
                            "Tampilkan Semua Aplikasi"
                        </button>
                    </div>
                })}

                // Info Banner
                <div class="mt-12 bg-gradient-to-r from-blue-50 to-blue-100 dark:from-blue-900/20 dark:to-blue-800/20 rounded-2xl p-8 border border-blue-200 dark:border-blue-800">
                    <div class="flex flex-col md:flex-row items-center gap-6">
                        <div class="flex-shrink-0">
                            <div class="w-16 h-16 bg-blue-500 rounded-full flex items-center justify-center">
                                <svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                                </svg>
                            </div>
                        </div>
                        <div class="flex-1 text-center md:text-left">
                            <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-2">
                                "Butuh Bantuan?"
                            </h3>
                            <p class="text-gray-600 dark:text-gray-400">
                                "Jika Anda mengalami kesulitan mengakses aplikasi, silakan hubungi tim IT support atau baca dokumentasi pengguna."
                            </p>
                        </div>
                        <div class="flex-shrink-0">
                            <a
                                href="/portal/help"
                                class="inline-flex items-center gap-2 px-6 py-3 bg-blue-600 text-white rounded-xl font-medium hover:bg-blue-700 transition-colors"
                            >
                                "Pusat Bantuan"
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                                </svg>
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        </MainLayout>
    }
}
