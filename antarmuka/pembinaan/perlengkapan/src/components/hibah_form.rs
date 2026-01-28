use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use crate::api::{create_hibah, CreateHibahRequest};

#[component]
pub fn HibahForm() -> impl IntoView {
    let (pemberi, set_pemberi) = signal("".to_string());
    let (penerima, set_penerima) = signal("".to_string());
    let (tanggal, set_tanggal) = signal("".to_string());
    let (keterangan, set_keterangan) = signal("".to_string());
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

        let req = CreateHibahRequest {
            asset_id: asset_id.get(),
            pemberi: pemberi.get(),
            penerima: penerima.get(),
            tanggal_hibah: tanggal.get(),
            keterangan: if keterangan.get().is_empty() { None } else { Some(keterangan.get()) },
        };

        let navigate = navigate.clone();
        spawn_local(async move {
            match create_hibah(req).await {
                Ok(_) => {
                    set_success.set(true);
                    gloo_timers::future::TimeoutFuture::new(1000).await;
                    navigate("/dashboard/pengelolaan/hibah/daftar", Default::default());
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
            <h2 class="text-xl font-bold text-gray-800 mb-6">"Catat Hibah BMN"</h2>

            <Show when=move || success.get()>
                <div class="mb-4 p-4 bg-green-50 text-green-700 rounded-lg border border-green-100 flex items-center gap-2">
                    <i class="fas fa-check-circle"></i>
                    "Data hibah berhasil disimpan!"
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

                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1" for="pemberi">"Pemberi Hibah"</label>
                        <input
                            id="pemberi"
                            type="text"
                            class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                            placeholder="Nama Pemberi"
                            prop:value=move || pemberi.get()
                            on:input=move |ev| set_pemberi.set(event_target_value(&ev))
                            required
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1" for="penerima">"Penerima Hibah"</label>
                        <input
                            id="penerima"
                            type="text"
                            class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                            placeholder="Nama Penerima"
                            prop:value=move || penerima.get()
                            on:input=move |ev| set_penerima.set(event_target_value(&ev))
                            required
                        />
                    </div>
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1" for="tanggal">"Tanggal Hibah"</label>
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
                    <label class="block text-sm font-medium text-gray-700 mb-1" for="keterangan">"Keterangan"</label>
                    <textarea
                        id="keterangan"
                        class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                        rows="3"
                        prop:value=move || keterangan.get()
                        on:input=move |ev| set_keterangan.set(event_target_value(&ev))
                    ></textarea>
                </div>

                <div class="pt-4 flex justify-end gap-3">
                    <a
                        href="/dashboard/pengelolaan/hibah/daftar"
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
