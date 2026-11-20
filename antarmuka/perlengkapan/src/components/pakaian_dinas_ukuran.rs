//! Ukuran Pegawai.
//!
//! Pegawai mengisi ukuran pakaian dinasnya sendiri. Kolom yang ditampilkan BUKAN tiga pilihan
//! tetap (baju, celana, sepatu) melainkan satu isian per spesifikasi pakaian aktif, dikelompokkan
//! per jenis (PDH, PDL, …): sepatu PDH dan sepatu PDL, kaos dan baju, topi, hingga nama dada adalah
//! isian terpisah. Grup ukurannya dari referensi: grup `PILIHAN` tampil sebagai pilihan dari
//! daftarnya, grup `TEKS` sebagai kotak teks. Jenis atau atribut baru yang ditambahkan admin
//! sebagai data muncul di sini tanpa perubahan kode.
//!
//! Semua isian opsional: yang dikosongkan menghapus isiannya. Hanya yang berubah yang dikirim.

use std::collections::HashMap;

use crate::api::ukuran_referensi::{
    SimpanUkuranProfilRequest, UkuranProfilInput, UkuranProfilItem, fetch_ukuran_profil, ikon_grup,
    kelompokkan_ukuran, simpan_ukuran_profil,
};
use crate::api::{AppError, PegawaiPakaianDinas, fetch_master_ukuran, fetch_pegawai_ukuran};
use crate::components::layout::{EmptyState, ErrorState, LoadingState, PageLayout, SectionCard};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_fetch::QueryClient;
use lib_ui::components::icon::{AppIcon, icon_from_fa_class};
use phosphor_leptos::{CHECK_CIRCLE, FLOPPY_DISK, INFO, WARNING_CIRCLE};

/// Panjang terpanjang isian teks (`validasi_ukuran::MAKS_TEKS` di backend).
const MAKS_TEKS: usize = 100;

const FIELD: &str = "focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] \
                     px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500";

/// Daftar ukuran semua grup, sekali: `()` menjadi satu slot cache karena sama untuk setiap pemakai.
async fn query_master_ukuran(_: ()) -> Result<HashMap<String, Vec<String>>, AppError> {
    let semua = fetch_master_ukuran(None).await?.data;
    Ok(kelompokkan_ukuran(&semua))
}

/// Kunci cache = id pegawai (bukan `()`): dua pegawai di satu komputer tak boleh berbagi isian.
async fn query_profil(_pegawai_id: String) -> Result<Vec<UkuranProfilItem>, AppError> {
    fetch_ukuran_profil().await
}

/// Hanya untuk bendera berhijab; ukurannya dibaca dari `query_profil`.
async fn query_pegawai(pegawai_id: String) -> Result<Option<PegawaiPakaianDinas>, AppError> {
    fetch_pegawai_ukuran(pegawai_id).await.map(|r| r.data)
}

fn peta_ukuran(items: &[UkuranProfilItem]) -> HashMap<String, String> {
    items
        .iter()
        .map(|i| {
            (
                i.spesifikasi_id.clone(),
                i.ukuran.clone().unwrap_or_default(),
            )
        })
        .collect()
}

/// Isian dikelompokkan per jenis menurut urutan kemunculannya (backend sudah mengurutkannya).
fn per_jenis(items: &[UkuranProfilItem]) -> Vec<(String, Vec<UkuranProfilItem>)> {
    let mut hasil: Vec<(String, Vec<UkuranProfilItem>)> = Vec::new();
    for item in items {
        match hasil
            .iter_mut()
            .find(|(j, _)| *j == item.jenis_pakaian_nama)
        {
            Some((_, daftar)) => daftar.push(item.clone()),
            None => hasil.push((item.jenis_pakaian_nama.clone(), vec![item.clone()])),
        }
    }
    hasil
}

fn catatan_gender(gender: &str) -> Option<&'static str> {
    match gender {
        "L" => Some("Khusus laki-laki"),
        "P" => Some("Khusus perempuan"),
        _ => None,
    }
}

