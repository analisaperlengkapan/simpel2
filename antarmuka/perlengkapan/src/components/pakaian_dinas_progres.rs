//! Pakaian Dinas — monitoring: how far a campaign ("kegiatan") has got.
//!
//! Shown at the top of the satker list of one campaign. It answers, in this order and
//! no further than the reader asks: how is the whole campaign doing (a handful of
//! figures and one bar), which wilayah are behind (a table, least filled first, the
//! first few rows and "show all"), and which satker are furthest behind (a short list
//! somebody can chase, each with a way to the employees' page). The employees
//! themselves are not repeated here: the roster page already lists who has filled in
//! and who has not.
//!
//! The figures come from one server answer that counts the way the confirmation
//! dialogs count, inside what the caller may see: a wilayah validator sees their
//! wilayah, pusat the whole country.

use leptos::prelude::*;
use leptos_fetch::QueryClient;

use crate::api::{AppError, aktivitas_label};
use crate::components::layout::{ErrorState, LoadingState, SectionCard};
use crate::features::pakaian_dinas::api::{
    ProgresPakaianDinas, ProgresRingkasan, ProgresSatker, ProgresWilayah,
    fetch_progres_pakaian_dinas, persen,
};

/// Rows of the wilayah table shown before "Tampilkan semua".
const WILAYAH_AWAL: usize = 8;

type KunciProgres = (String, i32);

async fn query_progres(key: KunciProgres) -> Result<ProgresPakaianDinas, AppError> {
    fetch_progres_pakaian_dinas(&key.0).await
}

/// "12 / 15 (80%)", or "-" when nobody is expected to fill in.
fn teks_terisi(terisi: i64, total: i64) -> String {
    match persen(terisi, total) {
        Some(p) => format!("{terisi} / {total} ({p}%)"),
        None => "-".to_string(),
    }
}

/// Width of a bar segment as a CSS percentage string; an empty whole is no segment.
fn lebar(bagian: i64, total: i64) -> String {
    format!("{}%", persen(bagian, total).unwrap_or(0))
}

/// A bar split by workflow stage: not submitted / at the wilayah / at pusat / finished.
fn bar_tahap(r: ProgresRingkasan) -> impl IntoView {
    let total = r.satker_total;
    view! {
        <div
            class="flex h-2.5 w-full overflow-hidden rounded-full bg-white/10"
            role="img"
            aria-label=format!(
                "{} belum mengajukan, {} di wilayah, {} di pusat, {} selesai",
                r.belum_mengajukan,
                r.di_wilayah,
                r.di_pusat,
                r.selesai,
            )
        >
            <div class="bg-success-500" style=format!("width:{}", lebar(r.selesai, total))></div>
            <div class="bg-info-500" style=format!("width:{}", lebar(r.di_pusat, total))></div>
            <div class="bg-warning-500" style=format!("width:{}", lebar(r.di_wilayah, total))></div>
            <div
                class="bg-danger-500"
                style=format!("width:{}", lebar(r.belum_mengajukan, total))
            ></div>
        </div>
    }
}

/// A bar of employees who have given a size.
fn bar_pegawai(terisi: i64, total: i64) -> impl IntoView {
    view! {
        <div class="h-2 w-full overflow-hidden rounded-full bg-white/10">
            <div class="h-full bg-success-500" style=format!("width:{}", lebar(terisi, total))></div>
        </div>
    }
}

/// A figure card. `titik` is the colour of its segment in the stage bar, so the card is
/// the legend of the bar; `None` for a figure that is not a stage.
fn kartu(
    label: &'static str,
    nilai: String,
    nada: &'static str,
    titik: Option<&'static str>,
    testid: &'static str,
) -> impl IntoView {
    view! {
        <div
            data-testid=testid
            class="rounded-xl border border-white/[0.06] bg-white/[0.02] px-4 py-3"
        >
            <p class="flex items-center gap-1.5 text-xs text-slate-400">
                {titik
                    .map(|warna| {
                        view! {
                            <span class=format!("inline-block h-2 w-2 rounded-full {warna}")></span>
                        }
                    })}
                {label}
            </p>
            <p class=format!("mt-1 text-xl font-semibold {nada}")>{nilai}</p>
        </div>
    }
}

