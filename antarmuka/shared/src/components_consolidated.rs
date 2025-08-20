//! SIMPelv2 Shared Components - Consolidated & Optimized
//!
//! This file contains all shared UI components for the SIMPelv2 system,
//! optimized for Leptos 0.7.8 compatibility and modern practices.

use leptos::prelude::*;
use crate::types::*;

// ========================================
// CORE DATA STRUCTURES
// ========================================

#[derive(Clone, Debug, PartialEq)]
pub struct NavItem {
    pub path: String,
    pub label: String,
    pub icon: String,
    pub active: bool,
    pub children: Option<Vec<NavItem>>,
}

impl NavItem {
    pub fn new(path: &str, label: &str, icon: &str) -> Self {
        Self {
            path: path.to_string(),
            label: label.to_string(),
            icon: icon.to_string(),
            active: false,
            children: None,
        }
    }

    pub fn with_children(mut self, children: Vec<NavItem>) -> Self {
        self.children = Some(children);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MessageType {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LoadingState {
    Idle,
    Loading,
    Success,
    Error(String),
}

// ========================================
// KEJAKSAAN BRANDING COMPONENTS
// ========================================

#[component]
pub fn KejaksaanLogo(
    #[prop(default = 40)] size: u32,
    #[prop(default = false)] show_text: bool,
) -> impl IntoView {
    view! {
        <div class="flex items-center space-x-3">
            <div class={format!("w-{} h-{} bg-gradient-to-br from-blue-600 to-blue-800 rounded-lg flex items-center justify-center", size/4, size/4)}>
                <svg class={format!("w-{} h-{} text-white", size/8, size/8)} fill="currentColor" viewBox="0 0 20 20">
                    <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                </svg>
            </div>
            {show_text.then(|| view! {
                <div class="flex flex-col">
                    <span class="text-sm font-bold text-gray-800">"KEJAKSAAN AGUNG RI"</span>
                    <span class="text-xs text-gray-600">"SIMPelv2"</span>
                </div>
            })}
        </div>
    }
}

#[component]
pub fn KejaksaanHeader() -> impl IntoView {
    view! {
        <header class="bg-white shadow-sm border-b border-gray-200">
            <div class="container mx-auto px-4 py-3">
                <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-4">
                        <KejaksaanLogo size=40 show_text=true />
                    </div>
                    <h1 class="text-xl font-bold text-gray-900">"Portal SIMPelv2"</h1>
                </div>
            </div>
        </header>
    }
}

#[component]
pub fn KejaksaanFooter() -> impl IntoView {
    view! {
        <footer class="bg-gradient-to-r from-blue-800 to-blue-900 text-white py-6 mt-8">
            <div class="container mx-auto px-4">
                <div class="flex flex-col md:flex-row justify-between items-center">
                    <div class="text-center md:text-left mb-4 md:mb-0">
                        <p class="text-sm">"© 2025 Kejaksaan Agung Republik Indonesia"</p>
                        <p class="text-xs text-blue-200 mt-1">"Sistem Informasi Manajemen Pengelolaan Barang Milik Negara v2.0"</p>
                    </div>
                    <div class="flex items-center space-x-4">
                        <span class="text-xs text-blue-200">"Powered by Rust & Leptos 0.7.8"</span>
                    </div>
                </div>
            </div>
        </footer>
    }
}

// ========================================
// LAYOUT COMPONENTS
// ========================================

#[component]
pub fn PageLayout(children: Children) -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50">
            <KejaksaanHeader />
            <main class="container mx-auto px-4 py-6">
                {children()}
            </main>
            <KejaksaanFooter />
        </div>
    }
}

#[component]
pub fn AppHeader(
    #[prop(into)] title: String,
    #[prop(into, optional)] subtitle: Option<String>,
    children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="bg-white shadow-sm border-b border-gray-200 mb-6">
            <div class="container mx-auto px-4 py-6">
                <div class="flex items-center justify-between">
                    <div>
                        <h1 class="text-2xl font-bold text-gray-900">{title}</h1>
                        {subtitle.map(|sub| view! {
                            <p class="text-gray-600 mt-1">{sub}</p>
                        })}
                    </div>
                    <div class="flex items-center space-x-4">
                        {children.map(|child| child())}
                    </div>
                </div>
            </div>
        </div>
    }
}

// ========================================
// UI COMPONENTS
// ========================================

#[component]
pub fn LoadingSpinner(
    #[prop(into, default = 32)] size: u32,
    #[prop(into, optional)] message: Option<String>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center p-8">
            <div class={format!("animate-spin rounded-full h-{} w-{} border-b-2 border-blue-600", size, size)}></div>
            {message.map(|msg| view! {
                <p class="text-gray-600 text-sm mt-4">{msg}</p>
            })}
        </div>
    }
}

