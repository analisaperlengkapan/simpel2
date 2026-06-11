//! # Kebutuhan BMN Analysis Page (Validator Pusat)
//!
//! This page allows Validator Pusat to:
//! - View kebutuhan BMN submissions with integrated SIMAN and MySIMKARI data
//! - Perform gap analysis
//! - Approve or reject submissions
//! - Generate analysis reports

use crate::components::layout::{LoadingState, PageLayout, SectionCard};
use leptos::prelude::*;
use leptos::task::spawn_local;
use lib_ui::components::dashboard::MetricCard;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{CHECK_CIRCLE, WARNING_CIRCLE};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// API Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatkerSubmission {
    pub id: Uuid,
    pub pengajuan_id: Uuid,
    pub satker_id: String,
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
    pub satker_id: String,
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
    pub action: String,
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
    let (_show_action_modal, _set_show_action_modal) = signal(false);
    let (_action_type, _set_action_type) = signal::<Option<String>>(None);
    let (_catatan, _set_catatan) = signal(String::new());
    let (approved_quantities, set_approved_quantities) =
        signal::<HashMap<Uuid, i32>>(HashMap::new());

    // Load submissions on mount
    Effect::new(move || {
        spawn_local(async move {
            set_loading.set(true);
            match fetch_submissions_for_analysis().await {
                Ok(subs) => set_submissions.set(subs),
                Err(e) => set_error.set(Some(e.user_message())),
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
                    let mut quantities = HashMap::new();
                    for item in &data.satker_detail.barang_items {
                        quantities.insert(item.id, item.jumlah);
                    }
                    set_approved_quantities.set(quantities);
                    set_selected_analysis.set(Some(data));
                }
                Err(e) => set_error.set(Some(e.user_message())),
            }
            set_loading.set(false);
        });
    };

    view! {
        <PageLayout
            title="Analisis Kebutuhan BMN"
            icon="fas fa-chart-bar"
            description="Validator Pusat — analisis dan keputusan pengajuan kebutuhan BMN"
        >
            // Error banner
            <Show when=move || error.get().is_some()>
                <div class="mb-4 flex items-center gap-2 rounded-xl border border-danger-500/30 bg-danger-500/[0.08] px-4 py-3 text-sm text-danger-300">
                    <AppIcon icon=WARNING_CIRCLE />
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            // Success banner
            <Show when=move || success_message.get().is_some()>
                <div class="mb-4 flex items-center gap-2 rounded-xl border border-success-500/30 bg-success-500/[0.08] px-4 py-3 text-sm text-success-300">
                    <AppIcon icon=CHECK_CIRCLE />
                    {move || success_message.get().unwrap_or_default()}
                </div>
            </Show>

            // Loading
            <Show when=move || loading.get()>
                <LoadingState message="Memuat data analisis...".to_string() />
            </Show>

            // Main grid: sidebar + content
            <div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
                // Sidebar: submission list
                <div class="lg:col-span-1">
                    <SectionCard title="Daftar Pengajuan">
                        {move || {
                            let subs = submissions.get();
                            if subs.is_empty() && !loading.get() {
                                view! {
                                    <div class="py-8 text-center text-sm text-slate-500">
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
                                                    class="cursor-pointer rounded-xl border border-white/[0.06] bg-white/[0.02] p-4 transition hover:border-white/10 hover:bg-white/[0.04]"
                                                    on:click=move |_| load_analysis(sub_id)
                                                >
                                                    <div class="text-sm font-semibold text-slate-200">
                                                        {sub.satker_nama.clone().unwrap_or_else(|| sub.satker_id.clone())}
                                                    </div>
                                                    <div class="mt-1 text-xs text-slate-400">
                                                        "Prioritas: " {sub.prioritas}
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            }
                        }}
                    </SectionCard>
                </div>

                // Content: analysis detail
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
                                <SectionCard title="Analisis">
                                    <div class="py-12 text-center text-sm text-slate-500">
                                        "Pilih pengajuan untuk melihat analisis"
                                    </div>
                                </SectionCard>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </PageLayout>
    }
}