fn ringkasan_kegiatan(r: ProgresRingkasan) -> impl IntoView {
    let pegawai = teks_terisi(r.pegawai_terisi, r.pegawai_total);
    let bar = bar_tahap(r.clone());
    view! {
        <div data-testid="progres-ringkasan" class="space-y-3">
            <div class="grid grid-cols-2 gap-3 lg:grid-cols-5">
                {kartu(
                    "Belum mengajukan",
                    r.belum_mengajukan.to_string(),
                    "text-danger-300",
                    Some("bg-danger-500"),
                    "progres-belum-mengajukan",
                )}
                {kartu(
                    "Di wilayah",
                    r.di_wilayah.to_string(),
                    "text-warning-300",
                    Some("bg-warning-500"),
                    "progres-di-wilayah",
                )}
                {kartu(
                    "Di pusat",
                    r.di_pusat.to_string(),
                    "text-info-300",
                    Some("bg-info-500"),
                    "progres-di-pusat",
                )}
                {kartu(
                    "Selesai",
                    r.selesai.to_string(),
                    "text-success-300",
                    Some("bg-success-500"),
                    "progres-selesai",
                )}
                {kartu("Pegawai sudah mengisi", pegawai, "text-slate-100", None, "progres-pegawai")}
            </div>
            {bar}
            <p class="text-xs text-slate-500">
                {format!("{} satker", r.satker_total)}
                {(r.tanpa_pegawai > 0)
                    .then(|| format!(" (termasuk {} tanpa pegawai di roster)", r.tanpa_pegawai))}
                ". Batang: hijau selesai, biru di pusat, kuning di wilayah, merah belum mengajukan."
            </p>
        </div>
    }
}

fn baris_wilayah(w: ProgresWilayah) -> impl IntoView {
    let r = w.ringkasan;
    let nama = w
        .wilayah_nama
        .unwrap_or_else(|| "Di luar Kejati (pusat)".to_string());
    let pegawai = teks_terisi(r.pegawai_terisi, r.pegawai_total);
    view! {
        <tr class="border-t border-white/[0.04] align-top">
            <td class="py-2 pr-3 text-sm text-slate-200">{nama}</td>
            <td class="py-2 pr-3 text-sm text-slate-300">{r.satker_total}</td>
            <td class="hidden py-2 pr-3 sm:table-cell">
                <div class="w-40 space-y-1">
                    {bar_tahap(r.clone())}
                    <span class="text-xs text-slate-500">
                        {format!(
                            "{} / {} / {} / {}",
                            r.belum_mengajukan,
                            r.di_wilayah,
                            r.di_pusat,
                            r.selesai,
                        )}
                    </span>
                </div>
            </td>
            <td class="py-2 text-sm text-slate-300">
                <div class="space-y-1">
                    <span>{pegawai}</span>
                    {bar_pegawai(r.pegawai_terisi, r.pegawai_total)}
                </div>
            </td>
        </tr>
    }
}

fn tabel_wilayah(wilayah: Vec<ProgresWilayah>) -> impl IntoView {
    let semua = RwSignal::new(false);
    let jumlah = wilayah.len();
    let wilayah = StoredValue::new(wilayah);
    view! {
        <div data-testid="progres-wilayah" class="overflow-x-auto">
            <table class="w-full text-left">
                <thead>
                    <tr class="text-xs text-slate-500">
                        <th class="pb-2 pr-3 font-medium">"Wilayah"</th>
                        <th class="pb-2 pr-3 font-medium">"Satker"</th>
                        <th class="hidden pb-2 pr-3 font-medium sm:table-cell">
                            "Belum / Wilayah / Pusat / Selesai"
                        </th>
                        <th class="pb-2 font-medium">"Pegawai sudah mengisi"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let batas = if semua.get() { usize::MAX } else { WILAYAH_AWAL };
                        wilayah
                            .get_value()
                            .into_iter()
                            .take(batas)
                            .map(baris_wilayah)
                            .collect_view()
                    }}
                </tbody>
            </table>
            {(jumlah > WILAYAH_AWAL)
                .then(|| {
                    view! {
                        <button
                            type="button"
                            data-testid="progres-wilayah-semua"
                            class="mt-2 text-xs text-gold-300 hover:underline"
                            on:click=move |_| semua.update(|v| *v = !*v)
                        >
                            {move || {
                                if semua.get() {
                                    "Tampilkan lebih sedikit".to_string()
                                } else {
                                    format!("Tampilkan semua ({jumlah} wilayah)")
                                }
                            }}
                        </button>
                    }
                })}
        </div>
    }
}

