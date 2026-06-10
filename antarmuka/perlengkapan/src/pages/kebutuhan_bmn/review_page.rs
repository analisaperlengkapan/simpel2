//! # Kebutuhan BMN Review Page (Validator Wilayah)
//!
//! Validator Wilayah reviews submissions from Operator Satker,
//! then either forwards to Validator Pusat or returns for revision.

use crate::api::AppError;
use crate::components::layout::{
    EmptyState, ErrorState, FormField, LoadingState, PageLayout, SectionCard,
};
use chrono::{DateTime, NaiveDate, Utc};
use leptos::prelude::*;
use leptos::task::spawn_local;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{
    ARROW_COUNTER_CLOCKWISE, CHECK_CIRCLE, CLOCK, FILE_ARROW_DOWN, PAPER_PLANE_TILT,
    WARNING_CIRCLE, X,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// API Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatkerSubmission {
    pub id: Uuid,
    pub pengajuan_id: Uuid,
    pub satker_id: String,
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
    pub satker_id: String,
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
    pub action: String,
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
    let (submissions, set_submissions) = signal::<Vec<SatkerSubmission>>(Vec::new());
    let (selected_submission, set_selected_submission) = signal::<Option<SatkerDetail>>(None);
    let (list_loading, set_list_loading) = signal(true);
    let (list_error, set_list_error) = signal::<Option<AppError>>(None);
    let (detail_loading, set_detail_loading) = signal(false);
    let (detail_error, set_detail_error) = signal::<Option<AppError>>(None);
    let (action_loading, set_action_loading) = signal(false);
    let (action_error, set_action_error) = signal::<Option<String>>(None);
    let (success_message, set_success_message) = signal::<Option<String>>(None);
    let (show_action_modal, set_show_action_modal) = signal(false);
    let (action_type, set_action_type) = signal::<Option<String>>(None);
    let (catatan, set_catatan) = signal(String::new());

    let load_submissions = move || {
        spawn_local(async move {
            set_list_loading.set(true);
            set_list_error.set(None);
            match fetch_submissions_for_review().await {
                Ok(subs) => set_submissions.set(subs),
                Err(e) => set_list_error.set(Some(e)),
            }
            set_list_loading.set(false);
        });
    };

    Effect::new(move |_| {
        load_submissions();
    });

    let load_submission_detail = move |submission_id: Uuid| {
        spawn_local(async move {
            set_detail_loading.set(true);
            set_detail_error.set(None);
            match fetch_satker_detail(submission_id).await {
                Ok(detail) => set_selected_submission.set(Some(detail)),
                Err(e) => set_detail_error.set(Some(e)),
            }
            set_detail_loading.set(false);
        });
    };

    let open_action_modal = move |action: String| {
        set_action_type.set(Some(action));
        set_catatan.set(String::new());
        set_action_error.set(None);
        set_show_action_modal.set(true);
    };

    let submit_action = move |_| {
        let Some(action) = action_type.get_untracked() else {
            return;
        };
        let Some(submission) = selected_submission.get_untracked() else {
            set_action_error.set(Some("Tidak ada pengajuan yang dipilih.".to_string()));
            return;
        };
        let catatan_text = catatan.get_untracked();
        if catatan_text.trim().is_empty() {
            set_action_error.set(Some("Catatan wajib diisi.".to_string()));
            return;
        }

        spawn_local(async move {
            set_action_loading.set(true);
            set_action_error.set(None);

            let request = ValidatorWilayahActionRequest {
                action: action.clone(),
                catatan: catatan_text,
            };

            match submit_validator_wilayah_action(submission.id, request).await {
                Ok(_) => {
                    let message = if action == "forward" {
                        "Pengajuan berhasil diteruskan ke Validator Pusat.".to_string()
                    } else {
                        "Pengajuan dikembalikan ke Operator Satker untuk revisi.".to_string()
                    };
                    set_success_message.set(Some(message));
                    set_show_action_modal.set(false);
                    set_selected_submission.set(None);

                    if let Ok(subs) = fetch_submissions_for_review().await {
                        set_submissions.set(subs);
                    }
                }
                Err(e) => set_action_error.set(Some(e.user_message())),
            }
            set_action_loading.set(false);
        });
    };

    view! {
        <PageLayout
            title="Review Kebutuhan BMN"
            icon="fas fa-clipboard-check"
            description="Validator Wilayah — tinjau pengajuan dan teruskan ke Validator Pusat atau kembalikan untuk revisi."
        >
            <Show when=move || success_message.get().is_some()>
                <div class="flex items-center gap-2 rounded-xl border border-success-500/30 bg-success-500/[0.08] px-4 py-3 text-sm text-success-300">
                    <AppIcon icon=CHECK_CIRCLE />
                    <span>{move || success_message.get().unwrap_or_default()}</span>
                </div>
            </Show>

            <Show when=move || action_error.get().is_some() && !show_action_modal.get()>
                <div class="flex items-center gap-2 rounded-xl border border-danger-500/30 bg-danger-500/[0.08] px-4 py-3 text-sm text-danger-300">
                    <AppIcon icon=WARNING_CIRCLE />
                    <span>{move || action_error.get().unwrap_or_default()}</span>
                </div>
            </Show>

            <div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
                <div class="lg:col-span-1">
                    <SectionCard title="Daftar Pengajuan" icon="fas fa-inbox">
                        {move || {
                            if list_loading.get() {
                                view! {
                                    <LoadingState message="Memuat pengajuan...".to_string() inline=true />
                                }.into_any()
                            } else if let Some(err) = list_error.get() {
                                view! {
                                    <ErrorState
                                        error=err
                                        title="Gagal memuat daftar".to_string()
                                        on_retry=Box::new(move || load_submissions())
                                    />
                                }.into_any()
                            } else {
                                let subs = submissions.get();
                                if subs.is_empty() {
                                    view! {
                                        <EmptyState
                                            title="Tidak ada pengajuan".to_string()
                                            description="Tidak ada pengajuan yang perlu direview saat ini.".to_string()
                                            icon="fas fa-inbox".to_string()
                                        />
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="flex flex-col gap-2">
                                            {subs.into_iter().map(|sub| {
                                                let sub_id = sub.id;
                                                let status_label = sub
                                                    .status_nama
                                                    .clone()
                                                    .unwrap_or_else(|| format!("Kode {}", sub.status_kode));
                                                let satker_label = sub
                                                    .satker_nama
                                                    .clone()
                                                    .unwrap_or_else(|| sub.satker_id.clone());
                                                let created_label = sub.created_at.format("%d %b %Y %H:%M").to_string();
                                                let is_selected = Signal::derive(move || {
                                                    selected_submission
                                                        .get()
                                                        .map(|s| s.id == sub_id)
                                                        .unwrap_or(false)
                                                });

                                                view! {
                                                    <button
                                                        type="button"
                                                        class=move || {
                                                            let base = "w-full rounded-xl border p-4 text-left transition";
                                                            if is_selected.get() {
                                                                format!("{} border-gold-500/40 bg-gold-500/10 ring-1 ring-gold-500/25", base)
                                                            } else {
                                                                format!("{} border-white/[0.06] bg-white/[0.02] hover:border-white/10 hover:bg-white/[0.04]", base)
                                                            }
                                                        }
                                                        on:click=move |_| load_submission_detail(sub_id)
                                                    >
                                                        <div class="text-sm font-semibold text-slate-100">
                                                            {satker_label}
                                                        </div>
                                                        <div class="mt-1 text-xs text-slate-400">
                                                            "Status: "
                                                            <span class="font-medium text-info-300">{status_label}</span>
                                                        </div>
                                                        <div class="mt-1 text-xs text-slate-500">
                                                            <span class="mr-1"><AppIcon icon=CLOCK /></span>
                                                            {created_label}
                                                        </div>
                                                    </button>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                }
                            }
                        }}
                    </SectionCard>
                </div>

                <div class="lg:col-span-2">
                    {move || {
                        if detail_loading.get() {
                            view! {
                                <SectionCard>
                                    <LoadingState message="Memuat detail pengajuan...".to_string() />
                                </SectionCard>
                            }.into_any()
                        } else if let Some(err) = detail_error.get() {
                            view! {
                                <SectionCard>
                                    <ErrorState
                                        error=err
                                        title="Gagal memuat detail pengajuan".to_string()
                                    />
                                </SectionCard>
                            }.into_any()
                        } else if let Some(detail) = selected_submission.get() {
                            view! {
                                <DetailPanel
                                    detail=detail
                                    action_loading=action_loading
                                    on_forward=Callback::new(move |_| open_action_modal("forward".to_string()))
                                    on_return=Callback::new(move |_| open_action_modal("return".to_string()))
                                />
                            }.into_any()
                        } else {
                            view! {
                                <SectionCard>
                                    <EmptyState
                                        title="Pilih pengajuan".to_string()
                                        description="Pilih pengajuan dari daftar di sebelah kiri untuk melihat detail dan melakukan tindakan.".to_string()
                                        icon="fas fa-hand-pointer".to_string()
                                    />
                                </SectionCard>
                            }.into_any()
                        }
                    }}
                </div>
            </div>

            <Show when=move || show_action_modal.get()>
                <ActionModal
                    action_type=action_type
                    catatan=catatan
                    set_catatan=set_catatan
                    action_error=action_error
                    action_loading=action_loading
                    on_close=Callback::new(move |_| set_show_action_modal.set(false))
                    on_submit=Callback::new(submit_action)
                />
            </Show>
        </PageLayout>
    }
}

#[component]
fn DetailPanel(
    detail: SatkerDetail,
    action_loading: ReadSignal<bool>,
    on_forward: Callback<()>,
    on_return: Callback<()>,
) -> impl IntoView {
    let status_label = detail
        .status_nama
        .clone()
        .unwrap_or_else(|| format!("Kode {}", detail.status_kode));
    let satker_label = detail
        .satker_nama
        .clone()
        .unwrap_or_else(|| detail.satker_id.clone());
    let barang_items = detail.barang_items.clone();
    let has_docs = barang_items
        .iter()
        .any(|item| !item.file_pendukung.is_empty());
    let aktivitas_history = detail.aktivitas_history.clone();
    let barang_count = barang_items.len();

    view! {
        <div class="flex flex-col gap-6">
            <SectionCard title="Informasi Satker" icon="fas fa-building">
                <dl class="grid grid-cols-1 gap-4 md:grid-cols-2">
                    <InfoField label="Nama Satker" value=satker_label />
                    <InfoField label="Kode Satker" value=detail.satker_id.clone() />
                    <div>
                        <dt class="text-xs font-semibold uppercase tracking-wide text-slate-500">"Status"</dt>
                        <dd class="mt-1">
                            <span class="inline-flex items-center gap-1.5 rounded-full bg-info-500/15 px-2.5 py-1 text-xs font-semibold text-info-300 ring-1 ring-info-500/25">
                                {status_label}
                            </span>
                        </dd>
                    </div>
                    <InfoField label="Prioritas" value=detail.prioritas.to_string() />
                </dl>
            </SectionCard>

            <SectionCard
                title="Daftar Barang Kebutuhan".to_string()
                icon="fas fa-box".to_string()
                description=format!("{} item diajukan", barang_count)
            >
                {
                    if barang_items.is_empty() {
                        view! {
                            <EmptyState
                                title="Belum ada barang".to_string()
                                description="Pengajuan ini tidak memiliki daftar barang.".to_string()
                                icon="fas fa-box-open".to_string()
                            />
                        }.into_any()
                    } else {
                        view! { <BarangTable items=barang_items.clone() /> }.into_any()
                    }
                }
            </SectionCard>

            <Show when=move || has_docs>
                {
                    let docs_items = detail.barang_items.clone();
                    view! {
                        <SectionCard title="Dokumen Pendukung" icon="fas fa-paperclip">
                            <div class="flex flex-col gap-3">
                                {docs_items
                                    .into_iter()
                                    .filter(|item| !item.file_pendukung.is_empty())
                                    .map(|item| view! {
                                        <div class="rounded-xl border-l-4 border-gold-400/60 bg-white/[0.02] px-4 py-3">
                                            <p class="text-sm font-semibold text-slate-100">{item.nama.clone()}</p>
                                            <div class="mt-2 flex flex-col gap-1">
                                                {item.file_pendukung.iter().map(|file| view! {
                                                    <a
                                                        href=file.clone()
                                                        target="_blank"
                                                        rel="noopener"
                                                        class="inline-flex items-center gap-2 text-xs font-medium text-gold-300 transition hover:text-gold-200"
                                                    >
                                                        <AppIcon icon=FILE_ARROW_DOWN />
                                                        <span>"Unduh dokumen"</span>
                                                    </a>
                                                }).collect_view()}
                                            </div>
                                        </div>
                                    }).collect_view()}
                            </div>
                        </SectionCard>
                    }
                }
            </Show>

            <SectionCard title="Riwayat Aktivitas" icon="fas fa-clock-rotate-left">
                {
                    if aktivitas_history.is_empty() {
                        view! {
                            <EmptyState
                                title="Belum ada aktivitas".to_string()
                                description="Belum ada riwayat tindakan pada pengajuan ini.".to_string()
                                icon="fas fa-clock-rotate-left".to_string()
                            />
                        }.into_any()
                    } else {
                        view! {
                            <ol class="flex flex-col gap-3 border-l border-white/[0.08] pl-4">
                                {aktivitas_history.into_iter().map(|aktivitas| {
                                    let name = aktivitas
                                        .aktivitas_nama
                                        .clone()
                                        .unwrap_or_else(|| format!("Aktivitas {}", aktivitas.aktivitas_id));
                                    let time = aktivitas.created_at.format("%d %b %Y %H:%M").to_string();
                                    view! {
                                        <li class="relative">
                                            <span class="absolute -left-[1.1rem] top-1.5 inline-block h-2 w-2 rounded-full bg-gold-400 ring-2 ring-navy-950"></span>
                                            <div class="flex items-center justify-between gap-3">
                                                <p class="text-sm font-semibold text-slate-100">{name}</p>
                                                <span class="text-xs text-slate-500">{time}</span>
                                            </div>
                                            {aktivitas.user_nama.as_ref().map(|user| view! {
                                                <p class="mt-1 text-xs text-slate-400">
                                                    "Oleh: "<span class="font-medium text-slate-300">{user.clone()}</span>
                                                </p>
                                            })}
                                            {aktivitas.catatan.as_ref().map(|catatan| view! {
                                                <p class="mt-1 text-xs italic leading-relaxed text-slate-400">
                                                    "\"" {catatan.clone()} "\""
                                                </p>
                                            })}
                                        </li>
                                    }
                                }).collect_view()}
                            </ol>
                        }.into_any()
                    }
                }
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
                        class="inline-flex items-center gap-2 rounded-lg border border-danger-500/40 bg-danger-500/10 px-4 py-2 text-sm font-medium text-danger-200 transition hover:bg-danger-500/20 disabled:opacity-50"
                        on:click=move |_| on_return.run(())
                        disabled=move || action_loading.get()
                    >
                        <AppIcon icon=ARROW_COUNTER_CLOCKWISE />
                        "Kembalikan untuk Revisi"
                    </button>
                    <button
                        type="button"
                        class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-5 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90 disabled:opacity-50"
                        on:click=move |_| on_forward.run(())
                        disabled=move || action_loading.get()
                    >
                        <AppIcon icon=PAPER_PLANE_TILT />
                        "Teruskan ke Validator Pusat"
                    </button>
                </div>
            </SectionCard>
        </div>
    }
}

#[component]
fn BarangTable(items: Vec<BarangItem>) -> impl IntoView {
    view! {
        <div class="overflow-hidden rounded-xl border border-white/[0.06]">
            <div class="overflow-x-auto">
                <table class="min-w-full divide-y divide-white/[0.04]">
                    <thead class="bg-white/[0.02]">
                        <tr>
                            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">"No"</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">"Nama Barang"</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">"Kode"</th>
                            <th class="px-4 py-3 text-right text-xs font-semibold uppercase tracking-wide text-slate-400">"Jumlah"</th>
                            <th class="px-4 py-3 text-right text-xs font-semibold uppercase tracking-wide text-slate-400">"Existing"</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">"Justifikasi"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {items.into_iter().enumerate().map(|(idx, item)| {
                            let bg = if idx % 2 == 0 { "bg-transparent" } else { "bg-white/[0.015]" };
                            view! {
                                <tr class=format!("border-b border-white/[0.04] {}", bg)>
                                    <td class="px-4 py-3 text-sm text-slate-400">{idx + 1}</td>
                                    <td class="px-4 py-3 text-sm font-medium text-slate-100">{item.nama.clone()}</td>
                                    <td class="px-4 py-3 font-mono text-xs text-slate-400">
                                        {item.kode_barang.clone().unwrap_or_else(|| "—".to_string())}
                                    </td>
                                    <td class="px-4 py-3 text-right text-sm text-slate-200">{item.jumlah}</td>
                                    <td class="px-4 py-3 text-right text-sm text-info-300">{item.existing_count}</td>
                                    <td class="px-4 py-3 text-sm text-slate-300">
                                        {item.justifikasi.clone().unwrap_or_else(|| "—".to_string())}
                                    </td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[component]
fn InfoField(#[prop(into)] label: String, #[prop(into)] value: String) -> impl IntoView {
    view! {
        <div>
            <dt class="text-xs font-semibold uppercase tracking-wide text-slate-500">{label}</dt>
            <dd class="mt-1 text-sm text-slate-100">{value}</dd>
        </div>
    }
}

#[component]
fn ActionModal(
    action_type: ReadSignal<Option<String>>,
    catatan: ReadSignal<String>,
    set_catatan: WriteSignal<String>,
    action_error: ReadSignal<Option<String>>,
    action_loading: ReadSignal<bool>,
    on_close: Callback<()>,
    on_submit: Callback<()>,
) -> impl IntoView {
    let is_forward = move || matches!(action_type.get().as_deref(), Some("forward"));
    let title = move || {
        if is_forward() {
            "Teruskan ke Validator Pusat".to_string()
        } else {
            "Kembalikan untuk Revisi".to_string()
        }
    };
    let description = move || {
        if is_forward() {
            "Pengajuan akan diteruskan ke Validator Pusat untuk dianalisis lebih lanjut."
                .to_string()
        } else {
            "Pengajuan akan dikembalikan ke Operator Satker untuk direvisi.".to_string()
        }
    };
    let submit_label = move || {
        if is_forward() {
            "Teruskan".to_string()
        } else {
            "Kembalikan".to_string()
        }
    };
    let submit_class = move || {
        if is_forward() {
            "inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-5 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90 disabled:opacity-50".to_string()
        } else {
            "inline-flex items-center gap-2 rounded-lg bg-warning-600 px-5 py-2.5 text-sm font-bold text-white shadow-sm transition hover:bg-warning-700 disabled:opacity-50".to_string()
        }
    };

    view! {
        <div class="fixed inset-0 z-modal flex items-center justify-center bg-black/60 backdrop-blur-sm">
            <div class="mx-4 w-full max-w-lg rounded-2xl border border-white/[0.06] bg-surface-panel p-6 shadow-xl">
                <div class="flex items-start justify-between gap-3">
                    <h3 class="text-lg font-bold text-slate-100">{title}</h3>
                    <button
                        type="button"
                        class="rounded-lg p-1.5 text-slate-400 transition hover:bg-white/[0.05] hover:text-slate-200"
                        on:click=move |_| on_close.run(())
                        aria-label="Tutup"
                    >
                        <AppIcon icon=X />
                    </button>
                </div>
                <p class="mt-2 text-sm leading-relaxed text-slate-400">{description}</p>

                <Show when=move || action_error.get().is_some()>
                    <div class="mt-4 flex items-center gap-2 rounded-xl border border-danger-500/30 bg-danger-500/[0.08] px-4 py-3 text-sm text-danger-300">
                        <AppIcon icon=WARNING_CIRCLE />
                        <span>{move || action_error.get().unwrap_or_default()}</span>
                    </div>
                </Show>

                <div class="mt-5">
                    <FormField label="Catatan" required=true full_width=true>
                        <textarea
                            required
                            rows="4"
                            placeholder="Masukkan catatan untuk tindakan ini..."
                            class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                            on:input=move |ev| set_catatan.set(event_target_value(&ev))
                            prop:value=move || catatan.get()
                        ></textarea>
                    </FormField>
                </div>

                <div class="mt-6 flex justify-end gap-3 border-t border-white/[0.04] pt-4">
                    <button
                        type="button"
                        class="rounded-lg border border-white/10 bg-white/[0.04] px-4 py-2 text-sm text-slate-300 transition hover:bg-white/[0.08]"
                        on:click=move |_| on_close.run(())
                    >
                        "Batal"
                    </button>
                    <button
                        type="button"
                        class=submit_class
                        on:click=move |_| on_submit.run(())
                        disabled=move || action_loading.get()
                    >
                        {move || if action_loading.get() {
                            "Memproses...".to_string()
                        } else {
                            submit_label()
                        }}
                    </button>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// API Functions
// ============================================================================

async fn fetch_submissions_for_review() -> Result<Vec<SatkerSubmission>, AppError> {
    // status_kode = 2002 (SUBMIT_SATKER — menunggu Validator Wilayah)
    let response = gloo_net::http::Request::get("/api/v1/perlengkapan/kebutuhan-bmn/satker?status_kode=2002")
        .send()
        .await?;

    if !response.ok() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::from_status(status, &body));
    }

    let api_response: ApiResponse<Vec<SatkerSubmission>> = response.json().await?;
    Ok(api_response.data.unwrap_or_default())
}

async fn fetch_satker_detail(satker_id: Uuid) -> Result<SatkerDetail, AppError> {
    let url = format!("/api/v1/perlengkapan/kebutuhan-bmn/satker/{}", satker_id);
    let response = gloo_net::http::Request::get(&url).send().await?;

    if !response.ok() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::from_status(status, &body));
    }

    let api_response: ApiResponse<SatkerDetail> = response.json().await?;
    api_response
        .data
        .ok_or_else(|| AppError::not_found("Detail satker tidak ditemukan."))
}

async fn submit_validator_wilayah_action(
    satker_id: Uuid,
    request: ValidatorWilayahActionRequest,
) -> Result<SatkerDetail, AppError> {
    let url = format!(
        "/api/v1/perlengkapan/kebutuhan-bmn/satker/{}/validator-wilayah",
        satker_id
    );
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
