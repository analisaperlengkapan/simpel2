//! Pengisian ukuran pakaian dinas satu satker.
//!
//! Operator membuka pengajuannya, melihat daftar pegawai satkernya sendiri,
//! menekan "Isi Ukuran" pada satu baris, mengisi di jendela kecil, menyimpan,
//! dan kembali ke daftar yang sudah diperbarui. Begitu seterusnya per pegawai,
//! lalu "Ajukan" sekali di akhir.
//!
//! Dua hal yang membentuk rancangan ini:
//!
//! - **Kolomnya milik kampanye, bukan ditulis mati.** Pengajuan memilih
//!   spesifikasi mana yang diminta, jadi kampanye PDH memunculkan kolom
//!   Pakaian Dinas / Celana / Sepatu Dinas, dan kampanye lain memunculkan
//!   miliknya sendiri. Tiga kolom baju/celana/sepatu yang dipatok di kode akan
//!   salah pada kampanye pertama yang tidak berbentuk begitu.
//! - **Simpan per pegawai, bukan satu kiriman raksasa di akhir.** Satker besar
//!   berisi ratusan orang; kehilangan seluruhnya karena satu kegagalan di
//!   langkah terakhir adalah kerugian yang tidak perlu. simpelv1 menyimpan
//!   semuanya di hidden input lalu satu tombol Simpan — bagian itu tidak
//!   diikuti.

use std::collections::HashMap;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_fetch::QueryClient;
use leptos_router::hooks::use_params_map;

use crate::api::pakaian_dinas::{
    PengajuanPakaianItem, RosterPegawai, RosterPengisian, SimpanUkuranPegawaiRequest, Ukuran,
    UkuranPegawaiItem, ValidatorActionRequest, fetch_master_ukuran, fetch_roster_pengisian,
    keluarkan_pegawai_dari_pengajuan, process_validator_action, simpan_ukuran_pegawai,
};
use crate::components::layout::{ErrorState, LoadingState, PageLayout, SectionCard};
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{
    ARROW_LEFT, CHECK_CIRCLE, PAPER_PLANE_TILT, PENCIL_SIMPLE, WARNING_CIRCLE, X,
};

const FIELD: &str = "focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] \
                     px-3 py-2 text-sm text-slate-100";

async fn query_roster(key: (String, String, i32)) -> Result<RosterPengisian, crate::api::AppError> {
    let (pengajuan_id, satker_code, _trigger) = key;
    fetch_roster_pengisian(pengajuan_id, satker_code).await
}

/// Seluruh master ukuran sekali, lalu dikelompokkan per grup di memori. Daftar
/// ini kecil (puluhan baris) dan sama untuk setiap baris tabel, jadi satu
/// permintaan mengalahkan satu-per-kolom.
async fn query_ukuran(_key: ()) -> Result<Vec<Ukuran>, crate::api::AppError> {
    fetch_master_ukuran(None).await.map(|r| r.data)
}

fn kelompokkan(ukuran: &[Ukuran]) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<(i32, String)>> = HashMap::new();
    for u in ukuran {
        map.entry(u.group.to_uppercase())
            .or_default()
            .push((u.urutan, u.ukuran.clone()));
    }
    map.into_iter()
        .map(|(k, mut v)| {
            v.sort_by_key(|(urutan, _)| *urutan);
            (k, v.into_iter().map(|(_, u)| u).collect())
        })
        .collect()
}