fn baris_tertinggal(pengajuan_id: String, s: ProgresSatker) -> impl IntoView {
    let nama = s.satker_nama.clone().unwrap_or_else(|| s.satker_id.clone());
    let tahap = aktivitas_label(s.aktivitas_id);
    let tautan = crate::routes::url::pakaian_pengisian(&pengajuan_id, &s.satker_id);
    let belum = s.pegawai_total - s.pegawai_terisi;
    view! {
        <li class="flex flex-wrap items-center justify-between gap-2 py-2">
            <div class="min-w-0">
                <p class="truncate text-sm text-slate-200">{nama}</p>
                <p class="text-xs text-slate-500">
                    {s.wilayah_nama.clone().unwrap_or_default()}
                    {s.wilayah_nama.as_ref().map(|_| " · ")}
                    {tahap}
                </p>
            </div>
            <div class="flex items-center gap-3">
                <span class="text-xs text-slate-300">
                    {format!(
                        "{} dari {} pegawai belum mengisi ({})",
                        belum,
                        s.pegawai_total,
                        teks_terisi(s.pegawai_terisi, s.pegawai_total),
                    )}
                </span>
                <a
                    data-testid="progres-lihat-pegawai"
                    href=tautan
                    class="rounded-lg border border-white/10 px-2.5 py-1 text-xs text-slate-200 hover:bg-white/[0.04]"
                >
                    "Lihat pegawai"
                </a>
            </div>
        </li>
    }
}

fn isi_panel(pengajuan_id: String, p: ProgresPakaianDinas) -> AnyView {
    if p.ringkasan.satker_total == 0 {
        return ().into_any();
    }
    let ProgresPakaianDinas {
        ringkasan,
        per_wilayah,
        tertinggal,
    } = p;
    view! {
        <SectionCard title="Progres Kegiatan">
            <div data-testid="progres-kegiatan" class="space-y-5">
                {ringkasan_kegiatan(ringkasan)}
                {(!per_wilayah.is_empty()).then(|| tabel_wilayah(per_wilayah))}
                {(!tertinggal.is_empty())
                    .then(|| {
                        view! {
                            <div data-testid="progres-tertinggal">
                                <h3 class="mb-1 text-sm font-semibold text-slate-200">
                                    "Satker paling tertinggal"
                                </h3>
                                <p class="mb-1 text-xs text-slate-500">
                                    "Belum selesai, diurutkan dari yang paling sedikit mengisi."
                                </p>
                                <ul class="divide-y divide-white/[0.04]">
                                    {tertinggal
                                        .into_iter()
                                        .map(|s| baris_tertinggal(pengajuan_id.clone(), s))
                                        .collect_view()}
                                </ul>
                            </div>
                        }
                    })}
            </div>
        </SectionCard>
    }
    .into_any()
}

/// The progress of one campaign in a few lines: the stage bar, how many satker are finished,
/// and how many employees have given a size. For the campaign cards and the dashboard, where
/// the full panel would be too much; `None` when the caller has no satker in the campaign.
pub fn progres_ringkas(r: &ProgresRingkasan) -> AnyView {
    if r.satker_total == 0 {
        return ().into_any();
    }
    let satker = format!("{} dari {} satker selesai", r.selesai, r.satker_total);
    let pegawai = match persen(r.pegawai_terisi, r.pegawai_total) {
        Some(p) => format!(
            "Pegawai sudah mengisi: {} / {} ({p}%)",
            r.pegawai_terisi, r.pegawai_total
        ),
        None => "Belum ada pegawai di roster".to_string(),
    };
    view! {
        <div data-testid="progres-ringkas" class="mt-3 space-y-1.5">
            {bar_tahap(r.clone())}
            <p class="text-xs text-slate-300">{satker}</p>
            <p class="text-xs text-slate-400">{pegawai}</p>
        </div>
    }
    .into_any()
}

/// The progress panel of one campaign. `refresh` is the page's reload counter, so a
/// move made on the page is reflected here too.
#[component]
pub fn PakaianDinasProgres(
    #[prop(into)] pengajuan_id: Signal<String>,
    #[prop(into)] refresh: Signal<i32>,
) -> impl IntoView {
    let client: QueryClient = expect_context();
    let progres = client.local_resource(query_progres, move || (pengajuan_id.get(), refresh.get()));
    view! {
        <div class="mb-4">
            <Suspense fallback=move || view! { <LoadingState /> }>
                {move || match progres.get() {
                    None => view! { <LoadingState /> }.into_any(),
                    // Monitoring is an addition to the page, not the page: a failure is said
                    // once, in place, and the satker list below keeps working.
                    Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                    Some(Ok(p)) => isi_panel(pengajuan_id.get_untracked(), p),
                }}
            </Suspense>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::{lebar, teks_terisi};

    #[test]
    fn the_figure_says_the_whole_and_the_share() {
        assert_eq!(teks_terisi(12, 15), "12 / 15 (80%)");
        assert_eq!(teks_terisi(0, 0), "-");
    }

    #[test]
    fn a_segment_of_an_empty_whole_has_no_width() {
        assert_eq!(lebar(0, 0), "0%");
        assert_eq!(lebar(5, 10), "50%");
    }
}
