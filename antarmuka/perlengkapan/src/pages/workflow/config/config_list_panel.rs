//! List of workflow definitions as cards — primary landing surface for
//! workflow admin. Owns presentation only; all state mutations flow back to
//! the parent via callbacks.

use leptos::prelude::*;

use crate::api::workflow::WorkflowDefinition;

#[component]
pub fn ConfigListPanel(
    workflows: Vec<WorkflowDefinition>,
    #[prop(into)] on_view: Callback<String>,
    #[prop(into)] on_edit: Callback<WorkflowDefinition>,
    #[prop(into)] on_delete: Callback<String>,
    #[prop(into)] on_create: Callback<()>,
) -> impl IntoView {
    if workflows.is_empty() {
        return view! {
            <div class="flex flex-col items-center gap-4 rounded-2xl border border-dashed border-white/[0.08] bg-white/[0.02] py-14 text-center">
                <i class="fas fa-project-diagram text-3xl text-slate-500"></i>
                <div>
                    <h3 class="text-base font-semibold text-white">"Belum Ada Workflow"</h3>
                    <p class="mt-1 text-sm text-slate-400">
                        "Buat konfigurasi workflow pertama untuk memulai."
                    </p>
                </div>
                <button
                    type="button"
                    class="focus-ring rounded-lg bg-gold-gradient px-4 py-2 text-sm font-semibold text-navy-950 shadow-card transition hover:brightness-105"
                    on:click=move |_| on_create.run(())
                >
                    <i class="fas fa-plus mr-2"></i>
                    "Buat Workflow Pertama"
                </button>
            </div>
        }
        .into_any();
    }

    let cards = workflows
        .into_iter()
        .map(|workflow| {
            let name_view = workflow.name.clone();
            let name_edit = workflow.name.clone();
            let name_delete = workflow.name.clone();
            let workflow_for_edit = workflow.clone();
            let (status_class, status_label) = match workflow.status.as_str() {
                "active" => (
                    "border-success-500/40 bg-success-500/15 text-success-400",
                    "Aktif",
                ),
                "draft" => (
                    "border-warning-500/40 bg-warning-500/15 text-warning-400",
                    "Draft",
                ),
                _ => (
                    "border-white/[0.08] bg-white/[0.04] text-slate-400",
                    "Tidak Aktif",
                ),
            };
            let parallel_chip = workflow.supports_parallel_approval.then(|| {
                view! {
                    <span class="inline-flex items-center gap-1.5 rounded-lg border border-success-500/30 bg-success-500/10 px-2.5 py-1 text-[0.7rem] font-semibold text-success-400">
                        <i class="fas fa-users text-[0.6rem]"></i>
                        "Parallel Approval"
                    </span>
                }
            });
            view! {
                <div class="flex flex-col gap-4 rounded-2xl border border-white/[0.06] bg-white/[0.02] p-5 transition hover:border-white/[0.12]">
                    <div class="flex items-start justify-between gap-3">
                        <div>
                            <div class="flex flex-wrap items-center gap-2">
                                <h3 class="text-base font-semibold text-white">{workflow.name.clone()}</h3>
                                <span class=format!(
                                    "inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-[0.7rem] font-semibold {}",
                                    status_class
                                )>
                                    <span class="h-1.5 w-1.5 rounded-full bg-current"></span>
                                    {status_label}
                                </span>
                            </div>
                            <p class="mt-2 text-sm leading-relaxed text-slate-400">
                                {workflow.description.clone()}
                            </p>
                        </div>
                    </div>
                    <div class="flex flex-wrap gap-2">
                        <span class="inline-flex items-center gap-1.5 rounded-lg border border-white/[0.06] bg-white/[0.04] px-2.5 py-1 text-[0.7rem] font-semibold text-slate-300">
                            <i class="fas fa-code-branch text-[0.6rem] text-info-400"></i>
                            {format!("v{}", workflow.version)}
                        </span>
                        {parallel_chip}
                    </div>
                    <div class="flex gap-2">
                        <button
                            type="button"
                            class="focus-ring flex-1 rounded-lg border border-info-500/30 bg-info-500/10 px-3 py-2 text-sm font-semibold text-info-300 transition hover:bg-info-500/20"
                            on:click=move |_| on_view.run(name_view.clone())
                        >
                            <i class="fas fa-eye mr-1.5"></i>
                            "Detail"
                        </button>
                        <button
                            type="button"
                            class="focus-ring flex-1 rounded-lg border border-warning-500/30 bg-warning-500/10 px-3 py-2 text-sm font-semibold text-warning-400 transition hover:bg-warning-500/20"
                            on:click={
                                let workflow = workflow_for_edit.clone();
                                move |_| on_edit.run(workflow.clone())
                            }
                        >
                            <i class="fas fa-pen mr-1.5"></i>
                            "Edit"
                        </button>
                        <button
                            type="button"
                            class="focus-ring rounded-lg border border-danger-500/30 bg-danger-500/10 px-3 py-2 text-sm font-semibold text-danger-400 transition hover:bg-danger-500/20"
                            on:click=move |_| on_delete.run(name_delete.clone())
                            title="Hapus"
                        >
                            <i class="fas fa-trash"></i>
                        </button>
                    </div>
                </div>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">{cards}</div>
    }
    .into_any()
}
