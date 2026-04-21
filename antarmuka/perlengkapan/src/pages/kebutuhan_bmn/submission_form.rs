//! # Kebutuhan BMN Submission Form Page (Operator Satker)
//!
//! Operator Satker submits kebutuhan BMN within period constraints,
//! adds BMN items with justification, uploads supporting documents,
//! and submits to Validator Wilayah.

use crate::api::AppError;
use crate::components::layout::{
    EmptyState, ErrorState, FormField, LoadingState, PageLayout, SectionCard,
};
use chrono::NaiveDate;
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    let (active_pengajuan, set_active_pengajuan) = signal::<Option<PengajuanSummary>>(None);
    let (satker_detail, set_satker_detail) = signal::<Option<SatkerDetail>>(None);
    let (barang_items, set_barang_items) = signal::<Vec<BarangItem>>(Vec::new());
    let (loading, set_loading) = signal(true);
    let (fatal_error, set_fatal_error) = signal::<Option<AppError>>(None);
    let (action_error, set_action_error) = signal::<Option<String>>(None);
    let (success_message, set_success_message) = signal::<Option<String>>(None);

    let (nama_barang, set_nama_barang) = signal(String::new());
    let (kode_barang, set_kode_barang) = signal(String::new());
    let (jumlah, set_jumlah) = signal(1);
    let (justifikasi, set_justifikasi) = signal(String::new());
    let (submitting, set_submitting) = signal(false);

    Effect::new(move || {
        spawn_local(async move {
            set_loading.set(true);
            set_fatal_error.set(None);
            match fetch_active_pengajuan().await {
                Ok(pengajuan) => {
                    let pengajuan_id = pengajuan.id;
                    set_active_pengajuan.set(Some(pengajuan));
                    match fetch_satker_detail(pengajuan_id).await {
                        Ok(satker) => {
                            let satker_id = satker.id;
                            set_satker_detail.set(Some(satker));
                            match fetch_barang_items(satker_id).await {
                                Ok(items) => set_barang_items.set(items),
                                Err(e) => set_fatal_error.set(Some(e)),
                            }
                        }
                        Err(e) => set_fatal_error.set(Some(e)),
                    }
                }
                Err(e) => set_fatal_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    let handle_add_barang = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_action_error.set(None);
        set_success_message.set(None);

        let nama = nama_barang.get_untracked().trim().to_string();
        let kode = kode_barang.get_untracked().trim().to_string();
        let jml = jumlah.get_untracked();
        let just = justifikasi.get_untracked().trim().to_string();

        if nama.is_empty() {
            set_action_error.set(Some("Nama barang wajib diisi.".to_string()));
            return;
        }
        if just.is_empty() {
            set_action_error.set(Some("Justifikasi kebutuhan wajib diisi.".to_string()));
            return;
        }
        if jml < 1 {
            set_action_error.set(Some("Jumlah harus lebih dari 0.".to_string()));
            return;
        }

        let Some(satker) = satker_detail.get_untracked() else {
            set_action_error
                .set(Some("Data satker tidak tersedia, muat ulang halaman.".to_string()));
            return;
        };

        spawn_local(async move {
            set_submitting.set(true);
            let request = CreateBarangRequest {
                nama,
                kode_barang: if kode.is_empty() { None } else { Some(kode) },
                jumlah: jml,
                justifikasi: just,
            };

            match create_barang_item(satker.id, request).await {
                Ok(new_item) => {
                    let mut items = barang_items.get_untracked();
                    items.push(new_item);
                    set_barang_items.set(items);
                    set_nama_barang.set(String::new());
                    set_kode_barang.set(String::new());
                    set_jumlah.set(1);
                    set_justifikasi.set(String::new());
                    set_success_message.set(Some("Barang berhasil ditambahkan.".to_string()));
                }
                Err(e) => set_action_error.set(Some(e.user_message())),
            }
            set_submitting.set(false);
        });
    };

    let submit_to_wilayah = move |_| {
        set_action_error.set(None);
        set_success_message.set(None);

        let Some(satker) = satker_detail.get_untracked() else {
            set_action_error.set(Some("Data satker tidak tersedia.".to_string()));
            return;
        };

        if barang_items.get_untracked().is_empty() {
            set_action_error.set(Some("Minimal harus ada 1 barang untuk diajukan.".to_string()));
            return;
        }

        spawn_local(async move {
            set_submitting.set(true);
            let request = SubmitKebutuhanSatkerRequest {
                catatan: Some("Pengajuan kebutuhan BMN dari Operator Satker".to_string()),
            };

            match submit_satker_to_wilayah(satker.id, request).await {
                Ok(_) => {
                    set_success_message.set(Some(
                        "Pengajuan berhasil disubmit ke Validator Wilayah.".to_string(),
                    ));
                    if let Some(pengajuan) = active_pengajuan.get_untracked() {
                        if let Ok(satker) = fetch_satker_detail(pengajuan.id).await {
                            set_satker_detail.set(Some(satker));
                        }
                    }
                }
                Err(e) => set_action_error.set(Some(e.user_message())),
            }
            set_submitting.set(false);
        });
    };

    view! {
        <PageLayout
            title="Pengajuan Kebutuhan BMN"
            icon="fas fa-file-import"
            description="Operator Satker — tambahkan barang kebutuhan dan submit ke Validator Wilayah"
        >
            <Show when=move || success_message.get().is_some()>
                <div class="flex items-center gap-2 rounded-xl border border-success-500/30 bg-success-500/[0.08] px-4 py-3 text-sm text-success-300">
                    <i class="fas fa-check-circle"></i>
                    <span>{move || success_message.get().unwrap_or_default()}</span>
                </div>
            </Show>

            <Show when=move || action_error.get().is_some()>
                <div class="flex items-center gap-2 rounded-xl border border-danger-500/30 bg-danger-500/[0.08] px-4 py-3 text-sm text-danger-300">
                    <i class="fas fa-exclamation-circle"></i>
                    <span>{move || action_error.get().unwrap_or_default()}</span>
                </div>
            </Show>

            {move || {
                if loading.get() {
                    view! {
                        <LoadingState message="Memuat data pengajuan...".to_string() />
                    }.into_any()
                } else if let Some(err) = fatal_error.get() {
                    view! {
                        <ErrorState error=err title="Gagal memuat data pengajuan".to_string() />
                    }.into_any()
                } else if let Some(pengajuan) = active_pengajuan.get() {
                    view! {
                        <SubmissionContent
                            pengajuan=pengajuan
                            barang_items=barang_items
                            nama_barang=nama_barang
                            set_nama_barang=set_nama_barang
                            kode_barang=kode_barang
                            set_kode_barang=set_kode_barang
                            jumlah=jumlah
                            set_jumlah=set_jumlah
                            justifikasi=justifikasi
                            set_justifikasi=set_justifikasi
                            submitting=submitting
                            handle_add_barang=handle_add_barang
                            submit_to_wilayah=submit_to_wilayah
                        />
                    }.into_any()
                } else {
                    view! {
                        <EmptyState
                            title="Tidak ada periode aktif".to_string()
                            description="Tidak ada periode pengajuan kebutuhan BMN yang sedang terbuka. Silakan hubungi Validator Pusat.".to_string()
                            icon="fas fa-calendar-xmark".to_string()
                        />
                    }.into_any()
                }
            }}
        </PageLayout>
    }
}

#[component]
fn SubmissionContent(
    pengajuan: PengajuanSummary,
    barang_items: ReadSignal<Vec<BarangItem>>,
    nama_barang: ReadSignal<String>,
    set_nama_barang: WriteSignal<String>,
    kode_barang: ReadSignal<String>,
    set_kode_barang: WriteSignal<String>,
    jumlah: ReadSignal<i32>,
    set_jumlah: WriteSignal<i32>,
    justifikasi: ReadSignal<String>,
    set_justifikasi: WriteSignal<String>,
    submitting: ReadSignal<bool>,
    handle_add_barang: impl Fn(SubmitEvent) + Copy + Send + Sync + 'static,
    submit_to_wilayah: impl Fn(web_sys::MouseEvent) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let period_expired = {
        let today = chrono::Local::now().naive_local().date();
        today > pengajuan.tgl_selesai
    };

    view! {
        <SectionCard title="Informasi Periode" icon="fas fa-calendar-alt">
            <dl class="grid grid-cols-1 gap-4 md:grid-cols-2">
                <PeriodField label="Nama Periode" value=pengajuan.nama.clone() />
                <PeriodField label="Tahun Anggaran" value=pengajuan.tahun.to_string() />
                <PeriodField label="Tanggal Mulai" value=pengajuan.tgl_mulai.to_string() />
                <PeriodField label="Tanggal Selesai" value=pengajuan.tgl_selesai.to_string() />
            </dl>

            <Show when=move || period_expired>
                <div class="mt-4 flex items-start gap-2 rounded-xl border border-warning-500/30 bg-warning-500/[0.08] px-4 py-3 text-sm text-warning-300">
                    <i class="fas fa-triangle-exclamation mt-0.5"></i>
                    <span>"Periode pengajuan telah berakhir. Anda tidak dapat menambah atau mengubah data."</span>
                </div>
            </Show>
        </SectionCard>

        <SectionCard title="Tambah Barang Kebutuhan" icon="fas fa-plus">
            <form on:submit=handle_add_barang class="flex flex-col gap-5">
                <div class="grid grid-cols-1 gap-5 md:grid-cols-2">
                    <FormField label="Nama Barang" required=true full_width=true>
                        <input
                            type="text"
                            required
                            placeholder="Contoh: Laptop Dell Latitude 5420"
                            class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                            on:input=move |ev| set_nama_barang.set(event_target_value(&ev))
                            prop:value=move || nama_barang.get()
                        />
                    </FormField>

                    <FormField label="Kode Barang" helper="Opsional — sesuai standar kodefikasi.".to_string()>
                        <input
                            type="text"
                            placeholder="Contoh: 3.02.01.01.001"
                            class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                            on:input=move |ev| set_kode_barang.set(event_target_value(&ev))
                            prop:value=move || kode_barang.get()
                        />
                    </FormField>

                    <FormField label="Jumlah" required=true>
                        <input
                            type="number"
                            min="1"
                            required
                            class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100"
                            on:input=move |ev| {
                                if let Ok(num) = event_target_value(&ev).parse::<i32>() {
                                    set_jumlah.set(num);
                                }
                            }
                            prop:value=move || jumlah.get()
                        />
                    </FormField>

                    <FormField label="Justifikasi" required=true full_width=true>
                        <textarea
                            required
                            rows="4"
                            placeholder="Jelaskan alasan kebutuhan barang ini..."
                            class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                            on:input=move |ev| set_justifikasi.set(event_target_value(&ev))
                            prop:value=move || justifikasi.get()
                        ></textarea>
                    </FormField>
                </div>

                <div class="flex flex-wrap items-center justify-end gap-2 border-t border-white/[0.04] pt-4">
                    <button
                        type="submit"
                        class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-5 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90 disabled:opacity-50"
                        disabled=move || submitting.get() || period_expired
                    >
                        <i class="fas fa-plus"></i>
                        {move || if submitting.get() { "Menyimpan..." } else { "Tambah Barang" }}
                    </button>
                </div>
            </form>
        </SectionCard>

        <SectionCard
            title="Daftar Barang Kebutuhan".to_string()
            icon="fas fa-list".to_string()
            description=Signal::derive(move || {
                format!("{} item", barang_items.get().len())
            }).get()
        >
            {move || {
                let items = barang_items.get();
                if items.is_empty() {
                    view! {
                        <EmptyState
                            title="Belum ada barang".to_string()
                            description="Tambahkan minimal satu barang sebelum melakukan submit ke Validator Wilayah.".to_string()
                            icon="fas fa-box-open".to_string()
                        />
                    }.into_any()
                } else {
                    view! {
                        <div class="space-y-3">
                            {items.into_iter().enumerate().map(|(idx, item)| view! {
                                <div class="rounded-xl border border-white/[0.06] bg-white/[0.02] p-4 transition hover:border-white/10 hover:bg-white/[0.04]">
                                    <div class="flex items-start gap-3">
                                        <span class="mt-0.5 inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-lg bg-gold-500/15 text-xs font-bold text-gold-300 ring-1 ring-gold-500/25">
                                            {idx + 1}
                                        </span>
                                        <div class="flex-1">
                                            <h3 class="text-sm font-semibold text-slate-100">
                                                {item.nama.clone()}
                                            </h3>
                                            {item.kode_barang.as_ref().map(|kode| view! {
                                                <p class="mt-1 font-mono text-xs text-slate-400">
                                                    "Kode: " {kode.clone()}
                                                </p>
                                            })}
                                            <p class="mt-1 text-xs text-slate-400">
                                                "Jumlah: "
                                                <span class="font-semibold text-slate-200">{item.jumlah}</span>
                                            </p>
                                            <p class="mt-2 text-sm leading-relaxed text-slate-300">
                                                <span class="text-xs font-semibold uppercase tracking-wide text-slate-500">"Justifikasi: "</span>
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
        </SectionCard>

        <SectionCard title="Dokumen Pendukung" icon="fas fa-paperclip">
            <div class="flex flex-col items-center gap-3 rounded-xl border border-dashed border-white/10 bg-white/[0.02] px-6 py-10 text-center">
                <span class="flex h-12 w-12 items-center justify-center rounded-full bg-gold-500/10 text-gold-400 ring-1 ring-gold-500/20">
                    <i class="fas fa-cloud-arrow-up"></i>
                </span>
                <div>
                    <h3 class="text-sm font-semibold text-slate-100">"Upload surat permohonan dan lampiran"</h3>
                    <p class="mt-1 text-xs text-slate-400">"Format: PDF, DOC, DOCX, JPG, PNG. Maksimal 10MB per file."</p>
                </div>
                <p class="text-xs italic text-slate-500">"Fitur upload dokumen akan tersedia setelah integrasi dengan layanan dokumen selesai."</p>
            </div>
        </SectionCard>

        <SectionCard>
            <div class="flex flex-wrap items-center justify-end gap-3">
                <button
                    type="button"
                    class="rounded-lg border border-white/10 bg-white/[0.04] px-4 py-2 text-sm text-slate-300 transition hover:bg-white/[0.08]"
                    on:click=move |_| {
                        if let Some(window) = web_sys::window() {
                            if let Ok(history) = window.history() {
                                let _ = history.back();
                            }
                        }
                    }
                >
                    "Kembali"
                </button>

                <button
                    type="button"
                    class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-5 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90 disabled:opacity-50"
                    on:click=submit_to_wilayah
                    disabled=move || submitting.get() || barang_items.get().is_empty() || period_expired
                >
                    <i class="fas fa-paper-plane"></i>
                    {move || if submitting.get() { "Mengirim..." } else { "Submit ke Validator Wilayah" }}
                </button>
            </div>
            <p class="mt-3 text-right text-xs text-slate-500">
                "Setelah submit, pengajuan akan diteruskan ke Validator Wilayah untuk ditinjau."
            </p>
        </SectionCard>
    }
}

#[component]
fn PeriodField(#[prop(into)] label: String, #[prop(into)] value: String) -> impl IntoView {
    view! {
        <div>
            <dt class="text-xs font-semibold uppercase tracking-wide text-slate-500">{label}</dt>
            <dd class="mt-1 text-sm text-slate-100">{value}</dd>
        </div>
    }
}

// ============================================================================
// API Functions
// ============================================================================

async fn fetch_active_pengajuan() -> Result<PengajuanSummary, AppError> {
    let response = gloo_net::http::Request::get(
        "/api/v1/kebutuhan-bmn/pengajuan?status_kode=2001",
    )
    .send()
    .await?;

    if !response.ok() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::from_status(status, &body));
    }

    let api_response: ApiResponse<Vec<PengajuanSummary>> = response.json().await?;
    api_response
        .data
        .and_then(|mut v| v.pop())
        .ok_or_else(|| AppError::not_found("Tidak ada periode pengajuan yang aktif."))
}

async fn fetch_satker_detail(pengajuan_id: Uuid) -> Result<SatkerDetail, AppError> {
    let url = format!("/api/v1/kebutuhan-bmn/pengajuan/{}/satker", pengajuan_id);
    let response = gloo_net::http::Request::get(&url).send().await?;

    if !response.ok() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::from_status(status, &body));
    }

    let api_response: ApiResponse<Vec<SatkerDetail>> = response.json().await?;
    api_response
        .data
        .and_then(|mut v| v.pop())
        .ok_or_else(|| AppError::not_found("Data satker tidak ditemukan."))
}

async fn fetch_barang_items(satker_id: Uuid) -> Result<Vec<BarangItem>, AppError> {
    let url = format!("/api/v1/kebutuhan-bmn/satker/{}", satker_id);
    let response = gloo_net::http::Request::get(&url).send().await?;

    if !response.ok() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::from_status(status, &body));
    }

    let api_response: ApiResponse<Vec<BarangItem>> = response.json().await?;
    Ok(api_response.data.unwrap_or_default())
}

async fn create_barang_item(
    satker_id: Uuid,
    request: CreateBarangRequest,
) -> Result<BarangItem, AppError> {
    let url = format!("/api/v1/kebutuhan-bmn/satker/{}/barang", satker_id);
    let response = gloo_net::http::Request::post(&url)
        .json(&request)?
        .send()
        .await?;

    if !response.ok() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::from_status(status, &body));
    }

    let api_response: ApiResponse<BarangItem> = response.json().await?;
    api_response
        .data
        .ok_or_else(|| AppError::parse("Respons kosong dari server."))
}

async fn submit_satker_to_wilayah(
    satker_id: Uuid,
    request: SubmitKebutuhanSatkerRequest,
) -> Result<SatkerDetail, AppError> {
    let url = format!("/api/v1/kebutuhan-bmn/satker/{}/submit-wilayah", satker_id);
    let response = gloo_net::http::Request::post(&url)
        .json(&request)?
        .send()
        .await?;

    if !response.ok() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::from_status(status, &body));
    }

    let api_response: ApiResponse<SatkerDetail> = response.json().await?;
    api_response
        .data
        .ok_or_else(|| AppError::parse("Respons kosong dari server."))
}
