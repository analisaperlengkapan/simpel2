//! Layout components: Card, Container

use leptos::prelude::*;

// ============================================================================
// CARD COMPONENT
// ============================================================================

#[component]
pub fn Card(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] title: Option<String>,
    #[prop(default = true)] padding: bool,
    #[prop(default = false)] hoverable: bool,
    #[prop(default = false)] clickable: bool,
    #[prop(optional)] on_click: Option<Box<dyn Fn()>>,
    children: Children,
) -> impl IntoView {
    let padding_class = if padding { "p-6" } else { "" };
    let class = class.unwrap_or_default();
    let hover_class = if hoverable {
        "hover:shadow-lg transition-shadow duration-200"
    } else {
        ""
    };
    let cursor_class = if clickable { "cursor-pointer" } else { "" };

    use std::rc::Rc;
    let on_click_rc = Rc::new(on_click);

    let handle_click = {
        let on_click = Rc::clone(&on_click_rc);
        move |_| {
            if let Some(ref callback) = *on_click {
                callback();
            }
        }
    };

    let handle_keydown = {
        let on_click = Rc::clone(&on_click_rc);
        move |ev: web_sys::KeyboardEvent| {
            if clickable && (ev.key() == "Enter" || ev.key() == " ")
                && let Some(ref callback) = *on_click {
                    callback();
                }
        }
    };

    view! {
        <div
            class=format!(
                "bg-white dark:bg-gray-800 rounded-lg shadow-md border border-gray-200 dark:border-gray-700 {} {} {} {}",
                hover_class, cursor_class, class, if clickable { "focus:outline-none focus:ring-2 focus:ring-primary-500" } else { "" }
            )
            role=if clickable { Some("button") } else { None }
            tabindex=if clickable { Some("0") } else { None }
            on:click=handle_click
            on:keydown=handle_keydown
        >
            {title.map(|t| view! {
                <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                    <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
                        {t}
                    </h3>
                </div>
            })}
            <div class=padding_class>
                {children()}
            </div>
        </div>
    }
}

// ============================================================================
// CONTAINER COMPONENT
// ============================================================================

#[component]
pub fn Container(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] max_width: Option<String>,
    #[prop(default = false)] fluid: bool,
    #[prop(optional, into)] as_element: Option<String>,
    children: Children,
) -> impl IntoView {
    let max_width = if fluid {
        "max-w-full".to_string()
    } else {
        max_width.unwrap_or_else(|| "max-w-7xl".to_string())
    };
    let class = class.unwrap_or_default();
    let element = as_element.unwrap_or_else(|| "div".to_string());

    let container_class = format!(
        "container mx-auto px-4 sm:px-6 lg:px-8 {} {}",
        max_width, class
    );

    match element.as_str() {
        "main" => view! {
            <main class=container_class role="main">
                {children()}
            </main>
        }
        .into_any(),
        "section" => view! {
            <section class=container_class>
                {children()}
            </section>
        }
        .into_any(),
        "article" => view! {
            <article class=container_class>
                {children()}
            </article>
        }
        .into_any(),
        _ => view! {
            <div class=container_class>
                {children()}
            </div>
        }
        .into_any(),
    }
}

// ============================================================================
// GRID COMPONENT
// ============================================================================

#[component]
pub fn Grid(
    #[prop(optional, into)] class: Option<String>,
    #[prop(default = 1)] cols: u8,
    #[prop(default = 4)] gap: u8,
    #[prop(default = false)] auto_fit: bool,
    #[prop(optional, into)] min_col_width: Option<String>,
    children: Children,
) -> impl IntoView {
    let class = class.unwrap_or_default();

    let (grid_class, style) = if auto_fit {
        // Auto-fit grid with minimum column width
        let min_width = min_col_width.unwrap_or_else(|| "250px".to_string());
        let grid_class = format!("grid gap-{} {}", gap, class);
        let style = Some(format!(
            "grid-template-columns: repeat(auto-fit, minmax({}, 1fr))",
            min_width
        ));
        (grid_class, style)
    } else {
        // Fixed column grid with responsive breakpoints
        let cols_class = match cols {
            1 => "grid-cols-1",
            2 => "grid-cols-1 md:grid-cols-2",
            3 => "grid-cols-1 md:grid-cols-2 lg:grid-cols-3",
            4 => "grid-cols-1 md:grid-cols-2 lg:grid-cols-4",
            5 => "grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5",
            6 => "grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6",
            _ => "grid-cols-1",
        };
        let grid_class = format!("grid {} gap-{} {}", cols_class, gap, class);
        (grid_class, None)
    };

    view! {
        <div
            class=grid_class
            style=style
            role="grid"
        >
            {children()}
        </div>
    }
}

