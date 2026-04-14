//! Workflow Monitoring Dashboard
//!
//! Real-time monitoring of workflow metrics, SLA compliance, and bottlenecks.

use crate::api::workflow::format_sla;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════
// API Types (matching backend monitoring.rs)
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetrics {
    pub total_active: i64,
    pub by_state: HashMap<String, i64>,
    pub avg_processing_time: HashMap<String, f64>,
    pub sla_breaches: i64,
    pub approaching_sla: i64,
    pub bottlenecks: Vec<BottleneckInfo>,
    pub calculated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckInfo {
    pub state: String,
    pub count: i64,
    pub avg_time_minutes: f64,
    pub sla_limit_minutes: Option<u32>,
    pub sla_usage_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSummary {
    pub id: String,
    pub satker_id: String,
    pub kode_barang: String,
    pub nama_barang: String,
    pub status: String,
    pub state_entered_at: String,
    pub time_in_state_minutes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// API Functions
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(target_arch = "wasm32")]
pub async fn fetch_workflow_metrics() -> Result<ApiResponse<WorkflowMetrics>, String> {
    use crate::api::client::{get_auth_token, API_BASE};
    use gloo_net::http::Request;

    let url = format!("{}/workflow/monitoring/metrics", API_BASE);
    let token = get_auth_token().unwrap_or_default();

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.ok() {
        return Err(format!("API error: HTTP {}", resp.status()));
    }

    resp.json::<ApiResponse<WorkflowMetrics>>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_workflow_metrics() -> Result<ApiResponse<WorkflowMetrics>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_active_workflows() -> Result<ApiResponse<Vec<WorkflowSummary>>, String> {
    use crate::api::client::{get_auth_token, API_BASE};
    use gloo_net::http::Request;

    let url = format!("{}/workflow/monitoring/active", API_BASE);
    let token = get_auth_token().unwrap_or_default();

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.ok() {
        return Err(format!("API error: HTTP {}", resp.status()));
    }

    resp.json::<ApiResponse<Vec<WorkflowSummary>>>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_active_workflows() -> Result<ApiResponse<Vec<WorkflowSummary>>, String> {
    Err("Server-side stub".to_string())
}

// ═══════════════════════════════════════════════════════════════════════════
// Metric Card Component
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn MetricCard(
    title: &'static str,
    value: String,
    icon: &'static str,
    color: &'static str,
    #[prop(optional)] subtitle: String,
) -> impl IntoView {
    view! {
        <div style=format!(
            "background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 16px; padding: 20px; position: relative; overflow: hidden;"
        )>
            // Background icon
            <div style=format!(
                "position: absolute; top: -10px; right: -10px; font-size: 5rem; color: {}10; opacity: 0.3;",
                color
            )>
                <i class=icon></i>
            </div>

            // Content
            <div style="position: relative; z-index: 1;">
                <div style="display: flex; align-items: center; gap: 10px; margin-bottom: 12px;">
                    <div style=format!(
                        "width: 40px; height: 40px; display: flex; align-items: center; justify-content: center; background: {}15; border: 1px solid {}40; border-radius: 10px;",
                        color, color
                    )>
                        <i class=icon style=format!("font-size: 1.1rem; color: {};", color)></i>
                    </div>
                    <div style="font-size: 0.8rem; font-weight: 600; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em;">
                        {title}
                    </div>
                </div>
                <div style="font-size: 2rem; font-weight: 800; color: #e2e8f0; margin-bottom: 4px;">
                    {value}
                </div>
                {if !subtitle.is_empty() {
                    view! {
                        <div style="font-size: 0.75rem; color: #64748b;">
                            {subtitle}
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }}
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Bottleneck Card Component
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn BottleneckCard(bottleneck: BottleneckInfo) -> impl IntoView {
    let usage_percent = bottleneck.sla_usage_percent.unwrap_or(0.0);
    let color = if usage_percent >= 90.0 {
        "#f87171"
    } else if usage_percent >= 75.0 {
        "#fb923c"
    } else {
        "#fbbf24"
    };

    let sla_display = bottleneck
        .sla_limit_minutes
        .map(format_sla)
        .unwrap_or_else(|| "—".to_string());

    view! {
        <div style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 12px; padding: 16px;">
            <div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 12px;">
                <div style="flex: 1;">
                    <div style="font-size: 0.9rem; font-weight: 700; color: #e2e8f0; margin-bottom: 4px;">
                        {bottleneck.state.clone()}
                    </div>
                    <div style="font-size: 0.75rem; color: #64748b;">
                        {bottleneck.count} " workflows aktif"
                    </div>
                </div>
                <div style=format!(
                    "padding: 4px 10px; background: {}15; border: 1px solid {}40; border-radius: 8px; font-size: 0.7rem; font-weight: 600; color: {};",
                    color, color, color
                )>
                    {format!("{:.1}%", usage_percent)}
                </div>
            </div>

            // Progress bar
            <div style="background: rgba(255,255,255,0.06); border-radius: 999px; height: 8px; overflow: hidden; margin-bottom: 12px;">
                <div style=format!(
                    "height: 100%; background: linear-gradient(90deg, {}, {}); width: {}%; transition: width 0.3s;",
                    color, color, usage_percent.min(100.0)
                )></div>
            </div>

            <div style="display: flex; justify-content: space-between; font-size: 0.7rem; color: #94a3b8;">
                <span>
                    "Avg: " <strong style="color: #e2e8f0;">{format!("{:.0}", bottleneck.avg_time_minutes)} " min"</strong>
                </span>
                <span>
                    "SLA: " <strong style="color: #e2e8f0;">{sla_display}</strong>
                </span>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Workflow Status Table Component
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn WorkflowStatusTable(workflows: Vec<WorkflowSummary>) -> impl IntoView {
    view! {
        <div style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 16px; overflow: hidden;">
            <div style="padding: 20px; border-bottom: 1px solid rgba(255,255,255,0.08);">
                <h3 style="font-size: 1rem; font-weight: 700; color: #e2e8f0; margin: 0;">
                    "Workflows Aktif"
                </h3>
            </div>
            <div style="overflow-x: auto;">
                <table style="width: 100%; border-collapse: collapse;">
                    <thead>
                        <tr style="background: rgba(255,255,255,0.02); border-bottom: 1px solid rgba(255,255,255,0.08);">
                            <th style="padding: 12px; text-align: left; font-size: 0.75rem; font-weight: 600; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em;">"Kode Barang"</th>
                            <th style="padding: 12px; text-align: left; font-size: 0.75rem; font-weight: 600; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em;">"Nama Barang"</th>
                            <th style="padding: 12px; text-align: left; font-size: 0.75rem; font-weight: 600; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em;">"Status"</th>
                            <th style="padding: 12px; text-align: left; font-size: 0.75rem; font-weight: 600; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em;">"Waktu di State"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {workflows.into_iter().map(|wf| {
                            let time_display = if wf.time_in_state_minutes < 60 {
                                format!("{} menit", wf.time_in_state_minutes)
                            } else if wf.time_in_state_minutes < 1440 {
                                format!("{} jam", wf.time_in_state_minutes / 60)
                            } else {
                                format!("{} hari", wf.time_in_state_minutes / 1440)
                            };

                            view! {
                                <tr style="border-bottom: 1px solid rgba(255,255,255,0.06);">
                                    <td style="padding: 14px 12px;">
                                        <span style="font-size: 0.8rem; font-weight: 600; color: #60a5fa; font-family: monospace;">
                                            {wf.kode_barang}
                                        </span>
                                    </td>
                                    <td style="padding: 14px 12px;">
                                        <span style="font-size: 0.8rem; color: #e2e8f0;">
                                            {wf.nama_barang}
                                        </span>
                                    </td>
                                    <td style="padding: 14px 12px;">
                                        <span style="display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); border-radius: 8px; font-size: 0.7rem; color: #94a3b8;">
                                            {wf.status}
                                        </span>
                                    </td>
                                    <td style="padding: 14px 12px;">
                                        <span style="font-size: 0.8rem; color: #94a3b8;">
                                            {time_display}
                                        </span>
                                    </td>
                                </tr>
                            }
                        }).collect::<Vec<_>>()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Main Monitoring Dashboard Component
// ═══════════════════════════════════════════════════════════════════════════

#[component]
pub fn WorkflowMonitoring() -> impl IntoView {
    let metrics_resource = LocalResource::new(|| fetch_workflow_metrics());
    let workflows_resource = LocalResource::new(|| fetch_active_workflows());

    // Auto-refresh every 30 seconds
    let (refresh_counter, set_refresh_counter) = signal(0);

    Effect::new(move |_| {
        let interval = gloo_timers::callback::Interval::new(30_000, move || {
            set_refresh_counter.update(|c| *c += 1);
            metrics_resource.refetch();
            workflows_resource.refetch();
        });

        // Keep interval alive
        std::mem::forget(interval);
    });

    let manual_refresh = move |_| {
        metrics_resource.refetch();
        workflows_resource.refetch();
    };

    view! {
        <div style="max-width: 1600px; margin: 0 auto;">
            // Header
            <div style="margin-bottom: 28px;">
                <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px;">
                    <div style="display: flex; align-items: center; gap: 12px;">
                        <div style="width: 4px; height: 32px; background: linear-gradient(180deg, #60a5fa, #3b82f6); border-radius: 2px;"></div>
                        <h1 style="font-size: 1.5rem; font-weight: 800; color: #e2e8f0; margin: 0;">
                            "Monitoring Workflow"
                        </h1>
                    </div>
                    <button
                        on:click=manual_refresh
                        style="padding: 10px 18px; background: rgba(96,165,250,0.1); border: 1px solid rgba(96,165,250,0.3); border-radius: 10px; color: #60a5fa; font-size: 0.85rem; font-weight: 600; cursor: pointer; transition: all 0.2s; display: flex; align-items: center; gap: 8px;"
                        class="hover:bg-blue-500/20"
                    >
                        <i class="fas fa-sync-alt"></i>
                        "Refresh"
                    </button>
                </div>
                <p style="font-size: 0.9rem; color: #94a3b8; margin: 0 0 0 16px;">
                    "Pantau performa workflow secara real-time dengan metrik SLA dan bottleneck"
                </p>
            </div>

            // Metrics Section
            <Suspense fallback=move || view! {
                <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 16px; margin-bottom: 28px;">
                    {(0..4).map(|_| view! {
                        <div style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 16px; padding: 20px; height: 140px;">
                            <div style="width: 60%; height: 16px; background: rgba(255,255,255,0.06); border-radius: 4px; margin-bottom: 16px;"></div>
                            <div style="width: 40%; height: 32px; background: rgba(255,255,255,0.06); border-radius: 4px;"></div>
                        </div>
                    }).collect::<Vec<_>>()}
                </div>
            }>
                {move || {
                    metrics_resource.get().map(|result| match result {
                        Ok(response) => {
                            let metrics = response.data;
                            view! {
                                <div>
                                    // Key Metrics Grid
                                    <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 16px; margin-bottom: 28px;">
                                        <MetricCard
                                            title="Total Aktif"
                                            value=metrics.total_active.to_string()
                                            icon="fas fa-tasks"
                                            color="#60a5fa"
                                            subtitle="Workflows dalam proses".to_string()
                                        />
                                        <MetricCard
                                            title="SLA Breaches"
                                            value=metrics.sla_breaches.to_string()
                                            icon="fas fa-exclamation-triangle"
                                            color="#f87171"
                                            subtitle="Melebihi batas waktu".to_string()
                                        />
                                        <MetricCard
                                            title="Mendekati SLA"
                                            value=metrics.approaching_sla.to_string()
                                            icon="fas fa-clock"
                                            color="#fb923c"
                                            subtitle="Dalam 25% batas waktu".to_string()
                                        />
                                        <MetricCard
                                            title="Bottlenecks"
                                            value=metrics.bottlenecks.len().to_string()
                                            icon="fas fa-hourglass-half"
                                            color="#fbbf24"
                                            subtitle="States dengan delay".to_string()
                                        />
                                    </div>

                                    // Bottlenecks Section
                                    {if !metrics.bottlenecks.is_empty() {
                                        view! {
                                            <div style="margin-bottom: 28px;">
                                                <div style="display: flex; align-items: center; gap: 10px; margin-bottom: 16px;">
                                                    <i class="fas fa-hourglass-half" style="color: #fbbf24; font-size: 1.2rem;"></i>
                                                    <h2 style="font-size: 1.2rem; font-weight: 700; color: #e2e8f0; margin: 0;">
                                                        "Bottlenecks Terdeteksi"
                                                    </h2>
                                                </div>
                                                <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 16px;">
                                                    {metrics.bottlenecks.into_iter().map(|bottleneck| view! {
                                                        <BottleneckCard bottleneck=bottleneck />
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div style="background: rgba(34,197,94,0.1); border: 1px solid rgba(34,197,94,0.3); border-radius: 12px; padding: 16px; margin-bottom: 28px; display: flex; align-items: center; gap: 12px;">
                                                <i class="fas fa-check-circle" style="color: #22c55e; font-size: 1.5rem;"></i>
                                                <div>
                                                    <div style="font-size: 0.9rem; font-weight: 600; color: #22c55e; margin-bottom: 4px;">
                                                        "Tidak Ada Bottleneck"
                                                    </div>
                                                    <div style="font-size: 0.8rem; color: #86efac;">
                                                        "Semua workflow states berjalan lancar dalam batas SLA"
                                                    </div>
                                                </div>
                                            </div>
                                        }.into_any()
                                    }}

                                    // Workflows by State
                                    {if !metrics.by_state.is_empty() {
                                        view! {
                                            <div style="margin-bottom: 28px;">
                                                <div style="display: flex; align-items: center; gap: 10px; margin-bottom: 16px;">
                                                    <i class="fas fa-chart-pie" style="color: #a78bfa; font-size: 1.2rem;"></i>
                                                    <h2 style="font-size: 1.2rem; font-weight: 700; color: #e2e8f0; margin: 0;">
                                                        "Distribusi per State"
                                                    </h2>
                                                </div>
                                                <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 12px;">
                                                    {metrics.by_state.into_iter().map(|(state, count)| view! {
                                                        <div style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 12px; padding: 16px;">
                                                            <div style="font-size: 0.75rem; font-weight: 600; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 8px;">
                                                                {state}
                                                            </div>
                                                            <div style="font-size: 1.5rem; font-weight: 800; color: #e2e8f0;">
                                                                {count}
                                                            </div>
                                                        </div>
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        }
                        Err(e) => view! {
                            <div style="text-align: center; padding: 60px 20px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 16px;">
                                <i class="fas fa-exclamation-triangle" style="font-size: 3rem; color: #f87171; margin-bottom: 16px;"></i>
                                <h3 style="font-size: 1.1rem; font-weight: 700; color: #e2e8f0; margin: 0 0 8px 0;">
                                    "Gagal Memuat Metrics"
                                </h3>
                                <p style="font-size: 0.85rem; color: #94a3b8; margin: 0;">
                                    {e}
                                </p>
                            </div>
                        }.into_any()
                    })
                }}
            </Suspense>

            // Active Workflows Table
            <div style="display: flex; align-items: center; gap: 10px; margin-bottom: 16px;">
                <i class="fas fa-list" style="color: #60a5fa; font-size: 1.2rem;"></i>
                <h2 style="font-size: 1.2rem; font-weight: 700; color: #e2e8f0; margin: 0;">
                    "Workflows Aktif"
                </h2>
            </div>

            <Suspense fallback=move || view! {
                <div style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 16px; padding: 20px; height: 300px;">
                    <div style="width: 40%; height: 20px; background: rgba(255,255,255,0.06); border-radius: 4px; margin-bottom: 20px;"></div>
                    {(0..5).map(|_| view! {
                        <div style="width: 100%; height: 40px; background: rgba(255,255,255,0.06); border-radius: 4px; margin-bottom: 12px;"></div>
                    }).collect::<Vec<_>>()}
                </div>
            }>
                {move || {
                    workflows_resource.get().map(|result| match result {
                        Ok(response) => {
                            if response.data.is_empty() {
                                view! {
                                    <div style="text-align: center; padding: 60px 20px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 16px;">
                                        <i class="fas fa-inbox" style="font-size: 3rem; color: #64748b; margin-bottom: 16px;"></i>
                                        <h3 style="font-size: 1.1rem; font-weight: 700; color: #e2e8f0; margin: 0 0 8px 0;">
                                            "Tidak Ada Workflow Aktif"
                                        </h3>
                                        <p style="font-size: 0.85rem; color: #94a3b8; margin: 0;">
                                            "Semua workflows telah selesai atau belum ada yang dimulai"
                                        </p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <WorkflowStatusTable workflows=response.data />
                                }.into_any()
                            }
                        }
                        Err(e) => view! {
                            <div style="text-align: center; padding: 60px 20px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 16px;">
                                <i class="fas fa-exclamation-triangle" style="font-size: 3rem; color: #f87171; margin-bottom: 16px;"></i>
                                <h3 style="font-size: 1.1rem; font-weight: 700; color: #e2e8f0; margin: 0 0 8px 0;">
                                    "Gagal Memuat Workflows"
                                </h3>
                                <p style="font-size: 0.85rem; color: #94a3b8; margin: 0;">
                                    {e}
                                </p>
                            </div>
                        }.into_any()
                    })
                }}
            </Suspense>

            // Auto-refresh indicator
            <div style="margin-top: 20px; text-align: center;">
                <span style="font-size: 0.7rem; color: #64748b;">
                    <i class="fas fa-sync-alt" style="margin-right: 6px;"></i>
                    "Auto-refresh setiap 30 detik"
                </span>
            </div>
        </div>
    }
}
