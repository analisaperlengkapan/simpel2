// Monitoring dashboard component for visualizing metrics, errors, and analytics
// Provides real-time alerts, log aggregation, and performance reports

use crate::components::icon::AppIcon;
use leptos::prelude::*;
use phosphor_leptos::{ARROW_CLOCKWISE, ARROW_DOWN, ARROW_UP};
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

use crate::components::feedback::Loading;
use crate::components::layout::{Card, Grid};
use crate::utils::monitoring::CoreWebVitals;

/// Metric card data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricCard {
    pub title: String,
    pub value: String,
    pub change: Option<f64>,
    pub status: MetricStatus,
}

/// Metric status
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MetricStatus {
    Good,
    Warning,
    Critical,
}

impl MetricStatus {
    pub fn color(&self) -> &'static str {
        match self {
            Self::Good => "text-green-600",
            Self::Warning => "text-yellow-600",
            Self::Critical => "text-red-600",
        }
    }

    pub fn bg_color(&self) -> &'static str {
        match self {
            Self::Good => "bg-green-50",
            Self::Warning => "bg-yellow-50",
            Self::Critical => "bg-red-50",
        }
    }
}

/// Error summary
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorSummary {
    pub id: String,
    pub message: String,
    pub count: u32,
    pub last_seen: String,
    pub severity: String,
}

/// Analytics summary
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub page_views: u64,
    pub unique_users: u64,
    pub avg_session_duration: f64,
    pub bounce_rate: f64,
}

/// Monitoring dashboard component
#[component]
pub fn MonitoringDashboard(
    #[prop(optional)] show_performance: bool,
    #[prop(optional)] show_errors: bool,
    #[prop(optional)] show_analytics: bool,
) -> impl IntoView {
    let (loading, set_loading) = signal(true);
    let (metrics, set_metrics) = signal(Vec::<MetricCard>::new());
    let (errors, set_errors) = signal(Vec::<ErrorSummary>::new());
    let (analytics, set_analytics) = signal(None::<AnalyticsSummary>);

    // Fetch monitoring data
    Effect::new(move |_| {
        spawn_local(async move {
            set_loading.set(true);

            // Fetch metrics
            if show_performance && let Ok(data) = fetch_performance_metrics().await {
                set_metrics.set(data);
            }

            // Fetch errors
            if show_errors && let Ok(data) = fetch_error_summary().await {
                set_errors.set(data);
            }

            // Fetch analytics
            if show_analytics && let Ok(data) = fetch_analytics_summary().await {
                set_analytics.set(Some(data));
            }

            set_loading.set(false);
        });
    });

    view! {
        <div class="monitoring-dashboard p-6 space-y-6">
            <div class="flex justify-between items-center">
                <h1 class="text-3xl font-bold text-gray-900">
                    "Monitoring Dashboard"
                </h1>
                <button
                    class="px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary-dark"
                    on:click=move |_| {
                        set_loading.set(true);
                        // Refresh data
                    }
                >
                    <span class="mr-2"><AppIcon icon=ARROW_CLOCKWISE /></span>
                    "Refresh"
                </button>
            </div>

            <Show
                when=move || loading.get()
                fallback=move || view! {
                    <div class="space-y-6">
                        {show_performance.then(|| view! {
                            <PerformanceSection metrics=metrics />
                        })}

                        {show_errors.then(|| view! {
                            <ErrorSection errors=errors />
                        })}

                        {show_analytics.then(|| view! {
                            <AnalyticsSection analytics=analytics />
                        })}
                    </div>
                }
            >
                <Loading />
            </Show>
        </div>
    }
}

/// Performance metrics section
#[component]
fn PerformanceSection(metrics: ReadSignal<Vec<MetricCard>>) -> impl IntoView {
    view! {
        <Card title="Performance Metrics".to_string()>
            <Grid cols=4>
                <For
                    each=move || metrics.get()
                    key=|metric| metric.title.clone()
                    children=move |metric: MetricCard| {
                        view! {
                            <MetricCardComponent metric=metric />
                        }
                    }
                />
            </Grid>

            <div class="mt-6">
                <h3 class="text-lg font-semibold mb-4">"Core Web Vitals"</h3>
                <CoreWebVitalsChart />
            </div>
        </Card>
    }
}