#[component]
fn AnalysisContent(
    analysis: AnalysisData,
    approved_quantities: ReadSignal<HashMap<Uuid, i32>>,
    set_approved_quantities: WriteSignal<HashMap<Uuid, i32>>,
) -> impl IntoView {
    view! {
        <div class="space-y-5">
            <SectionCard title="Data SIMAN">
                <div class="grid grid-cols-2 gap-4 md:grid-cols-4">
                    <MetricCard
                        title="Total Aset"
                        value=analysis.siman_data.total_assets.to_string()
                        icon="fas fa-box"
                    />
                    <MetricCard
                        title="Kondisi Baik"
                        value=analysis.siman_data.assets_by_condition.get("BAIK").unwrap_or(&0).to_string()
                        icon="fas fa-check"
                    />
                </div>
            </SectionCard>

            <SectionCard title="Data MySIMKARI">
                <div class="grid grid-cols-2 gap-4 md:grid-cols-4">
                    <MetricCard
                        title="Total Pegawai"
                        value=analysis.mysimkari_data.total_pegawai.to_string()
                        icon="fas fa-users"
                    />
                    <MetricCard
                        title="Jaksa"
                        value=analysis.mysimkari_data.jaksa_count.to_string()
                        icon="fas fa-balance-scale"
                    />
                    <MetricCard
                        title="Non-Jaksa"
                        value=analysis.mysimkari_data.non_jaksa_count.to_string()
                        icon="fas fa-user-tie"
                    />
                </div>
            </SectionCard>

            <SectionCard title="Gap Analysis">
                <GapAnalysisTable data=analysis.gap_analysis.clone() />
            </SectionCard>

            <LaporanAnalisisSection satker_id=analysis.satker_detail.id />
        </div>
    }
}

/// Section dengan tombol Preview (toggle iframe inline PDF) dan
/// Download PDF untuk Laporan Hasil Analisis Kebutuhan BMN.
/// URL endpoint backend: lihat handlers `preview_laporan_analisis` &
/// `download_laporan_analisis`.
#[component]
fn LaporanAnalisisSection(satker_id: Uuid) -> impl IntoView {
    let (show_preview, set_show_preview) = signal(false);
    let preview_url = format!(
        "/api/v1/perlengkapan/kebutuhan-bmn/satker/{}/laporan/preview?format=pdf",
        satker_id
    );
    let download_url = format!(
        "/api/v1/perlengkapan/kebutuhan-bmn/satker/{}/laporan/download?format=pdf",
        satker_id
    );
    let download_url_for_click = download_url.clone();
    let preview_for_iframe = preview_url.clone();

    view! {
        <SectionCard title="Laporan Hasil Analisis Kebutuhan BMN">
            <div class="flex flex-wrap items-center gap-3">
                <button
                    class="inline-flex items-center gap-2 rounded-lg border border-primary-500/30 bg-primary-500/10 px-4 py-2 text-sm font-medium text-primary-300 transition hover:bg-primary-500/20"
                    on:click=move |_| set_show_preview.update(|v| *v = !*v)
                >
                    <span class="text-xs">
                        <i class={move || if show_preview.get() { "fas fa-eye-slash" } else { "fas fa-eye" }}></i>
                    </span>
                    {move || if show_preview.get() { "Tutup Preview" } else { "Preview Laporan (PDF)" }}
                </button>
                <button
                    class="inline-flex items-center gap-2 rounded-lg border border-success-500/30 bg-success-500/10 px-4 py-2 text-sm font-medium text-success-300 transition hover:bg-success-500/20"
                    on:click=move |_| {
                        #[cfg(target_arch = "wasm32")]
                        if let Some(win) = web_sys::window() {
                            let _ = win.open_with_url_and_target(&download_url_for_click, "_blank");
                        }
                    }
                >
                    <span class="text-xs"><i class="fas fa-file-pdf"></i></span>
                    "Download PDF"
                </button>
            </div>
            <Show when=move || show_preview.get()>
                <div class="mt-4 overflow-hidden rounded-lg border border-white/[0.06]">
                    <iframe
                        src=preview_for_iframe.clone()
                        class="h-[70vh] w-full"
                        title="Preview Laporan Hasil Analisis Kebutuhan BMN"
                    />
                </div>
            </Show>
            <p class="mt-3 text-xs text-slate-500">
                "Laporan komprehensif: identitas satker, usulan vs eksisting SIMAN, ringkasan kelayakan, status integrasi, rekap pegawai (jika tersedia). DOCX export tersedia setelah template final disetujui Biro Hukum."
            </p>
        </SectionCard>
    }
}

