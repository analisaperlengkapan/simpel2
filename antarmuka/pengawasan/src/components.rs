use crate::types::*;
use leptos::prelude::*;

/// Supervision Header Component
#[component]
pub fn SupervisionHeader(
    #[prop(into)] title: String,
    #[prop(optional)] subtitle: Option<String>,
    #[prop(optional)] actions: Option<Vec<(String, String)>>,
) -> impl IntoView {
    view! {
        <div class="bg-white border-b border-gray-200 px-6 py-4">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">{title}</h1>
                    {subtitle.map(|sub| view! {
                        <p class="text-gray-600 mt-1">{sub}</p>
                    })}
                </div>
                {actions.map(|acts| view! {
                    <div class="flex space-x-3">
                        {acts.into_iter().map(|(label, action)| view! {
                            <ActionButton
                                label=label.clone()
                                action=action.clone()
                                class="bg-kejaksaan-blue-600 hover:bg-kejaksaan-blue-700 text-white font-medium py-2 px-4 rounded-lg transition-colors".to_string()
                            />
                        }).collect::<Vec<_>>()}
                    </div>
                })}
            </div>
        </div>
    }
}

/// Primary Action Button
#[component]
pub fn ActionButton(
    #[prop(into)] label: String,
    #[prop(into)] action: String,
    #[prop(optional)] class: Option<String>,
    #[prop(optional)] variant: Option<ButtonVariant>,
) -> impl IntoView {
    let button_class = match variant.unwrap_or(ButtonVariant::Primary) {
        ButtonVariant::Primary => "bg-kejaksaan-blue-600 hover:bg-kejaksaan-blue-700 text-white",
        ButtonVariant::Secondary => "bg-gray-200 hover:bg-gray-300 text-gray-800",
        ButtonVariant::Success => "bg-green-600 hover:bg-green-700 text-white",
        ButtonVariant::Warning => "bg-yellow-600 hover:bg-yellow-700 text-white",
        ButtonVariant::Danger => "bg-red-600 hover:bg-red-700 text-white",
    };

    let final_class = class.unwrap_or_else(|| {
        format!("{button_class} font-medium py-2 px-4 rounded-lg transition-colors")
    });

    view! {
        <button
            class={final_class}
            on:click=move |_| {
                web_sys::console::log_1(&format!("Action triggered: {action}").into());
            }
        >
            {label}
        </button>
    }
}

#[derive(Clone, Debug)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Success,
    Warning,
    Danger,
}

/// Statistics Card Component
#[component]
pub fn StatsCard(
    #[prop(into)] title: String,
    #[prop(into)] value: String,
    #[prop(optional)] icon: Option<String>,
    #[prop(optional)] color: Option<String>,
    #[prop(optional)] trend: Option<String>,
    #[prop(optional)] description: Option<String>,
) -> impl IntoView {
    let color_class = match color.as_deref().unwrap_or("blue") {
        "red" => "text-red-600",
        "green" => "text-green-600",
        "yellow" => "text-yellow-600",
        "purple" => "text-purple-600",
        _ => "text-kejaksaan-blue-600",
    };

    view! {
        <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6 hover:shadow-md transition-shadow">
            <div class="flex items-center justify-between">
                <div class="flex-1">
                    <h3 class="text-sm font-medium text-gray-600 uppercase tracking-wider">{title}</h3>
                    <p class=format!("text-3xl font-bold mt-2 {}", color_class)>{value}</p>
                    {description.map(|desc| view! {
                        <p class="text-sm text-gray-500 mt-1">{desc}</p>
                    })}
                </div>
                {icon.map(|ic| view! {
                    <div class=format!("text-2xl {}", color_class)>
                        <i class={ic}></i>
                    </div>
                })}
            </div>
            {trend.map(|tr| view! {
                <div class="mt-4 pt-4 border-t border-gray-100">
                    <span class="text-xs text-gray-500">{tr}</span>
                </div>
            })}
        </div>
    }
}

/// Status Badge Component
#[component]
pub fn StatusBadge(
    #[prop(into)] status: String,
    #[prop(optional)] variant: Option<BadgeVariant>,
) -> impl IntoView {
    let badge_class = match variant.unwrap_or(BadgeVariant::Default) {
        BadgeVariant::Success => "bg-green-100 text-green-800 border-green-200",
        BadgeVariant::Warning => "bg-yellow-100 text-yellow-800 border-yellow-200",
        BadgeVariant::Danger => "bg-red-100 text-red-800 border-red-200",
        BadgeVariant::Info => "bg-blue-100 text-blue-800 border-blue-200",
        BadgeVariant::Default => "bg-gray-100 text-gray-800 border-gray-200",
    };

    view! {
        <span class=format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border {}", badge_class)>
            {status}
        </span>
    }
}

#[derive(Clone, Debug)]
pub enum BadgeVariant {
    Success,
    Warning,
    Danger,
    Info,
    Default,
}

/// Priority Badge Component
#[component]
pub fn PriorityBadge(priority: Priority) -> impl IntoView {
    let (label, variant) = match priority {
        Priority::Critical => ("Kritis", BadgeVariant::Danger),
        Priority::High => ("Tinggi", BadgeVariant::Warning),
        Priority::Medium => ("Sedang", BadgeVariant::Info),
        Priority::Low => ("Rendah", BadgeVariant::Success),
    };

    view! {
        <StatusBadge status=label.to_string() variant=variant />
    }
}

