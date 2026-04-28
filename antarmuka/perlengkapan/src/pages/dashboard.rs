//! Dashboard Home for Perlengkapan.
//!
//! This page focuses on clear information hierarchy, responsive layout,
//! and maintainable utility-class styling.

use crate::api::dashboard::fetch_dashboard_stats;
use crate::api::types::DashboardStats;
use crate::components::role_switcher::get_active_role;
use crate::routes;
use leptos::prelude::*;
use leptos_fetch::QueryClient;
use lib_ui::components::icon::{AppIcon, icon_from_fa_class};
use phosphor_leptos::{CALENDAR, CARET_RIGHT, SHIELD};
use leptos_meta::Title;

#[component]
fn SectionHeader(title: &'static str, tone: &'static str) -> impl IntoView {
    let tone_class = match tone {
        "gold" => "from-gold-400 to-amber-300",
        "teal" => "from-teal-400 to-cyan-300",
        "red" => "from-red-400 to-rose-300",
        _ => "from-slate-400 to-slate-300",
    };

    view! {
        <div class="mb-4 flex items-center gap-3">
            <div class=format!("h-8 w-1 rounded-full bg-gradient-to-b {}", tone_class)></div>
            <h2 class="text-lg font-bold tracking-wide text-slate-100 sm:text-xl">{title}</h2>
        </div>
    }
}

#[component]
fn StatCard(
    icon: &'static str,
    label: &'static str,
    #[prop(into)] value: String,
    subtitle: &'static str,
    tone: &'static str,
) -> impl IntoView {
    let (icon_bg, icon_text, glow) = match tone {
        "blue" => ("bg-blue-500/15", "text-blue-300", "shadow-blue-500/30"),
        "green" => (
            "bg-emerald-500/15",
            "text-emerald-300",
            "shadow-emerald-500/30",
        ),
        "amber" => ("bg-amber-500/15", "text-amber-300", "shadow-amber-500/30"),
        "violet" => (
            "bg-violet-500/15",
            "text-violet-300",
            "shadow-violet-500/30",
        ),
        _ => ("bg-slate-500/15", "text-slate-300", "shadow-slate-500/30"),
    };

    view! {
        <article class="group relative overflow-hidden rounded-2xl border border-white/10 bg-slate-900/60 p-5 backdrop-blur transition-all duration-300 hover:-translate-y-0.5 hover:border-white/20 hover:bg-slate-900/80">
            <div class="absolute -right-8 -top-8 h-20 w-20 rounded-full bg-white/5 blur-2xl"></div>
            <div class="relative z-10">
                <div class="mb-4 flex items-center justify-between">
                    <div class=format!(
                        "flex h-11 w-11 items-center justify-center rounded-xl {} {} shadow-lg",
                        icon_bg, glow
                    )>
                        <span class=format!("inline-flex {}", icon_text)>
                            <AppIcon icon=icon_from_fa_class(icon) size=16 />
                        </span>
                    </div>
                </div>
                <p class="text-xs font-semibold uppercase tracking-[0.08em] text-slate-400">{label}</p>
                <p class="mt-2 text-3xl font-extrabold leading-none text-white">{value}</p>
                <p class="mt-2 text-xs text-slate-500">{subtitle}</p>
            </div>
        </article>
    }
}

#[component]
fn StatCardSkeleton() -> impl IntoView {
    view! {
        <div class="rounded-2xl border border-white/10 bg-slate-900/50 p-5 animate-pulse">
            <div class="mb-4 h-11 w-11 rounded-xl bg-slate-700/70"></div>
            <div class="mb-2 h-3 w-24 rounded bg-slate-700/70"></div>
            <div class="mb-2 h-8 w-16 rounded bg-slate-700/70"></div>
            <div class="h-3 w-32 rounded bg-slate-700/70"></div>
        </div>
    }
}

