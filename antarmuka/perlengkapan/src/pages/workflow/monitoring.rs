//! Workflow Monitoring Dashboard
//!
//! Real-time monitoring of workflow metrics, SLA compliance, and bottlenecks.

use std::collections::HashMap;

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};

use crate::api::client::api_get;
use crate::api::error::{AppError, AppResult};
use crate::api::workflow::format_sla;
use crate::components::layout::{
    EmptyState, ErrorState, LoadingState, PageLayout, SectionCard, StatCard, StatTone,
};

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

pub async fn fetch_workflow_metrics() -> AppResult<WorkflowMetrics> {
    let resp: ApiResponse<WorkflowMetrics> = api_get("/workflow/monitoring/metrics").await?;
    Ok(resp.data)
}

pub async fn fetch_active_workflows() -> AppResult<Vec<WorkflowSummary>> {
    let resp: ApiResponse<Vec<WorkflowSummary>> = api_get("/workflow/monitoring/active").await?;
    Ok(resp.data)
}

// ═══════════════════════════════════════════════════════════════════════════
// Presentational helpers
// ═══════════════════════════════════════════════════════════════════════════

fn bottleneck_tone(usage_percent: f64) -> (StatTone, &'static str) {
    if usage_percent >= 90.0 {
        (StatTone::Danger, "text-danger-400")
    } else if usage_percent >= 75.0 {
        (StatTone::Warning, "text-warning-400")
    } else {
        (StatTone::Gold, "text-gold-400")
    }
}

fn bottleneck_bar_classes(usage_percent: f64) -> &'static str {
    if usage_percent >= 90.0 {
        "bg-danger-500"
    } else if usage_percent >= 75.0 {
        "bg-warning-500"
    } else {
        "bg-gold-400"
    }
}

