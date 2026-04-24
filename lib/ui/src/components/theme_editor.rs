//! Theme Editor Component
//!
//! Advanced UI untuk customizing theme colors, preview real-time, dan save preferences.

use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_use::storage::use_local_storage;
use serde::{Deserialize, Serialize};
use web_sys::window;

use crate::core::theme::{ThemeMode, apply_theme};

// ============================================================================
// THEME CONFIGURATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeConfig {
    pub mode: ThemeMode,
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub success_color: String,
    pub warning_color: String,
    pub error_color: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            mode: ThemeMode::Light,
            primary_color: "#DC2626".to_string(), // Kejaksaan Red
            secondary_color: "#1E40AF".to_string(), // Government Blue
            accent_color: "#F59E0B".to_string(),  // Gold
            success_color: "#10B981".to_string(),
            warning_color: "#F59E0B".to_string(),
            error_color: "#EF4444".to_string(),
        }
    }
}

impl ThemeConfig {
    /// Apply this theme configuration to the document
    pub fn apply(&self) {
        apply_theme(self.mode);
        self.apply_custom_colors();
    }

    /// Apply custom colors as CSS variables
    fn apply_custom_colors(&self) {
        let window = match window() {
            Some(w) => w,
            None => return,
        };

        let document = match window.document() {
            Some(d) => d,
            None => return,
        };

        let html = match document.document_element() {
            Some(e) => e,
            None => return,
        };

        // Set CSS custom properties via setAttribute
        let _ = html.set_attribute("style", &format!(
            "--color-primary: {}; --color-secondary: {}; --color-accent: {}; --color-success: {}; --color-warning: {}; --color-error: {};",
            self.primary_color, self.secondary_color, self.accent_color,
            self.success_color, self.warning_color, self.error_color
        ));
    }

    /// Validate color contrast for accessibility (WCAG AA)
    pub fn validate_contrast(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check primary color contrast
        if !has_sufficient_contrast(&self.primary_color, "#FFFFFF") {
            warnings
                .push("Primary color may not have sufficient contrast with white text".to_string());
        }

        // Check secondary color contrast
        if !has_sufficient_contrast(&self.secondary_color, "#FFFFFF") {
            warnings.push(
                "Secondary color may not have sufficient contrast with white text".to_string(),
            );
        }

        warnings
    }
}

/// Check if two colors have sufficient contrast (WCAG AA: 4.5:1 for normal text)
fn has_sufficient_contrast(color1: &str, color2: &str) -> bool {
    // Simplified contrast check - in production, use proper color contrast calculation
    let l1 = calculate_relative_luminance(color1);
    let l2 = calculate_relative_luminance(color2);

    let contrast = if l1 > l2 {
        (l1 + 0.05) / (l2 + 0.05)
    } else {
        (l2 + 0.05) / (l1 + 0.05)
    };

    contrast >= 4.5
}

