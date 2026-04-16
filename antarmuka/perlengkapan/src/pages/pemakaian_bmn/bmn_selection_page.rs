//! # Pemakaian BMN - BMN Selection Page
//!
//! This page allows users to:
//! - Search and select BMN items from SIMAN integration
//! - Check BMN availability in real-time
//! - View BMN details and current usage status
//! - Add multiple BMN items to a permit
//!
//! Requirements: REQ-P002, REQ-P003, REQ-P004

use leptos::prelude::*;
use lib_ui::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDate;
use crate::routes;

// ============================================================================
// API Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmnItem {
    pub nup: String,
    pub kode_barang: String,
    pub nama_barang: String,
    pub merk: Option<String>,
    pub kondisi: String,
    pub tahun_perolehan: Option<i32>,
    pub nilai_perolehan: Option<f64>,
    pub satker_nama: String,
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmnAvailabilityResponse {
    pub bmn_nup: String,
    pub is_available: bool,
    pub active_permit_id: Option<Uuid>,
    pub active_permit_holder: Option<String>,
    pub active_permit_expires: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmnUsageStats {
    pub bmn_nup: String,
    pub bmn_nama: String,
    pub total_permits: i64,
    pub active_permits: i64,
    pub total_days_used: i64,
    pub current_holder: Option<String>,
    pub permit_history: Vec<PermitHistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermitHistoryEntry {
    pub id: Uuid,
    pub nomor_izin: Option<String>,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: NaiveDate,
    pub status: String,
    pub created_at: String,
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
pub fn BmnSelectionPage() -> impl IntoView {
    // State
    let (search_query, set_search_query) = signal(String::new());
    let (bmn_list, set_bmn_list) = signal::<Vec<BmnItem>>(Vec::new());
    let (selected_bmn, set_selected_bmn) = signal::<Vec<BmnItem>>(Vec::new());
    let (selected_bmn_detail, set_selected_bmn_detail) = signal::<Option<BmnItem>>(None);
    let (availability_info, set_availability_info) = signal::<Option<BmnAvailabilityResponse>>(None);
    let (usage_stats, set_usage_stats) = signal::<Option<BmnUsageStats>>(None);
    let (loading, set_loading) = signal(false);
    let (checking_availability, set_checking_availability) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let (filter_jenis, set_filter_jenis) = signal(String::from("all"));
    let (filter_kondisi, set_filter_kondisi) = signal(String::from("all"));

    // Load BMN list on mount
    Effect::new(move || {
        spawn_local(async move {
            set_loading.set(true);
            match fetch_bmn_list().await {
                Ok(list) => set_bmn_list.set(list),
                Err(e) => set_error.set(Some(e.to_string())),
            }
            set_loading.set(false);
        });
    });

    // Check availability when BMN is selected
    let check_availability = move |nup: String| {
        spawn_local(async move {
            set_checking_availability.set(true);
            set_error.set(None);

            match check_bmn_availability(&nup).await {
                Ok(info) => {
                    set_availability_info.set(Some(info));

                    // Also fetch usage history
                    if let Ok(stats) = fetch_bmn_usage_history(&nup).await {
                        set_usage_stats.set(Some(stats));
                    }
                }
                Err(e) => set_error.set(Some(e.to_string())),
            }
            set_checking_availability.set(false);
        });
    };

    // Add BMN to selection
    let add_bmn = move |bmn: BmnItem| {
        let mut current = selected_bmn.get();
        if !current.iter().any(|b| b.nup == bmn.nup) {
            current.push(bmn);
            set_selected_bmn.set(current);
        }
    };

    // Remove BMN from selection
    let remove_bmn = move |nup: String| {
        let current = selected_bmn.get();
        let filtered: Vec<BmnItem> = current.into_iter().filter(|b| b.nup != nup).collect();
        set_selected_bmn.set(filtered);
    };

    // Filter BMN list
    let filtered_bmn = move || {
        let list = bmn_list.get();
        let query = search_query.get().to_lowercase();
        let jenis = filter_jenis.get();
        let kondisi = filter_kondisi.get();

        list.into_iter()
            .filter(|bmn| {
                // Search filter
                let matches_search = query.is_empty()
                    || bmn.nama_barang.to_lowercase().contains(&query)
                    || bmn.nup.to_lowercase().contains(&query)
                    || bmn.kode_barang.to_lowercase().contains(&query);

                // Jenis filter (based on kode_barang prefix)
                let matches_jenis = jenis == "all" || match jenis.as_str() {
                    "kendaraan" => bmn.kode_barang.starts_with("3.1"),
                    "laptop" => bmn.kode_barang.starts_with("3.2"),
                    "rumah" => bmn.kode_barang.starts_with("4.1"),
                    _ => true,
                };

                // Kondisi filter
                let matches_kondisi = kondisi == "all" || bmn.kondisi == kondisi;

                matches_search && matches_jenis && matches_kondisi
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="container mx-auto px-4 py-8">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-6">
                "Pilih BMN untuk Izin Pemakaian"
            </h1>

            {move || error.get().map(|e| view! {
                <Alert message=e variant=AlertVariant::Error />
            })}

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                // Left Panel: Search and Filters
                <div class="lg:col-span-2">
                    <Card title="Cari BMN">
                        <div class="space-y-4">
                            // Search Input
                            <Input
                                label="Cari BMN"
                                placeholder="Cari berdasarkan nama, NUP, atau kode barang..."
                                value=move || search_query.get()
                                on_input=Box::new(move |v| set_search_query.set(v))
                            />

                            // Filters
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <Select
                                    label="Jenis BMN"
                                    value=Some(filter_jenis.get())
                                    on_change=Box::new(move |v| set_filter_jenis.set(v))
                                >
                                    <option value="all">"Semua Jenis"</option>
                                    <option value="kendaraan">"Kendaraan Bermotor"</option>
                                    <option value="laptop">"Laptop/Komputer"</option>
                                    <option value="rumah">"Rumah Negara"</option>
                                </Select>

                                <Select
                                    label="Kondisi"
                                    value=Some(filter_kondisi.get())
                                    on_change=Box::new(move |v| set_filter_kondisi.set(v))
                                >
                                    <option value="all">"Semua Kondisi"</option>
                                    <option value="BAIK">"Baik"</option>
                                    <option value="RUSAK RINGAN">"Rusak Ringan"</option>
                                    <option value="RUSAK BERAT">"Rusak Berat"</option>
                                </Select>
                            </div>
                        </div>
                    </Card>

                    // BMN List
                    <Card title=move || format!("Daftar BMN ({})", filtered_bmn().len()) class="mt-6">
                        {move || {
                            if loading.get() {
                                view! {
                                    <div class="flex justify-center py-8">
                                        <Spinner size="lg" />
                                    </div>
                                }.into_any()
                            } else {
                                let list = filtered_bmn();
                                if list.is_empty() {
                                    view! {
                                        <div class="text-center py-8 text-gray-500">
                                            "Tidak ada BMN ditemukan"
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="space-y-3 max-h-[600px] overflow-y-auto">
                                            {list.into_iter().map(|bmn| {
                                                let nup = bmn.nup.clone();
                                                let nup_check = nup.clone();
                                                let is_selected = selected_bmn.get().iter().any(|b| b.nup == nup);

                                                view! {
                                                    <div class="border rounded-lg p-4 hover:bg-gray-50 transition-colors">
                                                        <div class="flex justify-between items-start">
                                                            <div class="flex-1">
                                                                <h4 class="font-semibold text-gray-900">
                                                                    {bmn.nama_barang.clone()}
                                                                </h4>
                                                                <div class="mt-2 space-y-1 text-sm text-gray-600">
                                                                    <p>"NUP: " <span class="font-mono">{bmn.nup.clone()}</span></p>
                                                                    <p>"Kode: " {bmn.kode_barang.clone()}</p>
                                                                    {bmn.merk.as_ref().map(|m| view! {
                                                                        <p>"Merk: " {m.clone()}</p>
                                                                    })}
                                                                    {bmn.tahun_perolehan.map(|t| view! {
                                                                        <p>"Tahun: " {t}</p>
                                                                    })}
                                                                    <p>"Satker: " {bmn.satker_nama.clone()}</p>
                                                                </div>
                                                                <div class="mt-2 flex items-center space-x-2">
                                                                    <Badge
                                                                        label=bmn.kondisi.clone()
                                                                        variant=match bmn.kondisi.as_str() {
                                                                            "BAIK" => BadgeVariant::Success,
                                                                            "RUSAK RINGAN" => BadgeVariant::Warning,
                                                                            "RUSAK BERAT" => BadgeVariant::Error,
                                                                            _ => BadgeVariant::Info,
                                                                        }
                                                                    />
                                                                    {if bmn.is_available {
                                                                        view! {
                                                                            <Badge label="Tersedia" variant=BadgeVariant::Success />
                                                                        }.into_any()
                                                                    } else {
                                                                        view! {
                                                                            <Badge label="Sedang Digunakan" variant=BadgeVariant::Warning />
                                                                        }.into_any()
                                                                    }}
                                                                </div>
                                                            </div>
                                                            <div class="flex flex-col space-y-2 ml-4">
                                                                <Button
                                                                    variant=ButtonVariant::Secondary
                                                                    size=ButtonSize::Small
                                                                    on_click=Box::new(move |_| {
                                                                        set_selected_bmn_detail.set(Some(bmn.clone()));
                                                                        check_availability(nup_check.clone());
                                                                    })
                                                                >
                                                                    "Detail"
                                                                </Button>
                                                                <Button
                                                                    variant=if is_selected { ButtonVariant::Secondary } else { ButtonVariant::Primary }
                                                                    size=ButtonSize::Small
                                                                    on_click=Box::new(move |_| {
                                                                        if is_selected {
                                                                            remove_bmn(nup.clone());
                                                                        } else {
                                                                            add_bmn(bmn.clone());
                                                                        }
                                                                    })
                                                                    disabled=!bmn.is_available
                                                                >
                                                                    {if is_selected { "Hapus" } else { "Pilih" }}
                                                                </Button>
                                                            </div>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                }
                            }
                        }}
                    </Card>
                </div>

                // Right Panel: Selected BMN and Details
                <div class="lg:col-span-1">
                    // Selected BMN Summary
                    <Card title="BMN Terpilih">
                        <div class="space-y-3">
                            {move || {
                                let selected = selected_bmn.get();
                                if selected.is_empty() {
                                    view! {
                                        <div class="text-center py-8 text-gray-500">
                                            <p>"Belum ada BMN dipilih"</p>
                                            <p class="text-sm mt-2">"Pilih minimal 1 BMN (REQ-P002)"</p>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="space-y-2">
                                            <div class="p-3 bg-emerald-50 rounded-lg">
                                                <p class="font-semibold text-emerald-900">
                                                    "Total: " {selected.len()} " BMN"
                                                </p>
                                            </div>
                                            {selected.into_iter().map(|bmn| {
                                                let nup = bmn.nup.clone();
                                                view! {
                                                    <div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                                                        <div class="flex-1">
                                                            <p class="font-medium text-gray-900 text-sm">
                                                                {bmn.nama_barang.clone()}
                                                            </p>
                                                            <p class="text-xs text-gray-600 font-mono">
                                                                {bmn.nup.clone()}
                                                            </p>
                                                        </div>
                                                        <button
                                                            class="text-red-600 hover:text-red-800"
                                                            on:click=move |_| remove_bmn(nup.clone())
                                                        >
                                                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                                            </svg>
                                                        </button>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>
                    </Card>

                    // BMN Detail and Availability (REQ-P003, REQ-P004)
                    {move || selected_bmn_detail.get().map(|bmn| view! {
                        <Card title="Detail BMN" class="mt-6">
                            <div class="space-y-4">
                                <div>
                                    <h4 class="font-semibold text-gray-900">{bmn.nama_barang.clone()}</h4>
                                    <p class="text-sm text-gray-600 font-mono">"NUP: " {bmn.nup.clone()}</p>
                                </div>

                                {move || {
                                    if checking_availability.get() {
                                        view! {
                                            <div class="flex justify-center py-4">
                                                <Spinner size="sm" />
                                            </div>
                                        }.into_any()
                                    } else if let Some(info) = availability_info.get() {
                                        view! {
                                            <div class="space-y-3">
                                                <div class=move || format!(
                                                    "p-3 rounded-lg {}",
                                                    if info.is_available {
                                                        "bg-green-50 border border-green-200"
                                                    } else {
                                                        "bg-yellow-50 border border-yellow-200"
                                                    }
                                                )>
                                                    <p class=move || format!(
                                                        "font-semibold {}",
                                                        if info.is_available { "text-green-900" } else { "text-yellow-900" }
                                                    )>
                                                        {if info.is_available {
                                                            "✓ BMN Tersedia (REQ-P003)"
                                                        } else {
                                                            "⚠ BMN Sedang Digunakan (REQ-P004)"
                                                        }}
                                                    </p>
                                                    {if !info.is_available {
                                                        view! {
                                                            <div class="mt-2 text-sm text-yellow-800">
                                                                {info.active_permit_holder.as_ref().map(|holder| view! {
                                                                    <p>"Digunakan oleh: " <span class="font-semibold">{holder.clone()}</span></p>
                                                                })}
                                                                {info.active_permit_expires.map(|date| view! {
                                                                    <p>"Berakhir: " {date.to_string()}</p>
                                                                })}
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! { <div></div> }.into_any()
                                                    }}
                                                </div>

                                                // Usage History
                                                {move || usage_stats.get().map(|stats| view! {
                                                    <div class="border-t pt-3">
                                                        <h5 class="font-semibold text-gray-900 mb-2">"Riwayat Pemakaian"</h5>
                                                        <div class="space-y-2 text-sm">
                                                            <div class="flex justify-between">
                                                                <span class="text-gray-600">"Total Izin:"</span>
                                                                <span class="font-semibold">{stats.total_permits}</span>
                                                            </div>
                                                            <div class="flex justify-between">
                                                                <span class="text-gray-600">"Izin Aktif:"</span>
                                                                <span class="font-semibold">{stats.active_permits}</span>
                                                            </div>
                                                            <div class="flex justify-between">
                                                                <span class="text-gray-600">"Total Hari Digunakan:"</span>
                                                                <span class="font-semibold">{stats.total_days_used}</span>
                                                            </div>
                                                        </div>
                                                    </div>
                                                })}
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }
                                }}
                            </div>
                        </Card>
                    })}
                </div>
            </div>

            // Action Buttons
            {move || !selected_bmn.get().is_empty().then(|| view! {
                <div class="mt-6 flex justify-end space-x-4">
                    <Button
                        variant=ButtonVariant::Secondary
                        on_click=Box::new(|_| {
                            web_sys::window().unwrap().history().unwrap().back().ok();
                        })
                    >
                        "Batal"
                    </Button>
                    <Button
                        variant=ButtonVariant::Primary
                        size=ButtonSize::Large
                        on_click=Box::new(move |_| {
                            // Navigate to next step or save selection
                            web_sys::window()
                                .unwrap()
                                .location()
                                .set_href(routes::path::PENGELOLAAN_PEMAKAIAN_BUAT)
                                .ok();
                        })
                    >
                        "Lanjutkan"
                    </Button>
                </div>
            })}
        </div>
    }
}

// ============================================================================
// API Functions
// ============================================================================

async fn fetch_bmn_list() -> Result<Vec<BmnItem>, crate::api::AppError> {
    let response = gloo_net::http::Request::get("/api/v1/pemakaian-bmn/bmn/available")
        .send()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Network error: {}", e)))?;

    if !response.ok() {
        return Err(crate::api::AppError::Unknown(format!("HTTP error: {}", response.status())));
    }

    let api_response: ApiResponse<Vec<BmnItem>> = response
        .json()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Parse error: {}", e)))?;

    Ok(api_response.data.unwrap_or_default())
}

async fn check_bmn_availability(nup: &str) -> Result<BmnAvailabilityResponse, crate::api::AppError> {
    let url = format!("/api/v1/pemakaian-bmn/bmn/{}/availability", nup);
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Network error: {}", e)))?;

    if !response.ok() {
        return Err(crate::api::AppError::Unknown(format!("HTTP error: {}", response.status())));
    }

    let api_response: ApiResponse<BmnAvailabilityResponse> = response
        .json()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Parse error: {}", e)))?;

    api_response.data.ok_or_else(|| crate::api::AppError::Unknown("No data in response".to_string()))
}

async fn fetch_bmn_usage_history(nup: &str) -> Result<BmnUsageStats, crate::api::AppError> {
    let url = format!("/api/v1/pemakaian-bmn/bmn/{}/history", nup);
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Network error: {}", e)))?;

    if !response.ok() {
        return Err(crate::api::AppError::Unknown(format!("HTTP error: {}", response.status())));
    }

    let api_response: ApiResponse<BmnUsageStats> = response
        .json()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Parse error: {}", e)))?;

    api_response.data.ok_or_else(|| crate::api::AppError::Unknown("No data in response".to_string()))
}
