//! Pakaian Dinas — per-satker workflow detail (#40).
//!
//! For a given pengajuan period, lists each satker submission and lets a
//! validator drill into its workflow: a [`WorkflowTimeline`] built from the
//! real per-satker activity history (`GET /pakaian-dinas/satker/{id}/aktivitas`)
//! plus approve/return actions via [`ApprovalDialog`]. Validator Wilayah acts
//! on submissions pending wilayah review; Validator Pusat on those forwarded to
//! pusat. Everyone else sees the timeline read-only.

use std::collections::HashMap;

use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_fetch::QueryClient;
use leptos_router::hooks::use_params_map;

use crate::api::{
    AppError, BulkActionResult, BulkValidatorActionRequest, PaginatedResponse, PengajuanSatker,
    PengajuanSatkerAktivitas, ValidatorActionRequest, aktivitas_is_revisi, aktivitas_label,
    fetch_pakaian_satker_aktivitas, fetch_pengajuan_satker_terfilter, process_validator_action,
    process_validator_action_bulk,
};
use crate::components::layout::{ErrorState, LoadingState, PageLayout, SectionCard};
use crate::components::pagination_controls::PaginationControls;
use crate::components::workflow_ui::{
    ActionTone, ApprovalDialog, ConfirmDialog, StepStatus, WorkflowAction, WorkflowActions,
    WorkflowStep, WorkflowTimeline,
};
use crate::features::auth::AuthService;
use crate::features::pakaian_dinas::api::{fetch_kesiapan_pakaian_dinas, ringkasan_kesiapan};
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
    aksi_untuk(
        code,
        session.is_validator_wilayah(),
        session.is_validator_pusat(),
    )
}

/// [`available_actions`] with the roles handed in, so the rule is testable without a session.
fn aksi_untuk(code: i32, wilayah: bool, pusat: bool) -> Option<(&'static str, &'static str)> {
    // The validators' moves are the validators'. An administrator is not offered
    // them: the API refuses an admin every business decision (no bypass), so the
    // button would only ever answer 403.
    // Wilayah review pending (1001 / 1012), or Pusat sent it back to Wilayah
    // (1007). Same two moves either way: forward it again, or send it down to the
    // satker. Without 1007 here, "Kembalikan ke Wilayah" below parked the satker
    // on a screen with no button for anyone.
    if matches!(code, 1001 | 1007 | 1012) && wilayah {
        return Some(("Teruskan ke Pusat", "Kembalikan untuk Revisi"));
    }
    // Pusat review pending (1004 / 1010)
    if matches!(code, 1004 | 1010) && pusat {
        return Some(("Setujui (Selesai)", "Kembalikan ke Wilayah"));
    }
    None
}