/// Search Input Component
#[component]
pub fn SearchInput(
    #[prop(into)] placeholder: String,
    #[prop(optional)] on_search: Option<leptos::callback::Callback<String>>,
) -> impl IntoView {
    let (search_value, set_search_value) = signal("".to_string());

    view! {
        <div class="relative max-w-md">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <i class="fas fa-search text-gray-400"></i>
            </div>
            <input
                type="text"
                class="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-lg leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-2 focus:ring-kejaksaan-blue-500 focus:border-kejaksaan-blue-500"
                placeholder={placeholder}
                on:input=move |ev| {
                    let val = event_target_value(&ev);
                    set_search_value.set(val.clone());
                    if let Some(callback) = on_search {
                        callback.run(val);
                    }
                }
                prop:value=search_value
            />
        </div>
    }
}

/// Progress Bar Component
#[component]
pub fn ProgressBar(
    #[prop(into)] progress: u8,
    #[prop(optional)] label: Option<String>,
    #[prop(optional)] color: Option<String>,
) -> impl IntoView {
    let color_class = match color.as_deref().unwrap_or("blue") {
        "green" => "bg-green-500",
        "yellow" => "bg-yellow-500",
        "red" => "bg-red-500",
        _ => "bg-kejaksaan-blue-500",
    };

    view! {
        <div class="w-full">
            {label.map(|lbl| view! {
                <div class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                    <span>{lbl}</span>
                    <span>{progress}"%"</span>
                </div>
            })}
            <div class="w-full bg-gray-200 rounded-full h-2">
                <div
                    class=format!("h-2 rounded-full transition-all duration-300 {}", color_class)
                    style=format!("width: {}%", progress)
                ></div>
            </div>
        </div>
    }
}

/// Empty State Component
#[component]
pub fn EmptyState(
    #[prop(into)] title: String,
    #[prop(into)] description: String,
    #[prop(optional)] icon: Option<String>,
    #[prop(optional)] action: Option<(String, String)>,
) -> impl IntoView {
    view! {
        <div class="text-center py-12">
            {icon.map(|ic| view! {
                <div class="text-4xl text-gray-400 mb-4">
                    <i class={ic}></i>
                </div>
            })}
            <h3 class="text-lg font-medium text-gray-900 mb-2">{title}</h3>
            <p class="text-gray-500 max-w-sm mx-auto mb-6">{description}</p>
            {action.map(|(label, act)| view! {
                <ActionButton
                    label=label
                    action=act
                    variant=ButtonVariant::Primary
                />
            })}
        </div>
    }
}

/// Loading Spinner Component
#[component]
pub fn LoadingSpinner(
    #[prop(optional)] size: Option<String>,
    #[prop(optional)] message: Option<String>,
) -> impl IntoView {
    let size_class = match size.as_deref().unwrap_or("md") {
        "sm" => "h-4 w-4",
        "lg" => "h-8 w-8",
        "xl" => "h-12 w-12",
        _ => "h-6 w-6",
    };

    view! {
        <div class="flex flex-col items-center justify-center p-8">
            <div class=format!("animate-spin rounded-full border-2 border-gray-200 border-t-kejaksaan-blue-600 {}", size_class)></div>
            {message.map(|msg| view! {
                <p class="text-gray-600 mt-3 text-sm">{msg}</p>
            })}
        </div>
    }
}

/// Alert Component
#[component]
pub fn Alert(
    #[prop(into)] message: String,
    #[prop(optional)] variant: Option<AlertVariant>,
    #[prop(optional)] dismissible: Option<bool>,
) -> impl IntoView {
    let (bg_class, text_class, icon) = match variant.unwrap_or(AlertVariant::Info) {
        AlertVariant::Success => (
            "bg-green-50 border-green-200",
            "text-green-800",
            "fas fa-check-circle",
        ),
        AlertVariant::Warning => (
            "bg-yellow-50 border-yellow-200",
            "text-yellow-800",
            "fas fa-exclamation-triangle",
        ),
        AlertVariant::Error => (
            "bg-red-50 border-red-200",
            "text-red-800",
            "fas fa-exclamation-circle",
        ),
        AlertVariant::Info => (
            "bg-blue-50 border-blue-200",
            "text-blue-800",
            "fas fa-info-circle",
        ),
    };

    let (show_alert, set_show_alert) = signal(true);

    view! {
        <div class=format!("rounded-md border p-4 {} {}", bg_class, if show_alert.get() { "" } else { "hidden" })>
            <div class="flex">
                <div class="flex-shrink-0">
                    <i class=format!("{} {}", icon, text_class)></i>
                </div>
                <div class="ml-3 flex-1">
                    <p class=format!("text-sm {}", text_class)>{message}</p>
                </div>
                {dismissible.unwrap_or(false).then(|| view! {
                    <div class="ml-auto pl-3">
                        <button
                            class=format!("inline-flex rounded-md p-1.5 focus:outline-none focus:ring-2 focus:ring-offset-2 {}", text_class)
                            on:click=move |_| set_show_alert.set(false)
                        >
                            <i class="fas fa-times text-sm"></i>
                        </button>
                    </div>
                })}
            </div>
        </div>
    }
}

#[derive(Clone, Debug)]
pub enum AlertVariant {
    Success,
    Warning,
    Error,
    Info,
}
