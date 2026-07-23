//! List of workflow definitions as cards — primary landing surface for
//! workflow admin. Owns presentation only.
//!
//! View-only: the workflows are defined in Rust and this page reports them.
//! The edit/delete/create buttons were removed together with the backend write
//! endpoints they called, which never did anything but reject the request.

use leptos::prelude::*;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{EYE, GIT_BRANCH, TREE_STRUCTURE, USERS};

use crate::api::workflow::WorkflowDefinition;

#[component]
pub fn ConfigListPanel(
    workflows: Vec<WorkflowDefinition>,
    #[prop(into)] on_view: Callback<String>,
) -> impl IntoView {
    if workflows.is_empty() {
        // Reaching this means the service returned no definitions at all —
        // the four are compiled in, so it signals a backend/transport fault
        // rather than an empty collection the operator could fill.
        return view! {
            <div class="flex flex-col items-center gap-4 rounded-2xl border border-dashed border-white/[0.08] bg-white/[0.02] py-14 text-center">
                <span class="text-3xl text-slate-500">
                    <AppIcon icon=TREE_STRUCTURE />
                </span>
                <div>
                    <h3 class="text-base font-semibold text-white">"Definisi Workflow Tidak Termuat"</h3>
                    <p class="mt-1 text-sm text-slate-400">
                        "Workflow ditetapkan di sisi server. Daftar kosong menandakan gangguan layanan, bukan konfigurasi yang belum diisi."
                    </p>
                </div>
            </div>
        }
        .into_any();
    }

    let cards = workflows
        .into_iter()
        .map(|workflow| {
            let name_view = workflow.name.clone();
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
                        <span class="text-[0.6rem]">
                            <AppIcon icon=USERS />
                        </span>
                        "Parallel Approval"
                    </span>
                }
            });
            view! {
                <div class="flex flex-col gap-4 rounded-2xl border border-white/[0.06] bg-white/[0.02] p-5 transition hover:border-white/[0.12]">
                    <div class="flex items-start justify-between gap-3">
                        <div>
                            <div class="flex flex-wrap items-center gap-2">
                                <h3 class="text-base font-semibold text-white">
                                    {workflow.name.clone()}
                                </h3>
                                <span class=format!(
                                    "inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-[0.7rem] font-semibold {}",
                                    status_class,
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
                            <span class="text-[0.6rem] text-info-400">
                                <AppIcon icon=GIT_BRANCH />
                            </span>
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
                            <span class="mr-1.5">
                                <AppIcon icon=EYE />
                            </span>
                            "Detail"
                        </button>
                    </div>
                </div>
            }
        })
        .collect::<Vec<_>>();

    view! { <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">{cards}</div> }.into_any()
}
