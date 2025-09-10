use crate::types::*;
use leptos::prelude::*;

/// Military Case Header Component
#[component]
pub fn MilitaryHeader(
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
                                class="bg-red-600 hover:bg-red-700 text-white font-medium py-2 px-4 rounded-lg transition-colors".to_string()
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
        ButtonVariant::Primary => "bg-red-600 hover:bg-red-700 text-white",
        ButtonVariant::Secondary => "bg-gray-200 hover:bg-gray-300 text-gray-800",
        ButtonVariant::Success => "bg-green-600 hover:bg-green-700 text-white",
        ButtonVariant::Warning => "bg-yellow-600 hover:bg-yellow-700 text-white",
        ButtonVariant::Danger => "bg-red-600 hover:bg-red-700 text-white",
    };

    let base_class = "font-medium py-2 px-4 rounded-lg transition-colors";
    let final_class = if let Some(custom_class) = class {
        format!("{} {}", base_class, custom_class)
    } else {
        format!("{} {}", base_class, button_class)
    };

    view! {
        <button class=final_class on:click=move |_| { log::info!("Action: {}", action) }>
            {label}
        </button>
    }
}

/// Statistics Card Component
#[component]
pub fn StatsCard(
    #[prop(into)] title: String,
    #[prop(into)] value: String,
    #[prop(into)] icon: String,
    #[prop(into)] color: String,
    #[prop(into)] description: String,
) -> impl IntoView {
    let icon_bg = match color.as_str() {
        "red" => "bg-red-100",
        "blue" => "bg-blue-100",
        "green" => "bg-green-100",
        "yellow" => "bg-yellow-100",
        "purple" => "bg-purple-100",
        _ => "bg-gray-100",
    };

    let icon_color = match color.as_str() {
        "red" => "text-red-600",
        "blue" => "text-blue-600",
        "green" => "text-green-600",
        "yellow" => "text-yellow-600",
        "purple" => "text-purple-600",
        _ => "text-gray-600",
    };

    view! {
        <div class="bg-white rounded-lg shadow p-6">
            <div class="flex items-center">
                <div class={format!("{} p-3 rounded-full", icon_bg)}>
                    <i class={format!("fas {} {}", icon, icon_color)}></i>
                </div>
                <div class="ml-4">
                    <p class="text-sm font-medium text-gray-600">{title}</p>
                    <p class="text-2xl font-bold text-gray-900">{value}</p>
                    <p class="text-xs text-gray-500">{description}</p>
                </div>
            </div>
        </div>
    }
}

