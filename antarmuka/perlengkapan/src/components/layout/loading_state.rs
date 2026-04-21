use leptos::prelude::*;

/// Loading placeholder used while an async resource is pending. Always
/// includes a message so the UI never looks frozen — unlike the old
/// patterns that rendered an empty `<div></div>` during `.flatten()` chains.
#[component]
pub fn LoadingState(
    #[prop(optional, into)] message: Option<String>,
    #[prop(default = false)] inline: bool,
) -> impl IntoView {
    let label = message.unwrap_or_else(|| "Memuat data…".to_string());
    let container_class = if inline {
        "flex items-center gap-3 rounded-xl border border-white/[0.06] bg-white/[0.03] px-4 py-3 text-sm text-slate-300"
    } else {
        "flex items-center justify-center gap-3 rounded-2xl border border-white/[0.06] bg-white/[0.02] px-6 py-10 text-sm text-slate-300"
    };

    view! {
        <div class=container_class role="status" aria-live="polite">
            <span class="relative flex h-5 w-5 items-center justify-center">
                <span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-gold-500/40"></span>
                <span class="relative inline-flex h-3 w-3 rounded-full bg-gold-400"></span>
            </span>
            <span>{label}</span>
        </div>
    }
}
