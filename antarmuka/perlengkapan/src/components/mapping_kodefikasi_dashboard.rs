//! Mapping Kodefikasi Dashboard Component
//!
//! Read-only dashboard showing standard and non-standard BMN codes
//! with export capabilities (PDF/XLSX). No proposal/verification.

use leptos::prelude::*;
use leptos_fetch::QueryClient;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingProgress {
    pub total_non_standard: i64,
    pub total_mapped: i64,
    pub total_standard: i64,
    pub mapping_percentage: f64,
    pub non_standard_codes: Vec<NonStandardCodeWithStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonStandardCodeWithStatus {
    pub kode_lama: String,
    pub nama_lama: String,
    pub satker_id: Uuid,
    pub satker_nama: String,
    pub jumlah_aset: i64,
    pub status_mapping: Option<String>,
    pub kode_baru: Option<String>,
    pub nama_baru: Option<String>,
}

/// leptos-fetch query — `()` key, returns `Option<MappingProgress>`
/// (None on transport/HTTP error so the component can render a
/// dedicated empty/error state without bubbling a `Result`).
async fn query_mapping_progress(_: ()) -> Option<MappingProgress> {
    fetch_mapping_progress()
        .await
        .map_err(|e| {
            leptos::logging::error!("Failed to fetch mapping progress: {}", e);
        })
        .ok()
}

async fn fetch_mapping_progress() -> Result<MappingProgress, String> {
    let response = gloo_net::http::Request::get("/api/v1/perlengkapan/perlengkapan/mapping/progress")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    response
        .json::<MappingProgress>()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))
}

fn download_export(format: &str) {
    if let Some(window) = web_sys::window() {
        let url = format!(
            "/api/v1/perlengkapan/perlengkapan/mapping/export?format={}",
            format
        );
        let _ = window.open_with_url(&url);
    }
}

#[component]
pub fn MappingKodefikasiDashboard() -> impl IntoView {
    // leptos-fetch — same shape as before, but the value is now
    // de-duped + cached across mounts of this dashboard.
    let client: QueryClient = expect_context();
    let progress = client.local_resource(query_mapping_progress, || ());

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <div class="flex items-center justify-between mb-6">
                <h1 class="text-2xl font-bold text-gray-800">"Mapping Kodefikasi BMN"</h1>
                <div class="flex gap-2">
                    <button
                        class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 text-sm font-medium"
                        on:click=move |_| download_export("xlsx")
                    >
                        "📥 Export XLSX"
                    </button>
                    <button
                        class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 text-sm font-medium"
                        on:click=move |_| download_export("pdf")
                    >
                        "📥 Export PDF"
                    </button>
                </div>
            </div>

            <Suspense fallback=move || view! {
                <div class="flex justify-center items-center py-12">
                    <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
                </div>
            }>
                {move || {
                    progress.get().and_then(|data| data.map(|progress| {
                        view! {
                            <div>
                                // Metrics Cards
                                <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
                                    <MetricCard
                                        title="Kode Standar"
                                        value=progress.total_standard.to_string()
                                        icon="✅"
                                        color="green"
                                    />
                                    <MetricCard
                                        title="Kode Non-Standar"
                                        value=progress.total_non_standard.to_string()
                                        icon="⚠️"
                                        color="yellow"
                                    />
                                    <MetricCard
                                        title="Sudah Dipetakan"
                                        value=format!("{} ({:.1}%)", progress.total_mapped, progress.mapping_percentage)
                                        icon="📊"
                                        color="blue"
                                    />
                                </div>

                                // Non-Standard Codes Table
                                <div class="mt-8">
                                    <h2 class="text-xl font-bold mb-4 text-gray-800">"Daftar BMN Non-Standar"</h2>
                                    <MappingTable items=progress.non_standard_codes />
                                </div>
                            </div>
                        }
                    }))
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn MetricCard(
    title: &'static str,
    value: String,
    icon: &'static str,
    color: &'static str,
) -> impl IntoView {
    let bg_color = match color {
        "yellow" => "bg-yellow-50",
        "green" => "bg-green-50",
        "blue" => "bg-blue-50",
        "purple" => "bg-purple-50",
        _ => "bg-gray-50",
    };

    let text_color = match color {
        "yellow" => "text-yellow-600",
        "green" => "text-green-600",
        "blue" => "text-blue-600",
        "purple" => "text-purple-600",
        _ => "text-gray-600",
    };

    view! {
        <div class=format!("p-6 rounded-lg {}", bg_color)>
            <div class="flex items-center justify-between">
                <div>
                    <p class="text-sm text-gray-600 mb-1">{title}</p>
                    <p class=format!("text-2xl font-bold {}", text_color)>{value}</p>
                </div>
                <div class="text-4xl">{icon}</div>
            </div>
        </div>
    }
}

#[component]
fn MappingTable(items: Vec<NonStandardCodeWithStatus>) -> impl IntoView {
    view! {
        <div class="overflow-x-auto">
            <table class="min-w-full divide-y divide-gray-200">
                <thead class="bg-gray-50">
                    <tr>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            "Kode Lama"
                        </th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            "Nama Barang"
                        </th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            "Satker"
                        </th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            "Jumlah Aset"
                        </th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            "Status"
                        </th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            "Kode Standar Rujukan"
                        </th>
                    </tr>
                </thead>
                <tbody class="bg-white divide-y divide-gray-200">
                    <For
                        each=move || items.clone()
                        key=|item| format!("{}_{}", item.kode_lama, item.satker_id)
                        children=move |item| {
                            let status_badge = match item.status_mapping.as_deref() {
                                Some("VERIFIED") => view! {
                                    <span class="px-2 py-1 text-xs font-semibold rounded-full bg-green-100 text-green-800">
                                        "Standar"
                                    </span>
                                },
                                _ => view! {
                                    <span class="px-2 py-1 text-xs font-semibold rounded-full bg-yellow-100 text-yellow-800">
                                        "Non-Standar"
                                    </span>
                                },
                            };

                            view! {
                                <tr class="hover:bg-gray-50">
                                    <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                                        {item.kode_lama.clone()}
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                        {item.nama_lama.clone()}
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                        {item.satker_nama.clone()}
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                        {item.jumlah_aset}
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap">
                                        {status_badge}
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                        {item.kode_baru.clone().unwrap_or_else(|| "—".to_string())}
                                        {item.nama_baru.as_ref().map(|n| format!(" ({})", n)).unwrap_or_default()}
                                    </td>
                                </tr>
                            }
                        }
                    />
                </tbody>
            </table>
        </div>
    }
}
