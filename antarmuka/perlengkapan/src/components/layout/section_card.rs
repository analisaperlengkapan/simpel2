use leptos::prelude::*;
use lib_ui::components::icon::{AppIcon, icon_from_fa_class};

/// A panel containing one logical group of content inside a page. Owns its
/// own header row so callers can declare a title/description/actions tuple
/// without re-declaring the surface treatment for each usage.
#[component]
pub fn SectionCard(
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] description: Option<String>,
    #[prop(optional, into)] icon: Option<String>,
    #[prop(optional)] actions: Option<Children>,
    #[prop(default = false)] dense: bool,
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let padding = if dense { "p-4" } else { "p-6" };
    let extra = class.unwrap_or_default();
    let has_header = title.is_some() || description.is_some() || actions.is_some();

    view! {
        <section class=format!(
            "rounded-2xl border border-white/[0.06] bg-surface-panel shadow-card {} {}",
            padding, extra
        )>
            {has_header.then(|| view! {
                <header class="mb-4 flex flex-col gap-2 border-b border-white/[0.04] pb-4 sm:flex-row sm:items-start sm:justify-between">
                    <div class="flex items-start gap-3">
                        {icon.map(|i| view! {
                            <span class="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-gold-500/10 text-gold-400 ring-1 ring-gold-500/20">
                                <AppIcon icon=icon_from_fa_class(&i) size=14 />
                            </span>
                        })}
                        <div>
                            {title.map(|t| view! {
                                <h2 class="text-base font-semibold tracking-tight text-white">{t}</h2>
                            })}
                            {description.map(|d| view! {
                                <p class="mt-1 max-w-2xl text-sm leading-relaxed text-slate-400">{d}</p>
                            })}
                        </div>
                    </div>
                    {actions.map(|a| view! {
                        <div class="flex flex-wrap items-center gap-2">{a()}</div>
                    })}
                </header>
            })}
            <div>
                {children()}
            </div>
        </section>
    }
}