/// Metric card component
#[component]
fn MetricCardComponent(metric: MetricCard) -> impl IntoView {
    view! {
        <div class=format!("p-4 rounded-lg border {}", metric.status.bg_color())>
            <div class="text-sm text-gray-600 mb-1">{metric.title}</div>
            <div class=format!("text-2xl font-bold {}", metric.status.color())>
                {metric.value}
            </div>
            {metric.change.map(|change| {
                let is_positive = change > 0.0;
                let icon = if is_positive { ARROW_UP } else { ARROW_DOWN };
                let color = if is_positive { "text-green-600" } else { "text-red-600" };

                view! {
                    <div class=format!("text-sm mt-2 {}", color)>
                        <span class="mr-1 inline-flex">
                            <AppIcon icon=icon size=12 />
                        </span>
                        {format!("{:.1}%", change.abs())}
                    </div>
                }
            })}
        </div>
    }
}

/// Core Web Vitals chart
#[component]
fn CoreWebVitalsChart() -> impl IntoView {
    let (vitals, set_vitals) = signal(None::<CoreWebVitals>);

    Effect::new(move |_| {
        spawn_local(async move {
            if let Ok(data) = fetch_core_web_vitals().await {
                set_vitals.set(Some(data));
            }
        });
    });

    view! {
        <div class="grid grid-cols-3 gap-4">
            {move || vitals.get().map(|v| view! {
                <div class="space-y-2">
                    <VitalBar
                        label="LCP"
                        value=v.lcp.unwrap_or(0.0)
                        threshold=2500.0
                        unit="ms"
                    />
                    <VitalBar
                        label="FID"
                        value=v.fid.unwrap_or(0.0)
                        threshold=100.0
                        unit="ms"
                    />
                    <VitalBar
                        label="CLS"
                        value=v.cls.unwrap_or(0.0)
                        threshold=0.1
                        unit=""
                    />
                </div>
            })}
        </div>
    }
}

/// Vital bar component
#[component]
fn VitalBar(label: &'static str, value: f64, threshold: f64, unit: &'static str) -> impl IntoView {
    let percentage = (value / threshold * 100.0).min(100.0);
    let status = if value <= threshold {
        "bg-green-500"
    } else {
        "bg-red-500"
    };

    view! {
        <div>
            <div class="flex justify-between text-sm mb-1">
                <span class="font-medium">{label}</span>
                <span>{format!("{:.2}{}", value, unit)}</span>
            </div>
            <div class="w-full bg-gray-200 rounded-full h-2">
                <div
                    class=format!("h-2 rounded-full {}", status)
                    style=format!("width: {}%", percentage)
                ></div>
            </div>
        </div>
    }
}

/// Error section
#[component]
fn ErrorSection(errors: ReadSignal<Vec<ErrorSummary>>) -> impl IntoView {
    view! {
        <Card title="Recent Errors".to_string()>
            <div class="overflow-x-auto">
                <table class="min-w-full divide-y divide-gray-200">
                    <thead class="bg-gray-50">
                        <tr>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                                "Error"
                            </th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                                "Count"
                            </th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                                "Severity"
                            </th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                                "Last Seen"
                            </th>
                        </tr>
                    </thead>
                    <tbody class="bg-white divide-y divide-gray-200">
                        <For
                            each=move || errors.get()
                            key=|error| error.id.clone()
                            children=move |error: ErrorSummary| {
                                view! {
                                    <tr class="hover:bg-gray-50">
                                        <td class="px-6 py-4 text-sm text-gray-900">
                                            {error.message}
                                        </td>
                                        <td class="px-6 py-4 text-sm">
                                            <span class="px-2 py-1 bg-red-100 text-red-800 rounded text-xs font-medium">
                                                {error.count.to_string()}
                                            </span>
                                        </td>
                                        <td class="px-6 py-4 text-sm">
                                            <span class="px-2 py-1 bg-yellow-100 text-yellow-800 rounded text-xs font-medium">
                                                {error.severity}
                                            </span>
                                        </td>
                                        <td class="px-6 py-4 text-sm text-gray-500">
                                            {error.last_seen}
                                        </td>
                                    </tr>
                                }
                            }
                        />
                    </tbody>
                </table>
            </div>
        </Card>
    }
}

