//! Pilih data yang sudah ada rujukannya, jangan ketik ulang.
//!
//! Arahan pengguna (2026-08-29): *"jangan isi data yang sebenarnya sudah ada
//! referensi rujukannya di database, agar tidak salah input dan duplikasi"*.
//! Formulir permohonan sebelumnya meminta operator mengetik ulang identitas
//! pegawai (nama, golongan, pangkat, unit kerja) dan identitas aset (kode
//! barang, nama barang, NUP, merk) yang seluruhnya sudah ada di
//! `integrasi.mysimkari_pegawai` dan `integrasi.siman_aset` — dan usulan
//! penghapusan bahkan meminta **UUID aset diketik tangan**.
//!
//! # Kenapa daftar-hasil, bukan dropdown
//!
//! Diukur di staging, 2026-08-29:
//!
//! | | |
//! |---|---|
//! | baris `siman_aset` | 624.533 |
//! | nama barang berbeda | **2.030** |
//! | aset bernama "station wagon" | **785** |
//! | aset dengan nama+NUP paling ambigu | **515** ("Contacless Card Reader E-KTP" NUP 1) |
//!
//! Jadi `<select>` mustahil (624 ribu opsi), dan memilih *nama* tidak memilih
//! *aset*: satu nama rata-rata dipakai 307 aset. Bahkan **nama + NUP tidak
//! unik** secara nasional. Identitas aset adalah **kode satker + kode barang +
//! NUP**, sehingga baris hasil harus menampilkan aset konkret dan pencariannya
//! ter-scope ke satker pemanggil (server sudah melakukannya lewat
//! `AsetScope::from_claims`). Di dalam satu satker pencarian nama memakan 6 ms;
//! secara nasional 230 ms — debounce 300 ms sudah cukup.
//!
//! Pegawai berbeda kasus: 21.328 baris dengan NIP **unik di semuanya**, jadi
//! mengetik NIP saja sudah menentukan orangnya. Tidak perlu daftar.
//!
//! # Setelah dipilih, field menjadi tampilan — bukan input
//!
//! Itu bagian yang benar-benar mencegah salah ketik dan duplikasi. Yang
//! dikirim ke server hanyalah kunci rujukannya (NIP; kode satker + kode barang
//! + NUP), bukan salinan ketikan operator.

use leptos::prelude::*;
use leptos::task::spawn_local;
use lib_ui::components::foto_pegawai::foto_pegawai_url;
use lib_ui::components::icon::AppIcon;
use lib_ui::components::optimized_image::{Avatar, AvatarSize};
use phosphor_leptos::{ARROW_COUNTER_CLOCKWISE, MAGNIFYING_GLASS, WARNING};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use crate::api::bank_aset::{BankAsetItem, ListFilter, fetch_list};
use crate::api::pemakaian_bmn::{CekPegawaiResponse, fetch_cek_pegawai};

/// Kelas satu field pencarian, sama dengan konvensi form lain.
const FIELD: &str = "w-full rounded-lg border border-white/10 bg-slate-900/70 px-4 py-2 \
                     text-slate-100 placeholder:text-slate-500 outline-none transition-colors \
                     hover:border-white/20 focus:border-gold-400/60 focus:ring-2 \
                     focus:ring-gold-400/40";

/// Panjang NIP. Pencarian baru dijalankan setelah operator selesai mengetiknya
/// — NIP unik, jadi tidak ada gunanya menembak server pada prefix.
const NIP_LEN: usize = 18;

fn input_value(ev: &web_sys::Event) -> String {
    ev.target()
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .map(|i| i.value())
        .unwrap_or_default()
}

// ============================================================================
// Pegawai
// ============================================================================

