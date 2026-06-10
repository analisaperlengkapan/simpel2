//! # SK Penghapusan BMN Generation Page (Validator Pusat)
//!
//! This page allows Validator Pusat to:
//! - Review penghapusan requests
//! - Generate SK Penghapusan draft (DOCX)
//! - Upload signed SK (PDF)
//!
//! Requirements: REQ-PH007, REQ-PH008, REQ-PH009, REQ-PH010

use leptos::prelude::*;
use leptos::task::spawn_local;
use lib_ui::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// API Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenghapusanRequest {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub satker_nama: Option<String>,
    pub status: String,
    pub alasan: String,
    pub metode_penghapusan: String,
    pub sk_number: Option<String>,
    pub sk_document_url: Option<String>,
    pub signed_sk_url: Option<String>,
    pub bmn_items: Vec<BmnItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmnItem {
    pub nup: String,
    pub nama_barang: String,
    pub kode_barang: String,
    pub kondisi: String,
    pub tahun_perolehan: Option<i32>,
    pub nilai_perolehan: Option<f64>,
    pub alasan_penghapusan: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateSKResponse {
    pub document_id: Uuid,
    pub document_url: String,
    pub sk_number: String,
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
pub fn SkGenerationPage() -> impl IntoView {
    let (requests, set_requests) = signal::<Vec<PenghapusanRequest>>(Vec::new());
    let (selected_request, set_selected_request) = signal::<Option<PenghapusanRequest>>(None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let (success_message, set_success_message) = signal::<Option<String>>(None);
    let (uploading, set_uploading) = signal(false);

    // Load requests on mount
    Effect::new(move || {
        spawn_local(async move {
            set_loading.set(true);
            match fetch_requests_for_sk().await {
                Ok(list) => set_requests.set(list),
                Err(e) => set_error.set(Some(e.to_string())),
            }
            set_loading.set(false);
        });
    });

    // Generate SK
    let generate_sk = move |request_id: Uuid| {
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            match generate_sk_document(request_id).await {
                Ok(response) => {
                    set_success_message.set(Some(
                        format!("SK Penghapusan berhasil dibuat dengan nomor {} (REQ-PH008, REQ-PH009, REQ-PH010)", response.sk_number)
                    ));

                    // Reload requests
                    if let Ok(list) = fetch_requests_for_sk().await {
                        let selected = list.iter().find(|r| r.id == request_id).cloned();
                        set_requests.set(list);
                        if let Some(req) = selected {
                            set_selected_request.set(Some(req));
                        }
                    }
                }
                Err(e) => set_error.set(Some(e.to_string())),
            }
            set_loading.set(false);
        });
    };

    // Upload signed SK
    let upload_signed_sk = move |request_id: Uuid, file: web_sys::File| {
        spawn_local(async move {
            set_uploading.set(true);
            set_error.set(None);

            match upload_signed_sk_document(request_id, file).await {
                Ok(_) => {
                    set_success_message.set(Some(
                        "SK Penghapusan yang sudah ditandatangani berhasil diupload (REQ-PH010)"
                            .to_string(),
                    ));

                    // Reload requests
                    if let Ok(list) = fetch_requests_for_sk().await {
                        let selected = list.iter().find(|r| r.id == request_id).cloned();
                        set_requests.set(list);
                        if let Some(req) = selected {
                            set_selected_request.set(Some(req));
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
                "Generate SK Penghapusan BMN - Validator Pusat"
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
                // Left Panel: Requests List
                <div class="lg:col-span-1">
                    <Card title="Daftar Permohonan">
                        <div class="space-y-2">
                            {move || {
                                let list = requests.get();
                                if list.is_empty() && !loading.get() {
                                    view! {
                                        <div class="text-center py-8 text-gray-500">
                                            "Tidak ada permohonan"
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="space-y-2">
                                            {list.into_iter().map(|req| {
                                                let req_id = req.id;
                                                view! {
                                                    <div
                                                        class="p-4 border rounded-lg cursor-pointer hover:bg-gray-50"
                                                        on:click=move |_| set_selected_request.set(Some(req.clone()))
                                                    >
                                                        <div class="font-semibold text-gray-900">
                                                            {req.satker_nama.clone().unwrap_or_else(|| "Unknown".to_string())}
                                                        </div>
                                                        <div class="text-sm text-gray-600 mt-1">
                                                            {req.bmn_items.len()} " BMN"
                                                        </div>
                                                        {req.sk_number.as_ref().map(|num| view! {
                                                            <div class="text-xs text-gray-500 mt-1 font-mono">
                                                                {num.clone()}
                                                            </div>
                                                        })}
                                                        <Badge
                                                            label=req.status.clone()
                                                            variant=match req.status.as_str() {
                                                                "COMPLETED" => BadgeVariant::Success,
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

                // Right Panel: SK Generation
                <div class="lg:col-span-2">
                    {move || {
                        if let Some(request) = selected_request.get() {
                            let request_bmn_items_for_summary = request.bmn_items.clone();
                            let request_bmn_items_for_table = request.bmn_items.clone();
                            let request_sk_document_url_for_generation = request.sk_document_url.clone();
                            let request_sk_document_url_for_upload = request.sk_document_url.clone();
                            let request_signed_sk_url_for_upload = request.signed_sk_url.clone();
                            let request_signed_sk_url_for_final = request.signed_sk_url.clone();
                            let total_nilai: f64 = request_bmn_items_for_table.iter()
                                .filter_map(|item| item.nilai_perolehan)
                                .sum();

                            view! {
                                <div class="space-y-6">
                                    // Request Summary
                                    <Card title="Ringkasan Permohonan">
                                        <div class="grid grid-cols-2 gap-4">
                                            <div>
                                                <label class="text-sm font-medium text-gray-700">"Satker"</label>
                                                <p class="text-gray-900">
                                                    {request.satker_nama.clone().unwrap_or_else(|| "Unknown".to_string())}
                                                </p>
                                            </div>
                                            <div>
                                                <label class="text-sm font-medium text-gray-700">"Jumlah BMN"</label>
                                                <p class="text-gray-900">{request_bmn_items_for_summary.len()} " item"</p>
                                            </div>
                                            <div>
                                                <label class="text-sm font-medium text-gray-700">"Total Nilai"</label>
                                                <p class="text-gray-900 font-semibold">
                                                    "Rp " {format!("{:.2}", total_nilai)}
                                                </p>
                                            </div>
                                            <div>
                                                <label class="text-sm font-medium text-gray-700">"Metode"</label>
                                                <p class="text-gray-900">{request.metode_penghapusan.clone()}</p>
                                            </div>
                                        </div>
                                    </Card>

                                    // BMN Items
                                    <Card title="Daftar BMN">
                                        <div class="overflow-x-auto">
                                            <table class="min-w-full divide-y divide-gray-200">
                                                <thead class="bg-gray-50">
                                                    <tr>
                                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"No"</th>
                                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"Nama Barang"</th>
                                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"NUP"</th>
                                                        <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase">"Nilai"</th>
                                                    </tr>
                                                </thead>
                                                <tbody class="bg-white divide-y divide-gray-200">
                                                    {request_bmn_items_for_table.iter().enumerate().map(|(idx, item)| view! {
                                                        <tr>
                                                            <td class="px-4 py-3 text-sm">{idx + 1}</td>
                                                            <td class="px-4 py-3 text-sm">{item.nama_barang.clone()}</td>
                                                            <td class="px-4 py-3 text-sm">{item.nup.clone()}</td>
                                                            <td class="px-4 py-3 text-sm text-right">
                                                                {item.nilai_perolehan.map(|n| format!("Rp {:.2}", n)).unwrap_or_else(|| "-".to_string())}
                                                            </td>
                                                        </tr>
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        </div>
                                    </Card>

                                    // SK Generation (REQ-PH008, REQ-PH009)
                                    <Card title="Dokumen SK Penghapusan">
                                        {if request_sk_document_url_for_generation.is_none() {
                                            view! {
                                                <div class="text-center py-8">
                                                    <p class="text-gray-600 mb-4">
                                                        "SK Penghapusan belum dibuat. Klik tombol di bawah untuk generate SK."
                                                    </p>
                                                    <p class="text-sm text-gray-500 mb-4">
                                                        "Format: DOCX dengan nomor SK otomatis (SK/YEAR/SEQUENCE) (REQ-PH010)"
                                                    </p>
                                                    <Button
                                                        variant=ButtonVariant::Primary
                                                        on_click=Box::new(move || generate_sk(request.id))
                                                        disabled=loading.get()
                                                    >
                                                        "Generate SK Penghapusan"
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
                                                                <p class="font-medium text-emerald-900">"SK Penghapusan Tersedia"</p>
                                                                {request.sk_number.as_ref().map(|num| view! {
                                                                    <p class="text-sm text-emerald-700 font-mono">{num.clone()}</p>
                                                                })}
                                                            </div>
                                                        </div>
                                                        <a
                                                            href=request_sk_document_url_for_generation.clone().unwrap()
                                                            target="_blank"
                                                            class="px-4 py-2 bg-emerald-600 text-white rounded-lg hover:bg-emerald-700"
                                                        >
                                                            "Download DOCX"
                                                        </a>
                                                    </div>
                                                </div>
                                            }.into_any()
                                        }}
                                    </Card>

                                    // Upload Signed SK (REQ-PH010)
                                    {if request_sk_document_url_for_upload.is_some() && request_signed_sk_url_for_upload.is_none() {
                                        view! {
                                            <Card title="Upload SK Bertanda Tangan">
                                                <div class="space-y-4">
                                                    <p class="text-sm text-gray-600">
                                                        "Upload SK Penghapusan yang sudah ditandatangani (format PDF)"
                                                    </p>
                                                    <FileUpload
                                                        label="SK PDF Bertanda Tangan"
                                                        accept=".pdf"
                                                        multiple=false
                                                        on_change=Box::new(move |files: Vec<web_sys::File>| {
                                                            if let Some(file) = files.into_iter().next() {
                                                                upload_signed_sk(request.id, file);
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
                                    } else if request_signed_sk_url_for_final.is_some() {
                                        view! {
                                            <Card title="SK Final">
                                                <div class="flex items-center justify-between p-4 bg-green-50 rounded-lg">
                                                    <div class="flex items-center space-x-3">
                                                        <svg class="w-8 h-8 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                                        </svg>
                                                        <div>
                                                            <p class="font-medium text-green-900">"SK Lengkap"</p>
                                                            <p class="text-sm text-green-700">"Status: COMPLETED"</p>
                                                        </div>
                                                    </div>
                                                    <a
                                                        href=request_signed_sk_url_for_final.clone().unwrap()
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
                                        "Pilih permohonan dari daftar"
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

async fn fetch_requests_for_sk() -> Result<Vec<PenghapusanRequest>, crate::api::AppError> {
    let response = gloo_net::http::Request::get("/api/v1/perlengkapan/penghapusan-bmn?status=REVIEWED")
        .send()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Network error: {}", e)))?;

    if !response.ok() {
        return Err(crate::api::AppError::Unknown(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let api_response: ApiResponse<Vec<PenghapusanRequest>> = response
        .json()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Parse error: {}", e)))?;

    Ok(api_response.data.unwrap_or_default())
}

async fn generate_sk_document(
    request_id: Uuid,
) -> Result<GenerateSKResponse, crate::api::AppError> {
    let url = format!("/api/v1/perlengkapan/penghapusan-bmn/{}/generate-sk", request_id);
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

    let api_response: ApiResponse<GenerateSKResponse> = response
        .json()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Parse error: {}", e)))?;

    api_response
        .data
        .ok_or_else(|| crate::api::AppError::Unknown("No data in response".to_string()))
}

async fn upload_signed_sk_document(
    request_id: Uuid,
    file: web_sys::File,
) -> Result<(), crate::api::AppError> {
    let form_data = web_sys::FormData::new().map_err(|_| "Failed to create FormData")?;
    form_data
        .append_with_blob("file", &file)
        .map_err(|_| "Failed to append file")?;

    let url = format!("/api/v1/perlengkapan/penghapusan-bmn/{}/upload-signed-sk", request_id);

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
