//! Logo component for SIMPEL applications

use leptos::prelude::*;

/// Logo size variants
#[derive(Clone, PartialEq)]
pub enum LogoSize {
    Small,
    Medium,
    Large,
}

impl LogoSize {
    fn to_classes(&self) -> &'static str {
        match self {
            LogoSize::Small => "h-8 w-8",
            LogoSize::Medium => "h-12 w-12",
            LogoSize::Large => "h-16 w-16",
        }
    }
}

/// Logo component for SIMPEL applications
#[component]
pub fn Logo(
    /// Size of the logo
    #[prop(default = LogoSize::Medium)]
    size: LogoSize,
    /// Whether to show text alongside logo
    #[prop(default = true)]
    show_text: bool,
    /// Custom CSS classes
    #[prop(optional)]
    class: Option<String>,
) -> impl IntoView {
    let logo_classes = format!("{} {}", size.to_classes(), class.unwrap_or_default());

    view! {
        <div class="flex items-center space-x-3">
            <div class=format!("flex-shrink-0 {}", logo_classes)>
                <svg
                    viewBox="0 0 100 100"
                    class="w-full h-full text-blue-600"
                    fill="currentColor"
                >
                    // Simplified Indonesian government emblem-inspired design
                    <circle cx="50" cy="50" r="45" fill="none" stroke="currentColor" stroke-width="3"/>
                    <circle cx="50" cy="50" r="35" fill="none" stroke="currentColor" stroke-width="2"/>

                    // Central star (Pancasila symbol)
                    <polygon
                        points="50,20 55,35 70,35 58,45 63,60 50,50 37,60 42,45 30,35 45,35"
                        fill="currentColor"
                    />

                    // Text "RI" at bottom
                    <text x="50" y="80" text-anchor="middle" class="text-xs font-bold" fill="currentColor">
                        RI
                    </text>
                </svg>
            </div>

            <Show when=move || show_text>
                <div class="flex flex-col">
                    <span class="text-lg font-bold text-gray-900 dark:text-white">
                        SIMPEL
                    </span>
                    <span class="text-sm text-gray-600 dark:text-gray-300">
                        Kejaksaan RI
                    </span>
                </div>
            </Show>
        </div>
    }
}
