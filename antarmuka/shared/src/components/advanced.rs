//! Advanced interactive components: Tooltip, Popover, VirtualScroll, InfiniteScroll, ContextMenu

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Element;

// ============================================================================
// TOOLTIP COMPONENT
// ============================================================================

#[component]
pub fn Tooltip(
    #[prop(into)] content: String,
    #[prop(default = "top".to_string(), into)] position: String,
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let (show, set_show) = signal(false);

    let position_class = match position.as_str() {
        "top" => "bottom-full left-1/2 -translate-x-1/2 mb-2",
        "bottom" => "top-full left-1/2 -translate-x-1/2 mt-2",
        "left" => "right-full top-1/2 -translate-y-1/2 mr-2",
        "right" => "left-full top-1/2 -translate-y-1/2 ml-2",
        _ => "bottom-full left-1/2 -translate-x-1/2 mb-2",
    };

    view! {
        <div
            class=format!("relative inline-block {}", class)
            on:mouseenter=move |_| set_show.set(true)
            on:mouseleave=move |_| set_show.set(false)
            on:focus=move |_| set_show.set(true)
            on:blur=move |_| set_show.set(false)
        >
            {children()}

            <div
                class=format!(
                    "absolute {} z-50 px-3 py-2 text-sm font-medium text-white bg-gray-900 rounded-lg shadow-sm whitespace-nowrap transition-opacity duration-200 pointer-events-none {}",
                    position_class,
                    if show.get() { "opacity-100" } else { "opacity-0" }
                )
                role="tooltip"
            >
                {content}
                <div class="tooltip-arrow"></div>
            </div>
        </div>
    }
}

// ============================================================================
// POPOVER COMPONENT
// ============================================================================

#[component]
pub fn Popover(
    #[prop(into)] content: String,
    #[prop(default = "bottom".to_string(), into)] position: String,
    #[prop(default = false)] show: bool,
    #[prop(optional)] on_close: Option<Box<dyn Fn()>>,
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let class = class.unwrap_or_default();

    let position_class = match position.as_str() {
        "top" => "bottom-full left-1/2 -translate-x-1/2 mb-2",
        "bottom" => "top-full left-1/2 -translate-x-1/2 mt-2",
        "left" => "right-full top-1/2 -translate-y-1/2 mr-2",
        "right" => "left-full top-1/2 -translate-y-1/2 ml-2",
        _ => "top-full left-1/2 -translate-x-1/2 mt-2",
    };

    let handle_close = move |_| {
        if let Some(ref callback) = on_close {
            callback();
        }
    };

    view! {
        <div class=format!("relative inline-block {}", class)>
            {children()}

            {show.then(|| view! {
                <>
                    <div
                        class="fixed inset-0 z-40"
                        on:click=handle_close
                    ></div>

                    <div
                        class=format!(
                            "absolute {} z-50 w-64 p-4 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700",
                            position_class
                        )
                        role="dialog"
                    >
                        <div class="text-sm text-gray-700 dark:text-gray-300">
                            {content}
                        </div>
                    </div>
                </>
            })}
        </div>
    }
}

// ============================================================================
// DROPDOWN COMPONENT
// ============================================================================

#[derive(Clone, Debug)]
pub struct DropdownItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub disabled: bool,
}

