//! Accessibility-focused components with WCAG 2.1 AA compliance

use crate::utils::accessibility::*;
use leptos::prelude::*;

// ============================================================================
// ACCESSIBLE HEADING COMPONENT
// ============================================================================

/// Accessible heading with proper hierarchy
#[component]
pub fn Heading(
    /// Heading level (1-6)
    #[prop(default = 2)]
    level: u8,
    /// Heading text
    #[prop(into)]
    children: ViewFn,
    /// Additional CSS classes
    #[prop(optional, into)]
    class: Option<String>,
    /// Optional ID for anchor links
    #[prop(optional, into)]
    id: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let level = level.clamp(1, 6);

    let heading_class = match level {
        1 => format!("text-4xl font-bold {}", class),
        2 => format!("text-3xl font-bold {}", class),
        3 => format!("text-2xl font-semibold {}", class),
        4 => format!("text-xl font-semibold {}", class),
        5 => format!("text-lg font-medium {}", class),
        6 => format!("text-base font-medium {}", class),
        _ => class,
    };

    match level {
        1 => view! { <h1 id=id class=heading_class>{children.run()}</h1> }.into_any(),
        2 => view! { <h2 id=id class=heading_class>{children.run()}</h2> }.into_any(),
        3 => view! { <h3 id=id class=heading_class>{children.run()}</h3> }.into_any(),
        4 => view! { <h4 id=id class=heading_class>{children.run()}</h4> }.into_any(),
        5 => view! { <h5 id=id class=heading_class>{children.run()}</h5> }.into_any(),
        6 => view! { <h6 id=id class=heading_class>{children.run()}</h6> }.into_any(),
        _ => view! { <h2 id=id class=heading_class>{children.run()}</h2> }.into_any(),
    }
}

// ============================================================================
// ACCESSIBLE LINK COMPONENT
// ============================================================================

/// Accessible link with proper ARIA attributes
#[component]
pub fn AccessibleLink(
    /// Link href
    #[prop(into)]
    href: String,
    /// Link text/content
    #[prop(into)]
    children: ViewFn,
    /// Additional CSS classes
    #[prop(optional, into)]
    class: Option<String>,
    /// ARIA label (if different from visible text)
    #[prop(optional, into)]
    aria_label: Option<String>,
    /// Whether link opens in new tab
    #[prop(default = false)]
    external: bool,
    /// Whether link is disabled
    #[prop(default = false)]
    disabled: bool,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let base_class = "underline hover:no-underline focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 rounded";

    view! {
        <a
            href=href
            class=format!("{} {}", base_class, class)
            aria-label=aria_label
            target=if external { Some("_blank") } else { None }
            rel=if external { Some("noopener noreferrer") } else { None }
            aria-disabled=if disabled { Some("true") } else { None }
            tabindex=if disabled { Some("-1") } else { None }
        >
            {children.run()}
            {external.then(|| view! {
                <VisuallyHidden>
                    " (opens in new tab)"
                </VisuallyHidden>
            })}
        </a>
    }
}

// ============================================================================
// ACCESSIBLE IMAGE COMPONENT
// ============================================================================

/// Accessible image with proper alt text
#[component]
pub fn AccessibleImage(
    /// Image source URL
    #[prop(into)]
    src: String,
    /// Alt text (required for accessibility)
    #[prop(into)]
    alt: String,
    /// Additional CSS classes
    #[prop(optional, into)]
    class: Option<String>,
    /// Whether image is decorative (alt will be empty)
    #[prop(default = false)]
    decorative: bool,
    /// Loading strategy
    #[prop(default = "lazy".to_string(), into)]
    loading: String,
) -> impl IntoView {
    use crate::components::optimized_image::OptimizedImage;

    let class = class.unwrap_or_default();
    let alt_text = if decorative { String::new() } else { alt };
    let lazy = loading == "lazy";

    view! {
        <div
            role=if decorative { Some("presentation") } else { None }
            aria-hidden=if decorative { Some("true") } else { None }
        >
            <OptimizedImage
                src=src
                alt=alt_text
                class=class
                lazy=lazy
            />
        </div>
    }
}

// ============================================================================
// ACCESSIBLE BUTTON GROUP COMPONENT
// ============================================================================

