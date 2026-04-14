//! # Kebutuhan BMN Submission Form Page (Operator Satker)
//!
//! This page allows Operator Satker to:
//! - Submit kebutuhan BMN within period constraints
//! - Add BMN items with justification
//! - Upload supporting documents
//! - Submit to Validator Wilayah
//!
//! Requirements: REQ-K004 to REQ-K007

use leptos::prelude::*;
use leptos::task::spawn_local;
use lib_ui::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDate;

// ============================================================================
// API Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanSummary {
    pub id: Uuid,
    pub nama: String,
    pub tahun: i32,
    pub tgl_mulai: NaiveDate,
    pub tgl_selesai: NaiveDate,
    pub status_kode: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatkerDetail {
    pub id: Uuid,
    pub pengajuan_id: Uuid,
    pub ms_satker_id: String,
    pub status_kode: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarangItem {
    pub id: Option<Uuid>,
    pub nama: String,
    pub kode_barang: Option<String>,
    pub jumlah: i32,
    pub justifikasi: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBarangRequest {
    pub nama: String,
    pub kode_barang: Option<String>,
    pub jumlah: i32,
    pub justifikasi: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitKebutuhanSatkerRequest {
    pub catatan: Option<String>,
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
pub fn SubmissionFormPage() -> impl IntoView {
    // State management
    let (active_pengajuan, set_active_pengajuan) = signal::<Option<PengajuanSummary>>(None);
    let (satker_detail, set_satker_detail) = signal::<Option<SatkerDetail>>(None);
    let (barang_items, set_barang_items) = signal::<Vec<BarangItem>>(Vec::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let (success_message, set_success_message) = signal::<Option<String>>(None);

    // Form state for new barang
    let (nama_barang, set_nama_barang) = signal(String::new());
    let (kode_barang, set_kode_barang) = signal(String::new());
    let (jumlah, set_jumlah) = signal(1);
    let (justifikasi, set_justifikasi) = signal(String::new());
    let (uploaded_files, set_uploaded_files) = signal::<Vec<web_sys::File>>(Vec::new());

    // Load active pengajuan on mount
    Effect::new(move || {
        spawn_local(async move {
            set_loading.set(true);
            match fetch_active_pengajuan().await {
                Ok(pengajuan) => {
                    set_active_pengajuan.set(Some(pengajuan.clone()));
                    // Load satker detail
                    if let Ok(satker) = fetch_satker_detail(pengajuan.id).await {
                        set_satker_detail.set(Some(satker.clone()));
                        // Load existing barang items
                        if let Ok(items) = fetch_barang_items(satker.id).await {
                            set_barang_items.set(items);
                        }
                    }
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    // Add barang handler
    let add_barang = move || {
        let nama = nama_barang.get();
        let kode = kode_barang.get();
        let jml = jumlah.get();
        let just = justifikasi.get();

        // Validation
        if nama.trim().is_empty() {
            set_error.set(Some("Nama barang harus diisi".to_string()));
            return;
        }
        if just.trim().is_empty() {
            set_error.set(Some("Justifikasi harus diisi (REQ-K005)".to_string()));
            return;
        }
        if jml < 1 {
            set_error.set(Some("Jumlah harus lebih dari 0".to_string()));
            return;
        }

        let satker_id = match satker_detail.get() {
            Some(s) => s.id,
            None => {
                set_error.set(Some("Satker detail tidak ditemukan".to_string()));
                return;
            }
        };

        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            let request = CreateBarangRequest {
                nama: nama.clone(),
                kode_barang: if kode.is_empty() { None } else { Some(kode) },
                jumlah: jml,
                justifikasi: just.clone(),
            };

            match create_barang_item(satker_id, request).await {
                Ok(new_item) => {
                    // Add to list
                    let mut items = barang_items.get();
                    items.push(new_item);
                    set_barang_items.set(items);

                    // Clear form
                    set_nama_barang.set(String::new());
                    set_kode_barang.set(String::new());
                    set_jumlah.set(1);
                    set_justifikasi.set(String::new());

                    set_success_message.set(Some("Barang berhasil ditambahkan".to_string()));
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    };

    // Submit to Validator Wilayah handler
    let submit_to_wilayah = move || {
        let satker_id = match satker_detail.get() {
            Some(s) => s.id,
            None => {
                set_error.set(Some("Satker detail tidak ditemukan".to_string()));
                return;
            }
        };

        // Validation
        if barang_items.get().is_empty() {
            set_error.set(Some("Minimal harus ada 1 barang untuk diajukan".to_string()));
            return;
        }

        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            let request = SubmitKebutuhanSatkerRequest {
                catatan: Some("Pengajuan kebutuhan BMN dari Operator Satker".to_string()),
            };

            match submit_satker_to_wilayah(satker_id, request).await {
                Ok(_) => {
                    set_success_message.set(Some(
                        "Pengajuan berhasil disubmit ke Validator Wilayah (REQ-K007)".to_string(),
                    ));
                    // Reload satker detail to get updated status
                    if let Some(pengajuan) = active_pengajuan.get() {
                        if let Ok(satker) = fetch_satker_detail(pengajuan.id).await {
                            set_satker_detail.set(Some(satker));
                        }
                    }
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    };

    // File upload handler
    let handle_file_upload = move |files: Vec<web_sys::File>| {
        set_uploaded_files.set(files);
        // TODO: Upload files to server via POST /api/v1/kebutuhan-bmn/pengajuan/{id}/attachments
        // REQ-K006: Allow Operator Satker to upload supporting documents
    };

    view! {
        <div class="container mx-auto px-4 py-8">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-6">
                "Pengajuan Kebutuhan BMN"
            </h1>

            // Error/Success Messages
            {move || error.get().map(|e| view! {
                <Alert
                    message=e
                    variant=AlertVariant::Error
                />
            })}

            {move || success_message.get().map(|msg| view! {
                <Alert
                    message=msg
                    variant=AlertVariant::Success
                />
            })}

            // Loading State
            {move || loading.get().then(|| view! {
                <div class="flex justify-center items-center py-12">
                    <Spinner size="lg" />
                    <span class="ml-3 text-gray-600">"Memuat data..."</span>
                </div>
            })}

            // Main Content
            {move || {
                if let Some(pengajuan) = active_pengajuan.get() {
                    view! {
                        <div class="space-y-6">
                            // Period Information Card
                            <Card title="Informasi Periode">
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    <div>
                                        <label class="text-sm font-medium text-gray-700">"Nama Periode"</label>
                                        <p class="text-gray-900">{pengajuan.nama.clone()}</p>
                                    </div>
                                    <div>
                                        <label class="text-sm font-medium text-gray-700">"Tahun Anggaran"</label>
                                        <p class="text-gray-900">{pengajuan.tahun}</p>
                                    </div>
                                    <div>
                                        <label class="text-sm font-medium text-gray-700">"Tanggal Mulai"</label>
                                        <p class="text-gray-900">{pengajuan.tgl_mulai.to_string()}</p>
                                    </div>
                                    <div>
                                        <label class="text-sm font-medium text-gray-700">"Tanggal Selesai"</label>
                                        <p class="text-gray-900">{pengajuan.tgl_selesai.to_string()}</p>
                                    </div>
                                </div>

                                // Period constraint warning (REQ-K004, REQ-K017)
                                {move || {
                                    let today = chrono::Local::now().naive_local().date();
                                    if today > pengajuan.tgl_selesai {
                                        view! {
                                            <Alert
                                                message="Periode pengajuan telah berakhir. Anda tidak dapat menambah atau mengubah data."
                                                variant=AlertVariant::Warning
                                            />
                                        }.into_any()
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }
                                }}
                            </Card>

                            // Add Barang Form
                            <Card title="Tambah Barang Kebutuhan">
                                <div class="space-y-4">
                                    <Input
                                        label="Nama Barang"
                                        placeholder="Contoh: Laptop Dell Latitude 5420"
                                        value=nama_barang.get()
                                        on_input=Box::new(move |v| set_nama_barang.set(v))
                                        required=true
                                    />

                                    <Input
                                        label="Kode Barang (Opsional)"
                                        placeholder="Contoh: 3.02.01.01.001"
                                        value=kode_barang.get()
                                        on_input=Box::new(move |v| set_kode_barang.set(v))
                                        hint="Kode barang sesuai standar kodefikasi"
                                    />

                                    <Input
                                        label="Jumlah"
                                        input_type="number"
                                        value=jumlah.get().to_string()
                                        on_input=Box::new(move |v| {
                                            if let Ok(num) = v.parse::<i32>() {
                                                set_jumlah.set(num);
                                            }
                                        })
                                        required=true
                                    />

                                    <Textarea
                                        label="Justifikasi"
                                        placeholder="Jelaskan alasan kebutuhan barang ini (REQ-K005)"
                                        value=justifikasi.get()
                                        on_input=Box::new(move |v| set_justifikasi.set(v))
                                        rows=4
                                        required=true
                                    />

                                    <Button
                                        variant=ButtonVariant::Primary
                                        on_click=Box::new(add_barang)
                                        disabled=loading.get()
                                        full_width=true
                                    >
                                        "Tambah Barang"
                                    </Button>
                                </div>
                            </Card>

                            // Barang Items List
                            <Card>
                                <h2 class="text-xl font-semibold mb-4">
                                    "Daftar Barang Kebutuhan "
                                    <span class="text-emerald-600">
                                        "(" {move || barang_items.get().len()} " item)"
                                    </span>
                                </h2>
                                {move || {
                                    let items = barang_items.get();
                                    if items.is_empty() {
                                        view! {
                                            <div class="text-center py-8 text-gray-500">
                                                "Belum ada barang yang ditambahkan"
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="space-y-3">
                                                {items.into_iter().enumerate().map(|(idx, item)| view! {
                                                    <div class="border border-gray-200 rounded-lg p-4 hover:bg-gray-50">
                                                        <div class="flex justify-between items-start">
                                                            <div class="flex-1">
                                                                <div class="flex items-center space-x-2">
                                                                    <span class="font-semibold text-gray-700">
                                                                        {idx + 1} "."
                                                                    </span>
                                                                    <h3 class="font-semibold text-gray-900">
                                                                        {item.nama.clone()}
                                                                    </h3>
                                                                </div>
                                                                {item.kode_barang.as_ref().map(|kode| view! {
                                                                    <p class="text-sm text-gray-600 mt-1">
                                                                        "Kode: " {kode.clone()}
                                                                    </p>
                                                                })}
                                                                <p class="text-sm text-gray-600 mt-1">
                                                                    "Jumlah: " <span class="font-medium">{item.jumlah}</span>
                                                                </p>
                                                                <p class="text-sm text-gray-700 mt-2">
                                                                    <span class="font-medium">"Justifikasi: "</span>
                                                                    {item.justifikasi.clone()}
                                                                </p>
                                                            </div>
                                                        </div>
                                                    </div>
                                                }).collect_view()}
                                            </div>
                                        }.into_any()
                                    }
                                }}
                            </Card>

                            // Supporting Documents Upload (REQ-K006)
                            <Card title="Dokumen Pendukung">
                                <FileUpload
                                    label="Upload Surat Permohonan dan Lampiran"
                                    accept=".pdf,.doc,.docx,.jpg,.jpeg,.png"
                                    multiple=true
                                    show_preview=true
                                    on_change=Box::new(handle_file_upload)
                                    hint="Format: PDF, DOC, DOCX, JPG, PNG. Maksimal 10MB per file."
                                />
                            </Card>

                            // Submit Button
                            <Card>
                                <div class="flex justify-end space-x-4">
                                    <Button
                                        variant=ButtonVariant::Secondary
                                        on_click=Box::new(move || {
                                            // Navigate back
                                            web_sys::window()
                                                .unwrap()
                                                .history()
                                                .unwrap()
                                                .back()
                                                .ok();
                                        })
                                    >
                                        "Kembali"
                                    </Button>

                                    <Button
                                        variant=ButtonVariant::Primary
                                        size=ButtonSize::Large
                                        on_click=Box::new(submit_to_wilayah)
                                        disabled=loading.get() || barang_items.get().is_empty()
                                    >
                                        "Submit ke Validator Wilayah"
                                    </Button>
                                </div>

                                <p class="text-sm text-gray-500 mt-4 text-right">
                                    "Dengan menekan tombol submit, pengajuan akan dikirim ke Validator Wilayah untuk ditinjau (REQ-K007)"
                                </p>
                            </Card>
                        </div>
                    }.into_any()
                } else if !loading.get() {
                    view! {
                        <Alert
                            message="Tidak ada periode pengajuan yang aktif saat ini. Silakan hubungi Validator Pusat."
                            variant=AlertVariant::Warning
                        />
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>
    }
}

// ============================================================================
// API Functions
// ============================================================================

async fn fetch_active_pengajuan() -> Result<PengajuanSummary, String> {
    let response = gloo_net::http::Request::get("/api/v1/kebutuhan-bmn/pengajuan?status_kode=2001")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let api_response: ApiResponse<Vec<PengajuanSummary>> = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    api_response
        .data
        .and_then(|mut v| v.pop())
        .ok_or_else(|| "No active pengajuan found".to_string())
}

async fn fetch_satker_detail(pengajuan_id: Uuid) -> Result<SatkerDetail, String> {
    let url = format!("/api/v1/kebutuhan-bmn/pengajuan/{}/satker", pengajuan_id);
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let api_response: ApiResponse<Vec<SatkerDetail>> = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    api_response
        .data
        .and_then(|mut v| v.pop())
        .ok_or_else(|| "Satker detail not found".to_string())
}

async fn fetch_barang_items(satker_id: Uuid) -> Result<Vec<BarangItem>, String> {
    let url = format!("/api/v1/kebutuhan-bmn/satker/{}", satker_id);
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    // Parse response and extract barang items
    // Note: Actual response structure may differ, adjust as needed
    let api_response: ApiResponse<Vec<BarangItem>> = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(api_response.data.unwrap_or_default())
}

async fn create_barang_item(
    satker_id: Uuid,
    request: CreateBarangRequest,
) -> Result<BarangItem, String> {
    let url = format!("/api/v1/kebutuhan-bmn/satker/{}/barang", satker_id);
    let response = gloo_net::http::Request::post(&url)
        .json(&request)
        .map_err(|e| format!("Serialization error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let api_response: ApiResponse<BarangItem> = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    api_response
        .data
        .ok_or_else(|| "No data in response".to_string())
}

async fn submit_satker_to_wilayah(
    satker_id: Uuid,
    request: SubmitKebutuhanSatkerRequest,
) -> Result<SatkerDetail, String> {
    let url = format!("/api/v1/kebutuhan-bmn/satker/{}/submit-wilayah", satker_id);
    let response = gloo_net::http::Request::post(&url)
        .json(&request)
        .map_err(|e| format!("Serialization error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let api_response: ApiResponse<SatkerDetail> = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    api_response
        .data
        .ok_or_else(|| "No data in response".to_string())
}
