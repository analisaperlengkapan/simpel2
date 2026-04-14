//! # Kebutuhan BMN Review Page (Validator Wilayah)
//!
//! This page allows Validator Wilayah to:
//! - Review kebutuhan BMN submissions from Operator Satker
//! - View submission details, BMN items, and supporting documents
//! - View satker information and submission history
//! - Forward submissions to Validator Pusat
//! - Return submissions to Operator Satker with revision notes
//!
//! Requirements: REQ-K008, REQ-K009

use leptos::prelude::*;
use leptos::task::spawn_local;
use lib_ui::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};

// ============================================================================
// API Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatkerSubmission {
    pub id: Uuid,
    pub pengajuan_id: Uuid,
    pub ms_satker_id: String,
    pub satker_nama: Option<String>,
    pub status_kode: i32,
    pub status_nama: Option<String>,
    pub prioritas: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatkerDetail {
    pub id: Uuid,
    pub pengajuan_id: Uuid,
    pub ms_satker_id: String,
    pub satker_nama: Option<String>,
    pub status_kode: i32,
    pub status_nama: Option<String>,
    pub prioritas: i32,
    pub barang_items: Vec<BarangItem>,
    pub aktivitas_history: Vec<AktivitasItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarangItem {
    pub id: Uuid,
    pub nama: String,
    pub kode_barang: Option<String>,
    pub jumlah: i32,
    pub jml_setuju: i32,
    pub prioritas: i32,
    pub skor: f64,
    pub justifikasi: Option<String>,
    pub file_pendukung: Vec<String>,
    pub existing_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AktivitasItem {
    pub id: Uuid,
    pub aktivitas_id: i32,
    pub aktivitas_nama: Option<String>,
    pub user_id: Option<Uuid>,
    pub user_nama: Option<String>,
    pub catatan: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorWilayahActionRequest {
    pub action: String, // "forward" or "return"
    pub catatan: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanInfo {
    pub id: Uuid,
    pub nama: String,
    pub tahun: i32,
    pub tgl_mulai: NaiveDate,
    pub tgl_selesai: NaiveDate,
}

// ============================================================================
// Main Component
// ============================================================================

#[component]
pub fn ReviewPage() -> impl IntoView {
    // State management
    let (submissions, set_submissions) = signal::<Vec<SatkerSubmission>>(Vec::new());
    let (selected_submission, set_selected_submission) = signal::<Option<SatkerDetail>>(None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let (success_message, set_success_message) = signal::<Option<String>>(None);
    let (show_action_modal, set_show_action_modal) = signal(false);
    let (action_type, set_action_type) = signal::<Option<String>>(None);
    let (catatan, set_catatan) = signal(String::new());

    // Load submissions on mount
    Effect::new(move || {
        spawn_local(async move {
            set_loading.set(true);
            match fetch_submissions_for_review().await {
                Ok(subs) => set_submissions.set(subs),
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    // Load submission detail
    let load_submission_detail = move |submission_id: Uuid| {
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);
            match fetch_satker_detail(submission_id).await {
                Ok(detail) => set_selected_submission.set(Some(detail)),
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    };

    // Open action modal
    let open_action_modal = move |action: String| {
        set_action_type.set(Some(action));
        set_catatan.set(String::new());
        set_show_action_modal.set(true);
    };

    // Submit action
    let submit_action = move || {
        let Some(action) = action_type.get() else {
            return;
        };

        let Some(submission) = selected_submission.get() else {
            set_error.set(Some("Tidak ada submission yang dipilih".to_string()));
            return;
        };

        let catatan_text = catatan.get();
        if catatan_text.trim().is_empty() {
            set_error.set(Some("Catatan harus diisi".to_string()));
            return;
        }

        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            let request = ValidatorWilayahActionRequest {
                action: action.clone(),
                catatan: catatan_text.clone(),
            };

            match submit_validator_wilayah_action(submission.id, request).await {
                Ok(_) => {
                    let message = if action == "forward" {
                        "Pengajuan berhasil diteruskan ke Validator Pusat (REQ-K008)".to_string()
                    } else {
                        "Pengajuan berhasil dikembalikan ke Operator Satker untuk revisi (REQ-K009)".to_string()
                    };
                    set_success_message.set(Some(message));
                    set_show_action_modal.set(false);

                    // Reload submissions list
                    if let Ok(subs) = fetch_submissions_for_review().await {
                        set_submissions.set(subs);
                    }

                    // Clear selected submission
                    set_selected_submission.set(None);
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="container mx-auto px-4 py-8">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-6">
                "Review Kebutuhan BMN - Validator Wilayah"
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

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                // Left Panel: Submissions List
                <div class="lg:col-span-1">
                    <Card title="Daftar Pengajuan">
                        <div class="space-y-2">
                            {move || {
                                let subs = submissions.get();
                                if subs.is_empty() && !loading.get() {
                                    view! {
                                        <div class="text-center py-8 text-gray-500">
                                            "Tidak ada pengajuan yang perlu direview"
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="space-y-2">
                                            {subs.into_iter().map(|sub| {
                                                let sub_id = sub.id;
                                                let is_selected = selected_submission.get()
                                                    .map(|s| s.id == sub_id)
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
                                                        on:click=move |_| load_submission_detail(sub_id)
                                                    >
                                                        <div class="font-semibold text-gray-900">
                                                            {sub.satker_nama.clone().unwrap_or_else(|| sub.ms_satker_id.clone())}
                                                        </div>
                                                        <div class="text-sm text-gray-600 mt-1">
                                                            "Status: " {sub.status_nama.clone().unwrap_or_else(|| format!("Kode {}", sub.status_kode))}
                                                        </div>
                                                        <div class="text-xs text-gray-500 mt-1">
                                                            {sub.created_at.format("%d %b %Y %H:%M").to_string()}
                                                        </div>
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

                // Right Panel: Submission Detail
                <div class="lg:col-span-2">
                    {move || {
                        if let Some(detail) = selected_submission.get() {
                            let detail_barang_items = detail.barang_items.clone();
                            view! {
                                <div class="space-y-6">
                                    // Satker Information
                                    <Card title="Informasi Satker">
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                            <div>
                                                <label class="text-sm font-medium text-gray-700">"Nama Satker"</label>
                                                <p class="text-gray-900">{detail.satker_nama.clone().unwrap_or_else(|| detail.ms_satker_id.clone())}</p>
                                            </div>
                                            <div>
                                                <label class="text-sm font-medium text-gray-700">"Kode Satker"</label>
                                                <p class="text-gray-900">{detail.ms_satker_id.clone()}</p>
                                            </div>
                                            <div>
                                                <label class="text-sm font-medium text-gray-700">"Status"</label>
                                                <Badge
                                                    label=detail.status_nama.clone().unwrap_or_else(|| format!("Kode {}", detail.status_kode))
                                                    variant=BadgeVariant::Info
                                                />
                                            </div>
                                            <div>
                                                <label class="text-sm font-medium text-gray-700">"Prioritas"</label>
                                                <p class="text-gray-900">{detail.prioritas}</p>
                                            </div>
                                        </div>
                                    </Card>

                                    // BMN Items Table
                                    <Card>
                                        <h2 class="text-xl font-semibold mb-4">
                                            "Daftar Barang Kebutuhan "
                                            <span class="text-emerald-600">
                                                "(" {detail.barang_items.len()} " item)"
                                            </span>
                                        </h2>
                                        <div class="overflow-x-auto">
                                            <table class="min-w-full divide-y divide-gray-200">
                                                <thead class="bg-gray-50">
                                                    <tr>
                                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"No"</th>
                                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"Nama Barang"</th>
                                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"Kode"</th>
                                                        <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase">"Jumlah"</th>
                                                        <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase">"Existing"</th>
                                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"Justifikasi"</th>
                                                    </tr>
                                                </thead>
                                                <tbody class="bg-white divide-y divide-gray-200">
                                                    {detail.barang_items.iter().enumerate().map(|(idx, item)| view! {
                                                        <tr class="hover:bg-gray-50">
                                                            <td class="px-4 py-3 text-sm text-gray-900">{idx + 1}</td>
                                                            <td class="px-4 py-3 text-sm font-medium text-gray-900">{item.nama.clone()}</td>
                                                            <td class="px-4 py-3 text-sm text-gray-600">
                                                                {item.kode_barang.clone().unwrap_or_else(|| "-".to_string())}
                                                            </td>
                                                            <td class="px-4 py-3 text-sm text-right text-gray-900">{item.jumlah}</td>
                                                            <td class="px-4 py-3 text-sm text-right text-gray-600">{item.existing_count}</td>
                                                            <td class="px-4 py-3 text-sm text-gray-700">
                                                                {item.justifikasi.clone().unwrap_or_else(|| "-".to_string())}
                                                            </td>
                                                        </tr>
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        </div>
                                    </Card>

                                    // Supporting Documents
                                    {{
                                        let has_docs = detail_barang_items.iter()
                                            .any(|item| !item.file_pendukung.is_empty());

                                        if has_docs {
                                            view! {
                                                <Card title="Dokumen Pendukung">
                                                    <div class="space-y-3">
                                                        {detail_barang_items.iter().filter(|item| !item.file_pendukung.is_empty()).map(|item| view! {
                                                            <div class="border-l-4 border-emerald-500 pl-4">
                                                                <p class="font-medium text-gray-900">{item.nama.clone()}</p>
                                                                <div class="mt-2 space-y-1">
                                                                    {item.file_pendukung.iter().map(|file| view! {
                                                                        <a
                                                                            href=file.clone()
                                                                            target="_blank"
                                                                            class="text-sm text-emerald-600 hover:text-emerald-700 flex items-center"
                                                                        >
                                                                            <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                                                            </svg>
                                                                            "Download Dokumen"
                                                                        </a>
                                                                    }).collect_view()}
                                                                </div>
                                                            </div>
                                                        }).collect_view()}
                                                    </div>
                                                </Card>
                                            }.into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }
                                    }}

                                    // Activity History
                                    <Card title="Riwayat Aktivitas">
                                        <div class="space-y-3">
                                            {detail.aktivitas_history.iter().map(|aktivitas| view! {
                                                <div class="flex items-start space-x-3 border-l-2 border-gray-300 pl-4 py-2">
                                                    <div class="flex-shrink-0 w-2 h-2 bg-emerald-500 rounded-full mt-2"></div>
                                                    <div class="flex-1">
                                                        <div class="flex items-center justify-between">
                                                            <p class="font-medium text-gray-900">
                                                                {aktivitas.aktivitas_nama.clone().unwrap_or_else(|| format!("Aktivitas {}", aktivitas.aktivitas_id))}
                                                            </p>
                                                            <span class="text-xs text-gray-500">
                                                                {aktivitas.created_at.format("%d %b %Y %H:%M").to_string()}
                                                            </span>
                                                        </div>
                                                        {aktivitas.user_nama.as_ref().map(|user| view! {
                                                            <p class="text-sm text-gray-600 mt-1">
                                                                "Oleh: " {user.clone()}
                                                            </p>
                                                        })}
                                                        {aktivitas.catatan.as_ref().map(|catatan| view! {
                                                            <p class="text-sm text-gray-700 mt-1 italic">
                                                                "\"" {catatan.clone()} "\""
                                                            </p>
                                                        })}
                                                    </div>
                                                </div>
                                            }).collect_view()}
                                        </div>
                                    </Card>

                                    // Action Buttons
                                    <Card>
                                        <div class="flex justify-end space-x-4">
                                            <Button
                                                variant=ButtonVariant::Secondary
                                                on_click=Box::new(move || {
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
                                                variant=ButtonVariant::Danger
                                                on_click=Box::new(move || open_action_modal("return".to_string()))
                                                disabled=loading.get()
                                            >
                                                "Kembalikan untuk Revisi"
                                            </Button>

                                            <Button
                                                variant=ButtonVariant::Primary
                                                size=ButtonSize::Large
                                                on_click=Box::new(move || open_action_modal("forward".to_string()))
                                                disabled=loading.get()
                                            >
                                                "Teruskan ke Validator Pusat"
                                            </Button>
                                        </div>
                                    </Card>
                                </div>
                            }.into_any()
                        } else if !loading.get() {
                            view! {
                                <Card>
                                    <div class="text-center py-12 text-gray-500">
                                        <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                        </svg>
                                        <p class="mt-4">"Pilih pengajuan dari daftar untuk melihat detail"</p>
                                    </div>
                                </Card>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }}
                </div>
            </div>

            // Action Modal
            {move || show_action_modal.get().then(|| {
                let action = action_type.get().unwrap_or_default();
                let title = if action == "forward" {
                    "Teruskan ke Validator Pusat"
                } else {
                    "Kembalikan untuk Revisi"
                };
                let description = if action == "forward" {
                    "Pengajuan akan diteruskan ke Validator Pusat untuk analisis lebih lanjut (REQ-K008)"
                } else {
                    "Pengajuan akan dikembalikan ke Operator Satker untuk dilakukan revisi (REQ-K009)"
                };

                view! {
                    <Modal
                        show=show_action_modal.get()
                        on_close=Box::new(move || set_show_action_modal.set(false))
                        title=title.to_string()
                    >
                        <div class="space-y-4">
                            <p class="text-sm text-gray-600">{description}</p>

                            <Textarea
                                label="Catatan"
                                placeholder="Masukkan catatan untuk tindakan ini..."
                                value=catatan.get()
                                on_input=Box::new(move |v| set_catatan.set(v))
                                rows=4
                                required=true
                            />

                            <div class="flex justify-end space-x-3 mt-6">
                                <Button
                                    variant=ButtonVariant::Secondary
                                    on_click=Box::new(move || set_show_action_modal.set(false))
                                >
                                    "Batal"
                                </Button>

                                <Button
                                    variant=if action == "forward" { ButtonVariant::Primary } else { ButtonVariant::Danger }
                                    on_click=Box::new(submit_action)
                                    disabled=loading.get()
                                >
                                    {if action == "forward" { "Teruskan" } else { "Kembalikan" }}
                                </Button>
                            </div>
                        </div>
                    </Modal>
                }
            })}
        </div>
    }
}

// ============================================================================
// API Functions
// ============================================================================

async fn fetch_submissions_for_review() -> Result<Vec<SatkerSubmission>, String> {
    // Fetch submissions with status_kode = 2002 (SUBMIT_SATKER - waiting for Validator Wilayah)
    let response = gloo_net::http::Request::get("/api/v1/kebutuhan-bmn/satker?status_kode=2002")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let api_response: ApiResponse<Vec<SatkerSubmission>> = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(api_response.data.unwrap_or_default())
}

async fn fetch_satker_detail(satker_id: Uuid) -> Result<SatkerDetail, String> {
    let url = format!("/api/v1/kebutuhan-bmn/satker/{}", satker_id);
    let response = gloo_net::http::Request::get(&url)
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
        .ok_or_else(|| "Satker detail not found".to_string())
}

async fn submit_validator_wilayah_action(
    satker_id: Uuid,
    request: ValidatorWilayahActionRequest,
) -> Result<SatkerDetail, String> {
    let url = format!("/api/v1/kebutuhan-bmn/satker/{}/validator-wilayah", satker_id);
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
