//! Shared workflow-state helpers and UI primitives.
//!
//! The backend exposes one generic workflow engine that all four business
//! modules (kebutuhan_bmn, pakaian_dinas, pemakaian_bmn, penghapusan_bmn)
//! plug into via their own `WorkflowDefinition`. The modules still differ
//! in business-meaningful ways (parallel approval, permit lifecycle,
//! signed-SK upload, …), but the bits that should *not* diverge — status
//! colours, transition rendering, timeline shape, permission checks — live
//! here so each module can compose them instead of reinventing them.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::api::workflow::{WorkflowDefinitionDetail, WorkflowStep};
use crate::features::auth::UserSession;

/// High-level grouping used to pick status colours. Every concrete state
/// maps to one of these semantic buckets so the UI stays consistent even
/// when modules have their own state names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowPhase {
    Draft,
    Submitted,
    InReview,
    RevisionRequested,
    Approved,
    Rejected,
    Completed,
    Cancelled,
    Active,
    Expired,
    Revoked,
    Unknown,
}

impl WorkflowPhase {
    /// Classify a state code/name into a semantic phase. Matching is
    /// case-insensitive and uses substring matching so module-specific
    /// suffixes (e.g. `REVISI_WILAYAH`) still land in the right bucket.
    pub fn classify(state: &str) -> Self {
        let s = state.to_ascii_uppercase();
        if s.contains("DRAFT") {
            Self::Draft
        } else if s.contains("REVISI") || s.contains("REVISION") {
            Self::RevisionRequested
        } else if s.contains("SUBMIT") || s.contains("DIAJUKAN") {
            Self::Submitted
        } else if s.contains("REVIEW") || s.contains("VALIDASI") || s.contains("APPROVAL") {
            Self::InReview
        } else if s.contains("REJECT") || s.contains("DITOLAK") {
            Self::Rejected
        } else if s.contains("APPROVE") || s.contains("DISETUJUI") {
            Self::Approved
        } else if s.contains("COMPLETE") || s.contains("SELESAI") {
            Self::Completed
        } else if s.contains("CANCEL") || s.contains("BATAL") {
            Self::Cancelled
        } else if s.contains("ACTIVE") || s.contains("AKTIF") {
            Self::Active
        } else if s.contains("EXPIRED") || s.contains("KEDALUWARSA") {
            Self::Expired
        } else if s.contains("REVOKED") || s.contains("DICABUT") {
            Self::Revoked
        } else {
            Self::Unknown
        }
    }

    /// Tailwind classes for badge backgrounds. All pick semantic tokens
    /// from `tailwind.config.js` — no raw hex.
    pub fn badge_classes(self) -> &'static str {
        match self {
            Self::Draft => "bg-slate-500/20 text-slate-200 border border-slate-500/30",
            Self::Submitted => "bg-info-500/15 text-info-300 border border-info-500/30",
            Self::InReview => "bg-info-500/20 text-info-200 border border-info-500/40",
            Self::RevisionRequested => {
                "bg-warning-500/20 text-warning-200 border border-warning-500/40"
            }
            Self::Approved => "bg-success-500/15 text-success-300 border border-success-500/30",
            Self::Rejected => "bg-danger-500/20 text-danger-200 border border-danger-500/40",
            Self::Completed => "bg-success-500/25 text-success-200 border border-success-500/50",
            Self::Cancelled => "bg-slate-600/20 text-slate-300 border border-slate-600/30",
            Self::Active => "bg-success-500/15 text-success-300 border border-success-500/30",
            Self::Expired => "bg-warning-500/20 text-warning-200 border border-warning-500/40",
            Self::Revoked => "bg-danger-500/20 text-danger-200 border border-danger-500/40",
            Self::Unknown => "bg-slate-500/15 text-slate-300 border border-slate-500/30",
        }
    }

    /// Short Indonesian label used when a raw state code is not available.
    pub fn label(self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::Submitted => "Diajukan",
            Self::InReview => "Dalam Review",
            Self::RevisionRequested => "Perlu Revisi",
            Self::Approved => "Disetujui",
            Self::Rejected => "Ditolak",
            Self::Completed => "Selesai",
            Self::Cancelled => "Dibatalkan",
            Self::Active => "Aktif",
            Self::Expired => "Kedaluwarsa",
            Self::Revoked => "Dicabut",
            Self::Unknown => "Tidak diketahui",
        }
    }
}

