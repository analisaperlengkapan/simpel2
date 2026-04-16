use leptos::prelude::*;
use leptos_router::components::A;

#[derive(Clone, Debug)]
pub struct PageBreadcrumb {
    pub label: String,
    pub href: Option<String>,
}

impl PageBreadcrumb {
    pub fn new(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: Some(href.into()),
        }
    }

    pub fn leaf(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: None,
        }
    }
}

/// Top-level wrapper every page mounts inside. Provides the navy surface,
/// the standard heading row (title + description + actions), optional
/// breadcrumbs, and a scrollable content region that pages fill with
/// `SectionCard`s.
#[component]
pub fn PageLayout(
    #[prop(into)] title: String,
    #[prop(optional, into)] description: Option<String>,
    #[prop(optional, into)] icon: Option<String>,
    #[prop(default = vec![])] breadcrumbs: Vec<PageBreadcrumb>,
    #[prop(optional)] actions: Option<Children>,
    children: Children,
) -> impl IntoView {
    let has_breadcrumbs = !breadcrumbs.is_empty();
    let breadcrumb_items = breadcrumbs
        .into_iter()
        .map(|crumb| {
            let label = crumb.label.clone();
            match crumb.href {
                Some(href) => view! {
                    <li class="flex items-center gap-2">
                        <A
                            href=href
                            attr:class="text-xs font-medium text-slate-400 transition hover:text-gold-300"
                        >
                            {label.clone()}
                        </A>
                        <i class="fas fa-chevron-right text-[0.55rem] text-slate-600"></i>
                    </li>
                }
                .into_any(),
                None => view! {
                    <li class="text-xs font-semibold text-slate-200">{label.clone()}</li>
                }
                .into_any(),
            }
        })
        .collect_view();

    view! {
        <section class="mx-auto flex w-full max-w-[1440px] flex-col gap-6">
            {has_breadcrumbs.then(|| view! {
                <nav aria-label="breadcrumb">
                    <ol class="flex flex-wrap items-center gap-2">
                        {breadcrumb_items}
                    </ol>
                </nav>
            })}

            <header class="flex flex-col gap-4 border-b border-white/5 pb-6 lg:flex-row lg:items-start lg:justify-between">
                <div class="flex items-start gap-4">
                    {icon.map(|i| view! {
                        <span class="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl bg-gold-500/10 text-gold-400 ring-1 ring-gold-500/20">
                            <i class=i></i>
                        </span>
                    })}
                    <div>
                        <h1 class="text-2xl font-bold tracking-tight text-white">{title}</h1>
                        {description.map(|d| view! {
                            <p class="mt-1 max-w-3xl text-sm leading-relaxed text-slate-400">{d}</p>
                        })}
                    </div>
                </div>
                {actions.map(|a| view! {
                    <div class="flex flex-wrap items-center gap-2">
                        {a()}
                    </div>
                })}
            </header>

            <div class="flex flex-col gap-6">
                {children()}
            </div>
        </section>
    }
}
