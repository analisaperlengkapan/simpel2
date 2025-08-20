use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// === Data Structures ===

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategicPlan {
    pub id: String,
    pub name: String,
    pub description: String,
    pub timeline: PlanTimeline,
    pub budget: BudgetPlan,
    pub priority: PlanPriority,
    pub status: PlanStatus,
    pub progress: f32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BudgetPlan {
    pub total_budget: f64,
    pub allocated_budget: f64,
    pub utilized_budget: f64,
    pub remaining_budget: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanTimeline {
    pub start_date: String,
    pub end_date: String,
    pub duration_months: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlanPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlanStatus {
    Draft,
    Approved,
    InProgress,
    OnHold,
    Completed,
    Cancelled,
}

// === UI Components ===

#[component]
pub fn PlanningHeader(
    #[prop(into)] title: String,
    #[prop(into)] subtitle: String,
) -> impl IntoView {
    view! {
        <div class="bg-gradient-to-r from-blue-600 to-purple-600 text-white p-6 rounded-lg shadow-lg">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold">{title}</h1>
                    <p class="text-blue-100 mt-2">{subtitle}</p>
                </div>
                <div class="text-6xl opacity-20">
                    "📋"
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn StatsCard(
    #[prop(into)] title: String,
    #[prop(into)] value: String,
    #[prop(into)] change: String,
    #[prop(into)] icon: String,
    #[prop(default = false)] positive: bool,
) -> impl IntoView {
    let change_color = if positive {
        "text-green-600"
    } else {
        "text-red-600"
    };

    view! {
        <div class="bg-white p-6 rounded-xl shadow-lg border border-gray-200 hover:shadow-xl transition-all duration-300">
            <div class="flex items-center justify-between">
                <div>
                    <h3 class="text-gray-600 text-sm font-medium mb-2">{title}</h3>
                    <p class="text-3xl font-bold text-gray-900">{value}</p>
                    <span class={format!("text-sm font-medium {}", change_color)}>{change}</span>
                </div>
                <div class="text-4xl">{icon}</div>
            </div>
        </div>
    }
}

#[component]
pub fn StatusBadge(#[prop(into)] status: PlanStatus) -> impl IntoView {
    let (class, text) = match status {
        PlanStatus::Draft => ("bg-gray-100 text-gray-800", "Draft"),
        PlanStatus::Approved => ("bg-blue-100 text-blue-800", "Disetujui"),
        PlanStatus::InProgress => ("bg-yellow-100 text-yellow-800", "Berjalan"),
        PlanStatus::OnHold => ("bg-orange-100 text-orange-800", "Ditunda"),
        PlanStatus::Completed => ("bg-green-100 text-green-800", "Selesai"),
        PlanStatus::Cancelled => ("bg-red-100 text-red-800", "Dibatalkan"),
    };

    view! {
        <span class={format!("px-3 py-1 rounded-full text-sm font-medium {}", class)}>
            {text}
        </span>
    }
}

#[component]
pub fn PriorityBadge(#[prop(into)] priority: PlanPriority) -> impl IntoView {
    let (class, text) = match priority {
        PlanPriority::Critical => ("bg-red-100 text-red-800", "Kritis"),
        PlanPriority::High => ("bg-orange-100 text-orange-800", "Tinggi"),
        PlanPriority::Medium => ("bg-yellow-100 text-yellow-800", "Sedang"),
        PlanPriority::Low => ("bg-blue-100 text-blue-800", "Rendah"),
    };

    view! {
        <span class={format!("px-3 py-1 rounded-full text-sm font-medium {}", class)}>
            {text}
        </span>
    }
}

#[component]
pub fn ProgressBar(#[prop(into)] progress: f32) -> impl IntoView {
    let width = format!("{}%", progress);
    let color_class = if progress >= 80.0 {
        "bg-green-500"
    } else if progress >= 60.0 {
        "bg-yellow-500"
    } else {
        "bg-red-500"
    };

    view! {
        <div class="w-full bg-gray-200 rounded-full h-3">
            <div
                class={format!("h-3 rounded-full transition-all duration-300 {}", color_class)}
                style={format!("width: {}", width)}
            ></div>
        </div>
    }
}

#[component]
pub fn SearchInput(
    #[prop(into)] placeholder: String,
) -> impl IntoView {
    view! {
        <div class="relative">
            <input
                type="text"
                placeholder={placeholder}
                class="w-full px-4 py-3 pl-12 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <div class="absolute left-4 top-1/2 transform -translate-y-1/2 text-gray-400">
                "🔍"
            </div>
        </div>
    }
}

#[component]
pub fn EmptyState(#[prop(into)] message: String) -> impl IntoView {
    view! {
        <div class="text-center py-12">
            <div class="text-6xl mb-4">"📋"</div>
            <h3 class="text-lg font-medium text-gray-900 mb-2">"Belum ada data"</h3>
            <p class="text-gray-600">{message}</p>
        </div>
    }
}