fn format_time_in_state(minutes: i64) -> String {
    if minutes < 60 {
        format!("{minutes} menit")
    } else if minutes < 1440 {
        format!("{} jam", minutes / 60)
    } else {
        format!("{} hari", minutes / 1440)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Bottleneck Card
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn BottleneckCard(bottleneck: BottleneckInfo) -> impl IntoView {
    let usage_percent = bottleneck.sla_usage_percent.unwrap_or(0.0);
    let (tone, text_class) = bottleneck_tone(usage_percent);
    let bar_class = bottleneck_bar_classes(usage_percent);
    let width_class = match usage_percent.min(100.0) as u32 {
        0..=5 => "w-[5%]",
        6..=10 => "w-[10%]",
        11..=25 => "w-1/4",
        26..=50 => "w-1/2",
        51..=75 => "w-3/4",
        76..=90 => "w-[90%]",
        _ => "w-full",
    };

    let badge_class = match tone {
        StatTone::Danger => "bg-danger-500/15 text-danger-400 ring-danger-500/40",
        StatTone::Warning => "bg-warning-500/15 text-warning-400 ring-warning-500/40",
        _ => "bg-gold-500/15 text-gold-400 ring-gold-500/40",
    };

    let sla_display = bottleneck
        .sla_limit_minutes
        .map(format_sla)
        .unwrap_or_else(|| "—".to_string());

    view! {
        <div class="rounded-xl border border-white/[0.08] bg-white/[0.04] p-4">
            <div class="mb-3 flex items-start justify-between gap-3">
                <div class="flex-1">
                    <div class="text-sm font-bold text-slate-200">{bottleneck.state.clone()}</div>
                    <div class="mt-0.5 text-xs text-slate-500">
                        {bottleneck.count} " workflows aktif"
                    </div>
                </div>
                <span class=format!("rounded-lg px-2.5 py-1 text-[0.7rem] font-semibold ring-1 {badge_class}")>
                    {format!("{:.1}%", usage_percent)}
                </span>
            </div>

            <div class="mb-3 h-2 overflow-hidden rounded-full bg-white/[0.06]">
                <div class=format!("h-full rounded-full transition-all duration-300 {bar_class} {width_class}")></div>
            </div>

            <div class="flex justify-between text-[0.7rem] text-slate-400">
                <span>
                    "Avg: "
                    <strong class=format!("font-semibold {text_class}")>
                        {format!("{:.0}", bottleneck.avg_time_minutes)} " menit"
                    </strong>
                </span>
                <span>
                    "SLA: " <strong class="font-semibold text-slate-200">{sla_display}</strong>
                </span>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Workflow Status Table
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn WorkflowStatusTable(workflows: Vec<WorkflowSummary>) -> impl IntoView {
    view! {
        <div class="overflow-hidden rounded-xl border border-white/[0.08]">
            <div class="overflow-x-auto">
                <table class="min-w-full divide-y divide-white/[0.06]">
                    <thead class="bg-white/[0.02]">
                        <tr>
                            <th scope="col" class="px-3 py-3 text-left text-[0.7rem] font-semibold uppercase tracking-wider text-slate-400">
                                "Kode Barang"
                            </th>
                            <th scope="col" class="px-3 py-3 text-left text-[0.7rem] font-semibold uppercase tracking-wider text-slate-400">
                                "Nama Barang"
                            </th>
                            <th scope="col" class="px-3 py-3 text-left text-[0.7rem] font-semibold uppercase tracking-wider text-slate-400">
                                "Status"
                            </th>
                            <th scope="col" class="px-3 py-3 text-left text-[0.7rem] font-semibold uppercase tracking-wider text-slate-400">
                                "Waktu di State"
                            </th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-white/[0.04]">
                        {workflows.into_iter().map(|wf| {
                            let time_display = format_time_in_state(wf.time_in_state_minutes);
                            view! {
                                <tr class="transition hover:bg-white/[0.02]">
                                    <td class="px-3 py-3">
                                        <span class="font-mono text-xs font-semibold text-info-300">
                                            {wf.kode_barang}
                                        </span>
                                    </td>
                                    <td class="px-3 py-3 text-sm text-slate-200">
                                        {wf.nama_barang}
                                    </td>
                                    <td class="px-3 py-3">
                                        <span class="inline-flex items-center gap-1.5 rounded-lg border border-white/[0.08] bg-white/[0.04] px-2.5 py-1 text-[0.7rem] font-medium text-slate-300">
                                            {wf.status}
                                        </span>
                                    </td>
                                    <td class="px-3 py-3 text-sm text-slate-400">
                                        {time_display}
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
// State Distribution Grid
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn StateDistributionGrid(by_state: HashMap<String, i64>) -> impl IntoView {
    let mut entries: Vec<_> = by_state.into_iter().collect();
    entries.sort_by_key(|b| std::cmp::Reverse(b.1));

    view! {
        <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {entries.into_iter().map(|(state, count)| view! {
                <div class="rounded-xl border border-white/[0.06] bg-white/[0.03] p-4">
                    <div class="text-[0.7rem] font-semibold uppercase tracking-wider text-slate-500">
                        {state}
                    </div>
                    <div class="mt-2 text-2xl font-bold text-white">{count}</div>
                </div>
            }).collect::<Vec<_>>()}
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Main Monitoring Dashboard Component
// ═══════════════════════════════════════════════════════════════════════════

#[component]
pub fn WorkflowMonitoring() -> impl IntoView {
    let (metrics, set_metrics) = signal::<Option<WorkflowMetrics>>(None);
    let (workflows, set_workflows) = signal::<Option<Vec<WorkflowSummary>>>(None);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<AppError>>(None);
    let (reload_tick, set_reload_tick) = signal(0_u32);

    // Fetch both resources when tick changes.
    Effect::new(move |_| {
        let _ = reload_tick.get();
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            let metrics_result = fetch_workflow_metrics().await;
            let workflows_result = fetch_active_workflows().await;
            match (metrics_result, workflows_result) {
                (Ok(m), Ok(w)) => {
                    set_metrics.set(Some(m));
                    set_workflows.set(Some(w));
                }
                (Err(e), _) | (_, Err(e)) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    });

    // Auto-refresh every 30s.
    Effect::new(move |_| {
        let interval = gloo_timers::callback::Interval::new(30_000, move || {
            set_reload_tick.update(|c| *c += 1);
        });
        std::mem::forget(interval);
    });

    let trigger_refresh = move |_| {
        set_reload_tick.update(|c| *c += 1);
    };

    view! {
        <PageLayout
            title="Monitoring Workflow"
            description="Pantau performa workflow secara real-time dengan metrik SLA dan bottleneck."
            icon="fas fa-gauge-high"
            actions=Box::new(move || view! {
                <button
                    type="button"
                    on:click=trigger_refresh
                    class="focus-ring inline-flex items-center gap-2 rounded-lg border border-info-500/30 bg-info-500/10 px-3 py-1.5 text-xs font-semibold text-info-300 transition hover:bg-info-500/20"
                >
                    <i class="fas fa-sync-alt text-[0.7rem]"></i>
                    "Refresh"
                </button>
                <span class="inline-flex items-center gap-1.5 text-[0.7rem] text-slate-500">
                    <i class="fas fa-clock-rotate-left"></i>
                    "Auto-refresh 30 detik"
                </span>
            }.into_any())
        >
            {move || {
                if loading.get() && metrics.get().is_none() {
                    view! { <LoadingState message="Memuat metrics workflow..." /> }.into_any()
                } else if let Some(err) = error.get() {
                    view! {
                        <ErrorState
                            error=err
                            title="Gagal memuat metrics workflow".to_string()
                            on_retry=Box::new(move || { set_reload_tick.update(|t| *t += 1); })
                        />
                    }.into_any()
                } else if let Some(m) = metrics.get() {
                    let w = workflows.get().unwrap_or_default();
                    view! { <MonitoringContent metrics=m workflows=w /> }.into_any()
                } else {
                    view! { <LoadingState message="Memuat metrics workflow..." /> }.into_any()
                }
            }}
        </PageLayout>
    }
}

#[component]
fn MonitoringContent(metrics: WorkflowMetrics, workflows: Vec<WorkflowSummary>) -> impl IntoView {
    let bottlenecks = metrics.bottlenecks.clone();
    let bottleneck_count = bottlenecks.len();
    let has_bottlenecks = !bottlenecks.is_empty();
    let by_state = metrics.by_state.clone();
    let has_state_distribution = !by_state.is_empty();

    view! {
        <SectionCard
            title="Ringkasan Metrics"
            description="Indikator utama performa workflow."
            icon="fas fa-chart-simple"
        >
            <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
                <StatCard
                    label="Total Aktif"
                    value=metrics.total_active.to_string()
                    caption="Workflows dalam proses"
                    icon="fas fa-tasks"
                    tone=StatTone::Info
                />
                <StatCard
                    label="SLA Breaches"
                    value=metrics.sla_breaches.to_string()
                    caption="Melebihi batas waktu"
                    icon="fas fa-exclamation-triangle"
                    tone=StatTone::Danger
                />
                <StatCard
                    label="Mendekati SLA"
                    value=metrics.approaching_sla.to_string()
                    caption="Dalam 25% batas waktu"
                    icon="fas fa-clock"
                    tone=StatTone::Warning
                />
                <StatCard
                    label="Bottlenecks"
                    value=bottleneck_count.to_string()
                    caption="States dengan delay"
                    icon="fas fa-hourglass-half"
                    tone=StatTone::Gold
                />
            </div>
        </SectionCard>

        <SectionCard
            title="Bottlenecks"
            description="State yang mendekati atau melewati SLA."
            icon="fas fa-hourglass-half"
        >
            {if has_bottlenecks {
                view! {
                    <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
                        {bottlenecks.into_iter().map(|b| view! {
                            <BottleneckCard bottleneck=b />
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="flex items-center gap-3 rounded-xl border border-success-500/30 bg-success-500/[0.08] p-4">
                        <span class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-success-500/15 text-success-400 ring-1 ring-success-500/25">
                            <i class="fas fa-check-circle"></i>
                        </span>
                        <div>
                            <div class="text-sm font-semibold text-success-300">
                                "Tidak ada bottleneck"
                            </div>
                            <div class="mt-0.5 text-xs text-success-200/70">
                                "Semua workflow state berjalan dalam batas SLA."
                            </div>
                        </div>
                    </div>
                }.into_any()
            }}
        </SectionCard>

        {has_state_distribution.then(|| view! {
            <SectionCard
                title="Distribusi per State"
                description="Jumlah workflow aktif di setiap state."
                icon="fas fa-chart-pie"
            >
                <StateDistributionGrid by_state=by_state />
            </SectionCard>
        })}

        <SectionCard
            title="Workflows Aktif"
            description="Daftar instansi workflow yang sedang berjalan."
            icon="fas fa-list"
        >
            {if workflows.is_empty() {
                view! {
                    <EmptyState
                        title="Tidak ada workflow aktif".to_string()
                        description="Semua workflow telah selesai atau belum ada yang dimulai.".to_string()
                        icon="fas fa-inbox".to_string()
                    />
                }.into_any()
            } else {
                view! { <WorkflowStatusTable workflows=workflows /> }.into_any()
            }}
        </SectionCard>
    }
}
