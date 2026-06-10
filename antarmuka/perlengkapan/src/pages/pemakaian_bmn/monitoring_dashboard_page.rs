//! # Pemakaian BMN - Monitoring Dashboard
//!
//! This page provides monitoring dashboard for Validator Wilayah and Validator Pusat to:
//! - View active permits by satker and BMN type
//! - Monitor permits expiring soon
//! - View BMN utilization statistics
//! - Track recent activations
//!
//! Requirements: REQ-P016, REQ-P017, REQ-P021

use leptos::prelude::*;
use lib_ui::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDate;

// ============================================================================
// API Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveUsageMonitoringDashboard {
    pub total_active_permits: i64,
    pub permits_by_jenis_bmn: Vec<PermitsByJenisBmn>,
    pub permits_by_satker: Vec<PermitsBySatker>,
    pub expiring_soon: Vec<ExpiringPermitInfo>,
    pub recent_activations: Vec<RecentActivationInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermitsByJenisBmn {
    pub jenis_bmn: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermitsBySatker {
    pub satker_id: Uuid,
    pub satker_nama: String,
    pub active_permits: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpiringPermitInfo {
    pub id: Uuid,
    pub nomor_izin: Option<String>,
    pub bmn_nama: String,
    pub pegawai_nama: String,
    pub tanggal_selesai: NaiveDate,
    pub days_until_expiry: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentActivationInfo {
    pub id: Uuid,
    pub nomor_izin: Option<String>,
    pub bmn_nama: String,
    pub pegawai_nama: String,
    pub activated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmnUtilizationReport {
    pub total_bmn: i64,
    pub bmn_with_active_permits: i64,
    pub bmn_without_permits: i64,
    pub utilization_rate: f64,
    pub bmn_by_type: Vec<BmnUtilizationByType>,
    pub top_utilized_bmn: Vec<TopUtilizedBmn>,
    pub underutilized_bmn: Vec<UnderutilizedBmn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmnUtilizationByType {
    pub jenis_bmn: String,
    pub total_bmn: i64,
    pub utilized_bmn: i64,
    pub utilization_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopUtilizedBmn {
    pub bmn_nup: String,
    pub bmn_nama: String,
    pub jenis_bmn: String,
    pub total_permits: i64,
    pub total_days_used: i64,
    pub current_holder: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnderutilizedBmn {
    pub bmn_nup: String,
    pub bmn_nama: String,
    pub jenis_bmn: String,
    pub last_used_date: Option<NaiveDate>,
    pub days_since_last_use: Option<i64>,
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
pub fn MonitoringDashboardPage() -> impl IntoView {
    // State
    let (dashboard_data, set_dashboard_data) = signal::<Option<ActiveUsageMonitoringDashboard>>(None);
    let (utilization_report, set_utilization_report) = signal::<Option<BmnUtilizationReport>>(None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let (selected_tab, set_selected_tab) = signal(String::from("overview"));
    let (filter_satker, set_filter_satker) = signal::<Option<Uuid>>(None);
    let (filter_jenis_bmn, set_filter_jenis_bmn) = signal::<Option<String>>(None);

    // Load dashboard data on mount
    Effect::new(move || {
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            match fetch_dashboard_data(filter_satker.get(), filter_jenis_bmn.get()).await {
                Ok(data) => set_dashboard_data.set(Some(data)),
                Err(e) => set_error.set(Some(e.to_string())),
            }

            match fetch_utilization_report(filter_satker.get(), filter_jenis_bmn.get()).await {
                Ok(report) => set_utilization_report.set(Some(report)),
                Err(e) => set_error.set(Some(format!("Utilization error: {}", e))),
            }

            set_loading.set(false);
        });
    });

    // Refresh data
    let refresh_data = move || {
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            match fetch_dashboard_data(filter_satker.get(), filter_jenis_bmn.get()).await {
                Ok(data) => set_dashboard_data.set(Some(data)),
                Err(e) => set_error.set(Some(e.to_string())),
            }

            match fetch_utilization_report(filter_satker.get(), filter_jenis_bmn.get()).await {
                Ok(report) => set_utilization_report.set(Some(report)),
                Err(e) => set_error.set(Some(format!("Utilization error: {}", e))),
            }

            set_loading.set(false);
        });
    };

    view! {
        <div class="container mx-auto px-4 py-8">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">
                    "Dashboard Monitoring Pemakaian BMN"
                </h1>
                <Button
                    variant=ButtonVariant::Secondary
                    on_click=Box::new(move |_| refresh_data())
                    disabled=loading.get()
                >
                    {if loading.get() {
                        view! { <Spinner size="sm" /> }.into_any()
                    } else {
                        view! {
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                            </svg>
                        }.into_any()
                    }}
                    " Refresh"
                </Button>
            </div>

            {move || error.get().map(|e| view! {
                <Alert message=e variant=AlertVariant::Error />
            })}

            // Tabs
            <div class="border-b border-gray-200 mb-6">
                <nav class="flex space-x-8">
                    <button
                        class=move || format!(
                            "py-4 px-1 border-b-2 font-medium text-sm {}",
                            if selected_tab.get() == "overview" {
                                "border-emerald-500 text-emerald-600"
                            } else {
                                "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                            }
                        )
                        on:click=move |_| set_selected_tab.set("overview".to_string())
                    >
                        "Overview"
                    </button>
                    <button
                        class=move || format!(
                            "py-4 px-1 border-b-2 font-medium text-sm {}",
                            if selected_tab.get() == "utilization" {
                                "border-emerald-500 text-emerald-600"
                            } else {
                                "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                            }
                        )
                        on:click=move |_| set_selected_tab.set("utilization".to_string())
                    >
                        "Utilization Report"
                    </button>
                </nav>
            </div>

            // Overview Tab
            {move || {
                if selected_tab.get() == "overview" {
                    view! {
                        <div class="space-y-6">
                            {move || dashboard_data.get().map(|data| view! {
                                <div>
                                    // Summary Cards (REQ-P016)
                                    <div class="grid grid-cols-1 md:grid-cols-4 gap-6 mb-6">
                                        <MetricCard
                                            title="Total Izin Aktif"
                                            value=data.total_active_permits.to_string()
                                            icon="document-text"
                                            color="emerald"
                                        />
                                        <MetricCard
                                            title="Akan Berakhir (30 hari)"
                                            value=data.expiring_soon.len().to_string()
                                            icon="clock"
                                            color="yellow"
                                        />
                                        <MetricCard
                                            title="Jenis BMN"
                                            value=data.permits_by_jenis_bmn.len().to_string()
                                            icon="cube"
                                            color="blue"
                                        />
                                        <MetricCard
                                            title="Satker Aktif"
                                            value=data.permits_by_satker.len().to_string()
                                            icon="office-building"
                                            color="purple"
                                        />
                                    </div>

                                    // Permits by Jenis BMN
                                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                                        <Card title="Izin per Jenis BMN">
                                            <div class="space-y-3">
                                                {data.permits_by_jenis_bmn.iter().map(|item| view! {
                                                    <div class="flex items-center justify-between">
                                                        <div class="flex-1">
                                                            <div class="flex items-center justify-between mb-1">
                                                                <span class="text-sm font-medium text-gray-700">
                                                                    {item.jenis_bmn.clone()}
                                                                </span>
                                                                <span class="text-sm text-gray-600">
                                                                    {item.count} " (" {format!("{:.1}%", item.percentage)} ")"
                                                                </span>
                                                            </div>
                                                            <div class="w-full bg-gray-200 rounded-full h-2">
                                                                <div
                                                                    class="bg-emerald-600 h-2 rounded-full"
                                                                    style=format!("width: {}%", item.percentage)
                                                                ></div>
                                                            </div>
                                                        </div>
                                                    </div>
                                                }).collect_view()}
                                            </div>
                                        </Card>

                                        // Permits by Satker
                                        <Card title="Izin per Satker (Top 10)">
                                            <div class="space-y-2 max-h-[400px] overflow-y-auto">
                                                {data.permits_by_satker.iter().take(10).map(|item| view! {
                                                    <div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                                                        <span class="text-sm text-gray-900 flex-1">
                                                            {item.satker_nama.clone()}
                                                        </span>
                                                        <Badge
                                                            label=item.active_permits.to_string()
                                                            variant=BadgeVariant::Info
                                                        />
                                                    </div>
                                                }).collect_view()}
                                            </div>
                                        </Card>
                                    </div>

                                    // Expiring Soon (REQ-P017)
                                    <Card title="Izin yang Akan Berakhir (30 Hari)" class="mt-6">
                                        {if data.expiring_soon.is_empty() {
                                            view! {
                                                <div class="text-center py-8 text-gray-500">
                                                    "Tidak ada izin yang akan berakhir dalam 30 hari"
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="overflow-x-auto">
                                                    <table class="min-w-full divide-y divide-gray-200">
                                                        <thead class="bg-gray-50">
                                                            <tr>
                                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                                                    "No. Izin"
                                                                </th>
                                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                                                    "BMN"
                                                                </th>
                                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                                                    "Pegawai"
                                                                </th>
                                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                                                    "Tanggal Berakhir"
                                                                </th>
                                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                                                    "Sisa Hari"
                                                                </th>
                                                            </tr>
                                                        </thead>
                                                        <tbody class="bg-white divide-y divide-gray-200">
                                                            {data.expiring_soon.iter().map(|permit| view! {
                                                                <tr class="hover:bg-gray-50">
                                                                    <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                                                                        {permit.nomor_izin.clone().unwrap_or_else(|| "-".to_string())}
                                                                    </td>
                                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                        {permit.bmn_nama.clone()}
                                                                    </td>
                                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                        {permit.pegawai_nama.clone()}
                                                                    </td>
                                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                        {permit.tanggal_selesai.to_string()}
                                                                    </td>
                                                                    <td class="px-6 py-4 whitespace-nowrap">
                                                                        <Badge
                                                                            label=format!("{} hari", permit.days_until_expiry)
                                                                            variant=if permit.days_until_expiry <= 7 {
                                                                                BadgeVariant::Error
                                                                            } else if permit.days_until_expiry <= 14 {
                                                                                BadgeVariant::Warning
                                                                            } else {
                                                                                BadgeVariant::Info
                                                                            }
                                                                        />
                                                                    </td>
                                                                </tr>
                                                            }).collect_view()}
                                                        </tbody>
                                                    </table>
                                                </div>
                                            }.into_any()
                                        }}
                                    </Card>

                                    // Recent Activations
                                    <Card title="Aktivasi Terbaru" class="mt-6">
                                        {if data.recent_activations.is_empty() {
                                            view! {
                                                <div class="text-center py-8 text-gray-500">
                                                    "Belum ada aktivasi terbaru"
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="space-y-3">
                                                    {data.recent_activations.iter().map(|activation| view! {
                                                        <div class="flex items-center justify-between p-4 bg-gray-50 rounded-lg">
                                                            <div class="flex-1">
                                                                <p class="font-medium text-gray-900">
                                                                    {activation.bmn_nama.clone()}
                                                                </p>
                                                                <p class="text-sm text-gray-600">
                                                                    "Pegawai: " {activation.pegawai_nama.clone()}
                                                                </p>
                                                                {activation.nomor_izin.as_ref().map(|num| view! {
                                                                    <p class="text-sm text-gray-600 font-mono">
                                                                        "No. Izin: " {num.clone()}
                                                                    </p>
                                                                })}
                                                            </div>
                                                            <div class="text-right">
                                                                <p class="text-sm text-gray-600">
                                                                    {activation.activated_at.clone()}
                                                                </p>
                                                            </div>
                                                        </div>
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        }}
                                    </Card>
                                </div>
                            })}
                        </div>
                    }.into_any()
                } else {
                    // Utilization Tab (REQ-P021)
                    view! {
                        <div class="space-y-6">
                            {move || utilization_report.get().map(|report| view! {
                                <div>
                                    // Utilization Summary
                                    <div class="grid grid-cols-1 md:grid-cols-4 gap-6 mb-6">
                                        <MetricCard
                                            title="Total BMN"
                                            value=report.total_bmn.to_string()
                                            icon="cube"
                                            color="blue"
                                        />
                                        <MetricCard
                                            title="BMN Terpakai"
                                            value=report.bmn_with_active_permits.to_string()
                                            icon="check-circle"
                                            color="emerald"
                                        />
                                        <MetricCard
                                            title="BMN Tidak Terpakai"
                                            value=report.bmn_without_permits.to_string()
                                            icon="x-circle"
                                            color="red"
                                        />
                                        <MetricCard
                                            title="Tingkat Utilisasi"
                                            value=format!("{:.1}%", report.utilization_rate)
                                            icon="chart-bar"
                                            color="purple"
                                        />
                                    </div>

                                    // Utilization by Type
                                    <Card title="Utilisasi per Jenis BMN">
                                        <div class="space-y-4">
                                            {report.bmn_by_type.iter().map(|item| view! {
                                                <div>
                                                    <div class="flex items-center justify-between mb-2">
                                                        <span class="font-medium text-gray-900">
                                                            {item.jenis_bmn.clone()}
                                                        </span>
                                                        <span class="text-sm text-gray-600">
                                                            {item.utilized_bmn} "/" {item.total_bmn}
                                                            " (" {format!("{:.1}%", item.utilization_rate)} ")"
                                                        </span>
                                                    </div>
                                                    <div class="w-full bg-gray-200 rounded-full h-3">
                                                        <div
                                                            class=format!(
                                                                "h-3 rounded-full {}",
                                                                if item.utilization_rate >= 70.0 {
                                                                    "bg-emerald-600"
                                                                } else if item.utilization_rate >= 40.0 {
                                                                    "bg-yellow-500"
                                                                } else {
                                                                    "bg-red-500"
                                                                }
                                                            )
                                                            style=format!("width: {}%", item.utilization_rate)
                                                        ></div>
                                                    </div>
                                                </div>
                                            }).collect_view()}
                                        </div>
                                    </Card>

                                    // Top Utilized BMN
                                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mt-6">
                                        <Card title="BMN Paling Sering Digunakan">
                                            {if report.top_utilized_bmn.is_empty() {
                                                view! {
                                                    <div class="text-center py-8 text-gray-500">
                                                        "Tidak ada data"
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div class="space-y-3">
                                                        {report.top_utilized_bmn.iter().map(|bmn| view! {
                                                            <div class="p-4 bg-gray-50 rounded-lg">
                                                                <div class="flex items-start justify-between">
                                                                    <div class="flex-1">
                                                                        <p class="font-medium text-gray-900">
                                                                            {bmn.bmn_nama.clone()}
                                                                        </p>
                                                                        <p class="text-sm text-gray-600 font-mono">
                                                                            "NUP: " {bmn.bmn_nup.clone()}
                                                                        </p>
                                                                        <p class="text-sm text-gray-600">
                                                                            "Jenis: " {bmn.jenis_bmn.clone()}
                                                                        </p>
                                                                        {bmn.current_holder.as_ref().map(|holder| view! {
                                                                            <p class="text-sm text-emerald-600 mt-1">
                                                                                "Saat ini: " {holder.clone()}
                                                                            </p>
                                                                        })}
                                                                    </div>
                                                                    <div class="text-right">
                                                                        <Badge
                                                                            label=format!("{} izin", bmn.total_permits)
                                                                            variant=BadgeVariant::Success
                                                                        />
                                                                        <p class="text-xs text-gray-600 mt-1">
                                                                            {bmn.total_days_used} " hari"
                                                                        </p>
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        }).collect_view()}
                                                    </div>
                                                }.into_any()
                                            }}
                                        </Card>

                                        // Underutilized BMN
                                        <Card title="BMN Kurang Dimanfaatkan">
                                            {if report.underutilized_bmn.is_empty() {
                                                view! {
                                                    <div class="text-center py-8 text-gray-500">
                                                        "Tidak ada data"
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div class="space-y-3">
                                                        {report.underutilized_bmn.iter().map(|bmn| view! {
                                                            <div class="p-4 bg-yellow-50 rounded-lg border border-yellow-200">
                                                                <div class="flex items-start justify-between">
                                                                    <div class="flex-1">
                                                                        <p class="font-medium text-gray-900">
                                                                            {bmn.bmn_nama.clone()}
                                                                        </p>
                                                                        <p class="text-sm text-gray-600 font-mono">
                                                                            "NUP: " {bmn.bmn_nup.clone()}
                                                                        </p>
                                                                        <p class="text-sm text-gray-600">
                                                                            "Jenis: " {bmn.jenis_bmn.clone()}
                                                                        </p>
                                                                        {bmn.last_used_date.map(|date| view! {
                                                                            <p class="text-sm text-yellow-700 mt-1">
                                                                                "Terakhir digunakan: " {date.to_string()}
                                                                            </p>
                                                                        })}
                                                                    </div>
                                                                    <div class="text-right">
                                                                        {bmn.days_since_last_use.map(|days| view! {
                                                                            <Badge
                                                                                label=format!("{} hari", days)
                                                                                variant=BadgeVariant::Warning
                                                                            />
                                                                        })}
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        }).collect_view()}
                                                    </div>
                                                }.into_any()
                                            }}
                                        </Card>
                                    </div>
                                </div>
                            })}
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

async fn fetch_dashboard_data(
    satker_id: Option<Uuid>,
    jenis_bmn: Option<String>,
) -> Result<ActiveUsageMonitoringDashboard, crate::api::AppError> {
    let mut url = "/api/v1/perlengkapan/pemakaian-bmn/monitoring/active-usage".to_string();
    let mut params = Vec::new();

    if let Some(id) = satker_id {
        params.push(format!("satker_id={}", id));
    }
    if let Some(jenis) = jenis_bmn {
        params.push(format!("jenis_bmn={}", jenis));
    }

    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Network error: {}", e)))?;

    if !response.ok() {
        return Err(crate::api::AppError::Unknown(format!("HTTP error: {}", response.status())));
    }

    let api_response: ApiResponse<ActiveUsageMonitoringDashboard> = response
        .json()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Parse error: {}", e)))?;

    api_response.data.ok_or_else(|| crate::api::AppError::Unknown("No data in response".to_string()))
}

async fn fetch_utilization_report(
    satker_id: Option<Uuid>,
    jenis_bmn: Option<String>,
) -> Result<BmnUtilizationReport, crate::api::AppError> {
    let mut url = "/api/v1/perlengkapan/pemakaian-bmn/monitoring/utilization-report".to_string();
    let mut params = Vec::new();

    if let Some(id) = satker_id {
        params.push(format!("satker_id={}", id));
    }
    if let Some(jenis) = jenis_bmn {
        params.push(format!("jenis_bmn={}", jenis));
    }

    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Network error: {}", e)))?;

    if !response.ok() {
        return Err(crate::api::AppError::Unknown(format!("HTTP error: {}", response.status())));
    }

    let api_response: ApiResponse<BmnUtilizationReport> = response
        .json()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Parse error: {}", e)))?;

    api_response.data.ok_or_else(|| crate::api::AppError::Unknown("No data in response".to_string()))
}
