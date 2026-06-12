//! Display components: Table, Badge, List

use crate::core::types::*;
use leptos::prelude::*;

// ============================================================================
// TABLE COMPONENT
// ============================================================================

/// Type alias untuk render function yang kompleks
pub type RowRenderer<T> = Box<dyn Fn(&T) -> Vec<String>>;

#[component]
pub fn Table<T>(
    #[prop(into)] columns: Vec<TableColumn>,
    #[prop(into)] data: Vec<T>,
    #[prop(optional)] render_row: Option<RowRenderer<T>>,
    #[prop(default = false)] striped: bool,
    #[prop(default = false)] hoverable: bool,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView
where
    T: Clone + 'static,
{
    let class = class.unwrap_or_default();
    let table_class = format!(
        "min-w-full divide-y divide-gray-200 dark:divide-gray-700 {}",
        class
    );

    view! {
        <div class="overflow-x-auto">
            <table class=table_class>
                <thead class="bg-gray-50 dark:bg-gray-800">
                    <tr>
                        {columns
                            .iter()
                            .map(|col| {
                                let align_class = match col.align {
                                    TableAlign::Left => "text-left",
                                    TableAlign::Center => "text-center",
                                    TableAlign::Right => "text-right",
                                };
                                view! {
                                    <th
                                        scope="col"
                                        class=format!(
                                            "px-6 py-3 text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider {}",
                                            align_class,
                                        )
                                        style=col.width.as_ref().map(|w| format!("width: {}", w))
                                    >
                                        {col.label.clone()}
                                    </th>
                                }
                            })
                            .collect_view()}
                    </tr>
                </thead>
                <tbody class="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700">
                    {data
                        .iter()
                        .enumerate()
                        .map(|(idx, row)| {
                            let row_class = if striped && idx % 2 == 1 {
                                "bg-gray-50 dark:bg-gray-800"
                            } else {
                                ""
                            };
                            let hover_class = if hoverable {
                                "hover:bg-gray-100 dark:hover:bg-gray-700"
                            } else {
                                ""
                            };
                            let cells = if let Some(ref renderer) = render_row {
                                renderer(row)
                            } else {
                                vec![]
                            };

                            // Render cells based on render_row callback
                            // Empty if no renderer provided

                            view! {
                                <tr class=format!(
                                    "{} {}",
                                    row_class,
                                    hover_class,
                                )>
                                    {cells
                                        .into_iter()
                                        .enumerate()
                                        .map(|(col_idx, cell)| {
                                            let align_class = match columns
                                                .get(col_idx)
                                                .map(|c| &c.align)
                                            {
                                                Some(TableAlign::Center) => "text-center",
                                                Some(TableAlign::Right) => "text-right",
                                                _ => "text-left",
                                            };
                                            view! {
                                                <td class=format!(
                                                    "px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100 {}",
                                                    align_class,
                                                )>{cell}</td>
                                            }
                                        })
                                        .collect_view()}
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>

            {data
                .is_empty()
                .then(|| {
                    view! {
                        <div class="text-center py-12 text-gray-500 dark:text-gray-400">
                            <p>"Tidak ada data"</p>
                        </div>
                    }
                })}
        </div>
    }
}

// ============================================================================
// BADGE COMPONENT
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    Default,
    Primary,
    Success,
    Warning,
    Danger,
    Info,
}

#[component]
pub fn Badge(
    #[prop(into)] label: String,
    #[prop(default = BadgeVariant::Default)] variant: BadgeVariant,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let variant_class = match variant {
        BadgeVariant::Default => "bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300",
        BadgeVariant::Primary => {
            "bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-300"
        }
        BadgeVariant::Success => {
            "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300"
        }
        BadgeVariant::Warning => {
            "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300"
        }
        BadgeVariant::Danger => "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300",
        BadgeVariant::Info => "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300",
    };

    view! {
        <span class=format!(
            "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {} {}",
            variant_class,
            class,
        )>{label}</span>
    }
}

// ============================================================================
// LIST COMPONENT
// ============================================================================

#[component]
pub fn List<T, F, N>(
    #[prop(into)] items: Vec<T>,
    render_item: F,
    #[prop(optional, into)] class: Option<String>,
    #[prop(default = false)] divided: bool,
) -> impl IntoView
where
    T: Clone + 'static,
    F: Fn(T) -> N + 'static,
    N: IntoView,
{
    let class = class.unwrap_or_default();
    let list_class = if divided {
        "divide-y divide-gray-200 dark:divide-gray-700"
    } else {
        ""
    };

    let is_empty = items.is_empty();

    view! {
        {(!is_empty)
            .then(|| {
                view! {
                    <ul class=format!(
                        "{} {}",
                        list_class,
                        class,
                    )>
                        {items
                            .into_iter()
                            .map(|item| view! { <li>{render_item(item)}</li> })
                            .collect_view()}
                    </ul>
                }
            })}

        {is_empty
            .then(|| {
                view! {
                    <div class="text-center py-8 text-gray-500 dark:text-gray-400">
                        <p>"Tidak ada item"</p>
                    </div>
                }
            })}
    }
}

// ============================================================================
// EMPTY STATE COMPONENT
// ============================================================================

#[component]
pub fn EmptyState(
    #[prop(into)] message: String,
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] icon: Option<String>,
    #[prop(optional)] action: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="text-center py-12">
            {icon.map(|i| view! { <div class="text-6xl mb-4 text-gray-400">{i}</div> })}
            {title
                .map(|t| {
                    view! {
                        <h3 class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
                            {t}
                        </h3>
                    }
                })}
            <p class="text-gray-500 dark:text-gray-400 mb-6">{message}</p>
            {action.map(|a| view! { <div>{a()}</div> })}
        </div>
    }
}

// ============================================================================
// AVATAR COMPONENT
// ============================================================================
// Avatar component has been moved to optimized_image.rs for better
// image optimization with lazy loading and error handling.
// Use: use lib_ui::prelude::*; or
//      use lib_ui::components::optimized_image::Avatar;

// ============================================================================
// PAGINATION COMPONENT
// ============================================================================

#[component]
pub fn Pagination(
    pagination: Pagination,
    #[prop(optional)] on_page_change: Option<Box<dyn Fn(u32)>>,
) -> impl IntoView {
    use std::rc::Rc;
    let on_page_change_rc = Rc::new(on_page_change);
    let prev_page = pagination.current_page.saturating_sub(1);
    let next_page = pagination.current_page + 1;

    view! {
        <nav class="flex items-center justify-between border-t border-gray-200 dark:border-gray-700 px-4 py-3 sm:px-6">
            <div class="flex flex-1 justify-between sm:hidden">
                <button
                    type="button"
                    disabled=!pagination.has_prev()
                    class="relative inline-flex items-center px-4 py-2 text-sm font-medium rounded-md border border-gray-300 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                    on:click={
                        let on_change = Rc::clone(&on_page_change_rc);
                        move |_| {
                            if let Some(ref callback) = *on_change {
                                callback(prev_page);
                            }
                        }
                    }
                >
                    "Sebelumnya"
                </button>
                <button
                    type="button"
                    disabled=!pagination.has_next()
                    class="relative ml-3 inline-flex items-center px-4 py-2 text-sm font-medium rounded-md border border-gray-300 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                    on:click={
                        let on_change = Rc::clone(&on_page_change_rc);
                        move |_| {
                            if let Some(ref callback) = *on_change {
                                callback(next_page);
                            }
                        }
                    }
                >
                    "Selanjutnya"
                </button>
            </div>
            <div class="hidden sm:flex sm:flex-1 sm:items-center sm:justify-between">
                <div>
                    <p class="text-sm text-gray-700 dark:text-gray-300">
                        "Menampilkan "
                        <span class="font-medium">
                            {(pagination.current_page - 1) * pagination.page_size + 1}
                        </span> " sampai "
                        <span class="font-medium">
                            {(pagination.current_page * pagination.page_size)
                                .min(pagination.total_items as u32)}
                        </span> " dari " <span class="font-medium">{pagination.total_items}</span>
                        " hasil"
                    </p>
                </div>
                <div class="flex space-x-2">
                    <button
                        type="button"
                        disabled=!pagination.has_prev()
                        class="relative inline-flex items-center px-3 py-2 text-sm font-medium rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
                        on:click={
                            let on_change = Rc::clone(&on_page_change_rc);
                            move |_| {
                                if let Some(ref callback) = *on_change {
                                    callback(prev_page);
                                }
                            }
                        }
                    >
                        "Sebelumnya"
                    </button>
                    <span class="relative inline-flex items-center px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300">
                        {format!(
                            "Halaman {} dari {}",
                            pagination.current_page,
                            pagination.total_pages,
                        )}
                    </span>
                    <button
                        type="button"
                        disabled=!pagination.has_next()
                        class="relative inline-flex items-center px-3 py-2 text-sm font-medium rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
                        on:click={
                            let on_change = Rc::clone(&on_page_change_rc);
                            move |_| {
                                if let Some(ref callback) = *on_change {
                                    callback(next_page);
                                }
                            }
                        }
                    >
                        "Selanjutnya"
                    </button>
                </div>
            </div>
        </nav>
    }
}

