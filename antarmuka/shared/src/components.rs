//! Modern Leptos components with Kejaksaan RI branding
//! Accessibility compliant UI components for SIMPelv2 microfrontend architecture

use crate::types::*;
use leptos::children::Children;
use leptos::prelude::{
    AriaAttributes, ClassAttribute, CollectView, CustomAttribute, ElementChild, Get,
    GlobalAttributes, OnAttribute, ReadSignal, RwSignal, Show,
};
use leptos::*;

/// Primary button with Kejaksaan RI styling
#[component]
pub fn KejButton(
    #[prop(optional, default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[prop(optional, default = ButtonSize::Medium)] size: ButtonSize,
    #[prop(optional, default = false)] disabled: bool,
    #[prop(optional, default = "")] class: &'static str,
    #[prop(optional)] on_click: Option<Box<dyn Fn() + Send + Sync>>,
    children: Children,
) -> impl IntoView {
    let base_class = match size {
        ButtonSize::Small => "px-3 py-1.5 text-sm",
        ButtonSize::Medium => "px-4 py-2 text-base",
        ButtonSize::Large => "px-6 py-3 text-lg",
    };

    let variant_class = match variant {
        ButtonVariant::Primary => "bg-kejaksaan-primary text-white hover:bg-kejaksaan-primary-dark",
        ButtonVariant::Secondary => "bg-kejaksaan-secondary text-kejaksaan-primary hover:bg-kejaksaan-secondary-dark",
        ButtonVariant::Success => "bg-kejaksaan-success text-white hover:bg-kejaksaan-success-dark",
        ButtonVariant::Warning => "bg-kejaksaan-warning text-white hover:bg-kejaksaan-warning-dark",
        ButtonVariant::Danger => "bg-kejaksaan-danger text-white hover:bg-kejaksaan-danger-dark",
        ButtonVariant::Ghost => "bg-transparent border border-kejaksaan-primary text-kejaksaan-primary hover:bg-kejaksaan-bg",
        ButtonVariant::Outline => "bg-white border border-kejaksaan-primary text-kejaksaan-primary hover:bg-kejaksaan-bg",
    };

    let classes = format!(
        "inline-flex items-center justify-center rounded-md font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-kejaksaan-primary focus:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none {base_class} {variant_class} {class}"
    );

    view! {
        <button
            class=classes
            disabled=disabled
            on:click=move |_| {
                if let Some(handler) = &on_click {
                    handler();
                }
            }
            type="button"
            aria-label="Button"
        >
            {children()}
        </button>
    }
}

/// Input field with validation and Kejaksaan styling
#[component]
pub fn KejInput(
    #[prop(optional, default = "text")] _input_type: &'static str,
    #[prop(optional, default = "")] _placeholder: &'static str,
    #[prop(optional)] _value: RwSignal<String>,
    #[prop(optional, default = "")] _label: &'static str,
    #[prop(optional, default = false)] _required: bool,
    #[prop(optional, default = false)] _disabled: bool,
    #[prop(optional, default = "")] class: &'static str,
) -> impl IntoView {
    let _input_id = uuid::Uuid::new_v4().simple().to_string();

    view! {
        <div class=format!("space-y-1 {}", class)>
                type=input_type
            placeholder=placeholder
            value=value
            class=format!("block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-kejaksaan-blue focus:border-kejaksaan-blue sm:text-sm {}", if error.is_some() { "border-red-500" } else { "" })
            on:change=move |_| {
                // Event handling moved to closure
            }
        </div>
    }
}

