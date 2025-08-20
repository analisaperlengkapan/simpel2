//! # Sistem Tema Kejaksaan
//!
//! Mengatur tema visual dan branding sesuai dengan
//! panduan identitas visual Kejaksaan RI.

use crate::constants::{KejaksaanColors, UNIT_CODES};
use std::collections::HashMap;

/// Tema utama Kejaksaan
#[derive(Debug, Clone)]
pub struct KejaksaanTheme {
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub background_color: String,
    pub text_color: String,
    pub border_color: String,
}

impl Default for KejaksaanTheme {
    fn default() -> Self {
        Self {
            primary_color: KejaksaanColors::PRIMARY.to_string(),
            secondary_color: KejaksaanColors::GOLD.to_string(),
            accent_color: KejaksaanColors::PRIMARY_LIGHT.to_string(),
            background_color: KejaksaanColors::GRAY_50.to_string(),
            text_color: KejaksaanColors::GRAY_900.to_string(),
            border_color: KejaksaanColors::GRAY_100.to_string(),
        }
    }
}

/// Mendapatkan warna tema berdasarkan unit
pub fn get_theme_colors(unit_code: &str) -> HashMap<String, String> {
    let mut colors = HashMap::new();

    match unit_code {
        "PIDUM" => {
            colors.insert("primary".to_string(), "#DC2626".to_string()); // red-600
            colors.insert("secondary".to_string(), "#FCA5A5".to_string()); // red-300
            colors.insert("accent".to_string(), "#B91C1C".to_string()); // red-700
        }
        "PIDSUS" => {
            colors.insert("primary".to_string(), "#7C3AED".to_string()); // violet-600
            colors.insert("secondary".to_string(), "#C4B5FD".to_string()); // violet-300
            colors.insert("accent".to_string(), "#6D28D9".to_string()); // violet-700
        }
        "PIDMIL" => {
            colors.insert("primary".to_string(), "#D97706".to_string()); // amber-600
            colors.insert("secondary".to_string(), "#FCD34D".to_string()); // amber-300
            colors.insert("accent".to_string(), "#B45309".to_string()); // amber-700
        }
        "DATUN" => {
            colors.insert("primary".to_string(), "#2563EB".to_string()); // blue-600
            colors.insert("secondary".to_string(), "#93C5FD".to_string()); // blue-300
            colors.insert("accent".to_string(), "#1D4ED8".to_string()); // blue-700
        }
        "INTEL" => {
            colors.insert("primary".to_string(), "#4F46E5".to_string()); // indigo-600
            colors.insert("secondary".to_string(), "#A5B4FC".to_string()); // indigo-300
            colors.insert("accent".to_string(), "#4338CA".to_string()); // indigo-700
        }
        "PENGAWASAN" => {
            colors.insert("primary".to_string(), "#9333EA".to_string()); // purple-600
            colors.insert("secondary".to_string(), "#D8B4FE".to_string()); // purple-300
            colors.insert("accent".to_string(), "#7C2D12".to_string()); // purple-700
        }
        "PEMULIHAN_ASET" => {
            colors.insert("primary".to_string(), "#059669".to_string()); // emerald-600
            colors.insert("secondary".to_string(), "#6EE7B7".to_string()); // emerald-300
            colors.insert("accent".to_string(), "#047857".to_string()); // emerald-700
        }
        "BADIKLAT" => {
            colors.insert("primary".to_string(), "#16A34A".to_string()); // green-600
            colors.insert("secondary".to_string(), "#86EFAC".to_string()); // green-300
            colors.insert("accent".to_string(), "#15803D".to_string()); // green-700
        }
        _ => {
            // Default Kejaksaan colors
            colors.insert("primary".to_string(), KejaksaanColors::PRIMARY.to_string());
            colors.insert("secondary".to_string(), KejaksaanColors::GOLD.to_string());
            colors.insert(
                "accent".to_string(),
                KejaksaanColors::PRIMARY_LIGHT.to_string(),
            );
        }
    }

    colors
}

