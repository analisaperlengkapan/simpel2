//! Batch Operations Toolbar Component
//!
//! Provides batch action buttons and confirmation dialogs for multi-select operations.
//! Requirements: REQ-K004

use leptos::prelude::*;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{CHECK, CHECK_CIRCLE, CHECK_SQUARE, DOTS_THREE, SPINNER, WARNING, X};
use uuid::Uuid;

use crate::api::{batch_approve_kebutuhan, batch_reject_kebutuhan};

#[derive(Debug, Clone)]
pub struct BatchOperationResult {
    pub batch_id: String,
    pub total_items: usize,
    pub successful_items: usize,
    pub failed_items: usize,
}

#[component]
pub fn BatchOperationsToolbar(
    /// List of selected kebutuhan IDs
    selected_ids: ReadSignal<Vec<Uuid>>,
    /// Callback to clear selection after operation
    on_operation_complete: Callback<BatchOperationResult>,
) -> impl IntoView {
    let (show_approve_dialog, set_show_approve_dialog) = signal(false);
    let (show_reject_dialog, set_show_reject_dialog) = signal(false);
    let (_show_status_dialog, set_show_status_dialog) = signal(false);
    let (reject_reason, set_reject_reason) = signal(String::new());
    let (is_processing, set_is_processing) = signal(false);
    let (error_message, set_error_message) = signal::<Option<String>>(None);

    let selected_count = Memo::new(move |_| selected_ids.get().len());

    // Handle batch approve
    let handle_approve = Action::new_local(move |_: &()| {
        let ids = selected_ids.get();
        async move {
            set_is_processing.set(true);
            set_error_message.set(None);

            match batch_approve_kebutuhan(ids, None).await {
                Ok(response) => {
                    set_show_approve_dialog.set(false);
                    on_operation_complete.run(BatchOperationResult {
                        batch_id: response.batch_id,
                        total_items: response.total_items,
                        successful_items: response.successful_items,
                        failed_items: response.failed_items,
                    });
                }
                Err(e) => {
                    set_error_message.set(Some(format!("Gagal melakukan approval: {}", e)));
                }
            }

            set_is_processing.set(false);
        }
    });

    // Handle batch reject
    let handle_reject = Action::new_local(move |_: &()| {
        let ids = selected_ids.get();
        let reason = reject_reason.get();
        async move {
            if reason.len() < 10 {
                set_error_message.set(Some("Alasan penolakan minimal 10 karakter".to_string()));
                return;
            }

            set_is_processing.set(true);
            set_error_message.set(None);

            match batch_reject_kebutuhan(ids, reason).await {
                Ok(response) => {
                    set_show_reject_dialog.set(false);
                    set_reject_reason.set(String::new());
                    on_operation_complete.run(BatchOperationResult {
                        batch_id: response.batch_id,
                        total_items: response.total_items,
                        successful_items: response.successful_items,
                        failed_items: response.failed_items,
                    });
                }
                Err(e) => {
                    set_error_message.set(Some(format!("Gagal melakukan penolakan: {}", e)));
                }
            }

            set_is_processing.set(false);
        }
    });

    view! {
        <Show when=move || selected_count.get() != 0>
            <div class="fixed bottom-6 left-1/2 transform -translate-x-1/2 z-popover">
                <div class="bg-surface-panel rounded-lg shadow-2xl border border-white/[0.06] p-4 flex items-center gap-4">
                    // Selection count
                    <div class="flex items-center gap-2 px-3 py-2 bg-info-500/10 rounded-lg">
                        <span class="text-info-400">
                            <AppIcon icon=CHECK_SQUARE />
                        </span>
                        <span class="font-semibold text-blue-900">
                            {move || selected_count.get()} " item dipilih"
                        </span>
                    </div>

                    // Action buttons
                    <div class="flex gap-2">
                        // Approve button
                        <button
                            class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors inline-flex items-center"
                            on:click=move |_| set_show_approve_dialog.set(true)
                            disabled=move || is_processing.get()
                        >
                            <span class="mr-2">
                                <AppIcon icon=CHECK />
                            </span>
                            "Setujui"
                        </button>

                        // Reject button
                        <button
                            class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors inline-flex items-center"
                            on:click=move |_| set_show_reject_dialog.set(true)
                            disabled=move || is_processing.get()
                        >
                            <span class="mr-2">
                                <AppIcon icon=X />
                            </span>
                            "Tolak"
                        </button>

                        // More actions button
                        <button
                            class="px-4 py-2 rounded-lg border border-white/10 bg-white/[0.06] text-slate-100 hover:bg-white/[0.1] transition-colors inline-flex items-center"
                            on:click=move |_| set_show_status_dialog.set(true)
                            disabled=move || is_processing.get()
                        >
                            <span class="mr-2">
                                <AppIcon icon=DOTS_THREE />
                            </span>
                            "Lainnya"
                        </button>
                    </div>
                </div>
            </div>

            // Approve confirmation dialog
            <Show when=move || show_approve_dialog.get()>
                <BatchApproveDialog
                    selected_count=selected_count
                    is_processing=is_processing
                    error_message=error_message
                    on_confirm=Callback::new(move |_| {
                        handle_approve.dispatch(());
                    })
                    on_cancel=Callback::new(move |_| set_show_approve_dialog.set(false))
                />
            </Show>

            // Reject confirmation dialog
            <Show when=move || show_reject_dialog.get()>
                <BatchRejectDialog
                    selected_count=selected_count
                    is_processing=is_processing
                    error_message=error_message
                    reject_reason=reject_reason
                    set_reject_reason=set_reject_reason
                    on_confirm=Callback::new(move |_| {
                        handle_reject.dispatch(());
                    })
                    on_cancel=Callback::new(move |_| {
                        set_show_reject_dialog.set(false);
                        set_reject_reason.set(String::new());
                    })
                />
            </Show>
        </Show>
    }
}