#[component]
fn GapAnalysisTable(data: Vec<GapAnalysisItem>) -> impl IntoView {
    view! {
        <div class="overflow-hidden rounded-xl border border-white/[0.06]">
            <div class="overflow-x-auto">
                <table class="min-w-full divide-y divide-white/[0.04]">
                    <thead class="bg-white/[0.02]">
                        <tr>
                            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">"Kode"</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">"Nama Barang"</th>
                            <th class="px-4 py-3 text-right text-xs font-semibold uppercase tracking-wide text-slate-400">"Standar"</th>
                            <th class="px-4 py-3 text-right text-xs font-semibold uppercase tracking-wide text-slate-400">"Existing"</th>
                            <th class="px-4 py-3 text-right text-xs font-semibold uppercase tracking-wide text-slate-400">"Diminta"</th>
                            <th class="px-4 py-3 text-right text-xs font-semibold uppercase tracking-wide text-slate-400">"Gap"</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">"Rekomendasi"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {data.into_iter().enumerate().map(|(idx, item)| {
                            let bg = if idx % 2 == 0 { "bg-transparent" } else { "bg-white/[0.015]" };
                            let gap_color = if item.gap > 0 { "text-danger-400" } else if item.gap < 0 { "text-success-400" } else { "text-slate-300" };
                            view! {
                                <tr class=format!("border-b border-white/[0.04] {}", bg)>
                                    <td class="px-4 py-3 font-mono text-xs text-slate-400">{item.kode_barang}</td>
                                    <td class="px-4 py-3 text-sm text-slate-200">{item.nama_barang}</td>
                                    <td class="px-4 py-3 text-right text-sm text-slate-300">{item.standard_quantity}</td>
                                    <td class="px-4 py-3 text-right text-sm text-info-400">{item.existing_quantity}</td>
                                    <td class="px-4 py-3 text-right text-sm text-slate-300">{item.requested_quantity}</td>
                                    <td class=format!("px-4 py-3 text-right text-sm font-bold {}", gap_color)>{item.gap}</td>
                                    <td class="px-4 py-3 text-sm text-slate-400">{item.recommendation}</td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

// ============================================================================
// API Functions
// ============================================================================

async fn fetch_submissions_for_analysis() -> Result<Vec<SatkerSubmission>, crate::api::AppError> {
    let response =
        gloo_net::http::Request::get("/api/v1/perlengkapan/kebutuhan-bmn/satker?status_kode=2003")
            .send()
            .await
            .map_err(|e| crate::api::AppError::Unknown(format!("Network error: {}", e)))?;

    if !response.ok() {
        return Err(crate::api::AppError::Unknown(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let api_response: ApiResponse<Vec<SatkerSubmission>> = response
        .json()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Parse error: {}", e)))?;

    Ok(api_response.data.unwrap_or_default())
}

async fn fetch_analysis_data(satker_id: Uuid) -> Result<AnalysisData, crate::api::AppError> {
    let url = format!(
        "/api/v1/perlengkapan/kebutuhan-bmn/satker/{}/analisis",
        satker_id
    );
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

    let api_response: ApiResponse<AnalysisData> = response
        .json()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Parse error: {}", e)))?;

    api_response
        .data
        .ok_or_else(|| crate::api::AppError::Unknown("Data analisis tidak ditemukan".to_string()))
}
