//! # Pemakaian BMN List Component
//!
//! List view for BMN usage permits with filtering and history.
//! Requirements: REQ-P001, REQ-P011, REQ-P012

use crate::api::{IzinPemakaianBmn, PaginatedResponse, fetch_pemakaian_bmn_list};
use crate::components::list_feedback::{EmptyState, LoadingState};
use crate::components::page_header::PageHeader;
use crate::components::pagination_controls::PaginationControls;
use crate::routes;
use leptos::prelude::*;
use leptos_fetch::QueryClient;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{ARROW_CLOCKWISE, EYE};

/// leptos-fetch query keyed by `(page, status, jenis, search)` —
/// each unique filter combination caches separately so toggling
/// back to a previous filter is instant.
async fn query_pemakaian_bmn_page(
    key: (i32, String, String, String),
) -> Option<PaginatedResponse<IzinPemakaianBmn>> {
    let (page, status, jenis, search_term) = key;
    let to_opt = |s: String| if s.is_empty() { None } else { Some(s) };
    fetch_pemakaian_bmn_list(
        page,
        20,
        to_opt(status),
        to_opt(jenis),
        None, // pegawai_nip
        None, // satker_id
        to_opt(search_term),
    )
    .await
    .ok()
}

#[component]
pub fn PemakaianBmnList() -> impl IntoView {
    let (page, set_page) = signal(1);
    let (status_filter, set_status_filter) = signal("".to_string());
    let (jenis_bmn_filter, set_jenis_bmn_filter) = signal("".to_string());
    let (search, set_search) = signal("".to_string());

    // Resource to fetch permits — keyed cache per filter combination.
    let client: QueryClient = expect_context();
    let permits_resource = client.local_resource(query_pemakaian_bmn_page, move || {
        (
            page.get(),
            status_filter.get(),
            jenis_bmn_filter.get(),
            search.get(),
        )
    });

    let get_status_badge_class = |status: &str| match status {
        "DRAFT" => "bg-gray-100 text-gray-700",
        "SUBMITTED" => "bg-blue-100 text-blue-700",
        "APPROVED" => "bg-green-100 text-green-700",
        "REJECTED" => "bg-red-100 text-red-700",
        "ACTIVE" => "bg-emerald-100 text-emerald-700",
        "EXPIRED" => "bg-orange-100 text-orange-700",
        "REVOKED" => "bg-red-100 text-red-700",
        _ => "bg-gray-100 text-gray-700",
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
            <PageHeader
                title="Izin Pemakaian BMN"
                subtitle="Kelola izin pemakaian Barang Milik Negara"
                action_href=routes::path::PEMAKAIAN_BUAT_LEGACY
                action_label="Ajukan Izin Baru"
            />

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
                            <span class="mr-2"><AppIcon icon=ARROW_CLOCKWISE /></span>
                            "Reset Filter"
                        </button>
                    </div>
                </div>
            </div>

            // Permits List
            <div class="bg-white rounded-lg shadow-sm border border-gray-100">
                <Suspense fallback=move || view! { <LoadingState /> }>
                    {move || {
                        permits_resource.get().flatten().map(|response| {
                            if response.data.is_empty() {
                                view! {
                                    <EmptyState
                                        title="Belum ada data izin pemakaian"
                                        description="Klik tombol 'Ajukan Izin Baru' untuk membuat permohonan"
                                        icon_class="fas fa-clipboard-list"
                                    />
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
                                                                                href={format!("/perlengkapan/pemakaian-bmn/{}", permit.id)}
                                                                                class="text-blue-600 hover:text-blue-800"
                                                                                title="Detail"
                                                                            >
                                                                                <AppIcon icon=EYE />
                                                                            </a>
                                                                            <Show when=move || permit.status == "ACTIVE">
                                                                                <button
                                                                                    class="text-green-600 hover:text-green-800"
                                                                                    title="Perpanjang"
                                                                                >
                                                                                    <AppIcon icon=ARROW_CLOCKWISE />
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

                                        <div class="px-4 py-3 border-t border-gray-200">
                                            <PaginationControls
                                                current_page=response.page
                                                total_pages=response.total_pages
                                                total_items=Some(response.total)
                                                on_prev=Callback::new(move |_| set_page.update(|p| *p -= 1))
                                                on_next=Callback::new(move |_| set_page.update(|p| *p += 1))
                                            />
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