#[component]
pub fn PakaianDinasPengisian() -> impl IntoView {
    let params = use_params_map();
    let pengajuan_id =
        move || params.with(|p| p.get("pengajuan_id").unwrap_or_default().to_string());
    let satker_code = move || params.with(|p| p.get("satker_code").unwrap_or_default().to_string());

    let refresh = RwSignal::new(0);
    let client: QueryClient = expect_context();
    let roster = client.local_resource(query_roster, move || {
        (pengajuan_id(), satker_code(), refresh.get())
    });
    let ukuran = client.local_resource(query_ukuran, || ());

    // Baris yang sedang diedit. `None` berarti tidak ada jendela terbuka.
    let sedang_diisi = RwSignal::new(Option::<RosterPegawai>::None);
    let pesan = RwSignal::new(Option::<String>::None);
    let sibuk = RwSignal::new(false);

    let ajukan = Callback::new(move |pengajuan_satker_id: String| {
        sibuk.set(true);
        pesan.set(None);
        spawn_local(async move {
            let req = ValidatorActionRequest {
                pengajuan_satker_id,
                aksi: "submit".to_string(),
                komentar: None,
            };
            match process_validator_action(req).await {
                Ok(_) => refresh.update(|v| *v += 1),
                Err(e) => pesan.set(Some(e.user_message())),
            }
            sibuk.set(false);
        });
    });

    view! {
        <PageLayout
            title="Pengisian Ukuran Pakaian Dinas"
            icon="fas fa-tshirt"
            description="Pilih pegawai, isi ukurannya, simpan. Ajukan setelah seluruhnya sesuai."
        >
            <Suspense fallback=move || {
                view! { <LoadingState /> }
            }>
                {move || {
                    let (Some(hasil), Some(daftar_ukuran)) = (roster.get(), ukuran.get()) else {
                        return view! { <LoadingState /> }.into_any();
                    };
                    let data = match hasil {
                        Ok(d) => d,
                        Err(e) => return view! { <ErrorState error=e /> }.into_any(),
                    };
                    let grup = kelompokkan(&daftar_ukuran.unwrap_or_default());
                    halaman(data, pengajuan_id(), grup, sedang_diisi, pesan, sibuk, refresh, ajukan)
                }}
            </Suspense>

            {move || {
                sedang_diisi
                    .get()
                    .map(|pegawai| {
                        let daftar_ukuran = ukuran.get().and_then(|r| r.ok()).unwrap_or_default();
                        let data = roster.get().and_then(|r| r.ok());
                        data.map(|d| {
                            view! {
                                <DialogUkuran
                                    pegawai=pegawai.clone()
                                    pakaian=d.pakaian.clone()
                                    grup=kelompokkan(&daftar_ukuran)
                                    pengajuan_id=d.pengajuan_satker_id.clone()
                                    kode_pengajuan=params
                                        .with(|p| {
                                            p.get("pengajuan_id").unwrap_or_default().to_string()
                                        })
                                    satker_code=d.satker_kode.clone()
                                    tutup=Callback::new(move |_| sedang_diisi.set(None))
                                    tersimpan=Callback::new(move |_| {
                                        sedang_diisi.set(None);
                                        refresh.update(|v| *v += 1);
                                    })
                                />
                            }
                        })
                    })
            }}
        </PageLayout>
    }
}

#[allow(clippy::too_many_arguments)]
fn halaman(
    data: RosterPengisian,
    // `pengajuan_pakaian_dinas.id` dari URL — respons membawa id BARIS satker,
    // bukan id kampanyenya, jadi tautan kembali mengambilnya dari rute.
    pengajuan_id_url: String,
    grup: HashMap<String, Vec<String>>,
    sedang_diisi: RwSignal<Option<RosterPegawai>>,
    pesan: RwSignal<Option<String>>,
    sibuk: RwSignal<bool>,
    refresh: RwSignal<i32>,
    ajukan: Callback<String>,
) -> AnyView {
    let _ = grup;
    let total = data.pegawai.len();
    let terisi = data.pegawai.iter().filter(|p| p.sudah_diisi).count();
    let dapat_diubah = data.dapat_diubah;
    let pengajuan_satker_id = data.pengajuan_satker_id.clone();
    let kode_pengajuan = data.pengajuan_satker_id.clone();
    let _ = kode_pengajuan;
    let satker_kode = data.satker_kode.clone();
    let kode_tampil = satker_kode.clone();
    let kode_fallback = satker_kode.clone();
    let pakaian = data.pakaian.clone();

    view! {
        <SectionCard title=data.pengajuan_nama.clone()>
            <a
                href=crate::routes::url::pakaian_satker_list(&pengajuan_id_url)
                class="mb-4 inline-flex items-center gap-2 text-sm text-slate-400 transition hover:text-slate-200"
            >
                <AppIcon icon=ARROW_LEFT size=14 />
                "Kembali ke daftar satker"
            </a>
            <div class="flex flex-wrap items-center justify-between gap-4">
                <div>
                    <p class="text-sm text-slate-300">
                        {data.satker_nama.clone().unwrap_or(kode_fallback)}
                        <span class="ml-2 text-slate-500">{kode_tampil}</span>
                    </p>
                    <p class="mt-1 text-sm text-slate-400">
                        <span class="font-semibold text-slate-200">{terisi}</span>
                        " dari "
                        {total}
                        " pegawai sudah diisi"
                    </p>
                </div>
                <Show when=move || dapat_diubah>
                    <button
                        type="button"
                        data-testid="ajukan-pengisian"
                        class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-5 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90 disabled:opacity-50"
                        prop:disabled=move || sibuk.get() || terisi == 0
                        title=move || {
                            if terisi == 0 {
                                "Isi ukuran minimal satu pegawai sebelum mengajukan"
                            } else {
                                "Ajukan ke Validator Wilayah"
                            }
                        }
                        on:click={
                            let id = pengajuan_satker_id.clone();
                            move |_| ajukan.run(id.clone())
                        }
                    >
                        <AppIcon icon=PAPER_PLANE_TILT />
                        "Ajukan ke Validator Wilayah"
                    </button>
                </Show>
            </div>

            <Show when=move || !dapat_diubah>
                <p
                    data-testid="pengisian-terkunci"
                    class="mt-4 flex items-start gap-2 rounded-lg border border-warning-500/20 bg-warning-500/10 px-3 py-2 text-sm text-warning-300"
                >
                    <AppIcon icon=WARNING_CIRCLE />
                    "Pengajuan ini sudah diajukan, sehingga ukurannya tidak dapat diubah lagi. Isinya ditampilkan apa adanya."
                </p>
            </Show>

            <Show when=move || pesan.get().is_some()>
                <p class="mt-4 rounded-lg border border-danger-500/20 bg-danger-500/10 px-3 py-2 text-sm text-danger-300">
                    {move || pesan.get()}
                </p>
            </Show>
        </SectionCard>

        <SectionCard title="Daftar Pegawai">
            {if pakaian.is_empty() {
                view! {
                    <p class="rounded-lg border border-warning-500/20 bg-warning-500/10 px-3 py-2 text-sm text-warning-300">
                        "Pengajuan ini belum memuat satu pun jenis pakaian, jadi tidak ada ukuran yang bisa diisi. Hubungi pengelola pusat."
                    </p>
                }
                    .into_any()
            } else {
                tabel(
                    data.pegawai.clone(),
                    pakaian.clone(),
                    dapat_diubah,
                    sedang_diisi,
                    satker_kode.clone(),
                    refresh,
                )
            }}
        </SectionCard>
    }
    .into_any()
}

