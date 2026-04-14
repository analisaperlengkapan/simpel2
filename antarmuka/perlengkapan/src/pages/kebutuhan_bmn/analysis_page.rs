//! # Kebutuhan BMN Analysis Page (Validator Pusat)
//!
//! This page allows Validator Pusat to:
//! - View kebutuhan BMN submissions with integrated SIMAN and MySIMKARI data
//! - Perform gap analysis
//! - Approve or reject submissions
//! - Generate analysis reports
//!
//! Requirements: REQ-K010, REQ-K011, REQ-K012, REQ-K013, REQ-K014

use leptos::prelude::*;
use leptos::task::spawn_local;
use lib_ui::components::dashboard::MetricCard;
use lib_ui::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

// ============================================================================
// API Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatkerSubmission {
    pub id: Uuid,
    pub pengajuan_id: Uuid,
    pub ms_satker_id: String,
    pub satker_nama: Option<String>,
    pub status_kode: i32,
    pub status_nama: Option<String>,
    pub prioritas: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisData {
    pub satker_detail: SatkerDetail,
    pub siman_data: SimanData,
    pub mysimkari_data: MySIMKARIData,
    pub gap_analysis: Vec<GapAnalysisItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatkerDetail {
    pub id: Uuid,
    pub ms_satker_id: String,
    pub satker_nama: Option<String>,
    pub status_kode: i32,
    pub barang_items: Vec<BarangItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarangItem {
    pub id: Uuid,
    pub nama: String,
    pub kode_barang: Option<String>,
    pub jumlah: i32,
    pub jml_setuju: i32,
    pub prioritas: i32,
    pub skor: f64,
    pub justifikasi: Option<String>,
    pub existing_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimanData {
    pub total_assets: i32,
    pub assets_by_condition: HashMap<String, i32>,
    pub assets_by_kode: HashMap<String, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MySIMKARIData {
    pub total_pegawai: i32,
    pub eselon_count: HashMap<String, i32>,
    pub golongan_count: HashMap<String, i32>,
    pub jaksa_count: i32,
    pub non_jaksa_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapAnalysisItem {
    pub kode_barang: String,
    pub nama_barang: String,
    pub standard_quantity: i32,
    pub existing_quantity: i32,
    pub requested_quantity: i32,
    pub gap: i32,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorPusatActionRequest {
    pub action: String, // "approve" or "reject"
    pub catatan: String,
    pub approved_items: Option<Vec<ApprovedItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovedItem {
    pub barang_id: Uuid,
    pub jml_setuju: i32,
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
pub fn AnalysisPage() -> impl IntoView {
    let (submissions, set_submissions) = signal::<Vec<SatkerSubmission>>(Vec::new());
    let (selected_analysis, set_selected_analysis) = signal::<Option<AnalysisData>>(None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let (success_message, set_success_message) = signal::<Option<String>>(None);
    let (show_action_modal, set_show_action_modal) = signal(false);
    let (action_type, set_action_type) = signal::<Option<String>>(None);
    let (catatan, set_catatan) = signal(String::new());
    let (approved_quantities, set_approved_quantities) = signal::<HashMap<Uuid, i32>>(HashMap::new());

    // Load submissions on mount
    Effect::new(move || {
        spawn_local(async move {
            set_loading.set(true);
            match fetch_submissions_for_analysis().await {
                Ok(subs) => set_submissions.set(subs),
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    // Load analysis data
    let load_analysis = move |submission_id: Uuid| {
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);
            match fetch_analysis_data(submission_id).await {
                Ok(data) => {
                    // Initialize approved quantities with requested amounts
                    let mut quantities = HashMap::new();
                    for item in &data.satker_detail.barang_items {
                        quantities.insert(item.id, item.jumlah);
                    }
                    set_approved_quantities.set(quantities);
                    set_selected_analysis.set(Some(data));
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="container mx-auto px-4 py-8">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-6">
                "Analisis Kebutuhan BMN - Validator Pusat"
            </h1>

            {move || error.get().map(|e| view! {
                <Alert message=e variant=AlertVariant::Error />
            })}

            {move || success_message.get().map(|msg| view! {
                <Alert message=msg variant=AlertVariant::Success />
            })}

            {move || loading.get().then(|| view! {
                <div class="flex justify-center items-center py-12">
                    <Spinner size="lg" />
                    <span class="ml-3 text-gray-600">"Memuat data..."</span>
                </div>
            })}

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                <div class="lg:col-span-1">
                    <Card title="Daftar Pengajuan">
                        <div class="space-y-2">
                            {move || {
                                let subs = submissions.get();
                                if subs.is_empty() && !loading.get() {
                                    view! {
                                        <div class="text-center py-8 text-gray-500">
                                            "Tidak ada pengajuan yang perlu dianalisis"
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="space-y-2">
                                            {subs.into_iter().map(|sub| {
                                                let sub_id = sub.id;
                                                view! {
                                                    <div
                                                        class="p-4 border rounded-lg cursor-pointer hover:bg-gray-50"
                                                        on:click=move |_| load_analysis(sub_id)
                                                    >
                                                        <div class="font-semibold text-gray-900">
                                                            {sub.satker_nama.clone().unwrap_or_else(|| sub.ms_satker_id.clone())}
                                                        </div>
                                                        <div class="text-sm text-gray-600 mt-1">
                                                            "Prioritas: " {sub.prioritas}
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>
                    </Card>
                </div>

                <div class="lg:col-span-2">
                    {move || {
                        if let Some(analysis) = selected_analysis.get() {
                            view! {
                                <AnalysisContent
                                    analysis=analysis
                                    approved_quantities=approved_quantities
                                    set_approved_quantities=set_approved_quantities
                                />
                            }.into_any()
                        } else {
                            view! {
                                <Card>
                                    <div class="text-center py-12 text-gray-500">
                                        "Pilih pengajuan untuk melihat analisis"
                                    </div>
                                </Card>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn AnalysisContent(
    analysis: AnalysisData,
    approved_quantities: ReadSignal<HashMap<Uuid, i32>>,
    set_approved_quantities: WriteSignal<HashMap<Uuid, i32>>,
) -> impl IntoView {
    view! {
        <div class="space-y-6">
            <Card title="Data SIMAN (REQ-K010)">
                <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                    <MetricCard
                        title="Total Aset"
                        value=analysis.siman_data.total_assets.to_string()
                        icon="📦"
                    />
                    <MetricCard
                        title="Kondisi Baik"
                        value=analysis.siman_data.assets_by_condition.get("BAIK").unwrap_or(&0).to_string()
                        icon="✅"
                    />
                </div>
            </Card>

            <Card title="Data MySIMKARI (REQ-K011)">
                <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                    <MetricCard
                        title="Total Pegawai"
                        value=analysis.mysimkari_data.total_pegawai.to_string()
                        icon="👥"
                    />
                    <MetricCard
                        title="Jaksa"
                        value=analysis.mysimkari_data.jaksa_count.to_string()
                        icon="⚖️"
                    />
                    <MetricCard
                        title="Non-Jaksa"
                        value=analysis.mysimkari_data.non_jaksa_count.to_string()
                        icon="👔"
                    />
                </div>
            </Card>

            <Card title="Gap Analysis">
                <GapAnalysisTable data=analysis.gap_analysis.clone() />
            </Card>
        </div>
    }
}

#[component]
fn GapAnalysisTable(data: Vec<GapAnalysisItem>) -> impl IntoView {
    view! {
        <div class="overflow-x-auto">
            <table class="min-w-full divide-y divide-gray-200">
                <thead class="bg-gray-50">
                    <tr>
                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"Kode"</th>
                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"Nama Barang"</th>
                        <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase">"Standar"</th>
                        <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase">"Existing"</th>
                        <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase">"Diminta"</th>
                        <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase">"Gap"</th>
                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"Rekomendasi"</th>
                    </tr>
                </thead>
                <tbody class="bg-white divide-y divide-gray-200">
                    {data.into_iter().map(|item| view! {
                        <tr>
                            <td class="px-4 py-3 text-sm">{item.kode_barang}</td>
                            <td class="px-4 py-3 text-sm">{item.nama_barang}</td>
                            <td class="px-4 py-3 text-sm text-right">{item.standard_quantity}</td>
                            <td class="px-4 py-3 text-sm text-right">{item.existing_quantity}</td>
                            <td class="px-4 py-3 text-sm text-right">{item.requested_quantity}</td>
                            <td class="px-4 py-3 text-sm text-right font-semibold">{item.gap}</td>
                            <td class="px-4 py-3 text-sm">{item.recommendation}</td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
}

// ============================================================================
// API Functions
// ============================================================================

async fn fetch_submissions_for_analysis() -> Result<Vec<SatkerSubmission>, String> {
    let response = gloo_net::http::Request::get("/api/v1/kebutuhan-bmn/satker?status_kode=2003")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let api_response: ApiResponse<Vec<SatkerSubmission>> = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(api_response.data.unwrap_or_default())
}

async fn fetch_analysis_data(satker_id: Uuid) -> Result<AnalysisData, String> {
    let url = format!("/api/v1/kebutuhan-bmn/satker/{}/analisis", satker_id);
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let api_response: ApiResponse<AnalysisData> = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    api_response
        .data
        .ok_or_else(|| "Analysis data not found".to_string())
}
