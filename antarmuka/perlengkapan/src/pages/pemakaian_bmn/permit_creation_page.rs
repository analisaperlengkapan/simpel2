//! # Pemakaian BMN Permit Creation Page
//!
//! This page allows Operator Satker to:
//! - Create izin pemakaian BMN
//! - Select pegawai from MySIMKARI integration
//! - Select multiple BMN items with availability check
//! - Specify usage period
//! - Generate draft permit document
//!
//! Requirements: REQ-P001 to REQ-P010

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
pub struct CreatePermitRequest {
    pub satker_id: Uuid,
    pub pegawai_nip: String,
    pub pegawai_nama: String,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pegawai {
    pub nip: String,
    pub nama: String,
    pub jabatan: Option<String>,
    pub pangkat: Option<String>,
    pub golongan: Option<String>,
    pub photo_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmnItem {
    pub nup: String,
    pub nama_barang: String,
    pub kode_barang: String,
    pub kondisi: String,
    pub tahun_perolehan: Option<i32>,
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddBmnRequest {
    pub nup: String,
    pub nama_barang: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permit {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub pegawai_nip: String,
    pub pegawai_nama: String,
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
pub fn PermitCreationPage() -> impl IntoView {
    // State
    let (pegawai_list, set_pegawai_list) = signal::<Vec<Pegawai>>(Vec::new());
    let (selected_pegawai, set_selected_pegawai) = signal::<Option<Pegawai>>(None);
    let (available_bmn, set_available_bmn) = signal::<Vec<BmnItem>>(Vec::new());
    let (selected_bmn, set_selected_bmn) = signal::<Vec<BmnItem>>(Vec::new());
    let (start_date, set_start_date) = signal(String::new());
    let (end_date, set_end_date) = signal(String::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let (success_message, set_success_message) = signal::<Option<String>>(None);
    let (created_permit, set_created_permit) = signal::<Option<Permit>>(None);

    // Load pegawai list on mount
    Effect::new(move || {
        spawn_local(async move {
            set_loading.set(true);
            match fetch_pegawai_list().await {
                Ok(list) => set_pegawai_list.set(list),
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    // Load available BMN when pegawai selected
    let load_available_bmn = move || {
        spawn_local(async move {
            set_loading.set(true);
            match fetch_available_bmn().await {
                Ok(list) => set_available_bmn.set(list),
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    };

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

    // Create permit
    let create_permit = move || {
        let Some(pegawai) = selected_pegawai.get() else {
            set_error.set(Some("Pilih pegawai terlebih dahulu".to_string()));
            return;
        };

        if selected_bmn.get().is_empty() {
            set_error.set(Some("Pilih minimal 1 BMN (REQ-P002)".to_string()));
            return;
        }

        let start = start_date.get();
        let end = end_date.get();
        if start.is_empty() || end.is_empty() {
            set_error.set(Some("Tanggal mulai dan selesai harus diisi (REQ-P005)".to_string()));
            return;
        }

        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            // Create permit
            let request = CreatePermitRequest {
                satker_id: Uuid::new_v4(), // TODO: Get from auth context
                pegawai_nip: pegawai.nip.clone(),
                pegawai_nama: pegawai.nama.clone(),
                tanggal_mulai: NaiveDate::parse_from_str(&start, "%Y-%m-%d").unwrap(),
                tanggal_selesai: NaiveDate::parse_from_str(&end, "%Y-%m-%d").unwrap(),
            };

            match create_permit_api(request).await {
                Ok(permit) => {
                    set_created_permit.set(Some(permit.clone()));

                    // Add BMN items
                    for bmn in selected_bmn.get() {
                        let _ = add_bmn_to_permit(permit.id, bmn).await;
                    }

                    set_success_message.set(Some("Izin pemakaian berhasil dibuat (REQ-P001)".to_string()));
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="container mx-auto px-4 py-8">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-6">
                "Buat Izin Pemakaian BMN"
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

            <div class="space-y-6">
                // Step 1: Select Pegawai
                <Card title="1. Pilih Pegawai (REQ-P001)">
                    <Select
                        label="Pegawai"
                        value=selected_pegawai.get().map(|p| p.nip.clone()).unwrap_or_default()
                        on_change=Box::new(move |nip: String| {
                            let pegawai = pegawai_list.get().into_iter().find(|p| p.nip == nip);
                            set_selected_pegawai.set(pegawai);
                            if selected_pegawai.get().is_some() {
                                load_available_bmn();
                            }
                        })
                    >
                        <option value="">"-- Pilih Pegawai --"</option>
                        {move || pegawai_list.get().into_iter().map(|p| view! {
                            <option value=p.nip.clone()>
                                {format!("{} - {}", p.nip, p.nama)}
                            </option>
                        }).collect_view()}
                    </Select>

                    {move || selected_pegawai.get().map(|p| view! {
                        <div class="mt-4 p-4 bg-gray-50 rounded-lg">
                            <div class="grid grid-cols-2 gap-4">
                                <div>
                                    <label class="text-sm font-medium text-gray-700">"NIP"</label>
                                    <p class="text-gray-900">{p.nip.clone()}</p>
                                </div>
                                <div>
                                    <label class="text-sm font-medium text-gray-700">"Nama"</label>
                                    <p class="text-gray-900">{p.nama.clone()}</p>
                                </div>
                                {p.jabatan.as_ref().map(|j| view! {
                                    <div>
                                        <label class="text-sm font-medium text-gray-700">"Jabatan"</label>
                                        <p class="text-gray-900">{j.clone()}</p>
                                    </div>
                                })}
                                {p.pangkat.as_ref().map(|pk| view! {
                                    <div>
                                        <label class="text-sm font-medium text-gray-700">"Pangkat"</label>
                                        <p class="text-gray-900">{pk.clone()}</p>
                                    </div>
                                })}
                            </div>
                        </div>
                    })}
                </Card>

                // Step 2: Select BMN Items
                {move || selected_pegawai.get().is_some().then(|| view! {
                    <Card title="2. Pilih BMN (REQ-P002)">
                        <p class="text-sm text-gray-600 mb-4">
                            "Pilih BMN yang akan digunakan. Sistem akan memvalidasi ketersediaan (REQ-P003, REQ-P004)"
                        </p>

                        <div class="space-y-4">
                            {move || available_bmn.get().into_iter().map(|bmn| {
                                let nup = bmn.nup.clone();
                                let is_selected = selected_bmn.get().iter().any(|b| b.nup == nup);
                                let bmn_for_click = bmn.clone();
                                let bmn_is_available = bmn.is_available;

                                view! {
                                    <div class="border rounded-lg p-4 flex justify-between items-center">
                                        <div class="flex-1">
                                            <h4 class="font-semibold text-gray-900">{bmn.nama_barang.clone()}</h4>
                                            <p class="text-sm text-gray-600">"NUP: " {bmn.nup.clone()}</p>
                                            <p class="text-sm text-gray-600">"Kondisi: " {bmn.kondisi.clone()}</p>
                                            {if bmn.is_available {
                                                view! {
                                                    <Badge label="Tersedia" variant=BadgeVariant::Success />
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <Badge label="Sedang Digunakan" variant=BadgeVariant::Warning />
                                                }.into_any()
                                            }}
                                        </div>
                                        <Button
                                            variant=if is_selected { ButtonVariant::Secondary } else { ButtonVariant::Primary }
                                            on_click=Box::new(move || {
                                                if is_selected {
                                                    remove_bmn(nup.clone());
                                                } else {
                                                    add_bmn(bmn_for_click.clone());
                                                }
                                            })
                                            disabled=!bmn_is_available
                                        >
                                            {if is_selected { "Hapus" } else { "Pilih" }}
                                        </Button>
                                    </div>
                                }
                            }).collect_view()}
                        </div>

                        <div class="mt-4 p-4 bg-emerald-50 rounded-lg">
                            <p class="font-medium text-emerald-900">
                                "BMN Terpilih: " {move || selected_bmn.get().len()} " item"
                            </p>
                        </div>
                    </Card>
                })}

                // Step 3: Set Period
                {move || (!selected_bmn.get().is_empty()).then(|| view! {
                    <Card title="3. Tentukan Periode Pemakaian (REQ-P005)">
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <Input
                                label="Tanggal Mulai"
                                input_type="date"
                                value=start_date.get()
                                on_input=Box::new(move |v| set_start_date.set(v))
                                required=true
                            />
                            <Input
                                label="Tanggal Selesai"
                                input_type="date"
                                value=end_date.get()
                                on_input=Box::new(move |v| set_end_date.set(v))
                                required=true
                            />
                        </div>
                    </Card>
                })}

                // Submit Button
                {move || (!selected_bmn.get().is_empty() && !start_date.get().is_empty() && !end_date.get().is_empty()).then(|| view! {
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
                                on_click=Box::new(create_permit)
                                disabled=loading.get()
                            >
                                "Buat Izin Pemakaian"
                            </Button>
                        </div>
                    </Card>
                })}
            </div>
        </div>
    }
}

// ============================================================================
// API Functions
// ============================================================================

async fn fetch_pegawai_list() -> Result<Vec<Pegawai>, String> {
    let response = gloo_net::http::Request::get("/api/v1/pemakaian-bmn/pegawai")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let api_response: ApiResponse<Vec<Pegawai>> = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(api_response.data.unwrap_or_default())
}

async fn fetch_available_bmn() -> Result<Vec<BmnItem>, String> {
    let response = gloo_net::http::Request::get("/api/v1/pemakaian-bmn/bmn/available")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let api_response: ApiResponse<Vec<BmnItem>> = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(api_response.data.unwrap_or_default())
}

async fn create_permit_api(request: CreatePermitRequest) -> Result<Permit, String> {
    let response = gloo_net::http::Request::post("/api/v1/pemakaian-bmn")
        .json(&request)
        .map_err(|e| format!("Serialization error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let api_response: ApiResponse<Permit> = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    api_response.data.ok_or_else(|| "No data in response".to_string())
}

async fn add_bmn_to_permit(permit_id: Uuid, bmn: BmnItem) -> Result<(), String> {
    let request = AddBmnRequest {
        nup: bmn.nup,
        nama_barang: bmn.nama_barang,
    };

    let url = format!("/api/v1/pemakaian-bmn/{}/bmn", permit_id);
    let response = gloo_net::http::Request::post(&url)
        .json(&request)
        .map_err(|e| format!("Serialization error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    Ok(())
}
