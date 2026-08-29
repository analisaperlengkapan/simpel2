//! Dashboard Home for Perlengkapan.
//!
//! This page focuses on clear information hierarchy, responsive layout,
//! and maintainable utility-class styling.

use crate::api::bank_aset::{BankAsetDashboard, fetch_dashboard};
use crate::components::role_switcher::use_active_role;
use crate::routes;
use chrono::Datelike;
use leptos::prelude::*;
use leptos_fetch::QueryClient;
use leptos_meta::Title;
use lib_ui::components::icon::{AppIcon, icon_from_fa_class};
use phosphor_leptos::{BUILDINGS, CALENDAR, CARET_RIGHT, SHIELD};

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
        "red" => ("bg-rose-500/15", "text-rose-300", "shadow-rose-500/30"),
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
                        icon_bg,
                        glow,
                    )>
                        <span class=format!("inline-flex {}", icon_text)>
                            <AppIcon icon=icon_from_fa_class(icon) size=16 />
                        </span>
                    </div>
                </div>
                <p class="text-xs font-semibold uppercase tracking-[0.08em] text-slate-400">
                    {label}
                </p>
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
        <a
            href=href
            class="group block rounded-xl border border-white/10 bg-slate-900/50 p-4 transition-all duration-300 hover:border-white/20 hover:bg-slate-900/80"
        >
            <div class="flex items-start gap-3">
                <div class=format!(
                    "mt-0.5 flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg {} {} ring-1 ring-transparent transition-all",
                    icon_bg,
                    ring,
                )>
                    <span class=format!("inline-flex {}", icon_text)>
                        <AppIcon icon=icon_from_fa_class(icon) size=14 />
                    </span>
                </div>
                <div class="min-w-0 flex-1">
                    <h3 class="text-sm font-semibold text-slate-100 transition-colors group-hover:text-gold-300">
                        {label}
                    </h3>
                    <p class="mt-1 text-xs leading-relaxed text-slate-500">{description}</p>
                </div>
                <span class="pt-1 text-[10px] text-slate-600 transition-colors group-hover:text-slate-300">
                    <AppIcon icon=CARET_RIGHT />
                </span>
            </div>
        </a>
    }
}

/// leptos-fetch query wrapper for the SIMAN summary.
///
/// Reads `GET /bank-aset/dashboard` — the SAME endpoint the Bank Aset page
/// uses. The home page used to call `/dashboard/stats`, a second aggregate over
/// the same table whose payload was a strict subset of this one and which was
/// never row-scoped: measured on staging, an operator at satker 0200010 was
/// shown 624 533 assets across 556 satkers here while the Bank Aset page
/// correctly showed their own 1 681. One endpoint means the two pages cannot
/// disagree, and the scope is inherited rather than re-implemented.
async fn query_dashboard_stats(_: ()) -> Result<BankAsetDashboard, crate::api::AppError> {
    // No `map_err(Into::into)`: both sides are already `api::error::AppError`,
    // and clippy's `useless_conversion` is denied in CI.
    fetch_dashboard().await
}

/// Assets in a given condition, from the SIMAN `kondisi_breakdown`.
///
/// The breakdown keeps SIMAN's own labels ("Baik", "Rusak Ringan", "Rusak
/// Berat"), so "needs repair" is every RUSAK* bucket summed rather than a
/// single field. `/dashboard/stats` used to do this sum in SQL with
/// `LIKE 'RUSAK%'`; doing it here keeps the endpoint generic and the two
/// readers honest about which buckets they folded together.
fn count_kondisi(d: &BankAsetDashboard, pred: impl Fn(&str) -> bool) -> i64 {
    d.kondisi_breakdown
        .iter()
        .filter(|k| pred(&k.kondisi.to_uppercase()))
        .map(|k| k.count)
        .sum()
}

