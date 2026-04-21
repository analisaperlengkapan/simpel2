//! JSON preview of the current workflow definition — handy for admins
//! verifying how the backend will serialize the config before they save.

use leptos::prelude::*;

use crate::api::workflow::WorkflowDefinitionDetail;

#[component]
pub fn ConfigJsonPreview(detail: WorkflowDefinitionDetail) -> impl IntoView {
    let pretty = serde_json::to_string_pretty(&detail)
        .unwrap_or_else(|e| format!("<gagal serialisasi: {}>", e));

    view! {
        <div class="flex flex-col gap-3">
            <p class="text-xs text-slate-400">
                "Representasi JSON dari workflow — gunakan untuk verifikasi atau backup konfigurasi."
            </p>
            <pre class="max-h-96 overflow-auto rounded-xl border border-white/[0.06] bg-navy-950/70 p-4 text-[0.75rem] leading-relaxed text-slate-200">
                <code>{pretty}</code>
            </pre>
        </div>
    }
}
