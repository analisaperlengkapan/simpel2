//! # Pemakaian BMN Detail Component
//!
//! Detail view for BMN usage permit with workflow actions.
//! Requirements: REQ-P001, REQ-P004, REQ-P008, REQ-P009

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use crate::api::{
    fetch_pemakaian_bmn_detail, transition_pemakaian_bmn_status,
    revoke_pemakaian_bmn, IzinPemakaianDetailResponse,
    PemakaianWorkflowTransitionRequest, RevokePermitRequest,
};

#[component]
pub fn PemakaianBmnDetail() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.read().get("id").unwrap_or_default();

    // Resource to fetch permit detail
    let permit_resource = LocalResource::new(move || {
        let permit_id = id();
        async move {
            match fetch_pemakaian_bmn_detail(&permit_id).await {
                Ok(response) => Some(response.data),
                Err(_) => None,
            }
        }
    });

    // UI state
    let (show_transition_modal, set_show_transition_modal) = signal(false);
    let (selected_transition, set_selected_transition) = signal(None::<String>);
    let (transition_comment, set_transition_comment) = signal("".to_string());
    let (show_revoke_modal, set_show_revoke_modal) = signal(false);
    let (revoke_reason, set_revoke_reason) = signal("".to_string());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);

    // Handle workflow transition
    let handle_transition = move |_| {
        if let Some(target_status) = selected_transition.get() {
            set_loading.set(true);
            set_error.set(None);

            let permit_id = id();
            let comment = transition_comment.get();

            leptos::task::spawn_local(async move {
                let request = PemakaianWorkflowTransitionRequest {
                    target_status,
                    catatan: if comment.is_empty() { None } else { Some(comment) },
                };

                match transition_pemakaian_bmn_status(&permit_id, request).await {
                    Ok(_) => {
                        set_show_transition_modal.set(false);
                        set_transition_comment.set("".to_string());
                        permit_resource.refetch();
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Gagal mengubah status: {}", e)));
                    }
                }
                set_loading.set(false);
            });
        }
    };

    // Handle revocation
    let handle_revoke = move |_| {
        let reason = revoke_reason.get();
        if reason.len() < 10 {
            set_error.set(Some("Alasan pencabutan minimal 10 karakter".to_string()));
            return;
        }

        set_loading.set(true);
        set_error.set(None);

        let permit_id = id();

        leptos::task::spawn_local(async move {
            let request = RevokePermitRequest { alasan: reason };

            match revoke_pemakaian_bmn(&permit_id, request).await {
                Ok(_) => {
                    set_show_revoke_modal.set(false);
                    set_revoke_reason.set("".to_string());
                    permit_resource.refetch();
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal mencabut izin: {}", e)));
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="p-6 space-y-6">
            <Suspense fallback=move || view! {
                <div class="p-8 text-center">
                    <i class="fas fa-spinner fa-spin text-2xl text-gray-400 mb-2"></i>
                    <p class="text-gray-600">"Memuat data..."</p>
                </div>
            }>
                {move || {
                    permit_resource.get().flatten().map(|detail: IzinPemakaianDetailResponse| {
                        let izin = detail.izin.clone();
                        let transitions = detail.allowed_transitions.clone();

                        view! {
                            <div>
                                // Header
                                <div class="flex items-center justify-between mb-6">
                                    <div>
                                        <h2 class="text-2xl font-bold text-gray-800">"Detail Izin Pemakaian BMN"</h2>
                                        <p class="text-sm text-gray-600 mt-1">
                                            {izin.nomor_izin.clone().unwrap_or_else(|| "Belum ada nomor izin".to_string())}
                                        </p>
                                    </div>
                                    <a
                                        href="/dashboard/pemakaian-bmn"
                                        class="px-4 py-2 text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition-colors"
                                    >
                                        <i class="fas fa-arrow-left mr-2"></i>
                                        "Kembali"
                                    </a>
                                </div>

                                // Error message
                                <Show when=move || error.get().is_some()>
                                    <div class="mb-4 p-4 bg-red-50 text-red-700 rounded-lg border border-red-100">
                                        {error.get()}
                                    </div>
                                </Show>

                                // Expiry warning
                                <Show when=move || detail.is_expiring_soon>
                                    <div class="mb-4 p-4 bg-yellow-50 text-yellow-700 rounded-lg border border-yellow-100 flex items-center gap-2">
                                        <i class="fas fa-exclamation-triangle"></i>
                                        <span>
                                            "Izin akan berakhir dalam "
                                            {detail.days_until_expiry.unwrap_or(0)}
                                            " hari"
                                        </span>
                                    </div>
                                </Show>

                                // Main content
                                <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                                    // Left column - Details
                                    <div class="lg:col-span-2 space-y-6">
                                        // Pemohon Information
                                        <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-100">
                                            <h3 class="text-lg font-semibold text-gray-800 mb-4">"Informasi Pemohon"</h3>
                                            <div class="grid grid-cols-2 gap-4">
                                                <div>
                                                    <p class="text-sm text-gray-600">"NIP"</p>
                                                    <p class="font-medium">{izin.pegawai_nip.clone()}</p>
                                                </div>
                                                <div>
                                                    <p class="text-sm text-gray-600">"Nama"</p>
                                                    <p class="font-medium">{izin.pegawai_nama.clone()}</p>
                                                </div>
                                                <div>
                                                    <p class="text-sm text-gray-600">"Satker"</p>
                                                    <p class="font-medium">{izin.pegawai_satker_nama.clone()}</p>
                                                </div>
                                                <Show when=move || izin.pegawai_jabatan.is_some()>
                                                    <div>
                                                        <p class="text-sm text-gray-600">"Jabatan"</p>
                                                        <p class="font-medium">{izin.pegawai_jabatan.clone().unwrap_or_default()}</p>
                                                    </div>
                                                </Show>
                                            </div>
                                        </div>

                                        // BMN Information
                                        <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-100">
                                            <h3 class="text-lg font-semibold text-gray-800 mb-4">"Informasi BMN"</h3>
                                            <div class="grid grid-cols-2 gap-4">
                                                <div>
                                                    <p class="text-sm text-gray-600">"Jenis BMN"</p>
                                                    <p class="font-medium">{izin.jenis_bmn.clone()}</p>
                                                </div>
                                                <div>
                                                    <p class="text-sm text-gray-600">"NUP"</p>
                                                    <p class="font-medium">{izin.bmn_nup.clone()}</p>
                                                </div>
                                                <div class="col-span-2">
                                                    <p class="text-sm text-gray-600">"Nama Barang"</p>
                                                    <p class="font-medium">{izin.bmn_nama_barang.clone()}</p>
                                                </div>

                                                // Vehicle-specific fields
                                                <Show when=move || izin.no_polisi.is_some()>
                                                    <div>
                                                        <p class="text-sm text-gray-600">"Nomor Polisi"</p>
                                                        <p class="font-medium">{izin.no_polisi.clone().unwrap_or_default()}</p>
                                                    </div>
                                                </Show>

                                                // Housing-specific fields
                                                <Show when=move || izin.alamat.is_some()>
                                                    <div class="col-span-2">
                                                        <p class="text-sm text-gray-600">"Alamat"</p>
                                                        <p class="font-medium">{izin.alamat.clone().unwrap_or_default()}</p>
                                                    </div>
                                                </Show>
                                            </div>
                                        </div>

                                        // Permit Details
                                        <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-100">
                                            <h3 class="text-lg font-semibold text-gray-800 mb-4">"Detail Izin"</h3>
                                            <div class="space-y-4">
                                                <div class="grid grid-cols-2 gap-4">
                                                    <div>
                                                        <p class="text-sm text-gray-600">"Tanggal Mulai"</p>
                                                        <p class="font-medium">{izin.tanggal_mulai.clone()}</p>
                                                    </div>
                                                    <div>
                                                        <p class="text-sm text-gray-600">"Tanggal Selesai"</p>
                                                        <p class="font-medium">{izin.tanggal_selesai.clone()}</p>
                                                    </div>
                                                </div>
                                                <div>
                                                    <p class="text-sm text-gray-600">"Keperluan"</p>
                                                    <p class="font-medium">{izin.keperluan.clone()}</p>
                                                </div>
                                                <Show when=move || izin.lokasi_pemakaian.is_some()>
                                                    <div>
                                                        <p class="text-sm text-gray-600">"Lokasi Pemakaian"</p>
                                                        <p class="font-medium">{izin.lokasi_pemakaian.clone().unwrap_or_default()}</p>
                                                    </div>
                                                </Show>
                                            </div>
                                        </div>
                                    </div>

                                    // Right column - Actions
                                    <div class="space-y-6">
                                        // Status card
                                        <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-100">
                                            <h3 class="text-lg font-semibold text-gray-800 mb-4">"Status"</h3>
                                            <div class="text-center py-4">
                                                <span class="px-4 py-2 rounded-full text-sm font-medium bg-blue-100 text-blue-700">
                                                    {izin.status.clone()}
                                                </span>
                                            </div>
                                        </div>

                                        // Actions card
                                        <Show when=move || !transitions.is_empty()>
                                            <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-100">
                                                <h3 class="text-lg font-semibold text-gray-800 mb-4">"Aksi"</h3>
                                                <div class="space-y-2">
                                                    <For
                                                        each=move || transitions.clone()
                                                        key=|t| t.status.clone()
                                                        children=move |transition| {
                                                            view! {
                                                                <button
                                                                    class="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                                                                    on:click=move |_| {
                                                                        set_selected_transition.set(Some(transition.status.clone()));
                                                                        set_show_transition_modal.set(true);
                                                                    }
                                                                >
                                                                    {transition.label.clone()}
                                                                </button>
                                                            }
                                                        }
                                                    />

                                                    // Revoke button (if active)
                                                    <Show when=move || izin.status == "ACTIVE">
                                                        <button
                                                            class="w-full px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
                                                            on:click=move |_| set_show_revoke_modal.set(true)
                                                        >
                                                            <i class="fas fa-ban mr-2"></i>
                                                            "Cabut Izin"
                                                        </button>
                                                    </Show>

                                                    // Renew button (if active and expiring soon)
                                                    <Show when=move || izin.status == "ACTIVE" && detail.is_expiring_soon>
                                                        <a
                                                            href={format!("/dashboard/pemakaian-bmn/{}/renew", izin.id)}
                                                            class="block w-full px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors text-center"
                                                        >
                                                            <i class="fas fa-redo mr-2"></i>
                                                            "Perpanjang Izin"
                                                        </a>
                                                    </Show>
                                                </div>
                                            </div>
                                        </Show>
                                    </div>
                                </div>

                                // Transition Modal
                                <Show when=move || show_transition_modal.get()>
                                    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                                        <div class="bg-white rounded-lg p-6 max-w-md w-full mx-4">
                                            <h3 class="text-lg font-semibold mb-4">"Konfirmasi Perubahan Status"</h3>
                                            <div class="mb-4">
                                                <label class="block text-sm font-medium text-gray-700 mb-1">"Catatan (opsional)"</label>
                                                <textarea
                                                    class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                                    rows="3"
                                                    prop:value=move || transition_comment.get()
                                                    on:input=move |ev| set_transition_comment.set(event_target_value(&ev))
                                                ></textarea>
                                            </div>
                                            <div class="flex gap-3">
                                                <button
                                                    class="flex-1 px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors"
                                                    on:click=move |_| set_show_transition_modal.set(false)
                                                    prop:disabled=move || loading.get()
                                                >
                                                    "Batal"
                                                </button>
                                                <button
                                                    class="flex-1 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50"
                                                    on:click=handle_transition
                                                    prop:disabled=move || loading.get()
                                                >
                                                    <Show when=move || loading.get() fallback=|| view! { "Konfirmasi" }>
                                                        <i class="fas fa-spinner fa-spin"></i>
                                                    </Show>
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                </Show>

                                // Revoke Modal
                                <Show when=move || show_revoke_modal.get()>
                                    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                                        <div class="bg-white rounded-lg p-6 max-w-md w-full mx-4">
                                            <h3 class="text-lg font-semibold mb-4">"Pencabutan Izin"</h3>
                                            <div class="mb-4">
                                                <label class="block text-sm font-medium text-gray-700 mb-1">"Alasan Pencabutan" <span class="text-red-500">"*"</span></label>
                                                <textarea
                                                    class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                                    rows="4"
                                                    placeholder="Minimal 10 karakter"
                                                    prop:value=move || revoke_reason.get()
                                                    on:input=move |ev| set_revoke_reason.set(event_target_value(&ev))
                                                    required
                                                ></textarea>
                                            </div>
                                            <div class="flex gap-3">
                                                <button
                                                    class="flex-1 px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors"
                                                    on:click=move |_| set_show_revoke_modal.set(false)
                                                    prop:disabled=move || loading.get()
                                                >
                                                    "Batal"
                                                </button>
                                                <button
                                                    class="flex-1 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors disabled:opacity-50"
                                                    on:click=handle_revoke
                                                    prop:disabled=move || loading.get() || revoke_reason.get().len() < 10
                                                >
                                                    <Show when=move || loading.get() fallback=|| view! { "Cabut Izin" }>
                                                        <i class="fas fa-spinner fa-spin"></i>
                                                    </Show>
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                </Show>
                            </div>
                        }.into_any()
                    }).unwrap_or_else(|| view! {
                        <div class="p-12 text-center">
                            <i class="fas fa-exclamation-circle text-5xl text-red-300 mb-4"></i>
                            <p class="text-gray-600 text-lg">"Data tidak ditemukan"</p>
                        </div>
                    }.into_any())
                }}
            </Suspense>
        </div>
    }
}
