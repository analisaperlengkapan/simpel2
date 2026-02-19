//! Dashboard Home — ringkasan utama untuk semua role.

use crate::api::fetch_dashboard_stats;
use crate::components::role_switcher::get_active_role;
use leptos::prelude::*;

// ── Stat Card ─────────────────────────────────────────────────────────────

#[component]
pub fn StatCard(
    icon: &'static str,
    label: &'static str,
    value: String,
    subtitle: &'static str,
    color: &'static str,
) -> impl IntoView {
    let (bg, _text, _icon_bg) = match color {
        "blue" => ("from-blue-500 to-blue-600", "text-blue-600", "bg-blue-100"),
        "green" => (
            "from-emerald-500 to-green-600",
            "text-emerald-600",
            "bg-emerald-100",
        ),
        "amber" => (
            "from-amber-500 to-yellow-600",
            "text-amber-600",
            "bg-amber-100",
        ),
        "red" => ("from-red-500 to-pink-600", "text-red-600", "bg-red-100"),
        _ => ("from-gray-500 to-gray-600", "text-gray-600", "bg-gray-100"),
    };
    let gradient = format!("bg-gradient-to-br {bg}");

    view! {
        <div class=format!(
            "group relative overflow-hidden {gradient} text-white p-5 rounded-xl shadow-md \
             hover:shadow-lg transition-all duration-300 cursor-pointer"
        )>
            <div class="absolute top-0 right-0 w-20 h-20 bg-white/10 rounded-full -mr-10 -mt-10"></div>
            <div class="relative z-10">
                <div class="flex items-center justify-between mb-3">
                    <div class="p-2 bg-white/20 backdrop-blur-sm rounded-lg">
                        <i class=format!("{icon} text-xl")></i>
                    </div>
                </div>
                <p class="text-xs font-medium opacity-80 mb-0.5">{label}</p>
                <p class="text-3xl font-bold">{value}</p>
                <p class="text-[11px] opacity-70 mt-1">
                    <i class="fas fa-info-circle mr-1"></i>
                    {subtitle}
                </p>
            </div>
        </div>
    }
}

// ── Quick Navigation Card ─────────────────────────────────────────────────

#[component]
pub fn QuickNav(
    href: &'static str,
    icon: &'static str,
    label: &'static str,
    description: &'static str,
    color: &'static str,
) -> impl IntoView {
    let icon_bg = match color {
        "emerald" => "bg-emerald-100 text-emerald-600",
        "blue" => "bg-blue-100 text-blue-600",
        "indigo" => "bg-indigo-100 text-indigo-600",
        "purple" => "bg-purple-100 text-purple-600",
        "red" => "bg-red-100 text-red-600",
        "amber" => "bg-amber-100 text-amber-600",
        "teal" => "bg-teal-100 text-teal-600",
        "pink" => "bg-pink-100 text-pink-600",
        "orange" => "bg-orange-100 text-orange-600",
        "gray" => "bg-gray-100 text-gray-600",
        "cyan" => "bg-cyan-100 text-cyan-600",
        _ => "bg-gray-100 text-gray-600",
    };

    view! {
        <a
            href=href
            class="group bg-white p-4 rounded-xl shadow-sm border border-gray-100 \
                   hover:border-blue-200 hover:shadow-md transition-all duration-200"
        >
            <div class="flex items-start gap-3">
                <div class=format!(
                    "p-2.5 rounded-lg {icon_bg} group-hover:scale-110 transition-transform"
                )>
                    <i class=format!("{icon} text-lg")></i>
                </div>
                <div class="min-w-0">
                    <h3 class="font-semibold text-gray-800 text-sm leading-tight">{label}</h3>
                    <p class="text-[11px] text-gray-500 mt-0.5">{description}</p>
                </div>
            </div>
        </a>
    }
}

// ── Dashboard Home ────────────────────────────────────────────────────────

