//! # Reusable Workflow UI Components (Fase 2.5)
//!
//! Komponen workflow approval yang dipakai bersama oleh empat modul
//! perlengkapan (Kebutuhan BMN, Pakaian Dinas, Pemakaian BMN, Penghapusan
//! BMN). Sebelumnya tiap modul menyalin sendiri timeline, dialog catatan,
//! dan tombol aksi — komponen di sini menggantikan duplikasi tersebut.
//!
//! Tiga primitive:
//! - [`WorkflowTimeline`] — riwayat/tahapan workflow (data-driven, tiap modul
//!   merakit `Vec<WorkflowStep>` dari status + field catatannya sendiri).
//! - [`ApprovalDialog`] — modal input catatan/alasan untuk aksi approve/return,
//!   dengan validasi "catatan wajib".
//! - [`WorkflowActions`] — deretan tombol aksi dari `Vec<WorkflowAction>`.

use leptos::prelude::*;

// ============================================================================
// Shared tone → Tailwind class mapping
// ============================================================================

/// Nuansa visual tombol/aksi workflow.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActionTone {
    Primary,
    Success,
    Danger,
}

impl ActionTone {
    /// Kelas tombol solid (untuk `WorkflowActions` & tombol konfirmasi dialog).
    pub fn btn_class(self) -> &'static str {
        match self {
            ActionTone::Primary => "bg-blue-600 hover:bg-blue-700 text-white",
            ActionTone::Success => "bg-green-600 hover:bg-green-700 text-white",
            ActionTone::Danger => "bg-red-600 hover:bg-red-700 text-white",
        }
    }
}

// ============================================================================
// WorkflowTimeline
// ============================================================================

/// Status satu tahapan dalam timeline. Timeline dirakit event-log style —
/// hanya tahapan yang sudah terjadi yang ditambahkan, jadi tidak ada varian
/// "belum tercapai".
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    /// Tahapan sudah dilewati / selesai.
    Done,
    /// Tahapan yang sedang berjalan.
    Current,
    /// Tahapan terminal penolakan.
    Rejected,
}

impl StepStatus {
    fn dot_class(self) -> &'static str {
        match self {
            StepStatus::Done => "bg-success-500 border-success-500",
            StepStatus::Current => "bg-info-400 border-info-400 ring-4 ring-info-500/25",
            StepStatus::Rejected => "bg-danger-500 border-danger-500",
        }
    }

    fn label_class(self) -> &'static str {
        match self {
            StepStatus::Done => "text-slate-100",
            StepStatus::Current => "text-info-300 font-semibold",
            StepStatus::Rejected => "text-danger-300 font-semibold",
        }
    }
}

/// Satu langkah/tahapan untuk [`WorkflowTimeline`]. Setiap modul merakit ini
/// dari status workflow + field catatan (`catatan_operator`, dst).
#[derive(Clone)]
pub struct WorkflowStep {
    pub label: String,
    pub status: StepStatus,
    pub actor: Option<String>,
    pub timestamp: Option<String>,
    pub note: Option<String>,
}

impl WorkflowStep {
    /// Helper ringkas untuk membuat langkah tanpa metadata.
    pub fn new(label: impl Into<String>, status: StepStatus) -> Self {
        Self {
            label: label.into(),
            status,
            actor: None,
            timestamp: None,
            note: None,
        }
    }

    pub fn with_actor(mut self, actor: impl Into<String>) -> Self {
        self.actor = Some(actor.into());
        self
    }

    pub fn with_timestamp(mut self, ts: impl Into<String>) -> Self {
        self.timestamp = Some(ts.into());
        self
    }

    pub fn with_note(mut self, note: Option<String>) -> Self {
        self.note = note.filter(|n| !n.trim().is_empty());
        self
    }
}