#[component]
fn QuickNav(
    href: &'static str,
    icon: &'static str,
    label: &'static str,
    description: &'static str,
    tone: &'static str,
) -> impl IntoView {
    let (icon_bg, icon_text, ring) = match tone {
        "emerald" => (
            "bg-emerald-500/15",
            "text-emerald-300",
            "group-hover:ring-emerald-400/30",
        ),
        "blue" => (
            "bg-blue-500/15",
            "text-blue-300",
            "group-hover:ring-blue-400/30",
        ),
        "indigo" => (
            "bg-indigo-500/15",
            "text-indigo-300",
            "group-hover:ring-indigo-400/30",
        ),
        "purple" => (
            "bg-purple-500/15",
            "text-purple-300",
            "group-hover:ring-purple-400/30",
        ),
        "red" => (
            "bg-red-500/15",
            "text-red-300",
            "group-hover:ring-red-400/30",
        ),
        "amber" => (
            "bg-amber-500/15",
            "text-amber-300",
            "group-hover:ring-amber-400/30",
        ),
        "teal" => (
            "bg-teal-500/15",
            "text-teal-300",
            "group-hover:ring-teal-400/30",
        ),
        "pink" => (
            "bg-pink-500/15",
            "text-pink-300",
            "group-hover:ring-pink-400/30",
        ),
        "orange" => (
            "bg-orange-500/15",
            "text-orange-300",
            "group-hover:ring-orange-400/30",
        ),
        "cyan" => (
            "bg-cyan-500/15",
            "text-cyan-300",
            "group-hover:ring-cyan-400/30",
        ),
        _ => (
            "bg-slate-500/15",
            "text-slate-300",
            "group-hover:ring-slate-400/30",
        ),
    };

    view! {
        <a href=href class="group block rounded-xl border border-white/10 bg-slate-900/50 p-4 transition-all duration-300 hover:border-white/20 hover:bg-slate-900/80">
            <div class="flex items-start gap-3">
                <div class=format!(
                    "mt-0.5 flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg {} {} ring-1 ring-transparent transition-all",
                    icon_bg, ring
                )>
                    <span class=format!("inline-flex {}", icon_text)>
                        <AppIcon icon=icon_from_fa_class(icon) size=14 />
                    </span>
                </div>
                <div class="min-w-0 flex-1">
                    <h3 class="text-sm font-semibold text-slate-100 transition-colors group-hover:text-gold-300">{label}</h3>
                    <p class="mt-1 text-xs leading-relaxed text-slate-500">{description}</p>
                </div>
                <span class="pt-1 text-[10px] text-slate-600 transition-colors group-hover:text-slate-300"><AppIcon icon=CARET_RIGHT /></span>
            </div>
        </a>
    }
}

/// leptos-fetch query wrapper for the dashboard stats endpoint.
async fn query_dashboard_stats(_: ()) -> Result<DashboardStats, crate::api::AppError> {
    fetch_dashboard_stats().await
}

