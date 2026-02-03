use crate::api::{CreatePengadaanRequest, create_pengadaan};
use chrono::NaiveDate;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

#[component]
pub fn PengadaanForm() -> impl IntoView {
    let (judul, set_judul) = signal("".to_string());
    let (jenis, set_jenis) = signal("".to_string());
    let (anggaran, set_anggaran) = signal("".to_string());
    let (target, set_target) = signal("".to_string());
    let (error, set_error) = signal(None::<String>);
    let (success, set_success) = signal(false);
    let (loading, set_loading) = signal(false);
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);
        set_success.set(false);

        let tgl_target = if target.get().is_empty() {
            None
        } else {
            match NaiveDate::parse_from_str(&target.get(), "%Y-%m-%d") {
                Ok(d) => Some(d),
                Err(_) => {
                    set_error.set(Some("Target selesai tidak valid".to_string()));
                    set_loading.set(false);
                    return;
                }
            }
        };

        let req = CreatePengadaanRequest {
            judul: judul.get(),
            deskripsi: None,
            jenis: jenis.get(),
            anggaran: anggaran.get().parse::<f64>().ok(),
            target_selesai: tgl_target,
            pic_user_id: None,
        };

        let navigate = navigate.clone();
        spawn_local(async move {
            match create_pengadaan(req).await {
                Ok(_) => {
                    set_success.set(true);
                    // Redirect after short delay to show success
                    gloo_timers::future::TimeoutFuture::new(1000).await;
                    navigate("/dashboard/pengadaan/daftar", Default::default());
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
            <h2 class="text-xl font-bold text-gray-800 mb-6">"Buat Pengadaan Baru"</h2>

            <Show when=move || success.get()>
                <div class="mb-4 p-4 bg-green-50 text-green-700 rounded-lg border border-green-100 flex items-center gap-2">
                    <i class="fas fa-check-circle"></i>
                    "Data pengadaan berhasil disimpan!"
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
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Judul Pengadaan"</label>
                    <input
                        type="text"
                        class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                        placeholder="Contoh: Pengadaan Laptop 2024"
                        prop:value=move || judul.get()
                        on:input=move |ev| set_judul.set(event_target_value(&ev))
                        required
                    />
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"Jenis Pengadaan"</label>
                        <select
                            class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                            prop:value=move || jenis.get()
                            on:change=move |ev| set_jenis.set(event_target_value(&ev))
                            required
                        >
                            <option value="">"Pilih Jenis"</option>
                            <option value="TIK">"TIK"</option>
                            <option value="Non-TIK">"Non-TIK"</option>
                            <option value="Jasa">"Jasa"</option>
                            <option value="Konstruksi">"Konstruksi"</option>
                        </select>
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"Anggaran (Rp)"</label>
                        <input
                            type="number"
                            class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                            placeholder="0"
                            prop:value=move || anggaran.get()
                            on:input=move |ev| set_anggaran.set(event_target_value(&ev))
                        />
                    </div>
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Target Selesai"</label>
                    <input
                        type="date"
                        class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                        prop:value=move || target.get()
                        on:input=move |ev| set_target.set(event_target_value(&ev))
                    />
                </div>

                <div class="pt-4 flex justify-end gap-3">
                    <a
                        href="/dashboard/pengadaan/daftar"
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
