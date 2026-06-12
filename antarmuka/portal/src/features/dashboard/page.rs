//! Dashboard page - Main user dashboard after login
//!
//! Shows real data from /api/v1/iam/admin/stats (admin) or user session info.

use crate::components::layout::MainLayout;
use crate::features::auth::{AuthService, UserSession};
use leptos::prelude::*;
use leptos_fetch::QueryClient;
use serde::{Deserialize, Serialize};

/// Admin-stats query — keyed by `is_admin: bool` so the non-admin
/// path resolves to `None` instantly without hitting the network,
/// and a single cache slot serves any admin user that opens the
/// dashboard.
async fn query_admin_stats(is_admin: bool) -> Option<AdminStats> {
    if is_admin {
        fetch_admin_stats().await.ok()
    } else {
        None
    }
}

/// `()`-keyed wrapper around `fetch_me()` so the dashboard, the
/// profile page, and any future consumer of /me share a single
/// in-flight request and a single cached payload.
async fn query_me(_: ()) -> Option<MeResponse> {
    fetch_me().await.ok()
}

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

/// Dashboard page component — reads session from context
#[component]
pub fn DashboardPage() -> impl IntoView {
    let user_session = use_context::<ReadSignal<Option<UserSession>>>()
        .and_then(|sig| sig.get_untracked())
        .unwrap_or_default();
    let is_admin = user_session.role.is_admin();
    let client: QueryClient = expect_context();
    let admin_stats = client.local_resource(query_admin_stats, move || is_admin);
    let me_data = client.local_resource(query_me, || ());

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
        <MainLayout>
            <div class="mx-auto max-w-7xl space-y-8 px-4 py-6 sm:px-6 lg:px-8">
                <section class="relative overflow-hidden rounded-3xl border border-white/10 bg-gradient-to-br from-navy-900 via-navy-800 to-slate-950 p-6 shadow-[0_18px_50px_rgba(0,0,0,0.45)] sm:p-8">
                    <div class="pointer-events-none absolute -right-16 -top-16 h-56 w-56 rounded-full bg-gold-400/10 blur-3xl"></div>
                    <div class="pointer-events-none absolute -bottom-16 left-1/4 h-44 w-44 rounded-full bg-blue-400/10 blur-3xl"></div>

                    <div class="relative z-10">
                        <p class="text-sm font-semibold text-gold-300">{greeting}</p>
                        <h1 class="mt-1 text-2xl font-black text-white sm:text-3xl">
                            {user_session.name.clone()}
                        </h1>

                        <div class="mt-4 flex flex-wrap gap-2">
                            <span class="inline-flex items-center gap-1.5 rounded-full border border-white/15 bg-white/10 px-2.5 py-1 text-xs font-medium text-slate-100">
                                <svg class="h-3.5 w-3.5" fill="currentColor" viewBox="0 0 20 20">
                                    <path
                                        fill-rule="evenodd"
                                        d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z"
                                        clip-rule="evenodd"
                                    />
                                </svg>
                                {user_session.role.display_name()}
                            </span>
                            {user_session
                                .nip
                                .clone()
                                .map(|nip| {
                                    view! {
                                        <span class="inline-flex items-center gap-1.5 rounded-full border border-white/15 bg-white/10 px-2.5 py-1 text-xs font-medium text-slate-100">
                                            "NIP: " {nip}
                                        </span>
                                    }
                                })}
                            {user_session
                                .jabatan
                                .clone()
                                .map(|j| {
                                    view! {
                                        <span class="inline-flex items-center gap-1.5 rounded-full border border-white/15 bg-white/10 px-2.5 py-1 text-xs font-medium text-slate-100">
                                            {j}
                                        </span>
                                    }
                                })}
                        </div>
                    </div>
                </section>

                <section>
                    <SectionHeader label="Ringkasan Akun" />
                    <Suspense fallback=move || {
                        view! { <LoadingSkeleton count=3 /> }
                    }>
                        {move || {
                            me_data
                                .get()
                                .map(|me_opt| {
                                    if let Some(me) = me_opt {
                                        view! {
                                            <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
                                                <InfoCard
                                                    label="Email"
                                                    value=me.email.clone()
                                                    icon_path="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
                                                />
                                                <InfoCard
                                                    label="MFA"
                                                    value=if me.mfa_enabled {
                                                        "Aktif".to_string()
                                                    } else {
                                                        "Belum Aktif".to_string()
                                                    }
                                                    icon_path="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
                                                    accent=if me.mfa_enabled { "green" } else { "amber" }
                                                />
                                                <InfoCard
                                                    label="Email Verifikasi"
                                                    value=if me.email_verified {
                                                        "Terverifikasi".to_string()
                                                    } else {
                                                        "Belum".to_string()
                                                    }
                                                    icon_path="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                                                    accent=if me.email_verified { "green" } else { "amber" }
                                                />
                                            </div>
                                        }
                                            .into_any()
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }
                                })
                        }}
                    </Suspense>
                </section>

                {if is_admin {
                    Some(
                        view! {
                            <section>
                                <SectionHeader label="Statistik Sistem" />
                                <Suspense fallback=move || {
                                    view! { <LoadingSkeleton count=4 /> }
                                }>
                                    {move || {
                                        admin_stats
                                            .get()
                                            .map(|stats_opt| {
                                                if let Some(stats) = stats_opt {
                                                    view! {
                                                        <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
                                                            <StatCard
                                                                label="Total Pengguna"
                                                                value=stats.total_users
                                                                color="blue"
                                                            />
                                                            <StatCard
                                                                label="Pengguna Aktif"
                                                                value=stats.active_users
                                                                color="green"
                                                            />
                                                            <StatCard
                                                                label="Sesi Aktif"
                                                                value=stats.active_sessions
                                                                color="indigo"
                                                            />
                                                            <StatCard
                                                                label="Total Realm"
                                                                value=stats.total_realms
                                                                color="purple"
                                                            />
                                                        </div>

                                                        {(stats.security_events_today > 0
                                                            || stats.failed_login_attempts > 0)
                                                            .then(|| {
                                                                view! {
                                                                    <div class="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-2">
                                                                        <StatCard
                                                                            label="Event Keamanan Hari Ini"
                                                                            value=stats.security_events_today
                                                                            color="amber"
                                                                        />
                                                                        <StatCard
                                                                            label="Login Gagal"
                                                                            value=stats.failed_login_attempts
                                                                            color="red"
                                                                        />
                                                                    </div>
                                                                }
                                                            })}
                                                    }
                                                        .into_any()
                                                } else {
                                                    view! { <div></div> }.into_any()
                                                }
                                            })
                                    }}
                                </Suspense>
                            </section>
                        },
                    )
                } else {
                    None
                }}

                <section>
                    <SectionHeader label="Menu Utama" />
                    <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">
                        <QuickLink
                            href="/portal/apps"
                            label="Aplikasi"
                            desc="Akses layanan"
                            color="blue"
                            icon_path="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zm10 0a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zm10 0a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2"
                        />
                        <QuickLink
                            href="/portal/profile"
                            label="Profil"
                            desc="Data pegawai"
                            color="green"
                            icon_path="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                        />
                        <QuickLink
                            href="/portal/settings"
                            label="Pengaturan"
                            desc="Preferensi akun"
                            color="purple"
                            icon_path="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                        />
                        <QuickLink
                            href="/portal/notifications"
                            label="Notifikasi"
                            desc="Pemberitahuan"
                            color="amber"
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
    let (bg, text) = match color {
        "blue" => ("bg-blue-500/10 border-blue-400/30", "text-blue-200"),
        "green" => (
            "bg-emerald-500/10 border-emerald-400/30",
            "text-emerald-200",
        ),
        "indigo" => ("bg-indigo-500/10 border-indigo-400/30", "text-indigo-200"),
        "purple" => ("bg-violet-500/10 border-violet-400/30", "text-violet-200"),
        "amber" => ("bg-amber-500/10 border-amber-400/30", "text-amber-200"),
        "red" => ("bg-red-500/10 border-red-400/30", "text-red-200"),
        _ => ("bg-slate-500/10 border-slate-400/30", "text-slate-200"),
    };

    view! {
        <article class=format!("rounded-2xl border p-4 backdrop-blur {}", bg)>
            <p class="text-xs font-semibold uppercase tracking-[0.08em] text-slate-400">{label}</p>
            <p class=format!("mt-2 text-2xl font-black {}", text)>{value}</p>
        </article>
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
        "green" => ("text-emerald-300", "bg-emerald-500/15"),
        "amber" => ("text-amber-300", "bg-amber-500/15"),
        _ => ("text-blue-300", "bg-blue-500/15"),
    };

    view! {
        <article class="flex items-center gap-3 rounded-2xl border border-white/10 bg-slate-900/50 p-4 backdrop-blur">
            <div class=format!(
                "w-10 h-10 rounded-lg flex items-center justify-center flex-shrink-0 {}",
                bg_color,
            )>
                <svg
                    class=format!("w-5 h-5 {}", text_color)
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d=icon_path
                    />
                </svg>
            </div>
            <div class="min-w-0">
                <p class="text-xs text-slate-400">{label}</p>
                <p class="truncate text-sm font-semibold text-slate-100">{value}</p>
            </div>
        </article>
    }
}

