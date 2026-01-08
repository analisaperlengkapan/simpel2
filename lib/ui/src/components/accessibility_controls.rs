//! Accessibility control components for user preferences

use crate::hooks::use_storage;
use crate::utils::accessibility::*;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

// ============================================================================
// FONT SIZE CONTROL
// ============================================================================

/// Font size options
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FontSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

impl FontSize {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
            Self::ExtraLarge => "extra-large",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "small" => Self::Small,
            "large" => Self::Large,
            "extra-large" => Self::ExtraLarge,
            _ => Self::Medium,
        }
    }

    pub fn scale_factor(&self) -> f32 {
        match self {
            Self::Small => 0.875,
            Self::Medium => 1.0,
            Self::Large => 1.125,
            Self::ExtraLarge => 1.25,
        }
    }
}

/// Font size control component
#[component]
pub fn FontSizeControl(#[prop(optional, into)] class: Option<String>) -> impl IntoView {
    let class = class.unwrap_or_default();
    let (font_size, set_font_size) = use_storage::<String>("font-size", "medium".to_string());

    let current_size = move || FontSize::from_str(&font_size.get());

    // Apply font size to document root
    Effect::new(move |_| {
        if let Some(document) = web_sys::window().and_then(|w| w.document())
            && let Some(root) = document.document_element() {
                let scale = current_size().scale_factor();
                let _ = root.dyn_ref::<web_sys::HtmlElement>().map(|el| {
                    el.style()
                        .set_property("font-size", &format!("{}rem", scale))
                });
            }
    });

    let handle_change = move |size: FontSize| {
        set_font_size.set(size.as_str().to_string());
    };

    view! {
        <div class=format!("space-y-2 {}", class)>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                "Font Size"
            </label>
            <div class="flex items-center space-x-2" role="group" aria-label="Font size controls">
                <button
                    type="button"
                    class=move || format!(
                        "px-3 py-2 text-sm rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 {}",
                        if current_size() == FontSize::Small {
                            "bg-primary-600 text-white"
                        } else {
                            "bg-gray-200 text-gray-700 hover:bg-gray-300"
                        }
                    )
                    aria-label="Small font size"
                    aria-pressed=move || (current_size() == FontSize::Small).to_string()
                    on:click=move |_| handle_change(FontSize::Small)
                >
                    "A"
                </button>
                <button
                    type="button"
                    class=move || format!(
                        "px-3 py-2 text-base rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 {}",
                        if current_size() == FontSize::Medium {
                            "bg-primary-600 text-white"
                        } else {
                            "bg-gray-200 text-gray-700 hover:bg-gray-300"
                        }
                    )
                    aria-label="Medium font size"
                    aria-pressed=move || (current_size() == FontSize::Medium).to_string()
                    on:click=move |_| handle_change(FontSize::Medium)
                >
                    "A"
                </button>
                <button
                    type="button"
                    class=move || format!(
                        "px-3 py-2 text-lg rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 {}",
                        if current_size() == FontSize::Large {
                            "bg-primary-600 text-white"
                        } else {
                            "bg-gray-200 text-gray-700 hover:bg-gray-300"
                        }
                    )
                    aria-label="Large font size"
                    aria-pressed=move || (current_size() == FontSize::Large).to_string()
                    on:click=move |_| handle_change(FontSize::Large)
                >
                    "A"
                </button>
                <button
                    type="button"
                    class=move || format!(
                        "px-3 py-2 text-xl rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 {}",
                        if current_size() == FontSize::ExtraLarge {
                            "bg-primary-600 text-white"
                        } else {
                            "bg-gray-200 text-gray-700 hover:bg-gray-300"
                        }
                    )
                    aria-label="Extra large font size"
                    aria-pressed=move || (current_size() == FontSize::ExtraLarge).to_string()
                    on:click=move |_| handle_change(FontSize::ExtraLarge)
                >
                    "A"
                </button>
            </div>
        </div>
    }
}

