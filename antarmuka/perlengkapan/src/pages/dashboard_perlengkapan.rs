//! Perlengkapan Dashboard UI
//!
//! Displays comprehensive metrics for the Perlengkapan microfrontend with drill-down support

use crate::routes;
use leptos::prelude::*;
use leptos_fetch::QueryClient;
use leptos_router::hooks::use_query_map;
use lib_ui::components::dashboard::{BarChart, GapAnalysisTable, MetricCard, PieChart};
use lib_ui::components::forms::Select;
use lib_ui::components::icon::AppIcon;
use lib_ui::components::navigation::Breadcrumb;
use lib_ui::core::types::{BreadcrumbItem, ChartDataPoint, GapAnalysisRow};
use phosphor_leptos::{ARROW_CLOCKWISE, WARNING};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// TYPES
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum DrillDownLevel {
    National,
    Wilayah(String), // wilayah_code
    Satker(String),  // satker_id
}

impl DrillDownLevel {
    pub fn from_query_params(wilayah_code: Option<String>, satker_id: Option<String>) -> Self {
        if let Some(satker) = satker_id {
            DrillDownLevel::Satker(satker)
        } else if let Some(wilayah) = wilayah_code {
            DrillDownLevel::Wilayah(wilayah)
        } else {
            DrillDownLevel::National
        }
    }

