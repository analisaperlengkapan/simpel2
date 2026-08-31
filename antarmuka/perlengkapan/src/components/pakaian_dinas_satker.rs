//! Pakaian Dinas — per-satker workflow detail (#40).
//!
//! For a given pengajuan period, lists each satker submission and lets a
//! validator drill into its workflow: a [`WorkflowTimeline`] built from the
//! real per-satker activity history (`GET /pakaian-dinas/satker/{id}/aktivitas`)
//! plus approve/return actions via [`ApprovalDialog`]. Validator Wilayah acts
//! on submissions pending wilayah review; Validator Pusat on those forwarded to
//! pusat. Everyone else sees the timeline read-only.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_fetch::QueryClient;
use leptos_router::hooks::use_params_map;

use crate::api::{
    AppError, PaginatedResponse, PengajuanSatker, PengajuanSatkerAktivitas, ValidatorActionRequest,
    aktivitas_is_revisi, aktivitas_label, fetch_pakaian_satker_aktivitas, fetch_pengajuan_satker,
    process_validator_action,
};
use crate::components::layout::{ErrorState, LoadingState, PageLayout, SectionCard};
use crate::components::workflow_ui::{
    ActionTone, ApprovalDialog, StepStatus, WorkflowAction, WorkflowActions, WorkflowStep,
    WorkflowTimeline,
};
use crate::features::auth::AuthService;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::PENCIL_SIMPLE;

/// Status badge tailored to a workflow code.
fn status_badge(code: i32) -> impl IntoView {
    let (class, label) = if code == 1008 {
        (
            "bg-success-500/15 text-success-300 ring-success-500/25",
            "Selesai",
        )
    } else if aktivitas_is_revisi(code) {
        (
            "bg-danger-500/15 text-danger-300 ring-danger-500/25",
            "Perlu Revisi",
        )
    } else if code == 1000 {
        (
            "bg-slate-500/15 text-slate-300 ring-slate-500/25",
            "Penyiapan",
        )
    } else {
        (
            "bg-gold-500/15 text-gold-300 ring-gold-500/25",
            aktivitas_label(code),
        )
    };
    view! {
        <span class=format!(
            "inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ring-1 {}",
            class,
        )>{label}</span>
    }
}

/// Build the workflow timeline from the satker baseline + real activity rows.
fn build_timeline(
    satker: &PengajuanSatker,
    acts: &[PengajuanSatkerAktivitas],
) -> Vec<WorkflowStep> {
    let mut steps = vec![
        WorkflowStep::new("Pengajuan Dibuat", StepStatus::Done)
            .with_timestamp(satker.created_at.clone()),
    ];

    if acts.is_empty() {
        let st = if satker.aktivitas_id == 1008 {
            StepStatus::Done
        } else if aktivitas_is_revisi(satker.aktivitas_id) {
            StepStatus::Rejected
        } else {
            StepStatus::Current
        };
        steps.push(WorkflowStep::new(aktivitas_label(satker.aktivitas_id), st));
        return steps;
    }

    let last = acts.len() - 1;
    for (i, a) in acts.iter().enumerate() {
        let st = if aktivitas_is_revisi(a.aktivitas_id) {
            StepStatus::Rejected
        } else if a.aktivitas_id == 1008 {
            StepStatus::Done
        } else if i == last {
            StepStatus::Current
        } else {
            StepStatus::Done
        };
        let actor = match (&a.nama, &a.role) {
            (Some(n), Some(r)) => Some(format!("{n} · {r}")),
            (Some(n), None) => Some(n.clone()),
            (None, Some(r)) => Some(r.clone()),
            (None, None) => None,
        };
        let mut step = WorkflowStep::new(aktivitas_label(a.aktivitas_id), st)
            .with_timestamp(a.created_at.clone());
        if let Some(actor) = actor {
            step = step.with_actor(actor);
        }
        steps.push(step.with_note(a.komentar.clone()));
    }
    steps
}

/// Which validator actions the current user may take on `code`.
/// Returns `(can_act, approve_label, reject_label)`.
fn available_actions(code: i32) -> Option<(&'static str, &'static str)> {
    let session = AuthService::load_session()?;
    let is_admin = session.is_admin();
    // Wilayah review pending (1001 / 1012)
    if matches!(code, 1001 | 1012) && (session.is_validator_wilayah() || is_admin) {
        return Some(("Teruskan ke Pusat", "Kembalikan untuk Revisi"));
    }
    // Pusat review pending (1004 / 1010)
    if matches!(code, 1004 | 1010) && (session.is_validator_pusat() || is_admin) {
        return Some(("Setujui (Selesai)", "Kembalikan ke Wilayah"));
    }
    None
}

async fn query_satker_list(
    key: (String, i32),
) -> Result<PaginatedResponse<PengajuanSatker>, AppError> {
    let (pengajuan_id, _refresh) = key;
    fetch_pengajuan_satker(pengajuan_id, 1, 100).await
}

