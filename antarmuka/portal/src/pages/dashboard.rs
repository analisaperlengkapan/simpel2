//! Dashboard page - Main user dashboard after login

use crate::components::layout::MainLayout;
use crate::features::auth::{AuthService, UserSession};
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use lib_ui::components::dashboard::{BarChart, MetricCard};
use lib_ui::core::types::ChartDataPoint;
use serde::{Deserialize, Serialize};

// ============================================================================
// PORTAL DASHBOARD METRICS DATA STRUCTURES
// ============================================================================

/// Portal dashboard metrics response from backend
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortalDashboardMetrics {
    #[serde(default)]
    pub system: SystemDashboardMetrics,
    #[serde(default)]
    pub cross_domain: CrossDomainMetrics,
    #[serde(default)]
    pub auth: AuthMetrics,
    #[serde(default)]
    pub integration_health: IntegrationHealth,
    #[serde(default)]
    pub collected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemDashboardMetrics {
    pub total_users: i64,
    pub active_sessions: i64,
    pub uptime_seconds: i64,
    pub server_started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrossDomainMetrics {
    pub total_documents: i64,
    pub total_notifications: i64,
    pub api_calls_24h: i64,
    pub documents_by_status: Vec<StatusCount>,
    pub notifications_by_channel: Vec<ChannelCount>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthMetrics {
    pub login_attempts_24h: i64,
    pub successful_logins_24h: i64,
    pub failed_logins_24h: i64,
    pub mfa_enabled_users: i64,
    pub sessions_by_role: Vec<RoleCount>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntegrationHealth {
    pub siman: ServiceStatus,
    pub mysimkari: ServiceStatus,
    pub monsakti: Option<ServiceStatus>,
    pub overall_status: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub last_sync_duration_ms: Option<i64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusCount {
    pub status: String,
    pub count: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelCount {
    pub channel: String,
    pub count: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoleCount {
    pub role: String,
    pub count: i64,
}

/// Fetch portal dashboard metrics from backend
/// Uses IAM admin stats endpoint for dashboard data
async fn get_portal_dashboard_metrics() -> Result<PortalDashboardMetrics, String> {
    let token = AuthService::get_token().ok_or("Not authenticated")?;

    let resp = gloo_net::http::Request::get("/api/v1/iam/admin/stats")
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        // Fallback: return empty metrics if admin endpoint not accessible
        return Ok(PortalDashboardMetrics::default());
    }

    resp.json::<PortalDashboardMetrics>()
        .await
        .map_err(|e| e.to_string())
}

/// Dashboard page component - main user dashboard with statistics
#[component]
pub fn DashboardPage(
    /// Current user session data
    user_session: UserSession,
    /// Callback function to handle user logout
    on_logout: Box<dyn Fn()>,
) -> impl IntoView {
    // Resource to fetch portal dashboard metrics
    let portal_metrics =
        LocalResource::new(move || async move { get_portal_dashboard_metrics().await });

    // Get current time for greeting
    let greeting = {
        #[cfg(target_arch = "wasm32")]
        {
            let hour = js_sys::Date::new_0().get_hours();
            if hour < 12 {
                "Selamat Pagi"
            } else if hour < 15 {
                "Selamat Siang"
            } else if hour < 18 {
                "Selamat Sore"
            } else {
                "Selamat Malam"
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            "Selamat Datang"
        }
    };

    view! {
        <MainLayout user_session=user_session.clone() on_logout=on_logout>
            <div class="container mx-auto px-4 py-8">
                // Welcome Section
                <div class="relative bg-gradient-to-r from-navy-700 via-navy-800 to-navy-900 dark:from-navy-800 dark:to-navy-950 rounded-2xl shadow-2xl p-8 mb-8 text-white overflow-hidden border border-navy-600">
                    <div class="absolute inset-0 opacity-10">
                        <div class="absolute inset-0" style="background-image: radial-gradient(circle at 2px 2px, white 1px, transparent 0); background-size: 40px 40px;"></div>
                    </div>

                    <div class="relative flex flex-col md:flex-row items-center justify-between">
                        <div class="flex-1 mb-4 md:mb-0">
                            <div class="flex items-center gap-3 mb-3">
                                <div class="w-16 h-16 bg-white/30 rounded-full flex items-center justify-center">
                                    <span class="text-3xl">"👤"</span>
                                </div>
                                <div>
                                    <p class="text-gold-200 text-sm font-medium">{greeting}</p>
                                    <h1 class="text-3xl md:text-4xl font-bold">
                                        {user_session.name.clone()}
                                    </h1>
                                </div>
                            </div>
                            <div class="flex flex-wrap gap-3">
                                <span class="inline-flex items-center px-3 py-1 bg-white/30 rounded-full text-sm font-medium">
                                    <svg class="w-4 h-4 mr-2" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clip-rule="evenodd"/>
                                    </svg>
                                    {user_session.role.display_name()}
                                </span>
                                <span class="inline-flex items-center px-3 py-1 bg-white/30 rounded-full text-sm font-medium">
                                    <svg class="w-4 h-4 mr-2" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M4 4a2 2 0 012-2h8a2 2 0 012 2v12a1 1 0 110 2h-3a1 1 0 01-1-1v-2a1 1 0 00-1-1H9a1 1 0 00-1 1v2a1 1 0 01-1 1H4a1 1 0 110-2V4zm3 1h2v2H7V5zm2 4H7v2h2V9zm2-4h2v2h-2V5zm2 4h-2v2h2V9z" clip-rule="evenodd"/>
                                    </svg>
                                    {user_session.division.clone()}
                                </span>
                            </div>
                        </div>
                        <div class="hidden md:block">
                            <div class="text-8xl opacity-50">"🏛️"</div>
                        </div>
                    </div>
                </div>

                // Portal Dashboard Metrics
                <Suspense fallback=move || view! {
                    <div class="flex items-center justify-center py-8 text-gray-400 dark:text-gray-500 gap-3">
                        <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-gold-500"></div>
                        <span class="text-sm">"Memuat metrik..."</span>
                    </div>
                }>
                    {move || {
                        portal_metrics.get().map(|result| match result {
                            Ok(metrics) => view! {
                                <div class="space-y-8">
                                    // System Metrics Cards
                                    <div>
                                        <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-4 flex items-center">
                                            <span class="text-3xl mr-3">"📊"</span>
                                            "Metrik Sistem"
                                        </h2>
                                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                                            <MetricCard
                                                title="Total Pengguna".to_string()
                                                value=metrics.system.total_users.to_string()
                                                icon="👥".to_string()
                                                subtitle="Pengguna terdaftar".to_string()
                                            />
                                            <MetricCard
                                                title="Sesi Aktif".to_string()
                                                value=metrics.system.active_sessions.to_string()
                                                icon="🔐".to_string()
                                                subtitle="Pengguna online".to_string()
                                            />
                                            <MetricCard
                                                title="Uptime".to_string()
                                                value=format!("{}h", metrics.system.uptime_seconds / 3600)
                                                icon="⏱️".to_string()
                                                subtitle="Sejak restart".to_string()
                                            />
                                            <MetricCard
                                                title="MFA Enabled".to_string()
                                                value=metrics.auth.mfa_enabled_users.to_string()
                                                icon="🛡️".to_string()
                                                subtitle="Pengguna dengan MFA".to_string()
                                            />
                                        </div>
                                    </div>

                                    // Cross-Domain Metrics
                                    <div>
                                        <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-4 flex items-center">
                                            <span class="text-3xl mr-3">"📁"</span>
                                            "Metrik Lintas Domain"
                                        </h2>
                                        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                                            <MetricCard
                                                title="Total Dokumen".to_string()
                                                value=metrics.cross_domain.total_documents.to_string()
                                                icon="📄".to_string()
                                                subtitle="Dokumen tersimpan".to_string()
                                            />
                                            <MetricCard
                                                title="Notifikasi".to_string()
                                                value=metrics.cross_domain.total_notifications.to_string()
                                                icon="🔔".to_string()
                                                subtitle="Total notifikasi".to_string()
                                            />
                                            <MetricCard
                                                title="API Calls (24h)".to_string()
                                                value=metrics.cross_domain.api_calls_24h.to_string()
                                                icon="🔌".to_string()
                                                subtitle="Panggilan API".to_string()
                                            />
                                        </div>
                                    </div>

                                    // Charts Section
                                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                                        // Documents by Status Chart
                                        {
                                            let docs_data = metrics.cross_domain.documents_by_status.iter().map(|s| {
                                                ChartDataPoint::new(s.status.clone(), s.count as f64)
                                                    .with_color("bg-blue-500".to_string())
                                            }).collect::<Vec<_>>();

                                            view! {
                                                <BarChart
                                                    title="Dokumen per Status".to_string()
                                                    data=docs_data
                                                    show_values=true
                                                    height=250
                                                />
                                            }
                                        }

                                        // Notifications by Channel Chart
                                        {
                                            let notif_data = metrics.cross_domain.notifications_by_channel.iter().map(|c| {
                                                ChartDataPoint::new(c.channel.clone(), c.count as f64)
                                                    .with_color("bg-purple-500".to_string())
                                            }).collect::<Vec<_>>();

                                            view! {
                                                <BarChart
                                                    title="Notifikasi per Channel".to_string()
                                                    data=notif_data
                                                    show_values=true
                                                    height=250
                                                />
                                            }
                                        }
                                    </div>

                                    // Authentication Metrics
                                    <div>
                                        <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-4 flex items-center">
                                            <span class="text-3xl mr-3">"🔐"</span>
                                            "Metrik Autentikasi (24 Jam)"
                                        </h2>
                                        <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
                                            <MetricCard
                                                title="Login Attempts".to_string()
                                                value=metrics.auth.login_attempts_24h.to_string()
                                                icon="🔑".to_string()
                                                subtitle="Total percobaan".to_string()
                                            />
                                            {
                                                if metrics.auth.login_attempts_24h > 0 {
                                                    let change_val = (metrics.auth.successful_logins_24h as f64 / metrics.auth.login_attempts_24h as f64) * 100.0;
                                                    view! {
                                                        <MetricCard
                                                            title="Successful Logins".to_string()
                                                            value=metrics.auth.successful_logins_24h.to_string()
                                                            icon="✅".to_string()
                                                            subtitle="Login berhasil".to_string()
                                                            change=change_val
                                                        />
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <MetricCard
                                                            title="Successful Logins".to_string()
                                                            value=metrics.auth.successful_logins_24h.to_string()
                                                            icon="✅".to_string()
                                                            subtitle="Login berhasil".to_string()
                                                        />
                                                    }.into_any()
                                                }
                                            }
                                            {
                                                if metrics.auth.login_attempts_24h > 0 {
                                                    let change_val = -((metrics.auth.failed_logins_24h as f64 / metrics.auth.login_attempts_24h as f64) * 100.0);
                                                    view! {
                                                        <MetricCard
                                                            title="Failed Logins".to_string()
                                                            value=metrics.auth.failed_logins_24h.to_string()
                                                            icon="❌".to_string()
                                                            subtitle="Login gagal".to_string()
                                                            change=change_val
                                                        />
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <MetricCard
                                                            title="Failed Logins".to_string()
                                                            value=metrics.auth.failed_logins_24h.to_string()
                                                            icon="❌".to_string()
                                                            subtitle="Login gagal".to_string()
                                                        />
                                                    }.into_any()
                                                }
                                            }
                                        </div>

                                        // Sessions by Role Chart
                                        {
                                            let role_data = metrics.auth.sessions_by_role.iter().map(|r| {
                                                ChartDataPoint::new(r.role.clone(), r.count as f64)
                                                    .with_color("bg-green-500".to_string())
                                            }).collect::<Vec<_>>();

                                            view! {
                                                <BarChart
                                                    title="Sesi Aktif per Role".to_string()
                                                    data=role_data
                                                    show_values=true
                                                    height=250
                                                />
                                            }
                                        }
                                    </div>

                                    // Integration Health Status
                                    <div>
                                        <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-4 flex items-center">
                                            <span class="text-3xl mr-3">"🔗"</span>
                                            "Status Integrasi"
                                        </h2>
                                        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                                            // SIMAN Status
                                            <IntegrationStatusCard service=metrics.integration_health.siman.clone() />

                                            // MySIMKARI Status
                                            <IntegrationStatusCard service=metrics.integration_health.mysimkari.clone() />

                                            // MonSAKTI Status (optional)
                                            {metrics.integration_health.monsakti.as_ref().map(|service| view! {
                                                <IntegrationStatusCard service=service.clone() />
                                            })}
                                        </div>
                                    </div>

                                    // Quick Actions (inside metrics for layout consistency)
                                    <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-lg p-6 border border-gray-100 dark:border-gray-700">
                                        <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-6 flex items-center">
                                            <div class="w-10 h-10 bg-gradient-to-br from-blue-500 to-blue-600 rounded-lg flex items-center justify-center mr-3">
                                                <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 20 20">
                                                    <path d="M11 3a1 1 0 10-2 0v1a1 1 0 102 0V3zM15.657 5.757a1 1 0 00-1.414-1.414l-.707.707a1 1 0 001.414 1.414l.707-.707zM18 10a1 1 0 01-1 1h-1a1 1 0 110-2h1a1 1 0 011 1zM5.05 6.464A1 1 0 106.464 5.05l-.707-.707a1 1 0 00-1.414 1.414l.707.707zM5 10a1 1 0 01-1 1H3a1 1 0 110-2h1a1 1 0 011 1zM8 16v-1h4v1a2 2 0 11-4 0zM12 14c.015-.34.208-.646.477-.859a4 4 0 10-4.954 0c.27.213.462.519.476.859h4.002z"/>
                                                </svg>
                                            </div>
                                            "Aksi Cepat"
                                        </h3>
                                        <QuickActions />
                                    </div>
                                </div>
                            }.into_any(),
                            Err(e) => view! {
                                // Show Quick Actions even when metrics fail
                                <div class="space-y-6">
                                    <div class="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg p-4 flex items-start gap-3">
                                        <span class="text-xl mt-0.5">"⚠️"</span>
                                        <div>
                                            <p class="text-amber-800 dark:text-amber-300 font-semibold text-sm">"Metrik tidak tersedia"</p>
                                            <p class="text-amber-600 dark:text-amber-400 text-xs mt-1">{e}</p>
                                        </div>
                                    </div>
                                    <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-lg p-6 border border-gray-100 dark:border-gray-700">
                                        <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-6 flex items-center">
                                            <div class="w-10 h-10 bg-gradient-to-br from-blue-500 to-blue-600 rounded-lg flex items-center justify-center mr-3">
                                                <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 20 20">
                                                    <path d="M11 3a1 1 0 10-2 0v1a1 1 0 102 0V3zM15.657 5.757a1 1 0 00-1.414-1.414l-.707.707a1 1 0 001.414 1.414l.707-.707zM18 10a1 1 0 01-1 1h-1a1 1 0 110-2h1a1 1 0 011 1zM5.05 6.464A1 1 0 106.464 5.05l-.707-.707a1 1 0 00-1.414 1.414l.707.707zM5 10a1 1 0 01-1 1H3a1 1 0 110-2h1a1 1 0 011 1zM8 16v-1h4v1a2 2 0 11-4 0zM12 14c.015-.34.208-.646.477-.859a4 4 0 10-4.954 0c.27.213.462.519.476.859h4.002z"/>
                                                </svg>
                                            </div>
                                            "Aksi Cepat"
                                        </h3>
                                        <QuickActions />
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

/// Quick action navigation buttons (always visible, no API dependency)
#[component]
fn QuickActions() -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 md:grid-cols-2 lg:grid-cols-4 gap-3">
            <a href="/portal/apps" class="block p-4 bg-gradient-to-r from-blue-50 to-blue-100 dark:from-blue-900/20 dark:to-blue-800/20 rounded-xl hover:shadow-md transition-all group border border-blue-200 dark:border-blue-800">
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-3">
                        <div class="w-10 h-10 bg-blue-500 rounded-lg flex items-center justify-center">
                            <span class="text-xl">"📱"</span>
                        </div>
                        <div>
                            <p class="font-semibold text-gray-900 dark:text-white">"Aplikasi"</p>
                            <p class="text-xs text-gray-600 dark:text-gray-400">"Akses sistem"</p>
                        </div>
                    </div>
                    <svg class="w-5 h-5 text-blue-600 dark:text-blue-400 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                    </svg>
                </div>
            </a>
            <a href="/portal/notifications" class="block p-4 bg-gradient-to-r from-purple-50 to-purple-100 dark:from-purple-900/20 dark:to-purple-800/20 rounded-xl hover:shadow-md transition-all group border border-purple-200 dark:border-purple-800">
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-3">
                        <div class="w-10 h-10 bg-purple-500 rounded-lg flex items-center justify-center">
                            <span class="text-xl">"🔔"</span>
                        </div>
                        <div>
                            <p class="font-semibold text-gray-900 dark:text-white">"Notifikasi"</p>
                            <p class="text-xs text-gray-600 dark:text-gray-400">"Pemberitahuan"</p>
                        </div>
                    </div>
                    <svg class="w-5 h-5 text-purple-600 dark:text-purple-400 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                    </svg>
                </div>
            </a>
            <a href="/portal/profile" class="block p-4 bg-gradient-to-r from-green-50 to-green-100 dark:from-green-900/20 dark:to-green-800/20 rounded-xl hover:shadow-md transition-all group border border-green-200 dark:border-green-800">
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-3">
                        <div class="w-10 h-10 bg-green-500 rounded-lg flex items-center justify-center">
                            <span class="text-xl">"👤"</span>
                        </div>
                        <div>
                            <p class="font-semibold text-gray-900 dark:text-white">"Profil"</p>
                            <p class="text-xs text-gray-600 dark:text-gray-400">"Data pegawai"</p>
                        </div>
                    </div>
                    <svg class="w-5 h-5 text-green-600 dark:text-green-400 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                    </svg>
                </div>
            </a>
            <a href="/portal/settings" class="block p-4 bg-gradient-to-r from-orange-50 to-orange-100 dark:from-orange-900/20 dark:to-orange-800/20 rounded-xl hover:shadow-md transition-all group border border-orange-200 dark:border-orange-800">
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-3">
                        <div class="w-10 h-10 bg-orange-500 rounded-lg flex items-center justify-center">
                            <span class="text-xl">"⚙️"</span>
                        </div>
                        <div>
                            <p class="font-semibold text-gray-900 dark:text-white">"Pengaturan"</p>
                            <p class="text-xs text-gray-600 dark:text-gray-400">"Kelola profil"</p>
                        </div>
                    </div>
                    <svg class="w-5 h-5 text-orange-600 dark:text-orange-400 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                    </svg>
                </div>
            </a>
        </div>
    }
}

/// Integration status card component
#[component]
fn IntegrationStatusCard(
    /// Service status data
    service: ServiceStatus,
) -> impl IntoView {
    let (status_color, status_icon, status_text) = match service.status.as_str() {
        "healthy" => (
            "bg-green-100 dark:bg-green-900/20 border-green-200 dark:border-green-800",
            "✅",
            "Sehat",
        ),
        "degraded" => (
            "bg-yellow-100 dark:bg-yellow-900/20 border-yellow-200 dark:border-yellow-800",
            "⚠️",
            "Terdegradasi",
        ),
        "down" => (
            "bg-red-100 dark:bg-red-900/20 border-red-200 dark:border-red-800", // Keep error state red
            "❌",
            "Mati",
        ),
        _ => (
            "bg-gray-100 dark:bg-gray-900/20 border-gray-200 dark:border-gray-800",
            "❓",
            "Tidak Diketahui",
        ),
    };

    view! {
        <div class=format!("rounded-lg p-6 border {}", status_color)>
            <div class="flex items-center justify-between mb-4">
                <h4 class="text-lg font-semibold text-gray-900 dark:text-white">
                    {service.name}
                </h4>
                <span class="text-2xl">{status_icon}</span>
            </div>

            <div class="space-y-2">
                <div class="flex justify-between text-sm">
                    <span class="text-gray-600 dark:text-gray-400">"Status:"</span>
                    <span class="font-medium text-gray-900 dark:text-white">{status_text}</span>
                </div>

                {service.last_sync.map(|sync_time| view! {
                    <div class="flex justify-between text-sm">
                        <span class="text-gray-600 dark:text-gray-400">"Last Sync:"</span>
                        <span class="font-medium text-gray-900 dark:text-white">
                            {sync_time.format("%H:%M:%S").to_string()}
                        </span>
                    </div>
                })}

                {service.last_sync_duration_ms.map(|duration| view! {
                    <div class="flex justify-between text-sm">
                        <span class="text-gray-600 dark:text-gray-400">"Duration:"</span>
                        <span class="font-medium text-gray-900 dark:text-white">
                            {format!("{}ms", duration)}
                        </span>
                    </div>
                })}

                {service.error_message.map(|error| view! {
                    <div class="mt-2 p-2 bg-red-50 dark:bg-red-900/30 rounded text-xs text-red-700 dark:text-red-300">
                        {error}
                    </div>
                })}
            </div>
        </div>
    }
}