// ============================================================================
// OPTIMIZED IMAGE COMPONENT
// ============================================================================

/// OptimizedImage component with lazy loading, responsive srcset, and error handling
#[component]
pub fn OptimizedImage(
    /// Image source URL
    #[prop(into)]
    src: String,
    /// Alt text for accessibility
    #[prop(into)]
    alt: String,
    /// Enable lazy loading (default: true)
    #[prop(default = true)]
    lazy: bool,
    /// Responsive image sources (srcset)
    #[prop(optional, into)]
    srcset: Option<String>,
    /// Image sizes attribute for responsive images
    #[prop(optional, into)]
    sizes: Option<String>,
    /// Width attribute
    #[prop(optional, into)]
    width: Option<String>,
    /// Height attribute
    #[prop(optional, into)]
    height: Option<String>,
    /// Object fit (cover, contain, fill, none, scale-down)
    #[prop(default = "cover".to_string(), into)]
    object_fit: String,
    /// Additional CSS classes
    #[prop(optional, into)]
    class: Option<String>,
    /// Show skeleton loader while loading
    #[prop(default = true)]
    show_skeleton: bool,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let (loaded, set_loaded) = signal(false);
    let (error, set_error) = signal(false);

    let object_fit_class = match object_fit.as_str() {
        "contain" => "object-contain",
        "fill" => "object-fill",
        "none" => "object-none",
        "scale-down" => "object-scale-down",
        _ => "object-cover",
    };

    view! {
        <div class=format!(
            "relative overflow-hidden {}",
            class,
        )>
            // Skeleton loader
            {move || {
                (show_skeleton && !loaded.get() && !error.get())
                    .then(|| {
                        view! {
                            <div class="absolute inset-0 bg-gray-200 dark:bg-gray-700 animate-pulse">
                                <div class="flex items-center justify-center h-full">
                                    <svg
                                        class="w-12 h-12 text-gray-400"
                                        fill="none"
                                        stroke="currentColor"
                                        viewBox="0 0 24 24"
                                    >
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            stroke-width="2"
                                            d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
                                        />
                                    </svg>
                                </div>
                            </div>
                        }
                    })
            }}
            // Actual image
            <img
                src=src.clone()
                alt=alt.clone()
                srcset=srcset
                sizes=sizes
                width=width
                height=height
                loading=if lazy { "lazy" } else { "eager" }
                decoding="async"
                class=format!(
                    "w-full h-full {} transition-opacity duration-300 {}",
                    object_fit_class,
                    if loaded.get() { "opacity-100" } else { "opacity-0" },
                )
                on:load=move |_| set_loaded.set(true)
                on:error=move |_| {
                    set_error.set(true);
                    set_loaded.set(false);
                }
            />
            // Error state
            {move || {
                error
                    .get()
                    .then(|| {
                        view! {
                            <div class="absolute inset-0 flex flex-col items-center justify-center bg-gray-100 dark:bg-gray-800">
                                <svg
                                    class="w-12 h-12 text-gray-400 mb-2"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-1.964-1.333-2.732 0L3.732 16c-.77 1.333.192 3 1.732 3z"
                                    />
                                </svg>
                                <span class="text-sm text-gray-500 dark:text-gray-400">
                                    "Failed to load image"
                                </span>
                            </div>
                        }
                    })
            }}
        </div>
    }
}

