//! Simple theming system untuk aplikasi Kejaksaan RI
//!
//! Focused pada light/dark mode dengan government branding.

use serde::{Deserialize, Serialize};
use web_sys::window;

// ============================================================================
// THEME VARIANT
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

impl ThemeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::System => "system",
        }
    }
}

impl std::str::FromStr for ThemeMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "dark" => Self::Dark,
            "system" => Self::System,
            _ => Self::Light,
        })
    }
}

// ============================================================================
// THEME MANAGEMENT
// ============================================================================

/// Get current theme from localStorage or system preference
pub fn get_theme() -> ThemeMode {
    let window = match window() {
        Some(w) => w,
        None => return ThemeMode::Light,
    };

    // Try localStorage first
    if let Ok(Some(storage)) = window.local_storage() {
        if let Ok(Some(theme_str)) = storage.get_item("simpelv2_theme") {
            use std::str::FromStr;
            return ThemeMode::from_str(&theme_str).unwrap_or(ThemeMode::Light);
        }
    }

    // Fall back to system preference
    if prefers_dark_mode() {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    }
}

/// Set theme and persist to localStorage
pub fn set_theme(mode: ThemeMode) {
    let window = match window() {
        Some(w) => w,
        None => return,
    };

    // Save to localStorage
    if let Ok(Some(storage)) = window.local_storage() {
        let _ = storage.set_item("simpelv2_theme", mode.as_str());
    }

    // Apply to document
    apply_theme(mode);
}

/// Apply theme to document root
pub fn apply_theme(mode: ThemeMode) {
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

    // Remove existing theme classes
    let _ = html.class_list().remove_2("light", "dark");

    // Add new theme class
    let theme_class = match mode {
        ThemeMode::System => {
            if prefers_dark_mode() {
                "dark"
            } else {
                "light"
            }
        }
        ThemeMode::Dark => "dark",
        ThemeMode::Light => "light",
    };

    let _ = html.class_list().add_1(theme_class);

    // Set data-theme attribute for CSS targeting
    let _ = html.set_attribute("data-theme", theme_class);
}

/// Check if user prefers dark mode (system setting)
pub fn prefers_dark_mode() -> bool {
    let window = match window() {
        Some(w) => w,
        None => return false,
    };

    window
        .match_media("(prefers-color-scheme: dark)")
        .ok()
        .flatten()
        .map(|mql| mql.matches())
        .unwrap_or(false)
}

/// Toggle between light and dark mode
pub fn toggle_theme() {
    let current = get_theme();
    let new_theme = match current {
        ThemeMode::Light | ThemeMode::System => ThemeMode::Dark,
        ThemeMode::Dark => ThemeMode::Light,
    };
    set_theme(new_theme);
}

// ============================================================================
// ACCESSIBILITY
// ============================================================================

/// Check if user prefers reduced motion
pub fn prefers_reduced_motion() -> bool {
    let window = match window() {
        Some(w) => w,
        None => return false,
    };

    window
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .map(|mql| mql.matches())
        .unwrap_or(false)
}

/// Check if user prefers high contrast
pub fn prefers_high_contrast() -> bool {
    let window = match window() {
        Some(w) => w,
        None => return false,
    };

    window
        .match_media("(prefers-contrast: high)")
        .ok()
        .flatten()
        .map(|mql| mql.matches())
        .unwrap_or(false)
}

// ============================================================================
// CSS VARIABLES
// ============================================================================

/// Generate CSS custom properties based on theme
pub fn generate_css_vars(mode: ThemeMode) -> String {
    let is_dark = matches!(mode, ThemeMode::Dark)
        || (matches!(mode, ThemeMode::System) && prefers_dark_mode());

    if is_dark {
        r#"
            --color-primary: #10b981;
            --color-primary-hover: #059669;
            --color-background: #1f2937;
            --color-surface: #374151;
            --color-text: #f9fafb;
            --color-text-secondary: #d1d5db;
            --color-border: #4b5563;
        "#
    } else {
        r#"
            --color-primary: #047857;
            --color-primary-hover: #065f46;
            --color-background: #ffffff;
            --color-surface: #f9fafb;
            --color-text: #1f2937;
            --color-text-secondary: #6b7280;
            --color-border: #e5e7eb;
        "#
    }
    .to_string()
}

// ============================================================================
// THEME HOOK FOR LEPTOS
// ============================================================================

use leptos::prelude::*;

/// Hook for managing theme in Leptos components
pub fn use_theme() -> (ReadSignal<ThemeMode>, impl Fn(ThemeMode) + Clone + 'static) {
    let (theme, set_theme_signal) = signal(get_theme());

    // Apply theme on mount
    Effect::new(move |_| {
        apply_theme(theme.get());
    });

    // Custom setter that also applies the theme
    let set_theme_with_apply = move |new_theme: ThemeMode| {
        set_theme(new_theme);
        set_theme_signal.set(new_theme);
    };

    (theme, set_theme_with_apply)
}

/// Hook for toggling theme
pub fn use_theme_toggle() -> impl Fn() + Clone {
    let (theme, set_theme) = use_theme();

    move || {
        let new_theme = match theme.get() {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::System => {
                if prefers_dark_mode() {
                    ThemeMode::Light
                } else {
                    ThemeMode::Dark
                }
            }
        };
        set_theme(new_theme);
    }
}