// ============================================================================
// STACK COMPONENT (Flexbox)
// ============================================================================

#[component]
pub fn Stack(
    #[prop(optional, into)] class: Option<String>,
    #[prop(default = false)] horizontal: bool,
    #[prop(default = 4)] gap: u8,
    #[prop(optional, into)] align: Option<String>,
    #[prop(optional, into)] justify: Option<String>,
    #[prop(default = false)] wrap: bool,
    #[prop(default = false)] responsive: bool,
    children: Children,
) -> impl IntoView {
    let class = class.unwrap_or_default();

    // Responsive: stack vertically on mobile, horizontally on desktop
    let direction = if responsive {
        if horizontal {
            "flex-col md:flex-row"
        } else {
            "flex-row md:flex-col"
        }
    } else if horizontal {
        "flex-row"
    } else {
        "flex-col"
    };

    let gap_class = format!("gap-{}", gap);
    let align_class = align.unwrap_or_else(|| "items-start".to_string());
    let justify_class = justify.unwrap_or_else(|| "justify-start".to_string());
    let wrap_class = if wrap { "flex-wrap" } else { "" };

    view! {
        <div
            class=format!("flex {} {} {} {} {} {}", direction, gap_class, align_class, justify_class, wrap_class, class)
            role="group"
        >
            {children()}
        </div>
    }
}

// ============================================================================
// FOOTER COMPONENT
// ============================================================================

#[component]
pub fn Footer(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] copyright: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let copyright = copyright.unwrap_or_else(|| "© 2024 Kejaksaan Republik Indonesia".to_string());

    view! {
        <footer class=format!("bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 mt-auto {}", class)>
            <div class="container mx-auto px-4 py-6">
                <div class="flex flex-col md:flex-row justify-between items-center space-y-4 md:space-y-0">
                    <div class="text-sm text-gray-600 dark:text-gray-400">
                        {copyright}
                    </div>
                    {children.map(|children| view! {
                        <div class="flex items-center space-x-4">
                            {children()}
                        </div>
                    })}
                </div>
            </div>
        </footer>
    }
}

// ============================================================================
// DIVIDER COMPONENT
// ============================================================================

#[component]
pub fn Divider(
    #[prop(optional, into)] class: Option<String>,
    #[prop(default = false)] vertical: bool,
    #[prop(optional, into)] label: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();

    if let Some(text) = label {
        // Divider with label
        view! {
            <div class=format!("relative flex items-center {}", class)>
                <div class="flex-grow border-t border-gray-300 dark:border-gray-600"></div>
                <span class="flex-shrink mx-4 text-sm text-gray-500 dark:text-gray-400">
                    {text}
                </span>
                <div class="flex-grow border-t border-gray-300 dark:border-gray-600"></div>
            </div>
        }
        .into_any()
    } else if vertical {
        // Vertical divider
        view! {
            <div
                class=format!("w-px bg-gray-300 dark:bg-gray-600 {}", class)
                role="separator"
                aria-orientation="vertical"
            ></div>
        }
        .into_any()
    } else {
        // Horizontal divider
        view! {
            <hr
                class=format!("border-t border-gray-300 dark:border-gray-600 {}", class)
                role="separator"
                aria-orientation="horizontal"
            />
        }
        .into_any()
    }
}

// ============================================================================
// SECTION COMPONENT
// ============================================================================

#[component]
pub fn Section(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] description: Option<String>,
    #[prop(default = false)] centered: bool,
    children: Children,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let text_align = if centered { "text-center" } else { "" };

    view! {
        <section class=format!("py-8 {}", class)>
            {(title.is_some() || description.is_some()).then(|| view! {
                <div class=format!("mb-6 {}", text_align)>
                    {title.map(|t| view! {
                        <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">
                            {t}
                        </h2>
                    })}
                    {description.map(|d| view! {
                        <p class="text-gray-600 dark:text-gray-400">
                            {d}
                        </p>
                    })}
                </div>
            })}
            <div>
                {children()}
            </div>
        </section>
    }
}

// ============================================================================
// SPACER COMPONENT
// ============================================================================

#[component]
pub fn Spacer(
    #[prop(default = 4)] size: u8,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let height_class = format!("h-{}", size);

    view! {
        <div class=format!("{} {}", height_class, class) aria-hidden="true"></div>
    }
}
