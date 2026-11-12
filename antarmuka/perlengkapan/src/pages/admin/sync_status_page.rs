//! `/admin/status-sinkronisasi`: are the master-data syncs running, and is anything wrong.
//!
//! Every form that fills a person in from a NIP, or lets the operator pick an asset, reads a copy
//! of MySIMKARI (satker, employees) and SIMAN (assets) that integrasi's CronJobs refresh. When a
//! sync stops, nothing on those forms fails: they answer from an older copy, or find nobody. This
//! page shows what every run recorded (`GET /admin/status-sinkronisasi`, reading
//! `integrasi.sync_status`), with the server's judgement per row, so an administrator can see it
//! without cluster access.

use leptos::prelude::*;
use leptos::task::spawn_local;
use lib_ui::hooks::LatestOnly;
use lib_ui::utils::formatters::format_iso_local_opt;

use crate::api::admin::{SyncStatusEntry, fetch_sync_status};
use crate::api::error::AppError;
use crate::components::layout::{
    EmptyState, ErrorState, LoadingState, PageBreadcrumb, PageLayout, SectionCard,
};

#[component]
pub fn AdminSyncStatusPage() -> impl IntoView {
    let (entries, set_entries) = signal::<Vec<SyncStatusEntry>>(Vec::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<AppError>>(None);
    let (reload_tick, set_reload_tick) = signal(0_u32);
    // A slow answer to an earlier "Muat ulang" must not overwrite a newer one.
    let latest = LatestOnly::new();

    Effect::new(move |_| {
        let _ = reload_tick.get();
        set_loading.set(true);
        set_error.set(None);
        let ticket = latest.ticket();
        spawn_local(async move {
            let result = fetch_sync_status().await;
            if !latest.is_current(ticket) {
                return;
            }
            match result {
                Ok(rows) => set_entries.set(rows),
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    let breadcrumbs = vec![
        PageBreadcrumb::new("Dashboard", crate::routes::path::DASHBOARD),
        PageBreadcrumb::new("Admin", crate::routes::path::ADMIN_WORKFLOW),
        PageBreadcrumb::leaf("Status Sinkronisasi"),
    ];

    // Rows an administrator has to act on.
    let attention = move || {
        entries.with(|rows| {
            rows.iter()
                .filter(|r| matches!(r.keadaan.as_str(), "GAGAL" | "TERLAMBAT"))
                .count()
        })
    };

    view! {
        <PageLayout
            title="Status Sinkronisasi"
            description="Hasil setiap sinkronisasi data induk: pegawai dan satuan kerja dari MySIMKARI, aset BMN dari SIMAN. Form yang mengisi data dari NIP atau memilih aset membaca salinan ini."
            icon="fas fa-sync-alt"
            breadcrumbs=breadcrumbs
        >
            <SectionCard title="Sumber data induk" icon="fas fa-database">
                <div class="mb-3 flex flex-wrap items-center justify-between gap-3">
                    <p class="text-xs text-slate-400">
                        "Waktu ditampilkan menurut jam perangkat Anda. Data dicatat oleh setiap run CronJob integrasi."
                    </p>
                    <button
                        type="button"
                        data-testid="sync-reload"
                        class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] px-4 py-2 text-sm font-medium text-slate-200 transition hover:bg-white/[0.08] disabled:opacity-50"
                        disabled=move || loading.get()
                        on:click=move |_| set_reload_tick.update(|t| *t += 1)
                    >
                        <i class="fas fa-redo mr-1.5"></i>
                        "Muat ulang"
                    </button>
                </div>
                {move || {
                    if loading.get() && entries.with(Vec::is_empty) {
                        view! { <LoadingState message="Memuat status sinkronisasi..." /> }.into_any()
                    } else if let Some(e) = error.get() {
                        let retry: Box<dyn Fn()> = Box::new(move || {
                            set_reload_tick.update(|t| *t += 1);
                        });
                        view! { <ErrorState error=e on_retry=retry /> }.into_any()
                    } else if entries.with(Vec::is_empty) {
                        view! {
                            <EmptyState
                                title="Belum ada sumber tercatat"
                                description="Tabel integrasi.sync_status kosong. Migrasi integrasi belum dijalankan di lingkungan ini."
                                icon="fas fa-inbox"
                            />
                        }
                            .into_any()
                    } else {
                        let n = attention();
                        view! {
                            {(n != 0)
                                .then(|| {
                                    view! {
                                        <div
                                            data-testid="sync-attention"
                                            class="mb-3 rounded-lg border border-danger-500/30 bg-danger-500/10 px-3 py-2 text-sm text-danger-300"
                                        >
                                            <i class="fas fa-exclamation-triangle mr-1.5"></i>
                                            {format!(
                                                "{n} sumber perlu perhatian: sinkronisasi terakhirnya gagal atau jadwalnya tidak berjalan.",
                                            )}
                                        </div>
                                    }
                                })}
                            <SyncTable rows=entries />
                        }
                            .into_any()
                    }
                }}
            </SectionCard>

            <SectionCard title="Arti keadaan" icon="fas fa-info-circle" dense=true>
                <dl class="grid grid-cols-1 gap-2 text-xs md:grid-cols-2">
                    {KEADAAN
                        .iter()
                        .map(|(code, label, class, meaning)| {
                            view! {
                                <div class="flex items-start gap-2">
                                    <dt>
                                        <span
                                            class=format!(
                                                "inline-flex rounded-lg border px-2 py-0.5 text-[0.7rem] font-semibold whitespace-nowrap {class}",
                                            )
                                            data-keadaan=*code
                                        >
                                            {*label}
                                        </span>
                                    </dt>
                                    <dd class="text-slate-400">{*meaning}</dd>
                                </div>
                            }
                        })
                        .collect_view()}
                </dl>
            </SectionCard>
        </PageLayout>
    }
}

/// `(code, label, badge classes, meaning)` of every judgement the server makes.
const KEADAAN: &[(&str, &str, &str, &str)] = &[
    (
        "BERHASIL",
        "Berhasil",
        "border-success-500/30 bg-success-500/10 text-success-400",
        "Run terakhir selesai. Jumlah data = baris yang kini dibaca aplikasi.",
    ),
    (
        "BERJALAN",
        "Sedang berjalan",
        "border-info-500/30 bg-info-500/10 text-info-300",
        "Run sedang berlangsung.",
    ),
    (
        "GAGAL",
        "Gagal",
        "border-danger-500/30 bg-danger-500/10 text-danger-400",
        "Run terakhir gagal; alasannya tertulis di Keterangan. Aplikasi masih memakai salinan dari \"Terakhir berhasil\".",
    ),
    (
        "TERLAMBAT",
        "Terlambat",
        "border-warning-500/30 bg-warning-500/10 text-warning-400",
        "Jadwal berikutnya sudah lewat lebih dari satu jam tanpa run baru: CronJob tidak berjalan, atau run sebelumnya terhenti di tengah jalan.",
    ),
    (
        "BELUM_PERNAH",
        "Belum pernah berjalan",
        "border-white/[0.08] bg-white/[0.04] text-slate-300",
        "Belum ada run yang tercatat untuk sumber ini (mis. sumber dinonaktifkan, atau token API belum diisi).",
    ),
];

fn keadaan_badge(code: &str) -> (&'static str, &'static str) {
    KEADAAN
        .iter()
        .find(|(c, ..)| *c == code)
        .map(|(_, label, class, _)| (*label, *class))
        .unwrap_or((
            "Tidak dikenal",
            "border-white/[0.08] bg-white/[0.04] text-slate-300",
        ))
}

/// What a person calls the source and the part.
fn sumber_label(sumber: &str, bagian: &str) -> (String, String) {
    let source = match sumber {
        "MonSAKTI_ADM" => "MonSAKTI",
        other => other,
    };
    let part = match (sumber, bagian) {
        ("MySIMKARI", "satker") => "Satuan kerja",
        ("MySIMKARI", "pegawai") => "Pegawai aktif",
        ("SIMAN", "aset") => "Aset BMN",
        (_, "ref_admin") => "Referensi administrasi",
        (_, "ref_bank") => "Referensi bank",
        (_, "ref_jns_spp") => "Referensi jenis SPP",
        (_, other) => other,
    };
    (source.to_string(), part.to_string())
}

const TH: &str = "border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400";

#[component]
fn SyncTable(rows: ReadSignal<Vec<SyncStatusEntry>>) -> impl IntoView {
    view! {
        <div class="overflow-x-auto rounded-xl border border-white/[0.06]">
            <table class="min-w-full border-collapse">
                <thead>
                    <tr class="bg-white/[0.02]">
                        <th class=TH>"Sumber"</th>
                        <th class=TH>"Keadaan"</th>
                        <th class=TH>"Terakhir berhasil"</th>
                        <th class=TH>"Run terakhir"</th>
                        <th class=TH>"Jadwal berikutnya"</th>
                        <th class=TH>"Jumlah data"</th>
                        <th class=TH>"Keterangan"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        rows.get()
                            .into_iter()
                            .map(|r| {
                                let (source, part) = sumber_label(&r.sumber, &r.bagian);
                                let (label, class) = keadaan_badge(&r.keadaan);
                                let ran = r.mulai_terakhir.is_some();
                                let run = if ran {
                                    format!(
                                        "{} → {}",
                                        format_iso_local_opt(r.mulai_terakhir.as_deref()),
                                        if r.status_terakhir == "running" {
                                            "berjalan".to_string()
                                        } else {
                                            format_iso_local_opt(r.selesai_terakhir.as_deref())
                                        },
                                    )
                                } else {
                                    "—".to_string()
                                };
                                let note = r
                                    .galat
                                    .clone()
                                    .filter(|g| !g.trim().is_empty())
                                    .unwrap_or_else(|| "—".to_string());
                                let counts = format!(
                                    "{} berhasil · {} gagal",
                                    r.total_berhasil,
                                    r.total_gagal,
                                );
                                view! {
                                    <tr
                                        class="border-b border-white/[0.04] align-top text-xs"
                                        data-testid="sync-row"
                                        data-sumber=r.sumber.clone()
                                        data-bagian=r.bagian.clone()
                                        data-keadaan=r.keadaan.clone()
                                    >
                                        <td class="px-3 py-2">
                                            <div class="font-semibold text-white">{source}</div>
                                            <div class="text-[0.7rem] text-slate-400">{part}</div>
                                        </td>
                                        <td class="px-3 py-2">
                                            <span
                                                data-testid="sync-keadaan"
                                                class=format!(
                                                    "inline-flex rounded-lg border px-2 py-0.5 text-[0.7rem] font-semibold whitespace-nowrap {class}",
                                                )
                                            >
                                                {label}
                                            </span>
                                            <div class="mt-1 text-[0.65rem] text-slate-500">
                                                {counts}
                                            </div>
                                        </td>
                                        <td
                                            class="px-3 py-2 whitespace-nowrap text-slate-300"
                                            data-testid="sync-berhasil"
                                        >
                                            {format_iso_local_opt(r.berhasil_terakhir.as_deref())}
                                        </td>
                                        <td class="px-3 py-2 whitespace-nowrap text-slate-400">
                                            {run}
                                        </td>
                                        <td class="px-3 py-2 text-slate-400">
                                            <div class="whitespace-nowrap">
                                                {format_iso_local_opt(
                                                    r.jadwal_berikutnya.as_deref(),
                                                )}
                                            </div>
                                            <div class="text-[0.65rem] text-slate-500 font-mono">
                                                {r.jadwal.clone().unwrap_or_default()}
                                            </div>
                                        </td>
                                        <td
                                            class="px-3 py-2 text-right font-mono text-slate-200"
                                            data-testid="sync-jumlah"
                                        >
                                            {r.jumlah_data.to_string()}
                                        </td>
                                        <td
                                            class="px-3 py-2 text-slate-300 break-words max-w-md"
                                            data-testid="sync-galat"
                                        >
                                            {note}
                                        </td>
                                    </tr>
                                }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_judgement_the_server_makes_has_a_label() {
        // The five values `admin::sync_status` can answer with.
        for code in ["BERHASIL", "BERJALAN", "GAGAL", "TERLAMBAT", "BELUM_PERNAH"] {
            assert_ne!(keadaan_badge(code).0, "Tidak dikenal", "{code}");
        }
    }

    #[test]
    fn sources_are_named_as_people_know_them() {
        assert_eq!(
            sumber_label("MySIMKARI", "pegawai"),
            ("MySIMKARI".to_string(), "Pegawai aktif".to_string())
        );
        assert_eq!(
            sumber_label("MonSAKTI_ADM", "ref_bank"),
            ("MonSAKTI".to_string(), "Referensi bank".to_string())
        );
    }
}