/// One legal transition the UI can offer. The helper derives these from
/// the backend `WorkflowDefinitionDetail` so modules do not hardcode them.
#[derive(Debug, Clone)]
pub struct WorkflowTransition {
    pub from_state: String,
    pub to_state: String,
    pub label: String,
    pub tone: TransitionTone,
    pub required_role: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionTone {
    Primary,
    Approve,
    Reject,
    Revise,
    Neutral,
}

impl TransitionTone {
    pub fn button_classes(self) -> &'static str {
        match self {
            Self::Primary => {
                "inline-flex items-center gap-2 rounded-lg bg-gold-500 px-4 py-2 text-sm \
                 font-semibold text-navy-950 shadow-sm transition hover:bg-gold-400 \
                 disabled:cursor-not-allowed disabled:opacity-60"
            }
            Self::Approve => {
                "inline-flex items-center gap-2 rounded-lg bg-success-500/20 px-4 py-2 \
                 text-sm font-semibold text-success-200 border border-success-500/40 \
                 transition hover:bg-success-500/30 disabled:cursor-not-allowed \
                 disabled:opacity-60"
            }
            Self::Reject => {
                "inline-flex items-center gap-2 rounded-lg bg-danger-500/20 px-4 py-2 \
                 text-sm font-semibold text-danger-200 border border-danger-500/40 \
                 transition hover:bg-danger-500/30 disabled:cursor-not-allowed \
                 disabled:opacity-60"
            }
            Self::Revise => {
                "inline-flex items-center gap-2 rounded-lg bg-warning-500/20 px-4 py-2 \
                 text-sm font-semibold text-warning-200 border border-warning-500/40 \
                 transition hover:bg-warning-500/30 disabled:cursor-not-allowed \
                 disabled:opacity-60"
            }
            Self::Neutral => {
                "inline-flex items-center gap-2 rounded-lg bg-white/5 px-4 py-2 text-sm \
                 font-semibold text-slate-200 border border-white/10 transition \
                 hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-60"
            }
        }
    }
}

fn classify_transition(target: &str) -> TransitionTone {
    let s = target.to_ascii_uppercase();
    if s.contains("REJECT") || s.contains("TOLAK") {
        TransitionTone::Reject
    } else if s.contains("REVISI") || s.contains("REVISION") {
        TransitionTone::Revise
    } else if s.contains("APPROVE") || s.contains("SETUJU") || s.contains("COMPLETE") {
        TransitionTone::Approve
    } else if s.contains("SUBMIT") || s.contains("AJUKAN") {
        TransitionTone::Primary
    } else {
        TransitionTone::Neutral
    }
}

fn humanize_transition(target: &str) -> String {
    let cleaned = target.replace('_', " ");
    let lower = cleaned.to_ascii_lowercase();
    let mut out = String::with_capacity(lower.len());
    let mut upper_next = true;
    for ch in lower.chars() {
        if upper_next && ch.is_ascii_alphabetic() {
            out.push(ch.to_ascii_uppercase());
            upper_next = false;
        } else {
            out.push(ch);
        }
        if ch == ' ' {
            upper_next = true;
        }
    }
    out
}

/// Return the transitions a user is allowed to trigger from a given state.
/// Applies the backend's declared `next_states` and filters by role from
/// the user's JWT. If a step declares no required role, it is visible to
/// anyone with access to the entity.
pub fn allowed_transitions(
    definition: &WorkflowDefinitionDetail,
    current_state: &str,
    session: Option<&UserSession>,
) -> Vec<WorkflowTransition> {
    let Some(step) = definition
        .steps
        .iter()
        .find(|s| s.state_name.eq_ignore_ascii_case(current_state))
    else {
        return Vec::new();
    };

    let user_roles: Vec<String> = session
        .map(|s| s.roles.iter().map(|r| r.to_ascii_lowercase()).collect())
        .unwrap_or_default();

    step.next_states
        .iter()
        .filter_map(|target| {
            let target_step = definition
                .steps
                .iter()
                .find(|s| s.state_name.eq_ignore_ascii_case(target));

            let required = target_step.and_then(|s| s.required_role.as_ref());
            if let Some(role) = required {
                let needle = role.to_ascii_lowercase();
                let has_role = user_roles.iter().any(|r| r == &needle)
                    || session.map(|s| s.is_admin()).unwrap_or(false);
                if !has_role {
                    return None;
                }
            }

            Some(WorkflowTransition {
                from_state: step.state_name.clone(),
                to_state: target.clone(),
                label: humanize_transition(target),
                tone: classify_transition(target),
                required_role: required.cloned(),
            })
        })
        .collect()
}

/// Convenience lookup: is a given state terminal in this definition?
pub fn is_terminal(definition: &WorkflowDefinitionDetail, state: &str) -> bool {
    definition
        .steps
        .iter()
        .find(|s| s.state_name.eq_ignore_ascii_case(state))
        .map(|s| s.is_terminal)
        .unwrap_or(false)
}

