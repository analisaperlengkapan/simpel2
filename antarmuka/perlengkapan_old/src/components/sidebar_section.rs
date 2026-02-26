#![allow(dead_code)]
use leptos::prelude::*;
use lib_ui::utils::security::sanitize_html;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct MenuItem {
    pub href: String,
    pub label: String,
}

impl MenuItem {
    #[allow(dead_code)]
    pub fn new(href: &str, label: &str) -> Self {
        Self {
            href: href.to_string(),
            label: label.to_string(),
        }
    }
}

#[allow(dead_code)]
#[component]
#[allow(dead_code)]
pub fn SidebarSection(
    #[allow(unused)] title: String,
    #[allow(unused)] icon: String,
    #[allow(unused)] is_expanded: RwSignal<bool>,
    #[allow(unused)] items: Vec<MenuItem>,
    /// Roles allowed to see this section. Empty = visible to all.
    #[prop(optional)]
    #[allow(unused)]
    allowed_roles: Option<Vec<String>>,
    /// The current user's active role key.
    #[prop(optional)]
    #[allow(unused)]
    active_role: Option<String>,
) -> impl IntoView {
    // Role-based visibility: if allowed_roles is set & non-empty, check membership
    let visible = match (&allowed_roles, &active_role) {
        (Some(roles), Some(role)) if !roles.is_empty() => roles.contains(role),
        _ => true, // no restriction or no role info → show
    };

    if !visible {
        return view! { <div class="hidden"></div> }.into_any();
    }

    let toggle_expand = move |_| {
        is_expanded.update(|v| *v = !*v);
    };

    let chevron_class = move || {
        if is_expanded.get() {
            "transform rotate-90"
        } else {
            ""
        }
    };

    let items = StoredValue::new(items);

    // Sanitize icon HTML to prevent XSS
    let sanitized_icon = sanitize_html(&icon);

    view! {
        <div class="space-y-1">
            <button
                on:click=toggle_expand
                class="w-full text-gray-300 hover:text-white hover:bg-white/10 px-4 py-2.5 rounded-lg flex items-center justify-between transition-all duration-200 text-sm"
                aria-expanded=move || if is_expanded.get() { "true" } else { "false" }
            >
                <div class="flex items-center space-x-3">
                    <div class="w-5 h-5 flex items-center justify-center" inner_html=sanitized_icon></div>
                    <span>{title}</span>
                </div>
                <svg class=move || format!("w-4 h-4 transition-transform duration-200 {}", chevron_class())
                     fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                </svg>
            </button>

            <Show when=move || is_expanded.get()>
                <div class="ml-4 space-y-1">
                    {move || items.with_value(|items| {
                        items.iter().map(|item| {
                            let href = item.href.clone();
                            let label = item.label.clone();

                            view! {
                                <a
                                    href=href
                                    class="block w-full text-left text-gray-400 hover:text-white hover:bg-white/10 px-3 py-1.5 rounded text-sm transition-all duration-200"
                                >
                                    {label}
                                </a>
                            }
                        }).collect::<Vec<_>>()
                    })}
                </div>
            </Show>
        </div>
    }.into_any()
}