    pub fn to_query_string(&self) -> String {
        match self {
            DrillDownLevel::National => String::new(),
            DrillDownLevel::Wilayah(code) => format!("wilayah_code={}", code),
            DrillDownLevel::Satker(id) => format!("satker_id={}", id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerlengkapanDashboardMetrics {
    pub kebutuhan_metrics: KebutuhanMetrics,
    pub gap_analysis: Vec<GapAnalysisResult>,
    pub pakaian_dinas_metrics: PakaianDinasMetrics,
    pub workflow_metrics: WorkflowMetrics,
    pub asset_utilization: AssetUtilization,
    /// Rekap status Pemakaian BMN (Fase 2.7). `default` agar tetap kompatibel
    /// dgn respons backend lama yg belum punya field ini.
    #[serde(default)]
    pub pemakaian_metrics: ModuleStatusMetrics,
    #[serde(default)]
    pub penghapusan_metrics: ModuleStatusMetrics,
}

/// Rekap jumlah per status untuk satu modul (Fase 2.7).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModuleStatusMetrics {
    #[serde(default)]
    pub total_by_status: HashMap<String, i64>,
    #[serde(default)]
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WilayahOption {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatkerOption {
    pub id: String,
    pub name: String,
    pub wilayah_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KebutuhanMetrics {
    pub total_by_status: HashMap<String, i64>,
    pub total_by_satker: Vec<SatkerCount>,
    pub total_by_tahun: HashMap<i32, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatkerCount {
    pub satker_nama: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapAnalysisResult {
    pub kode_barang: String,
    pub nama_barang: String,
    pub standard_quantity: i32,
    pub existing_good_quantity: i32,
    pub gap: i32,
    pub satker_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PakaianDinasMetrics {
    pub total_by_jenis: HashMap<String, i64>,
    pub total_by_ukuran: HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetrics {
    pub average_processing_time_hours: f64,
    pub bottlenecks: Vec<BottleneckInfo>,
    pub sla_breaches_today: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckInfo {
    pub state: String,
    pub average_time_hours: f64,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetUtilization {
    pub total_assets: i64,
    pub assets_in_good_condition: i64,
    pub utilization_percentage: f64,
}

// ============================================================================
// API CLIENT
// ============================================================================

async fn fetch_perlengkapan_dashboard(
    tahun_anggaran: i32,
    wilayah_code: Option<String>,
    satker_id: Option<String>,
) -> Result<PerlengkapanDashboardMetrics, crate::api::AppError> {
    let mut url = format!(
        "/api/v1/perlengkapan/dashboard/perlengkapan?tahun_anggaran={}",
        tahun_anggaran
    );

    if let Some(satker) = satker_id {
        url.push_str(&format!("&satker_id={}", satker));
    } else if let Some(wilayah) = wilayah_code {
        url.push_str(&format!("&wilayah_code={}", wilayah));
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

    response
        .json::<PerlengkapanDashboardMetrics>()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("JSON parse error: {}", e)))
}

/// leptos-fetch query wrappers — keyed cache + dedup.
///
/// Each query folds a `refresh_trigger: i32` into the cache key so the
/// Refresh button can force a real re-fetch by bumping the trigger.
/// `.refetch()` on a leptos-fetch resource re-evaluates the keyer and,
/// for an unchanged key, serves the stale cached value — so we instead
/// rely on key change to allocate a new cache slot.
async fn query_wilayah_options(_trigger: i32) -> Result<Vec<WilayahOption>, crate::api::AppError> {
    fetch_wilayah_options().await
}

async fn query_satker_options(
    key: (Option<String>, i32),
) -> Result<Vec<SatkerOption>, crate::api::AppError> {
    let (wilayah, _trigger) = key;
    fetch_satker_options(wilayah).await
}

/// Dashboard-metrics query — keyed by `(tahun, wilayah, satker, refresh_trigger)`.
/// Each drill-down combination caches independently so navigating
/// back to a previous level re-uses the prior payload, and the
/// Refresh button bumps the trigger to force a real network round-trip.
async fn query_perlengkapan_dashboard(
    key: (i32, Option<String>, Option<String>, i32),
) -> Result<PerlengkapanDashboardMetrics, crate::api::AppError> {
    let (tahun_anggaran, wilayah_code, satker_id, _trigger) = key;
    fetch_perlengkapan_dashboard(tahun_anggaran, wilayah_code, satker_id).await
}

async fn fetch_wilayah_options() -> Result<Vec<WilayahOption>, crate::api::AppError> {
    let response = gloo_net::http::Request::get("/api/v1/perlengkapan/wilayah")
        .send()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Network error: {}", e)))?;

    if !response.ok() {
        return Err(crate::api::AppError::Unknown(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    response
        .json::<Vec<WilayahOption>>()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("JSON parse error: {}", e)))
}

async fn fetch_satker_options(
    wilayah_code: Option<String>,
) -> Result<Vec<SatkerOption>, crate::api::AppError> {
    let url = if let Some(code) = wilayah_code {
        format!("/api/v1/perlengkapan/satker?wilayah_code={}", code)
    } else {
        "/api/v1/perlengkapan/satker".to_string()
    };

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

    response
        .json::<Vec<SatkerOption>>()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("JSON parse error: {}", e)))
}

// ============================================================================
// COMPONENTS
// ============================================================================

#[component]
fn LoadingState() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center min-h-screen">
            <div class="text-center">
                <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-emerald-500"></div>
                <p class="mt-4 text-gray-600 dark:text-gray-400">"Memuat data dashboard..."</p>
            </div>
        </div>
    }
}

#[component]
fn ErrorState(message: String) -> impl IntoView {
    view! {
        <div class="flex items-center justify-center min-h-screen">
            <div class="text-center max-w-md">
                <div class="text-red-500 text-5xl mb-4">
                    <AppIcon icon=WARNING />
                </div>
                <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-2">
                    "Gagal Memuat Dashboard"
                </h2>
                <p class="text-gray-600 dark:text-gray-400">{message}</p>
            </div>
        </div>
    }
}

// ============================================================================
// DRILL-DOWN SELECTORS COMPONENT
// ============================================================================

#[component]
fn DrillDownSelectors(
    wilayah_options_resource: LocalResource<Result<Vec<WilayahOption>, crate::api::AppError>>,
    satker_options_resource: LocalResource<Result<Vec<SatkerOption>, crate::api::AppError>>,
    current_level: DrillDownLevel,
) -> impl IntoView {
    let (selected_wilayah, set_selected_wilayah) = signal::<Option<String>>(None);
    let (selected_satker, set_selected_satker) = signal::<Option<String>>(None);

    let current_level_for_effect = current_level.clone();
    let current_level_for_satker_visibility = current_level.clone();
    let current_level_for_indicator = current_level.clone();

    // Initialize from current level
    Effect::new(move |_| match &current_level_for_effect {
        DrillDownLevel::Wilayah(code) => set_selected_wilayah.set(Some(code.clone())),
        DrillDownLevel::Satker(id) => set_selected_satker.set(Some(id.clone())),
        DrillDownLevel::National => {
            set_selected_wilayah.set(None);
            set_selected_satker.set(None);
        }
    });

    let handle_wilayah_change = move |value: String| {
        if value.is_empty() {
            // Navigate to national level
            window().location().set_href(routes::path::DASHBOARD).ok();
        } else {
            // Navigate to wilayah level
            window()
                .location()
                .set_href(&routes::url::dashboard_perlengkapan_with_query(&format!(
                    "wilayah_code={}",
                    value
                )))
                .ok();
        }
    };

    let handle_satker_change = move |value: String| {
        if value.is_empty() {
            // Navigate back to wilayah level
            if let Some(wilayah) = selected_wilayah.get() {
                window()
                    .location()
                    .set_href(&routes::url::dashboard_perlengkapan_with_query(&format!(
                        "wilayah_code={}",
                        wilayah
                    )))
                    .ok();
            } else {
                window().location().set_href(routes::path::DASHBOARD).ok();
            }
        } else {
            // Navigate to satker level
            window()
                .location()
                .set_href(&routes::url::dashboard_perlengkapan_with_query(&format!(
                    "satker_id={}",
                    value
                )))
                .ok();
        }
    };

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-4 border border-gray-200 dark:border-gray-700">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                // Wilayah selector
                <div>
                    <Suspense fallback=move || {
                        view! { <p>"Loading wilayah..."</p> }
                    }>
                        {move || {
                            wilayah_options_resource
                                .get()
                                .map(|result| {
                                    match result {
                                        Ok(options) => {
                                            view! {
                                                <Select
                                                    label="Filter by Wilayah"
                                                    value=selected_wilayah.get().unwrap_or_default()
                                                    on_change=Box::new(handle_wilayah_change)
                                                >
                                                    <option value="">"-- Semua Wilayah (Nasional) --"</option>
                                                    {options
                                                        .into_iter()
                                                        .map(|opt| {
                                                            view! { <option value=opt.code.clone()>{opt.name}</option> }
                                                        })
                                                        .collect_view()}
                                                </Select>
                                            }
                                                .into_any()
                                        }
                                        Err(e) => {
                                            view! {
                                                <p class="text-sm text-red-600">
                                                    "Error loading wilayah: " {e.to_string()}
                                                </p>
                                            }
                                                .into_any()
                                        }
                                    }
                                })
                        }}
                    </Suspense>
                </div>

                // Satker selector (only show if wilayah is selected or at satker level)
                {move || {
                    if selected_wilayah.get().is_some()
                        || matches!(current_level_for_satker_visibility, DrillDownLevel::Satker(_))
                    {
                        view! {
                            <div>
                                <Suspense fallback=move || {
                                    view! { <p>"Loading satker..."</p> }
                                }>
                                    {move || {
                                        satker_options_resource
                                            .get()
                                            .map(|result| {
                                                match result {
                                                    Ok(options) => {
                                                        view! {
                                                            <Select
                                                                label="Filter by Satker"
                                                                value=selected_satker.get().unwrap_or_default()
                                                                on_change=Box::new(handle_satker_change)
                                                            >
                                                                <option value="">"-- Semua Satker di Wilayah --"</option>
                                                                {options
                                                                    .into_iter()
                                                                    .map(|opt| {
                                                                        view! { <option value=opt.id.clone()>{opt.name}</option> }
                                                                    })
                                                                    .collect_view()}
                                                            </Select>
                                                        }
                                                            .into_any()
                                                    }
                                                    Err(e) => {
                                                        view! {
                                                            <p class="text-sm text-red-600">
                                                                "Error loading satker: " {e.to_string()}
                                                            </p>
                                                        }
                                                            .into_any()
                                                    }
                                                }
                                            })
                                    }}
                                </Suspense>
                            </div>
                        }
                            .into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
            </div>

            // Current level indicator
            <div class="mt-3 pt-3 border-t border-gray-200 dark:border-gray-700">
                <p class="text-sm text-gray-600 dark:text-gray-400">
                    "Level saat ini: "
                    <span class="font-semibold text-gray-900 dark:text-gray-100">
                        {move || match current_level_for_indicator {
                            DrillDownLevel::National => "Nasional (Seluruh Indonesia)".to_string(),
                            DrillDownLevel::Wilayah(ref code) => format!("Wilayah {}", code),
                            DrillDownLevel::Satker(ref id) => format!("Satker {}", id),
                        }}
                    </span>
                </p>
            </div>
        </div>
    }
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

// ============================================================================
// MAIN COMPONENT
// ============================================================================

#[component]
pub fn DashboardPerlengkapan() -> impl IntoView {
    let tahun_anggaran = 2025;

    // Get query parameters from URL
    let query_map = use_query_map();
    let wilayah_code = move || query_map.get().get("wilayah_code");
    let satker_id = move || query_map.get().get("satker_id");

    // Drill-down level state
    let drill_down_level = move || DrillDownLevel::from_query_params(wilayah_code(), satker_id());

    // Refresh trigger folded into the leptos-fetch cache keys below so
    // the Refresh button can force a real re-fetch by bumping it.
    // Calling `.refetch()` directly on a leptos-fetch resource with an
    // unchanged key returns the stale cached value instead of re-issuing
    // the network request.
    let refresh_trigger = RwSignal::new(0i32);

    // Fetch dashboard + wilayah + satker options through the leptos-fetch
    // cache — each drill-down combination caches independently, the same
    // wilayah list is shared with any other component that asks for it,
    // and per-wilayah satker lists each get their own cache slot keyed
    // by `(Option<String>, refresh_trigger)`.
    let client: QueryClient = expect_context();
    let dashboard_resource = client.local_resource(query_perlengkapan_dashboard, move || {
        (
            tahun_anggaran,
            wilayah_code(),
            satker_id(),
            refresh_trigger.get(),
        )
    });
    let wilayah_options_resource =
        client.local_resource(query_wilayah_options, move || refresh_trigger.get());
    let satker_options_resource = client.local_resource(query_satker_options, move || {
        (wilayah_code(), refresh_trigger.get())
    });

    // Build breadcrumb items based on drill-down level
    let breadcrumb_items = move || {
        let mut items = vec![BreadcrumbItem {
            label: "Nasional".to_string(),
            path: Some("/perlengkapan/perlengkapan".to_string()),
            icon: Some("🇮🇩".to_string()),
        }];

        match drill_down_level() {
            DrillDownLevel::Wilayah(code) => {
                items.push(BreadcrumbItem {
                    label: format!("Wilayah {}", code),
                    path: None,
                    icon: Some("📍".to_string()),
                });
            }
            DrillDownLevel::Satker(id) => {
                if let Some(wil_code) = wilayah_code() {
                    items.push(BreadcrumbItem {
                        label: format!("Wilayah {}", wil_code),
                        path: Some(format!(
                            "/perlengkapan/perlengkapan?wilayah_code={}",
                            wil_code
                        )),
                        icon: Some("📍".to_string()),
                    });
                }
                items.push(BreadcrumbItem {
                    label: format!("Satker {}", id),
                    path: None,
                    icon: Some("🏢".to_string()),
                });
            }
            DrillDownLevel::National => {}
        }

        items
    };

    view! {
        <div class="container mx-auto px-4 py-6 space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">
                        "Dashboard Perlengkapan"
                    </h1>
                    <p class="text-gray-600 dark:text-gray-400 mt-1">
                        "Tahun Anggaran " {tahun_anggaran}
                    </p>
                </div>
                <button
                    on:click=move |_| {
                        refresh_trigger.update(|v| *v += 1);
                    }
                    class="px-4 py-2 bg-emerald-500 text-white rounded-lg hover:bg-emerald-600 transition-colors"
                >
                    <span class="mr-2">
                        <AppIcon icon=ARROW_CLOCKWISE />
                    </span>
                    "Refresh"
                </button>
            </div>

            // Breadcrumb navigation
            <Breadcrumb items=breadcrumb_items() />

            // Drill-down selectors
            <DrillDownSelectors
                wilayah_options_resource=wilayah_options_resource
                satker_options_resource=satker_options_resource
                current_level=drill_down_level()
            />

            <Suspense fallback=LoadingState>
                {move || {
                    dashboard_resource
                        .get()
                        .map(|result| {
                            match result {
                                Ok(metrics) => {
                                    view! { <DashboardContent metrics=metrics /> }.into_any()
                                }
                                Err(e) => view! { <ErrorState message=e.to_string() /> }.into_any(),
                            }
                        })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn DashboardContent(metrics: PerlengkapanDashboardMetrics) -> impl IntoView {
    let total_kebutuhan: i64 = metrics.kebutuhan_metrics.total_by_status.values().sum();
    let approved = metrics
        .kebutuhan_metrics
        .total_by_status
        .get("APPROVED")
        .copied()
        .unwrap_or(0);
    let pending = metrics
        .kebutuhan_metrics
        .total_by_status
        .get("SUBMITTED")
        .copied()
        .unwrap_or(0)
        + metrics
            .kebutuhan_metrics
            .total_by_status
            .get("REVIEWED_WILAYAH")
            .copied()
            .unwrap_or(0);
    let rejected = metrics
        .kebutuhan_metrics
        .total_by_status
        .get("REJECTED")
        .copied()
        .unwrap_or(0);

    let gap_rows: Vec<GapAnalysisRow> = metrics
        .gap_analysis
        .iter()
        .take(10)
        .map(|g| GapAnalysisRow {
            kode_barang: g.kode_barang.clone(),
            nama_barang: g.nama_barang.clone(),
            satker_name: g.satker_name.clone().unwrap_or_else(|| "N/A".to_string()),
            standard_quantity: g.standard_quantity,
            existing_quantity: g.existing_good_quantity,
            gap: g.gap,
        })
        .collect();

    let status_chart_data: Vec<ChartDataPoint> = vec![
        ChartDataPoint {
            label: "Disetujui".to_string(),
            value: approved as f64,
            color: Some("bg-green-500".to_string()),
        },
        ChartDataPoint {
            label: "Pending".to_string(),
            value: pending as f64,
            color: Some("bg-yellow-500".to_string()),
        },
        ChartDataPoint {
            label: "Ditolak".to_string(),
            value: rejected as f64,
            color: Some("bg-red-500".to_string()),
        },
    ];

    let satker_chart_data: Vec<ChartDataPoint> = metrics
        .kebutuhan_metrics
        .total_by_satker
        .iter()
        .take(10)
        .map(|s| ChartDataPoint {
            label: s.satker_nama.clone(),
            value: s.count as f64,
            color: Some("bg-emerald-500".to_string()),
        })
        .collect();

    // Fase 2.7: rekap status Pemakaian & Penghapusan BMN (urut stabil per status).
    let to_points = |m: &HashMap<String, i64>| -> Vec<ChartDataPoint> {
        let mut entries: Vec<(&String, &i64)> = m.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        entries
            .into_iter()
            .map(|(k, v)| ChartDataPoint {
                label: k.clone(),
                value: *v as f64,
                color: Some("bg-indigo-500".to_string()),
            })
            .collect()
    };
    let pemakaian_chart = to_points(&metrics.pemakaian_metrics.total_by_status);
    let penghapusan_chart = to_points(&metrics.penghapusan_metrics.total_by_status);
    let total_pemakaian = metrics.pemakaian_metrics.total;
    let total_penghapusan = metrics.penghapusan_metrics.total;

    view! {
        <div class="space-y-6">
            <section>
                <h2 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
                    "Kebutuhan BMN Overview"
                </h2>
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                    <MetricCard
                        title="Total Kebutuhan"
                        value=total_kebutuhan.to_string()
                        icon="📋"
                        subtitle="Total pengajuan"
                    />
                    <MetricCard
                        title="Disetujui"
                        value=approved.to_string()
                        icon="✅"
                        subtitle="Kebutuhan disetujui"
                    />
                    <MetricCard
                        title="Pending"
                        value=pending.to_string()
                        icon="⏳"
                        subtitle="Menunggu review"
                    />
                    <MetricCard
                        title="Ditolak"
                        value=rejected.to_string()
                        icon="❌"
                        subtitle="Kebutuhan ditolak"
                    />
                </div>

                <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 mt-4">
                    <PieChart
                        title="Status Kebutuhan BMN"
                        data=status_chart_data
                        show_legend=true
                        size=200
                    />
                    <BarChart
                        title="Top 10 Satker by Kebutuhan"
                        data=satker_chart_data
                        show_values=true
                        height=300
                    />
                </div>
            </section>

            <section>
                <h2 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
                    "Gap Analysis - Top 10 Kekurangan BMN"
                </h2>
                <GapAnalysisTable data=gap_rows show_satker=true sortable=true />
            </section>

            <section>
                <h2 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
                    "Pemakaian & Penghapusan BMN"
                </h2>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                    <MetricCard
                        title="Total Izin Pemakaian BMN"
                        value=total_pemakaian.to_string()
                        icon="🚗"
                        subtitle="Seluruh izin pemakaian (semua status)"
                    />
                    <MetricCard
                        title="Total Usulan SK Penghapusan"
                        value=total_penghapusan.to_string()
                        icon="🗑️"
                        subtitle="Seluruh usulan penghapusan (semua status)"
                    />
                </div>
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
                    <PieChart
                        title="Status Pemakaian BMN"
                        data=pemakaian_chart
                        show_legend=true
                        size=200
                    />
                    <PieChart
                        title="Status Usulan SK Penghapusan BMN"
                        data=penghapusan_chart
                        show_legend=true
                        size=200
                    />
                </div>
            </section>

            <section>
                <h2 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
                    "Workflow Metrics"
                </h2>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <MetricCard
                        title="Avg Processing Time"
                        value=format!(
                            "{:.1}h",
                            metrics.workflow_metrics.average_processing_time_hours,
                        )
                        icon="⏱️"
                        subtitle="Rata-rata waktu proses"
                    />
                    <MetricCard
                        title="SLA Breaches Today"
                        value=metrics.workflow_metrics.sla_breaches_today.to_string()
                        icon="⚠️"
                        subtitle="Pelanggaran SLA hari ini"
                    />
                    <MetricCard
                        title="Bottlenecks"
                        value=metrics.workflow_metrics.bottlenecks.len().to_string()
                        icon="🚧"
                        subtitle="Tahapan terhambat"
                    />
                </div>
            </section>

            <section>
                <h2 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
                    "Asset Utilization (SIMAN)"
                </h2>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <MetricCard
                        title="Total Assets"
                        value=metrics.asset_utilization.total_assets.to_string()
                        icon="📦"
                        subtitle="Total BMN dari SIMAN"
                    />
                    <MetricCard
                        title="Good Condition"
                        value=metrics.asset_utilization.assets_in_good_condition.to_string()
                        icon="✨"
                        subtitle="BMN kondisi baik"
                    />
                    <MetricCard
                        title="Utilization Rate"
                        value=format!("{:.1}%", metrics.asset_utilization.utilization_percentage)
                        icon="📊"
                        subtitle="Tingkat pemanfaatan"
                    />
                </div>
            </section>
        </div>
    }
}
