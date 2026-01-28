use crate::api::{create_rencana, delete_rencana, get_rencana_list};
use crate::types::CreateRencanaRequest;
use chrono::NaiveDate;
use leptos::prelude::*;
use lib_ui::components::auth::ProtectedRoute;

#[component]
pub fn RencanaPengadaan() -> impl IntoView {
    // Resources
    let rencana_resource = LocalResource::new(|| async move { get_rencana_list().await });

    // Actions
    let create_action = Action::new_local(move |input: &CreateRencanaRequest| {
        let input = input.clone();
        async move {
            match create_rencana(input).await {
                Ok(_) => {
                    rencana_resource.refetch();
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
    });

    let delete_action = Action::new_local(move |id: &uuid::Uuid| {
        let id = *id;
        async move {
            match delete_rencana(id).await {
                Ok(_) => {
                    rencana_resource.refetch();
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
    });

    // Form State
    let (show_modal, set_show_modal) = signal(false);
    let (nama_kegiatan, set_nama_kegiatan) = signal(String::new());
    let (kode_rekening, set_kode_rekening) = signal(String::new());
    let (pagu_anggaran, set_pagu_anggaran) = signal(String::new());
    let (tanggal_mulai, set_tanggal_mulai) = signal(String::new());
    let (tanggal_selesai, set_tanggal_selesai) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    // Derived error state from actions
    let create_error = move || create_action.value().get().and_then(|res| res.err());
    let delete_error = move || delete_action.value().get().and_then(|res| res.err());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error_msg.set(None);

        let pagu_res = pagu_anggaran.get().parse::<i64>();
        let pagu = match pagu_res {
            Ok(val) => val,
            Err(_) => {
                set_error_msg.set(Some("Pagu Anggaran harus berupa angka".to_string()));
                return;
            }
        };

        let tgl_mulai_res = NaiveDate::parse_from_str(&tanggal_mulai.get(), "%Y-%m-%d");
        let tgl_mulai = match tgl_mulai_res {
            Ok(val) => val,
            Err(_) => {
                set_error_msg.set(Some("Tanggal Mulai tidak valid".to_string()));
                return;
            }
        };

        let tgl_selesai_res = NaiveDate::parse_from_str(&tanggal_selesai.get(), "%Y-%m-%d");
        let tgl_selesai = match tgl_selesai_res {
            Ok(val) => val,
            Err(_) => {
                set_error_msg.set(Some("Tanggal Selesai tidak valid".to_string()));
                return;
            }
        };

        if tgl_mulai > tgl_selesai {
            set_error_msg.set(Some(
                "Tanggal Mulai tidak boleh lebih besar dari Tanggal Selesai".to_string(),
            ));
            return;
        }

        let req = CreateRencanaRequest {
            nama_kegiatan: nama_kegiatan.get(),
            kode_rekening: kode_rekening.get(),
            pagu_anggaran: pagu,
            tanggal_mulai: tgl_mulai,
            tanggal_selesai: tgl_selesai,
        };

        create_action.dispatch(req);
        // Modal closing logic is now simpler: close if no validation error.
        // Backend errors will be shown in the modal if it stays open, or we can close it.
        // For better UX, we'll keep it open on backend error (handled by create_error view below)
        // OR we can close it and show a toast.
        // Here we close it optimistically. If there's an error, the user might need to re-open or we rely on resource refetch.
        // Actually, let's keep it open if there is a backend error... but `create_action` is async.
        // We'll just close it for now as per previous logic, but display global errors if any.
        set_show_modal.set(false);

        // Reset form
        set_nama_kegiatan.set("".to_string());
        set_kode_rekening.set("".to_string());
        set_pagu_anggaran.set("".to_string());
        set_tanggal_mulai.set("".to_string());
        set_tanggal_selesai.set("".to_string());
    };

    let content = move || {
        view! {
                <div class="p-6">
                    <div class="flex justify-between items-center mb-6">
                        <h1 class="text-2xl font-bold text-gray-800">"Rencana Pengadaan"</h1>
                        <button
                            class="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition-colors flex items-center gap-2"
                            on:click=move |_| set_show_modal.set(true)
                        >
                            <i class="fas fa-plus"></i>
                            "Tambah Rencana"
                        </button>
                    </div>

                    // Global Error Display (e.g. Deletion errors)
                    {move || delete_error().map(|msg| view! {
                        <div class="mb-6 bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative" role="alert">
                            <strong class="font-bold">"Error: "</strong>
                            <span class="block sm:inline">{msg}</span>
                        </div>
                    }).into_any()}

                    // Modal
                    {move || if show_modal.get() {
                        view! {
                            <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                                <div class="bg-white rounded-xl shadow-xl p-6 w-full max-w-lg mx-4">
                                    <h2 class="text-xl font-bold mb-4">"Tambah Rencana Pengadaan"</h2>

                                    // Validation Errors
                                    {move || error_msg.get().map(|msg| view! {
                                        <div class="mb-4 bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative" role="alert">
                                            <span class="block sm:inline">{msg}</span>
                                        </div>
                                    }).into_any()}

                                    // Backend Creation Errors
                                    {move || create_error().map(|msg| view! {
                                        <div class="mb-4 bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative" role="alert">
                                            <strong class="font-bold">"Server Error: "</strong>
                                            <span class="block sm:inline">{msg}</span>
                                        </div>
                                    }).into_any()}

                                    <form on:submit=on_submit>
                                        <div class="space-y-4">
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700 mb-1">"Nama Kegiatan"</label>
                                                <input
                                                    type="text"
                                                    class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500"
                                                    prop:value=nama_kegiatan
                                                    on:input=move |ev| set_nama_kegiatan.set(event_target_value(&ev))
                                                    required
                                                />
                                            </div>
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700 mb-1">"Kode Rekening"</label>
                                                <input
                                                    type="text"
                                                    class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500"
                                                    prop:value=kode_rekening
                                                    on:input=move |ev| set_kode_rekening.set(event_target_value(&ev))
                                                    required
                                                />
                                            </div>
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700 mb-1">"Pagu Anggaran"</label>
                                                <input
                                                    type="number"
                                                    class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500"
                                                    prop:value=pagu_anggaran
                                                    on:input=move |ev| set_pagu_anggaran.set(event_target_value(&ev))
                                                    required
                                                />
                                            </div>
                                            <div class="grid grid-cols-2 gap-4">
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700 mb-1">"Tanggal Mulai"</label>
                                                    <input
                                                        type="date"
                                                        class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500"
                                                        prop:value=tanggal_mulai
                                                        on:input=move |ev| set_tanggal_mulai.set(event_target_value(&ev))
                                                        required
                                                    />
                                                </div>
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700 mb-1">"Tanggal Selesai"</label>
                                                    <input
                                                        type="date"
                                                        class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500"
                                                        prop:value=tanggal_selesai
                                                        on:input=move |ev| set_tanggal_selesai.set(event_target_value(&ev))
                                                        required
                                                    />
                                                </div>
                                            </div>
                                        </div>
                                        <div class="flex justify-end gap-3 mt-6">
                                            <button
                                                type="button"
                                                class="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg"
                                                on:click=move |_| set_show_modal.set(false)
                                                prop:disabled=move || create_action.pending().get()
                                            >
                                                "Batal"
                                            </button>
                                            <button
                                                type="submit"
                                                class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-blue-300"
                                                prop:disabled=move || create_action.pending().get()
                                            >
                                                {move || if create_action.pending().get() { "Menyimpan..." } else { "Simpan" }}
                                            </button>
                                        </div>
                                    </form>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div style="display: none"></div> }.into_any()
                    }}

                    // Data Table
                    <div class="bg-white rounded-xl shadow overflow-hidden">
                        <div class="overflow-x-auto">
                            <table class="min-w-full divide-y divide-gray-200">
                                <thead class="bg-gray-50">
                                    <tr>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Nama Kegiatan"</th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Kode Rekening"</th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Pagu Anggaran"</th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Periode"</th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Status"</th>
                                        <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">"Aksi"</th>
                                    </tr>
                                </thead>
                                <tbody class="bg-white divide-y divide-gray-200">
                                    <Suspense fallback=move || view! { <tr><td colspan="6" class="px-6 py-4 text-center">"Loading..."</td></tr> }>
                                        {move || {
                                            rencana_resource.get().map(|res| match res {
                                                Ok(items) => {
                                                    if items.is_empty() {
                                                        view! { <tr><td colspan="6" class="px-6 py-4 text-center text-gray-500">"Belum ada data"</td></tr> }.into_any()
                                                    } else {
                                                        items.into_iter().map(|item| {
                                                            let id = item.id;
                                                            view! {
                                                                <tr>
                                                                    <td class="px-6 py-4 whitespace-nowrap">
                                                                        <div class="text-sm font-medium text-gray-900">{item.nama_kegiatan}</div>
                                                                    </td>
                                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                                                        {item.kode_rekening}
                                                                    </td>
                                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                        "Rp "{format_currency(item.pagu_anggaran)}
                                                                    </td>
                                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                                                        {item.tanggal_mulai.format("%d/%m/%Y").to_string()} " - " {item.tanggal_selesai.format("%d/%m/%Y").to_string()}
                                                                    </td>
                                                                    <td class="px-6 py-4 whitespace-nowrap">
                                                                        <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-yellow-100 text-yellow-800">
                                                                            {item.status}
                                                                        </span>
                                                                    </td>
                                                                    <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                                                        <button
                                                                            class="text-red-600 hover:text-red-900 ml-4"
                                                                            on:click=move |_| { delete_action.dispatch(id); }
                                                                        >
                                                                            "Hapus"
                                                                        </button>
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }).collect_view().into_any()
                                                    }
                                                }
                                                Err(e) => view! { <tr><td colspan="6" class="px-6 py-4 text-center text-red-500">"Error: " {e}</td></tr> }.into_any()
                                            })
                                        }}
                                    </Suspense>
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
        }
    };

    view! {
        <ProtectedRoute children=content />
    }
}

fn format_currency(amount: i64) -> String {
    let s = amount.to_string();
    let chars: Vec<char> = s.chars().rev().collect();
    let chunks: Vec<String> = chars
        .chunks(3)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect();
    chunks.join(".").chars().rev().collect()
}
