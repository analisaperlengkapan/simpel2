//! Portal Dashboard UI - System-wide metrics and monitoring
//!
//! Displays:
//! - System metrics (total users, active sessions, system health)
//! - Cross-domain metrics (kebutuhan, pemakaian, penghapusan counts)
//! - Auth metrics (login attempts, MFA usage)
//! - Integration health (SIMAN, MySIMKARI status)

use crate::components::layout::MainLayout;
use crate::features::auth::UserSession;
use leptos::prelude::*;
use lib_ui::components::dashboard::{
    BarChart, DashboardLoadingSkeleton, IntegrationStatusCard, MetricCard, PieChart,
};
use lib_ui::core::types::ChartDataPoint;
use serde::{Deserialize, Serialize};

// ============================================================================
// API DATA STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalDashboardMetrics {
    pub system_metrics: SystemMetrics,
    pub cross_domain_metrics: CrossDomainMetrics,
    pub auth_metrics: AuthMetrics,
    pub integration_health: IntegrationHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub total_users: i64,
    pub active_sessions: i64,
    pub system_health: String,
    pub uptime_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainMetrics {
    pub kebutuhan_total: i64,
    pub kebutuhan_approved: i64,
    pub kebutuhan_pending: i64,
    pub pemakaian_active: i64,
    pub pemakaian_expired: i64,
    pub penghapusan_total: i64,
    pub penghapusan_completed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthMetrics {
    pub login_attempts_today: i64,
    pub login_success_rate: f64,
    pub mfa_enabled_users: i64,
    pub mfa_usage_percentage: f64,
    pub failed_attempts_today: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationHealth {
    pub siman_status: String,
    pub siman_last_sync: String,
    pub mysimkari_status: String,
    pub mysimkari_last_sync: String,
}

// ============================================================================
// API FETCH FUNCTION
// ============================================================================

async fn fetch_portal_dashboard() -> Result<PortalDashboardMetrics, String> {
    let token = crate::features::auth::AuthService::get_token()
        .ok_or("Not authenticated")?;

    let resp = gloo_net::http::Request::get("/api/v1/dashboard/portal")
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("HTTP {}: {}", resp.status(), resp.status_text()));
    }

    resp.json::<PortalDashboardMetrics>()
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// MAIN COMPONENT
// ============================================================================

#[component]
pub fn PortalDashboardPage(
    user_session: UserSession,
    on_logout: Box<dyn Fn()>,
) -> impl IntoView {
    let metrics = LocalResource::new(|| async move { fetch_portal_dashboard().await });

    // Auto-refresh every 30 seconds
    let (refresh_trigger, set_refresh_trigger) = signal(0);

    #[cfg(target_arch = "wasm32")]
    {
        use gloo_timers::callback::Interval;
        use std::cell::RefCell;
        use std::rc::Rc;

        let interval = Rc::new(RefCell::new(None::<Interval>));
        let interval_clone = interval.clone();

        Effect::new(move |_| {
            let interval_handle = Interval::new(30_000, move || {
                set_refresh_trigger.update(|v| *v += 1);
            });
            *interval_clone.borrow_mut() = Some(interval_handle);
        });
    }

    view! {
        <MainLayout user_session=user_session.clone() on_logout=on_logout>
            <div class="mx-auto max-w-7xl space-y-8 px-4 py-6 sm:px-6 lg:px-8">
                // Header
                <div class="flex items-center justify-between">
                    <div>
                        <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">
                            "Dashboard Portal"
                        </h1>
                        <p class="mt-1 text-sm text-gray-600 dark:text-gray-400">
                            "Ringkasan sistem dan integrasi"
                        </p>
                    </div>
                    <button
                        on:click=move |_| set_refresh_trigger.update(|v| *v += 1)
                        class="inline-flex items-center gap-2 rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-700 focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:ring-offset-2"
                    >
                        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                        </svg>
                        "Refresh"
                    </button>
                </div>

                <Suspense fallback=move || view! { <DashboardLoadingSkeleton /> }>
                    {move || {
                        let _ = refresh_trigger.get();
                        metrics.get().map(|result| match result {
                            Ok(data) => view! {
                                <div class="space-y-8">
                                    // System Overview Section
                                    <SystemOverviewSection metrics=data.system_metrics.clone() />

                                    // BMN Metrics Section
                                    <BmnMetricsSection metrics=data.cross_domain_metrics.clone() />

                                    // Auth Metrics Section
                                    <AuthMetricsSection metrics=data.auth_metrics.clone() />

                                    // Integration Health Section
                                    <IntegrationHealthSection health=data.integration_health.clone() />
                                </div>
                            }.into_any(),
                            Err(e) => view! {
                                <div class="rounded-lg border border-red-200 bg-red-50 p-4 dark:border-red-800 dark:bg-red-900/20">
                                    <div class="flex items-center gap-3">
                                        <svg class="h-5 w-5 text-red-600 dark:text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                        </svg>
                                        <div>
                                            <p class="font-medium text-red-800 dark:text-red-200">"Gagal memuat data dashboard"</p>
                                            <p class="text-sm text-red-600 dark:text-red-400">{e}</p>
                                        </div>
                                    </div>
                                </div>
                            }.into_any(),
                        })
                    }}
                </Suspense>
            </div>
        </MainLayout>
    }
}

// ============================================================================
// SECTION COMPONENTS
// ============================================================================

#[component]
fn SystemOverviewSection(metrics: SystemMetrics) -> impl IntoView {
    let health_color = match metrics.system_health.as_str() {
        "healthy" => "text-green-600 dark:text-green-400",
        "degraded" => "text-yellow-600 dark:text-yellow-400",
        _ => "text-red-600 dark:text-red-400",
    };

    view! {
        <section>
            <h2 class="mb-4 text-xl font-semibold text-gray-900 dark:text-gray-100">
                "Ringkasan Sistem"
            </h2>
            <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
                <MetricCard
                    title="Total Pengguna"
                    value=metrics.total_users.to_string()
                    icon="👥"
                />
                <MetricCard
                    title="Sesi Aktif"
                    value=metrics.active_sessions.to_string()
                    icon="🔐"
                />
                <MetricCard
                    title="Status Sistem"
                    value=metrics.system_health.clone()
                    icon="💚"
                    class=health_color
                />
                <MetricCard
                    title="Uptime"
                    value=format!("{:.2}%", metrics.uptime_percentage)
                    icon="⏱️"
                />
            </div>
        </section>
    }
}

#[component]
fn BmnMetricsSection(metrics: CrossDomainMetrics) -> impl IntoView {
    let kebutuhan_data = vec![
        ChartDataPoint::new("Disetujui", metrics.kebutuhan_approved as f64)
            .with_color("bg-green-500".to_string()),
        ChartDataPoint::new("Pending", metrics.kebutuhan_pending as f64)
            .with_color("bg-yellow-500".to_string()),
    ];

    let domain_data = vec![
        ChartDataPoint::new("Kebutuhan", metrics.kebutuhan_total as f64)
            .with_color("#10b981".to_string()),
        ChartDataPoint::new("Pemakaian Aktif", metrics.pemakaian_active as f64)
            .with_color("#3b82f6".to_string()),
        ChartDataPoint::new("Penghapusan", metrics.penghapusan_total as f64)
            .with_color("#ef4444".to_string()),
    ];

    view! {
        <section>
            <h2 class="mb-4 text-xl font-semibold text-gray-900 dark:text-gray-100">
                "Metrik BMN"
            </h2>
            <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
                // Kebutuhan Status
                <BarChart
                    title="Status Kebutuhan BMN"
                    data=kebutuhan_data
                    show_values=true
                    height=250
                />

                // Cross-Domain Overview
                <PieChart
                    title="Distribusi Lintas Domain"
                    data=domain_data
                    show_legend=true
                    size=200
                />
            </div>

            // Additional Metrics Cards
            <div class="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-3">
                <MetricCard
                    title="Pemakaian Aktif"
                    value=metrics.pemakaian_active.to_string()
                    icon="📋"
                    subtitle=format!("{} kadaluarsa", metrics.pemakaian_expired)
                />
                <MetricCard
                    title="Penghapusan Total"
                    value=metrics.penghapusan_total.to_string()
                    icon="🗑️"
                    subtitle=format!("{} selesai", metrics.penghapusan_completed)
                />
                <MetricCard
                    title="Kebutuhan Total"
                    value=metrics.kebutuhan_total.to_string()
                    icon="📊"
                    subtitle=format!("{} disetujui", metrics.kebutuhan_approved)
                />
            </div>
        </section>
    }
}

#[component]
fn AuthMetricsSection(metrics: AuthMetrics) -> impl IntoView {
    let auth_data = vec![
        ChartDataPoint::new("Login Berhasil", metrics.login_success_rate)
            .with_color("bg-green-500".to_string()),
        ChartDataPoint::new("Login Gagal", 100.0 - metrics.login_success_rate)
            .with_color("bg-red-500".to_string()),
    ];

    view! {
        <section>
            <h2 class="mb-4 text-xl font-semibold text-gray-900 dark:text-gray-100">
                "Metrik Autentikasi"
            </h2>
            <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
                <MetricCard
                    title="Login Hari Ini"
                    value=metrics.login_attempts_today.to_string()
                    icon="🔑"
                />
                <MetricCard
                    title="Tingkat Keberhasilan"
                    value=format!("{:.1}%", metrics.login_success_rate)
                    icon="✅"
                />
                <MetricCard
                    title="Pengguna MFA"
                    value=metrics.mfa_enabled_users.to_string()
                    icon="🛡️"
                    subtitle=format!("{:.1}% dari total", metrics.mfa_usage_percentage)
                />
                <MetricCard
                    title="Login Gagal"
                    value=metrics.failed_attempts_today.to_string()
                    icon="⚠️"
                    subtitle="hari ini"
                    class="text-red-600 dark:text-red-400"
                />
            </div>

            <div class="mt-4">
                <BarChart
                    title="Distribusi Login"
                    data=auth_data
                    max_value=100.0
                    show_values=true
                    height=200
                />
            </div>
        </section>
    }
}

#[component]
fn IntegrationHealthSection(health: IntegrationHealth) -> impl IntoView {
    view! {
        <section>
            <h2 class="mb-4 text-xl font-semibold text-gray-900 dark:text-gray-100">
                "Status Integrasi"
            </h2>
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                <IntegrationStatusCard
                    name="SIMAN"
                    description="Sistem Informasi Manajemen Aset Negara"
                    status=health.siman_status.clone()
                    last_sync=health.siman_last_sync.clone()
                    icon_container_class="bg-blue-100 dark:bg-blue-900"
                    icon_color_class="text-blue-600 dark:text-blue-400"
                    icon_path="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4"
                />

                <IntegrationStatusCard
                    name="MySIMKARI"
                    description="Sistem Informasi Manajemen Kepegawaian"
                    status=health.mysimkari_status.clone()
                    last_sync=health.mysimkari_last_sync.clone()
                    icon_container_class="bg-purple-100 dark:bg-purple-900"
                    icon_color_class="text-purple-600 dark:text-purple-400"
                    icon_path="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
                />
            </div>
        </section>
    }
}