/// Ketik NIP → identitas pegawai terisi sendiri.
///
/// Menampilkan juga izin pemakaian yang sedang aktif atas nama pegawai itu:
/// backend sudah mengirimkannya, dan itu justru hal yang perlu dilihat
/// operator sebelum mengajukan izin baru.
#[component]
pub fn PegawaiPicker(
    /// Dipanggil setiap kali seorang pegawai berhasil dikenali, dan dengan
    /// `None` saat pilihan dibatalkan.
    #[prop(into)]
    on_pick: Callback<Option<CekPegawaiResponse>>,
) -> impl IntoView {
    let (nip, set_nip) = signal(String::new());
    let (found, set_found) = signal(None::<CekPegawaiResponse>);
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);

    // Debounce: setiap perubahan menjadwalkan tugas tertunda; kalau nilainya
    // masih sama setelah jeda (operator berhenti mengetik) barulah dikirim.
    // Pola yang sama dengan daftar kebutuhan BMN.
    Effect::new(move |_| {
        let pending = nip.get();
        if pending.len() != NIP_LEN {
            set_found.set(None);
            set_error.set(None);
            on_pick.run(None);
            return;
        }
        set_busy.set(true);
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(300).await;
            if nip.get_untracked() != pending {
                return; // ketikan sudah berubah — tugas ini basi
            }
            match fetch_cek_pegawai(&pending).await {
                Ok(Some(resp)) => {
                    set_error.set(None);
                    set_found.set(Some(resp.clone()));
                    on_pick.run(Some(resp));
                }
                Ok(None) => {
                    set_found.set(None);
                    on_pick.run(None);
                    set_error.set(Some(
                        "NIP tidak ditemukan pada data kepegawaian satker ini.".to_string(),
                    ));
                }
                Err(e) => {
                    set_found.set(None);
                    on_pick.run(None);
                    set_error.set(Some(e.user_message()));
                }
            }
            set_busy.set(false);
        });
    });

    view! {
        <div class="space-y-3">
            <div>
                <label class="mb-1 block text-sm font-medium text-slate-300" for="pegawai-nip">
                    "NIP Pegawai"
                </label>
                <input
                    id="pegawai-nip"
                    class=FIELD
                    inputmode="numeric"
                    maxlength=NIP_LEN.to_string()
                    placeholder="18 digit NIP — data pegawai terisi sendiri"
                    prop:value=move || nip.get()
                    on:input=move |ev| set_nip.set(input_value(&ev))
                />
                <p class="mt-1 text-xs text-slate-500">
                    {move || {
                        let n = nip.get().len();
                        if busy.get() {
                            "Mencari…".to_string()
                        } else if n == 0 {
                            "Cukup NIP-nya. Nama, pangkat, jabatan, dan satker diambil dari data kepegawaian."
                                .to_string()
                        } else if n < NIP_LEN {
                            format!("{} dari {} digit", n, NIP_LEN)
                        } else {
                            String::new()
                        }
                    }}
                </p>
            </div>

            {move || {
                error
                    .get()
                    .map(|msg| {
                        view! {
                            <p class="rounded-lg border border-danger-500/20 bg-danger-500/10 px-3 py-2 text-sm text-danger-300">
                                {msg}
                            </p>
                        }
                    })
            }}

            {move || {
                found
                    .get()
                    .map(|resp| {
                        let p = resp.pegawai.clone();
                        let aktif = resp.pemakaian_aktif.len();
                        let nama = p.nama.clone().unwrap_or_else(|| "-".to_string());
                        view! {
                            // `data-testid` seperti pada AsetPicker. Kartu ini
                            // tak punya satu pun, sehingga tak ada locator yang
                            // bisa menunjuknya — dan `getByText` di sini sudah
                            // pernah lulus sambil membaca elemen lain.
                            <div
                                data-testid="pegawai-terpilih"
                                class="rounded-xl border border-white/[0.06] bg-white/[0.03] p-4"
                            >
                                // Foto pegawai dari MySIMKARI. Kolomnya sudah
                                // ada, sinkronisasi sudah menulisnya, API sudah
                                // mengembalikannya — tapi tak pernah ada satu
                                // pun `<img>` yang menampilkannya di kedua
                                // frontend. Operator melihat wajah orang yang
                                // ia ajukan, bukan hanya deretan angka NIP.
                                <div class="flex items-start gap-4">
                                    <Avatar
                                        src=foto_pegawai_url(p.foto.as_deref()).unwrap_or_default()
                                        name=nama.clone()
                                        size=AvatarSize::XLarge
                                    />
                                    <div class="min-w-0 flex-1">
                                        <p class="text-base font-semibold text-slate-100">{nama}</p>
                                    </div>
                                </div>
                                <dl class="mt-3 grid grid-cols-1 gap-x-6 gap-y-2 sm:grid-cols-2">
                                    <ReadOnlyField label="NIP" value=p.nip.clone() />
                                    <ReadOnlyField
                                        label="Pangkat / Golongan"
                                        value=p.pangkat.clone().unwrap_or_else(|| "-".to_string())
                                    />
                                    <ReadOnlyField
                                        label="Jabatan"
                                        value=p.jabatan.clone().unwrap_or_else(|| "-".to_string())
                                    />
                                    <ReadOnlyField
                                        label="Satuan Kerja"
                                        value=p.nama_satker.clone().unwrap_or_else(|| "-".to_string())
                                    />
                                </dl>
                                {(aktif > 0)
                                    .then(|| {
                                        view! {
                                            <p class="mt-3 flex items-start gap-2 rounded-lg border border-warning-500/20 bg-warning-500/10 px-3 py-2 text-sm text-warning-300">
                                                <span class="mt-0.5">
                                                    <AppIcon icon=WARNING size=16 />
                                                </span>
                                                {format!(
                                                    "Pegawai ini sedang memegang {aktif} izin pemakaian BMN yang masih aktif.",
                                                )}
                                            </p>
                                        }
                                    })}
                            </div>
                        }
                    })
            }}
        </div>
    }
}