fn format_number(n: i64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i != 0 && i % 3 == 0 {
            result.push('.');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

#[component]
pub fn DashboardHome() -> impl IntoView {
    let active_role = get_active_role();
    let role_label = match active_role.as_str() {
        "validator_wilayah" => "Validator Wilayah",
        "validator_pusat" => "Validator Pusat",
        "admin" => "Administrator",
        _ => "Operator Satker",
    };

    // leptos-fetch — `()` keyed cache so a tab-switch back to the
    // dashboard re-uses the previous load instantly.
    let client: QueryClient = expect_context();
    let stats_resource = client.local_resource(query_dashboard_stats, || ());
    let is_admin = active_role == "admin";

    view! {
        <Title text="Dashboard — SIMPEL Perlengkapan" />
        <div class="mx-auto max-w-7xl space-y-8 px-1 sm:px-2">
            <section class="relative overflow-hidden rounded-3xl border border-white/10 bg-gradient-to-br from-navy-900 via-navy-800 to-slate-950 p-6 shadow-[0_18px_50px_rgba(0,0,0,0.45)] sm:p-8 lg:p-10">
                <div class="pointer-events-none absolute -right-16 -top-16 h-56 w-56 rounded-full bg-gold-400/10 blur-3xl"></div>
                <div class="pointer-events-none absolute -bottom-16 left-1/4 h-44 w-44 rounded-full bg-blue-400/10 blur-3xl"></div>

                <div class="relative z-10">
                    <div class="inline-flex items-center gap-2 rounded-full border border-white/15 bg-white/5 px-3 py-1 text-xs font-semibold text-gold-300">
                        <span class="h-2 w-2 rounded-full bg-emerald-400"></span>
                        "Portal Perlengkapan Kejaksaan"
                    </div>

                    <h1 class="mt-4 text-2xl font-black leading-tight text-white sm:text-4xl">
                        "Ringkasan "
                        <span class="bg-gradient-to-r from-gold-300 to-amber-400 bg-clip-text text-transparent">
                            "Sistem Manajemen"
                        </span>
                    </h1>

                    <p class="mt-3 max-w-2xl text-sm leading-relaxed text-slate-300 sm:text-base">
                        "Akses cepat untuk memantau aset, kebutuhan, dan operasional perlengkapan secara terintegrasi."
                    </p>

                    <div class="mt-6 flex flex-wrap gap-3">
                        <div class="inline-flex items-center gap-2 rounded-xl border border-white/10 bg-navy-950/50 px-3 py-2 text-xs text-slate-300 sm:text-sm">
                            <span class="text-emerald-300"><AppIcon icon=SHIELD /></span>
                            <span>"Role: " <strong class="text-white">{role_label}</strong></span>
                        </div>
                        <div class="inline-flex items-center gap-2 rounded-xl border border-white/10 bg-navy-950/50 px-3 py-2 text-xs text-slate-300 sm:text-sm">
                            <span class="text-blue-300"><AppIcon icon=CALENDAR /></span>
                            "Tahun Anggaran 2025"
                        </div>
                    </div>
                </div>
            </section>

            <Suspense fallback=move || view! {
                <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
                    <StatCardSkeleton />
                    <StatCardSkeleton />
                    <StatCardSkeleton />
                    <StatCardSkeleton />
                </div>
            }>
                {move || {
                    let stats = stats_resource.get().and_then(|r| r.ok());
                    let (total, baik, rusak, satker) = match &stats {
                        Some(s) => (
                            format_number(s.total_aset),
                            format_number(s.aset_baik),
                            format_number(s.aset_rusak),
                            format_number(s.total_satker),
                        ),
                        None => ("-".into(), "-".into(), "-".into(), "-".into()),
                    };

                    view! {
                        <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
                            <StatCard icon="fas fa-box"                  label="Total Aset BMN"  value=total  subtitle="Terintegrasi SIMAN"   tone="blue" />
                            <StatCard icon="fas fa-check-circle"         label="Kondisi Baik"    value=baik   subtitle="Siap pakai"           tone="green" />
                            <StatCard icon="fas fa-exclamation-triangle" label="Perlu Perbaikan" value=rusak  subtitle="Tindakan diperlukan"  tone="amber" />
                            <StatCard icon="fas fa-building"             label="Satuan Kerja"    value=satker subtitle="Unit kerja aktif"     tone="violet" />
                        </div>
                    }
                }}
            </Suspense>

            <section>
                <SectionHeader title="Modul Utama" tone="gold" />
                <div class="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
                    <QuickNav href=routes::path::BANK_ASET_DAFTAR      icon="fas fa-boxes"          label="Bank Aset"       description="Katalog dan registrasi BMN"          tone="emerald" />
                    <QuickNav href=routes::path::KEBUTUHAN_DAFTAR      icon="fas fa-clipboard-list" label="Kebutuhan BMN"   description="Analisis kebutuhan dan perencanaan"  tone="blue" />
                    <QuickNav href=routes::path::PAKAIAN_PENGAJUAN     icon="fas fa-tshirt"         label="Pakaian Dinas"   description="Pengajuan dan distribusi atribut"    tone="purple" />
                    <QuickNav href=routes::path::PENGELOLAAN_PEMAKAIAN icon="fas fa-file-signature" label="Pemakaian BMN"   description="Izin pemakaian dan monitoring"       tone="indigo" />
                    <QuickNav href=routes::path::PENGELOLAAN_PENGHAPUSAN icon="fas fa-trash-alt"    label="Penghapusan BMN" description="Disposal dan penghapusan BMN"        tone="red" />
                </div>
            </section>

            <section>
                <SectionHeader title="Analitik" tone="teal" />
                <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
                    <QuickNav href=routes::path::ANALITIK_ROADMAP    icon="fas fa-road"    label="Roadmap Sarpras" description="Prediksi kebutuhan sarana prasarana" tone="teal" />
                    <QuickNav href=routes::path::ANALITIK_KODEFIKASI icon="fas fa-barcode" label="Kodefikasi BMN"  description="Mapping kode barang standar"         tone="purple" />
                </div>
            </section>

            <Show when=move || is_admin>
                <section>
                    <SectionHeader title="Panel Administrator" tone="red" />
                    <div class="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
                        <QuickNav href=routes::path::ADMIN_USERS      icon="fas fa-users-cog"      label="Manajemen Pengguna" description="Data pengguna dan sesi aktif"         tone="red" />
                        <QuickNav href=routes::path::ADMIN_ROLES      icon="fas fa-user-tag"       label="Otorisasi (RBAC)"   description="Konfigurasi hak akses peran"       tone="orange" />
                        <QuickNav href=routes::path::ADMIN_AUDIT      icon="fas fa-history"        label="Audit Log"          description="Jejak audit seluruh aktivitas"       tone="indigo" />
                        <QuickNav href=routes::path::ADMIN_MASTER     icon="fas fa-database"       label="Master Data"        description="Pengelolaan data referensi"          tone="cyan" />
                        <QuickNav href=routes::path::ADMIN_WORKFLOW   icon="fas fa-project-diagram" label="Workflow Config"    description="Konfigurasi workflow persetujuan"    tone="blue" />
                    </div>
                </section>
            </Show>
        </div>
    }
}
