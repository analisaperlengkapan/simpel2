//! # Global Search Page
//!
//! This page provides global search functionality across all modules:
//! - Search kebutuhan BMN
//! - Search pemakaian BMN
//! - Search penghapusan BMN
//! - Unified search results with pagination
//! - Filters by status, tahun, satker
//!
//! Requirements: REQ-K015 (search), Task 11.2

use leptos::prelude::*;
use leptos::task::spawn_local;
use lib_ui::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// API Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub module: String, // "kebutuhan", "pemakaian", "penghapusan"
    pub title: String,
    pub description: String,
    pub status: String,
    pub tahun: Option<i32>,
    pub satker_nama: Option<String>,
    pub created_at: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

// ============================================================================
// Main Component
// ============================================================================

#[component]
pub fn SearchPage() -> impl IntoView {
    // State
    let (search_query, set_search_query) = signal(String::new());
    let (search_results, set_search_results) = signal::<Vec<SearchResult>>(Vec::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let (total_results, set_total_results) = signal(0i64);
    let (current_page, set_current_page) = signal(1i32);
    let (filter_module, set_filter_module) = signal(String::from("all"));
    let (filter_status, set_filter_status) = signal(String::from("all"));
    let (filter_tahun, set_filter_tahun) = signal::<Option<i32>>(None);
    let (has_searched, set_has_searched) = signal(false);

    // Perform search
    let perform_search = move || {
        let query = search_query.get();
        if query.trim().is_empty() {
            set_error.set(Some("Masukkan kata kunci pencarian".to_string()));
            return;
        }

        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);
            set_has_searched.set(true);

            match search_all_modules(
                &query,
                filter_module.get(),
                filter_status.get(),
                filter_tahun.get(),
                current_page.get(),
            )
            .await
            {
                Ok(response) => {
                    set_search_results.set(response.results);
                    set_total_results.set(response.total);
                }
                Err(e) => set_error.set(Some(e.to_string())),
            }
            set_loading.set(false);
        });
    };

    // Change page
    let change_page = move |page: i32| {
        set_current_page.set(page);
        perform_search();
    };

    // Calculate pagination
    let total_pages = move || {
        let total = total_results.get();
        let per_page = 20;
        ((total as f64) / (per_page as f64)).ceil() as i32
    };

    view! {
        <div class="container mx-auto px-4 py-8">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-6">
                "Pencarian Global"
            </h1>

            {move || error.get().map(|e| view! {
                <Alert message=e variant=AlertVariant::Error />
            })}

            // Search Bar
            <Card class="mb-6">
                <div class="space-y-4">
                    <div class="flex gap-4">
                        <div class="flex-1">
                            <Input
                                label="Cari"
                                placeholder="Masukkan kata kunci (nama, nomor, satker, dll)..."
                                value=search_query.get()
                                on_input=Box::new(move |v| set_search_query.set(v))
                            />
                        </div>
                        <div class="flex items-end">
                            <Button
                                variant=ButtonVariant::Primary
                                size=ButtonSize::Large
                                on_click=Box::new(move || perform_search())
                                disabled=loading.get()
                            >
                                {if loading.get() {
                                    view! {
                                        <div class="flex items-center space-x-2">
                                            <Spinner size="sm" />
                                            <span>"Mencari..."</span>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="flex items-center space-x-2">
                                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                                            </svg>
                                            <span>"Cari"</span>
                                        </div>
                                    }.into_any()
                                }}
                            </Button>
                        </div>
                    </div>

                    // Filters
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4 pt-4 border-t">
                        <Select
                            label="Modul"
                            value=filter_module.get()
                            on_change=Box::new(move |v| {
                                set_filter_module.set(v);
                                if has_searched.get() {
                                    perform_search();
                                }
                            })
                        >
                            <option value="all">"Semua Modul"</option>
                            <option value="kebutuhan">"Kebutuhan BMN"</option>
                            <option value="pemakaian">"Pemakaian BMN"</option>
                            <option value="penghapusan">"Penghapusan BMN"</option>
                        </Select>

                        <Select
                            label="Status"
                            value=filter_status.get()
                            on_change=Box::new(move |v| {
                                set_filter_status.set(v);
                                if has_searched.get() {
                                    perform_search();
                                }
                            })
                        >
                            <option value="all">"Semua Status"</option>
                            <option value="DRAFT">"Draft"</option>
                            <option value="SUBMITTED">"Submitted"</option>
                            <option value="REVIEWED">"Reviewed"</option>
                            <option value="APPROVED">"Approved"</option>
                            <option value="REJECTED">"Rejected"</option>
                            <option value="COMPLETED">"Completed"</option>
                        </Select>

                        <Input
                            label="Tahun"
                            input_type="number"
                            placeholder="2024"
                            value=filter_tahun.get().map(|t| t.to_string()).unwrap_or_default()
                            on_input=Box::new(move |v| {
                                let tahun = if v.is_empty() {
                                    None
                                } else {
                                    v.parse::<i32>().ok()
                                };
                                set_filter_tahun.set(tahun);
                                if has_searched.get() {
                                    perform_search();
                                }
                            })
                        />
                    </div>
                </div>
            </Card>

            // Search Results
            {move || {
                if !has_searched.get() {
                    view! {
                        <Card>
                            <div class="text-center py-12">
                                <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                                </svg>
                                <h3 class="mt-2 text-sm font-medium text-gray-900">"Mulai Pencarian"</h3>
                                <p class="mt-1 text-sm text-gray-500">
                                    "Masukkan kata kunci untuk mencari di semua modul"
                                </p>
                            </div>
                        </Card>
                    }.into_any()
                } else if loading.get() {
                    view! {
                        <Card>
                            <div class="flex justify-center py-12">
                                <Spinner size="lg" />
                            </div>
                        </Card>
                    }.into_any()
                } else if search_results.get().is_empty() {
                    view! {
                        <Card>
                            <div class="text-center py-12">
                                <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                                <h3 class="mt-2 text-sm font-medium text-gray-900">"Tidak Ada Hasil"</h3>
                                <p class="mt-1 text-sm text-gray-500">
                                    "Tidak ditemukan hasil untuk kata kunci \"" {search_query.get()} "\""
                                </p>
                            </div>
                        </Card>
                    }.into_any()
                } else {
                    view! {
                        <div class="space-y-6">
                            // Results Summary
                            <div class="flex items-center justify-between">
                                <p class="text-sm text-gray-700">
                                    "Ditemukan " <span class="font-semibold">{total_results.get()}</span>
                                    " hasil untuk \"" <span class="font-semibold">{search_query.get()}</span> "\""
                                </p>
                            </div>

                            // Results List
                            <div class="space-y-4">
                                {search_results.get().into_iter().map(|result| {
                                    let url = result.url.clone();
                                    view! {
                                        <Card>
                                            <a
                                                href=url
                                                class="block hover:bg-gray-50 transition-colors -m-6 p-6"
                                            >
                                                <div class="flex items-start justify-between">
                                                    <div class="flex-1">
                                                        <div class="flex items-center space-x-2 mb-2">
                                                            <Badge
                                                                label=result.module.clone()
                                                                variant=match result.module.as_str() {
                                                                    "kebutuhan" => BadgeVariant::Info,
                                                                    "pemakaian" => BadgeVariant::Success,
                                                                    "penghapusan" => BadgeVariant::Warning,
                                                                    _ => BadgeVariant::Default,
                                                                }
                                                            />
                                                            <Badge
                                                                label=result.status.clone()
                                                                variant=match result.status.as_str() {
                                                                    "COMPLETED" | "APPROVED" => BadgeVariant::Success,
                                                                    "DRAFT" => BadgeVariant::Warning,
                                                                    "REJECTED" => BadgeVariant::Danger,
                                                                    _ => BadgeVariant::Info,
                                                                }
                                                            />
                                                            {result.tahun.map(|t| view! {
                                                                <span class="text-xs text-gray-500">
                                                                    "Tahun " {t}
                                                                </span>
                                                            })}
                                                        </div>
                                                        <h3 class="text-lg font-semibold text-gray-900 mb-1">
                                                            {result.title.clone()}
                                                        </h3>
                                                        <p class="text-sm text-gray-600 mb-2">
                                                            {result.description.clone()}
                                                        </p>
                                                        <div class="flex items-center space-x-4 text-xs text-gray-500">
                                                            {result.satker_nama.as_ref().map(|satker| view! {
                                                                <span class="flex items-center">
                                                                    <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
                                                                    </svg>
                                                                    {satker.clone()}
                                                                </span>
                                                            })}
                                                            <span class="flex items-center">
                                                                <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                                                                </svg>
                                                                {result.created_at.clone()}
                                                            </span>
                                                        </div>
                                                    </div>
                                                    <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                                                    </svg>
                                                </div>
                                            </a>
                                        </Card>
                                    }
                                }).collect_view()}
                            </div>

                            // Pagination
                            {move || {
                                let pages = total_pages();
                                let can_go_next = current_page.get() < pages;
                                if pages > 1 {
                                    view! {
                                        <div class="flex justify-center items-center space-x-2">
                                            <Button
                                                variant=ButtonVariant::Secondary
                                                size=ButtonSize::Small
                                                on_click=Box::new(move || {
                                                    let page = current_page.get();
                                                    if page > 1 {
                                                        change_page(page - 1);
                                                    }
                                                })
                                                disabled=current_page.get() == 1
                                            >
                                                "Previous"
                                            </Button>

                                            <span class="text-sm text-gray-700">
                                                "Halaman " {current_page.get()} " dari " {pages}
                                            </span>

                                            <Button
                                                variant=ButtonVariant::Secondary
                                                size=ButtonSize::Small
                                                on_click=Box::new(move || {
                                                    let page = current_page.get();
                                                    if page < pages {
                                                        change_page(page + 1);
                                                    }
                                                })
                                                disabled=!can_go_next
                                            >
                                                "Next"
                                            </Button>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <div></div> }.into_any()
                                }
                            }}
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

// ============================================================================
// API Functions
// ============================================================================

async fn search_all_modules(
    query: &str,
    module: String,
    status: String,
    tahun: Option<i32>,
    page: i32,
) -> Result<SearchResponse, crate::api::AppError> {
    let mut url = "/api/v1/perlengkapan/search?".to_string();
    url.push_str(&format!("q={}", query));
    url.push_str(&format!("&page={}", page));

    if module != "all" {
        url.push_str(&format!("&module={}", module));
    }
    if status != "all" {
        url.push_str(&format!("&status={}", status));
    }
    if let Some(t) = tahun {
        url.push_str(&format!("&tahun={}", t));
    }

    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Network error: {}", e)))?;

    if !response.ok() {
        return Err(crate::api::AppError::Unknown(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let api_response: ApiResponse<SearchResponse> = response
        .json()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Parse error: {}", e)))?;

    api_response
        .data
        .ok_or_else(|| crate::api::AppError::Unknown("No data in response".to_string()))
}