#[component]
fn SectionHeader(label: &'static str) -> impl IntoView {
    view! {
        <div class="mb-3 flex items-center gap-3">
            <div class="h-8 w-1 rounded-full bg-gradient-to-b from-gold-400 to-amber-300"></div>
            <h2 class="text-lg font-bold tracking-wide text-slate-100">{label}</h2>
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
        "blue" => "bg-blue-500/20 text-blue-300",
        "green" => "bg-emerald-500/20 text-emerald-300",
        "purple" => "bg-violet-500/20 text-violet-300",
        "amber" => "bg-amber-500/20 text-amber-300",
        _ => "bg-slate-500/20 text-slate-300",
    };

    view! {
        <a
            href=href
            class="group flex items-center gap-3 rounded-xl border border-white/10 bg-slate-900/50 p-4 transition-all duration-300 hover:border-white/20 hover:bg-slate-900/80"
        >
            <div class=format!(
                "w-10 h-10 rounded-lg flex items-center justify-center flex-shrink-0 {}",
                icon_bg,
            )>
                <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d=icon_path
                    />
                </svg>
            </div>
            <div class="min-w-0 flex-1">
                <p class="text-sm font-semibold text-slate-100 transition-colors group-hover:text-gold-300">
                    {label}
                </p>
                <p class="text-xs text-slate-500">{desc}</p>
            </div>
            <svg
                class="h-4 w-4 flex-shrink-0 text-slate-600 transition-colors group-hover:text-slate-300"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M9 5l7 7-7 7"
                />
            </svg>
        </a>
    }
}

#[component]
fn LoadingSkeleton(#[prop(optional, default = 3)] count: usize) -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
            {(0..count)
                .map(|_| {
                    view! {
                        <div class="animate-pulse rounded-2xl border border-white/10 bg-slate-900/50 p-4">
                            <div class="mb-2 h-3 w-20 rounded bg-slate-700/70"></div>
                            <div class="h-6 w-12 rounded bg-slate-700/70"></div>
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
}