#[component]
pub fn StatusBadge(
    #[prop(into)] status: String,
    #[prop(into, optional)] variant: Option<String>,
) -> impl IntoView {
    let badge_class = move || {
        let base = "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium";
        let variant_class = match variant.as_deref().unwrap_or("default") {
            "success" => "bg-green-100 text-green-800",
            "error" => "bg-red-100 text-red-800",
            "warning" => "bg-yellow-100 text-yellow-800",
            "info" => "bg-blue-100 text-blue-800",
            _ => "bg-gray-100 text-gray-800",
        };
        format!("{} {}", base, variant_class)
    };

    view! {
        <span class=badge_class>{status}</span>
    }
}

#[component]
pub fn ActionButton(
    #[prop(into)] label: String,
    #[prop(into, optional)] variant: Option<String>,
    #[prop(into, optional)] size: Option<String>,
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(default = false)] disabled: bool,
) -> impl IntoView {
    let button_class = move || {
        let base = "inline-flex items-center justify-center font-medium rounded-md focus:outline-none focus:ring-2 focus:ring-offset-2 transition-colors";

        let variant_class = match variant.as_deref().unwrap_or("primary") {
            "primary" => "bg-blue-600 text-white hover:bg-blue-700 focus:ring-blue-500",
            "secondary" => "bg-gray-200 text-gray-900 hover:bg-gray-300 focus:ring-gray-500",
            "danger" => "bg-red-600 text-white hover:bg-red-700 focus:ring-red-500",
            "success" => "bg-green-600 text-white hover:bg-green-700 focus:ring-green-500",
            _ => "bg-blue-600 text-white hover:bg-blue-700 focus:ring-blue-500",
        };

        let size_class = match size.as_deref().unwrap_or("md") {
            "sm" => "px-3 py-1.5 text-sm",
            "md" => "px-4 py-2 text-sm",
            "lg" => "px-6 py-3 text-base",
            _ => "px-4 py-2 text-sm",
        };

        let disabled_class = if disabled { "opacity-50 cursor-not-allowed" } else { "" };

        format!("{} {} {} {}", base, variant_class, size_class, disabled_class)
    };

    view! {
        <button
            class=button_class
            disabled=disabled
            on:click=move |_| {
                if let Some(callback) = &on_click {
                    callback.call(());
                }
            }
        >
            {label}
        </button>
    }
}

#[component]
pub fn StatCard(
    #[prop(into)] title: String,
    #[prop(into)] value: String,
    #[prop(into, optional)] subtitle: Option<String>,
    #[prop(into, optional)] icon: Option<String>,
    #[prop(into, optional)] trend: Option<String>,
) -> impl IntoView {
    view! {
        <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
            <div class="flex items-center justify-between">
                <div class="flex-1">
                    <p class="text-sm font-medium text-gray-600">{title}</p>
                    <p class="text-2xl font-bold text-gray-900 mt-1">{value}</p>
                    {subtitle.map(|sub| view! {
                        <p class="text-sm text-gray-500 mt-1">{sub}</p>
                    })}
                </div>
                {icon.map(|i| view! {
                    <div class="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center">
                        <i class={format!("fas {} text-blue-600 text-xl", i)}></i>
                    </div>
                })}
            </div>
            {trend.map(|t| view! {
                <div class="mt-4 pt-4 border-t border-gray-100">
                    <p class="text-sm text-gray-600">{t}</p>
                </div>
            })}
        </div>
    }
}

// ========================================
// FORM COMPONENTS
// ========================================

#[component]
pub fn SearchBox(
    #[prop(into)] placeholder: String,
    #[prop(into, optional)] value: Option<String>,
    #[prop(optional)] on_input: Option<Callback<String>>,
    #[prop(optional)] on_submit: Option<Callback<String>>,
) -> impl IntoView {
    let (search_value, set_search_value) = signal(value.unwrap_or_default());

    view! {
        <div class="relative">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <i class="fas fa-search text-gray-400"></i>
            </div>
            <input
                type="text"
                class="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-blue-500 focus:border-blue-500"
                placeholder=placeholder
                prop:value=move || search_value.get()
                on:input=move |ev| {
                    let val = event_target_value(&ev);
                    set_search_value.set(val.clone());
                    if let Some(callback) = &on_input {
                        callback.call(val);
                    }
                }
                on:keypress=move |ev| {
                    if ev.key() == "Enter" {
                        if let Some(callback) = &on_submit {
                            callback.call(search_value.get());
                        }
                    }
                }
            />
        </div>
    }
}