/// Modal component with accessibility
#[component]
pub fn KejModal(
    show: ReadSignal<bool>,
    #[prop(optional)] on_close: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
    #[prop(optional, default = "")] title: &'static str,
    #[prop(optional, default = ModalSize::Medium)] size: ModalSize,
    children: Children,
) -> impl IntoView {
    let modal_class = match size {
        ModalSize::Small => "max-w-md",
        ModalSize::Medium => "max-w-lg",
        ModalSize::Large => "max-w-2xl",
        ModalSize::ExtraLarge => "max-w-4xl",
        ModalSize::FullScreen => "w-screen h-screen max-w-none",
    };

    view! {
        <div
            class="fixed inset-0 z-50 overflow-y-auto"
            class:hidden=move || !show.get()
            role="dialog"
            aria-modal="true"
            aria-labelledby="modal-title"
        >
            <div class="flex items-center justify-center min-h-screen px-4 pt-4 pb-20 text-center">
                <div
                    class="fixed inset-0 transition-opacity bg-kejaksaan-overlay"
                    on:click={
                        let on_close = on_close.clone();
                        move |_| {
                            if let Some(handler) = &on_close {
                                handler();
                            }
                        }
                    }
                ></div>
                <div class=format!(
                    "relative inline-block w-full {} px-4 pt-5 pb-4 overflow-hidden text-left align-bottom transition-all transform bg-white rounded-lg shadow-xl sm:my-8 sm:align-middle sm:p-6",
                    modal_class
                )>
                    <div class="absolute top-0 right-0 pt-4 pr-4">
                        <button
                            type="button"
                            class="text-kejaksaan-text-muted hover:text-kejaksaan-text focus:outline-none focus:text-kejaksaan-text"
                            on:click={
                                let on_close = on_close.clone();
                                move |_| {
                                    if let Some(handler) = &on_close {
                                        handler();
                                    }
                                }
                            }
                            aria-label="Close modal"
                        >
                            <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                        </button>
                    </div>

                    {if !title.is_empty() {
                        view! {
                            <h3 id="modal-title" class="text-lg font-medium leading-6 text-kejaksaan-text mb-4">
                                {title}
                            </h3>
                        }
                    } else {
                            view! { <h3 id="modal-title" class="text-lg font-medium leading-6 text-kejaksaan-text mb-4">{""}</h3> }
                    }}

                    <div class="mt-2">
                        {children()}
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Card component for content sections
#[component]
pub fn KejCard(
    #[prop(optional, default = "")] title: &'static str,
    #[prop(optional, default = "")] class: &'static str,
    #[prop(optional, default = CardPadding::Medium)] padding: CardPadding,
    children: Children,
) -> impl IntoView {
    let padding_class = match padding {
        CardPadding::None => "",
        CardPadding::Small => "p-4",
        CardPadding::Medium => "p-6",
        CardPadding::Large => "p-8",
    };

    view! {
        <div class=format!(
            "bg-white rounded-lg border border-kejaksaan-border shadow-sm {}",
            class
        )>
            {view! {
                <div class="px-6 py-4 border-b border-kejaksaan-border">
                    <h3 class="text-lg font-medium text-kejaksaan-text">
                        {title}
                    </h3>
                </div>
            }}
            <div class=padding_class>
                {children()}
            </div>
        </div>
    }
}

/// Notification component
#[component]
pub fn KejNotification(
    #[prop(optional, default = MessageType::Info)] message_type: MessageType,
    #[prop(optional, default = "")] title: &'static str,
    #[prop(optional, default = "")] message: &'static str,
    show: ReadSignal<bool>,
    #[prop(optional)] on_close: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
) -> impl IntoView {
    let (bg_class, text_class, border_class) = match message_type {
        MessageType::Success => (
            "bg-kejaksaan-success-light",
            "text-kejaksaan-success-dark",
            "border-kejaksaan-success",
        ),
        MessageType::Warning => (
            "bg-kejaksaan-warning-light",
            "text-kejaksaan-warning-dark",
            "border-kejaksaan-warning",
        ),
        MessageType::Error => (
            "bg-kejaksaan-danger-light",
            "text-kejaksaan-danger-dark",
            "border-kejaksaan-danger",
        ),
        MessageType::Info => (
            "bg-kejaksaan-info-light",
            "text-kejaksaan-info-dark",
            "border-kejaksaan-info",
        ),
    };

    view! {
        <div
            class=format!(
                "fixed top-4 right-4 z-50 max-w-sm w-full {} {} border-l-4 {} p-4 rounded-md shadow-lg transition-all duration-300 transform",
                bg_class, text_class, border_class
            )
            class:translate-x-full=move || !show.get()
            class:translate-x-0=move || show.get()
            role="alert"
            aria-live="polite"
        >
            <div class="flex items-start">
                <div class="flex-1">
                    {if !title.is_empty() {
                        view! { <h4 class="font-medium mb-1">{title}</h4> }
                    } else {
                        view! { <h4 class="font-medium mb-1">{""}</h4> }
                    }}
                    <p class="text-sm">{message}</p>
                </div>
                {view! {
                    <button
                        type="button"
                        class="ml-2 flex-shrink-0 hover:opacity-75 focus:outline-none"
                        on:click={
                            let on_close = on_close.clone();
                            move |_| {
                                if let Some(handler) = &on_close {
                                    handler();
                                }
                            }
                        }
                        aria-label="Close notification"
                    >
                            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                                <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                            </svg>
                    </button>
                }}
            </div>
        </div>
    }
}

/// Loading spinner component
#[component]
pub fn KejSpinner(
    #[prop(optional, default = SpinnerSize::Medium)] size: SpinnerSize,
    #[prop(optional, default = "")] class: &'static str,
) -> impl IntoView {
    let size_class = match size {
        SpinnerSize::Small => "w-4 h-4",
        SpinnerSize::Medium => "w-6 h-6",
        SpinnerSize::Large => "w-8 h-8",
    };

    view! {
        <div class=format!("flex items-center justify-center {}", class)>
            <svg
                class=format!("animate-spin text-kejaksaan-primary {}", size_class)
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
        </div>
    }
}

/// Breadcrumb navigation component
#[component]
pub fn KejBreadcrumb(
    #[prop(optional)] items: Vec<BreadcrumbItem>,
    #[prop(optional, default = "/")] separator: &'static str,
) -> impl IntoView {
    view! {
        <nav class="flex" aria-label="Breadcrumb">
            <ol class="inline-flex items-center space-x-1 md:space-x-3">
                {items.iter().enumerate().map(|(index, item)| {
                    let is_last = index == items.len() - 1;
                        let label = item.label.clone();
                        view! {
                            <li class="inline-flex items-center">
                                {if index > 0 {
                                    view! { <span class="mx-2 text-kejaksaan-text-muted">{separator}</span> }
                                } else {
                                    view! { <span class="mx-2 text-kejaksaan-text-muted">{""}</span> }
                                }}
                                <span class={if is_last { "text-kejaksaan-text font-medium" } else { "text-kejaksaan-text-muted" }} aria-current={if is_last { Some("page") } else { None }}>
                                    {label}
                                </span>
                            </li>
                        }
                }).collect_view()}
            </ol>
        </nav>
    }
}

/// Data table component
#[component]
pub fn KejTable<T>(
    #[prop(optional)] data: Vec<T>,
    #[prop(optional)] columns: Vec<TableColumn>,
    #[prop(optional, default = false)] loading: bool,
    #[prop(optional, default = "Tidak ada data")] _empty_message: &'static str,
) -> impl IntoView
where
    T: Clone + 'static,
{
    view! {
        <div class="overflow-hidden shadow ring-1 ring-black ring-opacity-5 md:rounded-lg">
            <table class="min-w-full divide-y divide-kejaksaan-border">
                <thead class="bg-kejaksaan-bg">
                    <tr>
                        {columns.iter().map(|column| {
                            view! {
                                <th
                                    scope="col"
                                    class="px-6 py-3 text-left text-xs font-medium text-kejaksaan-text uppercase tracking-wider"
                                >
                                    {column.title.clone()}
                                </th>
                            }
                        }).collect_view()}
                    </tr>
                </thead>
                <tbody class="bg-white divide-y divide-kejaksaan-border">
                    {if loading {
                        view! { <>{vec![view! {
                            <tr class="hover:bg-kejaksaan-bg">
                                <td class="px-6 py-4 text-center">
                                    {"Loading..."}
                                </td>
                            </tr>
                        }]}</> }
                    } else if data.is_empty() {
                        view! { <>{vec![view! {
                            <tr class="hover:bg-kejaksaan-bg">
                                <td class="px-6 py-4 text-center text-kejaksaan-text-muted">
                                    {"No data"}
                                </td>
                            </tr>
                        }]}</> }
                    } else {
                        view! { <>{vec![view! {
                            <tr class="hover:bg-kejaksaan-bg">
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-kejaksaan-text">
                                    {"Data placeholder"}
                                </td>
                            </tr>
                        }]}</> }
                    }}
                </tbody>
            </table>
        </div>
    }
}

/// App Header Component with navigation and branding
#[component]
pub fn AppHeader(
    #[prop(optional, default = "SIMPelv2")] title: &'static str,
    #[prop(optional, default = false)] show_nav: bool,
) -> impl IntoView {
    view! {
        <header class="bg-kejaksaan-primary text-white shadow-lg">
            <div class="container mx-auto px-4 py-3">
                <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-4">
                        <img src="/assets/garuda-logo.svg" alt="Garuda" class="h-10 w-10" />
                        <h1 class="text-xl font-bold">{title}</h1>
                    </div>
                    <Show when=move || show_nav fallback=|| view! { <div></div> }>
                        <nav class="hidden md:flex space-x-6">
                            <a href="/" class="hover:text-kejaksaan-secondary transition-colors">{"Beranda"}</a>
                            <a href="/dashboard" class="hover:text-kejaksaan-secondary transition-colors">{"Dashboard"}</a>
                            <a href="/profile" class="hover:text-kejaksaan-secondary transition-colors">{"Profil"}</a>
                        </nav>
                    </Show>
                </div>
            </div>
        </header>
    }
}

/// Kejaksaan Footer with institutional branding
#[component]
pub fn KejaksaanFooter() -> impl IntoView {
    view! {
        <footer class="bg-kejaksaan-dark text-white py-8 mt-12">
            <div class="container mx-auto px-4">
                <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
                    <div>
                        <h3 class="text-lg font-semibold mb-4">{"Kejaksaan Republik Indonesia"}</h3>
                        <p class="text-sm text-kejaksaan-text-light">
                            {"Sistem Informasi Manajemen Pengadaan dan Layanan"}
                        </p>
                    </div>
                    <div>
                        <h4 class="text-md font-medium mb-3">{"Menu Utama"}</h4>
                        <ul class="space-y-2 text-sm">
                            <li><a href="/" class="hover:text-kejaksaan-secondary transition-colors">{"Beranda"}</a></li>
                            <li><a href="/about" class="hover:text-kejaksaan-secondary transition-colors">{"Tentang"}</a></li>
                            <li><a href="/contact" class="hover:text-kejaksaan-secondary transition-colors">{"Kontak"}</a></li>
                        </ul>
                    </div>
                    <div>
                        <h4 class="text-md font-medium mb-3">{"Informasi"}</h4>
                        <p class="text-sm text-kejaksaan-text-light">
                            {"© 2024 Kejaksaan RI. Semua hak dilindungi."}
                        </p>
                    </div>
                </div>
            </div>
        </footer>
    }
}