// ============================================================================
// HIGH CONTRAST MODE CONTROL
// ============================================================================

/// High contrast mode control component
#[component]
pub fn HighContrastControl(#[prop(optional, into)] class: Option<String>) -> impl IntoView {
    let class = class.unwrap_or_default();
    let (high_contrast, set_high_contrast) = use_storage::<bool>("high-contrast", false);

    // Apply high contrast mode to document
    Effect::new(move |_| {
        if let Some(document) = web_sys::window().and_then(|w| w.document())
            && let Some(root) = document.document_element() {
                if high_contrast.get() {
                    let _ = root.class_list().add_1("high-contrast");
                } else {
                    let _ = root.class_list().remove_1("high-contrast");
                }
            }
    });

    let toggle = move |_| {
        set_high_contrast.set(!high_contrast.get());
    };

    view! {
        <div class=format!("flex items-center justify-between {}", class)>
            <label for="high-contrast-toggle" class="text-sm font-medium text-gray-700 dark:text-gray-300">
                "High Contrast Mode"
            </label>
            <button
                type="button"
                id="high-contrast-toggle"
                role="switch"
                aria-checked=move || high_contrast.get().to_string()
                class=move || format!(
                    "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 {}",
                    if high_contrast.get() {
                        "bg-primary-600"
                    } else {
                        "bg-gray-200"
                    }
                )
                on:click=toggle
            >
                <span class="sr-only">"Toggle high contrast mode"</span>
                <span
                    class=move || format!(
                        "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {}",
                        if high_contrast.get() {
                            "translate-x-5"
                        } else {
                            "translate-x-0"
                        }
                    )
                ></span>
            </button>
        </div>
    }
}

// ============================================================================
// REDUCED MOTION CONTROL
// ============================================================================

/// Reduced motion control component
#[component]
pub fn ReducedMotionControl(#[prop(optional, into)] class: Option<String>) -> impl IntoView {
    let class = class.unwrap_or_default();
    let (reduced_motion, set_reduced_motion) =
        use_storage::<bool>("reduced-motion", prefers_reduced_motion());

    // Apply reduced motion preference to document
    Effect::new(move |_| {
        if let Some(document) = web_sys::window().and_then(|w| w.document())
            && let Some(root) = document.document_element() {
                if reduced_motion.get() {
                    let _ = root.class_list().add_1("reduce-motion");
                } else {
                    let _ = root.class_list().remove_1("reduce-motion");
                }
            }
    });

    let toggle = move |_| {
        set_reduced_motion.set(!reduced_motion.get());
    };

    view! {
        <div class=format!("flex items-center justify-between {}", class)>
            <label for="reduced-motion-toggle" class="text-sm font-medium text-gray-700 dark:text-gray-300">
                "Reduce Motion"
            </label>
            <button
                type="button"
                id="reduced-motion-toggle"
                role="switch"
                aria-checked=move || reduced_motion.get().to_string()
                class=move || format!(
                    "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 {}",
                    if reduced_motion.get() {
                        "bg-primary-600"
                    } else {
                        "bg-gray-200"
                    }
                )
                on:click=toggle
            >
                <span class="sr-only">"Toggle reduced motion"</span>
                <span
                    class=move || format!(
                        "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {}",
                        if reduced_motion.get() {
                            "translate-x-5"
                        } else {
                            "translate-x-0"
                        }
                    )
                ></span>
            </button>
        </div>
    }
}

// ============================================================================
// FOCUS INDICATORS CONTROL
// ============================================================================