/// Fetch the dashboard recap as bytes and hand it to the browser as a download.
/// Mirrors `components::laporan_kebutuhan_bmn::download_export` — same blob +
/// synthetic-anchor pattern, kept identical so there is one way to do this.
#[cfg(target_arch = "wasm32")]
fn download_dashboard_export(format: &'static str) {
    use leptos::task::spawn_local;
    use wasm_bindgen::JsCast;
    use web_sys::{Blob, BlobPropertyBag, Url};

    spawn_local(async move {
        // The endpoint requires `tahun_anggaran`; the recap is for the running
        // budget year, which is what the dashboard cards above already show.
        let tahun = js_sys::Date::new_0().get_full_year() as i32;
        let bytes = match crate::api::dashboard::export_dashboard(format, tahun).await {
            Ok(b) => b,
            Err(e) => {
                leptos::logging::error!("export dashboard {format} gagal: {e}");
                return;
            }
        };
        let mime = if format == "pdf" {
            "application/pdf"
        } else {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        };
        let array = js_sys::Uint8Array::from(bytes.as_slice());
        let parts = js_sys::Array::new();
        parts.push(&array);
        let opts = BlobPropertyBag::new();
        opts.set_type(mime);
        let Ok(blob) = Blob::new_with_u8_array_sequence_and_options(&parts, &opts) else {
            return;
        };
        let Ok(url) = Url::create_object_url_with_blob(&blob) else {
            return;
        };
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if let Ok(a) = doc.create_element("a") {
                let a: web_sys::HtmlAnchorElement = a.unchecked_into();
                a.set_href(&url);
                let ext = if format == "pdf" { "pdf" } else { "xlsx" };
                a.set_download(&format!("Rekap_Dashboard_Perlengkapan_{tahun}.{ext}"));
                a.click();
            }
        }
        let _ = Url::revoke_object_url(&url);
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn download_dashboard_export(_format: &'static str) {}

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
    // Reactive: closures below re-run on a role switch, so the greeting and
    // the admin-only panels follow immediately instead of waiting for a
    // browser refresh.
    let active_role = use_active_role();
    let role_label = move || match active_role.get().as_str() {
        "validator_wilayah" => "Validator Wilayah",
        "validator_pusat" => "Validator Pusat",
        "admin" => "Administrator",
        _ => "Operator Satker",
    };

    // What this page actually covers, said out loud. The numbers below are
    // scoped per tier, so a heading that reads the same for everyone invites
    // exactly the misreading the scoping was meant to end: an operator seeing
    // "Ringkasan Sistem Manajemen" above 1 681 assets has no way to tell
    // whether that is the country or their own office.
    let scope_word = move || match active_role.get().as_str() {
        "validator_wilayah" => "Wilayah Anda",
        "validator_pusat" | "admin" => "Nasional",
        _ => "Satuan Kerja Anda",
    };
    let scope_sentence = move || match active_role.get().as_str() {
        "validator_wilayah" => {
            "Aset BMN, kebutuhan, dan perlengkapan pada seluruh satuan kerja di wilayah Anda."
        }
        "validator_pusat" | "admin" => {
            "Aset BMN, kebutuhan, dan perlengkapan pada seluruh satuan kerja Kejaksaan RI."
        }
        _ => "Aset BMN, kebutuhan, dan perlengkapan pada satuan kerja Anda.",
    };

    // Derived, not written down. This chip read "Tahun Anggaran 2025" as a
    // hard-coded literal: wrong from 1 January 2026 onward, and already
    // disagreeing with the export button on this same page, which has always
    // taken the year from the clock.
    let tahun_anggaran = move || chrono::Local::now().year();

    // leptos-fetch — `()` keyed cache so a tab-switch back to the
    // dashboard re-uses the previous load instantly.
    let client: QueryClient = expect_context();
    let stats_resource = client.local_resource(query_dashboard_stats, || ());
    let is_admin = move || active_role.get() == "admin";

    view! {
        <Title text="Dashboard — SIMPEL Perlengkapan" />
        <div class="mx-auto max-w-7xl space-y-8 px-1 sm:px-2">
            <section class="relative overflow-hidden rounded-3xl border border-white/10 bg-gradient-to-br from-navy-900 via-navy-800 to-slate-950 p-6 shadow-[0_18px_50px_rgba(0,0,0,0.45)] sm:p-8 lg:p-10">
                <div class="pointer-events-none absolute -right-16 -top-16 h-56 w-56 rounded-full bg-gold-400/10 blur-3xl"></div>
                <div class="pointer-events-none absolute -bottom-16 left-1/4 h-44 w-44 rounded-full bg-blue-400/10 blur-3xl"></div>

                <div class="relative z-10">
                    <div class="inline-flex items-center gap-2 rounded-full border border-white/15 bg-white/5 px-3 py-1 text-xs font-semibold text-gold-300">
                        <span class="h-2 w-2 rounded-full bg-emerald-400"></span>
                        // "Portal" is a DIFFERENT application here
                        // (antarmuka/portal, the SSO landing app). Naming this
                        // microfrontend "Portal" too made the two
                        // indistinguishable in the one place a user looks to
                        // find out where they are.
                        "Perlengkapan · Kejaksaan RI"
                    </div>

                    <h1 class="mt-4 text-2xl font-black leading-tight text-white sm:text-4xl">
                        "Ringkasan "
                        <span class="bg-gradient-to-r from-gold-300 to-amber-400 bg-clip-text text-transparent">
                            {scope_word}
                        </span>
                    </h1>

                    <p class="mt-3 max-w-2xl text-sm leading-relaxed text-slate-300 sm:text-base">
                        {scope_sentence}
                    </p>

                    <div class="mt-6 flex flex-wrap gap-3">
                        <div class="inline-flex items-center gap-2 rounded-xl border border-white/10 bg-navy-950/50 px-3 py-2 text-xs text-slate-300 sm:text-sm">
                            <span class="text-emerald-300">
                                <AppIcon icon=SHIELD />
                            </span>
                            <span>"Role: " <strong class="text-white">{role_label}</strong></span>
                        </div>
                        <div class="inline-flex items-center gap-2 rounded-xl border border-white/10 bg-navy-950/50 px-3 py-2 text-xs text-slate-300 sm:text-sm">
                            <span class="text-blue-300">
                                <AppIcon icon=CALENDAR />
                            </span>
                            "Tahun Anggaran "
                            {tahun_anggaran}
                        </div>
                        // Coverage, beside the role that grants it. This used
                        // to be a headline metric ("Satuan Kerja / Unit kerja
                        // aktif"), wrong twice over: nothing here measures
                        // whether a satker is active, and once scoped the number
                        // is a constant 1 for every operator — the majority of
                        // users — while the four headline slots are needed for
                        // figures that actually move.
                        <div class="inline-flex items-center gap-2 rounded-xl border border-white/10 bg-navy-950/50 px-3 py-2 text-xs text-slate-300 sm:text-sm">
                            <span class="text-violet-300">
                                <AppIcon icon=BUILDINGS />
                            </span>
                            "Cakupan: "
                            <strong class="text-white">
                                {move || {
                                    stats_resource
                                        .get()
                                        .and_then(|r| r.ok())
                                        .map(|s| format_number(s.total_satker))
                                        .unwrap_or_else(|| "—".to_string())
                                }}
                            </strong>
                            " satuan kerja"
                        </div>
                    </div>
                </div>
            </section>

            <Suspense fallback=move || {
                view! {
                    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
                        <StatCardSkeleton />
                        <StatCardSkeleton />
                        <StatCardSkeleton />
                        <StatCardSkeleton />
                    </div>
                }
            }>
                {move || {
                    let stats = stats_resource.get().and_then(|r| r.ok());
                    let (total, baik, ringan, berat) = match &stats {
                        Some(s) => {
                            (
                                format_number(s.total_aset),
                                format_number(count_kondisi(s, |k| k == "BAIK")),
                                format_number(count_kondisi(s, |k| k == "RUSAK RINGAN")),
                                format_number(count_kondisi(s, |k| k == "RUSAK BERAT")),
                            )
                        }
                        None => ("-".into(), "-".into(), "-".into(), "-".into()),
                    };
                    // "Perlu Perbaikan" used to be ONE card summing every
                    // RUSAK* bucket. Measured on staging that card read 76 459,
                    // of which 64 522 (84%) are "Rusak Berat" — in BMN practice
                    // a candidate for PENGHAPUSAN, not repair. The single label
                    // therefore pointed the reader at the wrong next action for
                    // most of what it counted, so the bucket is split and each
                    // half names the action it actually implies.

                    view! {
                        <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
                            <StatCard
                                icon="fas fa-box"
                                label="Total Aset BMN"
                                value=total
                                subtitle="Sumber: SIMAN"
                                tone="blue"
                            />
                            <StatCard
                                icon="fas fa-check-circle"
                                label="Kondisi Baik"
                                value=baik
                                subtitle="Layak digunakan"
                                tone="green"
                            />
                            <StatCard
                                icon="fas fa-screwdriver-wrench"
                                label="Rusak Ringan"
                                value=ringan
                                subtitle="Kandidat perbaikan"
                                tone="amber"
                            />
                            <StatCard
                                icon="fas fa-trash-can"
                                label="Rusak Berat"
                                value=berat
                                subtitle="Kandidat penghapusan"
                                tone="red"
                            />
                        </div>
                    }
                }}
            </Suspense>

            // Rekap dashboard export (#97/#116). These two endpoints existed but
            // had no FE caller at all, which is why the 500 they returned went
            // unnoticed until an e2e probed them directly.
            <section class="flex flex-wrap items-center gap-3">
                <span class="text-xs font-semibold uppercase tracking-[0.08em] text-slate-400">
                    "Unduh rekap"
                </span>
                <button
                    type="button"
                    class="rounded-lg border border-white/10 bg-slate-900/60 px-3 py-2 text-xs font-semibold text-slate-100 transition-colors hover:border-white/20 hover:bg-slate-900/80"
                    on:click=move |_| download_dashboard_export("excel")
                >
                    "Excel"
                </button>
                <button
                    type="button"
                    class="rounded-lg border border-white/10 bg-slate-900/60 px-3 py-2 text-xs font-semibold text-slate-100 transition-colors hover:border-white/20 hover:bg-slate-900/80"
                    on:click=move |_| download_dashboard_export("pdf")
                >
                    "PDF"
                </button>
            </section>

            <section>
                <SectionHeader title="Modul Utama" tone="gold" />
                <div class="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
                    <QuickNav
                        href=routes::path::BANK_ASET_DAFTAR
                        icon="fas fa-boxes"
                        label="Bank Aset"
                        description="Katalog aset BMN dari SIMAN"
                        tone="emerald"
                    />
                    <QuickNav
                        href=routes::path::KEBUTUHAN_DAFTAR
                        icon="fas fa-clipboard-list"
                        label="Kebutuhan BMN"
                        description="Pengajuan RKBMN dan analisis kelayakan"
                        tone="blue"
                    />
                    <QuickNav
                        href=routes::path::PAKAIAN_PENGAJUAN
                        icon="fas fa-tshirt"
                        label="Pakaian Dinas"
                        description="Pengajuan dan rekap ukuran pakaian dinas"
                        tone="purple"
                    />
                    <QuickNav
                        href=routes::path::PENGELOLAAN_PEMAKAIAN
                        icon="fas fa-file-signature"
                        label="Pemakaian BMN"
                        description="Izin pemakaian dan pemantauannya"
                        tone="indigo"
                    />
                    <QuickNav
                        href=routes::path::PENGELOLAAN_PENGHAPUSAN
                        icon="fas fa-trash-alt"
                        label="Penghapusan BMN"
                        description="Usulan SK penghapusan aset"
                        tone="red"
                    />
                </div>
            </section>

            <section>
                <SectionHeader title="Analitik" tone="teal" />
                <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
                    <QuickNav
                        href=routes::path::ANALITIK_ROADMAP
                        icon="fas fa-road"
                        label="Roadmap Sarpras"
                        description="Prediksi kebutuhan sarana prasarana"
                        tone="teal"
                    />
                </div>
            </section>

            <Show when=is_admin>
                <section>
                    <SectionHeader title="Panel Administrator" tone="red" />
                    <div class="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
                        // No "Manajemen Pengguna" tile: that page was removed
                        // (user administration lives in portal, against authenc).
                        <QuickNav
                            href=routes::path::ADMIN_ROLES
                            icon="fas fa-user-tag"
                            label="Otorisasi Peran"
                            description="Konfigurasi hak akses peran"
                            tone="orange"
                        />
                        <QuickNav
                            href=routes::path::ADMIN_AUDIT
                            icon="fas fa-history"
                            label="Jejak Audit"
                            description="Riwayat seluruh aktivitas pengguna"
                            tone="indigo"
                        />
                        <QuickNav
                            href=routes::path::ADMIN_MASTER
                            icon="fas fa-database"
                            label="Master Data"
                            description="Pengelolaan data referensi"
                            tone="cyan"
                        />
                        <QuickNav
                            href=routes::path::ADMIN_WORKFLOW
                            icon="fas fa-project-diagram"
                            label="Konfigurasi Alur Kerja"
                            description="Alur dan tahapan persetujuan"
                            tone="blue"
                        />
                    </div>
                </section>
            </Show>
        </div>
    }
}
