//! Mapping Kodefikasi Dashboard Component
//!
//! Displays mapping progress statistics and non-standard codes

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingProgress {
    pub total_non_standard: i64,
    pub total_mapped: i64,
    pub pending_verification: i64,
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

async fn fetch_mapping_progress() -> Result<MappingProgress, String> {
    let response = gloo_net::http::Request::get("/api/pembinaan/perlengkapan/mapping/progress")
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

#[component]
pub fn MappingKodefikasiDashboard() -> impl IntoView {
    let progress = LocalResource::new(|| async move {
        match fetch_mapping_progress().await {
            Ok(data) => Some(data),
            Err(e) => {
                leptos::logging::error!("Failed to fetch mapping progress: {}", e);
                None
            }
        }
    });

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <h1 class="text-2xl font-bold mb-6 text-gray-800">"Mapping Kodefikasi Progress"</h1>

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
                                <div class="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
                                    <MetricCard
                                        title="Total Non-Standard Codes"
                                        value=progress.total_non_standard.to_string()
                                        icon="⚠️"
                                        color="yellow"
                                    />
                                    <MetricCard
                                        title="Mapped"
                                        value=progress.total_mapped.to_string()
                                        icon="✅"
                                        color="green"
                                    />
                                    <MetricCard
                                        title="Pending Verification"
                                        value=progress.pending_verification.to_string()
                                        icon="⏳"
                                        color="blue"
                                    />
                                    <MetricCard
                                        title="Progress"
                                        value=format!("{:.1}%", progress.mapping_percentage)
                                        icon="📊"
                                        color="purple"
                                    />
                                </div>

                                // Non-Standard Codes Table
                                <div class="mt-8">
                                    <h2 class="text-xl font-bold mb-4 text-gray-800">"Non-Standard Codes"</h2>
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
                            "Kode Baru"
                        </th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            "Actions"
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
                                        "Verified"
                                    </span>
                                },
                                Some("PROPOSED") => view! {
                                    <span class="px-2 py-1 text-xs font-semibold rounded-full bg-yellow-100 text-yellow-800">
                                        "Pending"
                                    </span>
                                },
                                Some("REJECTED") => view! {
                                    <span class="px-2 py-1 text-xs font-semibold rounded-full bg-red-100 text-red-800">
                                        "Rejected"
                                    </span>
                                },
                                _ => view! {
                                    <span class="px-2 py-1 text-xs font-semibold rounded-full bg-gray-100 text-gray-800">
                                        "Not Mapped"
                                    </span>
                                },
                            };

                            view! {
                                <tr>
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
                                        {item.kode_baru.clone().unwrap_or_else(|| "-".to_string())}
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap text-sm font-medium">
                                        {if item.status_mapping.is_none() {
                                            view! {
                                                <a
                                                    href=format!("/mapping/propose?kode={}&nama={}&satker={}",
                                                        item.kode_lama, item.nama_lama, item.satker_id)
                                                    class="text-blue-600 hover:text-blue-900"
                                                >
                                                    "Propose Mapping"
                                                </a>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <span class="text-gray-400">"—"</span>
                                            }.into_any()
                                        }}
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
