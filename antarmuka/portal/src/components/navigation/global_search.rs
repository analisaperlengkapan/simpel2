//! Global search component
//!
//! Search bar with autocomplete and cross-module search

use crate::features::microfrontends::MicrofrontendRegistry;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Search result item
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: SearchCategory,
    pub url: String,
    pub icon: String,
}

/// Search result category
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SearchCategory {
    Application,
    Page,
    Document,
    User,
}

impl SearchCategory {
    fn display_name(&self) -> &'static str {
        match self {
            Self::Application => "Aplikasi",
            Self::Page => "Halaman",
            Self::Document => "Dokumen",
            Self::User => "Pengguna",
        }
    }

    fn color_classes(&self) -> &'static str {
        match self {
            Self::Application => "bg-blue-100 dark:bg-blue-900 text-blue-600 dark:text-blue-300",
            Self::Page => "bg-green-100 dark:bg-green-900 text-green-600 dark:text-green-300",
            Self::Document => {
                "bg-yellow-100 dark:bg-yellow-900 text-yellow-600 dark:text-yellow-300"
            }
            Self::User => "bg-purple-100 dark:bg-purple-900 text-purple-600 dark:text-purple-300",
        }
    }
}

/// Global search component
#[component]
pub fn GlobalSearch() -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (is_open, set_is_open) = signal(false);
    let (results, set_results) = signal(Vec::<SearchResult>::new());

    // Perform search when query changes
    let perform_search = move || {
        let q = query.get().to_lowercase();
        if q.is_empty() {
            set_results.set(Vec::new());
            set_is_open.set(false);
            return;
        }

        // Search across applications
        let app_results: Vec<SearchResult> = MicrofrontendRegistry::get_all_apps()
            .into_iter()
            .filter(|app| {
                app.name.to_lowercase().contains(&q) || app.description.to_lowercase().contains(&q)
            })
            .map(|app| SearchResult {
                id: app.id.clone(),
                title: app.name,
                description: app.description,
                category: SearchCategory::Application,
                url: app.url,
                icon: app.icon,
            })
            .collect();

        // Search across pages (mock data)
        let page_results: Vec<SearchResult> = get_searchable_pages()
            .into_iter()
            .filter(|page| {
                page.title.to_lowercase().contains(&q)
                    || page.description.to_lowercase().contains(&q)
            })
            .collect();

        // Combine results
        let mut all_results = app_results;
        all_results.extend(page_results);

        let is_empty = all_results.is_empty();
        set_results.set(all_results);
        set_is_open.set(!is_empty);
    };

    // Handle input change with debouncing
    let handle_input = move |ev| {
        let value = event_target_value(&ev);
        set_query.set(value);
        perform_search();
    };

    // Handle result click
    let handle_result_click = move |url: String| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if url.starts_with("http") {
                    // External URL - open in new tab
                    let _ = window.open_with_url_and_target(&url, "_blank");
                } else {
                    // Internal route - navigate
                    let _ = window.location().set_href(&url);
                }
            }
        }
        set_query.set(String::new());
        set_is_open.set(false);
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
        <div class="relative flex-1 max-w-2xl mx-4">
            // Search input
            <div class="relative">
                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                    <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                    </svg>
                </div>
                <input
                    type="text"
                    placeholder="Cari aplikasi, halaman, atau dokumen..."
                    class="w-full pl-10 pr-4 py-2 bg-white/10 border border-white/20 rounded-lg text-white placeholder-white/60 focus:outline-none focus:ring-2 focus:ring-white/30 focus:bg-white/20 transition-all"
                    prop:value=move || query.get()
                    on:input=handle_input
                    on:focus=move |_| {
                        if !results.get().is_empty() {
                            set_is_open.set(true);
                        }
                    }
                    on:click=move |e| {
                        e.stop_propagation();
                    }
                />
            </div>

            // Search results dropdown
            {move || is_open.get().then(|| view! {
                <div
                    class="absolute top-full left-0 right-0 mt-2 bg-white dark:bg-gray-800 rounded-lg shadow-2xl border border-gray-200 dark:border-gray-700 max-h-96 overflow-y-auto z-50"
                    on:click=move |e| {
                        e.stop_propagation();
                    }
                >
                    // Results header
                    <div class="p-3 border-b border-gray-200 dark:border-gray-700">
                        <p class="text-sm text-gray-600 dark:text-gray-400">
                            {move || format!("{} hasil ditemukan", results.get().len())}
                        </p>
                    </div>

                    // Results list
                    <div class="py-2">
                        {move || {
                            results.get().into_iter().map(|result| {
                                let url = result.url.clone();
                                let color_classes = result.category.color_classes();

                                view! {
                                    <button
                                        on:click=move |_| {
                                            handle_result_click(url.clone());
                                        }
                                        class="w-full px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors text-left"
                                    >
                                        <div class="flex items-start space-x-3">
                                            <div class="flex-shrink-0 text-2xl">
                                                {result.icon}
                                            </div>
                                            <div class="flex-1 min-w-0">
                                                <div class="flex items-center space-x-2 mb-1">
                                                    <h4 class="text-sm font-semibold text-gray-900 dark:text-white truncate">
                                                        {result.title}
                                                    </h4>
                                                    <span class=format!("px-2 py-0.5 text-xs font-medium rounded-full {}", color_classes)>
                                                        {result.category.display_name()}
                                                    </span>
                                                </div>
                                                <p class="text-sm text-gray-600 dark:text-gray-400 line-clamp-2">
                                                    {result.description}
                                                </p>
                                            </div>
                                            <div class="flex-shrink-0">
                                                <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                                                </svg>
                                            </div>
                                        </div>
                                    </button>
                                }
                            }).collect_view()
                        }}
                    </div>

                    // No results
                    {move || (results.get().is_empty() && !query.get().is_empty()).then(|| view! {
                        <div class="p-8 text-center">
                            <div class="text-4xl mb-2">"🔍"</div>
                            <p class="text-gray-500 dark:text-gray-400">
                                "Tidak ada hasil ditemukan"
                            </p>
                        </div>
                    })}
                </div>
            })}
        </div>
    }
}

/// Get searchable pages (mock data)
/// In production, this would be dynamically generated or fetched from API
fn get_searchable_pages() -> Vec<SearchResult> {
    vec![
        SearchResult {
            id: "dashboard".to_string(),
            title: "Dashboard".to_string(),
            description: "Dashboard utama dengan statistik dan aktivitas terbaru".to_string(),
            category: SearchCategory::Page,
            url: "/dashboard".to_string(),
            icon: "📊".to_string(),
        },
        SearchResult {
            id: "apps".to_string(),
            title: "Aplikasi".to_string(),
            description: "Daftar semua aplikasi SIMPelv2 yang tersedia".to_string(),
            category: SearchCategory::Page,
            url: "/apps".to_string(),
            icon: "🚀".to_string(),
        },
        SearchResult {
            id: "notifications".to_string(),
            title: "Notifikasi".to_string(),
            description: "Semua notifikasi dan pemberitahuan sistem".to_string(),
            category: SearchCategory::Page,
            url: "/notifications".to_string(),
            icon: "🔔".to_string(),
        },
        SearchResult {
            id: "pembinaan".to_string(),
            title: "Pembinaan".to_string(),
            description: "Sistem pembinaan dan pengembangan SDM".to_string(),
            category: SearchCategory::Page,
            url: "/pembinaan".to_string(),
            icon: "🌱".to_string(),
        },
    ]
}
