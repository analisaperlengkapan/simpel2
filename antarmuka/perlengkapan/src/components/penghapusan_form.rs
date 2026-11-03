use crate::api::bank_aset::BankAsetItem;
use crate::api::{
    CreatePenghapusanBmnItemRequest, CreatePenghapusanBmnWorkflowRequest,
    create_penghapusan_bmn_workflow,
};
use crate::components::reference_picker::AsetPicker;
use crate::routes;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;
use lib_perlengkapan::utils::format_rupiah;
use lib_ui::components::icon::AppIcon;
use lib_ui::hooks::{use_form, use_toast::use_toast};
use phosphor_leptos::{FLOPPY_DISK, SPINNER, X};

/// Form data for BMN disposal request.
#[derive(Clone, Default)]
struct PenghapusanFormData {
    kode_barang: String,
    nup: String,
    tanggal: String,
    alasan: String,
    metode: String,
    nilai_perolehan: String,
    lampiran_persyaratan: String,
    catatan_operator: String,
}

/// Item BMN tambahan (Fase 2.8) — di luar item utama pada field form.
#[derive(Clone, Default)]
struct ExtraItem {
    kode_barang: String,
    nup: String,
    /// Shown, never sent: SIMAN's figure for the picked asset.
    nilai_perolehan: Option<f64>,
}

/// Satu baris item tambahan.
///
/// Isi baris tinggal di sinyalnya sendiri dan daftar hanya menyimpan kuncinya.
/// Sebelumnya barisnya dikenali dari indeks posisi: tiap `on:input` menangkap
/// `i` lalu menulis ke `v[i]`, sehingga menghapus satu baris menggeser makna
/// setiap penutup di bawahnya. Di sini kotak-kotaknya punya `prop:value`,
/// jadi tampilannya masih ikut terkoreksi — pada formulir pemakaian, yang
/// tidak punya `prop:value`, kekeliruan yang sama membuat layar dan muatan
/// berbeda. Dengan `For` berkunci, identitas melekat pada barisnya.
#[derive(Clone, Copy)]
struct BarisItemTambahan {
    kunci: usize,
    isi: RwSignal<ExtraItem>,
}