/// Timeline vertikal tahapan workflow.
///
/// The `dark` prop is gone. It existed because Pemakaian and Penghapusan were
/// light-themed pages while Kebutuhan BMN and Pakaian Dinas were not; both of
/// those pages are dark now, so the light arm had no callers left — and once
/// the palette was converted, two of its four arms had become identical to
/// their dark twin anyway. Both call sites passed `dark=true`.
#[component]
pub fn WorkflowTimeline(#[prop(into)] steps: Vec<WorkflowStep>) -> impl IntoView {
    let last = steps.len().saturating_sub(1);
    let line_class = "bg-white/10";
    let actor_class = "text-xs text-slate-400";
    let ts_class = "text-xs text-slate-500";
    let note_class = "mt-1 rounded bg-white/[0.04] px-2 py-1 text-xs italic text-slate-300";
    view! {
        <ol class="relative">
            {steps
                .into_iter()
                .enumerate()
                .map(|(idx, step)| {
                    let is_last = idx == last;
                    view! {
                        <li class="relative flex gap-4 pb-6">
                            // Connector line (kecuali langkah terakhir)
                            {(!is_last)
                                .then(|| {
                                    view! {
                                        <span class=format!(
                                            "absolute left-[7px] top-4 -bottom-0 w-px {}",
                                            line_class,
                                        )></span>
                                    }
                                })} // Dot
                            <span class=format!(
                                "relative z-10 mt-1 h-4 w-4 flex-shrink-0 rounded-full border-2 {}",
                                step.status.dot_class(),
                            )>// Body
                            </span> <div class="min-w-0 flex-1">
                                <p class=format!(
                                    "text-sm {}",
                                    step.status.label_class(),
                                )>{step.label}</p>
                                {step
                                    .actor
                                    .map(|a| {
                                        view! { <p class=actor_class>{a}</p> }
                                    })}
                                {step
                                    .timestamp
                                    .map(|t| {
                                        view! { <p class=ts_class>{t}</p> }
                                    })}
                                {step
                                    .note
                                    .map(|n| {
                                        view! { <p class=note_class>{n}</p> }
                                    })}
                            </div>
                        </li>
                    }
                })
                .collect_view()}
        </ol>
    }
}

// ============================================================================
// ApprovalDialog
// ============================================================================