/// Calculate relative luminance of a color (simplified)
fn calculate_relative_luminance(hex: &str) -> f64 {
    // Remove # if present
    let hex = hex.trim_start_matches('#');

    // Parse RGB values
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64 / 255.0;

    // Apply gamma correction
    let r = if r <= 0.03928 {
        r / 12.92
    } else {
        ((r + 0.055) / 1.055).powf(2.4)
    };
    let g = if g <= 0.03928 {
        g / 12.92
    } else {
        ((g + 0.055) / 1.055).powf(2.4)
    };
    let b = if b <= 0.03928 {
        b / 12.92
    } else {
        ((b + 0.055) / 1.055).powf(2.4)
    };

    // Calculate luminance
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

// ============================================================================
// THEME EDITOR COMPONENT
// ============================================================================

#[component]
pub fn ThemeEditor(#[prop(optional)] on_close: Option<Callback<()>>) -> impl IntoView {
    // Load saved theme config or use default
    let (config, set_config, _) =
        use_local_storage::<ThemeConfig, JsonSerdeCodec>("simpelv2_theme_config");

    // Snapshot of the persisted config at editor open time, used by `cancel`
    // to restore user-visible state. We can't rely on `config.get()` in cancel
    // because `set_config.update(...)` from the preview handlers persists to
    // localStorage immediately, overwriting the "saved" value.
    let original_config = StoredValue::new(config.get_untracked());

    // Preview mode - apply changes temporarily
    let (preview_mode, set_preview_mode) = signal(false);

    // Validation warnings
    let warnings = Memo::new(move |_| config.get().validate_contrast());

    // Apply theme when config changes in preview mode
    Effect::new(move || {
        if preview_mode.get() {
            config.get().apply();
        }
    });

    // Save theme configuration. The signal value is already persisted to
    // localStorage by `use_local_storage` on every preview update, so here we
    // only need to apply visually and close. We also refresh the snapshot so a
    // subsequent cancel wouldn't revert past this save point.
    let save_theme = move |_| {
        let current_config = config.get();
        current_config.apply();
        original_config.set_value(current_config);
        set_preview_mode.set(false);

        if let Some(on_close) = on_close {
            on_close.run(());
        }
    };

    // Reset to default
    let reset_theme = move |_| {
        let default_config = ThemeConfig::default();
        set_config.set(default_config.clone());
        default_config.apply();
        set_preview_mode.set(false);
    };

    // Cancel and revert changes to the snapshot captured on editor open.
    let cancel = move |_| {
        let saved = original_config.get_value();
        set_config.set(saved.clone());
        saved.apply();
        set_preview_mode.set(false);

        if let Some(on_close) = on_close {
            on_close.run(());
        }
    };

    view! {
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-2xl w-full max-h-[90vh] overflow-y-auto">
                // Header
                <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
                    <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
                        "Theme Editor"
                    </h2>
                    <button
                        on:click=cancel
                        class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                        aria-label="Close"
                    >
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                // Body
                <div class="p-6 space-y-6">
                    // Theme Mode Selection
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            "Theme Mode"
                        </label>
                        <div class="flex gap-2">
                            <ThemeModeButton
                                mode=ThemeMode::Light
                                current=Signal::derive(move || config.get().mode)
                                on_select=Callback::new(move |mode| {
                                    set_config.update(|c| c.mode = mode);
                                    set_preview_mode.set(true);
                                })
                            />
                            <ThemeModeButton
                                mode=ThemeMode::Dark
                                current=Signal::derive(move || config.get().mode)
                                on_select=Callback::new(move |mode| {
                                    set_config.update(|c| c.mode = mode);
                                    set_preview_mode.set(true);
                                })
                            />
                            <ThemeModeButton
                                mode=ThemeMode::System
                                current=Signal::derive(move || config.get().mode)
                                on_select=Callback::new(move |mode| {
                                    set_config.update(|c| c.mode = mode);
                                    set_preview_mode.set(true);
                                })
                            />
                        </div>
                    </div>

                    // Color Pickers
                    <div class="grid grid-cols-2 gap-4">
                        <ColorPicker
                            label="Primary Color"
                            value=Signal::derive(move || config.get().primary_color)
                            on_change=Callback::new(move |color| {
                                set_config.update(|c| c.primary_color = color);
                                set_preview_mode.set(true);
                            })
                        />
                        <ColorPicker
                            label="Secondary Color"
                            value=Signal::derive(move || config.get().secondary_color)
                            on_change=Callback::new(move |color| {
                                set_config.update(|c| c.secondary_color = color);
                                set_preview_mode.set(true);
                            })
                        />
                        <ColorPicker
                            label="Accent Color"
                            value=Signal::derive(move || config.get().accent_color)
                            on_change=Callback::new(move |color| {
                                set_config.update(|c| c.accent_color = color);
                                set_preview_mode.set(true);
                            })
                        />
                        <ColorPicker
                            label="Success Color"
                            value=Signal::derive(move || config.get().success_color)
                            on_change=Callback::new(move |color| {
                                set_config.update(|c| c.success_color = color);
                                set_preview_mode.set(true);
                            })
                        />
                        <ColorPicker
                            label="Warning Color"
                            value=Signal::derive(move || config.get().warning_color)
                            on_change=Callback::new(move |color| {
                                set_config.update(|c| c.warning_color = color);
                                set_preview_mode.set(true);
                            })
                        />
                        <ColorPicker
                            label="Error Color"
                            value=Signal::derive(move || config.get().error_color)
                            on_change=Callback::new(move |color| {
                                set_config.update(|c| c.error_color = color);
                                set_preview_mode.set(true);
                            })
                        />
                    </div>

                    // Validation Warnings
                    <Show when=move || !warnings.get().is_empty()>
                        <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
                            <div class="flex items-start">
                                <svg class="w-5 h-5 text-yellow-600 dark:text-yellow-500 mt-0.5 mr-3" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
                                </svg>
                                <div class="flex-1">
                                    <h3 class="text-sm font-medium text-yellow-800 dark:text-yellow-200 mb-1">
                                        "Accessibility Warnings"
                                    </h3>
                                    <ul class="text-sm text-yellow-700 dark:text-yellow-300 space-y-1">
                                        {move || warnings.get().into_iter().map(|warning| {
                                            view! {
                                                <li>"• " {warning}</li>
                                            }
                                        }).collect_view()}
                                    </ul>
                                </div>
                            </div>
                        </div>
                    </Show>

                    // Preview Section
                    <div class="border border-gray-200 dark:border-gray-700 rounded-lg p-4">
                        <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
                            "Preview"
                        </h3>
                        <ThemePreview />
                    </div>
                </div>

                // Footer
                <div class="px-6 py-4 border-t border-gray-200 dark:border-gray-700 flex items-center justify-between">
                    <button
                        on:click=reset_theme
                        class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white"
                    >
                        "Reset to Default"
                    </button>
                    <div class="flex gap-2">
                        <button
                            on:click=cancel
                            class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-lg"
                        >
                            "Cancel"
                        </button>
                        <button
                            on:click=save_theme
                            class="px-4 py-2 text-sm font-medium text-white bg-primary hover:bg-primary-dark rounded-lg"
                        >
                            "Save Theme"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// SUB-COMPONENTS
// ============================================================================

#[component]
fn ThemeModeButton(
    mode: ThemeMode,
    current: Signal<ThemeMode>,
    on_select: Callback<ThemeMode>,
) -> impl IntoView {
    let is_active = move || current.get() == mode;

    let (icon, label) = match mode {
        ThemeMode::Light => ("☀️", "Light"),
        ThemeMode::Dark => ("🌙", "Dark"),
        ThemeMode::System => ("💻", "System"),
    };

    view! {
        <button
            on:click=move |_| on_select.run(mode)
            class=move || format!(
                "flex-1 px-4 py-3 text-sm font-medium rounded-lg border-2 transition-colors {}",
                if is_active() {
                    "border-primary bg-primary/10 text-primary"
                } else {
                    "border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:border-gray-300 dark:hover:border-gray-600"
                }
            )
        >
            <div class="flex flex-col items-center gap-1">
                <span class="text-2xl">{icon}</span>
                <span>{label}</span>
            </div>
        </button>
    }
}

#[component]
fn ColorPicker(
    label: &'static str,
    value: Signal<String>,
    on_change: Callback<String>,
) -> impl IntoView {
    view! {
        <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                {label}
            </label>
            <div class="flex items-center gap-2">
                <input
                    type="color"
                    prop:value=move || value.get()
                    on:input=move |ev| {
                        on_change.run(event_target_value(&ev));
                    }
                    class="w-12 h-12 rounded border border-gray-300 dark:border-gray-600 cursor-pointer"
                />
                <input
                    type="text"
                    prop:value=move || value.get()
                    on:input=move |ev| {
                        on_change.run(event_target_value(&ev));
                    }
                    class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm font-mono"
                    placeholder="#000000"
                />
            </div>
        </div>
    }
}

#[component]
fn ThemePreview() -> impl IntoView {
    view! {
        <div class="space-y-3">
            // Buttons
            <div class="flex gap-2">
                <button class="px-4 py-2 text-sm font-medium text-white rounded-lg" style="background-color: var(--color-primary)">
                    "Primary"
                </button>
                <button class="px-4 py-2 text-sm font-medium text-white rounded-lg" style="background-color: var(--color-secondary)">
                    "Secondary"
                </button>
                <button class="px-4 py-2 text-sm font-medium text-white rounded-lg" style="background-color: var(--color-accent)">
                    "Accent"
                </button>
            </div>

            // Status badges
            <div class="flex gap-2">
                <span class="px-3 py-1 text-xs font-medium text-white rounded-full" style="background-color: var(--color-success)">
                    "Success"
                </span>
                <span class="px-3 py-1 text-xs font-medium text-white rounded-full" style="background-color: var(--color-warning)">
                    "Warning"
                </span>
                <span class="px-3 py-1 text-xs font-medium text-white rounded-full" style="background-color: var(--color-error)">
                    "Error"
                </span>
            </div>

            // Card
            <div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4">
                <h4 class="text-sm font-medium text-gray-900 dark:text-white mb-2">
                    "Sample Card"
                </h4>
                <p class="text-sm text-gray-600 dark:text-gray-400">
                    "This is how your theme will look in cards and containers."
                </p>
            </div>
        </div>
    }
}
