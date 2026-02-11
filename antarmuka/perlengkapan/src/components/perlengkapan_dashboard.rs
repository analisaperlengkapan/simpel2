//! Perlengkapan Dashboard - Domain-specific metrics and analytics

use leptos::prelude::*;
use lib_ui::components::dashboard::{BarChart, GapAnalysisTable, MetricCard, PieChart};
use lib_ui::core::types::ChartDataPoint;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::utils::{use_dashboard_websocket, ConnectionState, DashboardUpdate};

/// Dashboard query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardParams {
    pub tahun_anggaran: i32,
}

/// Complete perlengkapan dashboard metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerlengkapanDashboardMetrics {
    pub kebutuhan_metrics: KebutuhanMetrics,
    pub gap_analysis: Vec<GapAnalysisResult>,
    pub pakaian_dinas_metrics: PakaianDinasMetrics,
    pub workflow_metrics: WorkflowMetrics,
    pub asset_utilization: AssetUtilization,
}

/// Kebutuhan BMN metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KebutuhanMetrics {
    pub total_by_status: HashMap<String, i64>,
    pub total_by_satker: Vec<SatkerCount>,
    pub total_by_tahun: HashMap<i32, i64>,
}

/// Satker count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatkerCount {
    pub satker_id: String,
    pub satker_nama: String,
    pub count: i64,
}

/// Gap analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapAnalysisResult {
    pub kode_barang: String,
    pub nama_barang: String,
    pub standard_quantity: i32,
    pub existing_good_quantity: i32,
    pub gap: i32,
}

/// Pakaian dinas metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PakaianDinasMetrics {
    pub total_by_jenis: HashMap<String, i64>,
    pub total_by_ukuran: HashMap<String, i64>,
}

/// Workflow performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetrics {
    pub average_processing_time_hours: f64,
    pub bottlenecks: Vec<BottleneckInfo>,
    pub sla_breaches_today: i64,
}

/// Bottleneck information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckInfo {
    pub state: String,
    pub average_time_hours: f64,
    pub count: i64,
}

/// Asset utilization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetUtilization {
    pub total_assets: i64,
    pub assets_in_good_condition: i64,
    pub utilization_percentage: f64,
}

