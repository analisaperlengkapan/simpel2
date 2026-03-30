//! Dashboard page - Main user dashboard after login
//!
//! Shows real data from /api/v1/iam/admin/stats (admin) or user session info.

use crate::components::layout::MainLayout;
use crate::features::auth::{AuthService, UserSession};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// ============================================================================
// API DATA STRUCTURES (matching real backend responses)
// ============================================================================

/// Admin stats from /api/v1/iam/admin/stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdminStats {
    #[serde(default)]
    pub total_users: i64,
    #[serde(default)]
    pub active_users: i64,
    #[serde(default)]
    pub total_sessions: i64,
    #[serde(default)]
    pub active_sessions: i64,
    #[serde(default)]
    pub total_realms: i64,
    #[serde(default)]
    pub total_policies: i64,
    #[serde(default)]
    pub security_events_today: i64,
    #[serde(default)]
    pub failed_login_attempts: i64,
}

/// Profile from /api/v1/auth/me
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct MeResponse {
    #[serde(default)]
    id: String,
    #[serde(default)]
    username: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    role: String,
    #[serde(default)]
    mfa_enabled: bool,
    #[serde(default)]
    email_verified: bool,
}

async fn fetch_admin_stats() -> Result<AdminStats, String> {
    let token = AuthService::get_token().ok_or("Not authenticated")?;
    let resp = gloo_net::http::Request::get("/api/v1/iam/admin/stats")
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<AdminStats>().await.map_err(|e| e.to_string())
}

async fn fetch_me() -> Result<MeResponse, String> {
    let token = AuthService::get_token().ok_or("Not authenticated")?;
    let resp = gloo_net::http::Request::get("/api/v1/auth/me")
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<MeResponse>().await.map_err(|e| e.to_string())
}

