use crate::api::{CreatePenghapusanBmnWorkflowRequest, create_penghapusan_bmn_workflow};
use crate::routes;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;
use lib_ui::hooks::{use_form, use_toast::use_toast};

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
    residu: String,
    lampiran_persyaratan: String,
    catatan_operator: String,
}

#[component]
pub fn PenghapusanForm() -> impl IntoView {
    let form = use_form(PenghapusanFormData::default());
    let toast = use_toast();
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        let data = form.begin_submit();

        let req = CreatePenghapusanBmnWorkflowRequest {
            satker_id: data.satker_id,
            asset_id: data.asset_id,
            kode_barang: data.kode_barang,
            nama_barang: data.nama_barang,
            nup: data.nup,
            tanggal_penghapusan: data.tanggal,
            alasan: data.alasan,
            metode_penghapusan: data.metode,
            nilai_residu: data.residu.parse::<f64>().ok(),
            lampiran_persyaratan: data.lampiran_persyaratan,
            catatan_operator: if data.catatan_operator.is_empty() {
                None
            } else {
                Some(data.catatan_operator)
            },
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
                    form.finish_err(format!("Gagal menyimpan: {:?}", e));
                    toast.error(form.error.get_untracked().unwrap_or_default());
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
                    <label class="block text-sm font-medium text-gray-700 mb-1" for="residu">"Nilai Residu (Rp)"</label>
                    <input
                        id="residu"
                        type="number"
                        class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                        placeholder="0"
                        prop:value=move || form.get().residu.clone()
                        on:input=move |ev| form.update(|f| f.residu = event_target_value(&ev))
                    />
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
                        <Show when=move || form.submitting.get() fallback=|| view! { <i class="fas fa-save"></i> }>
                            <i class="fas fa-spinner fa-spin"></i>
                        </Show>
                        "Simpan Draft"
                    </button>
                </div>
            </form>
        </div>
    }
}
