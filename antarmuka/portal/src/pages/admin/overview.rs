//! Admin Overview / Dashboard Page
//!
//! Summary stats, quick actions, and health indicators for administrators.
//! REQ-PORTAL-016

use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{AppState, use_api_client, use_app_state};
use crate::utils::authenc_api::AdminStats;
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Admin overview dashboard
#[component]
pub fn AdminOverviewPage() -> impl IntoView {
    let state = use_app_state();
    let api = use_api_client();

    let (stats, set_stats) = signal(Option::<AdminStats>::None);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    {
        let api = api.clone();
        Effect::new(move || {
            let api = api.clone();
            spawn_local(async move {
                match api.iam_admin_stats().await {
                    Ok(s) => set_stats.set(Some(s)),
                    Err(e) => set_error.set(Some(format!("Gagal memuat statistik: {}", e))),
                }
                set_loading.set(false);
            });
        });
    }

    let on_logout = {
        Box::new(move || {
            crate::features::auth::AuthService::logout();
            state.set(AppState::default());
        }) as Box<dyn Fn()>
    };

    let session = state.get().user.unwrap_or_default();

    view! {
        <MainLayout user_session=session.clone() on_logout=on_logout>
            <div class="max-w-7xl mx-auto px-4 py-8">
                <div class="flex items-center justify-between mb-8">
                    <div>
                        <h1 class="text-2xl font-bold text-gray-900">"Admin Panel"</h1>
                        <p class="text-gray-600">"Administrasi Identity & Access Management"</p>
                    </div>
                </div>

                {move || error.get().map(|msg| view! {
                    <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-red-700">{msg}</div>
                })}

                // Quick navigation cards
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-8">
                    <AdminNavCard
                        icon="👥"
                        title="Manajemen Pengguna"
                        desc="Kelola pengguna, buat akun baru, atur peran"
                        href="/portal/admin/users"
                    />
                    <AdminNavCard
                        icon="🏢"
                        title="Realm"
                        desc="Kelola realm dan konfigurasi tenant"
                        href="/portal/admin/realms"
                    />
                    <AdminNavCard
                        icon="🔑"
                        title="Klien OAuth2"
                        desc="Kelola aplikasi klien dan credential"
                        href="/portal/admin/clients"
                    />
                    <AdminNavCard
                        icon="🛡️"
                        title="Peran & Hak Akses"
                        desc="Kelola role dan permission"
                        href="/portal/admin/roles"
                    />
                    <AdminNavCard
                        icon="🌐"
                        title="Federasi"
                        desc="Kelola Identity Provider eksternal"
                        href="/portal/admin/federation"
                    />
                    <AdminNavCard
                        icon="📋"
                        title="Audit Log"
                        desc="Lihat log aktivitas dan keamanan"
                        href="/portal/admin/audit"
                    />
                </div>

                // Stats cards
                <Show
                    when=move || !loading.get()
                    fallback=|| view! {
                        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
                            <div class="bg-white rounded-xl border p-4 animate-pulse"><div class="h-12 bg-gray-200 rounded"></div></div>
                            <div class="bg-white rounded-xl border p-4 animate-pulse"><div class="h-12 bg-gray-200 rounded"></div></div>
                            <div class="bg-white rounded-xl border p-4 animate-pulse"><div class="h-12 bg-gray-200 rounded"></div></div>
                            <div class="bg-white rounded-xl border p-4 animate-pulse"><div class="h-12 bg-gray-200 rounded"></div></div>
                        </div>
                    }
                >
                    {move || stats.get().map(|s| view! {
                        <h2 class="text-lg font-semibold text-gray-900 mb-4">"Statistik Sistem"</h2>
                        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
                            <StatCard label="Total Pengguna" value=s.total_users.to_string() icon="👤" />
                            <StatCard label="Pengguna Aktif" value=s.active_users.to_string() icon="✅" />
                            <StatCard label="Sesi Aktif" value=s.total_sessions.to_string() icon="📱" />
                            <StatCard label="Klien OAuth2" value=s.total_clients.to_string() icon="🔑" />
                        </div>
                    })}
                </Show>
            </div>
        </MainLayout>
    }
}

#[component]
fn AdminNavCard(
    icon: &'static str,
    title: &'static str,
    desc: &'static str,
    href: &'static str,
) -> impl IntoView {
    view! {
        <a href=href class="block bg-white rounded-xl border border-gray-200 p-5 hover:shadow-md hover:border-primary-300 transition-all group">
            <span class="text-3xl">{icon}</span>
            <h3 class="font-semibold text-gray-900 mt-3 group-hover:text-primary-700">{title}</h3>
            <p class="text-sm text-gray-500 mt-1">{desc}</p>
        </a>
    }
}

#[component]
fn StatCard(label: &'static str, value: String, icon: &'static str) -> impl IntoView {
    view! {
        <div class="bg-white rounded-xl border border-gray-200 p-4">
            <div class="flex items-center gap-2 mb-1">
                <span>{icon}</span>
                <span class="text-sm text-gray-500">{label}</span>
            </div>
            <p class="text-2xl font-bold text-gray-900">{value}</p>
        </div>
    }
}