/// Dashboard page component
#[component]
pub fn DashboardPage(user_session: UserSession, on_logout: Box<dyn Fn()>) -> impl IntoView {
    let is_admin = user_session.role.is_admin();
    let admin_stats = LocalResource::new(move || async move {
        if is_admin {
            fetch_admin_stats().await.ok()
        } else {
            None
        }
    });
    let me_data = LocalResource::new(move || async move { fetch_me().await.ok() });

    let greeting = {
        #[cfg(target_arch = "wasm32")]
        {
            let hour = js_sys::Date::new_0().get_hours();
            match hour {
                0..=11 => "Selamat Pagi",
                12..=14 => "Selamat Siang",
                15..=17 => "Selamat Sore",
                _ => "Selamat Malam",
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            "Selamat Datang"
        }
    };

    view! {
        <MainLayout user_session=user_session.clone() on_logout=on_logout>
            <div class="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-6 space-y-6">

                // ── Welcome Header
                <div class="bg-gradient-to-r from-navy-800 to-navy-900 rounded-xl p-6 text-white">
                    <p class="text-gold-300 text-sm font-medium">{greeting}</p>
                    <h1 class="text-2xl font-bold mt-1">{user_session.name.clone()}</h1>
                    <div class="flex flex-wrap gap-2 mt-3">
                        <span class="inline-flex items-center gap-1.5 px-2.5 py-1 bg-white/15 rounded-full text-xs font-medium">
                            <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 20 20">
                                <path fill-rule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clip-rule="evenodd"/>
                            </svg>
                            {user_session.role.display_name()}
                        </span>
                        {user_session.nip.clone().map(|nip| view! {
                            <span class="inline-flex items-center gap-1.5 px-2.5 py-1 bg-white/15 rounded-full text-xs font-medium">
                                "NIP: " {nip}
                            </span>
                        })}
                        {user_session.jabatan.clone().map(|j| view! {
                            <span class="inline-flex items-center gap-1.5 px-2.5 py-1 bg-white/15 rounded-full text-xs font-medium">
                                {j}
                            </span>
                        })}
                    </div>
                </div>

                // ── Account Info
                <Suspense fallback=move || view! { <LoadingSkeleton count=3 /> }>
                    {move || {
                        me_data.get().map(|me_opt| {
                            if let Some(me) = me_opt {
                                view! {
                                    <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                                        <InfoCard
                                            label="Email"
                                            value=me.email.clone()
                                            icon_path="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
                                        />
                                        <InfoCard
                                            label="MFA"
                                            value=if me.mfa_enabled { "Aktif".to_string() } else { "Belum Aktif".to_string() }
                                            icon_path="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
                                            accent=if me.mfa_enabled { "green" } else { "amber" }
                                        />
                                        <InfoCard
                                            label="Email Verifikasi"
                                            value=if me.email_verified { "Terverifikasi".to_string() } else { "Belum".to_string() }
                                            icon_path="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                                            accent=if me.email_verified { "green" } else { "amber" }
                                        />
                                    </div>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }
                        })
                    }}
                </Suspense>

                // ── Admin Stats (admin only)
                {if is_admin {
                    Some(view! {
                        <Suspense fallback=move || view! { <LoadingSkeleton count=4 /> }>
                            {move || {
                                admin_stats.get().map(|stats_opt| {
                                    if let Some(stats) = stats_opt {
                                        view! {
                                            <section>
                                                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-3">
                                                    "Statistik Sistem"
                                                </h2>
                                                <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
                                                    <StatCard label="Total Pengguna" value=stats.total_users color="blue" />
                                                    <StatCard label="Pengguna Aktif" value=stats.active_users color="green" />
                                                    <StatCard label="Sesi Aktif" value=stats.active_sessions color="indigo" />
                                                    <StatCard label="Total Realm" value=stats.total_realms color="purple" />
                                                </div>

                                                {(stats.security_events_today > 0 || stats.failed_login_attempts > 0).then(|| view! {
                                                    <div class="grid grid-cols-2 gap-4 mt-4">
                                                        <StatCard label="Event Keamanan Hari Ini" value=stats.security_events_today color="amber" />
                                                        <StatCard label="Login Gagal" value=stats.failed_login_attempts color="red" />
                                                    </div>
                                                })}
                                            </section>
                                        }.into_any()
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }
                                })
                            }}
                        </Suspense>
                    })
                } else {
                    None
                }}

                // ── Quick Actions
                <section>
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-3">
                        "Menu Utama"
                    </h2>
                    <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
                        <QuickLink href="/portal/apps" label="Aplikasi" desc="Akses layanan" color="blue"
                            icon_path="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zm10 0a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zm10 0a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"
                        />
                        <QuickLink href="/portal/profile" label="Profil" desc="Data pegawai" color="green"
                            icon_path="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                        />
                        <QuickLink href="/portal/settings" label="Pengaturan" desc="Preferensi akun" color="purple"
                            icon_path="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                        />
                        <QuickLink href="/portal/notifications" label="Notifikasi" desc="Pemberitahuan" color="amber"
                            icon_path="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
                        />
                    </div>
                </section>
            </div>
        </MainLayout>
    }
}

// ── Helper Components ────────────────────────────────────────────────────────

#[component]
fn StatCard(label: &'static str, value: i64, color: &'static str) -> impl IntoView {
    let bg = match color {
        "blue" => "bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800",
        "green" => "bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800",
        "indigo" => "bg-indigo-50 dark:bg-indigo-900/20 border-indigo-200 dark:border-indigo-800",
        "purple" => "bg-purple-50 dark:bg-purple-900/20 border-purple-200 dark:border-purple-800",
        "amber" => "bg-amber-50 dark:bg-amber-900/20 border-amber-200 dark:border-amber-800",
        "red" => "bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800",
        _ => "bg-gray-50 dark:bg-gray-900/20 border-gray-200 dark:border-gray-800",
    };

    view! {
        <div class=format!("rounded-lg border p-4 {}", bg)>
            <p class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">{label}</p>
            <p class="text-2xl font-bold text-gray-900 dark:text-white mt-1">{value}</p>
        </div>
    }
}

#[component]
fn InfoCard(
    label: &'static str,
    value: String,
    icon_path: &'static str,
    #[prop(optional)] accent: Option<&'static str>,
) -> impl IntoView {
    let accent = accent.unwrap_or("blue");
    let (text_color, bg_color) = match accent {
        "green" => (
            "text-green-600 dark:text-green-400",
            "bg-green-50 dark:bg-green-900/20",
        ),
        "amber" => (
            "text-amber-600 dark:text-amber-400",
            "bg-amber-50 dark:bg-amber-900/20",
        ),
        _ => (
            "text-blue-600 dark:text-blue-400",
            "bg-blue-50 dark:bg-blue-900/20",
        ),
    };

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4 flex items-center gap-3">
            <div class=format!("w-10 h-10 rounded-lg flex items-center justify-center flex-shrink-0 {}", bg_color)>
                <svg class=format!("w-5 h-5 {}", text_color) fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d=icon_path />
                </svg>
            </div>
            <div class="min-w-0">
                <p class="text-xs text-gray-500 dark:text-gray-400">{label}</p>
                <p class="text-sm font-semibold text-gray-900 dark:text-white truncate">{value}</p>
            </div>
        </div>
    }
}

#[component]
fn QuickLink(
    href: &'static str,
    label: &'static str,
    desc: &'static str,
    color: &'static str,
    icon_path: &'static str,
) -> impl IntoView {
    let icon_bg = match color {
        "blue" => "bg-blue-500",
        "green" => "bg-green-500",
        "purple" => "bg-purple-500",
        "amber" => "bg-amber-500",
        _ => "bg-gray-500",
    };

    view! {
        <a href=href class="flex items-center gap-3 p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-navy-300 dark:hover:border-gold-600 hover:shadow-md transition-all group">
            <div class=format!("w-10 h-10 rounded-lg flex items-center justify-center flex-shrink-0 {}", icon_bg)>
                <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d=icon_path />
                </svg>
            </div>
            <div class="min-w-0 flex-1">
                <p class="text-sm font-semibold text-gray-900 dark:text-white">{label}</p>
                <p class="text-xs text-gray-500 dark:text-gray-400">{desc}</p>
            </div>
            <svg class="w-4 h-4 text-gray-400 group-hover:text-navy-600 dark:group-hover:text-gold-400 transition-colors flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
            </svg>
        </a>
    }
}

#[component]
fn LoadingSkeleton(#[prop(optional, default = 3)] count: usize) -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
            {(0..count).map(|_| view! {
                <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4 animate-pulse">
                    <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-20 mb-2"></div>
                    <div class="h-6 bg-gray-200 dark:bg-gray-700 rounded w-12"></div>
                </div>
            }).collect_view()}
        </div>
    }
}
