//! Bank Aset sebaran: peta bertingkat (Indonesia, wilayah, satker), lalu tabel wilayah dan tabel satker
//! yang bisa dicari dan dihalamani.
//!
//! Halaman ini dulu hanya tabel 556 baris sepanjang 36.000 piksel, padahal judulnya "sebaran".
//! Peta menjawab "di mana", tabel menjawab "berapa". Tingkat yang dipilih di peta juga menyaring
//! tabel satker ke wilayah itu, jadi keduanya selalu membicarakan hal yang sama.

use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::LIST;

use super::dashboard_page::{format_rupiah, format_thousands};
use crate::api::bank_aset::{self, BankAsetSebaran, SebaranSatker};
use crate::api::error::AppError;
use crate::components::layout::{
    EmptyState, ErrorState, LoadingState, PageBreadcrumb, PageLayout, SectionCard,
};
use crate::components::pagination_controls::PaginationControls;
use crate::components::peta_sebaran::{
    PetaSebaran, Tingkat, Wilayah, susun_wilayah, tautan_daftar, wilayah_terbuka,
};
use crate::routes::path;

const PER_HALAMAN: usize = 25;

#[component]
pub fn BankAsetSebaranPage() -> impl IntoView {
    let (data, set_data) = signal::<Option<BankAsetSebaran>>(None);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<AppError>>(None);
    let (reload_tick, set_reload_tick) = signal(0_u32);

    Effect::new(move |_| {
        let _ = reload_tick.get();
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match bank_aset::fetch_sebaran().await {
                Ok(d) => set_data.set(Some(d)),
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    let breadcrumbs = vec![
        PageBreadcrumb::new("Dashboard", path::DASHBOARD),
        PageBreadcrumb::new("Bank Aset", path::BANK_ASET_DASHBOARD),
        PageBreadcrumb::leaf("Sebaran"),
    ];

    view! {
        <PageLayout
            title="Sebaran Aset per Satker"
            description="Di mana aset BMN berada: seluruh Indonesia, tiap wilayah, hingga tiap satuan kerja."
            icon="fas fa-map-location-dot"
            breadcrumbs=breadcrumbs
            actions=Box::new(move || {
                view! {
                    <A
                        href=path::BANK_ASET_DAFTAR
                        attr:class="focus-ring inline-flex items-center gap-2 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs font-medium text-slate-100 transition hover:bg-white/[0.08]"
                    >
                        <span class="text-[0.7rem]">
                            <AppIcon icon=LIST />
                        </span>
                        "Daftar Aset"
                    </A>
                }
                    .into_any()
            })
        >
            {move || {
                if loading.get() && data.get().is_none() {
                    view! { <LoadingState message="Memuat data sebaran..." /> }.into_any()
                } else if let Some(err) = error.get() {
                    view! {
                        <ErrorState
                            error=err
                            on_retry=Box::new(move || {
                                set_reload_tick.update(|t| *t += 1);
                            })
                        />
                    }
                        .into_any()
                } else if let Some(d) = data.get() {
                    if d.satker.is_empty() {
                        view! {
                            <EmptyState
                                title="Belum ada data sebaran"
                                description="Sinkronisasi SIMAN belum menghasilkan data aset per satker."
                                icon="fas fa-building-circle-exclamation"
                            />
                        }
                            .into_any()
                    } else {
                        isi_halaman(d.satker).into_any()
                    }
                } else {
                    view! { <LoadingState message="Memuat data sebaran..." /> }.into_any()
                }
            }}
        </PageLayout>
    }
}

fn isi_halaman(satker: Vec<SebaranSatker>) -> impl IntoView {
    let wilayah = Arc::new(susun_wilayah(&satker));
    let satker = Arc::new(satker);
    // The level the map is at; both tables follow it.
    let tingkat = RwSignal::new(Tingkat::Nasional);
    // One clone per consumer: the `view!` props below each take ownership.
    let (satker_peta, wilayah_peta) = (Arc::clone(&satker), Arc::clone(&wilayah));
    let wilayah_tabel = Arc::clone(&wilayah);
    view! {
        <div class="space-y-6">
            <SectionCard
                title="Peta Sebaran"
                description="Pilih sebuah titik wilayah untuk memperbesar, lalu pilih satuan kerja untuk melihat rinciannya."
                icon="fas fa-map-location-dot"
            >
                <PetaSebaran satker=satker_peta wilayah=wilayah_peta tingkat=tingkat />
            </SectionCard>
            <SectionCard
                title="Ringkasan per Wilayah"
                description="Satu wilayah per Kejaksaan Tinggi, diurutkan dari jumlah aset terbanyak."
                icon="fas fa-map"
            >
                <TabelWilayah wilayah=wilayah_tabel tingkat=tingkat />
            </SectionCard>
            <SectionCard
                title="Ringkasan per Satker"
                description="Satker diurutkan dari jumlah aset terbanyak. Memilih wilayah di peta menyaring tabel ini."
                icon="fas fa-building"
            >
                <TabelSatker satker=satker wilayah=wilayah tingkat=tingkat />
            </SectionCard>
        </div>
    }
}

/// Apakah `s` cocok dengan kata yang dicari: nama, kode, atau provinsi, tanpa memedulikan huruf besar.
pub fn cocok(s: &SebaranSatker, kata: &str) -> bool {
    let kata = kata.trim().to_lowercase();
    if kata.is_empty() {
        return true;
    }
    [
        Some(s.nama_satker.as_str()),
        s.kode_satker.as_deref(),
        s.provinsi.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(|f| f.to_lowercase().contains(&kata))
}

/// Jumlah halaman untuk `n` baris; selalu minimal satu supaya "halaman 1 dari 1" tetap tampil.
pub fn jumlah_halaman(n: usize) -> usize {
    n.div_ceil(PER_HALAMAN).max(1)
}

#[component]
fn TabelWilayah(wilayah: Arc<Vec<Wilayah>>, tingkat: RwSignal<Tingkat>) -> impl IntoView {
    let total_aset: i64 = wilayah.iter().map(|w| w.ringkasan.aset).sum();
    view! {
        <div class="overflow-x-auto">
            <table class="w-full">
                <thead class="border-b border-white/[0.08]">
                    <tr class="text-[0.65rem] uppercase tracking-wide text-slate-500">
                        <th class="py-2 pr-3 text-left font-medium">"#"</th>
                        <th class="py-2 pr-3 text-left font-medium">"Wilayah"</th>
                        <th class="py-2 pr-3 text-right font-medium">"Satker"</th>
                        <th class="py-2 pr-3 text-right font-medium">"Total Aset"</th>
                        <th class="py-2 pr-3 text-right font-medium">"Rusak"</th>
                        <th class="py-2 pr-3 text-right font-medium">"Nilai"</th>
                        <th class="py-2 text-right font-medium">"% dari Total"</th>
                    </tr>
                </thead>
                <tbody data-testid="baris-wilayah">
                    {wilayah
                        .iter()
                        .enumerate()
                        .map(|(i, w)| {
                            let pct = if total_aset > 0 {
                                w.ringkasan.aset as f64 / total_aset as f64 * 100.0
                            } else {
                                0.0
                            };
                            let nama = w.nama.clone();
                            let r = w.ringkasan.clone();
                            view! {
                                <tr class="border-b border-white/[0.04] last:border-0">
                                    <td class="py-3 pr-3 text-xs font-semibold text-gold-300">
                                        {(i + 1).to_string()}
                                    </td>
                                    <td class="py-3 pr-3">
                                        <button
                                            type="button"
                                            class="text-left text-sm font-semibold text-slate-100 hover:text-gold-300"
                                            on:click=move |_| {
                                                tingkat.set(Tingkat::Wilayah(i));
                                                scroll_ke_peta();
                                            }
                                        >
                                            {nama}
                                        </button>
                                    </td>
                                    <td class="py-3 pr-3 text-right text-sm text-slate-300">
                                        {format_thousands(r.satker as i64)}
                                    </td>
                                    <td class="py-3 pr-3 text-right text-sm text-slate-200">
                                        {format_thousands(r.aset)}
                                    </td>
                                    <td class="py-3 pr-3 text-right text-xs text-danger-300">
                                        {format_thousands(r.rusak)}
                                    </td>
                                    <td class="py-3 pr-3 text-right text-sm text-slate-300">
                                        {format_rupiah(r.nilai)}
                                    </td>
                                    <td class="py-3 text-right text-xs text-slate-400">
                                        {format!("{pct:.1}%")}
                                    </td>
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
        </div>
    }
}

/// Setelah memilih dari tabel, bawa peta ke layar: tabel ada di bawahnya.
fn scroll_ke_peta() {
    if let Some(el) = document().get_element_by_id("peta-bmn") {
        el.scroll_into_view();
    }
}

#[component]
fn TabelSatker(
    satker: Arc<Vec<SebaranSatker>>,
    wilayah: Arc<Vec<Wilayah>>,
    tingkat: RwSignal<Tingkat>,
) -> impl IntoView {
    let total_aset: i64 = satker.iter().map(|s| s.total_aset).sum();
    let kata = RwSignal::new(String::new());
    let halaman = RwSignal::new(1_i32);

    // Baris yang cocok: peringkat nasional (tetap walau disaring) dan indeks satkernya.
    let cocokan = {
        let (satker, wilayah) = (Arc::clone(&satker), Arc::clone(&wilayah));
        Memo::new(move |_| {
            let k = kata.get();
            let terbuka = wilayah_terbuka(tingkat.get(), &wilayah);
            satker
                .iter()
                .enumerate()
                .filter(|(i, _)| terbuka.is_none_or(|w| wilayah[w].anggota.contains(i)))
                .filter(|(_, s)| cocok(s, &k))
                .map(|(i, _)| i)
                .collect::<Vec<usize>>()
        })
    };
    let total_halaman = Memo::new(move |_| jumlah_halaman(cocokan.get().len()) as i32);

    // A new wilayah or a new search starts from the first page.
    Effect::new(move |_| {
        let _ = tingkat.get();
        halaman.set(1);
    });

    let on_cari = move |ev: leptos::ev::Event| {
        kata.set(event_target_value(&ev));
        halaman.set(1);
    };

    let judul_wilayah = {
        let wilayah = Arc::clone(&wilayah);
        move || {
            wilayah_terbuka(tingkat.get(), &wilayah).map(|w| {
                let nama = wilayah[w].nama.clone();
                view! {
                    <span class="text-xs text-slate-300">
                        "Hanya wilayah " <span class="font-semibold text-gold-300">{nama}</span>
                    </span>
                    <button
                        type="button"
                        class="text-xs text-slate-400 underline hover:text-slate-100"
                        on:click=move |_| tingkat.set(Tingkat::Nasional)
                    >
                        "Tampilkan semua"
                    </button>
                }
            })
        }
    };

    let jumlah_satker = satker.len();
    let isi = {
        let satker = Arc::clone(&satker);
        move || {
            let dari = (halaman.get().max(1) as usize - 1) * PER_HALAMAN;
            cocokan
                .get()
                .into_iter()
                .skip(dari)
                .take(PER_HALAMAN)
                .map(|i| baris(i + 1, i, satker[i].clone(), total_aset, tingkat))
                .collect_view()
        }
    };

    view! {
        <div class="space-y-3">
            <div class="flex flex-wrap items-center gap-3">
                <input
                    type="search"
                    data-testid="cari-sebaran"
                    class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100 placeholder:text-slate-500 md:max-w-md"
                    placeholder="Cari nama satker, kode, atau provinsi"
                    aria-label="Cari satker"
                    prop:value=move || kata.get()
                    on:input=on_cari
                />
                {judul_wilayah}
            </div>
            <p class="text-xs text-slate-400" data-testid="hitungan-sebaran">
                {move || {
                    format!(
                        "{} dari {} satker",
                        format_thousands(cocokan.get().len() as i64),
                        format_thousands(jumlah_satker as i64),
                    )
                }}
            </p>
            <div class="overflow-x-auto">
                <table class="w-full">
                    <thead class="border-b border-white/[0.08]">
                        <tr class="text-[0.65rem] uppercase tracking-wide text-slate-500">
                            <th class="py-2 pr-3 text-left font-medium">"#"</th>
                            <th class="py-2 pr-3 text-left font-medium">"Satker"</th>
                            <th class="py-2 pr-3 text-right font-medium">"Total Aset"</th>
                            <th class="py-2 pr-3 text-right font-medium">"Baik"</th>
                            <th class="py-2 pr-3 text-right font-medium">"Rusak"</th>
                            <th class="py-2 pr-3 text-right font-medium">"Nilai"</th>
                            <th class="py-2 text-right font-medium">"% dari Total"</th>
                        </tr>
                    </thead>
                    <tbody data-testid="baris-sebaran">{isi}</tbody>
                </table>
            </div>
            <PaginationControls
                current_page=Signal::derive(move || halaman.get())
                total_pages=Signal::derive(move || total_halaman.get())
                total_items=None
                on_prev=Callback::new(move |_| halaman.update(|h| *h = (*h - 1).max(1)))
                on_next=Callback::new(move |_| {
                    let maks = total_halaman.get_untracked();
                    halaman.update(|h| *h = (*h + 1).min(maks))
                })
            />
        </div>
    }
}

fn baris(
    peringkat: usize,
    indeks: usize,
    s: SebaranSatker,
    total_aset: i64,
    tingkat: RwSignal<Tingkat>,
) -> impl IntoView {
    let kode = s.kode_satker.clone().unwrap_or_else(|| "-".to_string());
    let sub = match &s.provinsi {
        Some(p) => format!("{kode} · {p}"),
        None => kode,
    };
    let belum_di_peta = s.lintang.is_none() || s.bujur.is_none();
    let pct = if total_aset > 0 {
        (s.total_aset as f64) / (total_aset as f64) * 100.0
    } else {
        0.0
    };
    let href = tautan_daftar(&s.nama_satker);
    view! {
        <tr class="border-b border-white/[0.04] last:border-0">
            <td class="py-3 pr-3 text-xs font-semibold text-gold-300">{peringkat.to_string()}</td>
            <td class="py-3 pr-3">
                <A href=href attr:class="text-sm font-semibold text-slate-100 hover:text-gold-300">
                    {s.nama_satker.clone()}
                </A>
                <p class="mt-0.5 text-[0.7rem] text-slate-500">
                    {sub}
                    {if belum_di_peta {
                        view! { <span class="ml-2 text-slate-600">"· belum di peta"</span> }
                            .into_any()
                    } else {
                        view! {
                            <button
                                type="button"
                                class="ml-2 text-slate-400 underline hover:text-slate-100"
                                on:click=move |_| {
                                    tingkat.set(Tingkat::Satker(indeks));
                                    scroll_ke_peta();
                                }
                            >
                                "lihat di peta"
                            </button>
                        }
                            .into_any()
                    }}
                </p>
            </td>
            <td class="py-3 pr-3 text-right text-sm text-slate-200">
                {format_thousands(s.total_aset)}
            </td>
            <td class="py-3 pr-3 text-right text-xs text-success-300">
                {format_thousands(s.aset_baik)}
            </td>
            <td class="py-3 pr-3 text-right text-xs text-danger-300">
                {format_thousands(s.aset_rusak)}
            </td>
            <td class="py-3 pr-3 text-right text-sm text-slate-300">
                {format_rupiah(s.nilai_perolehan)}
            </td>
            <td class="py-3 text-right text-xs text-slate-400">{format!("{pct:.1}%")}</td>
        </tr>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(nama: &str, kode: Option<&str>, prov: Option<&str>) -> SebaranSatker {
        SebaranSatker {
            kode_satker: kode.map(str::to_string),
            kode_mysimkari: None,
            nama_satker: nama.to_string(),
            total_aset: 1,
            nilai_perolehan: 0.0,
            aset_baik: 1,
            aset_rusak: 0,
            lintang: None,
            bujur: None,
            provinsi: prov.map(str::to_string),
            wilayah_kode: None,
            wilayah_nama: None,
            aset_bergps: 0,
        }
    }

    #[test]
    fn search_matches_name_code_or_province_in_any_case() {
        let a = s("KEJAKSAAN NEGERI BANDA ACEH", Some("01.01"), Some("Aceh"));
        assert!(cocok(&a, "banda"));
        assert!(cocok(&a, "01.01"));
        assert!(cocok(&a, "ACEH"));
        assert!(cocok(&a, "  banda  "));
        assert!(cocok(&a, ""));
        assert!(!cocok(&a, "medan"));
    }

    #[test]
    fn a_satker_without_a_code_or_province_is_still_found_by_name() {
        let a = s("RS ADHYAKSA", None, None);
        assert!(cocok(&a, "adhyaksa"));
        assert!(!cocok(&a, "jakarta"));
    }

    #[test]
    fn there_is_always_at_least_one_page() {
        assert_eq!(jumlah_halaman(0), 1);
        assert_eq!(jumlah_halaman(1), 1);
        assert_eq!(jumlah_halaman(PER_HALAMAN), 1);
        assert_eq!(jumlah_halaman(PER_HALAMAN + 1), 2);
        // 556 satkers on staging: 23 pages, not one 36,000 px table.
        assert_eq!(jumlah_halaman(556), 23);
    }
}