async fn query_satker_aktivitas(
    key: (String, i32),
) -> Result<Vec<PengajuanSatkerAktivitas>, AppError> {
    let (satker_id, _refresh) = key;
    if satker_id.is_empty() {
        return Ok(Vec::new());
    }
    fetch_pakaian_satker_aktivitas(satker_id).await
}

/// A pending validator action awaiting note confirmation.
#[derive(Clone)]
struct PendingAction {
    satker_id: String,
    aksi: String,
    title: String,
    tone: ActionTone,
    require_note: bool,
}

#[component]
pub fn PakaianDinasSatkerDetail() -> impl IntoView {
    let params = use_params_map();
    let pengajuan_id = move || {
        params.with(|p| {
            p.get("pengajuan_id")
                .map(|s| s.to_string())
                .unwrap_or_default()
        })
    };

    let refresh = RwSignal::new(0);
    let selected = RwSignal::new(Option::<PengajuanSatker>::None);
    let pending = RwSignal::new(Option::<PendingAction>::None);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    let client: QueryClient = expect_context();
    let list_resource =
        client.local_resource(query_satker_list, move || (pengajuan_id(), refresh.get()));
    let akt_resource = client.local_resource(query_satker_aktivitas, move || {
        (
            selected.get().map(|s| s.id).unwrap_or_default(),
            refresh.get(),
        )
    });

    // Confirm a pending action with the note from the dialog.
    let on_confirm = Callback::new(move |note: String| {
        let Some(act) = pending.get() else { return };
        busy.set(true);
        error.set(None);
        let req = ValidatorActionRequest {
            pengajuan_satker_id: act.satker_id.clone(),
            aksi: act.aksi.clone(),
            komentar: if note.trim().is_empty() {
                None
            } else {
                Some(note)
            },
        };
        spawn_local(async move {
            match process_validator_action(req).await {
                Ok(_) => {
                    pending.set(None);
                    refresh.update(|v| *v += 1);
                }
                Err(e) => error.set(Some(e.user_message())),
            }
            busy.set(false);
        });
    });

    view! {
        <PageLayout
            title="Daftar Satker — Pengajuan Pakaian Dinas"
            icon="fas fa-building"
            description="Pantau & proses pengajuan pakaian dinas per satuan kerja"
        >
            {move || {
                error
                    .get()
                    .map(|msg| {
                        view! {
                            <div class="mb-4 rounded-xl border border-danger-500/30 bg-danger-500/[0.08] px-4 py-3 text-sm text-danger-300">
                                {msg}
                            </div>
                        }
                    })
            }}

            <Suspense fallback=move || {
                view! { <LoadingState /> }
            }>
                {move || match list_resource.get() {
                    None => view! { <LoadingState /> }.into_any(),
                    Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                    Some(Ok(resp)) => {
                        let rows = resp.data;
                        if rows.is_empty() {
                            view! {
                                <div class="rounded-2xl border border-white/[0.06] bg-surface-panel p-8 text-center text-sm text-slate-400">
                                    "Belum ada satker pada pengajuan ini."
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
                                    <For
                                        each=move || rows.clone()
                                        key=|s| format!("{}-{}", s.id, s.aktivitas_id)
                                        children=move |s: PengajuanSatker| {
                                            let s_for_select = s.clone();
                                            let code = s.aktivitas_id;
                                            let nama = s
                                                .satker_nama
                                                .clone()
                                                .unwrap_or_else(|| "Satker".to_string());
                                            let kode = s.satker_kode.clone().unwrap_or_default();
                                            let pegawai = s.total_pegawai.unwrap_or(0);
                                            let is_selected = move || {
                                                selected
                                                    .get()
                                                    .as_ref()
                                                    .map(|x| x.id == s_for_select.id)
                                                    .unwrap_or(false)
                                            };
                                            let s_click = s.clone();
                                            let s_isi = s.clone();
                                            view! {
                                                <div class=move || {
                                                    format!(
                                                        "rounded-2xl border bg-surface-panel p-5 transition {}",
                                                        if is_selected() {
                                                            "border-gold-500/40"
                                                        } else {
                                                            "border-white/[0.06] hover:border-white/10"
                                                        },
                                                    )
                                                }>
                                                    <div class="flex items-start justify-between gap-3">
                                                        <div class="min-w-0">
                                                            <h3 class="truncate text-sm font-semibold text-slate-100">
                                                                {nama}
                                                            </h3>
                                                            <p class="text-xs text-slate-400">
                                                                {kode} " · " {pegawai.to_string()} " pegawai"
                                                            </p>
                                                        </div>
                                                        {status_badge(code)}
                                                    </div>
                                                    // Tautan pengisian. Sampai
                                                    // ini ada, satu-satunya hal
                                                    // yang bisa dilakukan pada
                                                    // sebuah satker adalah
                                                    // melihat riwayatnya — tidak
                                                    // ada jalan menuju daftar
                                                    // pegawainya untuk diisi.
                                                    <a
                                                        href=crate::routes::url::pakaian_pengisian(
                                                            &s_isi.pengajuan_id,
                                                            &s_isi.satker_id,
                                                        )
                                                        class="mt-4 flex w-full items-center justify-center gap-1.5 rounded-lg bg-gold-gradient px-3 py-1.5 text-xs font-bold text-navy-950 transition hover:opacity-90"
                                                    >
                                                        <AppIcon icon=PENCIL_SIMPLE size=14 />
                                                        "Isi Ukuran Pegawai"
                                                    </a>
                                                    <button
                                                        class="mt-2 w-full rounded-lg border border-info-500/30 bg-info-500/10 px-3 py-1.5 text-xs font-medium text-info-300 transition hover:bg-info-500/20"
                                                        on:click=move |_| selected.set(Some(s_click.clone()))
                                                    >
                                                        "Lihat Riwayat & Aksi"
                                                    </button>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            }
                                .into_any()
                        }
                    }
                }}
            </Suspense>

            // Detail panel for the selected satker.
            {move || {
                selected
                    .get()
                    .map(|s| {
                        let code = s.aktivitas_id;
                        let nama = s.satker_nama.clone().unwrap_or_else(|| "Satker".to_string());
                        let s_for_timeline = s.clone();
                        let actions_meta = available_actions(code);
                        let actions: Vec<WorkflowAction> = match actions_meta {
                            Some((approve_label, reject_label)) => {
                                let sid_a = s.id.clone();
                                let sid_r = s.id.clone();
                                let approve_title = approve_label.to_string();
                                let reject_title = reject_label.to_string();
                                vec![
                                    WorkflowAction::new(
                                        approve_label,
                                        ActionTone::Success,
                                        Callback::new(move |_: ()| {
                                            pending
                                                .set(
                                                    Some(PendingAction {
                                                        satker_id: sid_a.clone(),
                                                        aksi: "approve".to_string(),
                                                        title: approve_title.clone(),
                                                        tone: ActionTone::Success,
                                                        require_note: false,
                                                    }),
                                                );
                                        }),
                                    ),
                                    WorkflowAction::new(
                                        reject_label,
                                        ActionTone::Danger,
                                        Callback::new(move |_: ()| {
                                            pending
                                                .set(
                                                    Some(PendingAction {
                                                        satker_id: sid_r.clone(),
                                                        aksi: "reject".to_string(),
                                                        title: reject_title.clone(),
                                                        tone: ActionTone::Danger,
                                                        require_note: true,
                                                    }),
                                                );
                                        }),
                                    ),
                                ]
                            }
                            None => Vec::new(),
                        };
                        let has_actions = !actions.is_empty();

                        view! {
                            <div class="mt-6">
                                <SectionCard title="Riwayat & Aksi Workflow">
                                    <div class="mb-3 flex items-center justify-between">
                                        <p class="text-sm font-semibold text-slate-200">{nama}</p>
                                        <button
                                            class="text-xs text-slate-400 hover:text-slate-200"
                                            on:click=move |_| selected.set(None)
                                        >
                                            "Tutup"
                                        </button>
                                    </div>

                                    <Suspense fallback=move || {
                                        view! { <LoadingState /> }
                                    }>
                                        {move || match akt_resource.get() {
                                            None => view! { <LoadingState /> }.into_any(),
                                            Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                                            Some(Ok(acts)) => {
                                                let steps = build_timeline(&s_for_timeline, &acts);
                                                view! { <WorkflowTimeline steps=steps /> }.into_any()
                                            }
                                        }}
                                    </Suspense>

                                    {has_actions
                                        .then(|| {
                                            view! {
                                                <div class="mt-4 border-t border-white/[0.04] pt-4">
                                                    <WorkflowActions
                                                        actions=actions
                                                        busy=Signal::derive(move || busy.get())
                                                    />
                                                </div>
                                            }
                                        })}
                                    {(!has_actions)
                                        .then(|| {
                                            view! {
                                                <p class="mt-4 border-t border-white/[0.04] pt-4 text-xs text-slate-500">
                                                    "Tidak ada aksi tersedia untuk peran Anda pada status ini (mode pantau)."
                                                </p>
                                            }
                                        })}
                                </SectionCard>
                            </div>
                        }
                    })
            }}

            // Note-confirmation dialog for the pending action.
            {move || {
                pending
                    .get()
                    .map(|act| {
                        view! {
                            <ApprovalDialog
                                title=act.title.clone()
                                note_label=if act.require_note {
                                    "Alasan / Catatan (wajib)".to_string()
                                } else {
                                    "Catatan (opsional)".to_string()
                                }
                                require_note=act.require_note
                                confirm_label="Kirim"
                                confirm_tone=act.tone
                                on_confirm=on_confirm
                                on_close=Callback::new(move |_: ()| pending.set(None))
                                busy=Signal::derive(move || busy.get())
                            />
                        }
                    })
            }}
        </PageLayout>
    }
}