#[component]
pub fn FormField(
    #[prop(into)] label: String,
    #[prop(into, optional)] error: Option<String>,
    #[prop(default = false)] required: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="space-y-1">
            <label class="block text-sm font-medium text-gray-700">
                {label}
                {required.then(|| view! {
                    <span class="text-red-500 ml-1">"*"</span>
                })}
            </label>
            {children()}
            {error.map(|err| view! {
                <p class="text-sm text-red-600">{err}</p>
            })}
        </div>
    }
}

#[component]
pub fn TextInput(
    #[prop(into)] name: String,
    #[prop(into, optional)] value: Option<String>,
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(into, optional)] input_type: Option<String>,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] on_input: Option<Callback<String>>,
) -> impl IntoView {
    let (input_value, set_input_value) = signal(value.unwrap_or_default());
    let input_type = input_type.unwrap_or_else(|| "text".to_string());

    view! {
        <input
            type=input_type
            name=name
            class="block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-blue-500 focus:border-blue-500"
            placeholder=placeholder.unwrap_or_default()
            prop:value=move || input_value.get()
            disabled=disabled
            on:input=move |ev| {
                let val = event_target_value(&ev);
                set_input_value.set(val.clone());
                if let Some(callback) = &on_input {
                    callback.call(val);
                }
            }
        />
    }
}

// ========================================
// FEEDBACK COMPONENTS
// ========================================

#[component]
pub fn AlertMessage(
    #[prop(into)] message: String,
    #[prop(into, optional)] message_type: Option<MessageType>,
    #[prop(default = true)] dismissible: bool,
    #[prop(optional)] on_dismiss: Option<Callback<()>>,
) -> impl IntoView {
    let (show, set_show) = signal(true);
    let msg_type = message_type.unwrap_or(MessageType::Info);

    let alert_class = move || {
        let base = "p-4 rounded-md border-l-4";
        match msg_type {
            MessageType::Success => format!("{} bg-green-50 border-green-400 text-green-700", base),
            MessageType::Error => format!("{} bg-red-50 border-red-400 text-red-700", base),
            MessageType::Warning => format!("{} bg-yellow-50 border-yellow-400 text-yellow-700", base),
            MessageType::Info => format!("{} bg-blue-50 border-blue-400 text-blue-700", base),
        }
    };

    let icon_class = move || {
        match msg_type {
            MessageType::Success => "fas fa-check-circle text-green-400",
            MessageType::Error => "fas fa-times-circle text-red-400",
            MessageType::Warning => "fas fa-exclamation-triangle text-yellow-400",
            MessageType::Info => "fas fa-info-circle text-blue-400",
        }
    };

    view! {
        <div class="transition-all duration-300" class:hidden=move || !show.get()>
            <div class=alert_class>
                <div class="flex items-center">
                    <div class="flex-shrink-0">
                        <i class=icon_class></i>
                    </div>
                    <div class="ml-3 flex-1">
                        <p class="text-sm">{message}</p>
                    </div>
                    {dismissible.then(|| view! {
                        <div class="ml-auto pl-3">
                            <button
                                class="inline-flex text-gray-400 hover:text-gray-600 focus:outline-none"
                                on:click=move |_| {
                                    set_show.set(false);
                                    if let Some(callback) = &on_dismiss {
                                        callback.call(());
                                    }
                                }
                            >
                                <i class="fas fa-times"></i>
                            </button>
                        </div>
                    })}
                </div>
            </div>
        </div>
    }
}

// ========================================
// NAVIGATION COMPONENTS
// ========================================

#[component]
pub fn Breadcrumb(
    #[prop(into)] items: Vec<NavItem>,
) -> impl IntoView {
    view! {
        <nav class="flex" aria-label="Breadcrumb">
            <ol class="flex items-center space-x-2">
                {items.into_iter().enumerate().map(|(index, item)| {
                    let is_last = index == items.len() - 1;
                    view! {
                        <li class="flex items-center">
                            {(!is_last).then(|| view! {
                                <a href={item.path} class="text-blue-600 hover:text-blue-800 text-sm">
                                    {item.label}
                                </a>
                                <i class="fas fa-chevron-right text-gray-400 mx-2"></i>
                            }).unwrap_or_else(|| view! {
                                <span class="text-gray-500 text-sm">{item.label}</span>
                            })}
                        </li>
                    }
                }).collect::<Vec<_>>()}
            </ol>
        </nav>
    }
}
