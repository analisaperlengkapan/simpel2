use leptos::prelude::*;
use lib_ui::components::icon::{AppIcon, icon_from_fa_class};

/// Empty state block with a big icon, headline, helper message, and an
/// optional call-to-action. Every page that renders a list must mount this
/// when the list is empty — contract tests forbid `list.is_empty() { <></> }`
/// patterns that used to leave pages looking broken.
#[component]
pub fn EmptyState(
    #[prop(into)] title: String,
    #[prop(optional, into)] description: Option<String>,
    #[prop(default = "fas fa-inbox".to_string(), into)] icon: String,
    #[prop(optional)] action: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center rounded-2xl border border-dashed border-white/10 bg-white/[0.02] px-6 py-14 text-center">
            <span class="flex h-14 w-14 items-center justify-center rounded-full bg-gold-500/10 text-gold-400 ring-1 ring-gold-500/20">
                <AppIcon icon=icon_from_fa_class(&icon) size=20 />
            </span>
            <h3 class="mt-4 text-base font-semibold text-white">{title}</h3>
            {description.map(|d| view! {
                <p class="mt-2 max-w-md text-sm leading-relaxed text-slate-400">{d}</p>
            })}
            {action.map(|a| view! {
                <div class="mt-5 flex flex-wrap items-center justify-center gap-2">{a()}</div>
            })}
        </div>
    }
}