fn tabel(
    pegawai: Vec<RosterPegawai>,
    pakaian: Vec<PengajuanPakaianItem>,
    dapat_diubah: bool,
    sedang_diisi: RwSignal<Option<RosterPegawai>>,
    satker_kode: String,
    refresh: RwSignal<i32>,
) -> AnyView {
    let _ = (satker_kode, refresh);
    let kolom = pakaian.clone();
    view! {
        <div class="overflow-hidden rounded-2xl border border-white/[0.06] bg-surface-panel">
            <div class="overflow-x-auto">
                <table
                    data-testid="pengisian-tabel"
                    class="min-w-full divide-y divide-white/[0.04]"
                >
                    <thead class="bg-white/[0.02]">
                        <tr>
                            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">
                                "Nama / NIP"
                            </th>
                            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">
                                "Jabatan / Pangkat"
                            </th>
                            {kolom
                                .iter()
                                .map(|p| {
                                    view! {
                                        <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">
                                            {p.judul()}
                                        </th>
                                    }
                                })
                                .collect_view()}
                            <th class="px-4 py-3 text-right text-xs font-semibold uppercase tracking-wide text-slate-400">
                                "Aksi"
                            </th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-white/[0.04]">
                        {pegawai
                            .into_iter()
                            .map(|p| baris(p, pakaian.clone(), dapat_diubah, sedang_diisi))
                            .collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
    .into_any()
}

fn baris(
    p: RosterPegawai,
    pakaian: Vec<PengajuanPakaianItem>,
    dapat_diubah: bool,
    sedang_diisi: RwSignal<Option<RosterPegawai>>,
) -> AnyView {
    let untuk_dialog = p.clone();
    view! {
        <tr class="transition hover:bg-white/[0.02]">
            <td class="px-4 py-3">
                <p class="flex items-center gap-2 text-sm font-medium text-slate-100">
                    {p
                        .sudah_diisi
                        .then(|| {
                            view! {
                                <span class="text-success-400" title="Sudah diisi">
                                    <AppIcon icon=CHECK_CIRCLE size=14 />
                                </span>
                            }
                        })} {p.nama.clone()}
                </p>
                <p class="text-xs text-slate-500">{p.nip.clone()}</p>
            </td>
            <td class="px-4 py-3">
                <p class="text-sm text-slate-300">
                    {p.jabatan.clone().unwrap_or_else(|| "-".to_string())}
                </p>
                <p class="text-xs text-slate-500">
                    {p.pangkat.clone().unwrap_or_else(|| "-".to_string())}
                </p>
            </td>
            {pakaian
                .iter()
                .map(|item| {
                    let berlaku = item.berlaku_untuk(&p.jenis_kelamin);
                    let nilai = p.ukuran_untuk(&item.id).map(|s| s.to_string());
                    // Kolom yang tidak berlaku bagi gender pegawai ini ditandai
                    // jelas, bukan dibiarkan kosong seperti ukuran yang belum
                    // diisi — dua keadaan yang berbeda.
                    view! {
                        <td class="px-4 py-3 text-sm">
                            {if !berlaku {
                                view! {
                                    <span
                                        class="text-xs text-slate-600"
                                        title="Tidak berlaku untuk pegawai ini"
                                    >
                                        "—"
                                    </span>
                                }
                                    .into_any()
                            } else {
                                match nilai {
                                    Some(v) => {
                                        view! { <span class="text-slate-100">{v}</span> }.into_any()
                                    }
                                    None => {
                                        view! {
                                            <span class="text-xs text-slate-500 italic">
                                                "belum diisi"
                                            </span>
                                        }
                                            .into_any()
                                    }
                                }
                            }}
                        </td>
                    }
                })
                .collect_view()}
            <td class="px-4 py-3 text-right">
                <button
                    type="button"
                    aria-label=format!("Isi ukuran {}", p.nama)
                    class="inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-200 transition hover:bg-white/[0.08] disabled:opacity-40"
                    prop:disabled=!dapat_diubah
                    on:click=move |_| sedang_diisi.set(Some(untuk_dialog.clone()))
                >
                    <AppIcon icon=PENCIL_SIMPLE size=14 />
                    {if p.sudah_diisi { "Ubah Ukuran" } else { "Isi Ukuran" }}
                </button>
            </td>
        </tr>
    }
    .into_any()
}

/// Jendela kecil pengisian ukuran satu pegawai.
#[component]
fn DialogUkuran(
    pegawai: RosterPegawai,
    pakaian: Vec<PengajuanPakaianItem>,
    grup: HashMap<String, Vec<String>>,
    /// `pengajuan_pakaian_dinas_satker.id` — dipakai hanya untuk pembeda DOM.
    pengajuan_id: String,
    /// `pengajuan_pakaian_dinas.id` pada URL.
    kode_pengajuan: String,
    satker_code: String,
    tutup: Callback<()>,
    tersimpan: Callback<()>,
) -> impl IntoView {
    let _ = pengajuan_id;
    let nip = StoredValue::new(pegawai.nip.clone());
    let kode_pengajuan = StoredValue::new(kode_pengajuan);
    let satker_code = StoredValue::new(satker_code);

    // Kolom yang berlaku bagi pegawai ini saja — operator tidak diminta mengisi
    // ukuran yang memang tidak berlaku baginya.
    let berlaku: Vec<PengajuanPakaianItem> = pakaian
        .into_iter()
        .filter(|item| item.berlaku_untuk(&pegawai.jenis_kelamin))
        .collect();

    let pilihan = RwSignal::new(
        berlaku
            .iter()
            .map(|item| {
                (
                    item.id.clone(),
                    pegawai.ukuran_untuk(&item.id).unwrap_or("").to_string(),
                )
            })
            .collect::<HashMap<String, String>>(),
    );
    let hijab = RwSignal::new(pegawai.with_hijab);
    let sibuk = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    let simpan = move |_| {
        sibuk.set(true);
        error.set(None);
        let request = SimpanUkuranPegawaiRequest {
            with_hijab: hijab.get(),
            ukuran: pilihan
                .get()
                .into_iter()
                .filter(|(_, v)| !v.trim().is_empty())
                .map(|(pakaian_id, ukuran)| UkuranPegawaiItem { pakaian_id, ukuran })
                .collect(),
        };
        spawn_local(async move {
            match simpan_ukuran_pegawai(
                kode_pengajuan.get_value(),
                satker_code.get_value(),
                nip.get_value(),
                request,
            )
            .await
            {
                Ok(_) => tersimpan.run(()),
                Err(e) => error.set(Some(e.user_message())),
            }
            sibuk.set(false);
        });
    };

    let keluarkan = move |_| {
        sibuk.set(true);
        error.set(None);
        spawn_local(async move {
            match keluarkan_pegawai_dari_pengajuan(
                kode_pengajuan.get_value(),
                satker_code.get_value(),
                nip.get_value(),
            )
            .await
            {
                Ok(_) => tersimpan.run(()),
                Err(e) => error.set(Some(e.user_message())),
            }
            sibuk.set(false);
        });
    };

    let nama = pegawai.nama.clone();
    let nip_tampil = pegawai.nip.clone();
    let jabatan = pegawai.jabatan.clone().unwrap_or_else(|| "-".to_string());
    let pangkat = pegawai.pangkat.clone().unwrap_or_else(|| "-".to_string());
    let perempuan = pegawai.jenis_kelamin.eq_ignore_ascii_case("P");
    let sudah_diisi = pegawai.sudah_diisi;

    view! {
        <div
            data-testid="dialog-ukuran"
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
            role="dialog"
            aria-modal="true"
        >
            <div class="max-h-[90vh] w-full max-w-lg overflow-y-auto rounded-2xl border border-white/10 bg-surface-panel p-6 shadow-xl">
                <div class="flex items-start justify-between gap-4">
                    <div>
                        <h3 class="text-lg font-semibold text-slate-100">{nama}</h3>
                        <p class="text-xs text-slate-500">{nip_tampil}</p>
                        <p class="mt-1 text-sm text-slate-400">{jabatan} " · " {pangkat}</p>
                    </div>
                    <button
                        type="button"
                        aria-label="Tutup"
                        class="text-slate-400 transition hover:text-slate-200"
                        on:click=move |_| tutup.run(())
                    >
                        <AppIcon icon=X />
                    </button>
                </div>

                <div class="mt-5 flex flex-col gap-4">
                    {berlaku
                        .into_iter()
                        .map(|item| {
                            let id_select = format!("ukuran-{}", item.id);
                            let pakaian_id = item.id.clone();
                            let opsi = grup
                                .get(&item.spesifikasi_ukuran_group.to_uppercase())
                                .cloned()
                                .unwrap_or_default();
                            let judul = item.judul();
                            view! {
                                <div class="flex flex-col gap-1.5">
                                    <label
                                        class="text-xs font-semibold uppercase tracking-wide text-slate-300"
                                        for=id_select.clone()
                                    >
                                        {judul}
                                    </label>
                                    <select
                                        id=id_select
                                        class=FIELD
                                        prop:value={
                                            let pakaian_id = pakaian_id.clone();
                                            move || {
                                                pilihan.get().get(&pakaian_id).cloned().unwrap_or_default()
                                            }
                                        }
                                        on:change={
                                            let pakaian_id = pakaian_id.clone();
                                            move |ev| {
                                                let v = event_target_value(&ev);
                                                pilihan
                                                    .update(|m| {
                                                        m.insert(pakaian_id.clone(), v);
                                                    });
                                            }
                                        }
                                    >
                                        <option value="">"— Pilih Ukuran —"</option>
                                        {opsi
                                            .into_iter()
                                            .map(|u| {
                                                view! { <option value=u.clone()>{u.clone()}</option> }
                                            })
                                            .collect_view()}
                                    </select>
                                // Daftar ukurannya datang dari master
                                // `ms_ukuran`, jadi tidak ada panduan yang
                                // ditulis mati untuk ikut basi.
                                </div>
                            }
                        })
                        .collect_view()}
                    <Show when=move || perempuan>
                        <label class="flex cursor-pointer items-center gap-2 text-sm text-slate-200">
                            <input
                                type="checkbox"
                                class="h-4 w-4"
                                prop:checked=move || hijab.get()
                                on:change=move |ev| hijab.set(event_target_checked(&ev))
                            />
                            "Pakaian muslimah (berhijab)"
                        </label>
                    </Show>
                </div>

                <Show when=move || error.get().is_some()>
                    <p class="mt-4 rounded-lg border border-danger-500/20 bg-danger-500/10 px-3 py-2 text-sm text-danger-300">
                        {move || error.get()}
                    </p>
                </Show>

                <div class="mt-6 flex items-center justify-between gap-3 border-t border-white/[0.06] pt-4">
                    <Show when=move || sudah_diisi>
                        <button
                            type="button"
                            class="rounded-lg border border-danger-500/30 px-4 py-2 text-sm text-danger-300 transition hover:bg-danger-500/10 disabled:opacity-50"
                            prop:disabled=move || sibuk.get()
                            on:click=keluarkan
                            title="Untuk pegawai yang tidak memerlukan pakaian dinas"
                        >
                            "Keluarkan dari Pengajuan"
                        </button>
                    </Show>
                    <div class="ml-auto flex gap-3">
                        <button
                            type="button"
                            class="rounded-lg border border-white/10 bg-white/[0.04] px-4 py-2 text-sm text-slate-300 transition hover:bg-white/[0.08]"
                            on:click=move |_| tutup.run(())
                        >
                            "Batal"
                        </button>
                        <button
                            type="button"
                            data-testid="simpan-ukuran"
                            class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-5 py-2 text-sm font-bold text-navy-950 transition hover:opacity-90 disabled:opacity-50"
                            prop:disabled=move || sibuk.get()
                            on:click=simpan
                        >
                            {move || if sibuk.get() { "Menyimpan…" } else { "Simpan" }}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