#[component]
pub fn DashboardHome() -> impl IntoView {
    let stats_resource = LocalResource::new(move || async move {
        match fetch_dashboard_stats().await {
            Ok(response) => Some(response.data),
            Err(e) => {
                leptos::logging::error!("Failed to fetch stats: {:?}", e);
                None
            }
        }
    });

    let active_role = get_active_role();
    let is_admin = active_role == "admin";

    view! {
        <div class="space-y-6">
            // ── Welcome Banner ─────────────────────────────────────────────
            <div class="relative overflow-hidden bg-gradient-to-r from-blue-700 via-indigo-700 \
                        to-purple-700 rounded-2xl shadow-xl p-6 md:p-8 text-white">
                <div class="absolute inset-0 opacity-10">
                    <div class="absolute w-64 h-64 bg-white rounded-full -top-20 -right-20"></div>
                    <div class="absolute w-48 h-48 bg-white rounded-full -bottom-16 -left-16"></div>
                </div>
                <div class="relative z-10 flex flex-col md:flex-row items-start md:items-center \
                            justify-between gap-4">
                    <div>
                        <h1 class="text-2xl md:text-3xl font-bold tracking-tight">
                            "Dashboard Perlengkapan"
                        </h1>
                        <p class="text-blue-200 text-sm mt-1">
                            "Sistem Informasi Manajemen Perlengkapan Kejaksaan RI"
                        </p>
                    </div>
                    <div class="flex flex-wrap gap-2">
                        <span class="inline-flex items-center gap-1.5 px-3 py-1.5 bg-white/15 \
                                     backdrop-blur-sm rounded-lg text-xs font-medium">
                            <i class="fas fa-user-shield"></i>
                            {active_role.clone()}
                        </span>
                        <span class="inline-flex items-center gap-1.5 px-3 py-1.5 bg-white/15 \
                                     backdrop-blur-sm rounded-lg text-xs font-medium">
                            <i class="fas fa-calendar-day"></i>
                            "Tahun Anggaran 2025"
                        </span>
                    </div>
                </div>
            </div>

            // ── Stats Cards ────────────────────────────────────────────────
            <Suspense fallback=move || view! {
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                    {(0..4).map(|_| view! {
                        <div class="bg-white rounded-xl shadow-sm border p-6 animate-pulse">
                            <div class="h-4 bg-gray-200 rounded w-20 mb-3"></div>
                            <div class="h-8 bg-gray-200 rounded w-16 mb-2"></div>
                            <div class="h-3 bg-gray-200 rounded w-24"></div>
                        </div>
                    }).collect_view()}
                </div>
            }>
                {move || stats_resource.get().flatten().map(|stats| view! {
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                        <StatCard
                            icon="fas fa-box"
                            label="Total Aset"
                            value=stats.total_aset.to_string()
                            subtitle="Terintegrasi SIMAN"
                            color="blue"
                        />
                        <StatCard
                            icon="fas fa-check-circle"
                            label="Kondisi Baik"
                            value=stats.aset_baik.to_string()
                            subtitle="Siap Pakai"
                            color="green"
                        />
                        <StatCard
                            icon="fas fa-exclamation-triangle"
                            label="Kondisi Rusak"
                            value=stats.aset_rusak.to_string()
                            subtitle="Perlu Tindakan"
                            color="amber"
                        />
                        <StatCard
                            icon="fas fa-building"
                            label="Total Satker"
                            value=stats.total_satker.to_string()
                            subtitle="Unit Terdata"
                            color="red"
                        />
                    </div>
                })}
            </Suspense>

            // ── Quick Navigation Grid ──────────────────────────────────────
            <div>
                <h2 class="text-lg font-bold text-gray-800 mb-4">"Menu Utama"</h2>
                <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
                    <QuickNav href="/perlengkapan/dashboard/bank-aset/daftar"         icon="fas fa-boxes"          label="Bank Aset"        description="Daftar aset BMN"       color="emerald" />
                    <QuickNav href="/perlengkapan/dashboard/kebutuhan-bmn/dashboard"  icon="fas fa-clipboard-list" label="Kebutuhan BMN"    description="Analisis kebutuhan"    color="blue"    />
                    <QuickNav href="/perlengkapan/dashboard/pemakaian-bmn/daftar"     icon="fas fa-file-signature" label="Pemakaian BMN"    description="Pengajuan pemakaian"   color="indigo"  />
                    <QuickNav href="/perlengkapan/dashboard/pakaian-dinas/pengajuan"  icon="fas fa-tshirt"         label="Pakaian Dinas"    description="Kebutuhan pakaian"     color="purple"  />
                    <QuickNav href="/perlengkapan/dashboard/pengelolaan/penghapusan"  icon="fas fa-trash-alt"      label="Penghapusan BMN"  description="Proses penghapusan"    color="red"     />
                    <QuickNav href="/perlengkapan/dashboard/pengelolaan/mutasi"       icon="fas fa-exchange-alt"   label="Mutasi BMN"       description="Transfer antar satker" color="amber"   />
                    <QuickNav href="/perlengkapan/dashboard/pemeliharaan/daftar"      icon="fas fa-tools"          label="Pemeliharaan"     description="Jadwal dan riwayat"    color="teal"    />
                    <QuickNav href="/perlengkapan/dashboard/pengelolaan/hibah"        icon="fas fa-gift"           label="Hibah BMN"        description="Pengelolaan hibah"     color="pink"    />
                </div>
            </div>

            // ── Admin Quick Access ─────────────────────────────────────────
            <Show when=move || is_admin>
                <div>
                    <h2 class="text-lg font-bold text-gray-800 mb-4 flex items-center gap-2">
                        <i class="fas fa-shield-alt text-red-500"></i>
                        "Panel Admin"
                    </h2>
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                        <QuickNav href="/perlengkapan/dashboard/admin/users"  icon="fas fa-users-cog"     label="Manajemen User"  description="Kelola pengguna"    color="red"    />
                        <QuickNav href="/perlengkapan/dashboard/admin/roles"  icon="fas fa-user-tag"      label="Manajemen Role"  description="Kelola hak akses"   color="orange" />
                        <QuickNav href="/perlengkapan/dashboard/admin/audit"  icon="fas fa-clipboard-list" label="Audit Log"      description="Riwayat aktivitas"  color="gray"   />
                        <QuickNav href="/perlengkapan/dashboard/admin/master" icon="fas fa-database"      label="Master Data"     description="Data referensi"     color="cyan"   />
                    </div>
                </div>
            </Show>
        </div>
    }
}
