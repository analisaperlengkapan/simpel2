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
    let (icon_bg, icon_color, glow) = match color {
        "blue" => ("bg-blue-500/20", "text-blue-400", "shadow-[0_0_15px_rgba(59,130,246,0.3)]"),
        "green" => ("bg-emerald-500/20", "text-emerald-400", "shadow-[0_0_15px_rgba(16,185,129,0.3)]"),
        "amber" => ("bg-amber-500/20", "text-amber-400", "shadow-[0_0_15px_rgba(245,158,11,0.3)]"),
        "red" => ("bg-red-500/20", "text-red-400", "shadow-[0_0_15px_rgba(239,68,68,0.3)]"),
        "gold" => ("bg-gold-500/20", "text-gold-400", "shadow-[0_0_15px_rgba(234,179,8,0.3)]"),
        _ => ("bg-gray-500/20", "text-gray-400", "shadow-[0_0_15px_rgba(107,114,128,0.3)]"),
    };

    view! {
        <div class="group relative overflow-hidden bg-white/5 backdrop-blur-xl border border-white/10 p-5 rounded-2xl transition-all duration-300 cursor-pointer hover:bg-white/10 hover:-translate-y-1">
            // Decorative background glow
            <div class=format!("absolute -top-6 -right-6 w-24 h-24 rounded-full blur-2xl opacity-40 {icon_bg}")></div>

            <div class="relative z-10">
                <div class="flex items-center justify-between mb-4">
                    <div class=format!("p-3 rounded-xl {icon_bg} {icon_color} {glow} flex items-center justify-center")>
                        <i class=format!("{icon} text-xl")></i>
                    </div>
                </div>
                <p class="text-sm font-medium text-slate-400 tracking-wide">{label}</p>
                <p class="text-3xl font-extrabold text-white mt-1 drop-shadow-sm">{value}</p>
                <div class="flex items-center gap-1.5 mt-3 text-xs text-slate-500">
                    <i class="fas fa-info-circle opacity-70"></i>
                    <span>{subtitle}</span>
                </div>
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
    let (icon_bg, icon_text) = match color {
        "emerald" => ("bg-emerald-500/10", "text-emerald-400"),
        "blue" => ("bg-blue-500/10", "text-blue-400"),
        "indigo" => ("bg-indigo-500/10", "text-indigo-400"),
        "purple" => ("bg-purple-500/10", "text-purple-400"),
        "red" => ("bg-red-500/10", "text-red-400"),
        "amber" => ("bg-amber-500/10", "text-amber-400"),
        "teal" => ("bg-teal-500/10", "text-teal-400"),
        "pink" => ("bg-pink-500/10", "text-pink-400"),
        "orange" => ("bg-orange-500/10", "text-orange-400"),
        "cyan" => ("bg-cyan-500/10", "text-cyan-400"),
        "gold" => ("bg-gold-500/10", "text-gold-400"),
        _ => ("bg-gray-500/10", "text-gray-400"),
    };

    view! {
        <a
            href=href
            class="group relative bg-white/5 backdrop-blur-sm p-4 rounded-xl border border-white/10 \
                   hover:border-gold-500/50 hover:bg-white/[0.07] transition-all duration-300 overflow-hidden"
        >
            // Subtle hover glow effect
            <div class="absolute inset-0 bg-gradient-to-br from-gold-500/0 via-transparent to-transparent group-hover:from-gold-500/5 transition-colors duration-500"></div>

            <div class="relative flex items-start gap-4 z-10">
                <div class=format!(
                    "p-3 rounded-lg {icon_bg} {icon_text} group-hover:scale-110 group-hover:shadow-[0_0_15px_currentColor] shadow-black/0 transition-all duration-300"
                )>
                    <i class=format!("{icon} text-lg")></i>
                </div>
                <div class="min-w-0 flex-1">
                    <h3 class="font-bold text-slate-200 text-sm tracking-wide group-hover:text-gold-400 transition-colors">{label}</h3>
                    <p class="text-xs text-slate-500 mt-1 line-clamp-2 leading-relaxed">{description}</p>
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
        <div class="space-y-8 animate-fade-in p-2 md:p-4">

            // ── Premium Welcome Banner ─────────────────────────────────────────────
            <div class="relative overflow-hidden bg-gradient-to-br from-navy-800 via-navy-900 to-black \
                        rounded-3xl border border-white/10 shadow-[0_8px_30px_rgb(0,0,0,0.5)] p-8 md:p-10">

                // Abstract glowing art
                <div class="absolute -right-20 -top-20 w-80 h-80 bg-gold-500/10 rounded-full blur-[80px] animate-pulse-slow"></div>
                <div class="absolute right-40 -bottom-20 w-60 h-60 bg-emerald-500/10 rounded-full blur-[60px] animate-pulse-slow" style="animation-delay: 2s;"></div>

                // Decorative Logo watermark
                <div class="absolute right-0 top-1/2 -translate-y-1/2 opacity-5 pointer-events-none transform translate-x-1/4 scale-150">
                    <img src="/perlengkapan/assets/kejaksaan-logo.png" class="w-96 h-96 object-contain filter grayscale invert" alt="" />
                </div>

                <div class="relative z-10 flex flex-col md:flex-row items-center justify-between gap-8 h-full">
                    <div class="flex-1 space-y-4 text-center md:text-left">
                        <div class="inline-flex items-center gap-2 px-3 py-1 bg-white/5 backdrop-blur-md border border-white/10 rounded-full text-xs font-semibold text-gold-400 mb-2">
                            <span class="relative flex h-2 w-2">
                                <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-gold-400 opacity-75"></span>
                                <span class="relative inline-flex rounded-full h-2 w-2 bg-gold-500"></span>
                            </span>
                            "Portal Perlengkapan Kejaksaan"
                        </div>
                        <h1 class="text-3xl md:text-5xl font-extrabold tracking-tight">
                            "Ringkasan "
                            <span class="text-transparent bg-clip-text bg-gradient-to-r from-gold-300 via-gold-400 to-gold-600">
                                "Sistem Manajemen"
                            </span>
                        </h1>
                        <p class="text-slate-400 text-sm md:text-base max-w-xl leading-relaxed">
                            "Akses cepat ke seluruh modul operasional aset dan infrastruktur secara terintegrasi dan real-time."
                        </p>

                        <div class="flex flex-wrap items-center justify-center md:justify-start gap-3 mt-6 pt-4">
                            <div class="flex items-center gap-2 px-4 py-2 bg-navy-950/50 backdrop-blur-md border border-white/5 rounded-xl text-sm font-medium text-slate-300">
                                <i class="fas fa-shield-alt text-emerald-400"></i>
                                <span>"Role: " <span class="text-white capitalize">{active_role.clone()}</span></span>
                            </div>
                            <div class="flex items-center gap-2 px-4 py-2 bg-navy-950/50 backdrop-blur-md border border-white/5 rounded-xl text-sm font-medium text-slate-300">
                                <i class="fas fa-clock text-blue-400"></i>
                                <span>"Tahun Anggaran 2025"</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            // ── Stats Cards ────────────────────────────────────────────────
            <Suspense fallback=move || view! {
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
                    {(0..4).map(|_| view! {
                        <div class="bg-white/5 backdrop-blur-xl border border-white/10 rounded-2xl p-6 animate-pulse">
                            <div class="flex justify-between mb-4">
                                <div class="h-10 w-10 bg-white/10 rounded-xl"></div>
                            </div>
                            <div class="h-4 bg-white/10 rounded w-24 mb-3"></div>
                            <div class="h-8 bg-white/10 rounded w-16 mb-4"></div>
                            <div class="h-3 bg-white/10 rounded w-32"></div>
                        </div>
                    }).collect_view()}
                </div>
            }>
                {move || stats_resource.get().flatten().map(|stats| view! {
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
                        <StatCard
                            icon="fas fa-box"
                            label="Total Aset BMN"
                            value=stats.total_aset.to_string()
                            subtitle="Terintegrasi SIMAN"
                            color="blue"
                        />
                        <StatCard
                            icon="fas fa-check-circle"
                            label="Aset Kondisi Baik"
                            value=stats.aset_baik.to_string()
                            subtitle="Sipp Pakai"
                            color="green"
                        />
                        <StatCard
                            icon="fas fa-exclamation-triangle"
                            label="Perlu Perbaikan"
                            value=stats.aset_rusak.to_string()
                            subtitle="Tindakan Diperlukan"
                            color="gold"
                        />
                        <StatCard
                            icon="fas fa-building"
                            label="Cakupan Satker"
                            value=stats.total_satker.to_string()
                            subtitle="Unit Kerja Aktif"
                            color="indigo"
                        />
                    </div>
                })}
            </Suspense>

            // ── Quick Navigation Grid ──────────────────────────────────────
            <div class="pt-4">
                <div class="flex items-center gap-3 mb-6">
                    <div class="h-8 w-1 bg-gold-500 rounded-full shadow-[0_0_10px_rgba(234,179,8,0.5)]"></div>
                    <h2 class="text-xl font-bold text-white tracking-wide">"Modul Utama"</h2>
                </div>
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                    <QuickNav href="/perlengkapan/dashboard/bank-aset/daftar"         icon="fas fa-boxes"          label="Bank Aset"        description="Katalog dan registrasi BMN"       color="emerald" />
                    <QuickNav href="/perlengkapan/dashboard/kebutuhan-bmn/dashboard"  icon="fas fa-clipboard-list" label="Kebutuhan BMN"    description="Analisis dan usulan"    color="blue"    />
                    <QuickNav href="/perlengkapan/dashboard/pemakaian-bmn/daftar"     icon="fas fa-file-signature" label="Pemakaian BMN"    description="Pengajuan & SIP"   color="indigo"  />
                    <QuickNav href="/perlengkapan/dashboard/pakaian-dinas/pengajuan"  icon="fas fa-tshirt"         label="Pakaian Dinas"    description="Distribusi atribut"     color="gold"  />
                    <QuickNav href="/perlengkapan/dashboard/pengelolaan/penghapusan"  icon="fas fa-trash-alt"      label="Penghapusan"  description="Disposal aset"    color="red"     />
                    <QuickNav href="/perlengkapan/dashboard/pengelolaan/mutasi"       icon="fas fa-exchange-alt"   label="Mutasi & Transfer"       description="Perpindahan antar satker" color="amber"   />
                    <QuickNav href="/perlengkapan/dashboard/pemeliharaan/daftar"      icon="fas fa-tools"          label="Pemeliharaan"     description="Jadwal & log service"    color="teal"    />
                    <QuickNav href="/perlengkapan/dashboard/pengelolaan/hibah"        icon="fas fa-gift"           label="Hibah"        description="Penerimaan pihak ketiga"     color="pink"    />
                </div>
            </div>

            // ── Admin Quick Access ─────────────────────────────────────────
            <Show when=move || is_admin>
                <div class="pt-8">
                    <div class="flex items-center gap-3 mb-6">
                        <div class="h-8 w-1 bg-red-500 rounded-full shadow-[0_0_10px_rgba(239,68,68,0.5)]"></div>
                        <h2 class="text-xl font-bold text-white tracking-wide flex items-center gap-2">
                            "Panel Administrator Sistem"
                            <i class="fas fa-shield-alt text-red-500 text-sm ml-2 overflow-visible"></i>
                        </h2>
                    </div>
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                        <QuickNav href="/perlengkapan/dashboard/admin/users"  icon="fas fa-users-cog"     label="Manajemen Identitas"  description="Data pengguna & sesi"    color="red"    />
                        <QuickNav href="/perlengkapan/dashboard/admin/roles"  icon="fas fa-user-tag"      label="Otorisasi (RBAC)"  description="Konfigurasi hak akses"   color="orange" />
                        <QuickNav href="/perlengkapan/dashboard/admin/audit"  icon="fas fa-history"       label="Buku Log Keamanan"      description="Jejak audit seluruh aktivitas"  color="gray"   />
                        <QuickNav href="/perlengkapan/dashboard/admin/master" icon="fas fa-database"      label="Repositori Master Data"     description="Referensi sistem"     color="cyan"   />
                    </div>
                </div>
            </Show>
        </div>
    }
}
