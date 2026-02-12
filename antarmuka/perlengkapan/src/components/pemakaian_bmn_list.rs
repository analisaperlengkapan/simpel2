//! # Pemakaian BMN List Component
//!
//! List view for BMN usage permits with filtering and history.
//! Requirements: REQ-P001, REQ-P011, REQ-P012

use leptos::prelude::*;
use crate::api::{fetch_pemakaian_bmn_list, IzinPemakaianBmn};

#[component]
pub fn PemakaianBmnList() -> impl IntoView {
    let (page, set_page) = signal(1);
    let (status_filter, set_status_filter) = signal("".to_string());
    let (jenis_bmn_filter, set_jenis_bmn_filter) = signal("".to_string());
    let (search, set_search) = signal("".to_string());

    // Resource to fetch permits
    let permits_resource = LocalResource::new(move || {
        let p = page.get();
        let status = status_filter.get();
        let jenis = jenis_bmn_filter.get();
        let search_term = search.get();

        async move {
            let status_opt = if status.is_empty() { None } else { Some(status) };
            let jenis_opt = if jenis.is_empty() { None } else { Some(jenis) };
            let search_opt = if search_term.is_empty() { None } else { Some(search_term) };

            match fetch_pemakaian_bmn_list(
                p as i32,
                20,
                status_opt,
                jenis_opt,
                None, // pegawai_nip
                None, // satker_id
                search_opt,
            ).await {
                Ok(response) => Some(response),
                Err(_) => None,
            }
        }
    });

    let get_status_badge_class = |status: &str| {
        match status {
            "DRAFT" => "bg-gray-100 text-gray-700",
            "SUBMITTED" => "bg-blue-100 text-blue-700",
            "APPROVED" => "bg-green-100 text-green-700",
            "REJECTED" => "bg-red-100 text-red-700",
            "ACTIVE" => "bg-emerald-100 text-emerald-700",
            "EXPIRED" => "bg-orange-100 text-orange-700",
            "REVOKED" => "bg-red-100 text-red-700",
            _ => "bg-gray-100 text-gray-700",
        }
    };

    let get_status_label = |status: &str| -> &'static str {
        match status {
            "DRAFT" => "Draft",
            "SUBMITTED" => "Diajukan",
            "APPROVED" => "Disetujui",
            "REJECTED" => "Ditolak",
            "ACTIVE" => "Aktif",
            "EXPIRED" => "Kadaluarsa",
            "REVOKED" => "Dicabut",
            _ => "Lainnya",
        }
    };

    view! {
        <div class="p-6 space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h2 class="text-2xl font-bold text-gray-800">"Izin Pemakaian BMN"</h2>
                    <p class="text-sm text-gray-600 mt-1">"Kelola izin pemakaian Barang Milik Negara"</p>
                </div>
                <a
                    href="/dashboard/pemakaian-bmn/baru"
                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors inline-flex items-center gap-2"
                >
                    <i class="fas fa-plus"></i>
                    "Ajukan Izin Baru"
                </a>
            </div>

            // Filters
            <div class="bg-white p-4 rounded-lg shadow-sm border border-gray-100">
                <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"Cari"</label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                            placeholder="Nama, NUP, Nomor Izin..."
                            prop:value=move || search.get()
                            on:input=move |ev| {
                                set_search.set(event_target_value(&ev));
                                set_page.set(1);
                            }
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"Status"</label>
                        <select
                            class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                            prop:value=move || status_filter.get()
                            on:change=move |ev| {
                                set_status_filter.set(event_target_value(&ev));
                                set_page.set(1);
                            }
                        >
                            <option value="">"Semua Status"</option>
                            <option value="DRAFT">"Draft"</option>
                            <option value="SUBMITTED">"Diajukan"</option>
                            <option value="APPROVED">"Disetujui"</option>
                            <option value="ACTIVE">"Aktif"</option>
                            <option value="EXPIRED">"Kadaluarsa"</option>
                            <option value="REVOKED">"Dicabut"</option>
                        </select>
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"Jenis BMN"</label>
                        <select
                            class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                            prop:value=move || jenis_bmn_filter.get()
                            on:change=move |ev| {
                                set_jenis_bmn_filter.set(event_target_value(&ev));
                                set_page.set(1);
                            }
                        >
                            <option value="">"Semua Jenis"</option>
                            <option value="KENDARAAN_BERMOTOR">"Kendaraan Bermotor"</option>
                            <option value="RUMAH_NEGARA">"Rumah Negara"</option>
                            <option value="LAPTOP">"Laptop/Komputer"</option>
                            <option value="LAINNYA">"Lainnya"</option>
                        </select>
                    </div>
                    <div class="flex items-end">
                        <button
                            class="w-full px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors"
                            on:click=move |_| {
                                set_search.set("".to_string());
                                set_status_filter.set("".to_string());
                                set_jenis_bmn_filter.set("".to_string());
                                set_page.set(1);
                            }
                        >
                            <i class="fas fa-redo mr-2"></i>
                            "Reset Filter"
                        </button>
                    </div>
                </div>
            </div>

            // Permits List
            <div class="bg-white rounded-lg shadow-sm border border-gray-100">
                <Suspense fallback=move || view! {
                    <div class="p-8 text-center">
                        <i class="fas fa-spinner fa-spin text-2xl text-gray-400 mb-2"></i>
                        <p class="text-gray-600">"Memuat data..."</p>
                    </div>
                }>
                    {move || {
                        permits_resource.get().flatten().map(|response| {
                            if response.data.is_empty() {
                                view! {
                                    <div class="p-12 text-center">
                                        <i class="fas fa-clipboard-list text-5xl text-gray-300 mb-4"></i>
                                        <p class="text-gray-600 text-lg">"Belum ada data izin pemakaian"</p>
                                        <p class="text-gray-500 text-sm mt-2">"Klik tombol 'Ajukan Izin Baru' untuk membuat permohonan"</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div>
                                        <div class="overflow-x-auto">
                                            <table class="w-full">
                                                <thead class="bg-gray-50 border-b border-gray-200">
                                                    <tr>
                                                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">"Nomor Izin"</th>
                                                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">"Pemohon"</th>
                                                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">"BMN"</th>
                                                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">"Periode"</th>
                                                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">"Status"</th>
                                                        <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">"Aksi"</th>
                                                    </tr>
                                                </thead>
                                                <tbody class="divide-y divide-gray-200">
                                                    <For
                                                        each=move || response.data.clone()
                                                        key=|permit| permit.id.clone()
                                                        children=move |permit: IzinPemakaianBmn| {
                                                            let status_class = get_status_badge_class(&permit.status);
                                                            let status_label = get_status_label(&permit.status);

                                                            view! {
                                                                <tr class="hover:bg-gray-50 transition-colors">
                                                                    <td class="px-4 py-3">
                                                                        <div class="font-medium text-gray-900">
                                                                            {permit.nomor_izin.clone().unwrap_or_else(|| "-".to_string())}
                                                                        </div>
                                                                    </td>
                                                                    <td class="px-4 py-3">
                                                                        <div class="text-sm">
                                                                            <div class="font-medium text-gray-900">{permit.pegawai_nama.clone()}</div>
                                                                            <div class="text-gray-500">{permit.pegawai_nip.clone()}</div>
                                                                        </div>
                                                                    </td>
                                                                    <td class="px-4 py-3">
                                                                        <div class="text-sm">
                                                                            <div class="font-medium text-gray-900">{permit.bmn_nama_barang.clone()}</div>
                                                                            <div class="text-gray-500">{permit.bmn_nup.clone()}</div>
                                                                        </div>
                                                                    </td>
                                                                    <td class="px-4 py-3">
                                                                        <div class="text-sm text-gray-900">
                                                                            {permit.tanggal_mulai.clone()} " - " {permit.tanggal_selesai.clone()}
                                                                        </div>
                                                                    </td>
                                                                    <td class="px-4 py-3">
                                                                        <span class={format!("px-2 py-1 rounded-full text-xs font-medium {}", status_class)}>
                                                                            {status_label}
                                                                        </span>
                                                                    </td>
                                                                    <td class="px-4 py-3">
                                                                        <div class="flex gap-2">
                                                                            <a
                                                                                href={format!("/dashboard/pemakaian-bmn/{}", permit.id)}
                                                                                class="text-blue-600 hover:text-blue-800"
                                                                                title="Detail"
                                                                            >
                                                                                <i class="fas fa-eye"></i>
                                                                            </a>
                                                                            <Show when=move || permit.status == "ACTIVE">
                                                                                <button
                                                                                    class="text-green-600 hover:text-green-800"
                                                                                    title="Perpanjang"
                                                                                >
                                                                                    <i class="fas fa-redo"></i>
                                                                                </button>
                                                                            </Show>
                                                                        </div>
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }
                                                    />
                                                </tbody>
                                            </table>
                                        </div>

                                        // Pagination
                                        <div class="px-4 py-3 border-t border-gray-200 flex items-center justify-between">
                                            <div class="text-sm text-gray-700">
                                                "Menampilkan halaman " <span class="font-medium">{response.page}</span>
                                                " dari " <span class="font-medium">{response.total_pages}</span>
                                                " (" <span class="font-medium">{response.total}</span> " total)"
                                            </div>
                                            <div class="flex gap-2">
                                                <button
                                                    class="px-3 py-1 border rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                                                    prop:disabled=move || page.get() <= 1
                                                    on:click=move |_| set_page.update(|p| *p -= 1)
                                                >
                                                    <i class="fas fa-chevron-left"></i>
                                                </button>
                                                <button
                                                    class="px-3 py-1 border rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                                                    prop:disabled=move || page.get() >= response.total_pages
                                                    on:click=move |_| set_page.update(|p| *p += 1)
                                                >
                                                    <i class="fas fa-chevron-right"></i>
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
        </div>
    }
}
