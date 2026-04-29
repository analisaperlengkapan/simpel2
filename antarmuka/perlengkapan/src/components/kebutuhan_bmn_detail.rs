//! Kebutuhan BMN Detail Component
//!
//! Displays detailed view of a BMN needs request with satker list,
//! goods breakdown, and workflow actions.

use crate::api::{
    AppError, KebutuhanBmnStatus, PengajuanDetailResponse, PengajuanKebutuhanBmnSatker,
    WorkflowTransitionRequest, delete_kebutuhan_bmn, fetch_kebutuhan_bmn_detail,
    transition_kebutuhan_bmn_status,
};
use crate::components::layout::{ErrorState, LoadingState, PageLayout, SectionCard};
use crate::routes;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{ARROW_LEFT, BUILDING, EYE, PENCIL_SIMPLE, TRASH, WARNING_CIRCLE};

#[component]
pub fn KebutuhanBmnDetail() -> impl IntoView {
    let params = use_params_map();
    let id = Memo::new(move |_| params.read().get("id").clone().unwrap_or_default());

    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<AppError>>(None);
    let (detail, set_detail) = signal::<Option<PengajuanDetailResponse>>(None);
    let (satkers, set_satkers) = signal::<Vec<PengajuanKebutuhanBmnSatker>>(vec![]);
    let (transitioning, set_transitioning) = signal(false);
    let (deleting, set_deleting) = signal(false);
    let (show_delete_modal, set_show_delete_modal) = signal(false);
    let (transition_comment, set_transition_comment) = signal(String::new());
    let (action_error, set_action_error) = signal::<Option<String>>(None);

    // Load detail data
    let load_data = move |pengajuan_id: String| {
        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            match fetch_kebutuhan_bmn_detail(&pengajuan_id).await {
                Ok(response) => {
                    set_satkers.set(response.data.satkers.clone());
                    set_detail.set(Some(response.data));
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    };

    // Initial load
    Effect::new(move || {
        let current_id = id.get();
        if !current_id.is_empty() {
            load_data(current_id);
        }
    });

    // Handle workflow transition
    let handle_transition = move |target_status: i32| {
        set_transitioning.set(true);
        set_action_error.set(None);
        let current_id = id.get();
        let comment = transition_comment.get();

        spawn_local(async move {
            let request = WorkflowTransitionRequest {
                target_status,
                komentar: if comment.is_empty() {
                    None
                } else {
                    Some(comment)
                },
            };

            match transition_kebutuhan_bmn_status(&current_id, request).await {
                Ok(response) => {
                    set_detail.set(Some(response.data));
                    set_transition_comment.set(String::new());
                }
                Err(e) => {
                    set_action_error.set(Some(e.user_message()));
                }
            }
            set_transitioning.set(false);
        });
    };

    // Handle delete
    let handle_delete = move |_| {
        set_deleting.set(true);
        set_action_error.set(None);
        let current_id = id.get();

        spawn_local(async move {
            match delete_kebutuhan_bmn(&current_id).await {
                Ok(_) => {
                    if let Some(window) = web_sys::window() {
                        let _ = window
                            .location()
                            .set_href(routes::path::KEBUTUHAN_DAFTAR_LEGACY);
                    }
                }
                Err(e) => {
                    set_action_error.set(Some(e.user_message()));
                    set_show_delete_modal.set(false);
                }
            }
            set_deleting.set(false);
        });
    };

    view! {
        <PageLayout
            title="Detail Pengajuan Kebutuhan BMN"
            icon="fas fa-clipboard-list"
            description="Detail dan status pengajuan analisis kebutuhan BMN"
        >
            // Back link
            <a
                href=routes::path::KEBUTUHAN_DAFTAR
                class="mb-4 inline-flex items-center gap-2 text-sm text-gold-400 transition hover:text-gold-300"
            >
                <span class="text-xs"><AppIcon icon=ARROW_LEFT /></span>
                "Kembali ke Daftar"
            </a>

            // Action error
            <Show when=move || action_error.get().is_some()>
                <div class="mb-4 flex items-center gap-2 rounded-xl border border-danger-500/30 bg-danger-500/[0.08] px-4 py-3 text-sm text-danger-300">
                    <AppIcon icon=WARNING_CIRCLE />
                    {move || action_error.get().unwrap_or_default()}
                </div>
            </Show>

            // Main content: loading / error / detail
            {move || {
                if loading.get() {
                    view! { <LoadingState message="Memuat data pengajuan...".to_string() /> }.into_any()
                } else if let Some(err) = error.get() {
                    view! { <ErrorState error=err /> }.into_any()
                } else if let Some(d) = detail.get() {
                    render_detail(
                        d,
                        satkers,
                        transitioning,
                        handle_transition,
                        transition_comment,
                        set_transition_comment,
                        set_show_delete_modal,
                    ).into_any()
                } else {
                    view! { <LoadingState /> }.into_any()
                }
            }}

            // Delete confirmation modal
            <Show when=move || show_delete_modal.get()>
                <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
                    <div class="mx-4 w-full max-w-md rounded-2xl border border-white/[0.06] bg-surface-panel p-6 shadow-xl">
                        <h3 class="mb-4 text-lg font-bold text-slate-100">"Konfirmasi Hapus"</h3>
                        <p class="mb-6 text-sm text-slate-400">
                            "Apakah Anda yakin ingin menghapus pengajuan ini? Tindakan ini tidak dapat dibatalkan."
                        </p>
                        <div class="flex justify-end gap-3">
                            <button
                                class="rounded-lg border border-white/10 bg-white/[0.04] px-4 py-2 text-sm text-slate-300 transition hover:bg-white/[0.08]"
                                on:click=move |_| set_show_delete_modal.set(false)
                            >
                                "Batal"
                            </button>
                            <button
                                class="rounded-lg bg-danger-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-danger-700 disabled:opacity-50"
                                disabled=move || deleting.get()
                                on:click=handle_delete
                            >
                                {move || if deleting.get() { "Menghapus..." } else { "Hapus" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </PageLayout>
    }
}

/// Renders the detail content once data is loaded.
fn render_detail(
    d: PengajuanDetailResponse,
    satkers: ReadSignal<Vec<PengajuanKebutuhanBmnSatker>>,
    transitioning: ReadSignal<bool>,
    handle_transition: impl Fn(i32) + Copy + Send + Sync + 'static,
    transition_comment: ReadSignal<String>,
    set_transition_comment: WriteSignal<String>,
    set_show_delete_modal: WriteSignal<bool>,
) -> impl IntoView {
    let pengajuan = d.pengajuan.clone();
    let status = KebutuhanBmnStatus::from_code(pengajuan.status_kode);
    let badge_class = status
        .map(|s| s.badge_class())
        .unwrap_or("bg-slate-500/15 text-slate-300 ring-1 ring-slate-500/25");
    let status_label = status.map(|s| s.label()).unwrap_or("Unknown");
    let allowed_transitions = StoredValue::new(d.allowed_transitions.clone());
    let has_deskripsi = pengajuan.deskripsi.is_some();
    let deskripsi_text = StoredValue::new(pengajuan.deskripsi.clone().unwrap_or_default());
    let pengajuan_id = pengajuan.id.clone();
    let satker_count = d.satkers.len();

    view! {
        <div class="space-y-5">
            // Title & Status header
            <div class="flex flex-col items-start justify-between gap-4 lg:flex-row lg:items-center">
                <div>
                    <h2 class="text-xl font-bold text-slate-100">{pengajuan.nama.clone()}</h2>
                    <p class="mt-1 text-sm text-slate-400">
                        "Tahun Anggaran: " <span class="font-medium text-slate-200">{pengajuan.tahun}</span>
                    </p>
                </div>
                <div class="flex items-center gap-3">
                    <span class=format!("inline-flex items-center rounded-full px-3 py-1 text-xs font-medium {}", badge_class)>
                        {status_label}
                    </span>
                    <div class="flex gap-2">
                        <a
                            href=routes::url::kebutuhan_edit(&pengajuan_id)
                            class="inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-300 transition hover:bg-white/[0.08]"
                        >
                            <span class="text-2xs"><AppIcon icon=PENCIL_SIMPLE /></span>
                            "Edit"
                        </a>
                        <button
                            class="rounded-lg border border-danger-500/30 bg-danger-500/10 px-3 py-1.5 text-xs text-danger-300 transition hover:bg-danger-500/20"
                            on:click=move |_| set_show_delete_modal.set(true)
                        >
                            <span class="text-2xs"><AppIcon icon=TRASH /></span>
                        </button>
                    </div>
                </div>
            </div>

            // Info stat cards
            <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
                <div class="rounded-xl border border-white/[0.06] bg-surface-panel p-4">
                    <div class="text-xs font-medium text-info-400">"Periode"</div>
                    <div class="mt-1 text-sm font-medium text-slate-200">
                        {pengajuan.tgl_mulai.clone()} " s.d. " {pengajuan.tgl_selesai.clone()}
                    </div>
                </div>
                <div class="rounded-xl border border-white/[0.06] bg-surface-panel p-4">
                    <div class="text-xs font-medium text-purple-400">"Total Satker"</div>
                    <div class="mt-1 text-2xl font-bold text-slate-100">{satker_count}</div>
                </div>
                <div class="rounded-xl border border-white/[0.06] bg-surface-panel p-4">
                    <div class="text-xs font-medium text-success-400">"Persetujuan DASKRIMTI"</div>
                    <div class="mt-1 text-sm font-medium text-slate-200">
                        {if pengajuan.is_appv_daskrimti { "Ya" } else { "Belum" }}
                    </div>
                </div>
                <div class="rounded-xl border border-white/[0.06] bg-surface-panel p-4">
                    <div class="text-xs font-medium text-gold-400">"Versi"</div>
                    <div class="mt-1 text-2xl font-bold text-slate-100">{pengajuan.version}</div>
                </div>
            </div>

            // Description
            <Show when=move || has_deskripsi>
                <SectionCard title="Deskripsi">
                    <p class="text-sm text-slate-300">{deskripsi_text.get_value()}</p>
                </SectionCard>
            </Show>

            // Workflow transitions
            <Show when=move || !allowed_transitions.with_value(|t| t.is_empty())>
                <SectionCard title="Aksi Workflow">
                    <div class="flex flex-wrap gap-3">
                        <For
                            each=move || allowed_transitions.get_value()
                            key=|t| t.status_kode
                            children=move |transition| {
                                let status_kode = transition.status_kode;
                                let btn_class = transition_btn_class(status_kode);
                                view! {
                                    <button
                                        class=format!("rounded-lg px-4 py-2 text-sm font-medium transition disabled:opacity-50 {}", btn_class)
                                        disabled=move || transitioning.get()
                                        on:click=move |_| handle_transition(status_kode)
                                    >
                                        {transition.status_nama.clone()}
                                    </button>
                                }
                            }
                        />
                    </div>
                    <div class="mt-3">
                        <input
                            type="text"
                            placeholder="Komentar (opsional)"
                            class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                            on:input=move |ev| set_transition_comment.set(event_target_value(&ev))
                            prop:value=move || transition_comment.get()
                        />
                    </div>
                </SectionCard>
            </Show>

            // Satker list table
            <SectionCard title="Daftar Satker">
                {move || {
                    let s = satkers.get();
                    if s.is_empty() {
                        view! {
                            <div class="py-8 text-center text-sm text-slate-500">
                                <span class="mb-2 text-2xl text-slate-600"><AppIcon icon=BUILDING /></span>
                                <p>"Belum ada satker yang terdaftar"</p>
                            </div>
                        }.into_any()
                    } else {
                        render_satker_table(s).into_any()
                    }
                }}
            </SectionCard>
        </div>
    }
}

/// Button class based on transition target status code.
fn transition_btn_class(status_kode: i32) -> &'static str {
    match status_kode {
        2006 => "bg-success-600 text-white hover:bg-success-700",
        2007 | 2009 => "bg-danger-600 text-white hover:bg-danger-700",
        _ => "bg-info-600 text-white hover:bg-info-700",
    }
}

/// Renders the satker table rows.
fn render_satker_table(satkers: Vec<PengajuanKebutuhanBmnSatker>) -> impl IntoView {
    view! {
        <div class="overflow-hidden rounded-xl border border-white/[0.06]">
            <div class="overflow-x-auto">
                <table class="min-w-full divide-y divide-white/[0.04]">
                    <thead class="bg-white/[0.02]">
                        <tr>
                            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">"Nama Satker"</th>
                            <th class="px-4 py-3 text-center text-xs font-semibold uppercase tracking-wide text-slate-400">"Status"</th>
                            <th class="px-4 py-3 text-center text-xs font-semibold uppercase tracking-wide text-slate-400">"Prioritas"</th>
                            <th class="px-4 py-3 text-center text-xs font-semibold uppercase tracking-wide text-slate-400">"Aksi"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {satkers.into_iter().enumerate().map(|(idx, satker)| {
                            let status = KebutuhanBmnStatus::from_code(satker.status_kode);
                            let badge_class = status.map(|s| s.badge_class()).unwrap_or("bg-slate-500/15 text-slate-300 ring-1 ring-slate-500/25");
                            let status_label = status.map(|s| s.label()).unwrap_or("Unknown");
                            let satker_id = satker.id.clone();
                            let nm = satker.nm_satker.unwrap_or_else(|| satker.ms_satker_id.clone());
                            let bg = if idx % 2 == 0 { "bg-transparent" } else { "bg-white/[0.015]" };
                            view! {
                                <tr class=format!("border-b border-white/[0.04] {}", bg)>
                                    <td class="px-4 py-3 text-sm font-medium text-slate-200">{nm}</td>
                                    <td class="px-4 py-3 text-center">
                                        <span class=format!("inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {}", badge_class)>
                                            {status_label}
                                        </span>
                                    </td>
                                    <td class="px-4 py-3 text-center text-sm text-slate-300">{satker.prioritas}</td>
                                    <td class="px-4 py-3 text-center">
                                        <a
                                            href=routes::url::kebutuhan_satker_detail(&satker_id)
                                            class="inline-flex items-center gap-1.5 text-xs text-info-400 transition hover:text-info-300"
                                        >
                                            <span class="text-2xs"><AppIcon icon=EYE /></span>
                                            "Detail"
                                        </a>
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
