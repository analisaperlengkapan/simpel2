//! Sidebar tree navigation for authenticated portal pages.

use crate::components::navigation::menu::{PortalMenuItem, resolve_menu_sections};
use crate::features::auth::UserSession;
use leptos::prelude::*;

#[component]
pub fn SidebarNavigation(
    #[prop(optional)]
    user_session: Option<UserSession>,
) -> impl IntoView {
    let sections = resolve_menu_sections(user_session.as_ref());

    view! {
        <aside class="hidden lg:block w-72 border-r border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900/50" aria-label="Menu portal">
            <div class="sticky top-16 h-[calc(100vh-4rem)] overflow-y-auto px-4 py-5 space-y-5">
                {sections.clone().into_iter().map(|section| {
                    let items = section.items.clone();
                    view! {
                        <section>
                            <h2 class="px-2 text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
                                {section.title}
                            </h2>
                            <ul class="mt-2 space-y-1">
                                {items.into_iter().map(|item| {
                                    view! {
                                        <li>
                                            <MenuItemNode item=item depth=0 />
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        </section>
                    }
                }).collect_view()}
            </div>
        </aside>
    }
}

#[component]
fn MenuItemNode(item: PortalMenuItem, depth: usize) -> impl IntoView {
    let has_children = !item.children.is_empty();
    let children = item.children.clone();
    let depth_class = if depth == 0 { "pl-3" } else { "pl-6" };
    let class = if current_path().starts_with(item.href) {
        format!(
            "group flex items-center rounded-lg pr-3 py-2 text-sm font-medium transition-colors {} bg-navy-50 text-navy-700 dark:bg-navy-900/30 dark:text-gold-300",
            depth_class
        )
    } else {
        format!(
            "group flex items-center rounded-lg pr-3 py-2 text-sm transition-colors {} text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-800/70",
            depth_class
        )
    };

    view! {
        <div class="space-y-1">
            <a href=item.href class=class>
                <span>{item.label}</span>
            </a>

            {if has_children {
                view! {
                    <ul class="space-y-1">
                        {children.into_iter().map(|child| {
                            view! {
                                <li>
                                    <MenuItemNode item=child depth=depth + 1 />
                                </li>
                            }
                        }).collect_view()}
                    </ul>
                }.into_any()
            } else {
                view! { <></> }.into_any()
            }}
        </div>
    }
}

fn current_path() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.location().pathname().ok())
            .unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        String::new()
    }
}