/// Focus indicators control component
#[component]
pub fn FocusIndicatorsControl(#[prop(optional, into)] class: Option<String>) -> impl IntoView {
    let class = class.unwrap_or_default();
    let (enhanced_focus, set_enhanced_focus) = use_storage::<bool>("enhanced-focus", false);

    // Apply enhanced focus indicators to document
    Effect::new(move |_| {
        if let Some(document) = web_sys::window().and_then(|w| w.document())
            && let Some(root) = document.document_element() {
                if enhanced_focus.get() {
                    let _ = root.class_list().add_1("enhanced-focus");
                } else {
                    let _ = root.class_list().remove_1("enhanced-focus");
                }
            }
    });

    let toggle = move |_| {
        set_enhanced_focus.set(!enhanced_focus.get());
    };

    view! {
        <div class=format!("flex items-center justify-between {}", class)>
            <label for="focus-indicators-toggle" class="text-sm font-medium text-gray-700 dark:text-gray-300">
                "Enhanced Focus Indicators"
            </label>
            <button
                type="button"
                id="focus-indicators-toggle"
                role="switch"
                aria-checked=move || enhanced_focus.get().to_string()
                class=move || format!(
                    "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 {}",
                    if enhanced_focus.get() {
                        "bg-primary-600"
                    } else {
                        "bg-gray-200"
                    }
                )
                on:click=toggle
            >
                <span class="sr-only">"Toggle enhanced focus indicators"</span>
                <span
                    class=move || format!(
                        "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {}",
                        if enhanced_focus.get() {
                            "translate-x-5"
                        } else {
                            "translate-x-0"
                        }
                    )
                ></span>
            </button>
        </div>
    }
}

// ============================================================================
// ACCESSIBILITY SETTINGS PANEL
// ============================================================================

/// Complete accessibility settings panel with all controls
#[component]
pub fn AccessibilitySettingsPanel(#[prop(optional, into)] class: Option<String>) -> impl IntoView {
    let class = class.unwrap_or_default();

    view! {
        <div class=format!("space-y-6 p-6 bg-white dark:bg-gray-800 rounded-lg shadow-md {}", class)>
            <div>
                <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100 mb-4">
                    "Accessibility Settings"
                </h2>
                <p class="text-sm text-gray-600 dark:text-gray-400 mb-6">
                    "Customize your experience to meet your accessibility needs"
                </p>
            </div>

            <div class="space-y-4">
                <FontSizeControl />
                <hr class="border-gray-200 dark:border-gray-700" />
                <HighContrastControl />
                <hr class="border-gray-200 dark:border-gray-700" />
                <ReducedMotionControl />
                <hr class="border-gray-200 dark:border-gray-700" />
                <FocusIndicatorsControl />
            </div>

            <div class="pt-4 border-t border-gray-200 dark:border-gray-700">
                <p class="text-xs text-gray-500 dark:text-gray-400">
                    "These settings are saved locally and will persist across sessions"
                </p>
            </div>
        </div>
    }
}

// ============================================================================
// ACCESSIBILITY MENU BUTTON
// ============================================================================

/// Floating accessibility menu button
#[component]
pub fn AccessibilityMenuButton() -> impl IntoView {
    let (show_panel, set_show_panel) = signal(false);

    let toggle_panel = move |_| {
        set_show_panel.set(!show_panel.get());
    };

    view! {
        <div class="fixed bottom-4 right-4 z-50">
            <button
                type="button"
                class="p-3 bg-primary-600 text-white rounded-full shadow-lg hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 transition-colors"
                aria-label="Open accessibility settings"
                aria-expanded=move || show_panel.get().to_string()
                on:click=toggle_panel
            >
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4"></path>
                </svg>
            </button>

            <Show when=move || show_panel.get()>
                <div class="absolute bottom-16 right-0 w-96 max-w-[calc(100vw-2rem)]">
                    <AccessibilitySettingsPanel />
                    <button
                        type="button"
                        class="absolute top-2 right-2 p-2 text-gray-400 hover:text-gray-600 focus:outline-none focus:ring-2 focus:ring-primary-500 rounded"
                        aria-label="Close accessibility settings"
                        on:click=move |_| set_show_panel.set(false)
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                        </svg>
                    </button>
                </div>
            </Show>
        </div>
    }
}