/// Case Card Component
#[component]
pub fn CaseCard(case: MilitaryCase) -> impl IntoView {
    let status_badge = match case.status {
        CaseStatus::Reported => ("bg-blue-100 text-blue-800", "Dilaporkan"),
        CaseStatus::UnderInvestigation => ("bg-yellow-100 text-yellow-800", "Dalam Penyidikan"),
        CaseStatus::EvidenceCollection => ("bg-orange-100 text-orange-800", "Pengumpulan Bukti"),
        CaseStatus::SuspectIdentified => ("bg-purple-100 text-purple-800", "Tersangka Teridentifikasi"),
        CaseStatus::AwaitingTrial => ("bg-indigo-100 text-indigo-800", "Menunggu Sidang"),
        CaseStatus::InTrial => ("bg-pink-100 text-pink-800", "Dalam Sidang"),
        CaseStatus::Concluded => ("bg-green-100 text-green-800", "Selesai"),
        CaseStatus::Dismissed => ("bg-gray-100 text-gray-800", "Dibatalkan"),
        CaseStatus::Appealed => ("bg-red-100 text-red-800", "Banding"),
    };

    let priority_color = match case.priority {
        CasePriority::Low => "border-l-green-500",
        CasePriority::Medium => "border-l-yellow-500",
        CasePriority::High => "border-l-red-500",
        CasePriority::Critical => "border-l-red-700",
        CasePriority::TopSecret => "border-l-black",
    };

    view! {
        <div class={format!("bg-white rounded-lg shadow border-l-4 {} p-6 hover:shadow-lg transition-shadow", priority_color)}>
            <div class="flex items-start justify-between">
                <div class="flex-1">
                    <div class="flex items-center space-x-3 mb-2">
                        <h3 class="text-lg font-semibold text-gray-900">{case.title}</h3>
                        <span class={format!("px-2 py-1 rounded-full text-xs font-medium {}", status_badge.0)}>
                            {status_badge.1}
                        </span>
                    </div>
                    <p class="text-sm text-gray-600 mb-3">{case.description}</p>
                    <div class="grid grid-cols-2 gap-4 text-sm">
                        <div>
                            <span class="font-medium text-gray-500">"Nomor Kasus: "</span>
                            <span class="text-gray-900">{case.case_number}</span>
                        </div>
                        <div>
                            <span class="font-medium text-gray-500">"Penyidik: "</span>
                            <span class="text-gray-900">{case.assigned_investigator}</span>
                        </div>
                        <div>
                            <span class="font-medium text-gray-500">"Satuan: "</span>
                            <span class="text-gray-900">{case.unit_involved}</span>
                        </div>
                        <div>
                            <span class="font-medium text-gray-500">"Lokasi: "</span>
                            <span class="text-gray-900">{case.location}</span>
                        </div>
                    </div>
                </div>
                <div class="ml-4 text-right">
                    <div class="text-sm text-gray-500 mb-1">"Suspek"</div>
                    <div class="text-lg font-bold text-gray-900">{case.suspects_count}</div>
                    <div class="text-sm text-gray-500 mb-1">"Bukti"</div>
                    <div class="text-lg font-bold text-gray-900">{case.evidence_count}</div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn SearchBox(#[prop(into)] on_search: Callback<String>) -> impl IntoView {
    let (search_value, set_search_value) = signal(String::new());

    let handle_search = move |_| {
        on_search.run(search_value.get());
    };

    let handle_input = move |ev| {
        let value = event_target_value(&ev);
        set_search_value.set(value);
    };

    view! {
        <div class="search-box">
            <input
                type="text"
                placeholder="Cari..."
                value=move || search_value.get()
                on:input=handle_input
            />
            <button class="search-btn" on:click=handle_search>
                <i class="fas fa-search"></i>
            </button>
        </div>
    }
}

#[component]
pub fn StatusBadge(#[prop(into)] status: String) -> impl IntoView {
    let badge_class = match status.as_str() {
        "Tersedia" => "status-badge status-tersedia",
        "Habis" => "status-badge status-habis",
        "Perbaikan" => "status-badge status-perbaikan",
        _ => "status-badge status-tersedia",
    };

    view! {
        <span class=badge_class>{status}</span>
    }
}

#[component]
pub fn FormGroup(
    #[prop(into)] label: String,
    #[prop(into)] input_type: &'static str,
    #[prop(into)] placeholder: String,
    #[prop(into)] value: Signal<String>,
    #[prop(into)] on_change: Callback<String>
) -> impl IntoView {
    let handle_input = move |ev| {
        let value = event_target_value(&ev);
        on_change.run(value);
    };

    view! {
        <div class="form-group">
            <label>{label}</label>
            <input
                type=input_type
                placeholder=placeholder
                value=move || value.get()
                on:input=handle_input
            />
        </div>
    }
}

#[component]
pub fn FormSelect(
    #[prop(into)] label: String,
    #[prop(into)] options: Vec<(String, String)>,
    #[prop(into)] value: Signal<String>,
    #[prop(into)] on_change: Callback<String>
) -> impl IntoView {
    let handle_change = move |ev| {
        let value = event_target_value(&ev);
        on_change.run(value);
    };

    view! {
        <div class="form-group">
            <label>{label}</label>
            <select on:change=handle_change>
                <option value="">"Pilih..."</option>
                {options.into_iter().map(|(opt_value, text)| {
                    let opt_value_clone = opt_value.clone();
                    let is_selected = move || value.get() == opt_value_clone;
                    view! {
                        <option value=opt_value selected=is_selected()>{text}</option>
                    }
                }).collect::<Vec<_>>()}
            </select>
        </div>
    }
}
