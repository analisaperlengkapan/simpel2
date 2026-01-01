//! Navigation components: AppHeader, Breadcrumb, Menu

use crate::core::{constants::*, types::*};
use leptos::prelude::*;
use leptos_router::components::A;

// ============================================================================
// APP HEADER COMPONENT
// ============================================================================

#[component]
pub fn AppHeader(
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] subtitle: Option<String>,
    #[prop(default = true)] show_logo: bool,
    children: Children,
) -> impl IntoView {
    let title = title.unwrap_or_else(|| APP_NAME.to_string());

    view! {
        <header class="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700">
            <div class="container mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex items-center justify-between h-16">
                    // Left section: Logo and Title
                    <div class="flex items-center space-x-4">
                        {show_logo.then(|| view! {
                            <Logo size="sm" />
                        })}
                        <div>
                            <h1 class="text-xl font-bold text-gray-900 dark:text-gray-100">
                                {title}
                            </h1>
                            {subtitle.map(|s| view! {
                                <p class="text-sm text-gray-600 dark:text-gray-400">{s}</p>
                            })}
                        </div>
                    </div>

                    // Right section: Actions (slot)
                    <div class="flex items-center space-x-4">
                        {children()}
                    </div>
                </div>
            </div>
        </header>
    }
}

// ============================================================================
// BREADCRUMB COMPONENT
// ============================================================================

#[component]
pub fn Breadcrumb(#[prop(into)] items: Vec<BreadcrumbItem>) -> impl IntoView {
    let total = items.len();
    view! {
        <nav class="flex" aria-label="Breadcrumb">
            <ol class="inline-flex items-center space-x-1 md:space-x-3">
                {items.into_iter().enumerate().map(|(index, item)| {
                    let is_last = index == total - 1;
                    view! {
                        <li class="inline-flex items-center">
                            {(!index == 0).then(|| view! {
                                <svg class="w-3 h-3 text-gray-400 mx-1" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd"/>
                                </svg>
                            })}

                            {if is_last {
                                view! {
                                    <span class="text-sm font-medium text-gray-500 dark:text-gray-400">
                                        {item.label}
                                    </span>
                                }.into_any()
                            } else {
                                view! {
                                    <A
                                        href=item.path.unwrap_or_else(|| "#".to_string())
                                        attr:class="inline-flex items-center text-sm font-medium text-gray-700 hover:text-emerald-600 dark:text-gray-300"
                                    >
                                        {item.icon.map(|icon| view! {
                                            <span class="mr-2">{icon}</span>
                                        })}
                                        {item.label}
                                    </A>
                                }.into_any()
                            }}
                        </li>
                    }
                }).collect_view()}
            </ol>
        </nav>
    }
}

// ============================================================================
// NAVIGATION MENU COMPONENT
// ============================================================================

#[component]
pub fn NavMenu(
    #[prop(into)] items: Vec<NavItem>,
    #[prop(default = false)] vertical: bool,
) -> impl IntoView {
    let menu_class = if vertical {
        "flex flex-col space-y-1"
    } else {
        "flex flex-row space-x-1"
    };

    view! {
        <nav>
            <ul class=menu_class>
                {items.into_iter().map(|item| view! {
                    <NavMenuItem item=item vertical=vertical />
                }).collect_view()}
            </ul>
        </nav>
    }
}

#[component]
pub fn NavMenuItem(item: NavItem, #[prop(default = false)] vertical: bool) -> impl IntoView {
    let active_class = if item.active {
        "bg-emerald-100 dark:bg-emerald-900 text-emerald-700 dark:text-emerald-300"
    } else {
        "text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
    };

    view! {
        <li>
            <A
                href=item.path
                attr:class=format!(
                    "flex items-center px-3 py-2 rounded-md text-sm font-medium transition-colors {}",
                    active_class
                )
            >
                {item.icon.map(|icon| view! {
                    <span class="mr-2">{icon}</span>
                })}
                <span>{item.label.clone()}</span>
                {item.badge.map(|badge| view! {
                    <span class="ml-2 px-2 py-0.5 text-xs font-semibold rounded-full bg-emerald-600 text-white">
                        {badge}
                    </span>
                })}
            </A>

            {(!item.children.is_empty()).then(|| {
                let items: Vec<_> = item.children.into_iter().map(|child| {
                    view! {
                        <NavMenuItem item=child vertical=vertical />
                    }.into_any()
                }).collect();
                view! {
                    <ul class="ml-4 mt-1 space-y-1">
                        {items}
                    </ul>
                }.into_any()
            })}
        </li>
    }
}

// ============================================================================
// LOGO COMPONENT
// ============================================================================

#[component]
pub fn Logo(
    #[prop(default = "md".to_string(), into)] size: String,
    #[prop(default = true)] show_text: bool,
) -> impl IntoView {
    let (img_size, text_size) = match size.as_str() {
        "sm" => ("h-8 w-8", "text-sm"),
        "lg" => ("h-16 w-16", "text-xl"),
        _ => ("h-12 w-12", "text-lg"),
    };

    view! {
        <div class="flex items-center space-x-2">
            <div class=format!("{} bg-emerald-700 rounded-lg flex items-center justify-center", img_size)>
                <span class="text-white font-bold text-2xl">"K"</span>
            </div>
            {show_text.then(|| view! {
                <div class="flex flex-col">
                    <span class=format!("font-bold text-gray-900 dark:text-gray-100 {}", text_size)>
                        {APP_NAME}
                    </span>
                    <span class="text-xs text-gray-600 dark:text-gray-400">
                        {ORG_SHORT}
                    </span>
                </div>
            })}
        </div>
    }
}

