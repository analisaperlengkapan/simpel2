//! Workflow configuration admin page.
//!
//! Thin orchestrator that composes the decomposed modules under
//! [`crate::pages::workflow::config`]. Presentation and per-modal state live
//! in the submodules; this file owns routing concerns, the `Resource` that
//! feeds the list and the detail drawer, and the modal-visibility signals.

use leptos::prelude::*;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{CHECK, CIRCLE, CLOCK, FLAG_CHECKERED, GEAR, MINUS, PLUS, USER_LIST, X};

use super::config::{
    ConfigEditorPanel, ConfigJsonPreview, ConfigListPanel, DeleteConfigModal, RoleMatrix,
    StepEditorModal, TransitionMatrix,
};
use crate::api::workflow::{
    WorkflowDefinition, WorkflowStep, fetch_workflow_definition_detail, fetch_workflow_definitions,
    format_sla,
};
use crate::components::layout::{ErrorState, LoadingState, PageLayout, SectionCard};

#[component]
pub fn WorkflowConfigManagement() -> impl IntoView {
    let list_resource = LocalResource::new(fetch_workflow_definitions);

    let (selected, set_selected) = signal::<Option<String>>(None);
    let (editing, set_editing) = signal::<Option<WorkflowDefinition>>(None);
    let (deleting, set_deleting) = signal::<Option<String>>(None);
    let (show_create, set_show_create) = signal(false);

    let on_view = Callback::new(move |name: String| set_selected.set(Some(name)));
    let on_edit = Callback::new(move |wf: WorkflowDefinition| set_editing.set(Some(wf)));
    let on_delete = Callback::new(move |name: String| set_deleting.set(Some(name)));
    let on_create = Callback::new(move |_| set_show_create.set(true));

    let close_create_or_edit = Callback::new(move |_| {
        set_editing.set(None);
        set_show_create.set(false);
    });
    let save_and_refresh = Callback::new(move |_| {
        set_editing.set(None);
        set_show_create.set(false);
        list_resource.refetch();
    });
    let close_delete = Callback::new(move |_| set_deleting.set(None));
    let confirm_delete = Callback::new(move |_| {
        set_deleting.set(None);
        list_resource.refetch();
    });
    let close_detail = Callback::new(move |_| set_selected.set(None));

    view! {
        <PageLayout
            title="Konfigurasi Workflow"
            description="Kelola definisi workflow, langkah, role, transisi dan SLA untuk seluruh proses BMN."
            icon="fas fa-project-diagram"
            actions=Box::new(move || {
                view! {
                    <button
                        type="button"
                        class="focus-ring rounded-lg bg-gold-gradient px-4 py-2 text-sm font-semibold text-navy-950 shadow-card transition hover:brightness-105"
                        on:click=move |_| set_show_create.set(true)
                    >
                        <span class="mr-2"><AppIcon icon=PLUS /></span>
                        "Workflow Baru"
                    </button>
                }
                .into_any()
            })
        >
            <SectionCard title="Definisi Workflow" icon="fas fa-list">
                <Suspense fallback=move || view! { <LoadingState message="Memuat daftar workflow..." /> }>
                    {move || list_resource.get().map(|result| match result {
                        Ok(response) => view! {
                            <ConfigListPanel
                                workflows=response.data
                                on_view=on_view
                                on_edit=on_edit
                                on_delete=on_delete
                                on_create=on_create
                            />
                        }
                        .into_any(),
                        Err(e) => {
                            let retry: Box<dyn Fn()> = Box::new(move || { list_resource.refetch(); });
                            view! {
                                <ErrorState error=e on_retry=retry />
                            }
                            .into_any()
                        },
                    })}
                </Suspense>
            </SectionCard>

            {move || selected.get().map(|name| view! {
                <WorkflowDetailDrawer
                    workflow_name=name
                    on_close=close_detail
                />
            })}

            {move || editing.get().map(|wf| view! {
                <ConfigEditorPanel
                    workflow=Some(wf)
                    on_close=close_create_or_edit
                    on_save=save_and_refresh
                />
            })}

            {move || show_create.get().then(|| view! {
                <ConfigEditorPanel
                    workflow=None
                    on_close=close_create_or_edit
                    on_save=save_and_refresh
                />
            })}

            {move || deleting.get().map(|name| view! {
                <DeleteConfigModal
                    workflow_name=name
                    on_close=close_delete
                    on_confirm=confirm_delete
                />
            })}
        </PageLayout>
    }
}