/// Menerapkan tema unit ke elemen
pub fn apply_unit_theme(unit_code: &str) -> String {
    let colors = get_theme_colors(unit_code);
    let units = &*UNIT_CODES;

    if let Some(unit_info) = units.get(unit_code) {
        format!(
            r#"
            /* Tema {} */
            :root {{
                --unit-primary: {};
                --unit-secondary: {};
                --unit-accent: {};
                --unit-icon: {};
            }}

            .unit-theme {{
                --tw-bg-opacity: 1;
                background-color: rgb({} / var(--tw-bg-opacity));
            }}

            .unit-text {{
                --tw-text-opacity: 1;
                color: rgb({} / var(--tw-text-opacity));
            }}

            .unit-border {{
                --tw-border-opacity: 1;
                border-color: rgb({} / var(--tw-border-opacity));
            }}
            "#,
            unit_info.name,
            colors
                .get("primary")
                .unwrap_or(&KejaksaanColors::PRIMARY.to_string()),
            colors
                .get("secondary")
                .unwrap_or(&KejaksaanColors::GOLD.to_string()),
            colors
                .get("accent")
                .unwrap_or(&KejaksaanColors::PRIMARY_LIGHT.to_string()),
            unit_info.icon,
            hex_to_rgb(
                colors
                    .get("primary")
                    .unwrap_or(&KejaksaanColors::PRIMARY.to_string())
            ),
            hex_to_rgb(
                colors
                    .get("primary")
                    .unwrap_or(&KejaksaanColors::PRIMARY.to_string())
            ),
            hex_to_rgb(
                colors
                    .get("secondary")
                    .unwrap_or(&KejaksaanColors::GOLD.to_string())
            ),
        )
    } else {
        String::new()
    }
}

/// Konversi hex ke rgb
fn hex_to_rgb(hex: &str) -> String {
    if hex.starts_with('#') && hex.len() == 7 {
        let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(0);
        format!("{r} {g} {b}")
    } else {
        "0 0 0".to_string()
    }
}

/// CSS untuk komponen Kejaksaan
pub fn get_kejaksaan_css() -> &'static str {
    r#"
    /* Kejaksaan Design System CSS */

    /* Logo dan Branding */
    .kejaksaan-logo {
        background: linear-gradient(135deg, #1E40AF 0%, #1E3A8A 100%);
        border-radius: 0.5rem;
        box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
    }

    .kejaksaan-brand-text {
        font-family: 'Poppins', sans-serif;
        font-weight: 600;
        color: #111827;
    }

    .kejaksaan-subtitle {
        font-size: 0.75rem;
        color: #6B7280;
        font-weight: 400;
    }

    /* Header Styles */
    .kejaksaan-header {
        background: linear-gradient(90deg, #ffffff 0%, #f8fafc 100%);
        border-bottom: 1px solid #e5e7eb;
        backdrop-filter: blur(8px);
    }

    /* Navigation Styles */
    .kejaksaan-nav-item {
        transition: all 0.2s ease-in-out;
        border-radius: 0.375rem;
    }

    .kejaksaan-nav-item:hover {
        background-color: rgba(59, 130, 246, 0.1);
        transform: translateY(-1px);
    }

    .kejaksaan-nav-item.active {
        background-color: rgba(59, 130, 246, 0.15);
        border-left: 3px solid #3B82F6;
    }

    /* Footer Styles */
    .kejaksaan-footer {
        background: linear-gradient(135deg, #1E40AF 0%, #1E3A8A 100%);
        color: white;
    }

    /* Button Styles */
    .btn-kejaksaan {
        background: linear-gradient(135deg, #3B82F6 0%, #1E40AF 100%);
        border: none;
        border-radius: 0.375rem;
        color: white;
        font-weight: 500;
        transition: all 0.2s ease-in-out;
    }

    .btn-kejaksaan:hover {
        transform: translateY(-1px);
        box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
    }

    .btn-kejaksaan:active {
        transform: translateY(0);
    }

    /* Card Styles */
    .kejaksaan-card {
        background: white;
        border-radius: 0.75rem;
        box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1);
        border: 1px solid #e5e7eb;
        transition: all 0.2s ease-in-out;
    }

    .kejaksaan-card:hover {
        transform: translateY(-2px);
        box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
    }

    /* Typography */
    .text-kejaksaan-heading {
        font-family: 'Poppins', sans-serif;
        font-weight: 600;
        color: #1E40AF;
    }

    .text-kejaksaan-body {
        font-family: 'Inter', sans-serif;
        line-height: 1.6;
        color: #374151;
    }

    /* Responsive Utilities */
    @media (max-width: 768px) {
        .kejaksaan-mobile-hidden {
            display: none;
        }

        .kejaksaan-mobile-show {
            display: block;
        }
    }

    /* Animation Classes */
    .kejaksaan-fade-in {
        animation: kejaksaanFadeIn 0.3s ease-in-out;
    }

    @keyframes kejaksaanFadeIn {
        from {
            opacity: 0;
            transform: translateY(10px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }

    /* Status Indicators */
    .status-success { color: #10B981; }
    .status-warning { color: #F59E0B; }
    .status-error { color: #EF4444; }
    .status-info { color: #3B82F6; }

    /* Accessibility */
    .sr-only {
        position: absolute;
        width: 1px;
        height: 1px;
        padding: 0;
        margin: -1px;
        overflow: hidden;
        clip: rect(0, 0, 0, 0);
        white-space: nowrap;
        border-width: 0;
    }

    /* Focus States */
    .focus\\:ring-kejaksaan:focus {
        outline: 2px solid transparent;
        outline-offset: 2px;
        box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
        border-color: #3B82F6;
    }
    "#
}
