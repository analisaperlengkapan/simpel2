//! Roadmap Sarpras Timeline Visualization Component
//!
//! Displays 5-year roadmap with timeline visualization and realization tracking.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapRealizationComparison {
    pub roadmap_id: Uuid,
    pub satker_id: Uuid,
    pub kode_barang: String,
    pub nama_barang: String,
    pub tahun_rencana: i32,
    pub jumlah_kebutuhan: i32,
    pub jumlah_terpenuhi: i32,
    pub persentase_pemenuhan: f64,
    pub estimasi_anggaran: Option<f64>,
    pub realisasi_anggaran: Option<f64>,
    pub status_pemenuhan: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapSummary {
    pub total_items: i64,
    pub total_kebutuhan: i64,
    pub total_terpenuhi: i64,
    pub persentase_pemenuhan_rata_rata: f64,
    pub total_estimasi_anggaran: f64,
    pub total_realisasi_anggaran: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapComparisonResponse {
    pub comparisons: Vec<RoadmapRealizationComparison>,
    pub summary: RoadmapSummary,
}

#[component]
pub fn RoadmapSarprasTimeline(
    satker_id: Uuid,
    periode_mulai: i32,
    periode_akhir: i32,
) -> impl IntoView {
    // Fetch roadmap comparison data
    let data_resource = LocalResource::new(move || async move {
        fetch_roadmap_comparison(satker_id, periode_mulai, periode_akhir).await
    });

    view! {
        <div class="bg-white rounded-xl shadow-sm border border-gray-100 p-6">
            <h2 class="text-2xl font-bold text-gray-900 mb-6">
                {format!("Roadmap Sarpras {} - {}", periode_mulai, periode_akhir)}
            </h2>

            <Suspense fallback=move || view! {
                <div class="flex justify-center items-center py-12">
                    <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
                </div>
            }>
                {move || {
                    data_resource.get().map(|result| {
                        match result {
                            Ok(data) => view! {
                                <div>
                                    // Summary Cards
                                    <div class="grid grid-cols-4 gap-4 mb-8">
                                        <div class="p-4 bg-blue-50 rounded-lg">
                                            <div class="text-sm text-gray-600">"Total Item"</div>
                                            <div class="text-2xl font-bold text-blue-600">
                                                {data.summary.total_items}
                                            </div>
                                        </div>
                                        <div class="p-4 bg-green-50 rounded-lg">
                                            <div class="text-sm text-gray-600">"Total Kebutuhan"</div>
                                            <div class="text-2xl font-bold text-green-600">
                                                {data.summary.total_kebutuhan}
                                            </div>
                                        </div>
                                        <div class="p-4 bg-purple-50 rounded-lg">
                                            <div class="text-sm text-gray-600">"Total Terpenuhi"</div>
                                            <div class="text-2xl font-bold text-purple-600">
                                                {data.summary.total_terpenuhi}
                                            </div>
                                        </div>
                                        <div class="p-4 bg-orange-50 rounded-lg">
                                            <div class="text-sm text-gray-600">"Rata-rata Pemenuhan"</div>
                                            <div class="text-2xl font-bold text-orange-600">
                                                {format!("{:.1}%", data.summary.persentase_pemenuhan_rata_rata)}
                                            </div>
                                        </div>
                                    </div>

                                    // Timeline Visualization
                                    <div class="mb-8">
                                        <h3 class="text-lg font-semibold text-gray-900 mb-4">
                                            "Timeline Roadmap"
                                        </h3>
                                        <RoadmapTimelineChart
                                            comparisons=data.comparisons.clone()
                                            periode_mulai=periode_mulai
                                            periode_akhir=periode_akhir
                                        />
                                    </div>

                                    // Detailed Table
                                    <div>
                                        <h3 class="text-lg font-semibold text-gray-900 mb-4">
                                            "Detail Roadmap vs Realisasi"
                                        </h3>
                                        <RoadmapComparisonTable comparisons=data.comparisons />
                                    </div>
                                </div>
                            }.into_any(),
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

#[component]
fn RoadmapTimelineChart(
    comparisons: Vec<RoadmapRealizationComparison>,
    periode_mulai: i32,
    periode_akhir: i32,
) -> impl IntoView {
    // Group by year
    let years: Vec<i32> = (periode_mulai..=periode_akhir).collect();

    view! {
        <div class="overflow-x-auto">
            <div class="min-w-full">
                {years.into_iter().map(|year| {
                    let year_items: Vec<_> = comparisons.iter()
                        .filter(|c| c.tahun_rencana == year)
                        .collect();

                    let total_kebutuhan: i32 = year_items.iter().map(|c| c.jumlah_kebutuhan).sum();
                    let total_terpenuhi: i32 = year_items.iter().map(|c| c.jumlah_terpenuhi).sum();
                    let persentase = if total_kebutuhan > 0 {
                        (total_terpenuhi as f64 / total_kebutuhan as f64) * 100.0
                    } else {
                        0.0
                    };

                    view! {
                        <div class="mb-4 p-4 border border-gray-200 rounded-lg">
                            <div class="flex justify-between items-center mb-2">
                                <h4 class="font-semibold text-gray-900">{year}</h4>
                                <span class="text-sm text-gray-600">
                                    {format!("{} / {} item ({:.1}%)", total_terpenuhi, total_kebutuhan, persentase)}
                                </span>
                            </div>
                            <div class="w-full bg-gray-200 rounded-full h-4">
                                <div
                                    class="bg-blue-600 h-4 rounded-full transition-all duration-300"
                                    style=format!("width: {}%", persentase.min(100.0))
                                ></div>
                            </div>
                            <div class="mt-2 text-xs text-gray-500">
                                {format!("{} item direncanakan", year_items.len())}
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn RoadmapComparisonTable(
    comparisons: Vec<RoadmapRealizationComparison>,
) -> impl IntoView {
    view! {
        <div class="overflow-x-auto">
            <table class="min-w-full divide-y divide-gray-200">
                <thead class="bg-gray-50">
                    <tr>
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
                            "Pemenuhan"
                        </th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            "Status"
                        </th>
                    </tr>
                </thead>
                <tbody class="bg-white divide-y divide-gray-200">
                    {comparisons.into_iter().map(|item| {
                        let status_class = match item.status_pemenuhan.as_str() {
                            "COMPLETED" => "bg-green-100 text-green-800",
                            "IN_PROGRESS" => "bg-yellow-100 text-yellow-800",
                            _ => "bg-gray-100 text-gray-800",
                        };

                        view! {
                            <tr class="hover:bg-gray-50">
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
                                                style=format!("width: {}%", item.persentase_pemenuhan.min(100.0))
                                            ></div>
                                        </div>
                                        <span class="text-gray-900">
                                            {format!("{:.1}%", item.persentase_pemenuhan)}
                                        </span>
                                    </div>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <span class=format!("px-2 py-1 text-xs font-medium rounded-full {}", status_class)>
                                        {item.status_pemenuhan}
                                    </span>
                                </td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
}

// API function
async fn fetch_roadmap_comparison(
    satker_id: Uuid,
    periode_mulai: i32,
    periode_akhir: i32,
) -> Result<RoadmapComparisonResponse, String> {
    use gloo_net::http::Request;

    let url = format!(
        "/api/v1/roadmap-sarpras/comparison?satker_id={}&periode_mulai={}&periode_akhir={}",
        satker_id, periode_mulai, periode_akhir
    );

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