/// Analytics section
#[component]
fn AnalyticsSection(analytics: ReadSignal<Option<AnalyticsSummary>>) -> impl IntoView {
    view! {
        <Card title="Analytics Summary".to_string()>
            {move || analytics.get().map(|data| view! {
                <Grid cols=4>
                    <div class="p-4 bg-blue-50 rounded-lg">
                        <div class="text-sm text-gray-600 mb-1">"Page Views"</div>
                        <div class="text-2xl font-bold text-blue-600">
                            {format!("{}", data.page_views)}
                        </div>
                    </div>

                    <div class="p-4 bg-green-50 rounded-lg">
                        <div class="text-sm text-gray-600 mb-1">"Unique Users"</div>
                        <div class="text-2xl font-bold text-green-600">
                            {format!("{}", data.unique_users)}
                        </div>
                    </div>

                    <div class="p-4 bg-purple-50 rounded-lg">
                        <div class="text-sm text-gray-600 mb-1">"Avg Session"</div>
                        <div class="text-2xl font-bold text-purple-600">
                            {format!("{:.1}m", data.avg_session_duration / 60.0)}
                        </div>
                    </div>

                    <div class="p-4 bg-orange-50 rounded-lg">
                        <div class="text-sm text-gray-600 mb-1">"Bounce Rate"</div>
                        <div class="text-2xl font-bold text-orange-600">
                            {format!("{:.1}%", data.bounce_rate)}
                        </div>
                    </div>
                </Grid>
            })}
        </Card>
    }
}

// API functions

async fn fetch_performance_metrics() -> Result<Vec<MetricCard>, String> {
    // Mock data for now - replace with actual API call
    Ok(vec![
        MetricCard {
            title: "LCP".to_string(),
            value: "1.8s".to_string(),
            change: Some(-5.2),
            status: MetricStatus::Good,
        },
        MetricCard {
            title: "FID".to_string(),
            value: "85ms".to_string(),
            change: Some(2.1),
            status: MetricStatus::Good,
        },
        MetricCard {
            title: "CLS".to_string(),
            value: "0.08".to_string(),
            change: Some(-1.5),
            status: MetricStatus::Good,
        },
        MetricCard {
            title: "TTFB".to_string(),
            value: "520ms".to_string(),
            change: Some(8.3),
            status: MetricStatus::Warning,
        },
    ])
}

async fn fetch_error_summary() -> Result<Vec<ErrorSummary>, String> {
    // Mock data - replace with actual API call
    Ok(vec![
        ErrorSummary {
            id: "1".to_string(),
            message: "Network request failed".to_string(),
            count: 15,
            last_seen: "2 minutes ago".to_string(),
            severity: "error".to_string(),
        },
        ErrorSummary {
            id: "2".to_string(),
            message: "Undefined property access".to_string(),
            count: 3,
            last_seen: "1 hour ago".to_string(),
            severity: "warning".to_string(),
        },
    ])
}

async fn fetch_analytics_summary() -> Result<AnalyticsSummary, String> {
    // Mock data - replace with actual API call
    Ok(AnalyticsSummary {
        page_views: 12543,
        unique_users: 3421,
        avg_session_duration: 342.5,
        bounce_rate: 32.4,
    })
}

async fn fetch_core_web_vitals() -> Result<CoreWebVitals, String> {
    // Mock data - replace with actual API call
    Ok(CoreWebVitals {
        lcp: Some(1800.0),
        fid: Some(85.0),
        cls: Some(0.08),
        fcp: Some(1200.0),
        ttfb: Some(520.0),
    })
}
