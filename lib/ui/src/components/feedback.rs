//! Feedback components: Toast, Modal, Alert, Loading

use crate::core::types::{AlertVariant, ToastType};
use leptos::prelude::*;

// ============================================================================
// TOAST COMPONENT
// ============================================================================

#[component]
pub fn Toast(
    #[prop(into)] message: String,
    #[prop(default = ToastType::Info)] toast_type: ToastType,
    #[prop(default = true)] show: bool,
    #[prop(optional)] on_close: Option<Box<dyn Fn()>>,
) -> impl IntoView {
    let handle_close = move |_| {
        if let Some(ref callback) = on_close {
            callback();
        }
    };

    view! {
        <div class=format!(
            "fixed top-4 right-4 max-w-sm w-full bg-white dark:bg-gray-800 rounded-lg shadow-lg border-l-4 {} p-4 transition-all duration-300 z-toast {}",
            match toast_type {
                ToastType::Success => "border-green-500",
                ToastType::Error => "border-red-500",
                ToastType::Warning => "border-yellow-500",
                ToastType::Info => "border-blue-500",
            },
            if show { "translate-x-0 opacity-100" } else { "translate-x-full opacity-0" },
        )>
            <div class="flex items-start">
                <div class="flex-shrink-0">
                    <span class=format!(
                        "text-2xl {}",
                        match toast_type {
                            ToastType::Success => "text-green-500",
                            ToastType::Error => "text-red-500",
                            ToastType::Warning => "text-yellow-500",
                            ToastType::Info => "text-blue-500",
                        },
                    )>{toast_type.icon()}</span>
                </div>
                <div class="ml-3 flex-1">
                    <p class="text-sm font-medium text-gray-900 dark:text-gray-100">{message}</p>
                </div>
                <div class="ml-4 flex-shrink-0 flex">
                    <button
                        type="button"
                        class="inline-flex text-gray-400 hover:text-gray-500 focus:outline-none"
                        on:click=handle_close
                    >
                        <span class="sr-only">"Close"</span>
                        <svg
                            class="h-5 w-5"
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 20 20"
                            fill="currentColor"
                        >
                            <path
                                fill-rule="evenodd"
                                d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                                clip-rule="evenodd"
                            />
                        </svg>
                    </button>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// MODAL COMPONENT
// ============================================================================

#[component]
pub fn Modal(
    #[prop(into)] title: String,
    #[prop(default = true)] show: bool,
    #[prop(default = true)] show_close: bool,
    #[prop(optional, into)] width: Option<String>,
    #[prop(optional)] on_close: Option<Box<dyn Fn()>>,
    children: Children,
) -> impl IntoView {
    use std::rc::Rc;

    let width = width.unwrap_or_else(|| "max-w-lg".to_string());
    let on_close = Rc::new(on_close);

    let handle_close = {
        let on_close = Rc::clone(&on_close);
        move |_| {
            if let Some(ref callback) = *on_close {
                callback();
            }
        }
    };

    let handle_backdrop_click = {
        let on_close = Rc::clone(&on_close);
        move |_| {
            if let Some(ref callback) = *on_close {
                callback();
            }
        }
    };

    view! {
        <div class=format!(
            "fixed inset-0 z-modal overflow-y-auto transition-all duration-300 {}",
            if show { "opacity-100 pointer-events-auto" } else { "opacity-0 pointer-events-none" },
        )>
            // Backdrop
            <div
                class="fixed inset-0 bg-black bg-opacity-50 transition-opacity"
                on:click=handle_backdrop_click
            ></div>

            // Modal content
            <div class="flex min-h-full items-center justify-center p-4">
                <div class=format!(
                    "relative bg-white dark:bg-gray-800 rounded-lg shadow-xl transform transition-all w-full {}",
                    width,
                )>
                    // Header
                    <div class="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700">
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
                            {title}
                        </h3>
                        {show_close
                            .then(|| {
                                view! {
                                    <button
                                        type="button"
                                        class="text-gray-400 hover:text-gray-500 focus:outline-none"
                                        on:click=handle_close
                                    >
                                        <span class="sr-only">"Close"</span>
                                        <svg
                                            class="h-6 w-6"
                                            xmlns="http://www.w3.org/2000/svg"
                                            fill="none"
                                            viewBox="0 0 24 24"
                                            stroke="currentColor"
                                        >
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                stroke-width="2"
                                                d="M6 18L18 6M6 6l12 12"
                                            />
                                        </svg>
                                    </button>
                                }
                            })}
                    </div>

                    // Body
                    <div class="p-6">{children()}</div>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// ALERT COMPONENT
// ============================================================================

#[component]
pub fn Alert(
    #[prop(into)] message: String,
    #[prop(default = AlertVariant::Info)] variant: AlertVariant,
    #[prop(optional, into)] title: Option<String>,
    #[prop(default = true)] dismissible: bool,
    #[prop(default = true)] show: bool,
    #[prop(optional)] on_dismiss: Option<Box<dyn Fn()>>,
) -> impl IntoView {
    let handle_dismiss = move |_| {
        if let Some(ref callback) = on_dismiss {
            callback();
        }
    };

    let (bg_class, text_class, border_class) = match variant {
        AlertVariant::Info => (
            "bg-blue-50 dark:bg-blue-900/20",
            "text-blue-800 dark:text-blue-200",
            "border-blue-200 dark:border-blue-800",
        ),
        AlertVariant::Success => (
            "bg-green-50 dark:bg-green-900/20",
            "text-green-800 dark:text-green-200",
            "border-green-200 dark:border-green-800",
        ),
        AlertVariant::Warning => (
            "bg-yellow-50 dark:bg-yellow-900/20",
            "text-yellow-800 dark:text-yellow-200",
            "border-yellow-200 dark:border-yellow-800",
        ),
        AlertVariant::Error => (
            "bg-red-50 dark:bg-red-900/20",
            "text-red-800 dark:text-red-200",
            "border-red-200 dark:border-red-800",
        ),
    };

    view! {
        <div class=format!(
            "rounded-md border p-4 {} {} {} {}",
            bg_class,
            text_class,
            border_class,
            if show { "block" } else { "hidden" },
        )>
            <div class="flex">
                <div class="flex-1">
                    {title.map(|t| view! { <h3 class="text-sm font-medium mb-1">{t}</h3> })}
                    <p class="text-sm">{message}</p>
                </div>
                {dismissible
                    .then(|| {
                        view! {
                            <button
                                type="button"
                                class="ml-3 inline-flex flex-shrink-0 focus:outline-none"
                                on:click=handle_dismiss
                            >
                                <span class="sr-only">"Dismiss"</span>
                                <svg
                                    class="h-5 w-5"
                                    xmlns="http://www.w3.org/2000/svg"
                                    viewBox="0 0 20 20"
                                    fill="currentColor"
                                >
                                    <path
                                        fill-rule="evenodd"
                                        d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                                        clip-rule="evenodd"
                                    />
                                </svg>
                            </button>
                        }
                    })}
            </div>
        </div>
    }
}

// ============================================================================
// LOADING SPINNER
// ============================================================================

#[component]
pub fn Loading(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] text: Option<String>,
    #[prop(default = "md".to_string(), into)] size: String,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let size_class = match size.as_str() {
        "sm" => "h-4 w-4",
        "lg" => "h-12 w-12",
        _ => "h-8 w-8",
    };

    view! {
        <div class=format!("flex flex-col items-center justify-center {}", class)>
            <svg
                class=format!("animate-spin {} text-emerald-600", size_class)
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
            {text
                .map(|t| {
                    view! { <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">{t}</p> }
                })}
        </div>
    }
}

// ============================================================================
// PROGRESS BAR
// ============================================================================

#[component]
pub fn ProgressBar(
    #[prop(default = 0)] value: u32,
    #[prop(default = 100)] max: u32,
    #[prop(optional, into)] class: Option<String>,
    #[prop(default = false)] show_label: bool,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let percentage = ((value as f32 / max as f32) * 100.0).min(100.0);

    view! {
        <div class=format!(
            "w-full {}",
            class,
        )>
            {show_label
                .then(|| {
                    view! {
                        <div class="flex justify-between mb-1">
                            <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                                {format!("{}%", percentage as u32)}
                            </span>
                        </div>
                    }
                })} <div class="w-full bg-gray-200 rounded-full h-2.5 dark:bg-gray-700">
                <div
                    class="bg-emerald-600 h-2.5 rounded-full transition-all duration-300"
                    style=format!("width: {}%", percentage)
                ></div>
            </div>
        </div>
    }
}

// ============================================================================
// SKELETON LOADER
// ============================================================================

#[component]
pub fn Skeleton(
    #[prop(optional, into)] class: Option<String>,
    #[prop(default = "rectangle".to_string(), into)] variant: String,
    #[prop(optional, into)] width: Option<String>,
    #[prop(optional, into)] height: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();

    let (base_class, style) = match variant.as_str() {
        "circle" => {
            let size = width.unwrap_or_else(|| "48px".to_string());
            ("rounded-full", format!("width: {}; height: {}", size, size))
        }
        "text" => {
            let h = height.unwrap_or_else(|| "1rem".to_string());
            let w = width.unwrap_or_else(|| "100%".to_string());
            ("rounded", format!("width: {}; height: {}", w, h))
        }
        _ => {
            let h = height.unwrap_or_else(|| "4rem".to_string());
            let w = width.unwrap_or_else(|| "100%".to_string());
            ("rounded-md", format!("width: {}; height: {}", w, h))
        }
    };

    view! {
        <div
            class=format!("animate-pulse bg-gray-200 dark:bg-gray-700 {} {}", base_class, class)
            style=style
            role="status"
            aria-label="Loading..."
        ></div>
    }
}

// ============================================================================
// NOTIFICATION/SNACKBAR COMPONENT
// ============================================================================

#[component]
pub fn Notification(
    #[prop(into)] message: String,
    #[prop(optional, into)] title: Option<String>,
    #[prop(default = ToastType::Info)] notification_type: ToastType,
    #[prop(default = true)] show: bool,
    #[prop(default = "top-right".to_string(), into)] position: String,
    #[prop(default = false)] auto_dismiss: bool,
    #[prop(default = 5000)] dismiss_after: u32,
    #[prop(optional)] on_close: Option<Box<dyn Fn()>>,
) -> impl IntoView {
    use std::rc::Rc;
    let on_close_rc = Rc::new(on_close);

    // Auto-dismiss logic
    if auto_dismiss && show {
        let on_close = Rc::clone(&on_close_rc);
        leptos::task::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(dismiss_after).await;
            if let Some(ref callback) = *on_close {
                callback();
            }
        });
    }

    let position_class = match position.as_str() {
        "top-left" => "top-4 left-4",
        "top-center" => "top-4 left-1/2 -translate-x-1/2",
        "top-right" => "top-4 right-4",
        "bottom-left" => "bottom-4 left-4",
        "bottom-center" => "bottom-4 left-1/2 -translate-x-1/2",
        "bottom-right" => "bottom-4 right-4",
        _ => "top-4 right-4",
    };

    let handle_close = {
        let on_close = Rc::clone(&on_close_rc);
        move |_| {
            if let Some(ref callback) = *on_close {
                callback();
            }
        }
    };

    view! {
        <div
            class=format!(
                "fixed {} max-w-sm w-full bg-white dark:bg-gray-800 rounded-lg shadow-lg border-l-4 {} p-4 transition-all duration-300 z-toast {}",
                position_class,
                match notification_type {
                    ToastType::Success => "border-green-500",
                    ToastType::Error => "border-red-500",
                    ToastType::Warning => "border-yellow-500",
                    ToastType::Info => "border-blue-500",
                },
                if show { "translate-x-0 opacity-100" } else { "translate-x-full opacity-0" },
            )
            role="alert"
            aria-live="polite"
        >
            <div class="flex items-start">
                <div class="flex-shrink-0">
                    <span class=format!(
                        "text-2xl {}",
                        match notification_type {
                            ToastType::Success => "text-green-500",
                            ToastType::Error => "text-red-500",
                            ToastType::Warning => "text-yellow-500",
                            ToastType::Info => "text-blue-500",
                        },
                    )>{notification_type.icon()}</span>
                </div>
                <div class="ml-3 flex-1">
                    {title
                        .map(|t| {
                            view! {
                                <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-1">
                                    {t}
                                </h3>
                            }
                        })} <p class="text-sm text-gray-700 dark:text-gray-300">{message}</p>
                </div>
                <div class="ml-4 flex-shrink-0 flex">
                    <button
                        type="button"
                        class="inline-flex text-gray-400 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 rounded"
                        on:click=handle_close
                        aria-label="Close notification"
                    >
                        <svg
                            class="h-5 w-5"
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 20 20"
                            fill="currentColor"
                        >
                            <path
                                fill-rule="evenodd"
                                d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                                clip-rule="evenodd"
                            />
                        </svg>
                    </button>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// SPINNER COMPONENT (Alternative to Loading)
// ============================================================================

#[component]
pub fn Spinner(
    #[prop(optional, into)] class: Option<String>,
    #[prop(default = "md".to_string(), into)] size: String,
    #[prop(optional, into)] color: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let size_class = match size.as_str() {
        "sm" => "h-4 w-4",
        "lg" => "h-12 w-12",
        "xl" => "h-16 w-16",
        _ => "h-8 w-8",
    };
    let color_class = color.unwrap_or_else(|| "text-emerald-600".to_string());

    view! {
        <svg
            class=format!("animate-spin {} {} {}", size_class, color_class, class)
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            role="status"
            aria-label="Loading"
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
    }
}