// ============================================================================
// SIDEBAR COMPONENT
// ============================================================================

#[component]
pub fn Sidebar(
    #[prop(into)] items: Vec<NavItem>,
    #[prop(default = false)] collapsed: bool,
    #[prop(optional)] on_toggle: Option<Box<dyn Fn()>>,
) -> impl IntoView {
    let width_class = if collapsed { "w-16" } else { "w-64" };

    let handle_toggle = move |_| {
        if let Some(ref callback) = on_toggle {
            callback();
        }
    };

    view! {
        <aside class=format!(
            "flex flex-col bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 transition-all duration-300 {}",
            width_class
        )>
            // Header
            <div class="h-16 flex items-center justify-between px-4 border-b border-gray-200 dark:border-gray-700">
                {(!collapsed).then(|| view! {
                    <Logo size="sm" />
                })}
                <button
                    type="button"
                    class="p-2 rounded-md hover:bg-gray-100 dark:hover:bg-gray-700"
                    on:click=handle_toggle
                >
                    <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>
                    </svg>
                </button>
            </div>

            // Navigation
            <nav class="flex-1 overflow-y-auto p-4">
                <NavMenu items=items vertical=true />
            </nav>
        </aside>
    }
}

// ============================================================================
// TABS COMPONENT
// ============================================================================

#[derive(Clone, Debug)]
pub struct TabItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub disabled: bool,
}

#[component]
pub fn Tabs(
    #[prop(into)] items: Vec<TabItem>,
    #[prop(into)] active_tab: String,
    #[prop(optional)] on_change: Option<Box<dyn Fn(String)>>,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    use std::rc::Rc;
    let class = class.unwrap_or_default();
    let on_change_rc = Rc::new(on_change);

    view! {
        <div class=format!("border-b border-gray-200 dark:border-gray-700 {}", class)>
            <nav class="-mb-px flex space-x-8" aria-label="Tabs" role="tablist">
                {items.into_iter().map(|item| {
                    let is_active = item.id == active_tab;
                    let on_change = Rc::clone(&on_change_rc);
                    let item_id = item.id.clone();

                    view! {
                        <button
                            type="button"
                            role="tab"
                            aria-selected=is_active
                            aria-controls=format!("tabpanel-{}", item.id)
                            id=format!("tab-{}", item.id)
                            disabled=item.disabled
                            class=format!(
                                "whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 disabled:opacity-50 disabled:cursor-not-allowed {}",
                                if is_active {
                                    "border-emerald-600 text-emerald-600"
                                } else {
                                    "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-300"
                                }
                            )
                            on:click=move |_| {
                                if !item.disabled {
                                    if let Some(ref callback) = *on_change {
                                        callback(item_id.clone());
                                    }
                                }
                            }
                        >
                            {item.icon.map(|icon| view! {
                                <span class="mr-2">{icon}</span>
                            })}
                            {item.label}
                        </button>
                    }
                }).collect_view()}
            </nav>
        </div>
    }
}

// ============================================================================
// TAB PANEL COMPONENT
// ============================================================================

#[component]
pub fn TabPanel(
    #[prop(into)] id: String,
    #[prop(into)] active_tab: String,
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let is_active = id == active_tab;

    view! {
        <div
            id=format!("tabpanel-{}", id)
            role="tabpanel"
            aria-labelledby=format!("tab-{}", id)
            class=format!("py-4 {} {}", if is_active { "block" } else { "hidden" }, class)
            tabindex="0"
        >
            {children()}
        </div>
    }
}

// ============================================================================
// MOBILE MENU BUTTON
// ============================================================================

#[component]
pub fn MobileMenuButton(
    #[prop(default = false)] open: bool,
    #[prop(optional)] on_toggle: Option<Box<dyn Fn()>>,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();

    let handle_click = move |_| {
        if let Some(ref callback) = on_toggle {
            callback();
        }
    };

    view! {
        <button
            type="button"
            class=format!(
                "inline-flex items-center justify-center p-2 rounded-md text-gray-400 hover:text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-primary-500 {}",
                class
            )
            aria-expanded=open
            aria-label=if open { "Close menu" } else { "Open menu" }
            on:click=handle_click
        >
            {if open {
                view! {
                    <svg class="h-6 w-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                    </svg>
                }.into_any()
            } else {
                view! {
                    <svg class="h-6 w-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>
                    </svg>
                }.into_any()
            }}
        </button>
    }
}

// ============================================================================
// BACK BUTTON
// ============================================================================

#[component]
pub fn BackButton(
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] href: Option<String>,
    #[prop(optional)] on_click: Option<Box<dyn Fn()>>,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let label = label.unwrap_or_else(|| "Back".to_string());

    let handle_click = move |_| {
        if let Some(ref callback) = on_click {
            callback();
        }
    };

    if let Some(path) = href {
        view! {
            <A
                href=path
                attr:class=format!(
                    "inline-flex items-center text-sm font-medium text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300 {}",
                    class
                )
            >
                <svg class="mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/>
                </svg>
                {label}
            </A>
        }.into_any()
    } else {
        view! {
            <button
                type="button"
                class=format!(
                    "inline-flex items-center text-sm font-medium text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 rounded {}",
                    class
                )
                on:click=handle_click
            >
                <svg class="mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/>
                </svg>
                {label}
            </button>
        }.into_any()
    }
}
