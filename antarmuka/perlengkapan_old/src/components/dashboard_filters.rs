//! Dashboard Filters Component
//!
//! Provides consistent filtering UI for dashboards with save/load functionality

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use web_sys::Storage;

/// Dashboard filter configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DashboardFilter {
    pub name: String,
    pub tahun_anggaran: i32,
    pub satker_id: Option<String>,
    pub status: Option<String>,
}

impl Default for DashboardFilter {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            tahun_anggaran: 2026,
            satker_id: None,
            status: None,
        }
    }
}

/// Saved filter bookmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterBookmark {
    pub id: String,
    pub name: String,
    pub filter: DashboardFilter,
    pub created_at: String,
}

/// Dashboard Filters Component
#[component]
pub fn DashboardFilters(
    /// Current filter state
    filter: RwSignal<DashboardFilter>,
) -> impl IntoView {
    let (show_save_dialog, set_show_save_dialog) = signal(false);
    let (filter_name, set_filter_name) = signal(String::new());
    let (saved_filters, set_saved_filters) = signal(load_saved_filters());

    // Handle filter change - just log for now
    let handle_filter_change = move || {
        leptos::logging::log!("Filter changed: {:?}", filter.get());
    };

    // Save current filter
    let save_filter = move |_| {
        let name = filter_name.get();
        if name.is_empty() {
            return;
        }

        let bookmark = FilterBookmark {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.clone(),
            filter: filter.get(),
            created_at: js_sys::Date::new_0()
                .to_iso_string()
                .as_string()
                .unwrap_or_default(),
        };

        let mut filters = saved_filters.get();
        filters.push(bookmark);
        save_filters_to_storage(&filters);
        set_saved_filters.set(filters);
        set_show_save_dialog.set(false);
        set_filter_name.set(String::new());
    };

    // Load saved filter
    let load_filter = move |bookmark: FilterBookmark| {
        filter.set(bookmark.filter.clone());
        handle_filter_change();
    };

    // Delete saved filter
    let delete_filter = move |id: String| {
        let mut filters = saved_filters.get();
        filters.retain(|f| f.id != id);
        save_filters_to_storage(&filters);
        set_saved_filters.set(filters);
    };

    view! {
        <div class="bg-white rounded-lg shadow-md p-6 space-y-4">
            <div class="flex justify-between items-center">
                <h3 class="text-lg font-semibold text-gray-900">"Filters"</h3>
                <button
                    class="px-4 py-2 bg-emerald-600 text-white rounded-lg hover:bg-emerald-700 transition-colors"
                    on:click=move |_| set_show_save_dialog.set(true)
                >
                    <i class="fas fa-bookmark mr-2"></i>
                    "Save Filter"
                </button>
            </div>

            // Filter controls
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                // Year filter
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">"Tahun Anggaran"</label>
                    <select
                        class="w-full form-select rounded-lg border-gray-300 shadow-sm focus:border-emerald-500 focus:ring-emerald-500"
                        on:change=move |ev| {
                            let value = event_target_value(&ev).parse::<i32>().unwrap_or(2026);
                            filter.update(|f| f.tahun_anggaran = value);
                            handle_filter_change();
                        }
                        prop:value=move || filter.get().tahun_anggaran
                    >
                        <option value="2024">"2024"</option>
                        <option value="2025">"2025"</option>
                        <option value="2026">"2026"</option>
                        <option value="2027">"2027"</option>
                    </select>
                </div>

                // Status filter
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">"Status"</label>
                    <select
                        class="w-full form-select rounded-lg border-gray-300 shadow-sm focus:border-emerald-500 focus:ring-emerald-500"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            filter.update(|f| f.status = if value.is_empty() { None } else { Some(value) });
                            handle_filter_change();
                        }
                    >
                        <option value="">"All Status"</option>
                        <option value="DRAFT">"Draft"</option>
                        <option value="SUBMITTED">"Submitted"</option>
                        <option value="REVIEWED">"Reviewed"</option>
                        <option value="APPROVED">"Approved"</option>
                        <option value="REJECTED">"Rejected"</option>
                    </select>
                </div>

                // Reset button
                <div class="flex items-end">
                    <button
                        class="w-full px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition-colors"
                        on:click=move |_| {
                            filter.set(DashboardFilter::default());
                            handle_filter_change();
                        }
                    >
                        <i class="fas fa-redo mr-2"></i>
                        "Reset"
                    </button>
                </div>
            </div>

            // Saved filters
            {move || {
                let filters = saved_filters.get();
                if !filters.is_empty() {
                    view! {
                        <div class="border-t pt-4">
                            <h4 class="text-sm font-medium text-gray-700 mb-3">"Saved Filters"</h4>
                            <div class="flex flex-wrap gap-2">
                                {filters.into_iter().map(|bookmark| {
                                    let bookmark_clone = bookmark.clone();
                                    let bookmark_id = bookmark.id.clone();
                                    let bookmark_name = bookmark.name.clone();
                                    view! {
                                        <div class="flex items-center gap-2 px-3 py-2 bg-gray-100 rounded-lg">
                                            <button
                                                class="text-sm text-emerald-600 hover:text-emerald-700 font-medium"
                                                on:click=move |_| load_filter(bookmark_clone.clone())
                                            >
                                                {bookmark_name.clone()}
                                            </button>
                                            <button
                                                class="text-red-500 hover:text-red-700"
                                                on:click=move |_| delete_filter(bookmark_id.clone())
                                            >
                                                <i class="fas fa-times text-xs"></i>
                                            </button>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}

            // Save dialog
            {move || {
                if show_save_dialog.get() {
                    view! {
                        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                            <div class="bg-white rounded-lg p-6 max-w-md w-full mx-4">
                                <h3 class="text-lg font-semibold text-gray-900 mb-4">"Save Filter"</h3>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:border-emerald-500 focus:ring-emerald-500"
                                    placeholder="Filter name..."
                                    on:input=move |ev| set_filter_name.set(event_target_value(&ev))
                                    prop:value=move || filter_name.get()
                                />
                                <div class="flex justify-end gap-3 mt-4">
                                    <button
                                        class="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300"
                                        on:click=move |_| set_show_save_dialog.set(false)
                                    >
                                        "Cancel"
                                    </button>
                                    <button
                                        class="px-4 py-2 bg-emerald-600 text-white rounded-lg hover:bg-emerald-700"
                                        on:click=save_filter
                                    >
                                        "Save"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>
    }
}

// Helper functions for localStorage

fn get_local_storage() -> Option<Storage> {
    web_sys::window()?.local_storage().ok()?
}

fn load_saved_filters() -> Vec<FilterBookmark> {
    let storage = match get_local_storage() {
        Some(s) => s,
        None => return Vec::new(),
    };

    let json = match storage.get_item("dashboard_filters") {
        Ok(Some(j)) => j,
        _ => return Vec::new(),
    };

    serde_json::from_str(&json).unwrap_or_default()
}

fn save_filters_to_storage(filters: &[FilterBookmark]) {
    if let Some(storage) = get_local_storage() {
        if let Ok(json) = serde_json::to_string(filters) {
            let _ = storage.set_item("dashboard_filters", &json);
        }
    }
}
