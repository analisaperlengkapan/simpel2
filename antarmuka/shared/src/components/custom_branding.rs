//! Custom Branding Component
//!
//! Support untuk custom logos dan color schemes per unit kerja dengan accessibility validation.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use web_sys::window;

use crate::hooks::use_storage;

// ============================================================================
// BRANDING CONFIGURATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BrandingConfig {
    pub unit_name: String,
    pub logo_url: Option<String>,
    pub logo_dark_url: Option<String>, // Logo for dark mode
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub tagline: Option<String>,
}

impl Default for BrandingConfig {
    fn default() -> Self {
        Self {
            unit_name: "Kejaksaan Agung RI".to_string(),
            logo_url: Some("/assets/logo-kejaksaan.svg".to_string()),
            logo_dark_url: Some("/assets/logo-kejaksaan-dark.svg".to_string()),
            primary_color: "#DC2626".to_string(), // Kejaksaan Red
            secondary_color: "#1E40AF".to_string(), // Government Blue
            accent_color: "#F59E0B".to_string(),  // Gold
            tagline: Some("Sistem Informasi Manajemen Pengelolaan BMN".to_string()),
        }
    }
}

impl BrandingConfig {
    /// Create branding for specific unit
    pub fn for_unit(unit: &str) -> Self {
        match unit {
            "badiklat" => Self {
                unit_name: "Badiklat Kejaksaan RI".to_string(),
                tagline: Some("Pusat Pendidikan dan Pelatihan".to_string()),
                ..Default::default()
            },
            "datun" => Self {
                unit_name: "Datun Kejaksaan RI".to_string(),
                tagline: Some("Direktorat Perdata dan Tata Usaha Negara".to_string()),
                ..Default::default()
            },
            "intel" => Self {
                unit_name: "Intel Kejaksaan RI".to_string(),
                tagline: Some("Direktorat Intelijen".to_string()),
                ..Default::default()
            },
            "pidum" => Self {
                unit_name: "Pidum Kejaksaan RI".to_string(),
                tagline: Some("Direktorat Pidana Umum".to_string()),
                ..Default::default()
            },
            "pidsus" => Self {
                unit_name: "Pidsus Kejaksaan RI".to_string(),
                tagline: Some("Direktorat Pidana Khusus".to_string()),
                ..Default::default()
            },
            "pidmil" => Self {
                unit_name: "Pidmil Kejaksaan RI".to_string(),
                tagline: Some("Direktorat Pidana Militer".to_string()),
                ..Default::default()
            },
            "pengawasan" => Self {
                unit_name: "Pengawasan Kejaksaan RI".to_string(),
                tagline: Some("Direktorat Pengawasan".to_string()),
                ..Default::default()
            },
            "pemulihan_aset" => Self {
                unit_name: "Pemulihan Aset Kejaksaan RI".to_string(),
                tagline: Some("Direktorat Pemulihan Aset".to_string()),
                ..Default::default()
            },
            "pembinaan" => Self {
                unit_name: "Pembinaan Kejaksaan RI".to_string(),
                tagline: Some("Direktorat Pembinaan".to_string()),
                ..Default::default()
            },
            _ => Self::default(),
        }
    }

    /// Apply branding to document
    pub fn apply(&self) {
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
        let _ = html.set_attribute(
            "style",
            &format!(
                "--brand-primary: {}; --brand-secondary: {}; --brand-accent: {};",
                self.primary_color, self.secondary_color, self.accent_color
            ),
        );

        // Update document title
        if let Some(title) = document.query_selector("title").ok().flatten() {
            title.set_text_content(Some(&format!("{} - SIMPelv2", self.unit_name)));
        }
    }

