use crate::api::{CreateAnalisisRequest, create_analisis};
use crate::routes;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;
use lib_ui::hooks::{use_form, use_toast::use_toast};

/// Form data for creating an analysis request.
#[derive(Clone, Default)]
struct AnalisisFormData {
    judul: String,
    kategori: String,
    prioritas: String,
    estimasi: String,
}

#[component]
pub fn AnalisisForm() -> impl IntoView {
    let form = use_form(AnalisisFormData {
        prioritas: "sedang".to_string(),
        ..Default::default()
    });
    let toast = use_toast();
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        let data = form.begin_submit();

        let req = CreateAnalisisRequest {
            judul: data.judul,
            kategori: data.kategori,
            deskripsi: None,
            prioritas: data.prioritas,
            estimasi_biaya: data.estimasi.parse::<f64>().ok(),
            justifikasi: None,
        };

        let navigate = navigate.clone();
        spawn_local(async move {
            match create_analisis(req).await {
                Ok(_) => {
                    // Show toast immediately, but keep `submitting=true` during
                    // the navigation delay to prevent double-submission. Only
                    // call `finish_ok()` after the timeout, right before we
                    // navigate away.
                    toast.success("Data analisis berhasil disimpan!");
                    gloo_timers::future::TimeoutFuture::new(1000).await;
                    form.finish_ok();
                    navigate(routes::path::ANALITIK_ROADMAP, Default::default());
                }
                Err(e) => {
                    form.finish_err(format!("Gagal menyimpan: {:?}", e));
                    toast.error(form.error.get_untracked().unwrap_or_default());
                }
            }
        });
    };

    view! {
        <Title text="Buat Analisis — SIMPEL Perlengkapan" />
        <div class="max-w-2xl mx-auto p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <h2 class="text-xl font-bold text-gray-800 mb-6">"Buat Analisis Kebutuhan"</h2>

            <form on:submit=on_submit class="space-y-4">
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Judul Analisis"</label>
                    <input
                        type="text"
                        class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                        placeholder="Contoh: Kebutuhan Server Data Center"
                        prop:value=move || form.get().judul.clone()
                        on:input=move |ev| form.update(|f| f.judul = event_target_value(&ev))
                        required
                    />
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"Kategori"</label>
                        <select
                            class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                            prop:value=move || form.get().kategori.clone()
                            on:change=move |ev| form.update(|f| f.kategori = event_target_value(&ev))
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
                            prop:value=move || form.get().prioritas.clone()
                            on:change=move |ev| form.update(|f| f.prioritas = event_target_value(&ev))
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
                        prop:value=move || form.get().estimasi.clone()
                        on:input=move |ev| form.update(|f| f.estimasi = event_target_value(&ev))
                    />
                </div>

                <div class="pt-4 flex justify-end gap-3">
                    <a
                        href=routes::path::ANALITIK_ROADMAP
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
                        "Simpan"
                    </button>
                </div>
            </form>
        </div>
    }
}
