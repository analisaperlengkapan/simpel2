//! Simple delete-confirmation modal used by the workflow admin page.

use leptos::prelude::*;
use leptos::task::spawn_local;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::WARNING;

use crate::api::workflow::delete_workflow_definition;

#[component]
pub fn DeleteConfigModal(
    workflow_name: String,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_confirm: Callback<()>,
) -> impl IntoView {
    let (deleting, set_deleting) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    let name_for_submit = workflow_name.clone();
    let name_for_label = workflow_name.clone();
    let on_delete = move |_| {
        set_deleting.set(true);
        set_error.set(None);
        let name_val = name_for_submit.clone();
        spawn_local(async move {
            match delete_workflow_definition(&name_val).await {
                Ok(_) => on_confirm.run(()),
                Err(e) => {
                    set_error.set(Some(e.to_string()));
                    set_deleting.set(false);
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
            <div class="w-full max-w-md rounded-2xl border border-white/[0.08] bg-surface-panel p-6 shadow-panel">
                <div class="mb-5 flex flex-col items-center text-center">
                    <div class="mb-4 flex h-14 w-14 items-center justify-center rounded-full border-2 border-danger-500/30 bg-danger-500/10 text-danger-400">
                        <span class="text-xl"><AppIcon icon=WARNING /></span>
                    </div>
                    <h2 class="text-lg font-bold text-white">"Hapus Workflow?"</h2>
                    <p class="mt-2 text-sm text-slate-400">
                        "Anda yakin ingin menghapus workflow "
                        <strong class="text-white">{name_for_label}</strong>
                        "? Tindakan ini tidak dapat dibatalkan."
                    </p>
                </div>

                {move || error.get().map(|msg| view! {
                    <div class="mb-4 flex items-start gap-3 rounded-xl border border-danger-500/30 bg-danger-500/10 p-3">
                        <span class="mt-0.5 text-danger-400"><AppIcon icon=WARNING /></span>
                        <div class="text-xs text-danger-100">{msg}</div>
                    </div>
                })}

                <div class="flex gap-2">
                    <button
                        type="button"
                        class="focus-ring flex-1 rounded-lg border border-white/[0.08] bg-white/[0.04] px-4 py-2 text-sm font-medium text-slate-200 transition hover:bg-white/[0.08]"
                        prop:disabled=move || deleting.get()
                        on:click=move |_| on_close.run(())
                    >
                        "Batal"
                    </button>
                    <button
                        type="button"
                        class="focus-ring flex-1 rounded-lg bg-danger-500 px-4 py-2 text-sm font-semibold text-white transition hover:bg-danger-600 disabled:opacity-60"
                        prop:disabled=move || deleting.get()
                        on:click=on_delete
                    >
                        {move || if deleting.get() { "Menghapus..." } else { "Hapus" }}
                    </button>
                </div>
            </div>
        </div>
    }
}