/// The rows this user may move on, grouped by the move they are offered, in first-seen order.
///
/// One bulk bar for all of them was wrong for an account holding both validator roles (the seeded
/// administrator does): its label came from the first row ("Teruskan ke Pusat") while "Pilih
/// semua" ticked rows at both stages, and the server runs each row's own move, so the same click
/// that forwarded a wilayah-stage satker also gave a pusat-stage one its FINAL approval. One bar
/// per move, each acting only on its own rows, makes the button say what it does.
fn kelompok_massal(
    rows: &[(String, i32)],
    wilayah: bool,
    pusat: bool,
) -> Vec<(&'static str, Vec<String>)> {
    let mut grup: Vec<(&'static str, Vec<String>)> = Vec::new();
    for (id, code) in rows {
        let Some((setuju, _)) = aksi_untuk(*code, wilayah, pusat) else {
            continue;
        };
        match grup.iter_mut().find(|(label, _)| *label == setuju) {
            Some((_, ids)) => ids.push(id.clone()),
            None => grup.push((setuju, vec![id.clone()])),
        }
    }
    grup
}

/// Satker per page. The grid is two columns wide, so an even number keeps the last row full.
/// A nationwide campaign carries every MySIMKARI satker (545 on staging); this screen used to
/// ask for the first 100 by name and stop, so a validator pusat or an admin could never reach
/// the rest, and had no way to ask which ones were waiting on them.
const PER_HALAMAN: i32 = 30;

/// What the list is asked for: the pengajuan, a refresh counter, the page, the search text and
/// the workflow statuses (a comma list). One tuple so any of them changing refetches.
type KunciDaftar = (String, i32, i32, String, String);

async fn query_satker_list(
    key: KunciDaftar,
) -> Result<PaginatedResponse<PengajuanSatker>, AppError> {
    let (pengajuan_id, _refresh, halaman, q, status) = key;
    fetch_pengajuan_satker_terfilter(pengajuan_id, halaman, PER_HALAMAN, q, status).await
}

/// The `status` of the list request for one choice of the status select. Blank = every status.
/// "menunggu" is the rows [`available_actions`] offers this user a move on: wilayah's pending
/// ones (1001 / 1012) and Pusat's returns to wilayah (1007), or Pusat's (1004 / 1010), and both
/// groups for an account that holds both roles (the seeded administrator does).
fn status_query(pilihan: &str, wilayah: bool, pusat: bool) -> String {
    match pilihan {
        "menunggu" if wilayah && pusat => "1001,1007,1012,1004,1010".to_string(),
        "menunggu" if wilayah => "1001,1007,1012".to_string(),
        "menunggu" if pusat => "1004,1010".to_string(),
        "penyiapan" => "1000".to_string(),
        "diproses" => "1001,1012,1004,1010".to_string(),
        "revisi" => "1003,1007".to_string(),
        "selesai" => "1008".to_string(),
        _ => String::new(),
    }
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
    /// Advisory "others still behind" warning fetched before the dialog opened.
    /// `None` = nothing outstanding (or the readiness call failed).
    warning: Option<String>,
    /// Gate this dialog was opened for. Sent back with the action so the server
    /// refuses a row another validator moved while the dialog was open.
    tahap: &'static str,
    satker_nama: Option<String>,
}

/// A pending bulk move awaiting confirmation. `label` is the move the bar
/// offers ("Teruskan ke Pusat" / "Setujui (Selesai)"); the warning is the
/// advisory count of OTHER satker still behind, fetched with every selected row
/// excluded so the dialog does not warn about the rows it is about to move.
#[derive(Clone)]
struct PendingBulk {
    ids: Vec<String>,
    label: String,
    warning: Option<String>,
    /// Gate this bar acts on, pinned when the dialog opened.
    tahap: &'static str,
}

/// Names for the rows a bulk move refused. The reason alone ("tahap pengajuan
/// sudah berpindah") does not say WHICH row, and a bulk move can refuse several
/// at once; the page in hand already holds the names.
fn nama_satker_per_id(
    list: Option<Result<PaginatedResponse<PengajuanSatker>, AppError>>,
) -> HashMap<String, String> {
    match list {
        Some(Ok(resp)) => resp
            .data
            .iter()
            .map(|s| {
                let label = s.satker_nama.clone().unwrap_or_else(|| s.satker_id.clone());
                (s.id.clone(), label)
            })
            .collect(),
        _ => HashMap::new(),
    }
}

/// The gate a status is waiting on: `satker` while it is still being filled,
/// `wilayah` when the wilayah validator owes a move, `pusat` when pusat does.
fn tahap_untuk_status(status: i32) -> &'static str {
    match status {
        1000 | 1003 => "satker",
        1001 | 1007 | 1012 => "wilayah",
        _ => "pusat",
    }
}

/// The gate a bulk bar acts on. The label names the move; the gate names the
/// level it moves from, which is what the readiness count and the server check.
fn tahap_untuk_label(label: &str) -> &'static str {
    if label == "Teruskan ke Pusat" {
        "wilayah"
    } else {
        "pusat"
    }
}