/// Modal input catatan/alasan untuk aksi approve/return/reject workflow.
///
/// `on_confirm` menerima teks catatan (sudah di-trim). Bila `require_note`
/// aktif dan catatan kosong, dialog menampilkan error dan tidak memanggil
/// `on_confirm`.
#[component]
pub fn ApprovalDialog(
    #[prop(into)] title: String,
    #[prop(optional, into)] description: Option<String>,
    #[prop(optional, into)] note_label: Option<String>,
    #[prop(optional, into)] note_placeholder: Option<String>,
    #[prop(optional)] require_note: bool,
    /// Panjang minimum catatan (karakter). `None` = tanpa batas minimum.
    #[prop(optional)]
    min_note_len: Option<usize>,
    #[prop(optional, into)] confirm_label: Option<String>,
    #[prop(optional)] confirm_tone: Option<ActionTone>,
    #[prop(into)] on_confirm: Callback<String>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(optional, into)] busy: Option<Signal<bool>>,
) -> impl IntoView {
    let (note, set_note) = signal(String::new());
    let (error, set_error) = signal::<Option<String>>(None);

    let note_label = note_label.unwrap_or_else(|| {
        if require_note {
            "Catatan (wajib)".to_string()
        } else {
            "Catatan (opsional)".to_string()
        }
    });
    let confirm_label = confirm_label.unwrap_or_else(|| "Konfirmasi".to_string());
    let confirm_class = confirm_tone.unwrap_or(ActionTone::Primary).btn_class();

    let is_busy = move || busy.is_some_and(|s| s.get());

    let on_submit = move |_| {
        let text = note.get().trim().to_string();
        if require_note && text.is_empty() {
            set_error.set(Some("Catatan wajib diisi.".to_string()));
            return;
        }
        if let Some(min) = min_note_len {
            if text.chars().count() < min {
                set_error.set(Some(format!("Catatan minimal {} karakter.", min)));
                return;
            }
        }
        set_error.set(None);
        on_confirm.run(text);
    };

    view! {
        <div
            class="fixed inset-0 z-modal flex items-center justify-center bg-black/50 p-4"
            on:click=move |e| {
                if e.target() == e.current_target() && !is_busy() {
                    on_close.run(());
                }
            }
        >
            <div class="w-full max-w-md rounded-2xl border border-white/[0.06] bg-surface-panel p-6 shadow-xl">
                <h3 class="mb-2 text-lg font-semibold text-slate-100">{title}</h3>
                {description
                    .map(|d| {
                        view! { <p class="mb-4 text-sm text-slate-400">{d}</p> }
                    })}

                <div class="mb-4">
                    <label class="mb-1 block text-sm font-medium text-slate-200">{note_label}</label>
                    <textarea
                        class="w-full px-3 py-2 text-sm rounded-lg border border-white/10 bg-slate-900/70 text-slate-100 placeholder:text-slate-500 outline-none transition-colors hover:border-white/20 focus:border-gold-400/60 focus:ring-2 focus:ring-gold-400/40"
                        rows="3"
                        placeholder=note_placeholder.unwrap_or_default()
                        prop:value=move || note.get()
                        on:input=move |ev| set_note.set(event_target_value(&ev))
                    />
                    {move || {
                        error
                            .get()
                            .map(|msg| {
                                view! { <p class="mt-1 text-xs text-danger-400">{msg}</p> }
                            })
                    }}
                </div>

                <div class="flex justify-end gap-2">
                    <button
                        type="button"
                        class="rounded-lg border border-white/10 px-4 py-2 text-sm font-medium text-slate-200 hover:bg-white/[0.04] disabled:opacity-50"
                        prop:disabled=move || is_busy()
                        on:click=move |_| on_close.run(())
                    >
                        "Batal"
                    </button>
                    <button
                        type="button"
                        class=format!(
                            "rounded-lg px-4 py-2 text-sm font-semibold disabled:opacity-50 {}",
                            confirm_class,
                        )
                        prop:disabled=move || is_busy()
                        on:click=on_submit
                    >
                        {move || {
                            if is_busy() {
                                "Memproses...".to_string()
                            } else {
                                confirm_label.clone()
                            }
                        }}
                    </button>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// WorkflowActions
// ============================================================================

/// Satu tombol aksi workflow untuk [`WorkflowActions`].
#[derive(Clone)]
pub struct WorkflowAction {
    pub label: String,
    pub tone: ActionTone,
    pub on_click: Callback<()>,
}

impl WorkflowAction {
    pub fn new(label: impl Into<String>, tone: ActionTone, on_click: Callback<()>) -> Self {
        Self {
            label: label.into(),
            tone,
            on_click,
        }
    }
}

/// Deretan tombol aksi workflow. `busy` mendisable seluruh tombol saat ada
/// transition yang sedang berjalan.
#[component]
pub fn WorkflowActions(
    #[prop(into)] actions: Vec<WorkflowAction>,
    #[prop(optional, into)] busy: Option<Signal<bool>>,
) -> impl IntoView {
    let is_busy = move || busy.is_some_and(|s| s.get());
    view! {
        <div class="flex flex-wrap gap-3">
            {actions
                .into_iter()
                .map(|action| {
                    let cb = action.on_click;
                    view! {
                        <button
                            type="button"
                            class=format!(
                                "rounded-lg px-4 py-2 text-sm font-medium transition disabled:opacity-50 {}",
                                action.tone.btn_class(),
                            )
                            prop:disabled=move || is_busy()
                            on:click=move |_| cb.run(())
                        >
                            {action.label}
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}
