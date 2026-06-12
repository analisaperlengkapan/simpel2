//! # Pemakaian BMN Detail Component
//!
//! Detail view for BMN usage permit with workflow actions.
//! Requirements: REQ-P001, REQ-P004, REQ-P008, REQ-P009

use crate::api::{
    GenerateKonsepSuratRequest, IzinPemakaianDetailResponse, PemakaianWorkflowTransitionRequest,
    RevokePermitRequest, UploadSignedPdfRequest, fetch_pemakaian_bmn_detail,
    generate_pemakaian_konsep_surat, revoke_pemakaian_bmn, transition_pemakaian_bmn_status,
    upload_pemakaian_signed_pdf,
};
use crate::components::workflow_ui::{ActionTone, ApprovalDialog};
use crate::routes;
use leptos::prelude::*;
use leptos_fetch::QueryClient;
use leptos_router::hooks::use_params_map;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{
    ARROW_CLOCKWISE, ARROW_LEFT, CHECK_CIRCLE, DOWNLOAD_SIMPLE, FILE_DOC, FILE_PDF, GEAR, PROHIBIT,
    SPINNER, UPLOAD_SIMPLE, WARNING, WARNING_CIRCLE,
};

/// leptos-fetch query — keyed by `(permit_id, refresh_trigger)`.
///
/// `refresh_trigger` is folded into the cache key so post-mutation
/// code (workflow transition, revoke, generate konsep, upload signed
/// PDF) can force a real network re-fetch by bumping the trigger.
/// Calling `.refetch()` on a leptos-fetch resource with an unchanged
/// keyer would just return the stale cached value.
async fn query_pemakaian_bmn_detail(key: (String, i32)) -> Option<IzinPemakaianDetailResponse> {
    let (permit_id, _trigger) = key;
    fetch_pemakaian_bmn_detail(&permit_id)
        .await
        .ok()
        .map(|response| response.data)
}