/// Helper function to generate srcset for responsive images
/// Generates srcset string for common breakpoints
pub fn generate_srcset(base_url: &str, widths: &[u32]) -> String {
    widths
        .iter()
        .map(|w| format!("{} {}w", base_url.replace("{width}", &w.to_string()), w))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Helper function to generate sizes attribute for responsive images
/// Common responsive sizes based on Tailwind breakpoints
pub fn generate_sizes(mobile: &str, tablet: &str, desktop: &str) -> String {
    format!(
        "(max-width: 640px) {}, (max-width: 1024px) {}, {}",
        mobile, tablet, desktop
    )
}

// ============================================================================
// QR CODE DISPLAY COMPONENT
// ============================================================================

/// QR Code Display component for MFA setup
#[component]
pub fn QrCodeDisplay(
    #[prop(into)] qr_url: String,
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] alt_text: Option<String>,
    #[prop(default = "256px".to_string(), into)] size: String,
    #[prop(default = false)] loading: bool,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let alt_text = alt_text.unwrap_or_else(|| "QR Code for MFA Setup".to_string());

    view! {
        <div class=format!("flex flex-col items-center space-y-4 {}", class)>
            <div
                class="relative bg-white p-4 rounded-lg shadow-md border border-gray-200"
                style=format!("width: {}; height: {}", size, size)
            >
                {if loading {
                    view! {
                        <div class="flex items-center justify-center w-full h-full">
                            <div class="flex flex-col items-center space-y-2">
                                <svg
                                    class="animate-spin h-8 w-8 text-gray-400"
                                    xmlns="http://www.w3.org/2000/svg"
                                    fill="none"
                                    viewBox="0 0 24 24"
                                >
                                    <circle
                                        class="opacity-25"
                                        cx="12"
                                        cy="12"
                                        r="10"
                                        stroke="currentColor"
                                        stroke-width="4"
                                    ></circle>
                                    <path
                                        class="opacity-75"
                                        fill="currentColor"
                                        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                                    ></path>
                                </svg>
                                <span class="text-sm text-gray-500">"Generating QR Code..."</span>
                            </div>
                        </div>
                    }
                        .into_any()
                } else if qr_url.is_empty() {
                    view! {
                        <div class="flex items-center justify-center w-full h-full">
                            <div class="flex flex-col items-center space-y-2 text-gray-400">
                                <svg
                                    class="w-12 h-12"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z"
                                    />
                                </svg>
                                <span class="text-sm">"QR Code not available"</span>
                            </div>
                        </div>
                    }
                        .into_any()
                } else {
                    view! {
                        <img
                            src=qr_url
                            alt=alt_text
                            class="w-full h-full object-contain"
                            style="image-rendering: pixelated; image-rendering: -moz-crisp-edges; image-rendering: crisp-edges;"
                        />
                    }
                        .into_any()
                }}
            </div>

            // Instructions
            <div class="text-center max-w-sm">
                <p class="text-sm text-gray-600 dark:text-gray-400">
                    "Scan this QR code with your authenticator app"
                </p>
                <p class="text-xs text-gray-500 dark:text-gray-500 mt-1">
                    "Compatible with Google Authenticator, Microsoft Authenticator, FreeOTP, and Authy"
                </p>
            </div>

            // Accessibility features
            <div class="sr-only">
                "QR Code for Multi-Factor Authentication setup. If you cannot scan this code, use the manual setup option below."
            </div>
        </div>
    }
}

