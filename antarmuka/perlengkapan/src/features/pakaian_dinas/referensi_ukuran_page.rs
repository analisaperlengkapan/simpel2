//! Referensi Ukuran: grup ukuran dan daftar ukuran tiap grup.
//!
//! Dulu grup ukuran hanya boleh BAJU, CELANA, atau SEPATU (dua `CHECK` di basis data) dan
//! daftarnya ada di migrasi, jadi sebuah jenis pakaian baru (kaos, topi, sepatu PDL, nama dada)
//! berarti migrasi dan rilis. Kini keduanya data: admin menambah grup (daftar pilihan atau isian
//! teks), menambah ukuran ke daftarnya, lalu memakai grup itu pada sebuah spesifikasi.
//!
//! Peran lain boleh MELIHAT (daftar ukuran dipakai semua layar pengisian); kendali ubah hanya
//! tampil bagi admin dan server menolak yang lain (`require_admin`). Grup atau ukuran yang masih
//! dipakai isian tersimpan ditolak server dengan jumlahnya, dan pesan itu ditampilkan apa adanya.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_fetch::QueryClient;

use crate::api::{AppError, Ukuran, fetch_master_ukuran};
use crate::components::layout::{
    EmptyState, ErrorState, FormField, LoadingState, PageLayout, SectionCard,
};
use crate::features::auth::AuthService;
use crate::features::pakaian_dinas::api::{
    CreateUkuranGrupRequest, CreateUkuranRequest, UkuranGrup, UpdateUkuranGrupRequest, buat_ukuran,
    buat_ukuran_grup, fetch_ukuran_grup, hapus_ukuran, hapus_ukuran_grup, ubah_ukuran_grup,
    urutkan_ukuran_grup,
};
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{ARROW_DOWN, ARROW_UP, CHECK_CIRCLE, PLUS, TRASH, WARNING_CIRCLE};

const FIELD: &str = "focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] \
                     px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500";

const TOMBOL_EMAS: &str = "inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-4 py-2.5 \
                           text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90 \
                           disabled:opacity-50";

const TOMBOL_NETRAL: &str = "rounded-lg border border-white/10 bg-white/[0.04] px-4 py-2.5 text-sm \
                             text-slate-300 transition hover:bg-white/[0.08] disabled:opacity-50";

const TOMBOL_BAHAYA: &str = "inline-flex items-center gap-2 rounded-lg border border-danger-500/30 \
                             px-3 py-2 text-xs text-danger-300 transition hover:bg-danger-500/10 \
                             disabled:opacity-50";

#[derive(Clone, Debug, PartialEq)]
struct Referensi {
    grup: Vec<UkuranGrup>,
    ukuran: Vec<Ukuran>,
}

/// Grup dan daftar ukuran, dimuat bersama dan paralel. `_muat` hanya pemicu muat ulang.
async fn query_referensi(_muat: i32) -> Result<Referensi, AppError> {
    let (grup, ukuran) = futures::future::try_join(fetch_ukuran_grup(false), async {
        fetch_master_ukuran(None).await.map(|r| r.data)
    })
    .await?;
    Ok(Referensi { grup, ukuran })
}

/// Kode yang dikirim server mengikuti aturan ini; dipakai agar grup yang baru dibuat langsung terpilih.
fn normalkan_kode(kode: &str) -> String {
    kode.trim().to_uppercase().replace([' ', '-'], "_")
}

/// Segmen URL: ukuran seperti "L/XL" atau "2 XL" tak boleh memecah rute.
fn segmen(s: &str) -> String {
    String::from(js_sys::encode_uri_component(s))
}