/// Inline drawer-style panel with workflow detail: steps, transition matrix,
/// role matrix, and JSON preview. Opens from the list and hosts the step
/// editor as a nested modal.
#[component]
fn WorkflowDetailDrawer(
    workflow_name: String,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    let name_for_resource = workflow_name.clone();
    let detail_resource = LocalResource::new(move || {
        let name = name_for_resource.clone();
        async move { fetch_workflow_definition_detail(&name).await }
    });

    let (edit_step, set_edit_step) = signal::<Option<WorkflowStep>>(None);
    let workflow_name_for_modal = workflow_name.clone();

    let close_step_editor = Callback::new(move |_| set_edit_step.set(None));
    let save_step_editor = Callback::new(move |_| {
        set_edit_step.set(None);
        detail_resource.refetch();
    });

    view! {
        <div
            class="fixed inset-0 z-40 flex items-start justify-center overflow-y-auto bg-black/60 p-4 backdrop-blur-sm sm:p-8"
            on:click=move |e| {
                if e.target() == e.current_target() {
                    on_close.run(());
                }
            }
        >
            <div class="my-auto flex w-full max-w-5xl flex-col gap-5 rounded-2xl border border-white/[0.08] bg-surface-panel p-6 shadow-panel">
                <header class="flex items-start justify-between gap-3">
                    <div>
                        <div class="text-xs font-semibold uppercase tracking-wider text-gold-300">
                            "Detail Workflow"
                        </div>
                        <h2 class="mt-1 text-xl font-bold text-white">{workflow_name.clone()}</h2>
                    </div>
                    <button
                        type="button"
                        class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] p-2 text-slate-300 transition hover:bg-white/[0.08]"
                        on:click=move |_| on_close.run(())
                    >
                        <AppIcon icon=X />
                    </button>
                </header>

                <Suspense fallback=move || view! { <LoadingState message="Memuat detail workflow..." /> }>
                    {move || detail_resource.get().map(|result| match result {
                        Ok(response) => {
                            let detail = response.data;
                            let steps = detail.steps.clone();
                            let detail_for_json = detail.clone();
                            let steps_for_matrix = steps.clone();
                            let steps_for_roles = steps.clone();
                            let step_rows = steps
                                .into_iter()
                                .map(|step| {
                                    let step_for_edit = step.clone();
                                    let sla_label = step
                                        .sla_minutes
                                        .map(format_sla)
                                        .unwrap_or_else(|| "—".to_string());
                                    let has_sla = step.sla_minutes.is_some();
                                    let role_label = step
                                        .required_role
                                        .clone()
                                        .unwrap_or_else(|| "—".to_string());
                                    let has_role = step.required_role.is_some();
                                    let next_states_label = if step.next_states.is_empty() {
                                        "Terminal".to_string()
                                    } else {
                                        step.next_states.join(", ")
                                    };
                                    let is_terminal = step.is_terminal;
                                    let escalation_on = step.escalation_enabled;
                                    view! {
                                        <tr class="border-b border-white/[0.04]">
                                            <td class="px-3 py-3">
                                                <div class="flex items-center gap-2">
                                                    <span class=if is_terminal { "inline-flex text-danger-400" } else { "inline-flex text-info-400" }>
                                                        <AppIcon icon=if is_terminal { FLAG_CHECKERED } else { CIRCLE } size=10 />
                                                    </span>
                                                    <span class="text-sm font-semibold text-white">
                                                        {step.state_name.clone()}
                                                    </span>
                                                </div>
                                            </td>
                                            <td class="px-3 py-3">
                                                <span class=if has_role {
                                                    "inline-flex items-center gap-1.5 rounded-lg border border-info-500/30 bg-info-500/10 px-2.5 py-1 text-[0.7rem] font-semibold text-info-300"
                                                } else {
                                                    "inline-flex items-center gap-1.5 rounded-lg border border-white/[0.06] bg-white/[0.02] px-2.5 py-1 text-[0.7rem] font-semibold text-slate-500"
                                                }>
                                                    <span class="text-[0.6rem]"><AppIcon icon=USER_LIST /></span>
                                                    {role_label}
                                                </span>
                                            </td>
                                            <td class="px-3 py-3">
                                                <span class=if has_sla {
                                                    "inline-flex items-center gap-1.5 rounded-lg border border-warning-500/30 bg-warning-500/10 px-2.5 py-1 text-[0.7rem] font-semibold text-warning-400"
                                                } else {
                                                    "inline-flex items-center gap-1.5 rounded-lg border border-white/[0.06] bg-white/[0.02] px-2.5 py-1 text-[0.7rem] font-semibold text-slate-500"
                                                }>
                                                    <span class="text-[0.6rem]"><AppIcon icon=CLOCK /></span>
                                                    {sla_label}
                                                </span>
                                            </td>
                                            <td class="px-3 py-3 text-xs text-slate-300">
                                                {next_states_label}
                                            </td>
                                            <td class="px-3 py-3 text-center">
                                                {if escalation_on {
                                                    view! {
                                                        <span class="inline-flex h-6 w-6 items-center justify-center rounded-md border border-success-500/30 bg-success-500/15 text-success-400">
                                                            <span class="text-[0.65rem]"><AppIcon icon=CHECK /></span>
                                                        </span>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <span class="inline-flex h-6 w-6 items-center justify-center rounded-md border border-white/[0.06] bg-white/[0.02] text-slate-500">
                                                            <span class="text-[0.65rem]"><AppIcon icon=MINUS /></span>
                                                        </span>
                                                    }.into_any()
                                                }}
                                            </td>
                                            <td class="px-3 py-3 text-right">
                                                <button
                                                    type="button"
                                                    class="focus-ring rounded-md border border-info-500/30 bg-info-500/10 px-2.5 py-1 text-[0.7rem] font-semibold text-info-300 transition hover:bg-info-500/20"
                                                    title="Edit langkah"
                                                    on:click=move |_| set_edit_step.set(Some(step_for_edit.clone()))
                                                >
                                                    <AppIcon icon=GEAR />
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                })
                                .collect::<Vec<_>>();

                            view! {
                                <div class="flex flex-col gap-5">
                                    <SectionCard title="Langkah Workflow" icon="fas fa-stream" dense=true>
                                        <div class="overflow-x-auto rounded-xl border border-white/[0.06]">
                                            <table class="min-w-full border-collapse">
                                                <thead>
                                                    <tr class="bg-white/[0.02]">
                                                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"State"</th>
                                                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"Role"</th>
                                                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"SLA"</th>
                                                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"Next States"</th>
                                                        <th class="border-b border-white/[0.06] px-3 py-2 text-center text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"Eskalasi"</th>
                                                        <th class="border-b border-white/[0.06] px-3 py-2 text-right text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"Aksi"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>{step_rows}</tbody>
                                            </table>
                                        </div>
                                    </SectionCard>

                                    <SectionCard title="Matriks Transisi" icon="fas fa-random" dense=true
                                        description="Setiap tanda centang menunjukkan transisi yang diperbolehkan dari state baris ke state kolom.">
                                        <TransitionMatrix steps=steps_for_matrix />
                                    </SectionCard>

                                    <SectionCard title="Matriks Role" icon="fas fa-user-shield" dense=true
                                        description="Pemetaan role yang boleh mengeksekusi setiap state.">
                                        <RoleMatrix steps=steps_for_roles />
                                    </SectionCard>

                                    <SectionCard title="Preview JSON" icon="fas fa-code" dense=true>
                                        <ConfigJsonPreview detail=detail_for_json />
                                    </SectionCard>
                                </div>
                            }
                            .into_any()
                        }
                        Err(e) => {
                            let retry: Box<dyn Fn()> = Box::new(move || { detail_resource.refetch(); });
                            view! {
                                <ErrorState error=e on_retry=retry />
                            }
                            .into_any()
                        },
                    })}
                </Suspense>
            </div>

            {move || {
                detail_resource.get().and_then(|r| r.ok()).and_then(|resp| {
                    edit_step.get().map(|step| {
                        let all_states: Vec<String> = resp
                            .data
                            .steps
                            .iter()
                            .map(|s| s.state_name.clone())
                            .collect();
                        view! {
                            <StepEditorModal
                                workflow_name=workflow_name_for_modal.clone()
                                step=step
                                all_states=all_states
                                on_close=close_step_editor
                                on_save=save_step_editor
                            />
                        }
                    })
                })
            }}
        </div>
    }
}
