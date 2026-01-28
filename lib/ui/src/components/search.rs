//! Global search components for all microfrontends
//!
//! Provides reusable search UI components

use crate::hooks::use_search::{SearchResult, use_debounced_search, use_search};
use leptos::prelude::*;

/// Global search bar component with dropdown results
///
/// # Example
/// ```rust
/// use lib_ui::components::GlobalSearchBar;
/// use leptos::prelude::*;
///
/// #[component]
/// pub fn Navbar() -> impl IntoView {
///     view! {
///         <nav>
///             <GlobalSearchBar />
///         </nav>
///     }
/// }
/// ```
#[component]
pub fn GlobalSearchBar(
    /// Optional: Placeholder text
    #[prop(optional)]
    placeholder: Option<String>,
    /// Optional: Custom CSS class
    #[prop(optional)]
    class: Option<String>,
    /// Optional: Debounce delay in milliseconds (default: 300)
    #[prop(optional)]
    debounce_ms: Option<u32>,
) -> impl IntoView {
    let search_ctx = use_search();
    let debounced_search = use_debounced_search(debounce_ms.unwrap_or(300));
    let (is_open, set_is_open) = signal(false);
    let (input_value, set_input_value) = signal(String::new());

    let placeholder =
        placeholder.unwrap_or_else(|| "Cari aplikasi, halaman, atau dokumen...".to_string());
    let base_class = class.unwrap_or_default();

    // Handle input change
    let handle_input = move |ev| {
        let value = event_target_value(&ev);
        set_input_value.set(value.clone());
        debounced_search(value);
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

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = url; // Suppress unused warning
        }

        set_input_value.set(String::new());
        search_ctx.clear();
        set_is_open.set(false);
    };

    // Show dropdown when there are results
    Effect::new(move |_| {
        let has_results = search_ctx.results_count() > 0;
        let has_query = !input_value.get().is_empty();
        set_is_open.set(has_results && has_query);
    });

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

    view! {
        <div class=format!("relative {}", base_class)>
            // Search input
            <div class="relative">
                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                    <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                    </svg>
                </div>
                <input
                    type="text"
                    placeholder=placeholder
                    class="w-full pl-10 pr-4 py-2 bg-white/10 border border-white/20 rounded-lg text-white placeholder-white/60 focus:outline-none focus:ring-2 focus:ring-white/30 focus:bg-white/20 transition-all"
                    prop:value=move || input_value.get()
                    on:input=handle_input
                    on:focus=move |_| {
                        if search_ctx.results_count() > 0 {
                            set_is_open.set(true);
                        }
                    }
                    on:click=move |e| {
                        e.stop_propagation();
                    }
                />

                // Loading indicator
                {move || search_ctx.is_loading.get().then(|| view! {
                    <div class="absolute inset-y-0 right-0 pr-3 flex items-center">
                        <svg class="animate-spin h-5 w-5 text-white/60" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                    </div>
                })}
            </div>

            // Search results dropdown
            {move || is_open.get().then(|| {
                let results = search_ctx.results.get();

                view! {
                    <div
                        class="absolute top-full left-0 right-0 mt-2 bg-white dark:bg-gray-800 rounded-lg shadow-2xl border border-gray-200 dark:border-gray-700 max-h-96 overflow-y-auto z-50"
                        on:click=move |e| {
                            e.stop_propagation();
                        }
                    >
                        // Results header
                        <div class="p-3 border-b border-gray-200 dark:border-gray-700">
                            <p class="text-sm text-gray-600 dark:text-gray-400">
                                {format!("{} hasil ditemukan", results.len())}
                            </p>
                        </div>

                        // Results list
                        {if results.is_empty() {
                            view! {
                                <div class="p-8 text-center">
                                    <div class="text-4xl mb-2">"🔍"</div>
                                    <p class="text-gray-500 dark:text-gray-400">
                                        "Tidak ada hasil ditemukan"
                                    </p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="py-2">
                                    {results.into_iter().map(|result| {
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
                                                        {result.icon.clone()}
                                                    </div>
                                                    <div class="flex-1 min-w-0">
                                                        <div class="flex items-center space-x-2 mb-1">
                                                            <h4 class="text-sm font-semibold text-gray-900 dark:text-white truncate">
                                                                {result.title.clone()}
                                                            </h4>
                                                            <span class=format!("px-2 py-0.5 text-xs font-medium rounded-full {}", color_classes)>
                                                                {result.category.display_name()}
                                                            </span>
                                                        </div>
                                                        <p class="text-sm text-gray-600 dark:text-gray-400 line-clamp-2">
                                                            {result.description.clone()}
                                                        </p>
                                                        {result.module.as_ref().map(|module| view! {
                                                            <p class="text-xs text-gray-500 dark:text-gray-500 mt-1">
                                                                {module.clone()}
                                                            </p>
                                                        })}
                                                    </div>
                                                    <div class="flex-shrink-0">
                                                        <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                                                        </svg>
                                                    </div>
                                                </div>
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }}
                    </div>
                }
            })}
        </div>
    }
}

/// Compact search button that opens a modal
///
/// Useful for mobile or space-constrained layouts
#[component]
pub fn GlobalSearchButton(
    /// Optional: Custom CSS class
    #[prop(optional)]
    class: Option<String>,
) -> impl IntoView {
    let (is_modal_open, set_is_modal_open) = signal(false);
    let base_class = class.unwrap_or_default();

    view! {
        <>
            // Search button
            <button
                on:click=move |_| set_is_modal_open.set(true)
                class=format!("p-2 rounded-lg hover:bg-white/10 transition-colors {}", base_class)
                title="Cari"
                aria-label="Buka pencarian"
            >
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                </svg>
            </button>

            // Search modal
            {move || is_modal_open.get().then(|| view! {
                <div class="fixed inset-0 z-50 flex items-start justify-center pt-20 px-4 bg-black/50 backdrop-blur-sm">
                    <div class="w-full max-w-2xl bg-white dark:bg-gray-800 rounded-lg shadow-2xl">
                        <div class="p-4">
                            <div class="flex items-center justify-between mb-4">
                                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                                    "Pencarian Global"
                                </h3>
                                <button
                                    on:click=move |_| set_is_modal_open.set(false)
                                    class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                                >
                                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                                    </svg>
                                </button>
                            </div>
                            <GlobalSearchBar class="w-full".to_string() />
                        </div>
                    </div>
                </div>
            })}
        </>
    }
}

/// Search results list component
///
/// Displays search results in a list format
#[component]
pub fn SearchResultsList<F>(
    /// Search results to display
    results: Vec<SearchResult>,
    /// Callback when result is clicked
    on_result_click: F,
) -> impl IntoView
where
    F: Fn(SearchResult) + 'static + Clone,
{
    let on_click = on_result_click.clone();

    view! {
        <div class="space-y-2">
            {results.into_iter().map(move |result| {
                let result_clone = result.clone();
                let color_classes = result.category.color_classes();
                let on_click = on_click.clone();

                view! {
                    <button
                        on:click=move |_| {
                            on_click(result_clone.clone());
                        }
                        class="w-full p-4 bg-white dark:bg-gray-800 rounded-lg shadow hover:shadow-md transition-all text-left"
                    >
                        <div class="flex items-start space-x-3">
                            <div class="flex-shrink-0 text-2xl">
                                {result.icon.clone()}
                            </div>
                            <div class="flex-1 min-w-0">
                                <div class="flex items-center space-x-2 mb-1">
                                    <h4 class="text-base font-semibold text-gray-900 dark:text-white">
                                        {result.title.clone()}
                                    </h4>
                                    <span class=format!("px-2 py-0.5 text-xs font-medium rounded-full {}", color_classes)>
                                        {result.category.display_name()}
                                    </span>
                                </div>
                                <p class="text-sm text-gray-600 dark:text-gray-400">
                                    {result.description.clone()}
                                </p>
                                {result.module.as_ref().map(|module| view! {
                                    <p class="text-xs text-gray-500 dark:text-gray-500 mt-1">
                                        "📍 " {module.clone()}
                                    </p>
                                })}
                            </div>
                        </div>
                    </button>
                }
            }).collect_view()}
        </div>
    }
}
