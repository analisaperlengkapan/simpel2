//! Create / edit modal for a workflow definition. Replaces the legacy
//! `WorkflowEditModal` with design-token styling and lib-ui primitives.

use leptos::prelude::*;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{WARNING, X};
use leptos::task::spawn_local;

use crate::api::workflow::{
    CreateWorkflowRequest, UpdateWorkflowRequest, WorkflowDefinition, create_workflow_definition,
    update_workflow_definition,
};

#[component]
pub fn ConfigEditorPanel(
    /// Pass `None` to open in "create" mode; `Some(workflow)` to edit.
    workflow: Option<WorkflowDefinition>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_save: Callback<()>,
) -> impl IntoView {
    let is_create = workflow.is_none();
    let title = if is_create {
        "Buat Workflow Baru"
    } else {
        "Edit Workflow"
    };
    let subtitle = if is_create {
        "Buat definisi workflow baru untuk proses persetujuan"
    } else {
        "Perbarui metadata workflow yang sudah ada"
    };

    let (name, set_name) = signal(
        workflow
            .as_ref()
            .map(|w| w.name.clone())
            .unwrap_or_default(),
    );
    let (description, set_description) = signal(
        workflow
            .as_ref()
            .map(|w| w.description.clone())
            .unwrap_or_default(),
    );
    let (version, set_version) = signal(
        workflow
            .as_ref()
            .map(|w| w.version.clone())
            .unwrap_or_else(|| "1.0".to_string()),
    );
    let (status, set_status) = signal(
        workflow
            .as_ref()
            .map(|w| w.status.clone())
            .unwrap_or_else(|| "draft".to_string()),
    );
    let (parallel, set_parallel) = signal(
        workflow
            .as_ref()
            .map(|w| w.supports_parallel_approval)
            .unwrap_or(false),
    );
    let (saving, set_saving) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    let on_submit = move |_| {
        set_saving.set(true);
        set_error.set(None);

        let n = name.get();
        let d = description.get();
        let v = version.get();
        let s = status.get();
        let p = parallel.get();

        spawn_local(async move {
            let result = if is_create {
                create_workflow_definition(CreateWorkflowRequest {
                    name: n,
                    description: d,
                    version: v,
                    status: s,
                    supports_parallel_approval: p,
                })
                .await
                .map(|_| ())
            } else {
                update_workflow_definition(
                    &n,
                    UpdateWorkflowRequest {
                        description: Some(d),
                        version: Some(v),
                        status: Some(s),
                        supports_parallel_approval: Some(p),
                    },
                )
                .await
                .map(|_| ())
            };
            match result {
                Ok(_) => on_save.run(()),
                Err(e) => {
                    set_error.set(Some(e.to_string()));
                    set_saving.set(false);
                }
            }
        });
    };

    view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4 backdrop-blur-sm"
            on:click=move |e| {
                if e.target() == e.current_target() {
                    on_close.run(());
                }
            }
        >
            <div class="flex max-h-[92vh] w-full max-w-xl flex-col overflow-hidden rounded-2xl border border-white/[0.08] bg-surface-panel shadow-panel">
                <header class="flex items-start justify-between gap-4 border-b border-white/[0.06] p-6">
                    <div>
                        <h2 class="text-lg font-bold text-white">{title}</h2>
                        <p class="mt-1 text-xs text-slate-400">{subtitle}</p>
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
                    {move || error.get().map(|msg| view! {
                        <div class="mb-4 flex items-start gap-3 rounded-xl border border-danger-500/30 bg-danger-500/10 p-3">
                            <span class="mt-0.5 text-danger-400"><AppIcon icon=WARNING /></span>
                            <div class="text-xs text-danger-100">{msg}</div>
                        </div>
                    })}

                    <div class="flex flex-col gap-4">
                        <div>
                            <label class="mb-2 block text-sm font-semibold text-white">
                                "Nama Workflow "
                                <span class="text-danger-400">"*"</span>
                            </label>
                            <input
                                type="text"
                                placeholder="kebutuhan_bmn"
                                prop:value=move || name.get()
                                prop:disabled=move || saving.get() || !is_create
                                on:input=move |e| set_name.set(event_target_value(&e))
                                class="focus-ring w-full rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white placeholder:text-slate-500 disabled:cursor-not-allowed disabled:opacity-60"
                            />
                            <p class="mt-1 text-xs text-slate-500">
                                "Nama unik — tidak dapat diubah setelah workflow dibuat."
                            </p>
                        </div>

                        <div>
                            <label class="mb-2 block text-sm font-semibold text-white">
                                "Deskripsi "
                                <span class="text-danger-400">"*"</span>
                            </label>
                            <textarea
                                rows="3"
                                placeholder="Workflow persetujuan kebutuhan BMN..."
                                prop:value=move || description.get()
                                prop:disabled=move || saving.get()
                                on:input=move |e| set_description.set(event_target_value(&e))
                                class="focus-ring w-full rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white placeholder:text-slate-500"
                            ></textarea>
                        </div>

                        <div class="grid gap-4 sm:grid-cols-2">
                            <div>
                                <label class="mb-2 block text-sm font-semibold text-white">"Versi"</label>
                                <input
                                    type="text"
                                    placeholder="1.0"
                                    prop:value=move || version.get()
                                    prop:disabled=move || saving.get()
                                    on:input=move |e| set_version.set(event_target_value(&e))
                                    class="focus-ring w-full rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white placeholder:text-slate-500"
                                />
                            </div>
                            <div>
                                <label class="mb-2 block text-sm font-semibold text-white">"Status"</label>
                                <select
                                    prop:disabled=move || saving.get()
                                    on:change=move |e| set_status.set(event_target_value(&e))
                                    class="focus-ring w-full rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white"
                                >
                                    <option value="draft" selected=move || status.get() == "draft">"Draft"</option>
                                    <option value="active" selected=move || status.get() == "active">"Active"</option>
                                    <option value="inactive" selected=move || status.get() == "inactive">"Inactive"</option>
                                </select>
                            </div>
                        </div>

                        <label class="flex items-start gap-3 rounded-xl border border-white/[0.06] bg-white/[0.02] p-3">
                            <input
                                type="checkbox"
                                prop:checked=move || parallel.get()
                                prop:disabled=move || saving.get()
                                on:change=move |e| set_parallel.set(event_target_checked(&e))
                                class="mt-1 h-4 w-4 cursor-pointer accent-gold-400"
                            />
                            <div>
                                <div class="text-sm font-semibold text-white">"Dukungan Parallel Approval"</div>
                                <div class="mt-1 text-xs text-slate-400">
                                    "Izinkan beberapa approver menyetujui secara bersamaan (khusus kebutuhan_bmn)."
                                </div>
                            </div>
                        </label>
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
                            saving.get() || name.get().trim().is_empty() || description.get().trim().is_empty()
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
