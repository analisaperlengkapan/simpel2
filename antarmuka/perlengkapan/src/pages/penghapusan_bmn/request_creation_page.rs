//! # Penghapusan BMN Request Creation Page
//!
//! This page allows Operator Satker to:
//! - Create SK Penghapusan BMN request
//! - Select BMN items for deletion
//! - Upload supporting documents
//! - Submit request to Validator Wilayah
//!
//! Requirements: REQ-PH001 to REQ-PH005, REQ-PH019

use leptos::prelude::*;
use leptos::task::spawn_local;
use lib_ui::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::features::auth::AuthService;

// ============================================================================
// API Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePenghapusanRequest {
    pub satker_id: Uuid,
    pub alasan: String,
    pub metode_penghapusan: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmnItem {
    pub nup: String,
    pub nama_barang: String,
    pub kode_barang: String,
    pub kondisi: String,
    pub tahun_perolehan: Option<i32>,
    pub nilai_perolehan: Option<f64>,
    pub has_active_permit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddBmnRequest {
    pub nup: String,
    pub nama_barang: String,
    pub kode_barang: String,
    pub alasan_penghapusan: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenghapusanRequest {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub status: String,
    pub bmn_items: Vec<BmnItem>,
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
pub fn RequestCreationPage() -> impl IntoView {
    let (available_bmn, set_available_bmn) = signal::<Vec<BmnItem>>(Vec::new());
    let (selected_bmn, set_selected_bmn) = signal::<Vec<BmnItem>>(Vec::new());
    let (alasan_umum, set_alasan_umum) = signal(String::new());
    let (metode, set_metode) = signal(String::from("DIMUSNAHKAN"));
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let (success_message, set_success_message) = signal::<Option<String>>(None);
    let (uploaded_files, set_uploaded_files) = signal::<Vec<web_sys::File>>(Vec::new());

    // Load available BMN on mount
    Effect::new(move || {
        spawn_local(async move {
            set_loading.set(true);
            match fetch_available_bmn().await {
                Ok(list) => set_available_bmn.set(list),
                Err(e) => set_error.set(Some(e.to_string())),
            }
            set_loading.set(false);
        });
    });

    // Add BMN to selection
    let add_bmn = move |bmn: BmnItem| {
        let mut current = selected_bmn.get();
        if !current.iter().any(|b| b.nup == bmn.nup) {
            current.push(bmn);
            set_selected_bmn.set(current);
        }
    };

    // Remove BMN from selection
    let remove_bmn = move |nup: String| {
        let current = selected_bmn.get();
        let filtered: Vec<BmnItem> = current.into_iter().filter(|b| b.nup != nup).collect();
        set_selected_bmn.set(filtered);
    };

    // Create request
    let create_request = move || {
        if selected_bmn.get().is_empty() {
            set_error.set(Some(
                "Pilih minimal 1 BMN untuk dihapus (REQ-PH002)".to_string(),
            ));
            return;
        }

        let alasan = alasan_umum.get();
        if alasan.trim().is_empty() {
            set_error.set(Some("Alasan penghapusan harus diisi".to_string()));
            return;
        }

        if uploaded_files.get().is_empty() {
            set_error.set(Some(
                "Upload minimal 1 dokumen pendukung (REQ-PH003)".to_string(),
            ));
            return;
        }

        // Project satker_id from the JWT-backed session. Refuse the
        // submission with a user-facing message if the claim is missing —
        // the backend will reject the request anyway, so failing early on
        // the client gives a cleaner UX.
        let satker_id = match AuthService::load_session()
            .and_then(|s| s.satker_id)
            .and_then(|s| Uuid::parse_str(&s).ok())
        {
            Some(id) => id,
            None => {
                set_error.set(Some(
                    "Anda belum terdaftar di satker manapun (satker_id kosong). \
                     Hubungi admin perlengkapan untuk pemetaan satker."
                        .to_string(),
                ));
                return;
            }
        };

        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            let request = CreatePenghapusanRequest {
                satker_id,
                alasan: alasan.clone(),
                metode_penghapusan: metode.get(),
            };

            match create_penghapusan_request(request).await {
                Ok(penghapusan) => {
                    // Add BMN items
                    for bmn in selected_bmn.get() {
                        let _ = add_bmn_to_request(penghapusan.id, bmn).await;
                    }

                    // Upload documents
                    for file in uploaded_files.get() {
                        let _ = upload_document(penghapusan.id, file).await;
                    }

                    set_success_message.set(Some(
                        "Permohonan penghapusan BMN berhasil dibuat (REQ-PH001)".to_string(),
                    ));
                }
                Err(e) => set_error.set(Some(e.to_string())),
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="container mx-auto px-4 py-8">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-6">
                "Buat Permohonan SK Penghapusan BMN"
            </h1>

            {move || error.get().map(|e| view! { <Alert message=e variant=AlertVariant::Error /> })}

            {move || {
                success_message
                    .get()
                    .map(|msg| view! { <Alert message=msg variant=AlertVariant::Success /> })
            }}

            {move || {
                loading
                    .get()
                    .then(|| {
                        view! {
                            <div class="flex justify-center py-4">
                                <Spinner size="lg" />
                            </div>
                        }
                    })
            }}

            <div class="space-y-6">
                // Step 1: Select BMN Items
                <Card title="1. Pilih BMN yang Akan Dihapus (REQ-PH002)">
                    <p class="text-sm text-gray-600 mb-4">
                        "Sistem akan memvalidasi bahwa BMN tidak sedang digunakan (REQ-PH019)"
                    </p>

                    <div class="space-y-4">
                        {move || {
                            available_bmn
                                .get()
                                .into_iter()
                                .map(|bmn| {
                                    let nup = bmn.nup.clone();
                                    let is_selected = selected_bmn
                                        .get()
                                        .iter()
                                        .any(|b| b.nup == nup);
                                    let bmn_for_click = bmn.clone();
                                    let bmn_has_active_permit = bmn.has_active_permit;

                                    view! {
                                        <div class="border rounded-lg p-4">
                                            <div class="flex justify-between items-start">
                                                <div class="flex-1">
                                                    <h4 class="font-semibold text-gray-900">
                                                        {bmn.nama_barang.clone()}
                                                    </h4>
                                                    <p class="text-sm text-gray-600">
                                                        "NUP: " {bmn.nup.clone()}
                                                    </p>
                                                    <p class="text-sm text-gray-600">
                                                        "Kode: " {bmn.kode_barang.clone()}
                                                    </p>
                                                    <p class="text-sm text-gray-600">
                                                        "Kondisi: " {bmn.kondisi.clone()}
                                                    </p>
                                                    {bmn
                                                        .nilai_perolehan
                                                        .map(|nilai| {
                                                            view! {
                                                                <p class="text-sm text-gray-600">
                                                                    "Nilai: Rp " {format!("{:.2}", nilai)}
                                                                </p>
                                                            }
                                                        })}
                                                    {if bmn.has_active_permit {
                                                        view! {
                                                            <Badge
                                                                label="Sedang Digunakan - Tidak Dapat Dihapus"
                                                                variant=BadgeVariant::Danger
                                                            />
                                                        }
                                                            .into_any()
                                                    } else {
                                                        view! {
                                                            <Badge
                                                                label="Dapat Dihapus"
                                                                variant=BadgeVariant::Success
                                                            />
                                                        }
                                                            .into_any()
                                                    }}
                                                </div>
                                                <Button
                                                    variant=if is_selected {
                                                        ButtonVariant::Secondary
                                                    } else {
                                                        ButtonVariant::Primary
                                                    }
                                                    on_click=Box::new(move || {
                                                        if is_selected {
                                                            remove_bmn(nup.clone());
                                                        } else {
                                                            add_bmn(bmn_for_click.clone());
                                                        }
                                                    })
                                                    disabled=bmn_has_active_permit
                                                >
                                                    {if is_selected { "Hapus" } else { "Pilih" }}
                                                </Button>
                                            </div>
                                        </div>
                                    }
                                })
                                .collect_view()
                        }}
                    </div>

                    <div class="mt-4 p-4 bg-emerald-50 rounded-lg">
                        <p class="font-medium text-emerald-900">
                            "BMN Terpilih: " {move || selected_bmn.get().len()} " item"
                        </p>
                    </div>
                </Card>

                // Step 2: Reason and Method
                {move || {
                    (!selected_bmn.get().is_empty())
                        .then(|| {
                            view! {
                                <Card title="2. Alasan dan Metode Penghapusan">
                                    <div class="space-y-4">
                                        <Textarea
                                            label="Alasan Penghapusan"
                                            placeholder="Jelaskan alasan penghapusan BMN..."
                                            value=alasan_umum.get()
                                            on_input=Box::new(move |v| set_alasan_umum.set(v))
                                            rows=4
                                            required=true
                                        />

                                        <Select
                                            label="Metode Penghapusan"
                                            value=metode.get()
                                            on_change=Box::new(move |v: String| set_metode.set(v))
                                        >
                                            <option value="DIMUSNAHKAN">"Dimusnahkan"</option>
                                            <option value="DIJUAL">"Dijual"</option>
                                            <option value="DIHIBAHKAN">"Dihibahkan"</option>
                                        </Select>
                                    </div>
                                </Card>
                            }
                        })
                }}

                // Step 3: Upload Documents
                {move || {
                    (!selected_bmn.get().is_empty())
                        .then(|| {
                            view! {
                                <Card title="3. Dokumen Pendukung (REQ-PH003)">
                                    <p class="text-sm text-gray-600 mb-4">
                                        "Upload dokumen persyaratan penghapusan (minimal 1 dokumen)"
                                    </p>
                                    <FileUpload
                                        label="Dokumen Pendukung"
                                        accept=".pdf,.doc,.docx,.jpg,.jpeg,.png"
                                        multiple=true
                                        show_preview=true
                                        on_change=Box::new(move |files: Vec<web_sys::File>| {
                                            set_uploaded_files.set(files);
                                        })
                                        hint="Format: PDF, DOC, DOCX, JPG, PNG. Maksimal 10MB per file."
                                    />
                                </Card>
                            }
                        })
                }}

                // Submit Button
                {move || {
                    (!selected_bmn.get().is_empty() && !alasan_umum.get().is_empty())
                        .then(|| {
                            view! {
                                <Card>
                                    <div class="flex justify-end space-x-4">
                                        <Button
                                            variant=ButtonVariant::Secondary
                                            on_click=Box::new(move || {
                                                web_sys::window().unwrap().history().unwrap().back().ok();
                                            })
                                        >
                                            "Batal"
                                        </Button>
                                        <Button
                                            variant=ButtonVariant::Primary
                                            size=ButtonSize::Large
                                            on_click=Box::new(create_request)
                                            disabled=loading.get()
                                        >
                                            "Buat Permohonan"
                                        </Button>
                                    </div>
                                    <p class="text-sm text-gray-500 mt-4 text-right">
                                        "Permohonan akan dikirim ke Validator Wilayah untuk ditinjau (REQ-PH004)"
                                    </p>
                                </Card>
                            }
                        })
                }}
            </div>
        </div>
    }
}

// ============================================================================
// API Functions
// ============================================================================

async fn fetch_available_bmn() -> Result<Vec<BmnItem>, crate::api::AppError> {
    let response =
        gloo_net::http::Request::get("/api/v1/perlengkapan/penghapusan-bmn/bmn/available")
            .send()
            .await
            .map_err(|e| crate::api::AppError::Unknown(format!("Network error: {}", e)))?;

    if !response.ok() {
        return Err(crate::api::AppError::Unknown(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let api_response: ApiResponse<Vec<BmnItem>> = response
        .json()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Parse error: {}", e)))?;

    Ok(api_response.data.unwrap_or_default())
}

async fn create_penghapusan_request(
    request: CreatePenghapusanRequest,
) -> Result<PenghapusanRequest, crate::api::AppError> {
    let response = gloo_net::http::Request::post("/api/v1/perlengkapan/penghapusan-bmn")
        .json(&request)
        .map_err(|e| crate::api::AppError::Unknown(format!("Serialization error: {}", e)))?
        .send()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Network error: {}", e)))?;

    if !response.ok() {
        return Err(crate::api::AppError::Unknown(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let api_response: ApiResponse<PenghapusanRequest> = response
        .json()
        .await
        .map_err(|e| crate::api::AppError::Unknown(format!("Parse error: {}", e)))?;

    api_response
        .data
        .ok_or_else(|| crate::api::AppError::Unknown("No data in response".to_string()))
}

async fn add_bmn_to_request(request_id: Uuid, bmn: BmnItem) -> Result<(), crate::api::AppError> {
    let request = AddBmnRequest {
        nup: bmn.nup,
        nama_barang: bmn.nama_barang,
        kode_barang: bmn.kode_barang,
        alasan_penghapusan: "Sesuai alasan umum".to_string(),
    };

    let url = format!("/api/v1/perlengkapan/penghapusan-bmn/{}/bmn", request_id);
    let response = gloo_net::http::Request::post(&url)
        .json(&request)
        .map_err(|e| crate::api::AppError::Unknown(format!("Serialization error: {}", e)))?
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

async fn upload_document(
    request_id: Uuid,
    file: web_sys::File,
) -> Result<(), crate::api::AppError> {
    let form_data = web_sys::FormData::new().map_err(|_| "Failed to create FormData")?;
    form_data
        .append_with_blob("file", &file)
        .map_err(|_| "Failed to append file")?;

    let url = format!(
        "/api/v1/perlengkapan/penghapusan-bmn/{}/attachments",
        request_id
    );

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