#[component]
pub fn PakaianDinasSatkerDetail() -> impl IntoView {
    // Only the satker's operator fills sizes; everyone else reads them. The link
    // is the same page either way, so it says which it is.
    let is_operator = AuthService::load_session()
        .map(|s| s.has_role("operator_satker"))
        .unwrap_or(false);
    let params = use_params_map();
    let pengajuan_id = move || {
        params.with(|p| {
            p.get("pengajuan_id")
                .map(|s| s.to_string())
                .unwrap_or_default()
        })
    };

    let refresh = RwSignal::new(0);
    let halaman = RwSignal::new(1_i32);
    // `cari_input` is what is typed; `cari` is what the list was last asked for, so the list
    // does not refetch on every keystroke.
    let cari_input = RwSignal::new(String::new());
    let cari = RwSignal::new(String::new());
    let status_pilihan = RwSignal::new(String::new());
    let (is_wilayah, is_pusat) = AuthService::load_session()
        .map(|s| (s.is_validator_wilayah(), s.is_validator_pusat()))
        .unwrap_or((false, false));
    let selected = RwSignal::new(Option::<PengajuanSatker>::None);
    let panel_ref = NodeRef::<html::Div>::new();
    let pending = RwSignal::new(Option::<PendingAction>::None);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    // Rows ticked for a bulk move, and what the last bulk move came to. A tick is the satker AND the
    // move it was ticked for ("Teruskan ke Pusat" or "Setujui (Selesai)"), not the satker alone:
    // forward a ticked satker from its own panel and it is now at the next stage, and a bare id
    // would carry the tick over to the final-approval bar, so that bar's button finished it too.
    let terpilih = RwSignal::new(Vec::<(String, &'static str)>::new());
    let hasil_massal = RwSignal::new(Option::<BulkActionResult>::None);
    let pending_massal = RwSignal::new(Option::<PendingBulk>::None);

    let client: QueryClient = expect_context();
    let list_resource = client.local_resource(query_satker_list, move || {
        (
            pengajuan_id(),
            refresh.get(),
            halaman.get(),
            cari.get(),
            status_query(&status_pilihan.get(), is_wilayah, is_pusat),
        )
    });
    // The detail panel sits below the list; with a page of cards between the button and the
    // panel, picking a satker looked like it did nothing.
    Effect::new(move |_| {
        if selected.get().is_some()
            && let Some(el) = panel_ref.get()
        {
            el.scroll_into_view();
        }
    });
    // Approving the last row of a filtered page empties it while later pages may remain.
    Effect::new(move |_| {
        if let Some(Ok(r)) = list_resource.get()
            && r.data.is_empty()
            && r.total > 0
            && halaman.get_untracked() > 1
        {
            halaman.set(1);
        }
    });
    let ke_halaman_awal = move || {
        halaman.set(1);
        terpilih.set(Vec::new());
    };
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
            tahap: Some(act.tahap.to_string()),
        };
        spawn_local(async move {
            match process_validator_action(req).await {
                Ok(resp) => {
                    // The panel's buttons are derived from the status of the row it
                    // holds, and it holds a COPY taken when it was opened. Without
                    // this the list and timeline refreshed while the panel kept
                    // offering the moves the row had just left — "Teruskan ke Pusat"
                    // on a row already at Pusat, which can only answer an error.
                    // The server returns the row as it is now; hold that.
                    selected.set(Some(resp.data));
                    pending.set(None);
                    refresh.update(|v| *v += 1);
                }
                Err(e) => error.set(Some(e.user_message())),
            }
            busy.set(false);
        });
    });

    // Approve every ticked satker in one request. Each goes through the same
    // per-row checks a single approval does; the answer says which went through.
    let on_massal = Callback::new(move |(ids, tahap): (Vec<String>, &'static str)| {
        if ids.is_empty() {
            return;
        }
        busy.set(true);
        error.set(None);
        hasil_massal.set(None);
        let req = BulkValidatorActionRequest {
            pengajuan_satker_ids: ids,
            aksi: "approve".to_string(),
            komentar: None,
            tahap: Some(tahap.to_string()),
        };
        spawn_local(async move {
            match process_validator_action_bulk(req).await {
                Ok(resp) => {
                    hasil_massal.set(Some(resp.data));
                    terpilih.set(Vec::new());
                    refresh.update(|v| *v += 1);
                }
                Err(e) => error.set(Some(e.user_message())),
            }
            // Close the confirmation only once the request is done, so the
            // dialog's busy state is what the person watches, not a bar that
            // vanished mid-request.
            pending_massal.set(None);
            busy.set(false);
        });
    });

    view! {
        <PageLayout
            title="Daftar Satker — Pengajuan Pakaian Dinas"
            icon="fas fa-building"
            description="Pantau & proses pengajuan pakaian dinas per satuan kerja"
        >
            <form
                data-testid="filter-satker"
                class="mb-4 flex flex-wrap items-end gap-3 rounded-xl border border-white/[0.06] bg-surface-panel px-4 py-3 text-sm"
                on:submit=move |ev| {
                    ev.prevent_default();
                    cari.set(cari_input.get().trim().to_string());
                    ke_halaman_awal();
                }
            >
                <label class="flex min-w-[14rem] flex-1 flex-col gap-1 text-xs text-slate-400">
                    "Cari satker"
                    <input
                        type="search"
                        data-testid="cari-satker"
                        placeholder="Nama atau kode satker"
                        maxlength="100"
                        class="rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100 placeholder:text-slate-500"
                        prop:value=move || cari_input.get()
                        on:input=move |ev| cari_input.set(event_target_value(&ev))
                    />
                </label>
                <label class="flex flex-col gap-1 text-xs text-slate-400">
                    "Status"
                    // `prop:value`, not `selected` on each option: `selected` only sets the default,
                    // so after the user has picked something Reset emptied the filter and left the
                    // dropdown showing the old choice beside a list that had stopped honouring it.
                    <select
                        data-testid="status-satker"
                        class="rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100"
                        prop:value=move || status_pilihan.get()
                        on:change=move |ev| {
                            status_pilihan.set(event_target_value(&ev));
                            ke_halaman_awal();
                        }
                    >
                        <option value="">
                            "Semua status"
                        </option>
                        {(is_wilayah || is_pusat)
                            .then(|| {
                                view! {
                                    <option value="menunggu">
                                        "Menunggu tindakan saya"
                                    </option>
                                }
                            })}
                        <option value="penyiapan">
                            "Penyiapan"
                        </option>
                        <option value="diproses">
                            "Sedang diproses"
                        </option>
                        <option value="revisi">
                            "Perlu revisi"
                        </option>
                        <option value="selesai">
                            "Selesai"
                        </option>
                    </select>
                </label>
                <button
                    type="submit"
                    data-testid="cari-satker-kirim"
                    class="rounded-lg bg-gold-gradient px-4 py-2 text-xs font-bold text-navy-950 transition hover:opacity-90"
                >
                    "Cari"
                </button>
                <button
                    type="button"
                    data-testid="cari-satker-reset"
                    class="rounded-lg border border-white/10 bg-white/[0.04] px-4 py-2 text-xs text-slate-300 transition hover:bg-white/[0.08]"
                    on:click=move |_| {
                        cari_input.set(String::new());
                        cari.set(String::new());
                        status_pilihan.set(String::new());
                        ke_halaman_awal();
                    }
                >
                    "Reset"
                </button>
            </form>

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

            {move || {
                hasil_massal
                    .get()
                    .map(|h| {
                        let berhasil = h.berhasil.len();
                        let gagal = h.gagal;
                        let nama = nama_satker_per_id(list_resource.get());
                        view! {
                            <div
                                data-testid="hasil-massal"
                                class="mb-4 rounded-xl border border-success-500/30 bg-success-500/[0.08] px-4 py-3 text-sm text-success-300"
                            >
                                <p class="font-semibold">
                                    {format!("{berhasil} satker diproses")}
                                    {if gagal.is_empty() {
                                        String::new()
                                    } else {
                                        format!(", {} ditolak:", gagal.len())
                                    }}
                                </p>
                                <ul class="mt-1 list-disc pl-5 text-warning-300">
                                    {gagal
                                        .into_iter()
                                        .map(|g| {
                                            let label = nama.get(&g.pengajuan_satker_id).cloned();
                                            view! {
                                                <li>
                                                    {match label {
                                                        Some(n) => format!("{n}: {}", g.alasan),
                                                        None => g.alasan,
                                                    }}
                                                </li>
                                            }
                                        })
                                        .collect_view()}
                                </ul>
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
                        let total = resp.total;
                        let total_pages = resp.total_pages;
                        let ada_filter = !cari.get_untracked().is_empty()
                            || !status_pilihan.get_untracked().is_empty();
                        let rows = resp.data;
                        // The rows this user may move on now (a wilayah validator's
                        // pending ones, or pusat's); only those get a tick box.
                        let baris: Vec<(String, i32)> = rows
                            .iter()
                            .map(|s| (s.id.clone(), s.aktivitas_id))
                            .collect();
                        let grup = kelompok_massal(&baris, is_wilayah, is_pusat);
                        let banyak_grup = grup.len() > 1;
                        let bilah = grup
                            .into_iter()
                            .map(|(label, ids)| {
                                let n = ids.len();
                                let ids = std::sync::Arc::new(ids);
                                let dipilih = {
                                    let ids = ids.clone();
                                    move || {
                                        terpilih
                                            .get()
                                            .into_iter()
                                            .filter(|(x, l)| *l == label && ids.contains(x))
                                            .map(|(x, _)| x)
                                            .collect::<Vec<String>>()
                                    }
                                };
                                let semua_terpilih = {
                                    let ids = ids.clone();
                                    move || {
                                        let t = terpilih.get();
                                        n > 0
                                            && ids
                                                .iter()
                                                .all(|i| t.iter().any(|(x, l)| x == i && *l == label))
                                    }
                                };
                                let ubah_semua = {
                                    let ids = ids.clone();
                                    move |ev| {
                                        let on = event_target_checked(&ev);
                                        terpilih
                                            .update(|v| {
                                                v.retain(|(x, l)| !(*l == label && ids.contains(x)));
                                                if on {
                                                    v.extend(ids.iter().map(|i| (i.clone(), label)));
                                                }
                                            });
                                    }
                                };
                                let teks = match (banyak_grup, total_pages > 1) {
                                    (true, _) => format!("Pilih semua untuk \u{201c}{label}\u{201d} ({n})"),
                                    (false, true) => format!("Pilih semua yang menunggu di halaman ini ({n})"),
                                    (false, false) => format!("Pilih semua yang menunggu ({n})"),
                                };
                                view! {
                                    <div
                                        data-testid="bilah-massal"
                                        data-aksi=label
                                        class="mb-4 flex flex-wrap items-center justify-between gap-3 rounded-xl border border-white/[0.06] bg-surface-panel px-4 py-3 text-sm"
                                    >
                                        <label class="flex items-center gap-2 text-slate-200">
                                            <input
                                                type="checkbox"
                                                data-testid="pilih-semua"
                                                class="h-4 w-4 rounded border-white/20 bg-white/[0.04]"
                                                prop:checked=semua_terpilih
                                                prop:disabled=move || busy.get()
                                                on:change=ubah_semua
                                            />
                                            {teks}
                                        </label>
                                        <div class="flex items-center gap-3">
                                            <span class="text-slate-400">
                                                {
                                                    let dipilih = dipilih.clone();
                                                    move || format!("{} dipilih", dipilih().len())
                                                }
                                            </span>
                                            <button
                                                type="button"
                                                data-testid="setujui-massal"
                                                class="rounded-lg bg-gold-gradient px-4 py-2 text-xs font-bold text-navy-950 transition hover:opacity-90 disabled:opacity-40"
                                                prop:disabled={
                                                    let dipilih = dipilih.clone();
                                                    move || busy.get() || dipilih().is_empty()
                                                }
                                                on:click={
                                                    let dipilih = dipilih.clone();
                                                    move |_| {
                                                        let ids = dipilih();
                                                        if ids.is_empty() {
                                                            return;
                                                        }
                                                        busy.set(true);
                                                        error.set(None);
                                                        let pid = pengajuan_id();
                                                        let tahap = tahap_untuk_label(label);
                                                        let kecuali = ids.clone();
                                                        spawn_local(async move {
                                                            let warning = fetch_kesiapan_pakaian_dinas(
                                                                    &pid,
                                                                    tahap,
                                                                    None,
                                                                    &kecuali,
                                                                )
                                                                .await
                                                                .ok()
                                                                .filter(|k| !k.lengkap)
                                                                .and_then(|k| ringkasan_kesiapan(&k));
                                                            pending_massal
                                                                .set(
                                                                    Some(PendingBulk {
                                                                        ids,
                                                                        label: label.to_string(),
                                                                        warning,
                                                                        tahap,
                                                                    }),
                                                                );
                                                            busy.set(false);
                                                        });
                                                    }
                                                }
                                            >
                                                {
                                                    let dipilih = dipilih.clone();
                                                    move || format!("{label} ({})", dipilih().len())
                                                }
                                            </button>
                                        </div>
                                    </div>
                                }
                            })
                            .collect_view();
                        if rows.is_empty() {
                            view! {
                                <div class="rounded-2xl border border-white/[0.06] bg-surface-panel p-8 text-center text-sm text-slate-400">
                                    {if ada_filter {
                                        "Tidak ada satker yang cocok dengan pencarian atau status ini."
                                    } else {
                                        "Belum ada satker pada pengajuan ini."
                                    }}
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                {bilah}
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
                                            let s_pilih = s.clone();
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
                                                                {kode} " · " {pegawai.to_string()} " pegawai terisi"
                                                            </p>
                                                        </div>
                                                        <div class="flex items-center gap-2">
                                                            {if available_actions(code).is_some() {
                                                                let id_pilih = s_pilih.id.clone();
                                                                let id_cek = id_pilih.clone();
                                                                let aksi_baris = available_actions(code)
                                                                    .map(|(setuju, _)| setuju)
                                                                    .unwrap_or("");
                                                                Some(
                                                                    view! {
                                                                        <input
                                                                            type="checkbox"
                                                                            data-testid="pilih-satker"
                                                                            aria-label=format!("Pilih {}", s_pilih.satker_nama.clone().unwrap_or_default())
                                                                            class="h-4 w-4 rounded border-white/20 bg-white/[0.04]"
                                                                            prop:checked=move || {
                                                                                terpilih
                                                                                    .get()
                                                                                    .iter()
                                                                                    .any(|(x, l)| *x == id_cek && *l == aksi_baris)
                                                                            }
                                                                            prop:disabled=move || busy.get()
                                                                            on:change=move |ev| {
                                                                                let on = event_target_checked(&ev);
                                                                                let id = id_pilih.clone();
                                                                                terpilih
                                                                                    .update(|v| {
                                                                                        v.retain(|(x, _)| *x != id);
                                                                                        if on {
                                                                                            v.push((id, aksi_baris));
                                                                                        }
                                                                                    });
                                                                            }
                                                                        />
                                                                    },
                                                                )
                                                            } else {
                                                                None
                                                            }}
                                                            {status_badge(code)}
                                                        </div>
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
                                                        class=if is_operator {
                                                            "mt-4 flex w-full items-center justify-center gap-1.5 rounded-lg bg-gold-gradient px-3 py-1.5 text-xs font-bold text-navy-950 transition hover:opacity-90"
                                                        } else {
                                                            "mt-4 flex w-full items-center justify-center gap-1.5 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs font-medium text-slate-200 transition hover:bg-white/[0.08]"
                                                        }
                                                    >
                                                        <AppIcon icon=PENCIL_SIMPLE size=14 />
                                                        {if is_operator {
                                                            "Isi Ukuran Pegawai"
                                                        } else {
                                                            "Lihat Ukuran Pegawai"
                                                        }}
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
                                <PaginationControls
                                    current_page=Signal::derive(move || halaman.get())
                                    total_pages=Signal::derive(move || total_pages.max(1))
                                    total_items=Some(total)
                                    on_prev=Callback::new(move |_: ()| {
                                        halaman.update(|p| *p = (*p - 1).max(1));
                                        terpilih.set(Vec::new());
                                    })
                                    on_next=Callback::new(move |_: ()| {
                                        halaman.update(|p| *p += 1);
                                        terpilih.set(Vec::new());
                                    })
                                />
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
                                let pid_a = s.pengajuan_id.clone();
                                // The single-satker warning is about THIS satker only.
                                let kode_a = s.satker_id.clone();
                                let approve_title = approve_label.to_string();
                                let reject_title = reject_label.to_string();
                                let satker_nama = s.satker_nama.clone();
                                let nama_r = s.satker_nama.clone();
                                let tahap = tahap_untuk_status(code);
                                vec![
                                    WorkflowAction::new(
                                        approve_label,
                                        ActionTone::Success,
                                        Callback::new(move |_: ()| {
                                            let sid = sid_a.clone();
                                            let pid = pid_a.clone();
                                            let kode = kode_a.clone();
                                            let title = approve_title.clone();
                                            let nama = satker_nama.clone();
                                            busy.set(true);
                                            spawn_local(async move {
                                                // One satker moves on: warn about that satker's own
                                                // employees, not about the rest of the campaign.
                                                let warning = fetch_kesiapan_pakaian_dinas(
                                                    &pid,
                                                    "satker",
                                                    Some(kode.as_str()),
                                                    &[],
                                                )
                                                .await
                                                .ok()
                                                .filter(|k| !k.lengkap)
                                                .and_then(|k| ringkasan_kesiapan(&k));
                                                // The count is a round-trip; the panel
                                                // may have moved to another satker in the
                                                // meantime. Opening then would act on a row
                                                // the person is no longer looking at.
                                                if selected.get().map(|s| s.id).as_deref()
                                                    != Some(sid.as_str())
                                                {
                                                    busy.set(false);
                                                    return;
                                                }
                                                pending
                                                    .set(
                                                        Some(PendingAction {
                                                            satker_id: sid,
                                                            aksi: "approve".to_string(),
                                                            title,
                                                            tone: ActionTone::Success,
                                                            require_note: false,
                                                            warning,
                                                            tahap,
                                                            satker_nama: nama,
                                                        }),
                                                    );
                                                busy.set(false);
                                            });
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
                                                        warning: None,
                                                        tahap,
                                                        satker_nama: nama_r.clone(),
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
                            // Leave room for the fixed top bar when the page scrolls to it.
                            <div class="mt-6" node_ref=panel_ref style="scroll-margin-top: 5rem">
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
                        let judul = match act.satker_nama.as_deref() {
                            Some(nama) => format!("{} — {}", act.title, nama),
                            None => act.title.clone(),
                        };
                        view! {
                            <ApprovalDialog
                                title=judul
                                warning=act.warning.clone()
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
            {move || {
                pending_massal
                    .get()
                    .map(|b| {
                        let n = b.ids.len();
                        let ids = b.ids.clone();
                        let (title, message) = if b.label == "Teruskan ke Pusat" {
                            (
                                format!("Teruskan {n} satker ke Validator Pusat?"),
                                "Satker terpilih akan diteruskan sekaligus.".to_string(),
                            )
                        } else {
                            (
                                format!("Setujui {n} satker (Selesai)?"),
                                "Satker terpilih akan diselesaikan dan tidak bisa diubah lagi."
                                    .to_string(),
                            )
                        };
                        view! {
                            <ConfirmDialog
                                title=title
                                message=message
                                warning=b.warning.clone()
                                confirm_label="Ya, lanjutkan"
                                confirm_tone=ActionTone::Success
                                on_confirm=Callback::new(move |_: ()| {
                                    on_massal.run((ids.clone(), b.tahap))
                                })
                                on_close=Callback::new(move |_: ()| pending_massal.set(None))
                                busy=Signal::derive(move || busy.get())
                            />
                        }
                    })
            }}
        </PageLayout>
    }
}

#[cfg(test)]
mod tests {
    use super::{aksi_untuk, kelompok_massal, status_query, tahap_untuk_label, tahap_untuk_status};

    fn baris() -> Vec<(String, i32)> {
        // Two waiting at wilayah (one of them returned by pusat), two at pusat, one with nobody's
        // turn (still being filled in), one done.
        vec![
            ("w1".into(), 1001),
            ("p1".into(), 1004),
            ("w2".into(), 1007),
            ("x1".into(), 1000),
            ("p2".into(), 1010),
            ("s1".into(), 1008),
        ]
    }

    /// The dialog's `tahap` and the readiness gate must agree with the status
    /// the row is actually in, and every status the workflow can produce must
    /// have a readable label (the previous gap: a returned row showed "Dalam
    /// Proses" and offered no action).
    #[test]
    fn every_reachable_status_has_a_gate_and_a_label() {
        for (code, gate) in [
            (1000, "satker"),
            (1003, "satker"),
            (1001, "wilayah"),
            (1007, "wilayah"),
            (1012, "wilayah"),
            (1004, "pusat"),
            (1010, "pusat"),
            (1008, "pusat"),
        ] {
            assert_eq!(tahap_untuk_status(code), gate, "status {code}");
            assert_ne!(
                crate::api::aktivitas_label(code),
                "Dalam Proses",
                "status {code} needs a real label"
            );
        }
        // The bar's label names the move; the gate names the level it acts from.
        assert_eq!(tahap_untuk_label("Teruskan ke Pusat"), "wilayah");
        assert_eq!(tahap_untuk_label("Setujui (Selesai)"), "pusat");
    }

    #[test]
    fn a_dual_role_account_gets_one_group_per_move_never_one_mixed_group() {
        // The bug: one bar labelled by the first row ("Teruskan ke Pusat") whose "select all"
        // ticked both stages, so forwarding also gave the pusat-stage rows their final approval.
        let grup = kelompok_massal(&baris(), true, true);
        assert_eq!(grup.len(), 2);
        assert_eq!(grup[0].0, "Teruskan ke Pusat");
        assert_eq!(grup[0].1, vec!["w1".to_string(), "w2".to_string()]);
        assert_eq!(grup[1].0, "Setujui (Selesai)");
        assert_eq!(grup[1].1, vec!["p1".to_string(), "p2".to_string()]);
    }

    #[test]
    fn a_single_role_sees_only_its_own_stage() {
        let w = kelompok_massal(&baris(), true, false);
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].1, vec!["w1".to_string(), "w2".to_string()]);
        let p = kelompok_massal(&baris(), false, true);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].0, "Setujui (Selesai)");
        assert_eq!(p[0].1, vec!["p1".to_string(), "p2".to_string()]);
        assert!(kelompok_massal(&baris(), false, false).is_empty());
    }

    #[test]
    fn a_row_nobody_may_move_is_in_no_group() {
        for kode in [1000, 1003, 1008] {
            assert_eq!(aksi_untuk(kode, true, true), None, "{kode}");
        }
    }

    #[test]
    fn menunggu_is_the_rows_this_role_can_move() {
        // The same codes `available_actions` offers a move on, per role.
        assert_eq!(status_query("menunggu", true, false), "1001,1007,1012");
        assert_eq!(status_query("menunggu", false, true), "1004,1010");
        // An account with both roles can act on both groups, so "waiting on me" is the union;
        // sending only the first group hid every satker waiting at pusat from it.
        assert_eq!(
            status_query("menunggu", true, true),
            "1001,1007,1012,1004,1010"
        );
        // Someone who can move nothing is not offered this choice (the option is hidden), so a
        // blank here only covers a stale selection; the server's satker scope still applies.
        assert_eq!(status_query("menunggu", false, false), "");
    }

    #[test]
    fn named_choices_map_to_workflow_codes() {
        assert_eq!(status_query("penyiapan", false, false), "1000");
        assert_eq!(
            status_query("diproses", false, false),
            "1001,1012,1004,1010"
        );
        assert_eq!(status_query("revisi", false, false), "1003,1007");
        assert_eq!(status_query("selesai", false, false), "1008");
        assert_eq!(status_query("", true, true), "");
        assert_eq!(status_query("bukan-pilihan", true, true), "");
    }
}
