//! # SK Penghapusan BMN View Page (Operator Satker, Validator Wilayah)
//!
//! This page allows Operator Satker and Validator Wilayah to:
//! - View uploaded signed SK Penghapusan (PDF)
//! - Download signed SK
//! - View BMN details in the SK
//!
//! Requirements: REQ-PH011, REQ-PH012

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
    pub created_at: String,
    pub updated_at: String,
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
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

// ============================================================================
// Main Component
// ============================================================================

#[component]
pub fn SkViewPage() -> impl IntoView {
    let (requests, set_requests) = signal::<Vec<PenghapusanRequest>>(Vec::new());
    let (selected_request, set_selected_request) = signal::<Option<PenghapusanRequest>>(None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    // Load completed requests on mount
    Effect::new(move || {
        spawn_local(async move {
            set_loading.set(true);
            match fetch_completed_requests().await {
                Ok(list) => set_requests.set(list),
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    view! {
        <div class="container mx-auto px-4 py-8">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-6">
                "SK Penghapusan BMN - Lihat Dokumen"
            </h1>

            {move || error.get().map(|e| view! {
                <Alert message=e variant=AlertVariant::Error />
            })}

            {move || loading.get().then(|| view! {
                <div class="flex justify-center py-4">
                    <Spinner size="lg" />
                </div>
            })}

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                // Left Panel: Requests List
                <div class="lg:col-span-1">
                    <Card title="Daftar SK Penghapusan">
                        <div class="space-y-2">
                            {move || {
                                let list = requests.get();
                                if list.is_empty() && !loading.get() {
                                    view! {
                                        <div class="text-center py-8 text-gray-500">
                                            "Tidak ada SK yang tersedia"
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="space-y-2">
                                            {list.into_iter().map(|req| {
                                                let has_signed_sk = req.signed_sk_url.is_some();
                                                view! {
                                                    <div
                                                        class="p-4 border rounded-lg cursor-pointer hover:bg-gray-50 transition-colors"
                                                        class:border-green-500=has_signed_sk
                                                        class:bg-green-50=has_signed_sk
                                                        on:click=move |_| set_selected_request.set(Some(req.clone()))
                                                    >
                                                        <div class="font-semibold text-gray-900">
                                                            {req.satker_nama.clone().unwrap_or_else(|| "Unknown".to_string())}
                                                        </div>
                                                        {req.sk_number.as_ref().map(|num| view! {
                                                            <div class="text-sm text-gray-700 mt-1 font-mono">
                                                                {num.clone()}
                                                            </div>
                                                        })}
                                                        <div class="text-xs text-gray-600 mt-1">
                                                            {req.bmn_items.len()} " BMN"
                                                        </div>
                                                        <div class="mt-2">
                                                            {if has_signed_sk {
                                                                view! {
                                                                    <Badge
                                                                        label="SK Tersedia"
                                                                        variant=BadgeVariant::Success
                                                                    />
                                                                }.into_any()
                                                            } else {
                                                                view! {
                                                                    <Badge
                                                                        label="Menunggu SK"
                                                                        variant=BadgeVariant::Warning
                                                                    />
                                                                }.into_any()
                                                            }}
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

                // Right Panel: SK Details and Viewer
                <div class="lg:col-span-2">
                    {move || {
                        if let Some(request) = selected_request.get() {
                            let request_sk_number_for_summary = request.sk_number.clone();
                            let request_sk_number_for_document = request.sk_number.clone();
                            let request_bmn_items_for_summary = request.bmn_items.clone();
                            let request_bmn_items_for_table = request.bmn_items.clone();
                            let total_nilai: f64 = request_bmn_items_for_table.iter()
                                .filter_map(|item| item.nilai_perolehan)
                                .sum();

                            view! {
                                <div class="space-y-6">
                                    // Request Summary
                                    <Card title="Informasi SK Penghapusan">
                                        <div class="grid grid-cols-2 gap-4">
                                            <div>
                                                <label class="text-sm font-medium text-gray-700">"Nomor SK"</label>
                                                <p class="text-gray-900 font-mono">
                                                    {request_sk_number_for_summary.clone().unwrap_or_else(|| "-".to_string())}
                                                </p>
                                            </div>
                                            <div>
                                                <label class="text-sm font-medium text-gray-700">"Status"</label>
                                                <div class="mt-1">
                                                    <Badge
                                                        label=request.status.clone()
                                                        variant=match request.status.as_str() {
                                                            "COMPLETED" => BadgeVariant::Success,
                                                            "DOCUMENT_GENERATED" => BadgeVariant::Info,
                                                            _ => BadgeVariant::Warning,
                                                        }
                                                    />
                                                </div>
                                            </div>
                                            <div>
                                                <label class="text-sm font-medium text-gray-700">"Satker"</label>
                                                <p class="text-gray-900">
                                                    {request.satker_nama.clone().unwrap_or_else(|| "Unknown".to_string())}
                                                </p>
                                            </div>
                                            <div>
                                                <label class="text-sm font-medium text-gray-700">"Metode Penghapusan"</label>
                                                <p class="text-gray-900">{request.metode_penghapusan.clone()}</p>
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
                                        </div>
                                    </Card>

                                    // BMN Items Table
                                    <Card title="Daftar BMN yang Dihapus">
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
                                                    {request_bmn_items_for_table.iter().enumerate().map(|(idx, item)| view! {
                                                        <tr>
                                                            <td class="px-4 py-3 text-sm">{idx + 1}</td>
                                                            <td class="px-4 py-3 text-sm">
                                                                <div class="font-medium">{item.nama_barang.clone()}</div>
                                                                <div class="text-xs text-gray-500">{item.kode_barang.clone()}</div>
                                                            </td>
                                                            <td class="px-4 py-3 text-sm font-mono">{item.nup.clone()}</td>
                                                            <td class="px-4 py-3 text-sm">
                                                                <Badge
                                                                    label=item.kondisi.clone()
                                                                    variant=match item.kondisi.as_str() {
                                                                        "BAIK" => BadgeVariant::Success,
                                                                        "RUSAK_RINGAN" => BadgeVariant::Warning,
                                                                        _ => BadgeVariant::Danger,
                                                                    }
                                                                />
                                                            </td>
                                                            <td class="px-4 py-3 text-sm text-right">
                                                                {item.nilai_perolehan.map(|n| format!("Rp {:.2}", n)).unwrap_or_else(|| "-".to_string())}
                                                            </td>
                                                        </tr>
                                                    }).collect_view()}
                                                </tbody>
                                                <tfoot class="bg-gray-50">
                                                    <tr>
                                                        <td colspan="4" class="px-4 py-3 text-sm font-semibold text-right">"Total Nilai:"</td>
                                                        <td class="px-4 py-3 text-sm font-bold text-right">
                                                            "Rp " {format!("{:.2}", total_nilai)}
                                                        </td>
                                                    </tr>
                                                </tfoot>
                                            </table>
                                        </div>
                                    </Card>

                                    // SK Document Viewer (REQ-PH011, REQ-PH012)
                                    <Card title="Dokumen SK Penghapusan">
                                        {if let Some(signed_sk_url) = request.signed_sk_url.clone() {
                                            view! {
                                                <div class="space-y-4">
                                                    // Download Button
                                                    <div class="flex items-center justify-between p-4 bg-green-50 rounded-lg border border-green-200">
                                                        <div class="flex items-center space-x-3">
                                                            <svg class="w-10 h-10 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                                            </svg>
                                                            <div>
                                                                <p class="font-semibold text-green-900">"SK Penghapusan Tersedia"</p>
                                                                <p class="text-sm text-green-700">"Dokumen telah ditandatangani dan siap diunduh"</p>
                                                                {request_sk_number_for_document.as_ref().map(|num| view! {
                                                                    <p class="text-xs text-green-600 font-mono mt-1">{num.clone()}</p>
                                                                })}
                                                            </div>
                                                        </div>
                                                        <a
                                                            href=signed_sk_url.clone()
                                                            target="_blank"
                                                            download
                                                            class="px-6 py-3 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors flex items-center space-x-2"
                                                        >
                                                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                                            </svg>
                                                            <span>"Download PDF"</span>
                                                        </a>
                                                    </div>

                                                    // PDF Viewer (embedded)
                                                    <div class="border rounded-lg overflow-hidden bg-gray-100">
                                                        <div class="bg-gray-800 text-white px-4 py-2 flex items-center justify-between">
                                                            <span class="text-sm font-medium">"Preview SK Penghapusan"</span>
                                                            <a
                                                                href=signed_sk_url.clone()
                                                                target="_blank"
                                                                class="text-xs text-blue-300 hover:text-blue-100"
                                                            >
                                                                "Buka di tab baru"
                                                            </a>
                                                        </div>
                                                        <div class="relative" style="height: 800px;">
                                                            <iframe
                                                                src=signed_sk_url.clone()
                                                                class="w-full h-full"
                                                                title="SK Penghapusan PDF"
                                                            />
                                                        </div>
                                                    </div>

                                                    // Additional Info
                                                    <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
                                                        <div class="flex items-start space-x-3">
                                                            <svg class="w-5 h-5 text-blue-600 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                                            </svg>
                                                            <div class="text-sm text-blue-800">
                                                                <p class="font-medium mb-1">"Informasi Dokumen"</p>
                                                                <ul class="list-disc list-inside space-y-1 text-blue-700">
                                                                    <li>"Dokumen SK telah ditandatangani oleh pejabat berwenang"</li>
                                                                    <li>"Format dokumen: PDF"</li>
                                                                    <li>"Dokumen dapat diunduh dan disimpan untuk arsip"</li>
                                                                </ul>
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                            }.into_any()
                                        } else if let Some(draft_url) = request.sk_document_url.clone() {
                                            view! {
                                                <div class="text-center py-12">
                                                    <svg class="w-16 h-16 text-yellow-500 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                                                    </svg>
                                                    <p class="text-lg font-medium text-gray-900 mb-2">
                                                        "SK Penghapusan Sedang Diproses"
                                                    </p>
                                                    <p class="text-sm text-gray-600 mb-4">
                                                        "Dokumen SK telah dibuat dan sedang menunggu tanda tangan"
                                                    </p>
                                                    <a
                                                        href=draft_url
                                                        target="_blank"
                                                        class="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
                                                    >
                                                        "Lihat Draft SK (DOCX)"
                                                    </a>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="text-center py-12">
                                                    <svg class="w-16 h-16 text-gray-400 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                                    </svg>
                                                    <p class="text-lg font-medium text-gray-900 mb-2">
                                                        "Dokumen SK Belum Tersedia"
                                                    </p>
                                                    <p class="text-sm text-gray-600">
                                                        "SK Penghapusan belum dibuat oleh Validator Pusat"
                                                    </p>
                                                </div>
                                            }.into_any()
                                        }}
                                    </Card>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <Card>
                                    <div class="text-center py-12 text-gray-500">
                                        <svg class="w-16 h-16 text-gray-300 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                        </svg>
                                        <p class="text-lg">"Pilih SK dari daftar untuk melihat detail"</p>
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

async fn fetch_completed_requests() -> Result<Vec<PenghapusanRequest>, String> {
    // Fetch requests with status DOCUMENT_GENERATED or COMPLETED
    let response = gloo_net::http::Request::get("/api/v1/penghapusan-bmn?status=DOCUMENT_GENERATED,COMPLETED")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let api_response: ApiResponse<Vec<PenghapusanRequest>> = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(api_response.data.unwrap_or_default())
}
