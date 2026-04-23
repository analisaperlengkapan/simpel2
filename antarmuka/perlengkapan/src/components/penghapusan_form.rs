use crate::api::{CreatePenghapusanBmnWorkflowRequest, create_penghapusan_bmn_workflow};
use crate::routes;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;

#[component]
pub fn PenghapusanForm() -> impl IntoView {
    let (satker_id, set_satker_id) = signal("".to_string());
    let (asset_id, set_asset_id) = signal("".to_string());
    let (kode_barang, set_kode_barang) = signal("".to_string());
    let (nama_barang, set_nama_barang) = signal("".to_string());
    let (nup, set_nup) = signal("".to_string());
    let (tanggal, set_tanggal) = signal("".to_string());
    let (alasan, set_alasan) = signal("".to_string());
    let (metode, set_metode) = signal("".to_string());
    let (residu, set_residu) = signal("".to_string());
    let (lampiran_persyaratan, set_lampiran_persyaratan) = signal("".to_string());
    let (catatan_operator, set_catatan_operator) = signal("".to_string());
    let (error, set_error) = signal(None::<String>);
    let (success, set_success) = signal(false);
    let (loading, set_loading) = signal(false);
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);
        set_success.set(false);

        let req = CreatePenghapusanBmnWorkflowRequest {
            satker_id: satker_id.get(),
            asset_id: asset_id.get(),
            kode_barang: kode_barang.get(),
            nama_barang: nama_barang.get(),
            nup: nup.get(),
            tanggal_penghapusan: tanggal.get(),
            alasan: alasan.get(),
            metode_penghapusan: metode.get(),
            nilai_residu: residu.get().parse::<f64>().ok(),
            lampiran_persyaratan: lampiran_persyaratan.get(),
            catatan_operator: if catatan_operator.get().is_empty() {
                None
            } else {
                Some(catatan_operator.get())
            },
        };

        let navigate = navigate.clone();
        spawn_local(async move {
            match create_penghapusan_bmn_workflow(req).await {
                Ok(resp) => {
                    set_success.set(true);
                    gloo_timers::future::TimeoutFuture::new(1000).await;
                    navigate(
                        &format!("/perlengkapan/pengelolaan/penghapusan/{}", resp.data.id),
                        Default::default(),
                    );
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal menyimpan: {:?}", e)));
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <Title text="Usulan Penghapusan BMN — SIMPEL" />
        <div class="max-w-2xl mx-auto p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <h2 class="text-xl font-bold text-gray-800 mb-6">"Usulan SK Penghapusan BMN"</h2>

            <Show when=move || success.get()>
                <div class="mb-4 p-4 bg-green-50 text-green-700 rounded-lg border border-green-100 flex items-center gap-2">
                    <i class="fas fa-check-circle"></i>
                    "Usulan penghapusan berhasil disimpan!"
                </div>
            </Show>

            <Show when=move || error.get().is_some()>
                <div class="mb-4 p-4 bg-red-50 text-red-700 rounded-lg border border-red-100 flex items-center gap-2">
                    <i class="fas fa-exclamation-circle"></i>
                    {error.get()}
                </div>
            </Show>

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
                            prop:value=move || satker_id.get()
                            on:input=move |ev| set_satker_id.set(event_target_value(&ev))
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
                            prop:value=move || asset_id.get()
                            on:input=move |ev| set_asset_id.set(event_target_value(&ev))
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
                            prop:value=move || kode_barang.get()
                            on:input=move |ev| set_kode_barang.set(event_target_value(&ev))
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
                            prop:value=move || nama_barang.get()
                            on:input=move |ev| set_nama_barang.set(event_target_value(&ev))
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
                            prop:value=move || nup.get()
                            on:input=move |ev| set_nup.set(event_target_value(&ev))
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
                            prop:value=move || tanggal.get()
                            on:input=move |ev| set_tanggal.set(event_target_value(&ev))
                            required
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1" for="metode">"Metode Penghapusan"</label>
                        <select
                            id="metode"
                            class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                            prop:value=move || metode.get()
                            on:change=move |ev| set_metode.set(event_target_value(&ev))
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
                        prop:value=move || alasan.get()
                        on:input=move |ev| set_alasan.set(event_target_value(&ev))
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
                        prop:value=move || residu.get()
                        on:input=move |ev| set_residu.set(event_target_value(&ev))
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
                        prop:value=move || lampiran_persyaratan.get()
                        on:input=move |ev| set_lampiran_persyaratan.set(event_target_value(&ev))
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
                        prop:value=move || catatan_operator.get()
                        on:input=move |ev| set_catatan_operator.set(event_target_value(&ev))
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
                        prop:disabled=move || loading.get()
                    >
                        <Show when=move || loading.get() fallback=|| view! { <i class="fas fa-save"></i> }>
                            <i class="fas fa-spinner fa-spin"></i>
                        </Show>
                        "Simpan Draft"
                    </button>
                </div>
            </form>
        </div>
    }
}