#[component]
pub fn Dropdown(
    #[prop(into)] items: Vec<DropdownItem>,
    #[prop(default = false)] show: bool,
    #[prop(optional)] on_select: Option<Box<dyn Fn(String)>>,
    #[prop(optional)] on_close: Option<Box<dyn Fn()>>,
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    use std::rc::Rc;
    let class = class.unwrap_or_default();
    let on_select_rc = Rc::new(on_select);
    let on_close_rc = Rc::new(on_close);

    let handle_close = {
        let on_close = Rc::clone(&on_close_rc);
        move |_| {
            if let Some(ref callback) = *on_close {
                callback();
            }
        }
    };

    view! {
        <div class=format!("relative inline-block {}", class)>
            {children()}

            {show.then(|| view! {
                <>
                    <div
                        class="fixed inset-0 z-40"
                        on:click=handle_close
                    ></div>

                    <div
                        class="absolute right-0 mt-2 w-56 rounded-md shadow-lg bg-white dark:bg-gray-800 ring-1 ring-black ring-opacity-5 z-50"
                        role="menu"
                    >
                        <div class="py-1">
                            {items.into_iter().map(|item| {
                                let on_select = Rc::clone(&on_select_rc);
                                let on_close = Rc::clone(&on_close_rc);
                                let item_id = item.id.clone();

                                view! {
                                    <button
                                        type="button"
                                        disabled=item.disabled
                                        class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
                                        role="menuitem"
                                        on:click=move |_| {
                                            if !item.disabled {
                                                if let Some(ref callback) = *on_select {
                                                    callback(item_id.clone());
                                                }
                                                if let Some(ref callback) = *on_close {
                                                    callback();
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
                        </div>
                    </div>
                </>
            })}
        </div>
    }
}

// ============================================================================
// VIRTUAL SCROLL COMPONENT
// ============================================================================

/// VirtualScroll component for rendering large lists efficiently
/// Only renders visible items plus a buffer, dramatically improving performance
#[component]
pub fn VirtualScroll<T>(
    /// The complete list of items
    #[prop(into)]
    items: Signal<Vec<T>>,
    /// Height of each item in pixels
    #[prop(default = 50.0)]
    item_height: f64,
    /// Height of the viewport in pixels
    #[prop(default = 400.0)]
    viewport_height: f64,
    /// Number of items to render outside visible area (buffer)
    #[prop(default = 5)]
    overscan: usize,
    /// Function to render each item (item, index) -> View
    #[prop(into)]
    render_item: Callback<(T, usize)>,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
{
    let class = class.unwrap_or_default();
    let (scroll_top, set_scroll_top) = signal(0.0);

    // Calculate visible range with overscan
    let visible_range = move || {
        let items_vec = items.get();
        let total_items = items_vec.len();

        if total_items == 0 {
            return 0..0;
        }

        let start = ((scroll_top.get() / item_height).floor() as usize).saturating_sub(overscan);
        let visible_count = (viewport_height / item_height).ceil() as usize;
        let end = (start + visible_count + overscan * 2).min(total_items);

        start..end
    };

    // Total height of all items
    let total_height = move || items.get().len() as f64 * item_height;

    // Handle scroll event
    let handle_scroll = move |ev: web_sys::Event| {
        if let Some(target) = ev.target() {
            if let Ok(element) = target.dyn_into::<Element>() {
                set_scroll_top.set(element.scroll_top() as f64);
            }
        }
    };

    view! {
        <div
            class=format!("overflow-y-auto {}", class)
            style=format!("height: {}px", viewport_height)
            on:scroll=handle_scroll
        >
            <div
                class="relative"
                style=move || format!("height: {}px", total_height())
            >
                {move || {
                    let range = visible_range();
                    let items_vec = items.get();

                    items_vec[range.clone()]
                        .iter()
                        .enumerate()
                        .map(|(idx, item)| {
                            let actual_idx = range.start + idx;
                            let top = actual_idx as f64 * item_height;

                            view! {
                                <div
                                    class="absolute w-full"
                                    style=format!("top: {}px; height: {}px", top, item_height)
                                >
                                    {render_item.run((item.clone(), actual_idx))}
                                </div>
                            }
                        })
                        .collect_view()
                }}
            </div>
        </div>
    }
}

// ============================================================================
// INFINITE SCROLL COMPONENT
// ============================================================================

/// InfiniteScroll component for lazy loading data as user scrolls
/// Automatically loads more data when user reaches the bottom
#[component]
pub fn InfiniteScroll<T>(
    /// Current list of items
    #[prop(into)]
    items: Signal<Vec<T>>,
    /// Whether more data is being loaded
    #[prop(into)]
    loading: Signal<bool>,
    /// Whether there are more items to load
    #[prop(into)]
    has_more: Signal<bool>,
    /// Callback to load more items
    #[prop(into)]
    on_load_more: Callback<()>,
    /// Distance from bottom (in pixels) to trigger load
    #[prop(default = 200.0)]
    threshold: f64,
    /// Function to render each item (item, index) -> View
    #[prop(into)]
    render_item: Callback<(T, usize)>,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
{
    let class = class.unwrap_or_default();

    // Handle scroll event
    let handle_scroll = move |ev: web_sys::Event| {
        if loading.get() || !has_more.get() {
            return;
        }

        if let Some(target) = ev.target() {
            if let Ok(element) = target.dyn_into::<Element>() {
                let scroll_top = element.scroll_top() as f64;
                let scroll_height = element.scroll_height() as f64;
                let client_height = element.client_height() as f64;

                let distance_to_bottom = scroll_height - (scroll_top + client_height);

                if distance_to_bottom < threshold {
                    on_load_more.run(());
                }
            }
        }
    };

    view! {
        <div
            class=format!("overflow-y-auto {}", class)
            on:scroll=handle_scroll
        >
            <div class="space-y-2">
                {move || {
                    let items_vec = items.get();
                    items_vec
                        .iter()
                        .enumerate()
                        .map(|(idx, item)| {
                            render_item.run((item.clone(), idx))
                        })
                        .collect_view()
                }}
            </div>

            {move || loading.get().then(|| view! {
                <div class="flex justify-center items-center py-4">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
                    <span class="ml-3 text-gray-600 dark:text-gray-400">"Loading more..."</span>
                </div>
            })}

            {move || {
                let items_vec = items.get();
                (!has_more.get() && !loading.get() && !items_vec.is_empty()).then(|| view! {
                    <div class="text-center py-4 text-gray-500 dark:text-gray-400">
                        "No more items to load"
                    </div>
                })
            }}
        </div>
    }
}

// ============================================================================
// CONTEXT MENU COMPONENT
// ============================================================================

/// MenuItem for ContextMenu
#[derive(Clone, Debug)]
pub struct ContextMenuItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub disabled: bool,
    pub divider: bool,
}

impl ContextMenuItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            disabled: false,
            divider: false,
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    pub fn divider() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            icon: None,
            disabled: false,
            divider: true,
        }
    }
}

/// ContextMenu component for right-click menus
/// Shows a menu at the cursor position when right-clicking
#[component]
pub fn ContextMenu(
    /// Menu items to display
    #[prop(into)]
    items: Vec<ContextMenuItem>,
    /// Callback when an item is selected
    #[prop(into)]
    on_select: Callback<String>,
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let (show, set_show) = signal(false);
    let (position, set_position) = signal((0, 0));

    // Handle right-click
    let handle_context_menu = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        set_position.set((ev.client_x(), ev.client_y()));
        set_show.set(true);
    };

    // Handle close
    let handle_close = move |_| {
        set_show.set(false);
    };

    view! {
        <div
            class=format!("relative {}", class)
            on:contextmenu=handle_context_menu
        >
            {children()}

            {move || show.get().then(|| {
                let (x, y) = position.get();
                view! {
                    <>
                        <div
                            class="fixed inset-0 z-40"
                            on:click=handle_close
                            on:contextmenu=move |ev: web_sys::MouseEvent| {
                                ev.prevent_default();
                                set_show.set(false);
                            }
                        ></div>

                        <div
                            class="fixed z-50 min-w-[200px] rounded-md shadow-lg bg-white dark:bg-gray-800 ring-1 ring-black ring-opacity-5"
                            style=format!("left: {}px; top: {}px", x, y)
                            role="menu"
                        >
                            <div class="py-1">
                                {items.iter().map(|item| {
                                    if item.divider {
                                        view! {
                                            <div class="border-t border-gray-200 dark:border-gray-700 my-1"></div>
                                        }.into_any()
                                    } else {
                                        let item_id = item.id.clone();
                                        let item_label = item.label.clone();
                                        let item_icon = item.icon.clone();
                                        let item_disabled = item.disabled;

                                        view! {
                                            <button
                                                type="button"
                                                disabled=item_disabled
                                                class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
                                                role="menuitem"
                                                on:click=move |_| {
                                                    if !item_disabled {
                                                        on_select.run(item_id.clone());
                                                        set_show.set(false);
                                                    }
                                                }
                                            >
                                                {item_icon.map(|icon| view! {
                                                    <span class="mr-2">{icon}</span>
                                                })}
                                                {item_label.clone()}
                                            </button>
                                        }.into_any()
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    </>
                }
            })}
        </div>
    }
}