#[component]
pub fn UkuranPegawai(
    /// Pegawai ID from JWT claims or context
    pegawai_id: String,
    /// Pegawai name for display
    pegawai_nama: String,
    /// Pegawai NIP for display
    pegawai_nip: String,
) -> impl IntoView {
    let nilai = RwSignal::new(HashMap::<String, String>::new());
    // Yang tersimpan saat ini: pembanding untuk mengirim hanya yang berubah.
    let dasar = StoredValue::new(HashMap::<String, String>::new());
    let hijab = RwSignal::new(false);
    let hijab_dasar = StoredValue::new(false);
    let (is_saving, set_is_saving) = signal(false);
    let (success_message, set_success_message) = signal(Option::<String>::None);
    let (error_message, set_error_message) = signal(Option::<String>::None);

    let client: QueryClient = expect_context();
    let master = client.local_resource(query_master_ukuran, || ());
    let id_profil = pegawai_id.clone();
    let profil = client.local_resource(query_profil, move || id_profil.clone());
    let id_hijab = pegawai_id.clone();
    let pegawai = client.local_resource(query_pegawai, move || id_hijab.clone());

    // Isi formulir dari yang tersimpan, SEKALI. Formulir baru digambar setelah ketiga jawaban
    // masuk (lihat `match` di bawah), jadi saat orang bisa memilih, isian lama sudah di tempatnya;
    // bendera ini menjaga pemuatan ulang agar tak menimpa pilihan yang sudah dibuat.
    let hydrated = StoredValue::new(false);
    Effect::new(move || {
        if hydrated.get_value() {
            return;
        }
        if let (Some(Ok(items)), Some(Ok(data))) = (profil.get(), pegawai.get()) {
            let peta = peta_ukuran(&items);
            dasar.set_value(peta.clone());
            nilai.set(peta);
            let h = data.map(|d| d.with_hijab).unwrap_or(false);
            hijab_dasar.set_value(h);
            hijab.set(h);
            hydrated.set_value(true);
        }
    });

    let initial = pegawai_nama.chars().next().unwrap_or('?').to_string();

    view! {
        <PageLayout
            title="Ukuran Pakaian Dinas"
            icon="fas fa-tshirt"
            description="Isi ukuran pakaian dinas Anda untuk keperluan pengajuan"
        >
            <SectionCard title="Data Pegawai" dense=true>
                <div class="flex items-center gap-4">
                    <div class="flex h-11 w-11 items-center justify-center rounded-full bg-gold-500/15 text-sm font-bold text-gold-400 ring-1 ring-gold-500/25">
                        {initial}
                    </div>
                    <div>
                        <p class="text-sm font-semibold text-slate-100">{pegawai_nama}</p>
                        <p class="text-xs text-slate-400">"NIP: " {pegawai_nip}</p>
                    </div>
                </div>
            </SectionCard>

            <Show when=move || success_message.get().is_some()>
                <div
                    data-testid="ukuran-profil-sukses"
                    class="mt-4 flex items-center gap-2 rounded-xl border border-success-500/30 bg-success-500/[0.08] px-4 py-3 text-sm text-success-300"
                >
                    <AppIcon icon=CHECK_CIRCLE />
                    {move || success_message.get()}
                </div>
            </Show>
            <Show when=move || error_message.get().is_some()>
                <div
                    data-testid="ukuran-profil-galat"
                    class="mt-4 flex items-center gap-2 rounded-xl border border-danger-500/30 bg-danger-500/[0.08] px-4 py-3 text-sm text-danger-300"
                >
                    <AppIcon icon=WARNING_CIRCLE />
                    {move || error_message.get()}
                </div>
            </Show>

            <div class="mt-4">
                <Suspense fallback=move || {
                    view! { <LoadingState message="Memuat data ukuran...".to_string() /> }
                }>
                    {move || {
                        // Butuh TIGA jawaban: daftar ukuran, kolom beserta isian tersimpan, dan
                        // bendera hijab. Menggambar formulir dari sebagian saja membiarkan jawaban
                        // yang lambat menimpa pilihan yang sudah dibuat, lalu "Simpan" mengirim
                        // yang lama dengan pesan sukses. Gagal memuat ditampilkan sebagai galat,
                        // bukan formulir kosong yang menimpa isian yang ada.
                        match (master.get(), profil.get(), pegawai.get()) {
                            (Some(Err(e)), _, _) | (_, Some(Err(e)), _) | (_, _, Some(Err(e))) => {
                                view! { <ErrorState error=e /> }.into_any()
                            }
                            (None, _, _) | (_, None, _) | (_, _, None) => {
                                view! { <LoadingState /> }.into_any()
                            }
                            (Some(Ok(daftar)), Some(Ok(items)), Some(Ok(_))) => {
                                if items.is_empty() {
                                    return view! {
                                        <EmptyState
                                            icon="fas fa-tshirt"
                                            title="Belum Ada Pakaian Dinas"
                                            description="Belum ada jenis pakaian dinas aktif yang memerlukan ukuran. Hubungi pengelola pusat."
                                        />
                                    }
                                        .into_any();
                                }
                                let jenis = per_jenis(&items);
                                let on_submit = move |ev: leptos::ev::SubmitEvent| {
                                    ev.prevent_default();
                                    set_success_message.set(None);
                                    set_error_message.set(None);
                                    let sekarang = nilai.get();
                                    let lama = dasar.get_value();
                                    // Hanya yang berubah: yang tak disentuh tak ikut dikirim,
                                    // jadi tak ada yang tertimpa tanpa disengaja.
                                    let mut berubah: Vec<UkuranProfilInput> = sekarang
                                        .iter()
                                        .filter(|(id, v)| {
                                            v.trim() != lama.get(*id).map(String::as_str).unwrap_or("")
                                        })
                                        .map(|(id, v)| UkuranProfilInput {
                                            spesifikasi_id: id.clone(),
                                            ukuran: v.trim().to_string(),
                                        })
                                        .collect();
                                    berubah.sort_by(|a, b| a.spesifikasi_id.cmp(&b.spesifikasi_id));
                                    let h = hijab.get();
                                    if berubah.is_empty() && h == hijab_dasar.get_value() {
                                        set_success_message
                                            .set(Some("Tidak ada perubahan untuk disimpan.".to_string()));
                                        return;
                                    }
                                    set_is_saving.set(true);
                                    spawn_local(async move {
                                        let request = SimpanUkuranProfilRequest {
                                            ukuran: berubah,
                                            with_hijab: Some(h),
                                        };
                                        match simpan_ukuran_profil(&request).await {
                                            Ok(terbaru) => {
                                                // Keadaan terbaru dari server (ejaan daftar,
                                                // mis. "xl" menjadi "XL") menjadi pembanding
                                                // berikutnya dan menggantikan isi kotak.
                                                let peta = peta_ukuran(&terbaru);
                                                dasar.set_value(peta.clone());
                                                nilai.set(peta);
                                                hijab_dasar.set_value(h);
                                                set_success_message
                                                    .set(Some("Ukuran berhasil disimpan!".to_string()));
                                            }
                                            Err(e) => set_error_message.set(Some(e.user_message())),
                                        }
                                        set_is_saving.set(false);
                                    });
                                };
                                view! {
                                    <form data-testid="ukuran-profil-form" on:submit=on_submit>
                                        <div class="flex flex-col gap-4">
                                            {jenis
                                                .into_iter()
                                                .map(|(nama_jenis, kolom)| {
                                                    let daftar = daftar.clone();
                                                    view! {
                                                        <SectionCard title=nama_jenis.clone()>
                                                            <div
                                                                data-testid="ukuran-profil-jenis"
                                                                data-jenis=nama_jenis
                                                                class="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3"
                                                            >
                                                                {kolom
                                                                    .into_iter()
                                                                    .map(|item| {
                                                                        let opsi = daftar
                                                                            .get(&item.ukuran_group.to_uppercase())
                                                                            .cloned()
                                                                            .unwrap_or_default();
                                                                        kolom_profil(item, opsi, nilai)
                                                                    })
                                                                    .collect_view()}
                                                            </div>
                                                        </SectionCard>
                                                    }
                                                })
                                                .collect_view()}
                                            // `with_hijab` mengalir ke laporan pakaian (Y/T). Tanpa
                                            // kendali di sini, formulir mengirim `false` bawaan di
                                            // tiap simpan dan diam-diam menghapus yang sudah diset.
                                            <SectionCard title="Lainnya" dense=true>
                                                <label class="flex items-center gap-2.5 text-sm text-slate-200">
                                                    <input
                                                        type="checkbox"
                                                        data-testid="ukuran-profil-hijab"
                                                        class="focus-ring h-4 w-4 rounded border-white/20 bg-white/[0.04]"
                                                        prop:checked=move || hijab.get()
                                                        on:change=move |ev| {
                                                            hijab.set(event_target_checked(&ev))
                                                        }
                                                    />
                                                    "Menggunakan hijab"
                                                </label>
                                            </SectionCard>
                                            <div class="flex justify-end">
                                                <button
                                                    type="submit"
                                                    data-testid="simpan-ukuran-profil"
                                                    class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-5 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90 disabled:opacity-50"
                                                    prop:disabled=move || is_saving.get()
                                                >
                                                    <span class="text-xs">
                                                        <AppIcon icon=FLOPPY_DISK />
                                                    </span>
                                                    {move || {
                                                        if is_saving.get() {
                                                            "Menyimpan..."
                                                        } else {
                                                            "Simpan Ukuran"
                                                        }
                                                    }}
                                                </button>
                                            </div>
                                        </div>
                                    </form>
                                }
                                    .into_any()
                            }
                        }
                    }}
                </Suspense>
            </div>

            <SectionCard title="Panduan Pengukuran">
                <ul class="flex flex-col gap-2 text-sm text-slate-300">
                    {[
                        ("Baju / Kaos:", " Ukur lingkar dada pada bagian terlebar, pilih ukuran yang sesuai."),
                        ("Celana:", " Ukur lingkar pinggang pada posisi normal."),
                        ("Sepatu:", " Ukur panjang kaki dari tumit ke ujung jari terpanjang."),
                        ("Topi:", " Ukur lingkar kepala tepat di atas alis dan telinga (cm)."),
                        ("Nama dada:", " Tulis seperti yang akan dicetak pada pakaian."),
                    ]
                        .into_iter()
                        .map(|(judul, isi)| {
                            view! {
                                <li class="flex items-start gap-2">
                                    <span class="mt-0.5 text-xs text-gold-400 shrink-0">
                                        <AppIcon icon=INFO />
                                    </span>
                                    <span>
                                        <strong class="text-slate-100">{judul}</strong>
                                        {isi}
                                    </span>
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </SectionCard>
        </PageLayout>
    }
}

/// Satu kolom isian: pilihan dari daftar grupnya, atau kotak teks bagi grup `TEKS`.
fn kolom_profil(
    item: UkuranProfilItem,
    opsi: Vec<String>,
    nilai: RwSignal<HashMap<String, String>>,
) -> impl IntoView {
    let id = item.spesifikasi_id.clone();
    let dom_id = format!("profil-{id}");
    let baca = {
        let id = id.clone();
        move || nilai.with(|m| m.get(&id).cloned().unwrap_or_default())
    };
    let tulis = {
        let id = id.clone();
        move |v: String| {
            nilai.update(|m| {
                m.insert(id.clone(), v);
            })
        }
    };

    // Kelas warna mengikuti ikon: `icon_from_fa_class` hanya memakai nama ikonnya, jadi warna
    // dipasang ulang pada pembungkus agar tidak rata menjadi `currentColor`.
    let ikon = ikon_grup(&item.ukuran_group);
    let warna: String = ikon
        .split_whitespace()
        .filter(|tok| !matches!(*tok, "fas" | "far" | "fab") && !tok.starts_with("fa-"))
        .collect::<Vec<_>>()
        .join(" ");
    let pembungkus = format!("inline-flex {warna}");
    let teks = item.is_teks();
    let catatan = catatan_gender(&item.gender);
    let petunjuk = if teks {
        format!("Isian bebas, paling banyak {MAKS_TEKS} karakter")
    } else {
        "Pilih dari daftar".to_string()
    };
    let judul = item.spesifikasi_nama.clone();

    let kontrol = if teks {
        let baca = baca.clone();
        view! {
            <input
                id=dom_id.clone()
                type="text"
                class=FIELD
                maxlength=MAKS_TEKS.to_string()
                placeholder="Kosongkan bila tidak diperlukan"
                prop:value=baca
                on:input=move |ev| tulis(event_target_value(&ev))
            />
        }
        .into_any()
    } else {
        view! {
            <select
                id=dom_id.clone()
                class=FIELD
                prop:value=baca
                on:change=move |ev| tulis(event_target_value(&ev))
            >
                <option value="">"-- Pilih Ukuran --"</option>
                {opsi
                    .into_iter()
                    .map(|u| view! { <option value=u.clone()>{u.clone()}</option> })
                    .collect_view()}
            </select>
        }
        .into_any()
    };

    view! {
        <div
            data-testid="ukuran-profil-kolom"
            data-spesifikasi=item.spesifikasi_nama.clone()
            class="rounded-xl border border-white/[0.06] bg-white/[0.03] p-4"
        >
            <label class="mb-2 flex items-center gap-2 text-sm font-medium text-slate-200" for=dom_id>
                <span class=pembungkus>
                    <AppIcon icon=icon_from_fa_class(ikon) size=14 />
                </span>
                {judul}
            </label>
            {kontrol}
            <p class="mt-1.5 text-xs text-slate-500">
                {petunjuk}
                {catatan.map(|c| view! { <span class="ml-1.5 text-gold-400/80">"· " {c}</span> })}
            </p>
        </div>
    }
}