/// Jalankan satu perubahan: tanda sibuk, pesan, dan muat ulang diurus di satu tempat supaya
/// semua tombol berperilaku sama (galat server tampil apa adanya, bukan ditelan).
fn jalankan<F>(
    sibuk: RwSignal<bool>,
    galat: RwSignal<Option<String>>,
    info: RwSignal<Option<String>>,
    muat: RwSignal<i32>,
    sukses: String,
    kerja: F,
) where
    F: std::future::Future<Output = Result<(), AppError>> + 'static,
{
    sibuk.set(true);
    galat.set(None);
    info.set(None);
    spawn_local(async move {
        match kerja.await {
            Ok(()) => {
                info.set(Some(sukses));
                muat.update(|v| *v += 1);
            }
            Err(e) => {
                galat.set(Some(e.user_message()));
                // Hasilnya mungkin tak pasti (ditolak karena daftar sudah berubah, atau terputus):
                // muat ulang supaya layar kembali sama dengan server, bukan menebak.
                muat.update(|v| *v += 1);
            }
        }
        sibuk.set(false);
    });
}

#[component]
pub fn ReferensiUkuranPage() -> impl IntoView {
    let is_admin = AuthService::load_session()
        .map(|s| s.is_admin())
        .unwrap_or(false);
    let muat = RwSignal::new(0);
    let terpilih = RwSignal::new(Option::<String>::None);
    let galat = RwSignal::new(Option::<String>::None);
    let info = RwSignal::new(Option::<String>::None);
    let sibuk = RwSignal::new(false);
    let form_grup_terbuka = RwSignal::new(false);

    let client: QueryClient = expect_context();
    let data = client.local_resource(query_referensi, move || muat.get());

    view! {
        <PageLayout
            title="Referensi Ukuran"
            icon="fas fa-ruler"
            description="Grup ukuran dan daftar ukurannya. Jenis atau atribut pakaian baru cukup ditambahkan di sini, lalu dipakai pada sebuah spesifikasi."
        >
            <Show when=move || info.get().is_some()>
                <div
                    data-testid="referensi-sukses"
                    class="mb-4 flex items-center gap-2 rounded-xl border border-success-500/30 bg-success-500/[0.08] px-4 py-3 text-sm text-success-300"
                >
                    <AppIcon icon=CHECK_CIRCLE />
                    {move || info.get()}
                </div>
            </Show>
            <Show when=move || galat.get().is_some()>
                <div
                    data-testid="referensi-galat"
                    class="mb-4 flex items-center gap-2 rounded-xl border border-danger-500/30 bg-danger-500/[0.08] px-4 py-3 text-sm text-danger-300"
                >
                    <AppIcon icon=WARNING_CIRCLE />
                    {move || galat.get()}
                </div>
            </Show>

            <Suspense fallback=move || {
                view! { <LoadingState /> }
            }>
                {move || match data.get() {
                    None => view! { <LoadingState /> }.into_any(),
                    Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                    Some(Ok(ref_data)) => {
                        // Pilihan awal / pilihan yang sudah tak ada (dihapus) jatuh ke grup pertama.
                        // Daftar KOSONG tetap menggambar kolom kiri: tombol dan formulir "Tambah
                        // Grup" ada di sana, dan tanpa mereka admin tak bisa membuat grup pertama
                        // atau memulihkan referensi setelah grup terakhir dihapus.
                        let grup_terpilih: Option<UkuranGrup> = terpilih
                            .get()
                            .and_then(|k| ref_data.grup.iter().find(|g| g.kode == k).cloned())
                            .or_else(|| ref_data.grup.first().cloned());
                        let kode_terpilih = grup_terpilih
                            .as_ref()
                            .map(|g| g.kode.clone())
                            .unwrap_or_default();
                        let detail = match grup_terpilih {
                            Some(g) => {
                                let ukuran_grup: Vec<Ukuran> = ref_data
                                    .ukuran
                                    .iter()
                                    .filter(|u| u.group.eq_ignore_ascii_case(&g.kode))
                                    .cloned()
                                    .collect();
                                detail_grup(g, ukuran_grup, is_admin, sibuk, galat, info, muat)
                            }
                            None => view! {
                                <EmptyState
                                    icon="fas fa-ruler"
                                    title="Belum Ada Grup Ukuran"
                                    description="Tanpa grup, spesifikasi pakaian tak punya daftar ukuran."
                                />
                            }
                                .into_any(),
                        };
                        view! {
                            <div class="grid grid-cols-1 gap-5 lg:grid-cols-3">
                                <div class="lg:col-span-1">
                                    {daftar_grup(
                                        ref_data.grup.clone(),
                                        ref_data.ukuran.clone(),
                                        kode_terpilih,
                                        terpilih,
                                        is_admin,
                                        form_grup_terbuka,
                                    )}
                                    <Show when=move || is_admin && form_grup_terbuka.get()>
                                        {form_tambah_grup(sibuk, galat, info, muat, terpilih, form_grup_terbuka)}
                                    </Show>
                                </div>
                                <div class="lg:col-span-2">{detail}</div>
                            </div>
                        }
                            .into_any()
                    }
                }}
            </Suspense>
        </PageLayout>
    }
}

