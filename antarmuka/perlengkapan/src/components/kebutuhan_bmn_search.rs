//! # Kebutuhan BMN Advanced Search Component
//!
//! Full-text search with filters, pagination, and sorting for BMN requirements.
//! Requirements: REQ-K005

use leptos::prelude::*;
use leptos::either::Either;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use gloo_net::http::Request;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub page: i32,
    pub per_page: i32,
    pub satker_id: Option<Uuid>,
    pub tahun_anggaran: Option<i32>,
    pub status: Option<String>,
    pub kode_barang: Option<String>,
    pub is_sbsk: Option<bool>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub sort_by: String,
    pub sort_dir: String,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            q: String::new(),
            page: 1,
            per_page: 20,
            satker_id: None,
            tahun_anggaran: None,
            status: None,
            kode_barang: None,
            is_sbsk: None,
            date_from: None,
            date_to: None,
            sort_by: "relevance".to_string(),
            sort_dir: "desc".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub nama: String,
    pub tahun: i32,
    pub status_kode: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub data: Vec<SearchResult>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub message: String,
}

// ============================================================================
// Search Component
// ============================================================================

#[component]
pub fn KebutuhanBmnSearch() -> impl IntoView {
    let (search_query, set_search_query) = signal(String::new());
    let (search_params, set_search_params) = signal(SearchParams::default());
    let (show_filters, set_show_filters) = signal(false);
    let (suggestions, set_suggestions) = signal(Vec::<String>::new());
    let (show_suggestions, set_show_suggestions) = signal(false);

    // Search results resource
    let search_results = LocalResource::new(move || {
        let params = search_params.get();
        async move {
            if params.q.trim().is_empty() {
                return Ok(SearchResponse {
                    data: vec![],
                    total: 0,
                    page: 1,
                    per_page: 20,
                    message: "Enter a search query".to_string(),
                });
            }

            // Build query string
            let mut query_params = vec![
                format!("q={}", urlencoding::encode(&params.q)),
                format!("page={}", params.page),
                format!("per_page={}", params.per_page),
                format!("sort_by={}", params.sort_by),
                format!("sort_dir={}", params.sort_dir),
            ];

            if let Some(satker_id) = params.satker_id {
                query_params.push(format!("satker_id={}", satker_id));
            }
            if let Some(tahun) = params.tahun_anggaran {
                query_params.push(format!("tahun_anggaran={}", tahun));
            }
            if let Some(ref status) = params.status {
                query_params.push(format!("status={}", status));
            }
            if let Some(ref kode) = params.kode_barang {
                query_params.push(format!("kode_barang={}", urlencoding::encode(kode)));
            }
            if let Some(is_sbsk) = params.is_sbsk {
                query_params.push(format!("is_sbsk={}", is_sbsk));
            }

            let url = format!("/api/v1/kebutuhan-bmn/search?{}", query_params.join("&"));

            Request::get(&url)
                .send()
                .await
                .map_err(|e| format!("Request failed: {}", e))?
                .json::<SearchResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        }
    });

    // Suggestions resource
    let fetch_suggestions = move |query: String| async move {
        if query.len() < 2 {
            return Ok(vec![]);
        }

        let url = format!(
            "/api/v1/kebutuhan-bmn/search/suggestions?q={}&limit=10",
            urlencoding::encode(&query)
        );

        #[derive(Deserialize)]
        struct SuggestionsResponse {
            data: Vec<String>,
        }

        Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?
            .json::<SuggestionsResponse>()
            .await
            .map(|r| r.data)
            .map_err(|e| format!("Failed to parse response: {}", e))
    };

    // Handle search input change
    let on_search_input = move |ev| {
        let value = event_target_value(&ev);
        set_search_query.set(value.clone());

        // Fetch suggestions
        if value.len() >= 2 {
            spawn_local(async move {
                match fetch_suggestions(value).await {
                    Ok(sugg) => {
                        set_suggestions.set(sugg);
                        set_show_suggestions.set(true);
                    }
                    Err(_) => {
                        set_suggestions.set(vec![]);
                    }
                }
            });
        } else {
            set_show_suggestions.set(false);
        }
    };

    // Handle search submit
    let on_search_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let query = search_query.get();

        if query.trim().is_empty() {
            return;
        }

        set_show_suggestions.set(false);
        set_search_params.update(|params| {
            params.q = query;
            params.page = 1;
        });
    };

    // Handle suggestion click
    let on_suggestion_click = move |suggestion: String| {
        set_search_query.set(suggestion.clone());
        set_show_suggestions.set(false);
        set_search_params.update(|params| {
            params.q = suggestion;
            params.page = 1;
        });
    };

    // Handle pagination
    let on_page_change = move |new_page: i32| {
        set_search_params.update(|params| {
            params.page = new_page;
        });
    };

    // Handle sort change
    let on_sort_change = move |field: String, direction: String| {
        set_search_params.update(|params| {
            params.sort_by = field;
            params.sort_dir = direction;
            params.page = 1;
        });
    };

    view! {
        <div class="kebutuhan-bmn-search">
            <div class="search-header">
                <h2 class="text-2xl font-bold mb-4">"Pencarian Kebutuhan BMN"</h2>
            </div>

            // Search input with suggestions
            <div class="search-input-container relative mb-6">
                <form on:submit=on_search_submit>
                    <div class="flex gap-2">
                        <div class="relative flex-1">
                            <input
                                type="text"
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                placeholder="Cari kebutuhan BMN..."
                                prop:value=move || search_query.get()
                                on:input=on_search_input
                                on:focus=move |_| {
                                    if !suggestions.get().is_empty() {
                                        set_show_suggestions.set(true);
                                    }
                                }
                            />

                            // Suggestions dropdown
                            {move || {
                                if show_suggestions.get() && !suggestions.get().is_empty() {
                                    Either::Left(view! {
                                        <div class="absolute z-10 w-full mt-1 bg-white border border-gray-300 rounded-lg shadow-lg max-h-60 overflow-y-auto">
                                            <For
                                                each=move || suggestions.get()
                                                key=|s| s.clone()
                                                children=move |suggestion: String| {
                                                    let sugg_clone = suggestion.clone();
                                                    view! {
                                                        <div
                                                            class="px-4 py-2 hover:bg-gray-100 cursor-pointer"
                                                            on:click=move |_| on_suggestion_click(sugg_clone.clone())
                                                        >
                                                            {suggestion}
                                                        </div>
                                                    }
                                                }
                                            />
                                        </div>
                                    })
                                } else {
                                    Either::Right(())
                                }
                            }}
                        </div>

                        <button
                            type="submit"
                            class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 focus:ring-2 focus:ring-blue-500"
                        >
                            "Cari"
                        </button>

                        <button
                            type="button"
                            class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50"
                            on:click=move |_| set_show_filters.update(|v| *v = !*v)
                        >
                            {move || if show_filters.get() { "Sembunyikan Filter" } else { "Tampilkan Filter" }}
                        </button>
                    </div>
                </form>
            </div>

            // Filters panel
            {move || {
                if show_filters.get() {
                    Either::Left(view! {
                        <SearchFilters
                            search_params=search_params
                            set_search_params=set_search_params
                        />
                    })
                } else {
                    Either::Right(())
                }
            }}

            // Search results
            <Suspense fallback=move || view! { <div class="text-center py-8">"Mencari..."</div> }>
                {move || {
                    search_results.get().map(|result| {
                        match result {
                            Ok(response) => Either::Left(view! {
                                <SearchResults
                                    response=response
                                    search_params=search_params
                                    on_page_change=on_page_change
                                    on_sort_change=on_sort_change
                                />
                            }),
                            Err(err) => Either::Right(view! {
                                <div class="bg-red-50 border border-red-200 rounded-lg p-4 text-red-700">
                                    "Error: " {err.to_string()}
                                </div>
                            }),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

// ============================================================================
// Search Filters Component
// ============================================================================

#[component]
fn SearchFilters(
    search_params: ReadSignal<SearchParams>,
    set_search_params: WriteSignal<SearchParams>,
) -> impl IntoView {
    view! {
        <div class="search-filters bg-gray-50 border border-gray-200 rounded-lg p-4 mb-6">
            <h3 class="font-semibold mb-3">"Filter Pencarian"</h3>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                // Tahun Anggaran filter
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">
                        "Tahun Anggaran"
                    </label>
                    <input
                        type="number"
                        class="w-full px-3 py-2 border border-gray-300 rounded-md"
                        placeholder="2024"
                        prop:value=move || search_params.get().tahun_anggaran.map(|t| t.to_string()).unwrap_or_default()
                        on:input=move |ev| {
                            let value = event_target_value(&ev);
                            set_search_params.update(|params| {
                                params.tahun_anggaran = value.parse().ok();
                                params.page = 1;
                            });
                        }
                    />
                </div>

                // Status filter
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">
                        "Status"
                    </label>
                    <select
                        class="w-full px-3 py-2 border border-gray-300 rounded-md"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            set_search_params.update(|params| {
                                params.status = if value.is_empty() { None } else { Some(value) };
                                params.page = 1;
                            });
                        }
                    >
                        <option value="">"Semua Status"</option>
                        <option value="DRAFT">"Draft"</option>
                        <option value="SUBMITTED">"Diajukan"</option>
                        <option value="REVIEWED">"Direview"</option>
                        <option value="APPROVED">"Disetujui"</option>
                        <option value="REJECTED">"Ditolak"</option>
                    </select>
                </div>

                // Kode Barang filter
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">
                        "Kode Barang"
                    </label>
                    <input
                        type="text"
                        class="w-full px-3 py-2 border border-gray-300 rounded-md"
                        placeholder="Cari kode barang..."
                        prop:value=move || search_params.get().kode_barang.unwrap_or_default()
                        on:input=move |ev| {
                            let value = event_target_value(&ev);
                            set_search_params.update(|params| {
                                params.kode_barang = if value.is_empty() { None } else { Some(value) };
                                params.page = 1;
                            });
                        }
                    />
                </div>

                // SBSK filter
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">
                        "Jenis BMN"
                    </label>
                    <select
                        class="w-full px-3 py-2 border border-gray-300 rounded-md"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            set_search_params.update(|params| {
                                params.is_sbsk = match value.as_str() {
                                    "true" => Some(true),
                                    "false" => Some(false),
                                    _ => None,
                                };
                                params.page = 1;
                            });
                        }
                    >
                        <option value="">"Semua"</option>
                        <option value="true">"SBSK"</option>
                        <option value="false">"Non-SBSK"</option>
                    </select>
                </div>
            </div>

            <div class="mt-4 flex justify-end">
                <button
                    class="px-4 py-2 text-sm text-gray-600 hover:text-gray-800"
                    on:click=move |_| {
                        set_search_params.update(|params| {
                            params.tahun_anggaran = None;
                            params.status = None;
                            params.kode_barang = None;
                            params.is_sbsk = None;
                            params.page = 1;
                        });
                    }
                >
                    "Reset Filter"
                </button>
            </div>
        </div>
    }
}

// ============================================================================
// Search Results Component
// ============================================================================

#[component]
fn SearchResults(
    response: SearchResponse,
    search_params: ReadSignal<SearchParams>,
    on_page_change: impl Fn(i32) + 'static + Copy + Send,
    on_sort_change: impl Fn(String, String) + 'static + Copy,
) -> impl IntoView {
    let total_pages = ((response.total as f64) / (response.per_page as f64)).ceil() as i32;

    view! {
        <div class="search-results">
            // Results header
            <div class="flex justify-between items-center mb-4">
                <div class="text-sm text-gray-600">
                    {format!("Menampilkan {} hasil dari {} total", response.data.len(), response.total)}
                </div>

                // Sort selector
                <div class="flex items-center gap-2">
                    <label class="text-sm text-gray-600">"Urutkan:"</label>
                    <select
                        class="px-3 py-1 border border-gray-300 rounded-md text-sm"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            let (field, dir) = match value.as_str() {
                                "relevance" => ("relevance".to_string(), "desc".to_string()),
                                "date_desc" => ("created_at".to_string(), "desc".to_string()),
                                "date_asc" => ("created_at".to_string(), "asc".to_string()),
                                "priority" => ("priority".to_string(), "desc".to_string()),
                                _ => ("relevance".to_string(), "desc".to_string()),
                            };
                            on_sort_change(field, dir);
                        }
                    >
                        <option value="relevance">"Relevansi"</option>
                        <option value="date_desc">"Terbaru"</option>
                        <option value="date_asc">"Terlama"</option>
                        <option value="priority">"Prioritas"</option>
                    </select>
                </div>
            </div>

            // Results list
            {if response.data.is_empty() {
                Either::Left(view! {
                    <div class="text-center py-12 text-gray-500">
                        "Tidak ada hasil yang ditemukan"
                    </div>
                })
            } else {
                Either::Right(view! {
                    <div class="space-y-4">
                        <For
                            each=move || response.data.clone()
                            key=|item| item.id
                            children=move |item: SearchResult| {
                                view! {
                                    <SearchResultItem item=item />
                                }
                            }
                        />
                    </div>
                })
            }}

            // Pagination
            {if total_pages > 1 {
                Either::Left(view! {
                    <SearchPagination
                        current_page=response.page
                        total_pages=total_pages
                        on_page_change=on_page_change
                    />
                })
            } else {
                Either::Right(())
            }}
        </div>
    }
}

// ============================================================================
// Search Result Item Component
// ============================================================================

#[component]
fn SearchResultItem(item: SearchResult) -> impl IntoView {
    view! {
        <div class="bg-white border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow">
            <div class="flex justify-between items-start">
                <div class="flex-1">
                    <h3 class="text-lg font-semibold text-blue-600 hover:text-blue-800">
                        <a href={format!("/kebutuhan-bmn/{}", item.id)}>
                            {item.nama}
                        </a>
                    </h3>
                    <div class="mt-2 flex items-center gap-4 text-sm text-gray-600">
                        <span>"Tahun: " {item.tahun}</span>
                        <span>"Status: " {item.status_kode}</span>
                        <span>"Dibuat: " {item.created_at}</span>
                    </div>
                </div>

                <a
                    href={format!("/kebutuhan-bmn/{}", item.id)}
                    class="px-4 py-2 text-sm text-blue-600 hover:text-blue-800"
                >
                    "Lihat Detail →"
                </a>
            </div>
        </div>
    }
}

// ============================================================================
// Pagination Component
// ============================================================================

#[component]
fn SearchPagination(
    current_page: i32,
    total_pages: i32,
    on_page_change: impl Fn(i32) + 'static + Copy + Send,
) -> impl IntoView {
    let pages = (1..=total_pages).collect::<Vec<_>>();

    view! {
        <div class="flex justify-center items-center gap-2 mt-6">
            // Previous button
            <button
                class="px-3 py-1 border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                disabled=move || current_page <= 1
                on:click=move |_| on_page_change(current_page - 1)
            >
                "← Sebelumnya"
            </button>

            // Page numbers
            <For
                each=move || pages.clone()
                key=|p| *p
                children=move |page: i32| {
                    let is_current = page == current_page;
                    view! {
                        <button
                            class=move || {
                                if is_current {
                                    "px-3 py-1 bg-blue-600 text-white rounded-md"
                                } else {
                                    "px-3 py-1 border border-gray-300 rounded-md hover:bg-gray-50"
                                }
                            }
                            on:click=move |_| on_page_change(page)
                        >
                            {page}
                        </button>
                    }
                }
            />

            // Next button
            <button
                class="px-3 py-1 border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                disabled=move || current_page >= total_pages
                on:click=move |_| on_page_change(current_page + 1)
            >
                "Selanjutnya →"
            </button>
        </div>
    }
}
