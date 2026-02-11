//! Roadmap Sarpras List Component
//!
//! Displays list of roadmap items with filtering and management.

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapSarpras {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub periode_mulai: i32,
    pub periode_akhir: i32,
    pub kode_barang: String,
    pub nama_barang: String,
    pub tahun_rencana: i32,
    pub jumlah_kebutuhan: i32,
    pub jumlah_terpenuhi: i32,
    pub estimasi_anggaran: Option<f64>,
    pub realisasi_anggaran: Option<f64>,
    pub status_pemenuhan: String,
    pub keterangan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRoadmapResponse {
    pub roadmaps: Vec<RoadmapSarpras>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[component]
pub fn RoadmapSarprasList() -> impl IntoView {
    let (page, set_page) = signal(1);
    let (per_page, _set_per_page) = signal(20);
    let (tahun_filter, set_tahun_filter) = signal::<Option<i32>>(None);
    let (periode_filter, set_periode_filter) = signal::<Option<i32>>(None);
    let (refresh_trigger, set_refresh_trigger) = signal(0);

    // Fetch data resource
    let data_resource = LocalResource::new(move || {
        let p = page.get();
        let pp = per_page.get();
        let tahun = tahun_filter.get();
        let periode = periode_filter.get();
        let _ = refresh_trigger.get();

        async move {
            fetch_roadmap_list(tahun, periode, p, pp).await
        }
    });

    // Reset page when filters change
    Effect::new(move || {
        let _ = tahun_filter.get();
        let _ = periode_filter.get();
        set_page.set(1);
    });

    // Current year for filter
    let current_year = 2025;
    let years: Vec<i32> = (2020..=current_year + 5).rev().collect();

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <div class="flex justify-between items-center mb-6">
                <h2 class="text-2xl font-bold text-gray-900">
                    "Daftar Roadmap Sarpras"
                </h2>
                <a
                    href="/roadmap-sarpras/create"
                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                >
                    "+ Buat Roadmap Baru"
                </a>
            </div>

            // Filters
            <div class="mb-6 flex gap-4">
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">
                        "Tahun Rencana"
                    </label>
                    <select
                        class="block w-48 rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            set_tahun_filter.set(if value.is_empty() {
                                None
                            } else {
                                value.parse::<i32>().ok()
                            });
                        }
                    >
                        <option value="">"Semua Tahun"</option>
                        {years.iter().map(|&year| view! {
                            <option value=year.to_string()>{year}</option>
                        }).collect_view()}
                    </select>
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">
                        "Periode Mulai"
                    </label>
                    <select
                        class="block w-48 rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            set_periode_filter.set(if value.is_empty() {
                                None
                            } else {
                                value.parse::<i32>().ok()
                            });
                        }
                    >
                        <option value="">"Semua Periode"</option>
                        {years.iter().map(|&year| view! {
                            <option value=year.to_string()>{format!("{} - {}", year, year + 4)}</option>
                        }).collect_view()}
                    </select>
                </div>
            </div>

            // Data Table
            <Suspense fallback=move || view! {
                <div class="flex justify-center items-center py-12">
                    <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
                </div>
            }>
                {move || {
                    data_resource.get().map(|result| {
                        match result {
                            Ok(data) => {
                                if data.roadmaps.is_empty() {
                                    view! {
                                        <div class="text-center py-12 text-gray-500">
                                            "Tidak ada data roadmap."
                                        </div>
                                    }.into_any()
                                } else {
                                    let roadmaps_count = data.roadmaps.len();
                                    view! {
                                        <div>
                                            <div class="overflow-x-auto">
                                                <table class="min-w-full divide-y divide-gray-200">
                                                    <thead class="bg-gray-50">
                                                        <tr>
                                                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                                                "Periode"
                                                            </th>
                                                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                                                "Tahun"
                                                            </th>
                                                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                                                "Kode Barang"
                                                            </th>
                                                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                                                "Nama Barang"
                                                            </th>
                                                            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                                                                "Kebutuhan"
                                                            </th>
                                                            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                                                                "Terpenuhi"
                                                            </th>
                                                            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                                                                "Progress"
                                                            </th>
                                                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                                                "Status"
                                                            </th>
                                                            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                                                                "Aksi"
                                                            </th>
                                                        </tr>
                                                    </thead>
                                                    <tbody class="bg-white divide-y divide-gray-200">
                                                        {data.roadmaps.into_iter().map(|item| {
                                                            let persentase = if item.jumlah_kebutuhan > 0 {
                                                                (item.jumlah_terpenuhi as f64 / item.jumlah_kebutuhan as f64) * 100.0
                                                            } else {
                                                                0.0
                                                            };

                                                            let status_class = match item.status_pemenuhan.as_str() {
                                                                "COMPLETED" => "bg-green-100 text-green-800",
                                                                "IN_PROGRESS" => "bg-yellow-100 text-yellow-800",
                                                                _ => "bg-gray-100 text-gray-800",
                                                            };

                                                            view! {
                                                                <tr class="hover:bg-gray-50">
                                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                        {format!("{} - {}", item.periode_mulai, item.periode_akhir)}
                                                                    </td>
                                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                        {item.tahun_rencana}
                                                                    </td>
                                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                        {item.kode_barang}
                                                                    </td>
                                                                    <td class="px-6 py-4 text-sm text-gray-900">
                                                                        {item.nama_barang}
                                                                    </td>
                                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">
                                                                        {item.jumlah_kebutuhan}
                                                                    </td>
                                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">
                                                                        {item.jumlah_terpenuhi}
                                                                    </td>
                                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-right">
                                                                        <div class="flex items-center justify-end gap-2">
                                                                            <div class="w-16 bg-gray-200 rounded-full h-2">
                                                                                <div
                                                                                    class="bg-blue-600 h-2 rounded-full"
                                                                                    style=format!("width: {}%", persentase.min(100.0))
                                                                                ></div>
                                                                            </div>
                                                                            <span class="text-gray-900">
                                                                                {format!("{:.0}%", persentase)}
                                                                            </span>
                                                                        </div>
                                                                    </td>
                                                                    <td class="px-6 py-4 whitespace-nowrap">
                                                                        <span class=format!("px-2 py-1 text-xs font-medium rounded-full {}", status_class)>
                                                                            {item.status_pemenuhan}
                                                                        </span>
                                                                    </td>
                                                                    <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                                                        <a
                                                                            href=format!("/roadmap-sarpras/{}", item.id)
                                                                            class="text-blue-600 hover:text-blue-900 mr-3"
                                                                        >
                                                                            "Detail"
                                                                        </a>
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }).collect_view()}
                                                    </tbody>
                                                </table>
                                            </div>

                                            // Pagination
                                            <div class="mt-6 flex justify-between items-center">
                                                <div class="text-sm text-gray-700">
                                                    {format!("Menampilkan {} dari {} total", roadmaps_count, data.total)}
                                                </div>
                                                <div class="flex gap-2">
                                                    <button
                                                        class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                                                        disabled=move || page.get() == 1
                                                        on:click=move |_| set_page.update(|p| *p -= 1)
                                                    >
                                                        "← Sebelumnya"
                                                    </button>
                                                    <button
                                                        class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                                                        disabled=move || {
                                                            let current_page = page.get();
                                                            let total_pages = (data.total as f64 / per_page.get() as f64).ceil() as i64;
                                                            current_page >= total_pages
                                                        }
                                                        on:click=move |_| set_page.update(|p| *p += 1)
                                                    >
                                                        "Selanjutnya →"
                                                    </button>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_any()
                                }
                            }
                            Err(e) => view! {
                                <div class="text-center py-12 text-red-600">
                                    {format!("Error: {}", e)}
                                </div>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

// API function
async fn fetch_roadmap_list(
    tahun_rencana: Option<i32>,
    periode_mulai: Option<i32>,
    page: i64,
    per_page: i64,
) -> Result<ListRoadmapResponse, String> {
    use gloo_net::http::Request;

    let mut url = format!("/api/v1/roadmap-sarpras?limit={}&offset={}", per_page, (page - 1) * per_page);

    if let Some(tahun) = tahun_rencana {
        url.push_str(&format!("&tahun_rencana={}", tahun));
    }

    if let Some(periode) = periode_mulai {
        url.push_str(&format!("&periode_mulai={}", periode));
    }

    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.ok() {
        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(error_text)
    }
}
