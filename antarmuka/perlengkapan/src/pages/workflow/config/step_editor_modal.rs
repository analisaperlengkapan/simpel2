//! Step editor modal — edit a workflow step's role, next states, SLA and
//! escalation in one place.

use leptos::prelude::*;
use leptos::task::spawn_local;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{WARNING, X};

use super::sla_editor::SlaEditor;
use super::time_unit::{TimeUnit, best_time_unit};
use crate::api::workflow::{UpsertStepRequest, WorkflowStep, upsert_workflow_step};

#[component]
pub fn StepEditorModal(
    workflow_name: String,
    step: WorkflowStep,
    all_states: Vec<String>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_save: Callback<()>,
) -> impl IntoView {
    let (role, set_role) = signal(step.required_role.clone().unwrap_or_default());
    let (next_states, set_next_states) = signal::<Vec<String>>(step.next_states.clone());
    let (sla_enabled, set_sla_enabled) = signal(step.sla_minutes.is_some());
    let (sla_value, set_sla_value) = {
        let (v, _) = step
            .sla_minutes
            .map(best_time_unit)
            .unwrap_or((1, TimeUnit::Hours));
        signal(v)
    };
    let (sla_unit, set_sla_unit) = {
        let (_, u) = step
            .sla_minutes
            .map(best_time_unit)
            .unwrap_or((1, TimeUnit::Hours));
        signal(u)
    };
    let (escalation, set_escalation) = signal(step.escalation_enabled);
    let (saving, set_saving) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    let state_name = step.state_name.clone();
    let state_code = step.state_code;
    let state_label = step.state_name.clone();

    let workflow_name_submit = workflow_name.clone();
    let state_name_submit = step.state_name.clone();

    let on_submit = move |_| {
        set_saving.set(true);
        set_error.set(None);

        let role_val = role.get();
        let role_opt = (!role_val.trim().is_empty()).then_some(role_val.trim().to_string());
        let minutes_opt = if sla_enabled.get() {
            Some(sla_unit.get().to_minutes(sla_value.get()))
        } else {
            None
        };
        let request = UpsertStepRequest {
            state_name: state_name_submit.clone(),
            state_code,
            required_role: role_opt,
            sla_minutes: minutes_opt,
            next_states: next_states.get(),
            escalation_enabled: escalation.get(),
        };
        let wf = workflow_name_submit.clone();

        spawn_local(async move {
            match upsert_workflow_step(&wf, request).await {
                Ok(_) => on_save.run(()),
                Err(e) => {
                    set_error.set(Some(e.to_string()));
                    set_saving.set(false);
                }
            }
        });
    };

    let toggle_next_state = move |target: String| {
        set_next_states.update(|v| {
            if let Some(pos) = v.iter().position(|s| s == &target) {
                v.remove(pos);
            } else {
                v.push(target);
            }
        });
    };

    let state_name_for_header = state_name.clone();

    view! {
        <div
            class="fixed inset-0 z-modal flex items-center justify-center bg-black/60 p-4 backdrop-blur-sm"
            on:click=move |e| {
                if e.target() == e.current_target() {
                    on_close.run(());
                }
            }
        >
            <div class="flex max-h-[92vh] w-full max-w-2xl flex-col overflow-hidden rounded-2xl border border-white/[0.08] bg-surface-panel shadow-panel">
                <header class="flex items-start justify-between gap-4 border-b border-white/[0.06] p-6">
                    <div>
                        <h2 class="text-lg font-bold text-white">"Edit Langkah Workflow"</h2>
                        <p class="mt-1 text-xs text-slate-400">
                            "Konfigurasi state "
                            <strong class="text-gold-300">{state_name_for_header}</strong>
                        </p>
                    </div>
                    <button
                        type="button"
                        class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] p-2 text-slate-300 transition hover:bg-white/[0.08]"
                        on:click=move |_| on_close.run(())
                    >
                        <AppIcon icon=X />
                    </button>
                </header>

                <div class="flex-1 overflow-y-auto p-6">
                    {move || {
                        error
                            .get()
                            .map(|msg| {
                                view! {
                                    <div class="mb-4 flex items-start gap-3 rounded-xl border border-danger-500/30 bg-danger-500/10 p-3">
                                        <span class="mt-0.5 text-danger-400">
                                            <AppIcon icon=WARNING />
                                        </span>
                                        <div class="text-xs text-danger-100">{msg}</div>
                                    </div>
                                }
                            })
                    }} <div class="flex flex-col gap-5">
                        <div>
                            <label class="mb-2 block text-sm font-semibold text-white">
                                "State"
                            </label>
                            <input
                                type="text"
                                value=state_label.clone()
                                disabled=true
                                class="w-full cursor-not-allowed rounded-lg border border-white/[0.06] bg-white/[0.02] px-3 py-2 text-sm text-slate-400"
                            />
                            <p class="mt-1 text-xs text-slate-500">
                                "Nama state tidak dapat diubah. Hapus langkah dan buat yang baru jika perlu."
                            </p>
                        </div>

                        <div>
                            <label class="mb-2 block text-sm font-semibold text-white">
                                "Role yang Dibutuhkan"
                            </label>
                            <input
                                type="text"
                                placeholder="admin_pusat"
                                prop:value=move || role.get()
                                prop:disabled=move || saving.get()
                                on:input=move |e| set_role.set(event_target_value(&e))
                                class="focus-ring w-full rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white placeholder:text-slate-500"
                            />
                            <p class="mt-1 text-xs text-slate-500">
                                "Kosongkan bila state ini tidak membutuhkan role tertentu."
                            </p>
                        </div>

                        <div>
                            <label class="mb-2 block text-sm font-semibold text-white">
                                "Next States"
                            </label>
                            {
                                let all = all_states.clone();
                                if all.is_empty() {
                                    view! {
                                        <div class="rounded-lg border border-dashed border-white/[0.08] bg-white/[0.02] px-3 py-4 text-xs text-slate-400">
                                            "Belum ada state lain di workflow ini — tambah langkah lain terlebih dahulu."
                                        </div>
                                    }
                                        .into_any()
                                } else {
                                    view! {
                                        <div class="flex flex-wrap gap-2">
                                            {all
                                                .into_iter()
                                                .filter(|s| s != &state_name)
                                                .map(|candidate| {
                                                    let candidate_for_check = candidate.clone();
                                                    let candidate_for_click = candidate.clone();
                                                    let label = candidate.clone();
                                                    let is_selected = move || {
                                                        next_states.with(|v| v.contains(&candidate_for_check))
                                                    };
                                                    view! {
                                                        <button
                                                            type="button"
                                                            class=move || {
                                                                let base = "focus-ring rounded-lg border px-3 py-1.5 text-xs font-semibold transition";
                                                                if is_selected() {
                                                                    format!(
                                                                        "{} border-gold-400/40 bg-gold-400/15 text-gold-200",
                                                                        base,
                                                                    )
                                                                } else {
                                                                    format!(
                                                                        "{} border-white/[0.08] bg-white/[0.04] text-slate-300 hover:bg-white/[0.08]",
                                                                        base,
                                                                    )
                                                                }
                                                            }
                                                            on:click=move |_| toggle_next_state(
                                                                candidate_for_click.clone(),
                                                            )
                                                        >
                                                            {label}
                                                        </button>
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </div>
                                    }
                                        .into_any()
                                }
                            }
                            <p class="mt-2 text-xs text-slate-500">
                                "Klik state untuk menandai sebagai transisi yang diperbolehkan dari langkah ini."
                            </p>
                        </div>

                        <div>
                            <label class="mb-2 block text-sm font-semibold text-white">
                                "SLA & Eskalasi"
                            </label>
                            <SlaEditor
                                enabled=sla_enabled
                                set_enabled=set_sla_enabled
                                value=sla_value
                                set_value=set_sla_value
                                unit=sla_unit
                                set_unit=set_sla_unit
                                escalation=escalation
                                set_escalation=set_escalation
                            />
                        </div>
                    </div>
                </div>

                <footer class="flex justify-end gap-2 border-t border-white/[0.06] p-4">
                    <button
                        type="button"
                        class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] px-4 py-2 text-sm font-medium text-slate-200 transition hover:bg-white/[0.08]"
                        prop:disabled=move || saving.get()
                        on:click=move |_| on_close.run(())
                    >
                        "Batal"
                    </button>
                    <button
                        type="button"
                        class="focus-ring rounded-lg bg-gold-gradient px-4 py-2 text-sm font-semibold text-navy-950 shadow-card transition hover:brightness-105 disabled:opacity-60"
                        prop:disabled=move || {
                            saving.get() || (sla_enabled.get() && sla_value.get() == 0)
                        }
                        on:click=on_submit
                    >
                        {move || if saving.get() { "Menyimpan..." } else { "Simpan" }}
                    </button>
                </footer>
            </div>
        </div>
    }
}
