//! # Penghapusan BMN Review Page (Validator Wilayah)
//!
//! This page allows Validator Wilayah to:
//! - Review SK Penghapusan BMN requests
//! - View BMN details and supporting documents
//! - Forward to Validator Pusat or return to Operator Satker
//!
//! Requirements: REQ-PH005, REQ-PH006

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
pub struct PenghapusanRequest {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub satker_nama: Option<String>,
    pub status: String,
    pub alasan: String,
    pub metode_penghapusan: String,
    pub bmn_items: Vec<BmnItem>,
    pub attachments: Vec<Attachment>,
    pub created_at: DateTime<Utc>,
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
pub struct Attachment {
    pub id: Uuid,
    pub filename: String,
    pub url: String,
    pub uploaded_at: DateTime<Utc>,
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

// ============================================================================
// Main Component
// ============================================================================

#[component]
pub fn ReviewPage() -> impl IntoView {
    let (requests, set_requests) = signal::<Vec<PenghapusanRequest>>(Vec::new());
    let (selected_request, set_selected_request) = signal::<Option<PenghapusanRequest>>(None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let (success_message, set_success_message) = signal::<Option<String>>(None);
    let (show_action_modal, set_show_action_modal) = signal(false);
    let (action_type, set_action_type) = signal::<Option<String>>(None);
    let (catatan, set_catatan) = signal(String::new());

    // Load requests on mount
    Effect::new(move || {
        spawn_local(async move {
            set_loading.set(true);
            match fetch_requests_for_review().await {
                Ok(list) => set_requests.set(list),
                Err(e) => set_error.set(Some(e.to_string())),
            }
            set_loading.set(false);
        });
    });

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

        let Some(request) = selected_request.get() else {
            set_error.set(Some("Tidak ada permohonan yang dipilih".to_string()));
            return;
        };

        let catatan_text = catatan.get();
        if catatan_text.trim().is_empty() {
            set_error.set(Some("Catatan harus diisi".to_string()));
            return;
        };

        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            let req = ValidatorWilayahActionRequest {
                action: action.clone(),
                catatan: catatan_text.clone(),
            };

            match submit_validator_wilayah_action(request.id, req).await {
                Ok(_) => {
                    let message = if action == "forward" {
                        "Permohonan berhasil diteruskan ke Validator Pusat (REQ-PH006)".to_string()
                    } else {
                        "Permohonan berhasil dikembalikan ke Operator Satker (REQ-PH006)"
                            .to_string()
                    };
                    set_success_message.set(Some(message));
                    set_show_action_modal.set(false);

                    // Reload requests
                    if let Ok(list) = fetch_requests_for_review().await {
                        set_requests.set(list);
                    }
                    set_selected_request.set(None);
                }
                Err(e) => set_error.set(Some(e.to_string())),
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="container mx-auto px-4 py-8">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-6">
                "Review SK Penghapusan BMN - Validator Wilayah"
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
                                            "Tidak ada permohonan yang perlu direview"
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="space-y-2">
                                            {list.into_iter().map(|req| {
                                                let req_id = req.id;
                                                let is_selected = selected_request.get()
                                                    .map(|r| r.id == req_id)
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
                                                        on:click=move |_| set_selected_request.set(Some(req.clone()))
                                                    >
                                                        <div class="font-semibold text-gray-900">
                                                            {req.satker_nama.clone().unwrap_or_else(|| "Unknown Satker".to_string())}
                                                        </div>
                                                        <div class="text-sm text-gray-600 mt-1">
                                                            {req.bmn_items.len()} " BMN"
                                                        </div>
                                                        <div class="text-xs text-gray-500 mt-1">
                                                            {req.created_at.format("%d %b %Y").to_string()}
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

                // Right Panel: Request Detail
                <div class="lg:col-span-2">
                    {move || {
                        if let Some(request) = selected_request.get() {
                            let total_nilai: f64 = request.bmn_items.iter()
                                .filter_map(|item| item.nilai_perolehan)
                                .sum();

                            view! {
                                <div class="space-y-6">
                                    // Request Information
                                    <Card title="Informasi Permohonan">
                                        <div class="grid grid-cols-2 gap-4">
                                            <div>
                                                <label class="text-sm font-medium text-gray-700">"Satker"</label>
                                                <p class="text-gray-900">
                                                    {request.satker_nama.clone().unwrap_or_else(|| "Unknown".to_string())}
                                                </p>
                                            </div>
                                            <div>
                                                <label class="text-sm font-medium text-gray-700">"Metode"</label>
                                                <p class="text-gray-900">{request.metode_penghapusan.clone()}</p>
                                            </div>
                                            <div class="col-span-2">
                                                <label class="text-sm font-medium text-gray-700">"Alasan"</label>
                                                <p class="text-gray-900">{request.alasan.clone()}</p>
                                            </div>
                                        </div>
                                    </Card>

                                    // BMN Items Table
                                    <Card>
                                        <h2 class="text-xl font-semibold mb-4">
                                            "Daftar BMN yang Akan Dihapus "
                                            <span class="text-emerald-600">
                                                "(" {request.bmn_items.len()} " item)"
                                            </span>
                                        </h2>
                                        <div class="overflow-x-auto">
                                            <table class="min-w-full divide-y divide-gray-200">
                                                <thead class="bg-gray-50">
                                                    <tr>
                                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"No"</th>
                                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"Nama Barang"</th>
                                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"NUP"</th>
                                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"Kondisi"</th>
                                                        <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase">"Nilai"</th>
                                                    </tr>
                                                </thead>
                                                <tbody class="bg-white divide-y divide-gray-200">
                                                    {request.bmn_items.iter().enumerate().map(|(idx, item)| view! {
                                                        <tr class="hover:bg-gray-50">
                                                            <td class="px-4 py-3 text-sm text-gray-900">{idx + 1}</td>
                                                            <td class="px-4 py-3 text-sm font-medium text-gray-900">{item.nama_barang.clone()}</td>
                                                            <td class="px-4 py-3 text-sm text-gray-600">{item.nup.clone()}</td>
                                                            <td class="px-4 py-3 text-sm text-gray-600">{item.kondisi.clone()}</td>
                                                            <td class="px-4 py-3 text-sm text-right text-gray-900">
                                                                {item.nilai_perolehan.map(|n| format!("Rp {:.2}", n)).unwrap_or_else(|| "-".to_string())}
                                                            </td>
                                                        </tr>
                                                    }).collect_view()}
                                                    <tr class="bg-gray-50 font-semibold">
                                                        <td colspan="4" class="px-4 py-3 text-sm text-right">"Total Nilai:"</td>
                                                        <td class="px-4 py-3 text-sm text-right">
                                                            "Rp " {format!("{:.2}", total_nilai)}
                                                        </td>
                                                    </tr>
                                                </tbody>
                                            </table>
                                        </div>
                                    </Card>

                                    // Supporting Documents
                                    <Card title="Dokumen Pendukung">
                                        <div class="space-y-2">
                                            {request.attachments.iter().map(|att| view! {
                                                <div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                                                    <div class="flex items-center space-x-3">
                                                        <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                                        </svg>
                                                        <div>
                                                            <p class="font-medium text-gray-900">{att.filename.clone()}</p>
                                                            <p class="text-xs text-gray-500">
                                                                {att.uploaded_at.format("%d %b %Y %H:%M").to_string()}
                                                            </p>
                                                        </div>
                                                    </div>
                                                    <a
                                                        href=att.url.clone()
                                                        target="_blank"
                                                        class="text-emerald-600 hover:text-emerald-700"
                                                    >
                                                        "Download"
                                                    </a>
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
                                                    web_sys::window().unwrap().history().unwrap().back().ok();
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
                        } else {
                            view! {
                                <Card>
                                    <div class="text-center py-12 text-gray-500">
                                        "Pilih permohonan dari daftar untuk melihat detail"
                                    </div>
                                </Card>
                            }.into_any()
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

                view! {
                    <Modal
                        show=show_action_modal.get()
                        on_close=Box::new(move || set_show_action_modal.set(false))
                        title=title.to_string()
                    >
                        <div class="space-y-4">
                            <Textarea
                                label="Catatan"
                                placeholder="Masukkan catatan..."
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

async fn fetch_requests_for_review() -> Result<Vec<PenghapusanRequest>, crate::api::AppError> {
    let response =
        gloo_net::http::Request::get("/api/v1/perlengkapan/penghapusan-bmn?status=SUBMITTED")
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

async fn submit_validator_wilayah_action(
    request_id: Uuid,
    request: ValidatorWilayahActionRequest,
) -> Result<(), crate::api::AppError> {
    let url = format!(
        "/api/v1/perlengkapan/penghapusan-bmn/{}/validator-wilayah",
        request_id
    );
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
