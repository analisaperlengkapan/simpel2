//! # Pemakaian BMN Document Management Page
//!
//! This page allows users to:
//! - Generate draft permit document (DOCX)
//! - Download draft permit
//! - Upload signed permit (PDF)
//! - View permit status
//!
//! Requirements: REQ-P006, REQ-P007, REQ-P008, REQ-P009, REQ-P010

use chrono::{DateTime, Utc};
use leptos::prelude::*;
use leptos::task::spawn_local;
use lib_ui::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// API Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permit {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub pegawai_nip: String,
    pub pegawai_nama: String,
    pub permit_number: Option<String>,
    pub status: String,
    pub tanggal_mulai: String,
    pub tanggal_selesai: String,
    pub document_id: Option<Uuid>,
    pub document_url: Option<String>,
    pub signed_document_url: Option<String>,
    pub bmn_items: Vec<BmnItem>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmnItem {
    pub nup: String,
    pub nama_barang: String,
    pub kondisi: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateDocumentResponse {
    pub document_id: Uuid,
    pub document_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

// ============================================================================
// Main Component
// ============================================================================

#[component]
pub fn DocumentManagementPage() -> impl IntoView {
    let (permits, set_permits) = signal::<Vec<Permit>>(Vec::new());
    let (selected_permit, set_selected_permit) = signal::<Option<Permit>>(None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let (success_message, set_success_message) = signal::<Option<String>>(None);
    let (uploading, set_uploading) = signal(false);

    // Load permits on mount
    Effect::new(move || {
        spawn_local(async move {
            set_loading.set(true);
            match fetch_permits().await {
                Ok(list) => set_permits.set(list),
                Err(e) => set_error.set(Some(e.to_string())),
            }
            set_loading.set(false);
        });
    });

    // Generate document
    let generate_document = move |permit_id: Uuid| {
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            match generate_permit_document(permit_id).await {
                Ok(response) => {
                    set_success_message.set(Some(
                        "Dokumen konsep izin berhasil dibuat (REQ-P006, REQ-P007)".to_string(),
                    ));

                    // Reload permits to get updated document_url
                    if let Ok(list) = fetch_permits().await {
                        let selected = list.iter().find(|p| p.id == permit_id).cloned();
                        set_permits.set(list);
                        if let Some(permit) = selected {
                            set_selected_permit.set(Some(permit));
                        }
                    }
                }
                Err(e) => set_error.set(Some(e.to_string())),
            }
            set_loading.set(false);
        });
    };

    // Upload signed document
    let upload_signed = move |permit_id: Uuid, file: web_sys::File| {
        spawn_local(async move {
            set_uploading.set(true);
            set_error.set(None);

            match upload_signed_document(permit_id, file).await {
                Ok(_) => {
                    set_success_message.set(Some(
                        "Dokumen izin yang sudah ditandatangani berhasil diupload (REQ-P008, REQ-P009)".to_string()
                    ));

                    // Reload permits
                    if let Ok(list) = fetch_permits().await {
                        let selected = list.iter().find(|p| p.id == permit_id).cloned();
                        set_permits.set(list);
                        if let Some(permit) = selected {
                            set_selected_permit.set(Some(permit));
                        }
                    }
                }
                Err(e) => set_error.set(Some(e.to_string())),
            }
            set_uploading.set(false);
        });
    };

    view! {
        <div class="container mx-auto px-4 py-8">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-6">
                "Manajemen Dokumen Izin Pemakaian"
            </h1>

            {move || error.get().map(|e| view! {
                <Alert message=e variant=AlertVariant::Error />
            })}

            {move || success_message.get().map(|msg| view! {
                <Alert message=msg variant=AlertVariant::Success />
            })}

            {move || loading.get().then(|| view! {
                <div class="flex justify-center py-4">
                    <Spinner size="lg" />
                </div>
            })}

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                // Left Panel: Permits List
                <div class="lg:col-span-1">
                    <Card title="Daftar Izin">
                        <div class="space-y-2">
                            {move || {
                                let list = permits.get();
                                if list.is_empty() && !loading.get() {
                                    view! {
                                        <div class="text-center py-8 text-gray-500">
                                            "Tidak ada izin pemakaian"
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="space-y-2">
                                            {list.into_iter().map(|permit| {
                                                let permit_id = permit.id;
                                                let is_selected = selected_permit.get()
                                                    .map(|p| p.id == permit_id)
                                                    .unwrap_or(false);

                                                view! {
                                                    <div
                                                        class=move || format!(
                                                            "p-4 border rounded-lg cursor-pointer transition-colors {}",
                                                            if is_selected {
                                                                "border-emerald-500 bg-emerald-50"
                                                            } else {
                                                                "border-gray-200 hover:bg-gray-50"
                                                            }
                                                        )
                                                        on:click=move |_| set_selected_permit.set(Some(permit.clone()))
                                                    >
                                                        <div class="font-semibold text-gray-900">
                                                            {permit.pegawai_nama.clone()}
                                                        </div>
                                                        <div class="text-sm text-gray-600 mt-1">
                                                            "NIP: " {permit.pegawai_nip.clone()}
                                                        </div>
                                                        {permit.permit_number.as_ref().map(|num| view! {
                                                            <div class="text-sm text-gray-600 mt-1">
                                                                "No. Izin: " {num.clone()}
                                                            </div>
                                                        })}
                                                        <Badge
                                                            label=permit.status.clone()
                                                            variant=match permit.status.as_str() {
                                                                "COMPLETED" => BadgeVariant::Success,
                                                                "DRAFT" => BadgeVariant::Warning,
                                                                _ => BadgeVariant::Info,
                                                            }
                                                        />
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>
                    </Card>
                </div>

                // Right Panel: Document Management
                <div class="lg:col-span-2">
                    {move || {
                        if let Some(permit) = selected_permit.get() {
                            let permit_document_url_for_generation = permit.document_url.clone();
                            let permit_document_url_for_upload = permit.document_url.clone();
                            let permit_signed_document_url_for_upload = permit.signed_document_url.clone();
                            let permit_signed_document_url_for_final = permit.signed_document_url.clone();
                            view! {
                                <div class="space-y-6">
                                    // Permit Information
                                    <Card title="Informasi Izin">
                                        <div class="grid grid-cols-2 gap-4">
                                            <div>
                                                <label class="text-sm font-medium text-gray-700">"Pegawai"</label>
                                                <p class="text-gray-900">{permit.pegawai_nama.clone()}</p>
                                                <p class="text-sm text-gray-600">"NIP: " {permit.pegawai_nip.clone()}</p>
                                            </div>
                                            <div>
                                                <label class="text-sm font-medium text-gray-700">"Status"</label>
                                                <Badge
                                                    label=permit.status.clone()
                                                    variant=match permit.status.as_str() {
                                                        "COMPLETED" => BadgeVariant::Success,
                                                        "DRAFT" => BadgeVariant::Warning,
                                                        _ => BadgeVariant::Info,
                                                    }
                                                />
                                            </div>
                                            <div>
                                                <label class="text-sm font-medium text-gray-700">"Periode"</label>
                                                <p class="text-gray-900">
                                                    {permit.tanggal_mulai.clone()} " s/d " {permit.tanggal_selesai.clone()}
                                                </p>
                                            </div>
                                            {permit.permit_number.as_ref().map(|num| view! {
                                                <div>
                                                    <label class="text-sm font-medium text-gray-700">"Nomor Izin (REQ-P010)"</label>
                                                    <p class="text-gray-900 font-mono">{num.clone()}</p>
                                                </div>
                                            })}
                                        </div>
                                    </Card>

                                    // BMN Items
                                    <Card title="Daftar BMN">
                                        <div class="space-y-2">
                                            {permit.bmn_items.iter().enumerate().map(|(idx, item)| view! {
                                                <div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                                                    <div>
                                                        <p class="font-medium text-gray-900">
                                                            {idx + 1} ". " {item.nama_barang.clone()}
                                                        </p>
                                                        <p class="text-sm text-gray-600">
                                                            "NUP: " {item.nup.clone()} " | Kondisi: " {item.kondisi.clone()}
                                                        </p>
                                                    </div>
                                                </div>
                                            }).collect_view()}
                                        </div>
                                    </Card>

                                    // Document Generation (REQ-P006, REQ-P007)
                                    <Card title="Dokumen Konsep Izin">
                                        {if permit_document_url_for_generation.is_none() {
                                            view! {
                                                <div class="text-center py-8">
                                                    <p class="text-gray-600 mb-4">
                                                        "Dokumen konsep belum dibuat. Klik tombol di bawah untuk generate dokumen."
                                                    </p>
                                                    <p class="text-sm text-gray-500 mb-4">
                                                        "Format: DOCX dengan foto pegawai (halaman 1) dan tabel BMN (halaman 2+)"
                                                    </p>
                                                    <Button
                                                        variant=ButtonVariant::Primary
                                                        on_click=Box::new(move || generate_document(permit.id))
                                                        disabled=loading.get()
                                                    >
                                                        "Generate Dokumen Konsep"
                                                    </Button>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="space-y-4">
                                                    <div class="flex items-center justify-between p-4 bg-emerald-50 rounded-lg">
                                                        <div class="flex items-center space-x-3">
                                                            <svg class="w-8 h-8 text-emerald-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                                            </svg>
                                                            <div>
                                                                <p class="font-medium text-emerald-900">"Dokumen Konsep Tersedia"</p>
                                                                <p class="text-sm text-emerald-700">"Format: DOCX"</p>
                                                            </div>
                                                        </div>
                                                        <a
                                                            href=permit_document_url_for_generation.clone().unwrap()
                                                            target="_blank"
                                                            class="px-4 py-2 bg-emerald-600 text-white rounded-lg hover:bg-emerald-700"
                                                        >
                                                            "Download"
                                                        </a>
                                                    </div>
                                                    <p class="text-sm text-gray-600">
                                                        "Silakan download dokumen, minta tanda tangan Pimpinan Satker, lalu upload kembali dalam format PDF."
                                                    </p>
                                                </div>
                                            }.into_any()
                                        }}
                                    </Card>

                                    // Upload Signed Document (REQ-P008, REQ-P009)
                                    {if permit_document_url_for_upload.is_some() && permit_signed_document_url_for_upload.is_none() {
                                        view! {
                                            <Card title="Upload Dokumen Bertanda Tangan">
                                                <div class="space-y-4">
                                                    <p class="text-sm text-gray-600">
                                                        "Upload dokumen izin yang sudah ditandatangani oleh Pimpinan Satker (format PDF)"
                                                    </p>
                                                    <FileUpload
                                                        label="Dokumen PDF Bertanda Tangan"
                                                        accept=".pdf"
                                                        multiple=false
                                                        on_change=Box::new(move |files: Vec<web_sys::File>| {
                                                            if let Some(file) = files.into_iter().next() {
                                                                upload_signed(permit.id, file);
                                                            }
                                                        })
                                                    />
                                                    {uploading.get().then(|| view! {
                                                        <div class="flex items-center space-x-2 text-emerald-600">
                                                            <Spinner size="sm" />
                                                            <span>"Uploading..."</span>
                                                        </div>
                                                    })}
                                                </div>
                                            </Card>
                                        }.into_any()
                                    } else if permit_signed_document_url_for_final.is_some() {
                                        view! {
                                            <Card title="Dokumen Final">
                                                <div class="flex items-center justify-between p-4 bg-green-50 rounded-lg">
                                                    <div class="flex items-center space-x-3">
                                                        <svg class="w-8 h-8 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                                        </svg>
                                                        <div>
                                                            <p class="font-medium text-green-900">"Dokumen Lengkap"</p>
                                                            <p class="text-sm text-green-700">"Status: COMPLETED (REQ-P009)"</p>
                                                        </div>
                                                    </div>
                                                    <a
                                                        href=permit_signed_document_url_for_final.clone().unwrap()
                                                        target="_blank"
                                                        class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700"
                                                    >
                                                        "Download PDF"
                                                    </a>
                                                </div>
                                            </Card>
                                        }.into_any()
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <Card>
                                    <div class="text-center py-12 text-gray-500">
                                        "Pilih izin dari daftar untuk mengelola dokumen"
                                    </div>
                                </Card>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// API Functions
// ============================================================================

async fn fetch_permits() -> Result<Vec<Permit>, crate::api::AppError> {
    let response = gloo_net::http::Request::get("/api/v1/perlengkapan/pemakaian-bmn")
        .send()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Network error: {}", e)))?;

    if !response.ok() {
        return Err(crate::api::AppError::Unknown(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let api_response: ApiResponse<Vec<Permit>> = response
        .json()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Parse error: {}", e)))?;

    Ok(api_response.data.unwrap_or_default())
}

async fn generate_permit_document(
    permit_id: Uuid,
) -> Result<GenerateDocumentResponse, crate::api::AppError> {
    let url = format!("/api/v1/perlengkapan/pemakaian-bmn/{}/generate-konsep-surat", permit_id);
    let response = gloo_net::http::Request::post(&url)
        .send()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Network error: {}", e)))?;

    if !response.ok() {
        return Err(crate::api::AppError::Unknown(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let api_response: ApiResponse<GenerateDocumentResponse> = response
        .json()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Parse error: {}", e)))?;

    api_response
        .data
        .ok_or_else(|| crate::api::AppError::Unknown("No data in response".to_string()))
}

async fn upload_signed_document(
    permit_id: Uuid,
    file: web_sys::File,
) -> Result<(), crate::api::AppError> {
    let form_data = web_sys::FormData::new().map_err(|_| "Failed to create FormData")?;
    form_data
        .append_with_blob("file", &file)
        .map_err(|_| "Failed to append file")?;

    let url = format!("/api/v1/perlengkapan/pemakaian-bmn/{}/upload-signed-pdf", permit_id);

    let response = gloo_net::http::Request::post(&url)
        .body(form_data)
        .map_err(|e| crate::api::AppError::Unknown(format!("Request error: {}", e)))?
        .send()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Network error: {}", e)))?;

    if !response.ok() {
        return Err(crate::api::AppError::Unknown(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    Ok(())
}
