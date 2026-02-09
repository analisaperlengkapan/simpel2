//! Pengajuan Pakaian Dinas List Component
//!
//! Displays list of uniform request periods with workflow management.

use crate::api::{
    CreatePengajuanPakaianDinasRequest, PengajuanPakaianDinas, create_pengajuan_pakaian_dinas,
    delete_pengajuan_pakaian_dinas, fetch_pengajuan_pakaian_dinas,
};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn PakaianDinasPengajuanList() -> impl IntoView {
    let (page, set_page) = signal(1);
    let (show_form, set_show_form) = signal(false);
    let (form_nama, set_form_nama) = signal(String::new());
    let (form_tahun, set_form_tahun) = signal(js_sys::Date::new_0().get_full_year().to_string());
    let (form_tgl_open, set_form_tgl_open) = signal(String::new());
    let (form_tgl_close, set_form_tgl_close) = signal(String::new());
    let (form_keterangan, set_form_keterangan) = signal(String::new());
    let (is_loading, set_is_loading) = signal(false);
    let (error_message, set_error_message) = signal(Option::<String>::None);
    let refresh_trigger = RwSignal::new(0);

    // Resource to fetch items when page changes
    let data_resource = LocalResource::new(move || {
        let p = page.get();
        let _trigger = refresh_trigger.get();
        async move {
            match fetch_pengajuan_pakaian_dinas(p, 20, None).await {
                Ok(response) => Some(response),
                Err(e) => {
                    leptos::logging::error!("Failed to fetch pengajuan pakaian dinas: {:?}", e);
                    None
                }
            }
        }
    });

    // Handle form submit
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_is_loading.set(true);
        set_error_message.set(None);

        let nama = form_nama.get();
        let tahun = form_tahun.get().parse::<i32>().unwrap_or(2025);
        let tgl_open = form_tgl_open.get();
        let tgl_close = form_tgl_close.get();
        let keterangan = form_keterangan.get();

        spawn_local(async move {
            let request = CreatePengajuanPakaianDinasRequest {
                nama,
                tahun,
                tgl_open: if tgl_open.is_empty() {
                    None
                } else {
                    Some(tgl_open)
                },
                tgl_close: if tgl_close.is_empty() {
                    None
                } else {
                    Some(tgl_close)
                },
                keterangan: if keterangan.is_empty() {
                    None
                } else {
                    Some(keterangan)
                },
            };

            match create_pengajuan_pakaian_dinas(request).await {
                Ok(_) => {
                    set_form_nama.set(String::new());
                    set_form_tgl_open.set(String::new());
                    set_form_tgl_close.set(String::new());
                    set_form_keterangan.set(String::new());
                    set_show_form.set(false);
                    refresh_trigger.update(|v| *v += 1);
                }
                Err(e) => {
                    set_error_message.set(Some(format!("Gagal menyimpan: {:?}", e)));
                }
            }
            set_is_loading.set(false);
        });
    };

    // Handle delete
    let on_delete = move |id: String| {
        let confirmed = web_sys::window()
            .and_then(|w| {
                w.confirm_with_message("Yakin ingin menghapus periode pengajuan ini?")
                    .ok()
            })
            .unwrap_or(false);

        if confirmed {
            spawn_local(async move {
                match delete_pengajuan_pakaian_dinas(id).await {
                    Ok(_) => {
                        refresh_trigger.update(|v| *v += 1);
                    }
                    Err(e) => {
                        leptos::logging::error!("Failed to delete: {:?}", e);
                    }
                }
            });
        }
    };

    // Status badge helper
    fn status_badge(status: String, is_open: bool) -> impl IntoView {
        let (color, text) = if is_open {
            ("bg-green-100 text-green-800", "Dibuka".to_string())
        } else {
            match status.as_str() {
                "draft" => ("bg-gray-100 text-gray-800", "Draft".to_string()),
                "selesai" => ("bg-blue-100 text-blue-800", "Selesai".to_string()),
                _ => ("bg-yellow-100 text-yellow-800", status),
            }
        };
        view! {
            <span class=format!("px-2 py-1 text-xs font-medium rounded-full {}", color)>
                {text}
            </span>
        }
    }

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h2 class="text-xl font-bold text-gray-800">"Pengajuan Pakaian Dinas"</h2>
                    <p class="text-sm text-gray-500 mt-1">
                        "Kelola periode pengajuan pakaian dinas per tahun"
                    </p>
                </div>
                <button
                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors inline-flex items-center"
                    on:click=move |_| set_show_form.update(|v| *v = !*v)
                >
                    <i class="fas fa-plus mr-2"></i>
                    "Buat Pengajuan"
                </button>
            </div>

            // Form modal
            <Show when=move || show_form.get()>
                <div class="mb-6 p-4 bg-gray-50 rounded-lg border border-gray-200">
                    <h3 class="text-lg font-semibold mb-4">"Buat Periode Pengajuan Baru"</h3>

                    <Show when=move || error_message.get().is_some()>
                        <div class="mb-4 p-3 bg-red-50 border border-red-200 text-red-700 rounded">
                            {move || error_message.get()}
                        </div>
                    </Show>

                    <form on:submit=on_submit>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">
                                    "Nama Pengajuan"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                    placeholder="Contoh: Pengajuan PDH Tahun 2025"
                                    prop:value=move || form_nama.get()
                                    on:input=move |ev| set_form_nama.set(event_target_value(&ev))
                                    required=true
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">
                                    "Tahun"
                                </label>
                                <input
                                    type="number"
                                    min="2020"
                                    max="2099"
                                    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                    prop:value=move || form_tahun.get()
                                    on:input=move |ev| set_form_tahun.set(event_target_value(&ev))
                                    required=true
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">
                                    "Tanggal Buka"
                                </label>
                                <input
                                    type="date"
                                    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                    prop:value=move || form_tgl_open.get()
                                    on:input=move |ev| set_form_tgl_open.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">
                                    "Tanggal Tutup"
                                </label>
                                <input
                                    type="date"
                                    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                    prop:value=move || form_tgl_close.get()
                                    on:input=move |ev| set_form_tgl_close.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                        <div class="mt-4">
                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                "Keterangan"
                            </label>
                            <textarea
                                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                rows="2"
                                placeholder="Keterangan tambahan (opsional)"
                                prop:value=move || form_keterangan.get()
                                on:input=move |ev| set_form_keterangan.set(event_target_value(&ev))
                            ></textarea>
                        </div>
                        <div class="mt-4 flex gap-2">
                            <button
                                type="submit"
                                class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:opacity-50"
                                prop:disabled=move || is_loading.get()
                            >
                                {move || if is_loading.get() { "Menyimpan..." } else { "Simpan" }}
                            </button>
                            <button
                                type="button"
                                class="px-4 py-2 bg-gray-300 text-gray-700 rounded-lg hover:bg-gray-400"
                                on:click=move |_| set_show_form.set(false)
                            >
                                "Batal"
                            </button>
                        </div>
                    </form>
                </div>
            </Show>

            <Suspense fallback=move || {
                view! { <div class="text-center py-8">"Memuat data..."</div> }
            }>
                {move || {
                    data_resource
                        .get()
                        .flatten()
                        .map(|response| {
                            if response.data.is_empty() {
                                view! {
                                    <div class="text-center py-12 text-gray-500">
                                        <i class="fas fa-calendar-alt text-4xl mb-3 text-gray-300"></i>
                                        <p>"Belum ada periode pengajuan pakaian dinas."</p>
                                        <p class="text-sm">
                                            "Klik tombol \"Buat Pengajuan\" untuk membuat periode baru."
                                        </p>
                                    </div>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                        <For
                                            each=move || response.data.clone()
                                            key=|item| item.id.clone()
                                            children=move |item: PengajuanPakaianDinas| {
                                                let item_id = item.id.clone();
                                                let item_id_for_delete = item_id.clone();
                                                let item_id_for_link = item_id.clone();
                                                let status = item.status.clone();
                                                let is_open = item.is_open;
                                                let keterangan = item.keterangan.clone();
                                                let has_keterangan = keterangan.is_some();
                                                view! {
                                                    <div class="p-4 bg-gray-50 rounded-lg border border-gray-200 hover:shadow-md transition-shadow">
                                                        <div class="flex items-start justify-between mb-3">
                                                            <div>
                                                                <h3 class="font-semibold text-gray-800">
                                                                    {item.nama}
                                                                </h3>
                                                                <p class="text-sm text-gray-500">
                                                                    "Tahun: " {item.tahun.to_string()}
                                                                </p>
                                                            </div>
                                                            {status_badge(status, is_open)}
                                                        </div>

                                                        <div class="text-sm text-gray-600 mb-3">
                                                            <p>
                                                                <i class="fas fa-calendar-check mr-2 text-green-500"></i>
                                                                "Buka: "
                                                                {item.tgl_open.unwrap_or_else(|| "-".to_string())}
                                                            </p>
                                                            <p>
                                                                <i class="fas fa-calendar-times mr-2 text-red-500"></i>
                                                                "Tutup: "
                                                                {item.tgl_close.unwrap_or_else(|| "-".to_string())}
                                                            </p>
                                                        </div>

                                                        <Show when=move || has_keterangan>
                                                            <p class="text-sm text-gray-500 mb-3 italic">
                                                                {keterangan.clone()}
                                                            </p>
                                                        </Show>

                                                        <div class="flex gap-2 pt-3 border-t border-gray-200">
                                                            <a
                                                                href=format!(
                                                                    "/dashboard/pakaian-dinas/pengajuan/{}/satker",
                                                                    item_id_for_link,
                                                                )
                                                                class="flex-1 px-3 py-1 text-center text-blue-600 border border-blue-600 rounded hover:bg-blue-50 text-sm"
                                                            >
                                                                <i class="fas fa-building mr-1"></i>
                                                                "Satker"
                                                            </a>
                                                            <button
                                                                class="px-3 py-1 text-red-600 border border-red-600 rounded hover:bg-red-50 text-sm"
                                                                on:click=move |_| on_delete(item_id_for_delete.clone())
                                                            >
                                                                <i class="fas fa-trash"></i>
                                                            </button>
                                                        </div>
                                                    </div>
                                                }
                                            }
                                        />
                                    </div>

                                    // Pagination
                                    <div class="flex items-center justify-between mt-6 pt-4 border-t border-gray-100">
                                        <div class="text-sm text-gray-500">
                                            "Total: " <span class="font-medium">{response.total}</span>
                                            " pengajuan"
                                        </div>
                                        <div class="flex gap-2">
                                            <button
                                                class="px-3 py-1 border rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                                                prop:disabled=move || page.get() <= 1
                                                on:click=move |_| set_page.update(|p| *p -= 1)
                                            >
                                                "Sebelumnya"
                                            </button>
                                            <button
                                                class="px-3 py-1 border rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                                                prop:disabled=move || page.get() >= response.total_pages
                                                on:click=move |_| set_page.update(|p| *p += 1)
                                            >
                                                "Selanjutnya"
                                            </button>
                                        </div>
                                    </div>
                                }
                                    .into_any()
                            }
                        })
                }}
            </Suspense>
        </div>
    }
}
