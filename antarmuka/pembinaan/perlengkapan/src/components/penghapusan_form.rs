use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use crate::api::{create_penghapusan, CreatePenghapusanRequest};

#[component]
pub fn PenghapusanForm() -> impl IntoView {
    let (tanggal, set_tanggal) = signal("".to_string());
    let (alasan, set_alasan) = signal("".to_string());
    let (metode, set_metode) = signal("".to_string());
    let (residu, set_residu) = signal("".to_string());
    let (asset_id, set_asset_id) = signal("".to_string());
    let (error, set_error) = signal(None::<String>);
    let (success, set_success) = signal(false);
    let (loading, set_loading) = signal(false);
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);
        set_success.set(false);

        let req = CreatePenghapusanRequest {
            asset_id: asset_id.get(),
            tanggal_penghapusan: tanggal.get(),
            alasan: alasan.get(),
            metode_penghapusan: metode.get(),
            nilai_residu: residu.get().parse::<f64>().ok(),
        };

        let navigate = navigate.clone();
        spawn_local(async move {
            match create_penghapusan(req).await {
                Ok(_) => {
                    set_success.set(true);
                    gloo_timers::future::TimeoutFuture::new(1000).await;
                    navigate("/dashboard/pengelolaan/penghapusan/daftar", Default::default());
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal menyimpan: {:?}", e)));
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="max-w-2xl mx-auto p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <h2 class="text-xl font-bold text-gray-800 mb-6">"Usul Penghapusan BMN"</h2>

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
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1" for="asset_id">"Asset ID (UUID)"</label>
                    <input
                        id="asset_id"
                        type="text"
                        class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                        placeholder="Contoh: 550e8400-e29b-41d4-a716-446655440000"
                        prop:value=move || asset_id.get()
                        on:input=move |ev| set_asset_id.set(event_target_value(&ev))
                        required
                    />
                </div>

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

                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1" for="alasan">"Alasan"</label>
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

                <div class="pt-4 flex justify-end gap-3">
                    <a
                        href="/dashboard/pengelolaan/penghapusan/daftar"
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
                        "Simpan"
                    </button>
                </div>
            </form>
        </div>
    }
}