    /// Validate color contrast for accessibility (WCAG AA: 4.5:1)
    pub fn validate_contrast(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check primary color contrast with white
        if !has_sufficient_contrast(&self.primary_color, "#FFFFFF", 4.5) {
            errors.push(format!(
                "Primary color {} does not have sufficient contrast with white text (WCAG AA requires 4.5:1)",
                self.primary_color
            ));
        }

        // Check secondary color contrast with white
        if !has_sufficient_contrast(&self.secondary_color, "#FFFFFF", 4.5) {
            errors.push(format!(
                "Secondary color {} does not have sufficient contrast with white text (WCAG AA requires 4.5:1)",
                self.secondary_color
            ));
        }

        // Check accent color contrast with white
        if !has_sufficient_contrast(&self.accent_color, "#FFFFFF", 4.5) {
            errors.push(format!(
                "Accent color {} does not have sufficient contrast with white text (WCAG AA requires 4.5:1)",
                self.accent_color
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Check if two colors have sufficient contrast ratio
fn has_sufficient_contrast(color1: &str, color2: &str, min_ratio: f64) -> bool {
    let l1 = calculate_relative_luminance(color1);
    let l2 = calculate_relative_luminance(color2);

    let contrast = if l1 > l2 {
        (l1 + 0.05) / (l2 + 0.05)
    } else {
        (l2 + 0.05) / (l1 + 0.05)
    };

    contrast >= min_ratio
}

/// Calculate relative luminance of a color (WCAG formula)
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
// BRANDING PROVIDER COMPONENT
// ============================================================================

#[component]
pub fn BrandingProvider(
    #[prop(optional)] unit: Option<String>,
    children: Children,
) -> impl IntoView {
    // Load branding config from storage or use default for unit
    let default_config = unit
        .as_ref()
        .map(|u| BrandingConfig::for_unit(u))
        .unwrap_or_default();

    let (branding, set_branding) = use_storage(
        &format!("simpelv2_branding_{}", unit.as_deref().unwrap_or("default")),
        default_config,
    );

    // Apply branding on mount and when it changes
    Effect::new(move || {
        branding.get().apply();
    });

    // Provide branding context
    provide_context(branding);
    provide_context(set_branding);

    view! {
        {children()}
    }
}

/// Hook to access branding configuration
pub fn use_branding() -> (ReadSignal<BrandingConfig>, WriteSignal<BrandingConfig>) {
    let branding =
        use_context::<ReadSignal<BrandingConfig>>().expect("BrandingProvider not found in context");
    let set_branding = use_context::<WriteSignal<BrandingConfig>>()
        .expect("BrandingProvider not found in context");

    (branding, set_branding)
}

// ============================================================================
// BRANDED LOGO COMPONENT
// ============================================================================

#[component]
pub fn BrandedLogo(
    #[prop(optional)] size: BrandedLogoSize,
    #[prop(optional)] class: String,
) -> impl IntoView {
    let (branding, _) = use_branding();

    let size_class = match size {
        BrandedLogoSize::Small => "h-8",
        BrandedLogoSize::Medium => "h-12",
        BrandedLogoSize::Large => "h-16",
        BrandedLogoSize::ExtraLarge => "h-24",
    }
    .to_string();

    let size_class_clone = size_class.clone();
    let size_class_clone2 = size_class.clone();

    view! {
        <div class=format!("flex items-center gap-3 {}", class)>
            <Show
                when=move || branding.get().logo_url.is_some()
                fallback=move || view! {
                    <div class=format!("bg-primary text-white font-bold rounded-lg flex items-center justify-center {}", size_class_clone)>
                        {move || branding.get().unit_name.chars().next().unwrap_or('K')}
                    </div>
                }
            >
                <picture>
                    // Dark mode logo
                    <source
                        srcset=move || branding.get().logo_dark_url.unwrap_or_default()
                        media="(prefers-color-scheme: dark)"
                    />
                    // Light mode logo
                    <img
                        src=move || branding.get().logo_url.unwrap_or_default()
                        alt=move || format!("{} Logo", branding.get().unit_name)
                        class=format!("object-contain {}", size_class_clone2)
                    />
                </picture>
            </Show>

            <Show when=move || matches!(size, BrandedLogoSize::Large | BrandedLogoSize::ExtraLarge)>
                <div class="flex flex-col">
                    <span class="font-bold text-gray-900 dark:text-white text-lg">
                        {move || branding.get().unit_name}
                    </span>
                    <Show when=move || branding.get().tagline.is_some()>
                        <span class="text-sm text-gray-600 dark:text-gray-400">
                            {move || branding.get().tagline.clone().unwrap_or_default()}
                        </span>
                    </Show>
                </div>
            </Show>
        </div>
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BrandedLogoSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

impl Default for BrandedLogoSize {
    fn default() -> Self {
        Self::Medium
    }
}

// ============================================================================
// BRANDING EDITOR COMPONENT
// ============================================================================

#[component]
pub fn BrandingEditor(#[prop(optional)] on_close: Option<Callback<()>>) -> impl IntoView {
    let (branding, set_branding) = use_branding();

    // Local state for editing
    let (editing, set_editing) = signal(branding.get());

    // Validation errors
    let validation_errors = Memo::new(move |_| editing.get().validate_contrast().err());

    // Save branding
    let save_branding = move |_| {
        let config = editing.get();

        // Validate before saving
        if config.validate_contrast().is_ok() {
            set_branding.set(config.clone());
            config.apply();

            if let Some(on_close) = on_close {
                on_close.run(());
            }
        }
    };

    // Cancel editing
    let cancel = move |_| {
        set_editing.set(branding.get());

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
                        "Custom Branding"
                    </h2>
                    <button
                        on:click=move |_| {
                            if let Some(on_close) = on_close {
                                on_close.run(());
                            }
                        }
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
                    // Unit Name
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            "Unit Name"
                        </label>
                        <input
                            type="text"
                            prop:value=move || editing.get().unit_name
                            on:input=move |ev| {
                                set_editing.update(|e| e.unit_name = event_target_value(&ev));
                            }
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                            placeholder="Nama Unit Kerja"
                        />
                    </div>

                    // Tagline
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            "Tagline (Optional)"
                        </label>
                        <input
                            type="text"
                            prop:value=move || editing.get().tagline.unwrap_or_default()
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                set_editing.update(|e| {
                                    e.tagline = if value.is_empty() { None } else { Some(value) };
                                });
                            }
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                            placeholder="Tagline atau deskripsi singkat"
                        />
                    </div>

                    // Logo URLs
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                "Logo URL (Light Mode)"
                            </label>
                            <input
                                type="url"
                                prop:value=move || editing.get().logo_url.unwrap_or_default()
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    set_editing.update(|e| {
                                        e.logo_url = if value.is_empty() { None } else { Some(value) };
                                    });
                                }
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm"
                                placeholder="/assets/logo.svg"
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                "Logo URL (Dark Mode)"
                            </label>
                            <input
                                type="url"
                                prop:value=move || editing.get().logo_dark_url.unwrap_or_default()
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    set_editing.update(|e| {
                                        e.logo_dark_url = if value.is_empty() { None } else { Some(value) };
                                    });
                                }
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm"
                                placeholder="/assets/logo-dark.svg"
                            />
                        </div>
                    </div>

                    // Color Scheme
                    <div class="grid grid-cols-3 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                "Primary Color"
                            </label>
                            <div class="flex items-center gap-2">
                                <input
                                    type="color"
                                    prop:value=move || editing.get().primary_color
                                    on:input=move |ev| {
                                        set_editing.update(|e| e.primary_color = event_target_value(&ev));
                                    }
                                    class="w-12 h-12 rounded border border-gray-300 dark:border-gray-600 cursor-pointer"
                                />
                                <input
                                    type="text"
                                    prop:value=move || editing.get().primary_color
                                    on:input=move |ev| {
                                        set_editing.update(|e| e.primary_color = event_target_value(&ev));
                                    }
                                    class="flex-1 px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-xs font-mono"
                                />
                            </div>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                "Secondary Color"
                            </label>
                            <div class="flex items-center gap-2">
                                <input
                                    type="color"
                                    prop:value=move || editing.get().secondary_color
                                    on:input=move |ev| {
                                        set_editing.update(|e| e.secondary_color = event_target_value(&ev));
                                    }
                                    class="w-12 h-12 rounded border border-gray-300 dark:border-gray-600 cursor-pointer"
                                />
                                <input
                                    type="text"
                                    prop:value=move || editing.get().secondary_color
                                    on:input=move |ev| {
                                        set_editing.update(|e| e.secondary_color = event_target_value(&ev));
                                    }
                                    class="flex-1 px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-xs font-mono"
                                />
                            </div>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                "Accent Color"
                            </label>
                            <div class="flex items-center gap-2">
                                <input
                                    type="color"
                                    prop:value=move || editing.get().accent_color
                                    on:input=move |ev| {
                                        set_editing.update(|e| e.accent_color = event_target_value(&ev));
                                    }
                                    class="w-12 h-12 rounded border border-gray-300 dark:border-gray-600 cursor-pointer"
                                />
                                <input
                                    type="text"
                                    prop:value=move || editing.get().accent_color
                                    on:input=move |ev| {
                                        set_editing.update(|e| e.accent_color = event_target_value(&ev));
                                    }
                                    class="flex-1 px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-xs font-mono"
                                />
                            </div>
                        </div>
                    </div>

                    // Validation Errors
                    <Show when=move || validation_errors.get().is_some()>
                        <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
                            <div class="flex items-start">
                                <svg class="w-5 h-5 text-red-600 dark:text-red-500 mt-0.5 mr-3" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                                </svg>
                                <div class="flex-1">
                                    <h3 class="text-sm font-medium text-red-800 dark:text-red-200 mb-1">
                                        "Accessibility Errors"
                                    </h3>
                                    <ul class="text-sm text-red-700 dark:text-red-300 space-y-1">
                                        {move || validation_errors.get().unwrap_or_default().into_iter().map(|error| {
                                            view! {
                                                <li>"• " {error}</li>
                                            }
                                        }).collect_view()}
                                    </ul>
                                </div>
                            </div>
                        </div>
                    </Show>

                    // Preview
                    <div class="border border-gray-200 dark:border-gray-700 rounded-lg p-4">
                        <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
                            "Preview"
                        </h3>
                        <div class="flex items-center gap-4">
                            <BrandedLogo size=BrandedLogoSize::Large />
                        </div>
                    </div>
                </div>

                // Footer
                <div class="px-6 py-4 border-t border-gray-200 dark:border-gray-700 flex items-center justify-end gap-2">
                    <button
                        on:click=cancel
                        class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-lg"
                    >
                        "Cancel"
                    </button>
                    <button
                        on:click=save_branding
                        disabled=move || validation_errors.get().is_some()
                        class="px-4 py-2 text-sm font-medium text-white bg-primary hover:bg-primary-dark rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                        "Save Branding"
                    </button>
                </div>
            </div>
        </div>
    }
}
