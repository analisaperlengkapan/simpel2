use crate::api::{
    CreatePenghapusanBmnItemRequest, CreatePenghapusanBmnWorkflowRequest,
    create_penghapusan_bmn_workflow,
};
use crate::routes;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;
use lib_ui::components::icon::AppIcon;
use lib_ui::hooks::{use_form, use_toast::use_toast};
use phosphor_leptos::{FLOPPY_DISK, SPINNER};

/// Form data for BMN disposal request.
#[derive(Clone, Default)]
struct PenghapusanFormData {
    satker_id: String,
    asset_id: String,
    kode_barang: String,
    nama_barang: String,
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
    nama_barang: String,
    nup: String,
    nilai_perolehan: String,
}

#[component]
pub fn PenghapusanForm() -> impl IntoView {
    let form = use_form(PenghapusanFormData::default());
    let toast = use_toast();
    let navigate = use_navigate();
    // Fase 2.8: item BMN tambahan (multi-item). Item utama = field form di atas.
    let extras = RwSignal::new(Vec::<ExtraItem>::new());

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        let data = form.begin_submit();

        // Rakit daftar item: item utama (field tunggal) + item tambahan valid.
        let mut items: Vec<CreatePenghapusanBmnItemRequest> =
            vec![CreatePenghapusanBmnItemRequest {
                asset_id: if data.asset_id.is_empty() {
                    None
                } else {
                    Some(data.asset_id.clone())
                },
                kode_barang: data.kode_barang.clone(),
                nama_barang: data.nama_barang.clone(),
                nup: data.nup.clone(),
                nilai_perolehan: data.nilai_perolehan.parse::<f64>().ok(),
                kondisi: None,
            }];
        for e in extras.get_untracked() {
            if !e.kode_barang.trim().is_empty() && !e.nup.trim().is_empty() {
                items.push(CreatePenghapusanBmnItemRequest {
                    asset_id: None,
                    kode_barang: e.kode_barang,
                    nama_barang: e.nama_barang,
                    nup: e.nup,
                    nilai_perolehan: e.nilai_perolehan.parse::<f64>().ok(),
                    kondisi: None,
                });
            }
        }

        let req = CreatePenghapusanBmnWorkflowRequest {
            satker_id: data.satker_id,
            asset_id: data.asset_id,
            kode_barang: data.kode_barang,
            nama_barang: data.nama_barang,
            nup: data.nup,
            tanggal_penghapusan: data.tanggal,
            alasan: data.alasan,
            metode_penghapusan: data.metode,
            nilai_perolehan: data.nilai_perolehan.parse::<f64>().ok(),
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
                    navigate(
                        &format!("/perlengkapan/pengelolaan/penghapusan/{}", resp.data.id),
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
        <div class="max-w-2xl mx-auto p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <h2 class="text-xl font-bold text-gray-800 mb-6">"Usulan SK Penghapusan BMN"</h2>

            <form on:submit=on_submit class="space-y-4">
                // -- Identifikasi BMN --
                <h3 class="text-lg font-semibold text-gray-700 border-b pb-2">"Identifikasi BMN"</h3>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1" for="satker_id">"Satuan Kerja (Satker ID)"</label>
                        <input
                            id="satker_id"
                            type="text"
                            class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                            placeholder="ID Satuan Kerja"
                            prop:value=move || form.get().satker_id.clone()
                            on:input=move |ev| form.update(|f| f.satker_id = event_target_value(&ev))
                            required
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1" for="asset_id">"Asset ID (UUID)"</label>
                        <input
                            id="asset_id"
                            type="text"
                            class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                            placeholder="550e8400-e29b-41d4-a716-446655440000"
                            prop:value=move || form.get().asset_id.clone()
                            on:input=move |ev| form.update(|f| f.asset_id = event_target_value(&ev))
                            required
                        />
                    </div>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1" for="kode_barang">"Kode Barang"</label>
                        <input
                            id="kode_barang"
                            type="text"
                            class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                            placeholder="Kode barang BMN"
                            prop:value=move || form.get().kode_barang.clone()
                            on:input=move |ev| form.update(|f| f.kode_barang = event_target_value(&ev))
                            required
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1" for="nama_barang">"Nama Barang"</label>
                        <input
                            id="nama_barang"
                            type="text"
                            class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                            placeholder="Nama barang BMN"
                            prop:value=move || form.get().nama_barang.clone()
                            on:input=move |ev| form.update(|f| f.nama_barang = event_target_value(&ev))
                            required
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1" for="nup">"NUP"</label>
                        <input
                            id="nup"
                            type="text"
                            class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                            placeholder="Nomor Urut Pendaftaran"
                            prop:value=move || form.get().nup.clone()
                            on:input=move |ev| form.update(|f| f.nup = event_target_value(&ev))
                            required
                        />
                    </div>
                </div>

                // -- Item BMN tambahan (Fase 2.8, multi-item) --
                <div class="mt-4 rounded-lg border border-gray-200 bg-gray-50 p-4">
                    <div class="flex items-center justify-between mb-2">
                        <h4 class="text-sm font-semibold text-gray-700">"Item BMN Tambahan (opsional)"</h4>
                        <button
                            type="button"
                            class="rounded-lg bg-blue-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-blue-700"
                            on:click=move |_| extras.update(|v| v.push(ExtraItem::default()))
                        >
                            "+ Tambah Item"
                        </button>
                    </div>
                    <p class="mb-3 text-xs text-gray-500">
                        "Item utama diisi di atas. Tambahkan BMN lain bila satu usulan SK mencakup beberapa aset."
                    </p>
                    {move || {
                        let rows = extras.get();
                        if rows.is_empty() {
                            view! { <p class="text-xs text-gray-400 italic">"Belum ada item tambahan."</p> }.into_any()
                        } else {
                            view! {
                                <div class="space-y-2">
                                    {rows.into_iter().enumerate().map(|(i, item)| view! {
                                        <div class="grid grid-cols-1 md:grid-cols-12 gap-2 items-center">
                                            <input
                                                type="text" placeholder="Kode Barang"
                                                class="md:col-span-3 px-3 py-1.5 border rounded text-sm"
                                                prop:value=item.kode_barang.clone()
                                                on:input=move |ev| extras.update(|v| { if let Some(r) = v.get_mut(i) { r.kode_barang = event_target_value(&ev); } })
                                            />
                                            <input
                                                type="text" placeholder="Nama Barang"
                                                class="md:col-span-4 px-3 py-1.5 border rounded text-sm"
                                                prop:value=item.nama_barang.clone()
                                                on:input=move |ev| extras.update(|v| { if let Some(r) = v.get_mut(i) { r.nama_barang = event_target_value(&ev); } })
                                            />
                                            <input
                                                type="text" placeholder="NUP"
                                                class="md:col-span-2 px-3 py-1.5 border rounded text-sm"
                                                prop:value=item.nup.clone()
                                                on:input=move |ev| extras.update(|v| { if let Some(r) = v.get_mut(i) { r.nup = event_target_value(&ev); } })
                                            />
                                            <input
                                                type="number" placeholder="Nilai"
                                                class="md:col-span-2 px-3 py-1.5 border rounded text-sm"
                                                prop:value=item.nilai_perolehan.clone()
                                                on:input=move |ev| extras.update(|v| { if let Some(r) = v.get_mut(i) { r.nilai_perolehan = event_target_value(&ev); } })
                                            />
                                            <button
                                                type="button"
                                                class="md:col-span-1 rounded bg-red-100 px-2 py-1.5 text-xs text-red-700 hover:bg-red-200"
                                                on:click=move |_| extras.update(|v| { if i < v.len() { v.remove(i); } })
                                            >
                                                "Hapus"
                                            </button>
                                        </div>
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                    }}
                </div>

                // -- Detail Penghapusan --
                <h3 class="text-lg font-semibold text-gray-700 border-b pb-2 mt-6">"Detail Penghapusan"</h3>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1" for="tanggal">"Tanggal Penghapusan"</label>
                        <input
                            id="tanggal"
                            type="date"
                            class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                            prop:value=move || form.get().tanggal.clone()
                            on:input=move |ev| form.update(|f| f.tanggal = event_target_value(&ev))
                            required
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1" for="metode">"Metode Penghapusan"</label>
                        <select
                            id="metode"
                            class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
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
                    <label class="block text-sm font-medium text-gray-700 mb-1" for="alasan">"Alasan Penghapusan"</label>
                    <textarea
                        id="alasan"
                        class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                        rows="3"
                        placeholder="Kondisi rusak berat, hilang, dsb."
                        prop:value=move || form.get().alasan.clone()
                        on:input=move |ev| form.update(|f| f.alasan = event_target_value(&ev))
                        required
                    ></textarea>
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1" for="nilai_perolehan">"Nilai Perolehan (Rp)"</label>
                    <input
                        id="nilai_perolehan"
                        type="number"
                        class="w-full px-4 py-2 border rounded-lg bg-gray-50 text-gray-600 cursor-not-allowed"
                        placeholder="Diambil otomatis dari SIMAN saat submit"
                        prop:value=move || form.get().nilai_perolehan.clone()
                        readonly=true
                    />
                    <p class="mt-1 text-xs text-gray-500">
                        "Nilai perolehan diambil langsung dari data SIMAN berdasarkan NUP + kode barang. "
                        "Jika BMN tidak ditemukan di SIMAN, sistem akan menolak usulan."
                    </p>
                </div>

                // -- Lampiran & Catatan --
                <h3 class="text-lg font-semibold text-gray-700 border-b pb-2 mt-6">"Lampiran & Catatan"</h3>

                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1" for="lampiran">"Lampiran Persyaratan (URL)"</label>
                    <input
                        id="lampiran"
                        type="text"
                        class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                        placeholder="URL dokumen persyaratan penghapusan"
                        prop:value=move || form.get().lampiran_persyaratan.clone()
                        on:input=move |ev| form.update(|f| f.lampiran_persyaratan = event_target_value(&ev))
                        required
                    />
                    <p class="text-xs text-gray-500 mt-1">"Upload dokumen persyaratan terlebih dahulu, kemudian tempel URL-nya di sini."</p>
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1" for="catatan">"Catatan Operator"</label>
                    <textarea
                        id="catatan"
                        class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                        rows="2"
                        placeholder="Catatan tambahan (opsional)"
                        prop:value=move || form.get().catatan_operator.clone()
                        on:input=move |ev| form.update(|f| f.catatan_operator = event_target_value(&ev))
                    ></textarea>
                </div>

                <div class="pt-4 flex justify-end gap-3">
                    <a
                        href=routes::path::PENGELOLAAN_PENGHAPUSAN_DAFTAR_LEGACY
                        class="px-4 py-2 text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition-colors"
                    >
                        "Batal"
                    </a>
                    <button
                        type="submit"
                        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 flex items-center gap-2"
                        prop:disabled=move || form.submitting.get()
                    >
                        <Show when=move || form.submitting.get() fallback=|| view! { <AppIcon icon=FLOPPY_DISK /> }>
                            <span class="fa-spin"><AppIcon icon=SPINNER /></span>
                        </Show>
                        "Simpan Draft"
                    </button>
                </div>
            </form>
        </div>
    }
}