#[component]
pub fn PenghapusanForm() -> impl IntoView {
    let form = use_form(PenghapusanFormData::default());
    let toast = use_toast();
    let navigate = use_navigate();
    // Fase 2.8: item BMN tambahan (multi-item). Item utama = field form di atas.
    let baris_tambahan = RwSignal::new(Vec::<BarisItemTambahan>::new());
    // Kunci dihitung maju dan tak pernah dipakai ulang: indeks posisi akan
    // menggeser identitas baris begitu baris di atasnya dihapus, menukar isi
    // dua pemilih.
    let kunci_berikutnya = StoredValue::new(0usize);

    // Picking the asset is the whole of naming it. The fields below are the
    // picked row, kept for display only: they are not editable, and the request
    // carries kode barang + NUP and nothing else about the asset. Its name and
    // acquisition value are SIMAN's, and the server reads both from SIMAN for
    // every asset on the usulan — what this form held in them was never used.
    let on_pick_aset = Callback::new(move |picked: Option<BankAsetItem>| {
        let Some(item) = picked else {
            form.update(|f| {
                f.kode_barang.clear();
                f.nup.clear();
                f.nilai_perolehan.clear();
            });
            return;
        };
        form.update(|f| {
            f.kode_barang = item.kode_barang.clone().unwrap_or_default();
            f.nup = item.nup.clone().unwrap_or_default();
            // Blank rather than "0" when SIMAN has no figure: a zero here would
            // travel into the SK as the asset's acquisition value.
            f.nilai_perolehan = item
                .nilai_perolehan
                .map(|v| v.to_string())
                .unwrap_or_default();
        });
    });

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        // The picker is the only way to name an asset, so an empty identity
        // means nothing was picked. The three text boxes this replaced carried
        // `required`; with them gone the browser no longer says so.
        let utama = form.get();
        if utama.kode_barang.trim().is_empty() || utama.nup.trim().is_empty() {
            let msg = "Pilih BMN yang diusulkan terlebih dahulu";
            form.finish_err(msg);
            toast.error(msg);
            return;
        }
        // An extra row nobody picked into used to be dropped without a word,
        // which reads as "this asset is on the SK" when it is not.
        let kosong = baris_tambahan
            .get_untracked()
            .iter()
            .position(|b| b.isi.get_untracked().kode_barang.trim().is_empty());
        if let Some(idx) = kosong {
            let msg = format!(
                "Item tambahan ke-{} belum memilih BMN. Pilih BMN-nya atau hapus barisnya.",
                idx + 1
            );
            form.finish_err(msg.clone());
            toast.error(msg);
            return;
        }
        let data = form.begin_submit();

        // Item utama + item tambahan, identitasnya saja.
        let mut items: Vec<CreatePenghapusanBmnItemRequest> =
            vec![CreatePenghapusanBmnItemRequest {
                kode_barang: data.kode_barang.clone(),
                nup: data.nup.clone(),
                kondisi: None,
            }];
        for baris in baris_tambahan.get_untracked() {
            let e = baris.isi.get_untracked();
            items.push(CreatePenghapusanBmnItemRequest {
                kode_barang: e.kode_barang,
                nup: e.nup,
                kondisi: None,
            });
        }

        let req = CreatePenghapusanBmnWorkflowRequest {
            kode_barang: data.kode_barang,
            nup: data.nup,
            tanggal_penghapusan: data.tanggal,
            alasan: data.alasan,
            metode_penghapusan: data.metode,
            lampiran_persyaratan: data.lampiran_persyaratan,
            catatan_operator: if data.catatan_operator.is_empty() {
                None
            } else {
                Some(data.catatan_operator)
            },
            items,
        };

        let navigate = navigate.clone();
        spawn_local(async move {
            match create_penghapusan_bmn_workflow(req).await {
                Ok(resp) => {
                    // Show toast immediately, but keep `submitting=true` during
                    // the navigation delay to prevent double-submission. Only
                    // call `finish_ok()` after the timeout, right before we
                    // navigate away.
                    toast.success("Usulan penghapusan berhasil disimpan!");
                    gloo_timers::future::TimeoutFuture::new(1000).await;
                    form.finish_ok();
                    // Base-relative: navigate() prepends the router base.
                    navigate(
                        &format!("/pengelolaan/penghapusan/detail/{}", resp.data.id),
                        Default::default(),
                    );
                }
                Err(e) => {
                    let msg = format!("Gagal menyimpan: {:?}", e);
                    form.finish_err(msg.clone());
                    toast.error(msg);
                }
            }
        });
    };

    view! {
        <Title text="Usulan Penghapusan BMN — SIMPEL" />
        <div class="max-w-2xl mx-auto p-6 bg-surface-panel rounded-xl shadow-sm border border-white/[0.06]">
            <h2 class="text-xl font-bold text-slate-100 mb-6">"Usulan SK Penghapusan BMN"</h2>

            <form on:submit=on_submit class="space-y-4">
                // -- Identifikasi BMN --
                <h3 class="text-lg font-semibold text-slate-200 border-b pb-2">
                    "Identifikasi BMN"
                </h3>

                <AsetPicker on_pick=on_pick_aset />

                // -- Item BMN tambahan (Fase 2.8, multi-item) --
                <div class="mt-4 rounded-lg border border-white/[0.06] bg-white/[0.03] p-4">
                    <div class="flex items-center justify-between mb-2">
                        <h4 class="text-sm font-semibold text-slate-200">
                            "Item BMN Tambahan (opsional)"
                        </h4>
                        <button
                            type="button"
                            class="rounded-lg bg-blue-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-blue-700"
                            on:click=move |_| {
                                let kunci = kunci_berikutnya.get_value();
                                kunci_berikutnya.set_value(kunci + 1);
                                baris_tambahan
                                    .update(|daftar| {
                                        daftar
                                            .push(BarisItemTambahan {
                                                kunci,
                                                isi: RwSignal::new(ExtraItem::default()),
                                            });
                                    });
                            }
                        >
                            "+ Tambah Item"
                        </button>
                    </div>
                    <p class="mb-3 text-xs text-slate-500">
                        "Item utama diisi di atas. Tambahkan BMN lain bila satu usulan SK mencakup beberapa aset."
                    </p>

                    // Tiap baris memilih asetnya: cari, pilih, selesai. Identitas
                    // aset (kode barang + NUP) datang dari baris yang dipilih, dan
                    // hanya itu yang dikirim; nama dan nilai perolehannya milik
                    // SIMAN dan diisi server untuk SETIAP BMN pada usulan.
                    //
                    // Dulu nilai tiap item tambahan adalah kotak yang diketik
                    // operator, dengan alasan server hanya menimpa nilai item
                    // utama. Itu benar, dan itu justru lubangnya: angka yang
                    // diketik tersimpan apa adanya dan menentukan apakah usulan
                    // masih dalam kewenangan Wilayah. Sekarang hanya ditampilkan.
                    <Show
                        when=move || !baris_tambahan.get().is_empty()
                        fallback=|| {
                            view! {
                                <p class="text-xs text-slate-500 italic">"Belum ada item tambahan."</p>
                            }
                        }
                    >
                        <div class="space-y-3">
                            <For
                                each=move || baris_tambahan.get()
                                key=|baris| baris.kunci
                                children=move |baris| {
                                    let isi = baris.isi;
                                    let on_pick = Callback::new(move |picked: Option<BankAsetItem>| {
                                        isi.update(|item| {
                                            let Some(aset) = picked else {
                                                *item = ExtraItem::default();
                                                return;
                                            };
                                            item.kode_barang = aset
                                                .kode_barang
                                                .clone()
                                                .unwrap_or_default();
                                            item.nup = aset.nup.clone().unwrap_or_default();
                                            item.nilai_perolehan = aset.nilai_perolehan;
                                        });
                                    });
                                    let slug = format!("aset-tambahan-{}", baris.kunci);
                                    let label = "Cari BMN tambahan".to_string();
                                    let testid_nilai = format!("nilai-tambahan-{}", baris.kunci);
                                    view! {
                                        <div class="relative rounded-lg border border-white/10 bg-slate-900/40 p-3">
                                            <button
                                                type="button"
                                                aria-label="Hapus item tambahan"
                                                class="absolute top-2 right-2 z-10 text-danger-400 hover:text-red-700"
                                                on:click=move |_| {
                                                    baris_tambahan
                                                        .update(|daftar| {
                                                            daftar.retain(|lain| lain.kunci != baris.kunci)
                                                        })
                                                }
                                            >
                                                <AppIcon icon=X />
                                            </button>
                                            <AsetPicker slug=slug label=label on_pick=on_pick />
                                            <Show when=move || !isi.get().kode_barang.is_empty()>
                                                <p
                                                    data-testid=testid_nilai.clone()
                                                    class="mt-3 text-xs text-slate-400"
                                                >
                                                    "Nilai perolehan (dari SIMAN): "
                                                    {move || match isi.get().nilai_perolehan {
                                                        Some(v) => format_rupiah(v),
                                                        None => "belum tercatat di SIMAN, usulan akan ditolak".to_string(),
                                                    }}
                                                </p>
                                            </Show>
                                        </div>
                                    }
                                }
                            />
                        </div>
                    </Show>
                </div>

                // -- Detail Penghapusan --
                <h3 class="text-lg font-semibold text-slate-200 border-b pb-2 mt-6">
                    "Detail Penghapusan"
                </h3>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-slate-200 mb-1" for="tanggal">
                            "Tanggal Penghapusan"
                        </label>
                        <input
                            id="tanggal"
                            type="date"
                            class="w-full px-4 py-2 rounded-lg border border-white/10 bg-slate-900/70 text-slate-100 placeholder:text-slate-500 outline-none transition-colors hover:border-white/20 focus:border-gold-400/60 focus:ring-2 focus:ring-gold-400/40"
                            prop:value=move || form.get().tanggal.clone()
                            on:input=move |ev| form.update(|f| f.tanggal = event_target_value(&ev))
                            required
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-slate-200 mb-1" for="metode">
                            "Metode Penghapusan"
                        </label>
                        <select
                            id="metode"
                            class="w-full px-4 py-2 rounded-lg border border-white/10 bg-slate-900/70 text-slate-100 placeholder:text-slate-500 outline-none transition-colors hover:border-white/20 focus:border-gold-400/60 focus:ring-2 focus:ring-gold-400/40"
                            prop:value=move || form.get().metode.clone()
                            on:change=move |ev| form.update(|f| f.metode = event_target_value(&ev))
                            required
                        >
                            <option value="">"Pilih Metode"</option>
                            <option value="Lelang">"Lelang"</option>
                            <option value="Musnah">"Musnah"</option>
                            <option value="Hibah">"Hibah Keluar"</option>
                            <option value="Tukar Menukar">"Tukar Menukar"</option>
                        </select>
                    </div>
                </div>

                <div>
                    <label class="block text-sm font-medium text-slate-200 mb-1" for="alasan">
                        "Alasan Penghapusan"
                    </label>
                    <textarea
                        id="alasan"
                        class="w-full px-4 py-2 rounded-lg border border-white/10 bg-slate-900/70 text-slate-100 placeholder:text-slate-500 outline-none transition-colors hover:border-white/20 focus:border-gold-400/60 focus:ring-2 focus:ring-gold-400/40"
                        rows="3"
                        placeholder="Kondisi rusak berat, hilang, dsb."
                        prop:value=move || form.get().alasan.clone()
                        on:input=move |ev| form.update(|f| f.alasan = event_target_value(&ev))
                        required
                    ></textarea>
                </div>

                <div>
                    <label
                        class="block text-sm font-medium text-slate-200 mb-1"
                        for="nilai_perolehan"
                    >
                        "Nilai Perolehan (Rp)"
                    </label>
                    <input
                        id="nilai_perolehan"
                        type="number"
                        class="w-full cursor-not-allowed rounded-lg border border-white/[0.06] bg-white/[0.03] px-4 py-2 text-slate-400"
                        placeholder="Terisi setelah BMN dipilih"
                        prop:value=move || form.get().nilai_perolehan.clone()
                        readonly=true
                    />
                    <p class="mt-1 text-xs text-slate-500">
                        "Nama dan nilai perolehan setiap BMN diambil langsung dari data SIMAN berdasarkan kode barang + NUP. "
                        "Jika ada BMN yang tidak ditemukan di SIMAN atau belum bernilai, sistem menolak seluruh usulan."
                    </p>
                </div>

                // -- Lampiran & Catatan --
                <h3 class="text-lg font-semibold text-slate-200 border-b pb-2 mt-6">
                    "Lampiran & Catatan"
                </h3>

                <div>
                    <label class="block text-sm font-medium text-slate-200 mb-1" for="lampiran">
                        "Lampiran Persyaratan (URL)"
                    </label>
                    <input
                        id="lampiran"
                        type="text"
                        class="w-full px-4 py-2 rounded-lg border border-white/10 bg-slate-900/70 text-slate-100 placeholder:text-slate-500 outline-none transition-colors hover:border-white/20 focus:border-gold-400/60 focus:ring-2 focus:ring-gold-400/40"
                        placeholder="URL dokumen persyaratan penghapusan"
                        prop:value=move || form.get().lampiran_persyaratan.clone()
                        on:input=move |ev| {
                            form.update(|f| f.lampiran_persyaratan = event_target_value(&ev))
                        }
                        required
                    />
                    <p class="text-xs text-slate-500 mt-1">
                        "Upload dokumen persyaratan terlebih dahulu, kemudian tempel URL-nya di sini."
                    </p>
                </div>

                <div>
                    <label class="block text-sm font-medium text-slate-200 mb-1" for="catatan">
                        "Catatan Operator"
                    </label>
                    <textarea
                        id="catatan"
                        class="w-full px-4 py-2 rounded-lg border border-white/10 bg-slate-900/70 text-slate-100 placeholder:text-slate-500 outline-none transition-colors hover:border-white/20 focus:border-gold-400/60 focus:ring-2 focus:ring-gold-400/40"
                        rows="2"
                        placeholder="Catatan tambahan (opsional)"
                        prop:value=move || form.get().catatan_operator.clone()
                        on:input=move |ev| {
                            form.update(|f| f.catatan_operator = event_target_value(&ev))
                        }
                    ></textarea>
                </div>

                <div class="pt-4 flex justify-end gap-3">
                    <a
                        href=routes::path::PENGELOLAAN_PENGHAPUSAN
                        class="px-4 py-2 text-slate-200 bg-white/[0.05] rounded-lg hover:bg-white/[0.09] transition-colors"
                    >
                        "Batal"
                    </a>
                    <button
                        type="submit"
                        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 flex items-center gap-2"
                        prop:disabled=move || form.submitting.get()
                    >
                        <Show
                            when=move || form.submitting.get()
                            fallback=|| view! { <AppIcon icon=FLOPPY_DISK /> }
                        >
                            <span class="fa-spin">
                                <AppIcon icon=SPINNER />
                            </span>
                        </Show>
                        "Simpan Draft"
                    </button>
                </div>
            </form>
        </div>
    }
}
