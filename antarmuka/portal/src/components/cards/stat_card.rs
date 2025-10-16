//! Statistics card component
//!
//! Reusable card for displaying numerical statistics

use leptos::prelude::*;

/// Statistical card properties
#[derive(Clone)]
pub struct StatCardData {
    pub title: String,
    pub value: String,
    pub icon: String,
    pub color: StatColor,
    pub trend: Option<String>,
}

/// Color variants for stat cards
#[derive(Clone, Copy)]
pub enum StatColor {
    Blue,
    Green,
    Yellow,
    Red,
    Purple,
    Indigo,
}

impl StatColor {
    fn border_class(&self) -> &'static str {
        match self {
            Self::Blue => "border-blue-500",
            Self::Green => "border-green-500",
            Self::Yellow => "border-yellow-500",
            Self::Red => "border-red-500",
            Self::Purple => "border-purple-500",
            Self::Indigo => "border-indigo-500",
        }
    }

    fn bg_class(&self) -> &'static str {
        match self {
            Self::Blue => "bg-blue-100 dark:bg-blue-900",
            Self::Green => "bg-green-100 dark:bg-green-900",
            Self::Yellow => "bg-yellow-100 dark:bg-yellow-900",
            Self::Red => "bg-red-100 dark:bg-red-900",
            Self::Purple => "bg-purple-100 dark:bg-purple-900",
            Self::Indigo => "bg-indigo-100 dark:bg-indigo-900",
        }
    }

    fn text_class(&self) -> &'static str {
        match self {
            Self::Blue => "text-blue-600 dark:text-blue-300",
            Self::Green => "text-green-600 dark:text-green-300",
            Self::Yellow => "text-yellow-600 dark:text-yellow-300",
            Self::Red => "text-red-600 dark:text-red-300",
            Self::Purple => "text-purple-600 dark:text-purple-300",
            Self::Indigo => "text-indigo-600 dark:text-indigo-300",
        }
    }
}

/// Statistics card component
#[component]
pub fn StatCard(
    /// Card data
    data: StatCardData,
) -> impl IntoView {
    let border_class = data.color.border_class();
    let bg_class = data.color.bg_class();
    let text_class = data.color.text_class();

    view! {
        <div class=format!(
            "bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border-l-4 {}",
            border_class
        )>
            <div class="flex items-center justify-between">
                <div class="flex-1">
                    <p class="text-sm font-medium text-gray-600 dark:text-gray-400 mb-1">
                        {data.title}
                    </p>
                    <p class="text-3xl font-bold text-gray-900 dark:text-white">
                        {data.value}
                    </p>
                    {data.trend.map(|trend| view! {
                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                            {trend}
                        </p>
                    })}
                </div>
                <div class=format!(
                    "p-3 rounded-lg {}",
                    bg_class
                )>
                    <span class=format!("text-2xl {}", text_class)>
                        {data.icon}
                    </span>
                </div>
            </div>
        </div>
    }
}