#[component]
pub fn PemakaianBmnDetail() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.read().get("id").unwrap_or_default();

    // Bumped after every successful mutation so the keyer below
    // emits a fresh cache slot and leptos-fetch issues a real
    // network request instead of serving stale data.
    let refresh_trigger = RwSignal::new(0i32);

    // Resource to fetch permit detail — keyed cache + dedup so
    // navigating away and back to the same permit re-uses the load.
    let client: QueryClient = expect_context();
    let permit_resource = client.local_resource(query_pemakaian_bmn_detail, move || {
        (id(), refresh_trigger.get())
    });

    // UI state
    let (show_transition_modal, set_show_transition_modal) = signal(false);
    let (selected_transition, set_selected_transition) = signal(None::<String>);
    let (transition_comment, set_transition_comment) = signal("".to_string());
    let (show_revoke_modal, set_show_revoke_modal) = signal(false);
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
                    catatan: if comment.is_empty() {
                        None
                    } else {
                        Some(comment)
                    },
                };

                match transition_pemakaian_bmn_status(&permit_id, request).await {
                    Ok(_) => {
                        set_show_transition_modal.set(false);
                        set_transition_comment.set("".to_string());
                        refresh_trigger.update(|v| *v += 1);
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Gagal mengubah status: {}", e)));
                    }
                }
                set_loading.set(false);
            });
        }
    };

    // Handle revocation — alasan dipasok oleh ApprovalDialog (min 10 char).
    let handle_revoke = Callback::new(move |reason: String| {
        set_loading.set(true);
        set_error.set(None);

        let permit_id = id();

        leptos::task::spawn_local(async move {
            let request = RevokePermitRequest { alasan: reason };

            match revoke_pemakaian_bmn(&permit_id, request).await {
                Ok(_) => {
                    set_show_revoke_modal.set(false);
                    refresh_trigger.update(|v| *v += 1);
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal mencabut izin: {}", e)));
                }
            }
            set_loading.set(false);
        });
    });

    // Konsep surat & signed PDF state
    let (show_upload_modal, set_show_upload_modal) = signal(false);
    let (signed_pdf_url, set_signed_pdf_url) = signal(String::new());
    let (generating_konsep, set_generating_konsep) = signal(false);
    let (uploading_pdf, set_uploading_pdf) = signal(false);

    // Handle generate konsep surat
    let handle_generate_konsep = move |_| {
        set_generating_konsep.set(true);
        set_error.set(None);
        let permit_id = id();

        leptos::task::spawn_local(async move {
            let request = GenerateKonsepSuratRequest { format: None };
            match generate_pemakaian_konsep_surat(&permit_id, request).await {
                Ok(_) => {
                    refresh_trigger.update(|v| *v += 1);
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal generate konsep surat: {}", e)));
                }
            }
            set_generating_konsep.set(false);
        });
    };

    // Handle upload signed PDF
    let handle_upload_signed_pdf = move |_| {
        let url = signed_pdf_url.get();
        if url.is_empty() {
            set_error.set(Some("URL file PDF harus diisi".to_string()));
            return;
        }

        set_uploading_pdf.set(true);
        set_error.set(None);
        let permit_id = id();

        leptos::task::spawn_local(async move {
            let request = UploadSignedPdfRequest {
                signed_pdf_url: url,
            };
            match upload_pemakaian_signed_pdf(&permit_id, request).await {
                Ok(_) => {
                    set_show_upload_modal.set(false);
                    set_signed_pdf_url.set(String::new());
                    refresh_trigger.update(|v| *v += 1);
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal upload PDF: {}", e)));
                }
            }
            set_uploading_pdf.set(false);
        });
    };

    view! {
        <div class="p-6 space-y-6">
            <Suspense fallback=move || {
                view! {
                    <div class="p-8 text-center">
                        <span class="fa-spin text-2xl text-gray-400 mb-2">
                            <AppIcon icon=SPINNER />
                        </span>
                        <p class="text-gray-600">"Memuat data..."</p>
                    </div>
                }
            }>
                {move || {
                    permit_resource
                        .get()
                        .flatten()
                        .map(|detail: IzinPemakaianDetailResponse| {
                            let izin = detail.izin.clone();
                            let transitions = detail.allowed_transitions.clone();
                            let bmn_items_empty = izin.bmn_items.is_empty();
                            let bmn_items_len = izin.bmn_items.len();
                            let bmn_items_data = StoredValue::new(izin.bmn_items.clone());
                            let transitions_empty = transitions.is_empty();
                            let transitions_data = StoredValue::new(transitions.clone());
                            let izin_status_is_active = izin.status == "ACTIVE";
                            let izin_id_for_renew = StoredValue::new(izin.id.clone());
                            let is_expiring_soon = detail.is_expiring_soon;

                            view! {
                                <div>
                                    // Header
                                    <div class="flex items-center justify-between mb-6">
                                        <div>
                                            <h2 class="text-2xl font-bold text-gray-800">
                                                "Detail Izin Pemakaian BMN"
                                            </h2>
                                            <p class="text-sm text-gray-600 mt-1">
                                                {izin
                                                    .nomor_izin
                                                    .clone()
                                                    .unwrap_or_else(|| "Belum ada nomor izin".to_string())}
                                            </p>
                                        </div>
                                        <a
                                            href=routes::path::PEMAKAIAN_DAFTAR_LEGACY
                                            class="px-4 py-2 text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition-colors"
                                        >
                                            <span class="mr-2">
                                                <AppIcon icon=ARROW_LEFT />
                                            </span>
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
                                            <AppIcon icon=WARNING />
                                            <span>
                                                "Izin akan berakhir dalam "
                                                {detail.days_until_expiry.unwrap_or(0)} " hari"
                                            </span>
                                        </div>
                                    </Show>

                                    // Main content
                                    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                                        // Left column - Details
                                        <div class="lg:col-span-2 space-y-6">
                                            // Pemohon Information
                                            <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-100">
                                                <h3 class="text-lg font-semibold text-gray-800 mb-4">
                                                    "Informasi Pemohon"
                                                </h3>
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
                                                        <p class="font-medium">
                                                            {izin.pegawai_satker_nama.clone()}
                                                        </p>
                                                    </div>
                                                    {izin
                                                        .pegawai_jabatan
                                                        .clone()
                                                        .map(|v| {
                                                            view! {
                                                                <div>
                                                                    <p class="text-sm text-gray-600">"Jabatan"</p>
                                                                    <p class="font-medium">{v}</p>
                                                                </div>
                                                            }
                                                        })}
                                                    {izin
                                                        .pegawai_golongan
                                                        .clone()
                                                        .map(|v| {
                                                            view! {
                                                                <div>
                                                                    <p class="text-sm text-gray-600">"Golongan"</p>
                                                                    <p class="font-medium">{v}</p>
                                                                </div>
                                                            }
                                                        })}
                                                    {izin
                                                        .pegawai_pangkat
                                                        .clone()
                                                        .map(|v| {
                                                            view! {
                                                                <div>
                                                                    <p class="text-sm text-gray-600">"Pangkat"</p>
                                                                    <p class="font-medium">{v}</p>
                                                                </div>
                                                            }
                                                        })}
                                                    {izin
                                                        .pegawai_unit_kerja
                                                        .clone()
                                                        .map(|v| {
                                                            view! {
                                                                <div>
                                                                    <p class="text-sm text-gray-600">"Unit Kerja"</p>
                                                                    <p class="font-medium">{v}</p>
                                                                </div>
                                                            }
                                                        })}
                                                </div>
                                            </div>

                                            // BMN Information
                                            <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-100">
                                                <h3 class="text-lg font-semibold text-gray-800 mb-4">
                                                    "Informasi BMN"
                                                </h3>
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
                                                    {izin
                                                        .no_polisi
                                                        .clone()
                                                        .map(|v| {
                                                            view! {
                                                                <div>
                                                                    <p class="text-sm text-gray-600">"Nomor Polisi"</p>
                                                                    <p class="font-medium">{v}</p>
                                                                </div>
                                                            }
                                                        })}

                                                    // Housing-specific fields
                                                    {izin
                                                        .alamat
                                                        .clone()
                                                        .map(|v| {
                                                            view! {
                                                                <div class="col-span-2">
                                                                    <p class="text-sm text-gray-600">"Alamat"</p>
                                                                    <p class="font-medium">{v}</p>
                                                                </div>
                                                            }
                                                        })}
                                                </div>
                                            </div>

                                            // Permit Details
                                            <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-100">
                                                <h3 class="text-lg font-semibold text-gray-800 mb-4">
                                                    "Detail Izin"
                                                </h3>
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
                                                    {izin
                                                        .lokasi_pemakaian
                                                        .clone()
                                                        .map(|v| {
                                                            view! {
                                                                <div>
                                                                    <p class="text-sm text-gray-600">"Lokasi Pemakaian"</p>
                                                                    <p class="font-medium">{v}</p>
                                                                </div>
                                                            }
                                                        })}
                                                </div>
                                            </div>

                                            // BMN Items (multi-BMN per pegawai)
                                            <Show when=move || !bmn_items_empty>
                                                <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-100">
                                                    <h3 class="text-lg font-semibold text-gray-800 mb-4">
                                                        "Daftar BMN yang Dipakai"
                                                        <span class="ml-2 text-sm font-normal text-gray-500">
                                                            {format!("({} item)", bmn_items_len)}
                                                        </span>
                                                    </h3>
                                                    <div class="overflow-x-auto">
                                                        <table class="w-full text-left border-collapse text-sm">
                                                            <thead>
                                                                <tr class="bg-gray-50 text-gray-600 text-xs uppercase">
                                                                    <th class="p-3 border-b">"Kode Barang"</th>
                                                                    <th class="p-3 border-b">"Nama Barang"</th>
                                                                    <th class="p-3 border-b">"NUP"</th>
                                                                    <th class="p-3 border-b text-center">"Merk"</th>
                                                                    <th class="p-3 border-b">"Kondisi"</th>
                                                                </tr>
                                                            </thead>
                                                            <tbody>
                                                                <For
                                                                    each=move || bmn_items_data.get_value()
                                                                    key=|item| item.id.clone()
                                                                    children=move |item| {
                                                                        view! {
                                                                            <tr class="border-b hover:bg-gray-50">
                                                                                <td class="p-3 font-mono text-xs">
                                                                                    {item.bmn_kode_barang.clone()}
                                                                                </td>
                                                                                <td class="p-3 font-medium">
                                                                                    {item.bmn_nama_barang.clone()}
                                                                                </td>
                                                                                <td class="p-3">{item.bmn_nup.clone()}</td>
                                                                                <td class="p-3 text-center">
                                                                                    {item.bmn_merk.clone().unwrap_or_default()}
                                                                                </td>
                                                                                <td class="p-3">
                                                                                    <span class={
                                                                                        let c = match item.bmn_kondisi.as_deref() {
                                                                                            Some("Baik") => "bg-green-100 text-green-800",
                                                                                            Some("Rusak Ringan") => "bg-yellow-100 text-yellow-800",
                                                                                            _ => "bg-gray-100 text-gray-700",
                                                                                        };
                                                                                        format!("px-2 py-0.5 rounded text-xs {}", c)
                                                                                    }>
                                                                                        {item.bmn_kondisi.clone().unwrap_or("-".to_string())}
                                                                                    </span>
                                                                                </td>
                                                                            </tr>
                                                                        }
                                                                    }
                                                                />
                                                            </tbody>
                                                        </table>
                                                    </div>
                                                </div>
                                            </Show>

                                            // Documents section (Konsep Surat & Signed PDF)
                                            <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-100">
                                                <h3 class="text-lg font-semibold text-gray-800 mb-4">
                                                    "Dokumen"
                                                </h3>
                                                <div class="space-y-3">
                                                    // Konsep surat — DOCX (editable) + PDF (final)
                                                    // produced side-by-side by the backend.
                                                    <div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                                                        <div class="flex items-center gap-3">
                                                            <span class="text-blue-500 text-lg">
                                                                <AppIcon icon=FILE_DOC />
                                                            </span>
                                                            <div>
                                                                <p class="font-medium text-sm">"Konsep Surat Izin"</p>
                                                                <p class="text-xs text-gray-500">
                                                                    {move || {
                                                                        izin.konsep_surat_generated_at
                                                                            .clone()
                                                                            .map(|d| format!("Digenerate: {}", d))
                                                                            .unwrap_or("Belum digenerate".to_string())
                                                                    }}
                                                                </p>
                                                            </div>
                                                        </div>
                                                        {move || {
                                                            let docx_url = izin.konsep_surat_url.clone();
                                                            let pdf_url = izin.konsep_surat_pdf_url.clone();
                                                            if docx_url.is_some() || pdf_url.is_some() {
                                                                view! {
                                                                    <div class="flex items-center gap-3">
                                                                        {docx_url
                                                                            .map(|u| {
                                                                                view! {
                                                                                    <a
                                                                                        href=u
                                                                                        target="_blank"
                                                                                        class="text-cyan-700 hover:text-cyan-900 text-sm flex items-center gap-1"
                                                                                    >
                                                                                        <AppIcon icon=DOWNLOAD_SIMPLE />
                                                                                        "DOCX"
                                                                                    </a>
                                                                                }
                                                                            })}
                                                                        {pdf_url
                                                                            .map(|u| {
                                                                                view! {
                                                                                    <a
                                                                                        href=u
                                                                                        target="_blank"
                                                                                        class="text-rose-700 hover:text-rose-900 text-sm flex items-center gap-1"
                                                                                    >
                                                                                        <AppIcon icon=DOWNLOAD_SIMPLE />
                                                                                        "PDF"
                                                                                    </a>
                                                                                }
                                                                            })}
                                                                    </div>
                                                                }
                                                                    .into_any()
                                                            } else if detail.can_generate_konsep.unwrap_or(false) {
                                                                view! {
                                                                    <button
                                                                        class="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-lg hover:bg-blue-700 disabled:opacity-50 flex items-center gap-1"
                                                                        on:click=handle_generate_konsep
                                                                        prop:disabled=move || generating_konsep.get()
                                                                    >
                                                                        <Show
                                                                            when=move || generating_konsep.get()
                                                                            fallback=|| view! { <AppIcon icon=GEAR /> }
                                                                        >
                                                                            <span class="fa-spin">
                                                                                <AppIcon icon=SPINNER />
                                                                            </span>
                                                                        </Show>
                                                                        "Generate"
                                                                    </button>
                                                                }
                                                                    .into_any()
                                                            } else {
                                                                view! {
                                                                    <span class="text-xs text-gray-400">"Tidak tersedia"</span>
                                                                }
                                                                    .into_any()
                                                            }
                                                        }}
                                                    </div>

                                                    // Signed PDF
                                                    <div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                                                        <div class="flex items-center gap-3">
                                                            <span class="text-red-500 text-lg">
                                                                <AppIcon icon=FILE_PDF />
                                                            </span>
                                                            <div>
                                                                <p class="font-medium text-sm">
                                                                    "Surat Izin Bertandatangan"
                                                                </p>
                                                                <p class="text-xs text-gray-500">
                                                                    {move || {
                                                                        izin.signed_pdf_uploaded_at
                                                                            .clone()
                                                                            .map(|d| format!("Diupload: {}", d))
                                                                            .unwrap_or("Belum diupload".to_string())
                                                                    }}
                                                                </p>
                                                            </div>
                                                        </div>
                                                        {move || {
                                                            if let Some(url) = izin.signed_pdf_url.clone() {
                                                                view! {
                                                                    <a
                                                                        href=url
                                                                        target="_blank"
                                                                        class="text-blue-600 hover:text-blue-800 text-sm flex items-center gap-1"
                                                                    >
                                                                        <AppIcon icon=DOWNLOAD_SIMPLE />
                                                                        "Download"
                                                                    </a>
                                                                }
                                                                    .into_any()
                                                            } else if detail.can_upload_signed_pdf.unwrap_or(false) {
                                                                view! {
                                                                    <button
                                                                        class="px-3 py-1.5 bg-green-600 text-white text-sm rounded-lg hover:bg-green-700 flex items-center gap-1"
                                                                        on:click=move |_| set_show_upload_modal.set(true)
                                                                    >
                                                                        <AppIcon icon=UPLOAD_SIMPLE />
                                                                        "Upload"
                                                                    </button>
                                                                }
                                                                    .into_any()
                                                            } else {
                                                                view! {
                                                                    <span class="text-xs text-gray-400">"Tidak tersedia"</span>
                                                                }
                                                                    .into_any()
                                                            }
                                                        }}
                                                    </div>

                                                    // Completed indicator
                                                    <Show when=move || izin.is_completed.unwrap_or(false)>
                                                        <div class="mt-2 p-3 bg-green-50 border border-green-200 rounded-lg flex items-center gap-2 text-green-700">
                                                            <AppIcon icon=CHECK_CIRCLE />
                                                            <span class="text-sm font-medium">
                                                                "Proses pemakaian BMN telah selesai"
                                                            </span>
                                                        </div>
                                                    </Show>
                                                </div>
                                            </div>
                                        </div>

                                        // Right column - Actions
                                        <div class="space-y-6">
                                            // Status card
                                            <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-100">
                                                <h3 class="text-lg font-semibold text-gray-800 mb-4">
                                                    "Status"
                                                </h3>
                                                <div class="text-center py-4">
                                                    <span class="px-4 py-2 rounded-full text-sm font-medium bg-blue-100 text-blue-700">
                                                        {izin.status.clone()}
                                                    </span>
                                                </div>
                                            </div>

                                            // Actions card
                                            <Show when=move || !transitions_empty>
                                                <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-100">
                                                    <h3 class="text-lg font-semibold text-gray-800 mb-4">
                                                        "Aksi"
                                                    </h3>
                                                    <div class="space-y-2">
                                                        <For
                                                            each=move || transitions_data.get_value()
                                                            key=|t| t.status.clone()
                                                            children=move |transition| {
                                                                view! {
                                                                    <button
                                                                        class="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                                                                        on:click=move |_| {
                                                                            set_selected_transition
                                                                                .set(Some(transition.status.clone()));
                                                                            set_show_transition_modal.set(true);
                                                                        }
                                                                    >
                                                                        {transition.label.clone()}
                                                                    </button>
                                                                }
                                                            }
                                                        />

                                                        // Revoke button (if active)
                                                        <Show when=move || izin_status_is_active>
                                                            <button
                                                                class="w-full px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
                                                                on:click=move |_| set_show_revoke_modal.set(true)
                                                            >
                                                                <span class="mr-2">
                                                                    <AppIcon icon=PROHIBIT />
                                                                </span>
                                                                "Cabut Izin"
                                                            </button>
                                                        </Show>

                                                        // Renew button (if active and expiring soon)
                                                        <Show when=move || {
                                                            izin_status_is_active && is_expiring_soon
                                                        }>
                                                            <a
                                                                href=format!(
                                                                    "/perlengkapan/pemakaian-bmn/{}/renew",
                                                                    izin_id_for_renew.get_value(),
                                                                )
                                                                class="block w-full px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors text-center"
                                                            >
                                                                <span class="mr-2">
                                                                    <AppIcon icon=ARROW_CLOCKWISE />
                                                                </span>
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
                                        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-modal">
                                            <div class="bg-white rounded-lg p-6 max-w-md w-full mx-4">
                                                <h3 class="text-lg font-semibold mb-4">
                                                    "Konfirmasi Perubahan Status"
                                                </h3>
                                                <div class="mb-4">
                                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                                        "Catatan (opsional)"
                                                    </label>
                                                    <textarea
                                                        class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                                        rows="3"
                                                        prop:value=move || transition_comment.get()
                                                        on:input=move |ev| {
                                                            set_transition_comment.set(event_target_value(&ev))
                                                        }
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
                                                        <Show
                                                            when=move || loading.get()
                                                            fallback=|| view! { "Konfirmasi" }
                                                        >
                                                            <span class="fa-spin">
                                                                <AppIcon icon=SPINNER />
                                                            </span>
                                                        </Show>
                                                    </button>
                                                </div>
                                            </div>
                                        </div>
                                    </Show>

                                    // Revoke Modal — ApprovalDialog reusable (Fase 2.5)
                                    <Show when=move || show_revoke_modal.get()>
                                        <ApprovalDialog
                                            title="Pencabutan Izin"
                                            description="Pencabutan bersifat final. Jelaskan alasan pencabutan izin pemakaian BMN ini."
                                            note_label="Alasan Pencabutan"
                                            note_placeholder="Minimal 10 karakter"
                                            require_note=true
                                            min_note_len=10
                                            confirm_label="Cabut Izin"
                                            confirm_tone=ActionTone::Danger
                                            on_confirm=handle_revoke
                                            on_close=Callback::new(move |_| {
                                                set_show_revoke_modal.set(false)
                                            })
                                            busy=Signal::derive(move || loading.get())
                                        />
                                    </Show>

                                    // Upload Signed PDF Modal
                                    <Show when=move || show_upload_modal.get()>
                                        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-modal">
                                            <div class="bg-white rounded-lg p-6 max-w-md w-full mx-4">
                                                <h3 class="text-lg font-semibold mb-4">
                                                    "Upload Surat Izin Bertandatangan"
                                                </h3>
                                                <div class="mb-4">
                                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                                        "URL File PDF" <span class="text-red-500">"*"</span>
                                                    </label>
                                                    <input
                                                        type="text"
                                                        class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                                        placeholder="https://... URL file PDF yang telah ditandatangani"
                                                        prop:value=move || signed_pdf_url.get()
                                                        on:input=move |ev| {
                                                            set_signed_pdf_url.set(event_target_value(&ev))
                                                        }
                                                    />
                                                    <p class="text-xs text-gray-500 mt-1">
                                                        "Upload file PDF terlebih dahulu, kemudian tempel URL-nya."
                                                    </p>
                                                </div>
                                                <div class="flex gap-3">
                                                    <button
                                                        class="flex-1 px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors"
                                                        on:click=move |_| set_show_upload_modal.set(false)
                                                        prop:disabled=move || uploading_pdf.get()
                                                    >
                                                        "Batal"
                                                    </button>
                                                    <button
                                                        class="flex-1 px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50"
                                                        on:click=handle_upload_signed_pdf
                                                        prop:disabled=move || {
                                                            uploading_pdf.get() || signed_pdf_url.get().is_empty()
                                                        }
                                                    >
                                                        <Show
                                                            when=move || uploading_pdf.get()
                                                            fallback=|| view! { "Upload" }
                                                        >
                                                            <span class="fa-spin">
                                                                <AppIcon icon=SPINNER />
                                                            </span>
                                                        </Show>
                                                    </button>
                                                </div>
                                            </div>
                                        </div>
                                    </Show>
                                </div>
                            }
                                .into_any()
                        })
                        .unwrap_or_else(|| {
                            view! {
                                <div class="p-12 text-center">
                                    <span class="text-5xl text-red-300 mb-4">
                                        <AppIcon icon=WARNING_CIRCLE />
                                    </span>
                                    <p class="text-gray-600 text-lg">"Data tidak ditemukan"</p>
                                </div>
                            }
                                .into_any()
                        })
                }}
            </Suspense>
        </div>
    }
}