#[component]
fn BatchApproveDialog(
    selected_count: Memo<usize>,
    is_processing: ReadSignal<bool>,
    error_message: ReadSignal<Option<String>>,
    on_confirm: Callback<()>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-modal">
            <div class="mx-4 w-full max-w-md rounded-2xl border border-white/[0.06] bg-surface-panel p-6 shadow-xl">
                <div class="flex items-center gap-3 mb-4">
                    <div class="w-12 h-12 bg-success-500/15 rounded-full flex items-center justify-center">
                        <span class="text-success-400 text-xl">
                            <AppIcon icon=CHECK />
                        </span>
                    </div>
                    <div>
                        <h3 class="text-lg font-bold text-slate-100">"Konfirmasi Approval"</h3>
                        <p class="text-sm text-slate-500">"Batch operation"</p>
                    </div>
                </div>

                <p class="text-slate-200 mb-6">
                    "Anda akan menyetujui "
                    <span class="font-bold">{move || selected_count.get()}</span>
                    " pengajuan kebutuhan BMN. Operasi ini tidak dapat dibatalkan."
                </p>

                <Show when=move || error_message.get().is_some()>
                    <div class="mb-4 p-3 bg-danger-500/10 border border-danger-500/20 rounded-lg">
                        <p class="text-sm text-danger-300">
                            {move || error_message.get().unwrap_or_default()}
                        </p>
                    </div>
                </Show>

                <div class="flex gap-3 justify-end">
                    <button
                        class="px-4 py-2 border border-white/10 rounded-lg hover:bg-white/[0.04] transition-colors"
                        on:click=move |_| on_cancel.run(())
                        disabled=move || is_processing.get()
                    >
                        "Batal"
                    </button>
                    <button
                        class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors inline-flex items-center"
                        on:click=move |_| on_confirm.run(())
                        disabled=move || is_processing.get()
                    >
                        <Show when=move || is_processing.get()>
                            <span class="fa-spin mr-2">
                                <AppIcon icon=SPINNER />
                            </span>
                        </Show>
                        "Ya, Setujui"
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn BatchRejectDialog(
    selected_count: Memo<usize>,
    is_processing: ReadSignal<bool>,
    error_message: ReadSignal<Option<String>>,
    reject_reason: ReadSignal<String>,
    set_reject_reason: WriteSignal<String>,
    on_confirm: Callback<()>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-modal">
            <div class="mx-4 w-full max-w-md rounded-2xl border border-white/[0.06] bg-surface-panel p-6 shadow-xl">
                <div class="flex items-center gap-3 mb-4">
                    <div class="w-12 h-12 bg-danger-500/15 rounded-full flex items-center justify-center">
                        <span class="text-danger-400 text-xl">
                            <AppIcon icon=X />
                        </span>
                    </div>
                    <div>
                        <h3 class="text-lg font-bold text-slate-100">"Konfirmasi Penolakan"</h3>
                        <p class="text-sm text-slate-500">"Batch operation"</p>
                    </div>
                </div>

                <p class="text-slate-200 mb-4">
                    "Anda akan menolak "
                    <span class="font-bold">{move || selected_count.get()}</span>
                    " pengajuan kebutuhan BMN."
                </p>

                <div class="mb-4">
                    <label class="block text-sm font-medium text-slate-200 mb-2">
                        "Alasan Penolakan" <span class="text-danger-400">"*"</span>
                    </label>
                    <textarea
                        class="w-full px-3 py-2 rounded-lg border border-white/10 bg-slate-900/70 text-slate-100 placeholder:text-slate-500 outline-none transition-colors focus:border-danger-400/60 focus:ring-2 focus:ring-danger-400/40"
                        rows="4"
                        placeholder="Masukkan alasan penolakan (minimal 10 karakter)..."
                        on:input=move |ev| set_reject_reason.set(event_target_value(&ev))
                        prop:value=move || reject_reason.get()
                    />
                    <p class="text-xs text-slate-500 mt-1">
                        {move || reject_reason.get().len()} " / 10 karakter minimum"
                    </p>
                </div>

                <Show when=move || error_message.get().is_some()>
                    <div class="mb-4 p-3 bg-danger-500/10 border border-danger-500/20 rounded-lg">
                        <p class="text-sm text-danger-300">
                            {move || error_message.get().unwrap_or_default()}
                        </p>
                    </div>
                </Show>

                <div class="flex gap-3 justify-end">
                    <button
                        class="px-4 py-2 border border-white/10 rounded-lg hover:bg-white/[0.04] transition-colors"
                        on:click=move |_| on_cancel.run(())
                        disabled=move || is_processing.get()
                    >
                        "Batal"
                    </button>
                    <button
                        class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors inline-flex items-center"
                        on:click=move |_| on_confirm.run(())
                        disabled=move || is_processing.get() || reject_reason.get().len() < 10
                    >
                        <Show when=move || is_processing.get()>
                            <span class="fa-spin mr-2">
                                <AppIcon icon=SPINNER />
                            </span>
                        </Show>
                        "Ya, Tolak"
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn BatchResultSummary(
    result: ReadSignal<Option<BatchOperationResult>>,
    on_close: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || {
            result.get().is_some()
        }>
            {move || {
                let r = result.get().unwrap();
                let is_success = r.failed_items == 0;
                let bg_color = if is_success { "bg-success-500/10" } else { "bg-warning-500/10" };
                let border_color = if is_success {
                    "border-success-500/25"
                } else {
                    "border-warning-500/25"
                };
                let icon_color = if is_success { "text-success-400" } else { "text-warning-400" };
                let icon = if is_success { CHECK_CIRCLE } else { WARNING };

                view! {
                    <div class=format!("mb-6 p-4 {} border {} rounded-lg", bg_color, border_color)>
                        <div class="flex items-start justify-between">
                            <div class="flex items-start gap-3">
                                <span class=format!("mt-1 inline-flex {}", icon_color)>
                                    <AppIcon icon=icon size=20 />
                                </span>
                                <div>
                                    <h4 class="font-semibold text-slate-100 mb-1">
                                        {if is_success {
                                            "Operasi Berhasil"
                                        } else {
                                            "Operasi Selesai dengan Peringatan"
                                        }}
                                    </h4>
                                    <p class="text-sm text-slate-200 mb-2">
                                        "Batch ID: "
                                        <code class="rounded bg-white/[0.06] px-2 py-1 text-xs text-slate-200">
                                            {r.batch_id.to_string()}
                                        </code>
                                    </p>
                                    <div class="flex gap-4 text-sm">
                                        <div>
                                            <span class="text-slate-400">"Total: "</span>
                                            <span class="font-semibold">{r.total_items}</span>
                                        </div>
                                        <div>
                                            <span class="text-slate-400">"Berhasil: "</span>
                                            <span class="font-semibold text-success-400">
                                                {r.successful_items}
                                            </span>
                                        </div>
                                        <Show when=move || r.failed_items != 0>
                                            <div>
                                                <span class="text-slate-400">"Gagal: "</span>
                                                <span class="font-semibold text-danger-400">
                                                    {r.failed_items}
                                                </span>
                                            </div>
                                        </Show>
                                    </div>
                                </div>
                            </div>
                            <button
                                class="text-slate-500 hover:text-slate-300"
                                on:click=move |_| on_close.run(())
                            >
                                <AppIcon icon=X />
                            </button>
                        </div>
                    </div>
                }
            }}
        </Show>
    }
}