// ============================================================================
// SORTABLE TABLE COMPONENT
// ============================================================================

/// Sortable table with pagination and filtering
/// Note: This is a simplified version that uses callbacks for sorting
#[component]
pub fn SortableTable<T>(
    #[prop(into)] columns: Vec<TableColumn>,
    #[prop(into)] data: Vec<T>,
    #[prop(optional)] render_row: Option<RowRenderer<T>>,
    #[prop(default = false)] striped: bool,
    #[prop(default = false)] hoverable: bool,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView
where
    T: Clone + 'static,
{
    let class = class.unwrap_or_default();

    view! {
        <div class="overflow-x-auto">
            <table class=format!(
                "min-w-full divide-y divide-gray-200 dark:divide-gray-700 {}",
                class,
            )>
                <thead class="bg-gray-50 dark:bg-gray-800">
                    <tr>
                        {columns
                            .iter()
                            .map(|col| {
                                let align_class = match col.align {
                                    TableAlign::Left => "text-left",
                                    TableAlign::Center => "text-center",
                                    TableAlign::Right => "text-right",
                                };

                                view! {
                                    <th
                                        scope="col"
                                        class=format!(
                                            "px-6 py-3 text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider {}",
                                            align_class,
                                        )
                                        style=col.width.as_ref().map(|w| format!("width: {}", w))
                                    >
                                        {col.label.clone()}
                                    </th>
                                }
                            })
                            .collect_view()}
                    </tr>
                </thead>
                <tbody class="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700">
                    {data
                        .iter()
                        .enumerate()
                        .map(|(idx, row)| {
                            let row_class = if striped && idx % 2 == 1 {
                                "bg-gray-50 dark:bg-gray-800"
                            } else {
                                ""
                            };
                            let hover_class = if hoverable {
                                "hover:bg-gray-100 dark:hover:bg-gray-700"
                            } else {
                                ""
                            };
                            let cells = if let Some(ref renderer) = render_row {
                                renderer(row)
                            } else {
                                vec![]
                            };

                            view! {
                                <tr class=format!(
                                    "{} {}",
                                    row_class,
                                    hover_class,
                                )>
                                    {cells
                                        .into_iter()
                                        .enumerate()
                                        .map(|(col_idx, cell)| {
                                            let align_class = match columns
                                                .get(col_idx)
                                                .map(|c| &c.align)
                                            {
                                                Some(TableAlign::Center) => "text-center",
                                                Some(TableAlign::Right) => "text-right",
                                                _ => "text-left",
                                            };
                                            view! {
                                                <td class=format!(
                                                    "px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100 {}",
                                                    align_class,
                                                )>{cell}</td>
                                            }
                                        })
                                        .collect_view()}
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>

            {data
                .is_empty()
                .then(|| {
                    view! {
                        <div class="text-center py-12 text-gray-500 dark:text-gray-400">
                            <p>"Tidak ada data"</p>
                        </div>
                    }
                })}
        </div>
    }
}

// ============================================================================
// FILTER PANEL COMPONENT
// ============================================================================

/// Filter panel for tables
#[component]
pub fn FilterPanel(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] title: Option<String>,
    #[prop(default = false)] collapsible: bool,
    children: ChildrenFn,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let (is_open, set_is_open) = signal(true);
    let title_text = title.unwrap_or_else(|| "Filters".to_string());

    view! {
        <div class=format!(
            "bg-white dark:bg-gray-800 rounded-lg shadow-md border border-gray-200 dark:border-gray-700 {}",
            class,
        )>
            {if collapsible {
                view! {
                    <div
                        class="flex items-center justify-between px-4 py-3 cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700"
                        on:click=move |_| set_is_open.update(|v| *v = !*v)
                    >
                        <h3 class="text-sm font-medium text-gray-900 dark:text-gray-100">
                            {title_text.clone()}
                        </h3>
                        <svg
                            class=format!(
                                "w-5 h-5 text-gray-500 transition-transform {}",
                                if is_open.get() { "rotate-180" } else { "" },
                            )
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M19 9l-7 7-7-7"
                            />
                        </svg>
                    </div>
                }
                    .into_any()
            } else {
                view! {
                    <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
                        <h3 class="text-sm font-medium text-gray-900 dark:text-gray-100">
                            {title_text}
                        </h3>
                    </div>
                }
                    .into_any()
            }}
            {move || (is_open.get() || !collapsible).then(|| children())}
        </div>
    }
}

// ============================================================================
// EXPORT BUTTON COMPONENT
// ============================================================================

/// Export button with dropdown options
#[component]
pub fn ExportButton(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] on_csv_click: Option<Callback<()>>,
    #[prop(optional, into)] on_excel_click: Option<Callback<()>>,
    #[prop(optional, into)] on_pdf_click: Option<Callback<()>>,
    #[prop(default = false)] disabled: bool,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let (is_open, set_is_open) = signal(false);

    view! {
        <div class=format!("relative inline-block text-left {}", class)>
            <button
                type="button"
                disabled=disabled
                class="inline-flex items-center px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-emerald-500 disabled:opacity-50 disabled:cursor-not-allowed dark:bg-gray-800 dark:text-gray-300 dark:border-gray-600"
                on:click=move |_| set_is_open.update(|v| *v = !*v)
            >
                <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
                    />
                </svg>
                "Export"
                <svg class="w-4 h-4 ml-2" fill="currentColor" viewBox="0 0 20 20">
                    <path
                        fill-rule="evenodd"
                        d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z"
                        clip-rule="evenodd"
                    />
                </svg>
            </button>

            <Show when=move || is_open.get()>
                <div class="absolute right-0 z-10 mt-2 w-48 rounded-md shadow-lg bg-white dark:bg-gray-800 ring-1 ring-black ring-opacity-5">
                    <div class="py-1" role="menu">
                        {on_csv_click
                            .map(|callback| {
                                view! {
                                    <button
                                        type="button"
                                        class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
                                        role="menuitem"
                                        on:click=move |_| {
                                            callback.run(());
                                            set_is_open.set(false);
                                        }
                                    >
                                        "Export as CSV"
                                    </button>
                                }
                            })}

                        {on_excel_click
                            .map(|callback| {
                                view! {
                                    <button
                                        type="button"
                                        class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
                                        role="menuitem"
                                        on:click=move |_| {
                                            callback.run(());
                                            set_is_open.set(false);
                                        }
                                    >
                                        "Export as Excel"
                                    </button>
                                }
                            })}

                        {on_pdf_click
                            .map(|callback| {
                                view! {
                                    <button
                                        type="button"
                                        class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
                                        role="menuitem"
                                        on:click=move |_| {
                                            callback.run(());
                                            set_is_open.set(false);
                                        }
                                    >
                                        "Export as PDF"
                                    </button>
                                }
                            })}
                    </div>
                </div>
            </Show>
        </div>
    }
}