/// Convenience lookup: find a step by state name.
pub fn find_step<'a>(
    definition: &'a WorkflowDefinitionDetail,
    state: &str,
) -> Option<&'a WorkflowStep> {
    definition
        .steps
        .iter()
        .find(|s| s.state_name.eq_ignore_ascii_case(state))
}

// ─── UI primitives ───────────────────────────────────────────────────────

/// A compact status badge. Accepts the raw backend state code and derives
/// colours + label automatically.
#[component]
pub fn WorkflowStatusBadge(#[prop(into)] state: String) -> impl IntoView {
    let phase = WorkflowPhase::classify(&state);
    let classes = phase.badge_classes();
    let display = if state.is_empty() {
        phase.label().to_string()
    } else {
        humanize_transition(&state)
    };
    view! {
        <span class=format!(
            "inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs \
             font-semibold {}",
            classes,
        )>
            <span class="h-1.5 w-1.5 rounded-full bg-current opacity-80"></span>
            {display}
        </span>
    }
}

/// A history entry for [`WorkflowTimeline`]. Callers typically build these
/// from an activity-log response.
#[derive(Debug, Clone)]
pub struct TimelineEntry {
    pub state: String,
    pub actor: String,
    pub timestamp: String,
    pub note: Option<String>,
}

/// Vertical timeline of workflow transitions. Emphasises the *current*
/// state and renders older entries muted.
#[component]
pub fn WorkflowTimeline(
    #[prop(into)] entries: Vec<TimelineEntry>,
    #[prop(optional, into)] current_state: Option<String>,
) -> impl IntoView {
    if entries.is_empty() {
        return view! {
            <p class="text-sm text-slate-500">"Belum ada riwayat alur kerja."</p>
        }
        .into_any();
    }

    let current = current_state.unwrap_or_default();
    let items = entries
        .into_iter()
        .map(|entry| {
            let is_current = !current.is_empty()
                && entry.state.eq_ignore_ascii_case(&current);
            let phase = WorkflowPhase::classify(&entry.state);
            let dot_classes = if is_current {
                "h-3 w-3 rounded-full bg-gold-500 ring-2 ring-gold-500/40"
            } else {
                "h-3 w-3 rounded-full bg-slate-500"
            };
            let note = entry.note.clone();
            view! {
                <li class="relative flex gap-3 pb-5 last:pb-0">
                    <div class="flex flex-col items-center">
                        <span class=dot_classes></span>
                        <span class="mt-1 flex-1 w-px bg-white/10"></span>
                    </div>
                    <div class="flex-1 -mt-0.5">
                        <div class="flex flex-wrap items-center gap-2">
                            <WorkflowStatusBadge state=entry.state.clone() />
                            <span class="text-xs text-slate-500">{entry.timestamp}</span>
                        </div>
                        <p class="mt-1 text-sm text-slate-200">
                            <span class="font-medium">{entry.actor}</span>
                            " · "
                            <span class="text-slate-400">{phase.label()}</span>
                        </p>
                        {note.map(|n| view! {
                            <p class="mt-1 text-xs text-slate-400">{n}</p>
                        })}
                    </div>
                </li>
            }
        })
        .collect_view();

    view! {
        <ol class="relative">{items}</ol>
    }
    .into_any()
}

/// Action bar that renders every transition currently available to the
/// user. Each button calls `on_transition` with the target state name.
#[component]
pub fn WorkflowActionBar(
    #[prop(into)] transitions: Vec<WorkflowTransition>,
    #[prop(into)] on_transition: Callback<WorkflowTransition>,
    #[prop(optional)] pending: Option<Signal<bool>>,
) -> impl IntoView {
    if transitions.is_empty() {
        return view! {
            <p class="text-sm text-slate-500">
                "Tidak ada aksi alur kerja yang tersedia untuk Anda pada tahap ini."
            </p>
        }
        .into_any();
    }

    let is_pending = move || pending.map(|p| p.get()).unwrap_or(false);

    let buttons = transitions
        .into_iter()
        .map(|t| {
            let tone_classes = t.tone.button_classes();
            let label = t.label.clone();
            let transition = t.clone();
            let on_click = move |_| on_transition.run(transition.clone());
            view! {
                <button
                    type="button"
                    class=tone_classes
                    on:click=on_click
                    disabled=move || is_pending()
                >
                    {label}
                </button>
            }
        })
        .collect_view();

    view! {
        <div class="flex flex-wrap gap-2">{buttons}</div>
    }
    .into_any()
}
