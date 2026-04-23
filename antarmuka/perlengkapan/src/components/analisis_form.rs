use crate::api::{CreateAnalisisRequest, create_analisis};
use crate::routes;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;

#[component]
pub fn AnalisisForm() -> impl IntoView {
    let (judul, set_judul) = signal("".to_string());
    let (kategori, set_kategori) = signal("".to_string());
    let (prioritas, set_prioritas) = signal("sedang".to_string());
    let (estimasi, set_estimasi) = signal("".to_string());
    let (error, set_error) = signal(None::<String>);
    let (success, set_success) = signal(false);
    let (loading, set_loading) = signal(false);
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);
        set_success.set(false);

        let req = CreateAnalisisRequest {
            judul: judul.get(),
            kategori: kategori.get(),
            deskripsi: None,
            prioritas: prioritas.get(),
            estimasi_biaya: estimasi.get().parse::<f64>().ok(),
            justifikasi: None,
        };

        let navigate = navigate.clone();
        spawn_local(async move {
            match create_analisis(req).await {
                Ok(_) => {
                    set_success.set(true);
                    // Redirect after short delay to show success
                    gloo_timers::future::TimeoutFuture::new(1000).await;
                    navigate(routes::path::ANALISIS_DAFTAR_LEGACY, Default::default());
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal menyimpan: {:?}", e)));
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <Title text="Buat Analisis — SIMPEL Perlengkapan" />
        <div class="max-w-2xl mx-auto p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <h2 class="text-xl font-bold text-gray-800 mb-6">"Buat Analisis Kebutuhan"</h2>

            <Show when=move || success.get()>
                <div class="mb-4 p-4 bg-green-50 text-green-700 rounded-lg border border-green-100 flex items-center gap-2">
                    <i class="fas fa-check-circle"></i>
                    "Data analisis berhasil disimpan!"
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
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Judul Analisis"</label>
                    <input
                        type="text"
                        class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                        placeholder="Contoh: Kebutuhan Server Data Center"
                        prop:value=move || judul.get()
                        on:input=move |ev| set_judul.set(event_target_value(&ev))
                        required
                    />
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"Kategori"</label>
                        <select
                            class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                            prop:value=move || kategori.get()
                            on:change=move |ev| set_kategori.set(event_target_value(&ev))
                            required
                        >
                            <option value="">"Pilih Kategori"</option>
                            <option value="TIK">"TIK"</option>
                            <option value="Kendaraan">"Kendaraan"</option>
                            <option value="Gedung">"Gedung"</option>
                            <option value="Lainnya">"Lainnya"</option>
                        </select>
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"Prioritas"</label>
                        <select
                            class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                            prop:value=move || prioritas.get()
                            on:change=move |ev| set_prioritas.set(event_target_value(&ev))
                        >
                            <option value="rendah">"Rendah"</option>
                            <option value="sedang">"Sedang"</option>
                            <option value="tinggi">"Tinggi"</option>
                        </select>
                    </div>
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Estimasi Biaya (Rp)"</label>
                    <input
                        type="number"
                        class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                        placeholder="0"
                        prop:value=move || estimasi.get()
                        on:input=move |ev| set_estimasi.set(event_target_value(&ev))
                    />
                </div>

                <div class="pt-4 flex justify-end gap-3">
                    <a
                        href=routes::path::ANALISIS_DAFTAR_LEGACY
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
