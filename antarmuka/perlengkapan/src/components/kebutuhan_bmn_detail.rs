//! Kebutuhan BMN Detail Component
//!
//! Displays detailed view of a BMN needs request with satker list,
//! goods breakdown, and workflow actions.

use crate::api::{
    KebutuhanBmnStatus, PengajuanDetailResponse, PengajuanKebutuhanBmnSatker,
    WorkflowTransitionRequest, delete_kebutuhan_bmn, fetch_kebutuhan_bmn_detail,
    fetch_pengajuan_satkers, transition_kebutuhan_bmn_status,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;

#[component]
pub fn KebutuhanBmnDetail() -> impl IntoView {
    let params = use_params_map();
    let id = Memo::new(move |_| params.read().get("id").clone().unwrap_or_default());

    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<String>>(None);
    let (detail, set_detail) = signal::<Option<PengajuanDetailResponse>>(None);
    let (satkers, set_satkers) = signal::<Vec<PengajuanKebutuhanBmnSatker>>(vec![]);
    let (transitioning, set_transitioning) = signal(false);
    let (deleting, set_deleting) = signal(false);
    let (show_delete_modal, set_show_delete_modal) = signal(false);
    let (transition_comment, set_transition_comment) = signal(String::new());

    // Load detail data
    let load_data = move |pengajuan_id: String| {
        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            match fetch_kebutuhan_bmn_detail(&pengajuan_id).await {
                Ok(response) => {
                    set_satkers.set(response.data.satkers.clone());
                    set_detail.set(Some(response.data));
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal memuat data: {:?}", e)));
                }
            }
            set_loading.set(false);
        });
    };

    // Initial load
    Effect::new(move || {
        let current_id = id.get();
        if !current_id.is_empty() {
            load_data(current_id);
        }
    });

    // Handle workflow transition
    let handle_transition = move |target_status: i32| {
        set_transitioning.set(true);
        let current_id = id.get();
        let comment = transition_comment.get();

        spawn_local(async move {
            let request = WorkflowTransitionRequest {
                target_status,
                komentar: if comment.is_empty() {
                    None
                } else {
                    Some(comment)
                },
            };

            match transition_kebutuhan_bmn_status(&current_id, request).await {
                Ok(response) => {
                    set_detail.set(Some(response.data));
                    set_transition_comment.set(String::new());
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal melakukan transisi: {:?}", e)));
                }
            }
            set_transitioning.set(false);
        });
    };

    // Handle delete
    let handle_delete = move |_| {
        set_deleting.set(true);
        let current_id = id.get();

        spawn_local(async move {
            match delete_kebutuhan_bmn(&current_id).await {
                Ok(_) => {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/perlengkapan/dashboard/kebutuhan-bmn");
                    }
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal menghapus: {:?}", e)));
                    set_show_delete_modal.set(false);
                }
            }
            set_deleting.set(false);
        });
    };

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            // Header with back button
            <div class="mb-6">
                <a
                    href="/perlengkapan/dashboard/kebutuhan-bmn"
                    class="text-blue-600 hover:text-blue-800 inline-flex items-center mb-4"
                >
                    <i class="fas fa-arrow-left mr-2"></i>
                    "Kembali ke Daftar"
                </a>
            </div>

            // Loading state
            <Show when=move || loading.get()>
                <div class="text-center py-12">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-3"></div>
                    <p class="text-gray-500">"Memuat data..."</p>
                </div>
            </Show>

            // Error message
            <Show when=move || error.get().is_some()>
                <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6 flex items-center">
                    <i class="fas fa-exclamation-circle mr-2"></i>
                    <span>{move || error.get().unwrap_or_default()}</span>
                </div>
            </Show>

            // Detail content
            <Show when=move || !loading.get() && detail.get().is_some()>
                {move || {
                    detail.get().map(|d| {
                        let pengajuan = d.pengajuan.clone();
                        let status = KebutuhanBmnStatus::from_code(pengajuan.status_kode);
                        let badge_class = status.map(|s| s.badge_class()).unwrap_or("bg-gray-100 text-gray-800");
                        let status_label = status.map(|s| s.label()).unwrap_or("Unknown");
                        let allowed_transitions = StoredValue::new(d.allowed_transitions.clone());

                        view! {
                            <div class="space-y-6">
                                // Title & Status
                                <div class="flex flex-col lg:flex-row justify-between items-start lg:items-center gap-4">
                                    <div>
                                        <h2 class="text-2xl font-bold text-gray-800">{pengajuan.nama.clone()}</h2>
                                        <p class="text-gray-500 mt-1">
                                            "Tahun Anggaran: " <span class="font-medium">{pengajuan.tahun}</span>
                                        </p>
                                    </div>
                                    <div class="flex items-center gap-3">
                                        <span class=format!("px-3 py-1.5 rounded-full text-sm font-medium {}", badge_class)>
                                            {status_label}
                                        </span>
                                        // Action buttons
                                        <div class="flex gap-2">
                                            <a
                                                href=format!("/dashboard/kebutuhan-bmn/{}/edit", pengajuan.id)
                                                class="px-4 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50 transition-colors inline-flex items-center"
                                            >
                                                <i class="fas fa-edit mr-2"></i>
                                                "Edit"
                                            </a>
                                            <button
                                                class="px-4 py-2 border border-red-300 rounded-lg text-red-600 hover:bg-red-50 transition-colors"
                                                on:click=move |_| set_show_delete_modal.set(true)
                                            >
                                                <i class="fas fa-trash"></i>
                                            </button>
                                        </div>
                                    </div>
                                </div>

                                // Info cards
                                <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                                    <div class="bg-blue-50 rounded-lg p-4">
                                        <div class="text-blue-600 text-sm font-medium">"Periode"</div>
                                        <div class="text-gray-800 mt-1 font-medium">
                                            {pengajuan.tgl_mulai.clone()} " s.d. " {pengajuan.tgl_selesai.clone()}
                                        </div>
                                    </div>
                                    <div class="bg-purple-50 rounded-lg p-4">
                                        <div class="text-purple-600 text-sm font-medium">"Total Satker"</div>
                                        <div class="text-2xl font-bold text-gray-800 mt-1">{d.satkers.len()}</div>
                                    </div>
                                    <div class="bg-green-50 rounded-lg p-4">
                                        <div class="text-green-600 text-sm font-medium">"Persetujuan DASKRIMTI"</div>
                                        <div class="text-gray-800 mt-1 font-medium">
                                            {if pengajuan.is_appv_daskrimti { "Ya" } else { "Belum" }}
                                        </div>
                                    </div>
                                    <div class="bg-amber-50 rounded-lg p-4">
                                        <div class="text-amber-600 text-sm font-medium">"Versi"</div>
                                        <div class="text-2xl font-bold text-gray-800 mt-1">{pengajuan.version}</div>
                                    </div>
                                </div>

                                // Description
                                {
                                    let has_deskripsi = pengajuan.deskripsi.is_some();
                                    view! {
                                        <Show when=move || has_deskripsi>
                                            <div class="bg-gray-50 rounded-lg p-4">
                                                <div class="text-gray-600 text-sm font-medium mb-2">"Deskripsi"</div>
                                                <p class="text-gray-800">{pengajuan.deskripsi.clone().unwrap_or_default()}</p>
                                            </div>
                                        </Show>
                                    }
                                }

                                // Workflow transitions
                                <Show when=move || !allowed_transitions.with_value(|t| t.is_empty())>
                                    <div class="bg-gray-50 rounded-lg p-4">
                                        <div class="text-gray-600 text-sm font-medium mb-3">"Aksi Workflow"</div>
                                        <div class="flex flex-wrap gap-3">
                                            <For
                                                each=move || allowed_transitions.get_value()
                                                key=|t| t.status_kode
                                                children=move |transition| {
                                                    let status_kode = transition.status_kode;
                                                    let btn_class = match status_kode {
                                                        2006 => "bg-green-600 hover:bg-green-700 text-white",
                                                        2007 | 2009 => "bg-red-600 hover:bg-red-700 text-white",
                                                        _ => "bg-blue-600 hover:bg-blue-700 text-white",
                                                    };
                                                    view! {
                                                        <button
                                                            class=format!("px-4 py-2 rounded-lg transition-colors disabled:opacity-50 {}", btn_class)
                                                            disabled=move || transitioning.get()
                                                            on:click=move |_| handle_transition(status_kode)
                                                        >
                                                            {transition.status_nama.clone()}
                                                        </button>
                                                    }
                                                }
                                            />
                                        </div>
                                        // Comment input for transitions
                                        <div class="mt-3">
                                            <input
                                                type="text"
                                                placeholder="Komentar (opsional)"
                                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                                                on:input=move |ev| set_transition_comment.set(event_target_value(&ev))
                                                prop:value=move || transition_comment.get()
                                            />
                                        </div>
                                    </div>
                                </Show>

                                // Satker list
                                <div>
                                    <h3 class="text-lg font-semibold text-gray-800 mb-4">"Daftar Satker"</h3>
                                    <Show
                                        when=move || !satkers.get().is_empty()
                                        fallback=|| view! {
                                            <div class="text-center py-8 text-gray-500">
                                                <p>"Belum ada satker yang terdaftar"</p>
                                            </div>
                                        }
                                    >
                                        <div class="overflow-x-auto">
                                            <table class="w-full text-left border-collapse">
                                                <thead>
                                                    <tr class="bg-gray-50 text-gray-600 text-sm uppercase tracking-wider">
                                                        <th class="p-3 font-semibold border-b">"Nama Satker"</th>
                                                        <th class="p-3 font-semibold border-b text-center">"Status"</th>
                                                        <th class="p-3 font-semibold border-b text-center">"Prioritas"</th>
                                                        <th class="p-3 font-semibold border-b text-center">"Aksi"</th>
                                                    </tr>
                                                </thead>
                                                <tbody class="text-gray-700 text-sm">
                                                    <For
                                                        each=move || satkers.get()
                                                        key=|s| s.id.clone()
                                                        children=move |satker| {
                                                            let status = KebutuhanBmnStatus::from_code(satker.status_kode);
                                                            let badge_class = status.map(|s| s.badge_class()).unwrap_or("bg-gray-100 text-gray-800");
                                                            let status_label = status.map(|s| s.label()).unwrap_or("Unknown");
                                                            let satker_id = satker.id.clone();

                                                            view! {
                                                                <tr class="hover:bg-gray-50 border-b last:border-0 transition-colors">
                                                                    <td class="p-3 font-medium">{satker.nm_satker.clone().unwrap_or_else(|| satker.ms_satker_id.clone())}</td>
                                                                    <td class="p-3 text-center">
                                                                        <span class=format!("px-2 py-1 rounded-full text-xs font-medium {}", badge_class)>
                                                                            {status_label}
                                                                        </span>
                                                                    </td>
                                                                    <td class="p-3 text-center">{satker.prioritas}</td>
                                                                    <td class="p-3 text-center">
                                                                        <a
                                                                            href=format!("/dashboard/kebutuhan-bmn/satker/{}", satker_id)
                                                                            class="text-blue-600 hover:text-blue-800"
                                                                        >
                                                                            <i class="fas fa-eye mr-1"></i>
                                                                            "Detail"
                                                                        </a>
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }
                                                    />
                                                </tbody>
                                            </table>
                                        </div>
                                    </Show>
                                </div>
                            </div>
                        }
                    })
                }}
            </Show>

            // Delete confirmation modal
            <Show when=move || show_delete_modal.get()>
                <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                    <div class="bg-white rounded-xl p-6 max-w-md w-full mx-4 shadow-xl">
                        <h3 class="text-lg font-bold text-gray-800 mb-4">"Konfirmasi Hapus"</h3>
                        <p class="text-gray-600 mb-6">
                            "Apakah Anda yakin ingin menghapus pengajuan ini? Tindakan ini tidak dapat dibatalkan."
                        </p>
                        <div class="flex justify-end gap-3">
                            <button
                                class="px-4 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50 transition-colors"
                                on:click=move |_| set_show_delete_modal.set(false)
                            >
                                "Batal"
                            </button>
                            <button
                                class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors disabled:opacity-50"
                                disabled=move || deleting.get()
                                on:click=handle_delete
                            >
                                {move || if deleting.get() { "Menghapus..." } else { "Hapus" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