/// Accessible button group with proper ARIA attributes
#[component]
pub fn ButtonGroup(
    /// Group label for screen readers
    #[prop(into)]
    label: String,
    /// Additional CSS classes
    #[prop(optional, into)]
    class: Option<String>,
    /// Whether buttons are arranged horizontally
    #[prop(default = true)]
    horizontal: bool,
    #[prop(into)] children: ViewFn,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let direction_class = if horizontal {
        "flex flex-row space-x-2"
    } else {
        "flex flex-col space-y-2"
    };

    view! {
        <div
            role="group"
            aria-label=label
            class=format!("{} {}", direction_class, class)
        >
            {children.run()}
        </div>
    }
}

// ============================================================================
// ACCESSIBLE ALERT COMPONENT
// ============================================================================

/// Accessible alert with proper ARIA attributes
#[component]
pub fn AccessibleAlert(
    /// Alert message
    #[prop(into)]
    children: ViewFn,
    /// Alert type
    #[prop(default = AlertType::Info)]
    alert_type: AlertType,
    /// Additional CSS classes
    #[prop(optional, into)]
    class: Option<String>,
    /// Whether alert can be dismissed
    #[prop(default = false)]
    dismissible: bool,
    /// Callback when alert is dismissed
    #[prop(optional)]
    on_dismiss: Option<Callback<()>>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let (visible, set_visible) = signal(true);

    let (bg_class, icon) = match alert_type {
        AlertType::Success => ("bg-green-50 border-green-200 text-green-800", "✓"),
        AlertType::Info => ("bg-blue-50 border-blue-200 text-blue-800", "ℹ"),
        AlertType::Warning => ("bg-yellow-50 border-yellow-200 text-yellow-800", "⚠"),
        AlertType::Error => ("bg-red-50 border-red-200 text-red-800", "✕"),
    };

    let role = match alert_type {
        AlertType::Error | AlertType::Warning => "alert",
        _ => "status",
    };

    view! {
        <Show when=move || visible.get()>
            <div
                role=role
                aria-live="polite"
                aria-atomic="true"
                class=format!("p-4 border rounded-md {} {}", bg_class, class)
            >
                <div class="flex items-start">
                    <span class="flex-shrink-0 mr-3 text-xl" aria-hidden="true">
                        {icon}
                    </span>
                    <div class="flex-1">
                        {children.run()}
                    </div>
                    {dismissible.then(|| {
                        view! {
                            <button
                                type="button"
                                class="flex-shrink-0 ml-3 hover:opacity-75 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 rounded"
                                aria-label="Dismiss alert"
                                on:click=move |_| {
                                    set_visible.set(false);
                                    if let Some(callback) = on_dismiss {
                                        callback.run(());
                                    }
                                }
                            >
                                <span aria-hidden="true">"×"</span>
                            </button>
                        }
                    })}
                </div>
            </div>
        </Show>
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AlertType {
    Success,
    Info,
    Warning,
    Error,
}

// ============================================================================
// ACCESSIBLE PROGRESS BAR COMPONENT
// ============================================================================

/// Accessible progress bar with proper ARIA attributes
#[component]
pub fn AccessibleProgressBar(
    /// Current progress value (0-100)
    #[prop(into)]
    value: Signal<f64>,
    /// Label for screen readers
    #[prop(into)]
    label: String,
    /// Additional CSS classes
    #[prop(optional, into)]
    class: Option<String>,
    /// Whether to show percentage text
    #[prop(default = true)]
    show_percentage: bool,
) -> impl IntoView {
    let class = class.unwrap_or_default();

    view! {
        <div class=format!("space-y-2 {}", class)>
            <div class="flex justify-between items-center">
                <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                    {label.clone()}
                </span>
                {show_percentage.then(|| view! {
                    <span class="text-sm text-gray-600 dark:text-gray-400">
                        {move || format!("{}%", value.get().round())}
                    </span>
                })}
            </div>
            <div
                role="progressbar"
                aria-valuenow=move || value.get().to_string()
                aria-valuemin="0"
                aria-valuemax="100"
                aria-label=label
                class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2.5 overflow-hidden"
            >
                <div
                    class="bg-primary-600 h-2.5 rounded-full transition-all duration-300"
                    style=move || format!("width: {}%", value.get().clamp(0.0, 100.0))
                ></div>
            </div>
        </div>
    }
}

// ============================================================================
// ACCESSIBLE TABS COMPONENT
// ============================================================================

/// Accessible tabs with proper ARIA attributes and keyboard navigation
#[component]
pub fn AccessibleTabs(
    /// Tab items
    #[prop(into)]
    tabs: Vec<TabItem>,
    /// Currently active tab index
    #[prop(into)]
    active: Signal<usize>,
    /// Callback when tab changes
    on_change: WriteSignal<usize>,
    /// Additional CSS classes
    #[prop(optional, into)]
    class: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let tabs_len = tabs.len();

    let handle_keydown = move |ev: web_sys::KeyboardEvent, index: usize| match ev.key().as_str() {
        "ArrowLeft" => {
            ev.prevent_default();
            let new_index = if index == 0 { tabs_len - 1 } else { index - 1 };
            on_change.set(new_index);
        }
        "ArrowRight" => {
            ev.prevent_default();
            let new_index = if index == tabs_len - 1 { 0 } else { index + 1 };
            on_change.set(new_index);
        }
        "Home" => {
            ev.prevent_default();
            on_change.set(0);
        }
        "End" => {
            ev.prevent_default();
            on_change.set(tabs_len - 1);
        }
        _ => {}
    };

    view! {
        <div class=format!("space-y-4 {}", class)>
            <div role="tablist" aria-label="Tabs" class="flex border-b border-gray-200 dark:border-gray-700">
                {tabs.iter().enumerate().map(|(index, tab)| {
                    let is_active = move || active.get() == index;
                    let tab_id = format!("tab-{}", index);
                    let panel_id = format!("panel-{}", index);

                    view! {
                        <button
                            id=tab_id.clone()
                            role="tab"
                            aria-selected=move || if is_active() { "true" } else { "false" }
                            aria-controls=panel_id
                            tabindex=move || if is_active() { "0" } else { "-1" }
                            class=move || format!(
                                "px-4 py-2 font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 {}",
                                if is_active() {
                                    "border-b-2 border-primary-600 text-primary-600"
                                } else {
                                    "text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-200"
                                }
                            )
                            on:click=move |_| on_change.set(index)
                            on:keydown=move |ev| handle_keydown(ev, index)
                        >
                            {tab.label.clone()}
                        </button>
                    }
                }).collect_view()}
            </div>

            {tabs.iter().enumerate().map(|(index, tab)| {
                let is_active = move || active.get() == index;
                let tab_id = format!("tab-{}", index);
                let panel_id = format!("panel-{}", index);

                view! {
                    <div
                        id=panel_id
                        role="tabpanel"
                        aria-labelledby=tab_id
                        hidden=move || !is_active()
                        class="focus:outline-none"
                        tabindex="0"
                    >
                        {(tab.content)()}
                    </div>
                }
            }).collect_view()}
        </div>
    }
}

#[derive(Clone)]
pub struct TabItem {
    pub label: String,
    pub content: fn() -> AnyView,
}

// ============================================================================
// ACCESSIBLE TOOLTIP COMPONENT
// ============================================================================

/// Accessible tooltip with proper ARIA attributes
#[component]
pub fn AccessibleTooltip(
    /// Tooltip content
    #[prop(into)]
    content: String,
    /// Element that triggers the tooltip
    #[prop(into)]
    children: ViewFn,
    /// Additional CSS classes
    #[prop(optional, into)]
    class: Option<String>,
    /// Tooltip position
    #[prop(default = TooltipPosition::Top)]
    position: TooltipPosition,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let (visible, set_visible) = signal(false);
    let tooltip_id = generate_a11y_id("tooltip");

    let position_class = match position {
        TooltipPosition::Top => "bottom-full left-1/2 -translate-x-1/2 mb-2",
        TooltipPosition::Bottom => "top-full left-1/2 -translate-x-1/2 mt-2",
        TooltipPosition::Left => "right-full top-1/2 -translate-y-1/2 mr-2",
        TooltipPosition::Right => "left-full top-1/2 -translate-y-1/2 ml-2",
    };

    let content_stored = StoredValue::new(content);
    let tooltip_id_stored = StoredValue::new(tooltip_id.clone());

    view! {
        <div class=format!("relative inline-block {}", class)>
            <div
                aria-describedby=tooltip_id
                on:mouseenter=move |_| set_visible.set(true)
                on:mouseleave=move |_| set_visible.set(false)
                on:focus=move |_| set_visible.set(true)
                on:blur=move |_| set_visible.set(false)
            >
                {children.run()}
            </div>

            <Show when=move || visible.get()>
                <div
                    id=tooltip_id_stored.get_value()
                    role="tooltip"
                    class=format!(
                        "absolute z-tooltip px-3 py-2 text-sm text-white bg-gray-900 rounded-md shadow-lg whitespace-nowrap {}",
                        position_class
                    )
                >
                    {content_stored.get_value()}
                </div>
            </Show>
        </div>
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TooltipPosition {
    Top,
    Bottom,
    Left,
    Right,
}
