//! SLA editor — value + unit + preview + escalation toggle.
//!
//! Rendered inline inside `StepEditorModal`. Owns no persistence; the parent
//! reads the signals and calls `upsert_workflow_step` when the user saves.

use leptos::prelude::*;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{INFO, WARNING};

use super::time_unit::TimeUnit;
use crate::api::workflow::format_sla;

#[component]
pub fn SlaEditor(
    enabled: ReadSignal<bool>,
    set_enabled: WriteSignal<bool>,
    value: ReadSignal<u32>,
    set_value: WriteSignal<u32>,
    unit: ReadSignal<TimeUnit>,
    set_unit: WriteSignal<TimeUnit>,
    escalation: ReadSignal<bool>,
    set_escalation: WriteSignal<bool>,
    #[prop(default = false)] disabled: bool,
) -> impl IntoView {
    let minutes = move || {
        if enabled.get() {
            Some(unit.get().to_minutes(value.get()))
        } else {
            None
        }
    };

    let warning = move || {
        minutes().and_then(|m| {
            if m > 0 && m < 5 {
                Some("SLA terlalu pendek. Disarankan minimal 5 menit.")
            } else if m > 43200 {
                Some("SLA terlalu panjang (lebih dari 30 hari). Pertimbangkan untuk mengurangi.")
            } else {
                None
            }
        })
    };

    view! {
        <div class="flex flex-col gap-4">
            <label class="flex items-center gap-3 rounded-xl border border-white/[0.06] bg-white/[0.04] p-4">
                <input
                    type="checkbox"
                    prop:checked=move || enabled.get()
                    prop:disabled=move || disabled
                    on:change=move |e| set_enabled.set(event_target_checked(&e))
                    class="h-4 w-4 cursor-pointer accent-gold-400"
                />
                <div>
                    <div class="text-sm font-semibold text-white">"Aktifkan SLA Tracking"</div>
                    <div class="text-xs text-slate-400">
                        "Pantau dan eskalasi jika langkah ini melebihi batas waktu."
                    </div>
                </div>
            </label>

            {move || {
                enabled
                    .get()
                    .then(|| {
                        view! {
                            <div class="flex flex-col gap-4">
                                <div>
                                    <label class="mb-2 block text-sm font-semibold text-white">
                                        "Batas Waktu SLA " <span class="text-danger-400">"*"</span>
                                    </label>
                                    <div class="flex gap-3">
                                        <input
                                            type="number"
                                            min="1"
                                            prop:value=move || value.get()
                                            prop:disabled=move || disabled
                                            on:input=move |e| {
                                                if let Ok(v) = event_target_value(&e).parse::<u32>() {
                                                    set_value.set(v);
                                                }
                                            }
                                            class="focus-ring flex-1 rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white"
                                        />
                                        <select
                                            prop:disabled=move || disabled
                                            on:change=move |e| {
                                                set_unit.set(TimeUnit::parse(&event_target_value(&e)))
                                            }
                                            class="focus-ring min-w-[8rem] rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white"
                                        >
                                            <option
                                                value="minutes"
                                                selected=move || unit.get() == TimeUnit::Minutes
                                            >
                                                "Menit"
                                            </option>
                                            <option
                                                value="hours"
                                                selected=move || unit.get() == TimeUnit::Hours
                                            >
                                                "Jam"
                                            </option>
                                            <option
                                                value="days"
                                                selected=move || unit.get() == TimeUnit::Days
                                            >
                                                "Hari"
                                            </option>
                                        </select>
                                    </div>
                                </div>

                                <div class="rounded-xl border border-info-500/30 bg-info-500/10 p-4">
                                    <div class="mb-2 flex items-center gap-2 text-sm font-semibold text-info-400">
                                        <AppIcon icon=INFO />
                                        "Preview SLA"
                                    </div>
                                    <div class="text-xs leading-relaxed text-info-100">
                                        "Batas waktu: "
                                        <strong class="text-white">
                                            {move || format_sla(minutes().unwrap_or(0))}
                                        </strong> <br /> "Total: "
                                        <strong class="text-white">
                                            {move || format!("{} menit", minutes().unwrap_or(0))}
                                        </strong>
                                    </div>
                                </div>

                                <label class="flex items-center gap-3 rounded-xl border border-white/[0.06] bg-white/[0.04] p-4">
                                    <input
                                        type="checkbox"
                                        prop:checked=move || escalation.get()
                                        prop:disabled=move || disabled
                                        on:change=move |e| {
                                            set_escalation.set(event_target_checked(&e))
                                        }
                                        class="h-4 w-4 cursor-pointer accent-gold-400"
                                    />
                                    <div>
                                        <div class="text-sm font-semibold text-white">
                                            "Aktifkan Eskalasi Otomatis"
                                        </div>
                                        <div class="text-xs text-slate-400">
                                            "Kirim notifikasi eskalasi ke supervisor bila SLA dilanggar."
                                        </div>
                                    </div>
                                </label>

                                {move || {
                                    warning()
                                        .map(|msg| {
                                            view! {
                                                <div class="flex items-start gap-3 rounded-xl border border-warning-500/30 bg-warning-500/10 p-3">
                                                    <span class="mt-0.5 text-warning-400">
                                                        <AppIcon icon=WARNING />
                                                    </span>
                                                    <div class="text-xs text-warning-100">{msg}</div>
                                                </div>
                                            }
                                        })
                                }}
                            </div>
                        }
                    })
            }}
        </div>
    }
}
