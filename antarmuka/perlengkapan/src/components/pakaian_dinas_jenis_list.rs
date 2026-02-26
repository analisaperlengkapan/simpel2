//! Jenis Pakaian Dinas List Component
//!
//! Displays list of official uniform types with CRUD operations.

use crate::api::{
    CreateJenisPakaianDinasRequest, JenisPakaianDinas, create_jenis_pakaian_dinas,
    delete_jenis_pakaian_dinas, fetch_jenis_pakaian_dinas,
};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn PakaianDinasJenisList() -> impl IntoView {
    let (page, set_page) = signal(1);
    let (show_form, set_show_form) = signal(false);
    let (form_nama, set_form_nama) = signal(String::new());
    let (form_keterangan, set_form_keterangan) = signal(String::new());
    let (is_loading, set_is_loading) = signal(false);
    let (error_message, set_error_message) = signal(Option::<String>::None);
    let refresh_trigger = RwSignal::new(0);

    // Resource to fetch items when page changes
    let data_resource = LocalResource::new(move || {
        let p = page.get();
        let _trigger = refresh_trigger.get();
        async move {
            match fetch_jenis_pakaian_dinas(p, 20).await {
                Ok(response) => Some(response),
                Err(e) => {
                    leptos::logging::error!("Failed to fetch jenis pakaian dinas: {:?}", e);
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
        let keterangan = form_keterangan.get();

        spawn_local(async move {
            let request = CreateJenisPakaianDinasRequest {
                nama,
                keterangan: if keterangan.is_empty() {
                    None
                } else {
                    Some(keterangan)
                },
            };

            match create_jenis_pakaian_dinas(request).await {
                Ok(_) => {
                    set_form_nama.set(String::new());
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
                w.confirm_with_message("Yakin ingin menghapus jenis pakaian dinas ini?")
                    .ok()
            })
            .unwrap_or(false);

        if confirmed {
            spawn_local(async move {
                match delete_jenis_pakaian_dinas(id).await {
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

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h2 class="text-xl font-bold text-gray-800">"Jenis Pakaian Dinas"</h2>
                    <p class="text-sm text-gray-500 mt-1">"Kelola master data jenis pakaian dinas"</p>
                </div>
                <button
                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors inline-flex items-center"
                    on:click=move |_| set_show_form.update(|v| *v = !*v)
                >
                    <i class="fas fa-plus mr-2"></i>
                    "Tambah Jenis"
                </button>
            </div>

            // Form modal
            <Show when=move || show_form.get()>
                <div class="mb-6 p-4 bg-gray-50 rounded-lg border border-gray-200">
                    <h3 class="text-lg font-semibold mb-4">"Tambah Jenis Pakaian Dinas Baru"</h3>

                    <Show when=move || error_message.get().is_some()>
                        <div class="mb-4 p-3 bg-red-50 border border-red-200 text-red-700 rounded">
                            {move || error_message.get()}
                        </div>
                    </Show>

                    <form on:submit=on_submit>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">"Nama Jenis"</label>
                                <input
                                    type="text"
                                    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                    placeholder="Contoh: PDH, PDL, Toga"
                                    prop:value=move || form_nama.get()
                                    on:input=move |ev| set_form_nama.set(event_target_value(&ev))
                                    required=true
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">"Keterangan"</label>
                                <input
                                    type="text"
                                    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                    placeholder="Keterangan (opsional)"
                                    prop:value=move || form_keterangan.get()
                                    on:input=move |ev| set_form_keterangan.set(event_target_value(&ev))
                                />
                            </div>
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

            <Suspense fallback=move || view! { <div class="text-center py-8">"Memuat data..."</div> }>
                {move || {
                    data_resource.get().flatten().map(|response| {
                        if response.data.is_empty() {
                            view! {
                                <div class="text-center py-12 text-gray-500">
                                    <i class="fas fa-tshirt text-4xl mb-3 text-gray-300"></i>
                                    <p>"Belum ada data jenis pakaian dinas."</p>
                                    <p class="text-sm">"Klik tombol \"Tambah Jenis\" untuk menambahkan."</p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="overflow-x-auto">
                                    <table class="w-full text-left border-collapse">
                                        <thead>
                                            <tr class="bg-gray-50 text-gray-600 text-sm uppercase tracking-wider">
                                                <th class="p-3 font-semibold border-b">"No"</th>
                                                <th class="p-3 font-semibold border-b">"Nama Jenis"</th>
                                                <th class="p-3 font-semibold border-b">"Keterangan"</th>
                                                <th class="p-3 font-semibold border-b">"Dibuat"</th>
                                                <th class="p-3 font-semibold border-b">"Aksi"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="text-gray-700 text-sm">
                                            <For
                                                each=move || response.data.clone().into_iter().enumerate()
                                                key=|(_, item)| item.id.clone()
                                                children=move |(idx, item): (usize, JenisPakaianDinas)| {
                                                    let item_id = item.id.clone();
                                                    let item_id_for_delete = item_id.clone();
                                                    view! {
                                                        <tr class="hover:bg-gray-50 border-b last:border-0 transition-colors">
                                                            <td class="p-3 font-medium">{((page.get() - 1) * 20 + idx as i32 + 1).to_string()}</td>
                                                            <td class="p-3 font-semibold text-blue-600">{item.nama}</td>
                                                            <td class="p-3">{item.keterangan.unwrap_or_else(|| "-".to_string())}</td>
                                                            <td class="p-3 text-gray-500">{item.created_at}</td>
                                                            <td class="p-3">
                                                                <div class="flex gap-2">
                                                                    <a
                                                                        href=format!("/dashboard/pakaian-dinas/jenis/{}/spesifikasi", item_id)
                                                                        class="text-green-600 hover:text-green-800"
                                                                        title="Lihat Spesifikasi"
                                                                    >
                                                                        <i class="fas fa-list"></i>
                                                                    </a>
                                                                    <button
                                                                        class="text-red-600 hover:text-red-800"
                                                                        title="Hapus"
                                                                        on:click=move |_| on_delete(item_id_for_delete.clone())
                                                                    >
                                                                        <i class="fas fa-trash"></i>
                                                                    </button>
                                                                </div>
                                                            </td>
                                                        </tr>
                                                    }
                                                }
                                            />
                                        </tbody>
                                    </table>

                                    // Pagination
                                    <div class="flex items-center justify-between mt-6 pt-4 border-t border-gray-100">
                                        <div class="text-sm text-gray-500">
                                            "Menampilkan halaman " <span class="font-medium">{response.page}</span>
                                            " dari " <span class="font-medium">{response.total_pages}</span>
                                            " (" <span class="font-medium">{response.total}</span> " data)"
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
                                </div>
                            }.into_any()
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