/// Perlengkapan Dashboard Component
#[component]
pub fn PerlengkapanDashboard() -> impl IntoView {
    let (tahun_anggaran, set_tahun_anggaran) = signal(2026);

    // Signal to trigger manual refresh
    let (refresh_trigger, set_refresh_trigger) = signal(0);

    // Fetch dashboard metrics
    let metrics = LocalResource::new(move || {
        let tahun = tahun_anggaran.get();
        let _ = refresh_trigger.get(); // track refresh trigger
        async move {
            fetch_dashboard_metrics(tahun).await
        }
    });

    // WebSocket connection for real-time updates
    let ws_url = format!(
        "ws://{}/api/pembinaan/perlengkapan/dashboard/ws",
        web_sys::window()
            .and_then(|w| w.location().host().ok())
            .unwrap_or_else(|| "localhost:8093".to_string())
    );

    let (ws_state, reconnect_fn) = use_dashboard_websocket(
        ws_url,
        move |update: DashboardUpdate| {
            leptos::logging::log!("Received dashboard update: {:?}", update);

            match update {
                DashboardUpdate::MetricsUpdate { tahun_anggaran: update_tahun, .. } => {
                    // If update is for current year, trigger refresh
                    if update_tahun == tahun_anggaran.get() {
                        leptos::logging::log!("Refreshing dashboard for year {}", update_tahun);
                        set_refresh_trigger.update(|v| *v += 1);
                    }
                }
                DashboardUpdate::WorkflowUpdate { .. } => {
                    // Workflow status changed, refresh dashboard
                    leptos::logging::log!("Workflow update received, refreshing dashboard");
                    set_refresh_trigger.update(|v| *v += 1);
                }
                DashboardUpdate::GapAnalysisUpdate { .. } => {
                    // Gap analysis updated, refresh dashboard
                    leptos::logging::log!("Gap analysis update received, refreshing dashboard");
                    set_refresh_trigger.update(|v| *v += 1);
                }
                DashboardUpdate::Connected { client_id, .. } => {
                    leptos::logging::log!("WebSocket connected with client_id: {}", client_id);
                }
                DashboardUpdate::LagWarning { skipped_messages, .. } => {
                    leptos::logging::warn!("WebSocket lagging, skipped {} messages", skipped_messages);
                }
                _ => {
                    // Ignore other message types (Ping, Pong, Subscribed)
                }
            }
        },
    );

    // Store reconnect function in a local StoredValue so it can be used in reactive closures (StoredValue is Copy)
    let reconnect = StoredValue::new_local(reconnect_fn);

    view! {
        <div class="dashboard-container space-y-8">
            // Header with year filter and connection status
            <div class="flex justify-between items-center">
                <h1 class="text-3xl font-bold text-gray-900">"Dashboard Perlengkapan"</h1>
                <div class="flex items-center gap-4">
                    // WebSocket connection indicator
                    <div class="flex items-center gap-2 px-3 py-1 rounded-lg text-sm"
                        class:bg-green-100=move || ws_state.get() == ConnectionState::Connected
                        class:text-green-700=move || ws_state.get() == ConnectionState::Connected
                        class:bg-yellow-100=move || ws_state.get() == ConnectionState::Connecting
                        class:text-yellow-700=move || ws_state.get() == ConnectionState::Connecting
                        class:bg-red-100=move || ws_state.get() == ConnectionState::Disconnected || ws_state.get() == ConnectionState::Error
                        class:text-red-700=move || ws_state.get() == ConnectionState::Disconnected || ws_state.get() == ConnectionState::Error
                    >
                        <span class="relative flex h-2 w-2">
                            <span class="animate-ping absolute inline-flex h-full w-full rounded-full opacity-75"
                                class:bg-green-400=move || ws_state.get() == ConnectionState::Connected
                                class:bg-yellow-400=move || ws_state.get() == ConnectionState::Connecting
                                class:bg-red-400=move || ws_state.get() == ConnectionState::Disconnected || ws_state.get() == ConnectionState::Error
                            ></span>
                            <span class="relative inline-flex rounded-full h-2 w-2"
                                class:bg-green-500=move || ws_state.get() == ConnectionState::Connected
                                class:bg-yellow-500=move || ws_state.get() == ConnectionState::Connecting
                                class:bg-red-500=move || ws_state.get() == ConnectionState::Disconnected || ws_state.get() == ConnectionState::Error
                            ></span>
                        </span>
                        <span class="font-medium">
                            {move || match ws_state.get() {
                                ConnectionState::Connected => "Live",
                                ConnectionState::Connecting => "Connecting...",
                                ConnectionState::Disconnected => "Disconnected",
                                ConnectionState::Error => "Error",
                            }}
                        </span>
                        {move || {
                            if ws_state.get() == ConnectionState::Disconnected || ws_state.get() == ConnectionState::Error {
                                view! {
                                    <button
                                        class="ml-2 text-xs underline hover:no-underline"
                                        on:click=move |_| reconnect.with_value(|f| f())
                                    >
                                        "Reconnect"
                                    </button>
                                }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }
                        }}
                    </div>

                    <label class="text-sm font-medium text-gray-700">"Tahun Anggaran:"</label>
                    <select
                        class="form-select rounded-lg border-gray-300 shadow-sm focus:border-emerald-500 focus:ring-emerald-500"
                        on:change=move |ev| {
                            let value = event_target_value(&ev).parse::<i32>().unwrap_or(2026);
                            set_tahun_anggaran.set(value);
                        }
                    >
                        <option value="2024">"2024"</option>
                        <option value="2025">"2025"</option>
                        <option value="2026" selected>"2026"</option>
                        <option value="2027">"2027"</option>
                    </select>
                </div>
            </div>

            // Dashboard content
            <Suspense fallback=move || view! {
                <div class="flex justify-center items-center py-12">
                    <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-emerald-600"></div>
                </div>
            }>
                {move || metrics.get().map(|result| match result {
                    Ok(data) => view! {
                        <div class="space-y-8">
                            // Key Metrics Cards
                            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                                <MetricCard
                                    title="Avg Processing Time".to_string()
                                    value=format!("{:.1} hours", data.workflow_metrics.average_processing_time_hours)
                                    subtitle="Workflow performance".to_string()
                                    icon="⏱️".to_string()
                                />
                                <MetricCard
                                    title="SLA Breaches".to_string()
                                    value=data.workflow_metrics.sla_breaches_today.to_string()
                                    subtitle="Today".to_string()
                                    icon="⚠️".to_string()
                                />
                                <MetricCard
                                    title="Asset Utilization".to_string()
                                    value=format!("{:.1}%", data.asset_utilization.utilization_percentage)
                                    subtitle=format!("{} / {} assets in good condition",
                                        data.asset_utilization.assets_in_good_condition,
                                        data.asset_utilization.total_assets)
                                    icon="📦".to_string()
                                />
                            </div>

                            // Kebutuhan BMN Section
                            <div class="bg-white rounded-lg shadow-md p-6">
                                <h2 class="text-2xl font-bold text-gray-900 mb-6">"Kebutuhan BMN"</h2>
                                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                                    // Status distribution
                                    <PieChart
                                        title="By Status".to_string()
                                        data=convert_status_to_chart_data(&data.kebutuhan_metrics.total_by_status)
                                    />
                                    // Top satkers
                                    <BarChart
                                        title="Top 10 Satker".to_string()
                                        data=convert_satker_to_chart_data(&data.kebutuhan_metrics.total_by_satker)
                                    />
                                </div>
                            </div>

                            // Gap Analysis Section
                            <div class="bg-white rounded-lg shadow-md p-6">
                                <h2 class="text-2xl font-bold text-gray-900 mb-6">"Gap Analysis (Top 10)"</h2>
                                <GapAnalysisTable
                                    data=convert_gap_analysis(&data.gap_analysis)
                                />
                            </div>

                            // Pakaian Dinas Section
                            <div class="bg-white rounded-lg shadow-md p-6">
                                <h2 class="text-2xl font-bold text-gray-900 mb-6">"Pakaian Dinas"</h2>
                                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                                    <PieChart
                                        title="By Jenis".to_string()
                                        data=convert_status_to_chart_data(&data.pakaian_dinas_metrics.total_by_jenis)
                                    />
                                    <BarChart
                                        title="By Ukuran".to_string()
                                        data=convert_status_to_chart_data(&data.pakaian_dinas_metrics.total_by_ukuran)
                                    />
                                </div>
                            </div>

                            // Workflow Bottlenecks
                            {if !data.workflow_metrics.bottlenecks.is_empty() {
                                view! {
                                    <div class="bg-white rounded-lg shadow-md p-6">
                                        <h2 class="text-2xl font-bold text-gray-900 mb-6">"Workflow Bottlenecks"</h2>
                                        <div class="overflow-x-auto">
                                            <table class="min-w-full divide-y divide-gray-200">
                                                <thead class="bg-gray-50">
                                                    <tr>
                                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"State"</th>
                                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Avg Time (hours)"</th>
                                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Count"</th>
                                                    </tr>
                                                </thead>
                                                <tbody class="bg-white divide-y divide-gray-200">
                                                    {data.workflow_metrics.bottlenecks.iter().map(|bottleneck| {
                                                        view! {
                                                            <tr>
                                                                <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">{bottleneck.state.clone()}</td>
                                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{format!("{:.1}", bottleneck.average_time_hours)}</td>
                                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{bottleneck.count}</td>
                                                            </tr>
                                                        }
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }}
                        </div>
                    }.into_any(),
                    Err(e) => view! {
                        <div class="bg-red-50 border border-red-200 rounded-lg p-6">
                            <div class="flex items-center gap-3">
                                <i class="fas fa-exclamation-circle text-red-600 text-2xl"></i>
                                <div>
                                    <h3 class="text-lg font-semibold text-red-900">"Error Loading Dashboard"</h3>
                                    <p class="text-sm text-red-700">{e}</p>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

// Helper functions to convert data to chart format

fn convert_status_to_chart_data(data: &HashMap<String, i64>) -> Vec<ChartDataPoint> {
    data.iter()
        .map(|(label, value)| ChartDataPoint {
            label: label.clone(),
            value: *value as f64,
            color: None,
        })
        .collect()
}

fn convert_satker_to_chart_data(data: &[SatkerCount]) -> Vec<ChartDataPoint> {
    data.iter()
        .map(|satker| ChartDataPoint {
            label: satker.satker_nama.clone(),
            value: satker.count as f64,
            color: None,
        })
        .collect()
}

fn convert_gap_analysis(data: &[GapAnalysisResult]) -> Vec<lib_ui::core::types::GapAnalysisRow> {
    data.iter()
        .map(|gap| lib_ui::core::types::GapAnalysisRow {
            kode_barang: gap.kode_barang.clone(),
            nama_barang: gap.nama_barang.clone(),
            satker_name: String::new(),
            standard_quantity: gap.standard_quantity,
            existing_quantity: gap.existing_good_quantity,
            gap: gap.gap,
        })
        .collect()
}

// API function to fetch dashboard metrics
async fn fetch_dashboard_metrics(tahun_anggaran: i32) -> Result<PerlengkapanDashboardMetrics, String> {
    let url = format!(
        "/api/pembinaan/perlengkapan/dashboard/perlengkapan?tahun_anggaran={}",
        tahun_anggaran
    );

    gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch dashboard metrics: {}", e))?
        .json::<PerlengkapanDashboardMetrics>()
        .await
        .map_err(|e| format!("Failed to parse dashboard metrics: {}", e))
}