// ============================================================================
// Aset
// ============================================================================

/// Cari aset lalu pilih barisnya — bukan namanya.
#[component]
pub fn AsetPicker(
    /// Dipanggil dengan aset terpilih, atau `None` saat pilihan dibatalkan.
    #[prop(into)]
    on_pick: Callback<Option<BankAsetItem>>,
) -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (results, set_results) = signal(Vec::<BankAsetItem>::new());
    let (chosen, set_chosen) = signal(None::<BankAsetItem>);
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (searched, set_searched) = signal(false);

    Effect::new(move |_| {
        let pending = query.get();
        if chosen.get_untracked().is_some() {
            return; // sudah memilih; kotak cari disembunyikan
        }
        if pending.trim().len() < 3 {
            set_results.set(Vec::new());
            set_searched.set(false);
            return;
        }
        set_busy.set(true);
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(300).await;
            if query.get_untracked() != pending {
                return;
            }
            let filter = ListFilter {
                page: 1,
                per_page: 15,
                search: Some(pending.trim().to_string()),
                ..Default::default()
            };
            match fetch_list(&filter).await {
                Ok(page) => {
                    set_error.set(None);
                    set_results.set(page.data);
                }
                Err(e) => {
                    set_results.set(Vec::new());
                    set_error.set(Some(e.user_message()));
                }
            }
            set_searched.set(true);
            set_busy.set(false);
        });
    });

    let clear = move |_| {
        set_chosen.set(None);
        set_query.set(String::new());
        set_results.set(Vec::new());
        set_searched.set(false);
        on_pick.run(None);
    };

    view! {
        <div class="space-y-3">
            <Show
                when=move || chosen.get().is_none()
                fallback=move || {
                    let item = chosen.get().expect("fallback hanya saat ada pilihan");
                    view! {
                        // Locator struktural untuk e2e: `getByText` mencocokkan
                        // substring dan mengabaikan huruf besar-kecil, sehingga
                        // pernah lulus sambil membaca elemen lain.
                        <div
                            data-testid="aset-terpilih"
                            class="rounded-xl border border-white/[0.06] bg-white/[0.03] p-4"
                        >
                            <div class="flex items-start justify-between gap-3">
                                <p class="text-base font-semibold text-slate-100">
                                    {item.nama_aset.clone().unwrap_or_else(|| "-".to_string())}
                                </p>
                                <button
                                    type="button"
                                    class="inline-flex shrink-0 items-center gap-1.5 rounded-lg border border-white/10 px-3 py-1.5 text-xs text-slate-200 transition-colors hover:bg-white/[0.06]"
                                    on:click=clear
                                >
                                    <AppIcon icon=ARROW_COUNTER_CLOCKWISE size=14 />
                                    "Ganti"
                                </button>
                            </div>
                            <dl class="mt-3 grid grid-cols-1 gap-x-6 gap-y-2 sm:grid-cols-2">
                                <ReadOnlyField label="Kode Barang" value=item.kode_barang.clone().unwrap_or_else(|| "-".to_string()) />
                                <ReadOnlyField label="NUP" value=item.no_aset.clone() />
                                <ReadOnlyField
                                    label="Merk / Tipe"
                                    value=merk_tipe(&item)
                                />
                                <ReadOnlyField label="Kondisi" value=item.kondisi.clone().unwrap_or_else(|| "-".to_string()) />
                                <ReadOnlyField label="Satuan Kerja" value=item.satker.clone().unwrap_or_else(|| "-".to_string()) />
                                <ReadOnlyField label="Kode Satker" value=item.kode_satker.clone().unwrap_or_else(|| "-".to_string()) />
                            </dl>
                        </div>
                    }
                }
            >
                <div>
                    <label class="mb-1 block text-sm font-medium text-slate-300" for="aset-cari">
                        "Cari BMN"
                    </label>
                    <div class="relative">
                        <span class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-slate-500">
                            <AppIcon icon=MAGNIFYING_GLASS size=16 />
                        </span>
                        <input
                            id="aset-cari"
                            class=format!("{FIELD} pl-9")
                            placeholder="Ketik nama barang, kode barang, NUP, atau merk — misal: station wagon"
                            prop:value=move || query.get()
                            on:input=move |ev| set_query.set(input_value(&ev))
                        />
                    </div>
                    <p class="mt-1 text-xs text-slate-500">
                        {move || {
                            if busy.get() {
                                "Mencari…".to_string()
                            } else if query.get().trim().len() < 3 {
                                "Minimal 3 huruf. Hasilnya hanya BMN di lingkup satker Anda."
                                    .to_string()
                            } else {
                                format!("{} hasil", results.get().len())
                            }
                        }}
                    </p>
                </div>

                {move || {
                    error
                        .get()
                        .map(|msg| {
                            view! {
                                <p class="rounded-lg border border-danger-500/20 bg-danger-500/10 px-3 py-2 text-sm text-danger-300">
                                    {msg}
                                </p>
                            }
                        })
                }}

                <Show when=move || searched.get() && results.get().is_empty() && !busy.get()>
                    <p
                        data-testid="aset-kosong"
                        class="rounded-lg border border-white/[0.06] bg-white/[0.03] px-3 py-2 text-sm text-slate-400"
                    >
                        "Tidak ada BMN yang cocok di lingkup satker Anda."
                    </p>
                </Show>

                <Show when=move || !results.get().is_empty()>
                    <ul
                        data-testid="aset-hasil"
                        class="max-h-72 divide-y divide-white/[0.06] overflow-y-auto rounded-xl border border-white/[0.06] bg-surface-panel"
                    >
                        <For
                            each=move || results.get()
                            key=|item| item.id.clone()
                            children=move |item| {
                                let picked = item.clone();
                                view! {
                                    <li>
                                        <button
                                            type="button"
                                            class="w-full px-4 py-3 text-left transition-colors hover:bg-white/[0.04]"
                                            on:click=move |_| {
                                                set_chosen.set(Some(picked.clone()));
                                                on_pick.run(Some(picked.clone()));
                                            }
                                        >
                                            <p class="text-sm font-medium text-slate-100">
                                                {item.nama_aset.clone().unwrap_or_else(|| "-".to_string())}
                                            </p>
                                            // Baris kedua memuat pembeda yang
                                            // benar-benar membedakan: satu nama
                                            // dipakai ratusan aset, dan nama+NUP
                                            // pun tidak unik lintas satker.
                                            <p class="mt-0.5 text-xs text-slate-400">
                                                {format!(
                                                    "NUP {} · {} · {}",
                                                    if item.no_aset.is_empty() { "-" } else { &item.no_aset },
                                                    item.kode_barang.clone().unwrap_or_else(|| "-".to_string()),
                                                    merk_tipe(&item),
                                                )}
                                            </p>
                                            <p class="mt-0.5 text-[0.7rem] text-slate-500">
                                                {item.satker.clone().unwrap_or_else(|| "-".to_string())}
                                            </p>
                                        </button>
                                    </li>
                                }
                            }
                        />
                    </ul>
                </Show>
            </Show>
        </div>
    }
}

/// Merk dan tipe adalah penamaan bebas operator SIMAN, bukan nama resmi
/// barang — digabung hanya untuk baris pembeda, tidak pernah menggantikan
/// `nama_aset`.
pub fn merk_tipe(item: &BankAsetItem) -> String {
    let merk = item
        .merk
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let tipe = item
        .tipe
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    match (merk, tipe) {
        (Some(m), Some(t)) => format!("{m} {t}"),
        (Some(m), None) => m.to_string(),
        (None, Some(t)) => t.to_string(),
        (None, None) => "-".to_string(),
    }
}

/// Nilai hasil rujukan ditampilkan, bukan di-input. Inilah bagian yang
/// mencegah salah ketik: tidak ada kotak untuk mengetik ulang.
#[component]
fn ReadOnlyField(#[prop(into)] label: String, #[prop(into)] value: String) -> impl IntoView {
    view! {
        <div>
            <dt class="text-[0.7rem] font-semibold uppercase tracking-wider text-slate-500">
                {label}
            </dt>
            <dd class="mt-0.5 text-sm text-slate-100">{value}</dd>
        </div>
    }
}