fn daftar_grup(
    grup: Vec<UkuranGrup>,
    ukuran: Vec<Ukuran>,
    kode_terpilih: String,
    terpilih: RwSignal<Option<String>>,
    is_admin: bool,
    form_terbuka: RwSignal<bool>,
) -> AnyView {
    view! {
        <SectionCard
            title="Grup Ukuran"
            actions=Box::new(move || {
                view! {
                    <Show when=move || is_admin>
                        <button
                            type="button"
                            data-testid="tambah-grup"
                            class=TOMBOL_EMAS
                            on:click=move |_| form_terbuka.update(|v| *v = !*v)
                        >
                            <AppIcon icon=PLUS />
                            "Tambah Grup"
                        </button>
                    </Show>
                }
                    .into_any()
            })
        >
            {grup
                .is_empty()
                .then(|| {
                    view! {
                        <p data-testid="grup-kosong" class="mb-2 text-sm text-slate-400">
                            "Belum ada grup."
                        </p>
                    }
                })}
            <ul data-testid="grup-daftar" class="flex flex-col gap-2">
                {grup
                    .into_iter()
                    .map(|g| {
                        let aktif = g.kode == kode_terpilih;
                        let jumlah = ukuran
                            .iter()
                            .filter(|u| u.group.eq_ignore_ascii_case(&g.kode))
                            .count();
                        let kode = g.kode.clone();
                        let kelas = if aktif {
                            "w-full rounded-xl border border-gold-500/40 bg-gold-500/[0.08] p-3 text-left transition"
                        } else {
                            "w-full rounded-xl border border-white/[0.06] bg-white/[0.03] p-3 text-left transition hover:bg-white/[0.06]"
                        };
                        let teks = g.is_teks();
                        view! {
                            <li>
                                <button
                                    type="button"
                                    data-testid=format!("grup-{}", g.kode)
                                    aria-pressed=aktif.to_string()
                                    class=kelas
                                    on:click=move |_| terpilih.set(Some(kode.clone()))
                                >
                                    <span class="flex items-center justify-between gap-2">
                                        <span class="text-sm font-semibold text-slate-100">
                                            {g.nama.clone()}
                                        </span>
                                        <span class="rounded-full border border-white/10 px-2 py-0.5 text-[10px] uppercase tracking-wide text-slate-400">
                                            {if teks { "Teks" } else { "Pilihan" }}
                                        </span>
                                    </span>
                                    <span class="mt-1 flex items-center justify-between gap-2 text-xs text-slate-500">
                                        <span class="font-mono">{g.kode.clone()}</span>
                                        <span>
                                            {if teks {
                                                "isian bebas".to_string()
                                            } else {
                                                format!("{jumlah} ukuran")
                                            }}
                                        </span>
                                    </span>
                                    {(!g.is_active)
                                        .then(|| {
                                            view! {
                                                <span class="mt-1.5 inline-block rounded-full bg-slate-500/20 px-2 py-0.5 text-[10px] text-slate-300">
                                                    "Nonaktif"
                                                </span>
                                            }
                                        })}
                                </button>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </SectionCard>
    }
    .into_any()
}

fn form_tambah_grup(
    sibuk: RwSignal<bool>,
    galat: RwSignal<Option<String>>,
    info: RwSignal<Option<String>>,
    muat: RwSignal<i32>,
    terpilih: RwSignal<Option<String>>,
    form_terbuka: RwSignal<bool>,
) -> AnyView {
    let kode = RwSignal::new(String::new());
    let nama = RwSignal::new(String::new());
    let tipe = RwSignal::new("PILIHAN".to_string());

    let kirim = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let kode_baru = normalkan_kode(&kode.get());
        let request = CreateUkuranGrupRequest {
            kode: kode.get(),
            nama: nama.get(),
            tipe: Some(tipe.get()),
            urutan: None,
        };
        let nama_grup = nama.get();
        jalankan(
            sibuk,
            galat,
            info,
            muat,
            format!("Grup \"{nama_grup}\" ditambahkan."),
            async move {
                buat_ukuran_grup(&request).await?;
                terpilih.set(Some(kode_baru));
                kode.set(String::new());
                nama.set(String::new());
                form_terbuka.set(false);
                Ok(())
            },
        );
    };

    view! {
        <div class="mt-5">
            <SectionCard title="Tambah Grup">
                <form data-testid="form-grup" on:submit=kirim class="flex flex-col gap-4">
                    <FormField
                        label="Kode"
                        for_id="grup-kode"
                        helper="Huruf, angka, dan garis bawah. Contoh: SEPATU_PDL. Tak bisa diubah setelah dibuat."
                        required=true
                    >
                        <input
                            id="grup-kode"
                            type="text"
                            class=FIELD
                            maxlength="50"
                            placeholder="SEPATU_PDL"
                            prop:value=move || kode.get()
                            on:input=move |ev| kode.set(event_target_value(&ev))
                            required=true
                        />
                    </FormField>
                    <FormField label="Nama" for_id="grup-nama" required=true>
                        <input
                            id="grup-nama"
                            type="text"
                            class=FIELD
                            maxlength="100"
                            placeholder="Sepatu PDL"
                            prop:value=move || nama.get()
                            on:input=move |ev| nama.set(event_target_value(&ev))
                            required=true
                        />
                    </FormField>
                    <FormField
                        label="Jenis isian"
                        for_id="grup-tipe"
                        helper="Tak bisa diubah setelah dibuat: isian yang tersimpan mengikuti aturannya."
                        required=true
                    >
                        <select
                            id="grup-tipe"
                            class=FIELD
                            prop:value=move || tipe.get()
                            on:change=move |ev| tipe.set(event_target_value(&ev))
                        >
                            <option value="PILIHAN">"Pilihan dari daftar ukuran"</option>
                            <option value="TEKS">"Teks bebas (mis. nama dada)"</option>
                        </select>
                    </FormField>
                    <div class="flex gap-3 border-t border-white/[0.04] pt-4">
                        <button
                            type="submit"
                            data-testid="simpan-grup"
                            class=TOMBOL_EMAS
                            prop:disabled=move || sibuk.get()
                        >
                            {move || if sibuk.get() { "Menyimpan..." } else { "Simpan" }}
                        </button>
                        <button type="button" class=TOMBOL_NETRAL on:click=move |_| form_terbuka.set(false)>
                            "Batal"
                        </button>
                    </div>
                </form>
            </SectionCard>
        </div>
    }
    .into_any()
}

fn detail_grup(
    grup: UkuranGrup,
    ukuran: Vec<Ukuran>,
    is_admin: bool,
    sibuk: RwSignal<bool>,
    galat: RwSignal<Option<String>>,
    info: RwSignal<Option<String>>,
    muat: RwSignal<i32>,
) -> AnyView {
    let teks = grup.is_teks();
    let kode = StoredValue::new(grup.kode.clone());
    let nama = RwSignal::new(grup.nama.clone());
    let aktif = RwSignal::new(grup.is_active);
    let konfirmasi_hapus = RwSignal::new(false);

    let simpan = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let request = UpdateUkuranGrupRequest {
            nama: nama.get(),
            urutan: None,
            is_active: aktif.get(),
        };
        let k = kode.get_value();
        jalankan(
            sibuk,
            galat,
            info,
            muat,
            "Perubahan grup disimpan.".to_string(),
            async move { ubah_ukuran_grup(&k, &request).await },
        );
    };
    let hapus = move |_| {
        if !konfirmasi_hapus.get() {
            konfirmasi_hapus.set(true);
            return;
        }
        konfirmasi_hapus.set(false);
        let k = kode.get_value();
        jalankan(
            sibuk,
            galat,
            info,
            muat,
            "Grup dihapus.".to_string(),
            async move { hapus_ukuran_grup(&k).await },
        );
    };

    view! {
        <div class="flex flex-col gap-5" data-testid="grup-detail" data-grup=grup.kode.clone()>
            <SectionCard title=grup.nama.clone() description=format!("Kode {}", grup.kode)>
                <Show
                    when=move || is_admin
                    fallback=move || {
                        view! {
                            <p class="text-sm text-slate-400">
                                "Hanya administrator yang dapat mengubah referensi ini."
                            </p>
                        }
                    }
                >
                    <form on:submit=simpan class="flex flex-col gap-4">
                        <div class="grid grid-cols-1 gap-5 sm:grid-cols-2">
                            <FormField label="Nama" for_id="grup-nama-ubah" required=true>
                                <input
                                    id="grup-nama-ubah"
                                    type="text"
                                    class=FIELD
                                    maxlength="100"
                                    prop:value=move || nama.get()
                                    on:input=move |ev| nama.set(event_target_value(&ev))
                                    required=true
                                />
                            </FormField>
                            <FormField
                                label="Status"
                                helper="Grup nonaktif tak ditawarkan untuk spesifikasi baru; yang sudah memakainya tetap berjalan."
                            >
                                <label class="flex items-center gap-2.5 py-2.5 text-sm text-slate-200">
                                    <input
                                        type="checkbox"
                                        data-testid="grup-aktif"
                                        class="focus-ring h-4 w-4 rounded border-white/20 bg-white/[0.04]"
                                        prop:checked=move || aktif.get()
                                        on:change=move |ev| aktif.set(event_target_checked(&ev))
                                    />
                                    "Aktif"
                                </label>
                            </FormField>
                        </div>
                        <div class="flex flex-wrap items-center justify-between gap-3 border-t border-white/[0.04] pt-4">
                            <button
                                type="button"
                                data-testid="hapus-grup"
                                class=TOMBOL_BAHAYA
                                prop:disabled=move || sibuk.get()
                                on:click=hapus
                            >
                                <AppIcon icon=TRASH size=14 />
                                {move || {
                                    if konfirmasi_hapus.get() {
                                        "Klik lagi untuk menghapus grup"
                                    } else {
                                        "Hapus Grup"
                                    }
                                }}
                            </button>
                            <button
                                type="submit"
                                data-testid="simpan-perubahan-grup"
                                class=TOMBOL_EMAS
                                prop:disabled=move || sibuk.get()
                            >
                                "Simpan Perubahan"
                            </button>
                        </div>
                    </form>
                </Show>
            </SectionCard>

            {if teks {
                view! {
                    <SectionCard title="Isian Teks">
                        <p class="text-sm text-slate-300">
                            "Grup ini berisi teks bebas (paling banyak 100 karakter), jadi tidak punya daftar ukuran. "
                            "Pakai pada sebuah spesifikasi untuk menyediakan kotak isian, misalnya nama dada."
                        </p>
                    </SectionCard>
                }
                    .into_any()
            } else {
                daftar_ukuran(grup.kode.clone(), ukuran, is_admin, sibuk, galat, info, muat)
            }}
        </div>
    }
    .into_any()
}

fn daftar_ukuran(
    grup: String,
    ukuran: Vec<Ukuran>,
    is_admin: bool,
    sibuk: RwSignal<bool>,
    galat: RwSignal<Option<String>>,
    info: RwSignal<Option<String>>,
    muat: RwSignal<i32>,
) -> AnyView {
    let mut urut = ukuran;
    urut.sort_by_key(|u| u.urutan);
    let baru = RwSignal::new(String::new());
    let grup_simpan = StoredValue::new(grup.clone());
    let jumlah = urut.len();
    let hapus_terminta = RwSignal::new(Option::<String>::None);

    let tambah = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let nilai = baru.get().trim().to_string();
        if nilai.is_empty() {
            galat.set(Some("Ukuran harus diisi.".to_string()));
            return;
        }
        let request = CreateUkuranRequest {
            ukuran: nilai.clone(),
            group: grup_simpan.get_value(),
            urutan: None,
        };
        jalankan(
            sibuk,
            galat,
            info,
            muat,
            format!("Ukuran \"{nilai}\" ditambahkan."),
            async move {
                buat_ukuran(&request).await?;
                baru.set(String::new());
                Ok(())
            },
        );
    };

    // Susun ulang: SATU permintaan membawa seluruh urutan baru, yang diterapkan server dalam satu
    // transaksi. Mengirim satu PUT per ukuran membiarkan separuhnya tersimpan bila salah satu gagal.
    let geser = {
        let urut = urut.clone();
        move |dari: usize, ke: usize| {
            let mut susunan: Vec<String> = urut.iter().map(|u| u.ukuran.clone()).collect();
            let item = susunan.remove(dari);
            susunan.insert(ke, item);
            let g = grup_simpan.get_value();
            jalankan(
                sibuk,
                galat,
                info,
                muat,
                "Urutan disimpan.".to_string(),
                async move { urutkan_ukuran_grup(&g, susunan).await },
            );
        }
    };

    view! {
        <SectionCard
            title="Daftar Ukuran"
            description="Urutan di sini menjadi urutan pilihan di layar pengisian."
        >
            <Show when=move || is_admin>
                <form on:submit=tambah class="mb-5 flex items-end gap-3">
                    <div class="flex-1">
                        <FormField label="Ukuran baru" for_id="ukuran-baru">
                            <input
                                id="ukuran-baru"
                                type="text"
                                data-testid="ukuran-baru"
                                class=FIELD
                                maxlength="20"
                                placeholder="Contoh: 4XL, 44, 58"
                                prop:value=move || baru.get()
                                on:input=move |ev| baru.set(event_target_value(&ev))
                            />
                        </FormField>
                    </div>
                    <button
                        type="submit"
                        data-testid="tambah-ukuran"
                        class=TOMBOL_EMAS
                        prop:disabled=move || sibuk.get()
                    >
                        <AppIcon icon=PLUS />
                        "Tambah"
                    </button>
                </form>
            </Show>

            {if jumlah == 0 {
                view! {
                    <p data-testid="ukuran-kosong" class="text-sm text-slate-400">
                        "Grup ini belum punya ukuran, jadi pilihannya kosong di layar pengisian."
                    </p>
                }
                    .into_any()
            } else {
                view! {
                    <ol data-testid="ukuran-daftar" class="grid grid-cols-1 gap-2 sm:grid-cols-2">
                        {urut
                            .into_iter()
                            .enumerate()
                            .map(|(i, u)| {
                                // Batas daftar dihitung di luar markup (`>`, `=` di atribut `view!`
                                // dibaca sebagai akhir tag oleh perkakas).
                                let paling_atas = i == 0;
                                let paling_bawah = i + 1 >= jumlah;
                                let nilai = u.ukuran.clone();
                                let nilai_hapus = u.ukuran.clone();
                                let nilai_tampil = u.ukuran.clone();
                                let geser_naik = geser.clone();
                                let geser_turun = geser.clone();
                                let g = grup.clone();
                                view! {
                                    <li
                                        data-testid="ukuran-baris"
                                        data-ukuran=u.ukuran.clone()
                                        class="flex items-center justify-between gap-2 rounded-lg border border-white/[0.06] bg-white/[0.03] px-3 py-2"
                                    >
                                        <span class="text-sm font-medium text-slate-100">{u.ukuran.clone()}</span>
                                        {is_admin.then(|| {
                                            view! {
                                            <span class="flex items-center gap-1">
                                                <button
                                                    type="button"
                                                    aria-label=format!("Naikkan {nilai}")
                                                    data-testid=format!("naik-{nilai}")
                                                    class="rounded p-1.5 text-slate-400 transition hover:bg-white/[0.08] hover:text-slate-100 disabled:opacity-30"
                                                    prop:disabled=move || sibuk.get() || paling_atas
                                                    on:click={
                                                        let geser_naik = geser_naik.clone();
                                                        move |_| geser_naik(i, i - 1)
                                                    }
                                                >
                                                    <AppIcon icon=ARROW_UP size=14 />
                                                </button>
                                                <button
                                                    type="button"
                                                    aria-label=format!("Turunkan {nilai}")
                                                    data-testid=format!("turun-{nilai}")
                                                    class="rounded p-1.5 text-slate-400 transition hover:bg-white/[0.08] hover:text-slate-100 disabled:opacity-30"
                                                    prop:disabled=move || sibuk.get() || paling_bawah
                                                    on:click={
                                                        let geser_turun = geser_turun.clone();
                                                        move |_| geser_turun(i, i + 1)
                                                    }
                                                >
                                                    <AppIcon icon=ARROW_DOWN size=14 />
                                                </button>
                                                <button
                                                    type="button"
                                                    aria-label=format!("Hapus {nilai_hapus}")
                                                    data-testid=format!("hapus-{nilai_hapus}")
                                                    class="rounded p-1.5 text-danger-300 transition hover:bg-danger-500/10 disabled:opacity-30"
                                                    prop:disabled=move || sibuk.get()
                                                    on:click={
                                                        let nilai_hapus = nilai_hapus.clone();
                                                        let g = g.clone();
                                                        move |_| {
                                                            // Dua langkah: klik pertama meminta
                                                            // konfirmasi di tempat, klik kedua menghapus.
                                                            if hapus_terminta.get().as_deref() != Some(nilai_hapus.as_str()) {
                                                                hapus_terminta.set(Some(nilai_hapus.clone()));
                                                                return;
                                                            }
                                                            hapus_terminta.set(None);
                                                            let n = nilai_hapus.clone();
                                                            let g = g.clone();
                                                            jalankan(
                                                                sibuk,
                                                                galat,
                                                                info,
                                                                muat,
                                                                format!("Ukuran \"{n}\" dihapus."),
                                                                async move { hapus_ukuran(&g, &segmen(&n)).await },
                                                            );
                                                        }
                                                    }
                                                >
                                                    {move || {
                                                        if hapus_terminta.get().as_deref() == Some(nilai_tampil.as_str()) {
                                                            view! { <span class="px-1 text-xs">"Yakin?"</span> }.into_any()
                                                        } else {
                                                            view! { <AppIcon icon=TRASH size=14 /> }.into_any()
                                                        }
                                                    }}
                                                </button>
                                            </span>
                                            }
                                        })}
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ol>
                }
                    .into_any()
            }}
        </SectionCard>
    }
    .into_any()
}
