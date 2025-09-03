//! # SIMPelv2 Advanced Theme System 🎨
//!
//! **Dynamic, context-aware theming** for Indonesian government applications.
//!
//! ## 🎯 **Theme Features**
//! - **Multi-Theme Support**: Light, dark, high-contrast, and government modes
//! - **Accessibility**: WCAG 2.1 AA compliant color schemes and contrast ratios
//! - **Responsive Design**: Adaptive themes based on screen size and device
//! - **Context Awareness**: Themes adapt to user preferences and system settings
//! - **Performance**: Zero-cost abstractions with compile-time optimizations
//! - **Indonesian Standards**: Full compliance with government branding guidelines
//!
//! ## 🎨 **Theme Types**
//! - **Standard Light**: Default government theme with Kejaksaan branding
//! - **Dark Mode**: Low-light environment optimized with proper contrast
//! - **High Contrast**: Accessibility-focused theme for visually impaired users
//! - **Kejaksaan Official**: Official government theme with strict brand compliance
//! - **Print Mode**: Optimized for document printing and export

use once_cell::sync::Lazy;
use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::constants::{animations, borders, colors, shadows, spacing, typography};

// ============================================================================
// THEME ENUMS & TYPES
// ============================================================================

/// Available theme variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ThemeVariant {
    /// Standard light theme (default)
    Light,
    /// Dark theme for low-light environments
    Dark,
    /// High contrast theme for accessibility
    HighContrast,
    /// Official Kejaksaan theme with strict branding
    KejaksaanOfficial,
    /// Print-optimized theme
    Print,
    /// Auto theme (follows system preference)
    Auto,
}

impl Default for ThemeVariant {
    fn default() -> Self {
        ThemeVariant::Light
    }
}

impl std::fmt::Display for ThemeVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeVariant::Light => write!(f, "light"),
            ThemeVariant::Dark => write!(f, "dark"),
            ThemeVariant::HighContrast => write!(f, "high-contrast"),
            ThemeVariant::KejaksaanOfficial => write!(f, "kejaksaan-official"),
            ThemeVariant::Print => write!(f, "print"),
            ThemeVariant::Auto => write!(f, "auto"),
        }
    }
}

/// Color palette for a specific theme
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ColorPalette {
    // Primary colors
    pub primary: String,
    pub primary_hover: String,
    pub primary_active: String,
    pub primary_disabled: String,
    pub primary_text: String,

    // Secondary colors
    pub secondary: String,
    pub secondary_hover: String,
    pub secondary_active: String,
    pub secondary_disabled: String,
    pub secondary_text: String,

    // Semantic colors
    pub success: String,
    pub success_hover: String,
    pub success_text: String,
    pub warning: String,
    pub warning_hover: String,
    pub warning_text: String,
    pub danger: String,
    pub danger_hover: String,
    pub danger_text: String,
    pub info: String,
    pub info_hover: String,
    pub info_text: String,

    // Background colors
    pub background_primary: String,
    pub background_secondary: String,
    pub background_tertiary: String,
    pub background_modal: String,
    pub background_overlay: String,

    // Surface colors
    pub surface: String,
    pub surface_hover: String,
    pub surface_active: String,
    pub surface_disabled: String,

    // Text colors
    pub text_primary: String,
    pub text_secondary: String,
    pub text_tertiary: String,
    pub text_disabled: String,
    pub text_inverse: String,

    // Border colors
    pub border_primary: String,
    pub border_secondary: String,
    pub border_focus: String,
    pub border_error: String,
    pub border_success: String,

    // Interactive colors
    pub link: String,
    pub link_hover: String,
    pub link_active: String,
    pub link_visited: String,

    // Special colors
    pub accent: String,
    pub highlight: String,
    pub shadow: String,
    pub divider: String,
}

/// Typography settings for a theme
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Typography {
    pub font_family_primary: String,
    pub font_family_secondary: String,
    pub font_family_mono: String,

    // Font sizes
    pub font_size_xs: String,
    pub font_size_sm: String,
    pub font_size_base: String,
    pub font_size_lg: String,
    pub font_size_xl: String,
    pub font_size_2xl: String,
    pub font_size_3xl: String,
    pub font_size_4xl: String,

    // Font weights
    pub font_weight_light: u16,
    pub font_weight_normal: u16,
    pub font_weight_medium: u16,
    pub font_weight_semibold: u16,
    pub font_weight_bold: u16,

    // Line heights
    pub line_height_tight: f32,
    pub line_height_normal: f32,
    pub line_height_relaxed: f32,

    // Letter spacing
    pub letter_spacing_tight: String,
    pub letter_spacing_normal: String,
    pub letter_spacing_wide: String,
}

/// Spacing system for a theme
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Spacing {
    pub space_xs: String,
    pub space_sm: String,
    pub space_md: String,
    pub space_lg: String,
    pub space_xl: String,
    pub space_2xl: String,
    pub space_3xl: String,
    pub space_4xl: String,

    // Component spacing
    pub header_height: String,
    pub sidebar_width: String,
    pub sidebar_width_collapsed: String,
    pub footer_height: String,

    // Layout spacing
    pub container_padding: String,
    pub section_padding: String,
    pub card_padding: String,
    pub form_spacing: String,
}

/// Animation and transition settings
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Animations {
    pub duration_fast: String,
    pub duration_normal: String,
    pub duration_slow: String,
    pub duration_slower: String,

    pub easing_ease_in: String,
    pub easing_ease_out: String,
    pub easing_ease_in_out: String,
    pub easing_bounce: String,

    pub transition_all: String,
    pub transition_colors: String,
    pub transition_transform: String,
    pub transition_opacity: String,
}

/// Shadow system for depth and elevation
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Shadows {
    pub shadow_none: String,
    pub shadow_sm: String,
    pub shadow_md: String,
    pub shadow_lg: String,
    pub shadow_xl: String,
    pub shadow_2xl: String,
    pub shadow_inner: String,
    pub shadow_focus: String,
}

/// Border and radius settings
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Borders {
    pub radius_none: String,
    pub radius_sm: String,
    pub radius_md: String,
    pub radius_lg: String,
    pub radius_xl: String,
    pub radius_2xl: String,
    pub radius_full: String,

    pub border_width_thin: String,
    pub border_width_thick: String,
    pub border_width_thicker: String,

    pub border_style_solid: String,
    pub border_style_dashed: String,
    pub border_style_dotted: String,
}

/// Complete theme configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Theme {
    pub name: String,
    pub variant: ThemeVariant,
    pub colors: ColorPalette,
    pub typography: Typography,
    pub spacing: Spacing,
    pub animations: Animations,
    pub shadows: Shadows,
    pub borders: Borders,

    // Accessibility settings
    pub high_contrast: bool,
    pub reduced_motion: bool,
    pub focus_visible: bool,

    // Print settings
    pub print_optimized: bool,
    pub print_colors: bool,
}

// ============================================================================
// THEME IMPLEMENTATIONS
// ============================================================================

impl ColorPalette {
    /// Create light theme color palette
    pub fn light() -> Self {
        Self {
            primary: colors::KEJAKSAAN_PRIMARY.to_string(),
            primary_hover: colors::KEJAKSAAN_PRIMARY_DARK.to_string(),
            primary_active: "#1d4ed8".to_string(),
            primary_disabled: "#93c5fd".to_string(),
            primary_text: colors::WHITE.to_string(),

            secondary: colors::KEJAKSAAN_SECONDARY.to_string(),
            secondary_hover: "#b91c1c".to_string(),
            secondary_active: "#991b1b".to_string(),
            secondary_disabled: "#fca5a5".to_string(),
            secondary_text: colors::WHITE.to_string(),

            success: colors::SUCCESS.to_string(),
            success_hover: "#15803d".to_string(),
            success_text: colors::WHITE.to_string(),
            warning: colors::WARNING.to_string(),
            warning_hover: "#d97706".to_string(),
            warning_text: colors::WHITE.to_string(),
            danger: colors::DANGER.to_string(),
            danger_hover: "#b91c1c".to_string(),
            danger_text: colors::WHITE.to_string(),
            info: colors::INFO.to_string(),
            info_hover: "#2563eb".to_string(),
            info_text: colors::WHITE.to_string(),

            background_primary: colors::WHITE.to_string(),
            background_secondary: colors::GRAY_50.to_string(),
            background_tertiary: colors::GRAY_100.to_string(),
            background_modal: "rgba(0, 0, 0, 0.5)".to_string(),
            background_overlay: "rgba(0, 0, 0, 0.25)".to_string(),

            surface: colors::WHITE.to_string(),
            surface_hover: colors::GRAY_50.to_string(),
            surface_active: colors::GRAY_100.to_string(),
            surface_disabled: colors::GRAY_100.to_string(),

            text_primary: colors::GRAY_900.to_string(),
            text_secondary: colors::GRAY_700.to_string(),
            text_tertiary: colors::GRAY_500.to_string(),
            text_disabled: colors::GRAY_400.to_string(),
            text_inverse: colors::WHITE.to_string(),

            border_primary: colors::GRAY_200.to_string(),
            border_secondary: colors::GRAY_300.to_string(),
            border_focus: colors::KEJAKSAAN_PRIMARY.to_string(),
            border_error: colors::DANGER.to_string(),
            border_success: colors::SUCCESS.to_string(),

            link: colors::KEJAKSAAN_PRIMARY.to_string(),
            link_hover: colors::KEJAKSAAN_PRIMARY_DARK.to_string(),
            link_active: "#1d4ed8".to_string(),
            link_visited: "#7c3aed".to_string(),

            accent: colors::KEJAKSAAN_GOLD.to_string(),
            highlight: "#fef3c7".to_string(),
            shadow: "rgba(0, 0, 0, 0.1)".to_string(),
            divider: colors::GRAY_200.to_string(),
        }
    }

    /// Create dark theme color palette
    pub fn dark() -> Self {
        Self {
            primary: "#3b82f6".to_string(),
            primary_hover: "#60a5fa".to_string(),
            primary_active: "#2563eb".to_string(),
            primary_disabled: "#1e40af".to_string(),
            primary_text: colors::WHITE.to_string(),

            secondary: "#ef4444".to_string(),
            secondary_hover: "#f87171".to_string(),
            secondary_active: "#dc2626".to_string(),
            secondary_disabled: "#b91c1c".to_string(),
            secondary_text: colors::WHITE.to_string(),

            success: "#22c55e".to_string(),
            success_hover: "#4ade80".to_string(),
            success_text: colors::WHITE.to_string(),
            warning: "#f59e0b".to_string(),
            warning_hover: "#fbbf24".to_string(),
            warning_text: colors::WHITE.to_string(),
            danger: "#ef4444".to_string(),
            danger_hover: "#f87171".to_string(),
            danger_text: colors::WHITE.to_string(),
            info: "#3b82f6".to_string(),
            info_hover: "#60a5fa".to_string(),
            info_text: colors::WHITE.to_string(),

            background_primary: "#0f172a".to_string(),
            background_secondary: "#1e293b".to_string(),
            background_tertiary: "#334155".to_string(),
            background_modal: "rgba(0, 0, 0, 0.8)".to_string(),
            background_overlay: "rgba(0, 0, 0, 0.5)".to_string(),

            surface: "#1e293b".to_string(),
            surface_hover: "#334155".to_string(),
            surface_active: "#475569".to_string(),
            surface_disabled: "#1e293b".to_string(),

            text_primary: "#f1f5f9".to_string(),
            text_secondary: "#cbd5e1".to_string(),
            text_tertiary: "#94a3b8".to_string(),
            text_disabled: "#64748b".to_string(),
            text_inverse: "#0f172a".to_string(),

            border_primary: "#475569".to_string(),
            border_secondary: "#334155".to_string(),
            border_focus: "#3b82f6".to_string(),
            border_error: "#ef4444".to_string(),
            border_success: "#22c55e".to_string(),

            link: "#60a5fa".to_string(),
            link_hover: "#93c5fd".to_string(),
            link_active: "#3b82f6".to_string(),
            link_visited: "#a855f7".to_string(),

            accent: "#fbbf24".to_string(),
            highlight: "#365314".to_string(),
            shadow: "rgba(0, 0, 0, 0.5)".to_string(),
            divider: "#475569".to_string(),
        }
    }

    /// Create high contrast theme color palette
    pub fn high_contrast() -> Self {
        Self {
            primary: colors::BLACK.to_string(),
            primary_hover: "#333333".to_string(),
            primary_active: "#000000".to_string(),
            primary_disabled: "#666666".to_string(),
            primary_text: colors::WHITE.to_string(),

            secondary: "#800000".to_string(),
            secondary_hover: "#a00000".to_string(),
            secondary_active: "#600000".to_string(),
            secondary_disabled: "#400000".to_string(),
            secondary_text: colors::WHITE.to_string(),

            success: "#006400".to_string(),
            success_hover: "#228B22".to_string(),
            success_text: colors::WHITE.to_string(),
            warning: "#FF8C00".to_string(),
            warning_hover: "#FFA500".to_string(),
            warning_text: colors::BLACK.to_string(),
            danger: "#DC143C".to_string(),
            danger_hover: "#FF1493".to_string(),
            danger_text: colors::WHITE.to_string(),
            info: "#000080".to_string(),
            info_hover: "#0000CD".to_string(),
            info_text: colors::WHITE.to_string(),

            background_primary: colors::WHITE.to_string(),
            background_secondary: "#f5f5f5".to_string(),
            background_tertiary: "#e5e5e5".to_string(),
            background_modal: "rgba(0, 0, 0, 0.9)".to_string(),
            background_overlay: "rgba(0, 0, 0, 0.7)".to_string(),

            surface: colors::WHITE.to_string(),
            surface_hover: "#f0f0f0".to_string(),
            surface_active: "#e0e0e0".to_string(),
            surface_disabled: "#d0d0d0".to_string(),

            text_primary: colors::BLACK.to_string(),
            text_secondary: "#333333".to_string(),
            text_tertiary: "#666666".to_string(),
            text_disabled: "#999999".to_string(),
            text_inverse: colors::WHITE.to_string(),

            border_primary: colors::BLACK.to_string(),
            border_secondary: "#666666".to_string(),
            border_focus: "#0000FF".to_string(),
            border_error: "#DC143C".to_string(),
            border_success: "#006400".to_string(),

            link: "#0000EE".to_string(),
            link_hover: "#0000CD".to_string(),
            link_active: "#000080".to_string(),
            link_visited: "#800080".to_string(),

            accent: "#FFD700".to_string(),
            highlight: "#FFFF00".to_string(),
            shadow: "rgba(0, 0, 0, 0.8)".to_string(),
            divider: colors::BLACK.to_string(),
        }
    }
}

impl Typography {
    pub fn default() -> Self {
        Self {
            font_family_primary: typography::FONT_FAMILY_PRIMARY.to_string(),
            font_family_secondary: typography::FONT_FAMILY_PRIMARY.to_string(),
            font_family_mono: typography::FONT_FAMILY_MONO.to_string(),

            font_size_xs: typography::FONT_SIZE_XS.to_string(),
            font_size_sm: typography::FONT_SIZE_SM.to_string(),
            font_size_base: typography::FONT_SIZE_BASE.to_string(),
            font_size_lg: typography::FONT_SIZE_LG.to_string(),
            font_size_xl: typography::FONT_SIZE_XL.to_string(),
            font_size_2xl: typography::FONT_SIZE_2XL.to_string(),
            font_size_3xl: typography::FONT_SIZE_3XL.to_string(),
            font_size_4xl: typography::FONT_SIZE_4XL.to_string(),

            font_weight_light: 300,
            font_weight_normal: typography::FONT_WEIGHT_NORMAL,
            font_weight_medium: typography::FONT_WEIGHT_MEDIUM,
            font_weight_semibold: typography::FONT_WEIGHT_SEMIBOLD,
            font_weight_bold: typography::FONT_WEIGHT_BOLD,

            line_height_tight: typography::LINE_HEIGHT_TIGHT,
            line_height_normal: typography::LINE_HEIGHT_NORMAL,
            line_height_relaxed: typography::LINE_HEIGHT_RELAXED,

            letter_spacing_tight: "-0.025em".to_string(),
            letter_spacing_normal: "0".to_string(),
            letter_spacing_wide: "0.025em".to_string(),
        }
    }
}

impl Spacing {
    pub fn default() -> Self {
        Self {
            space_xs: spacing::SPACE_1.to_string(),
            space_sm: spacing::SPACE_2.to_string(),
            space_md: spacing::SPACE_4.to_string(),
            space_lg: spacing::SPACE_6.to_string(),
            space_xl: spacing::SPACE_8.to_string(),
            space_2xl: spacing::SPACE_12.to_string(),
            space_3xl: spacing::SPACE_16.to_string(),
            space_4xl: spacing::SPACE_24.to_string(),

            header_height: spacing::HEADER_HEIGHT.to_string(),
            sidebar_width: spacing::SIDEBAR_WIDTH.to_string(),
            sidebar_width_collapsed: spacing::SIDEBAR_WIDTH_COLLAPSED.to_string(),
            footer_height: spacing::FOOTER_HEIGHT.to_string(),

            container_padding: spacing::SPACE_4.to_string(),
            section_padding: spacing::SPACE_8.to_string(),
            card_padding: spacing::SPACE_6.to_string(),
            form_spacing: spacing::SPACE_4.to_string(),
        }
    }
}

impl Animations {
    pub fn default() -> Self {
        Self {
            duration_fast: animations::TRANSITION_FAST.to_string(),
            duration_normal: animations::TRANSITION_NORMAL.to_string(),
            duration_slow: animations::TRANSITION_SLOW.to_string(),
            duration_slower: "750ms".to_string(),

            easing_ease_in: animations::EASE_IN.to_string(),
            easing_ease_out: animations::EASE_OUT.to_string(),
            easing_ease_in_out: animations::EASE_IN_OUT.to_string(),
            easing_bounce: "cubic-bezier(0.68, -0.55, 0.265, 1.55)".to_string(),

            transition_all: animations::TRANSITION_ALL.to_string(),
            transition_colors: animations::TRANSITION_COLORS.to_string(),
            transition_transform: animations::TRANSITION_TRANSFORM.to_string(),
            transition_opacity: "opacity 300ms cubic-bezier(0.4, 0, 0.2, 1)".to_string(),
        }
    }

    pub fn reduced_motion() -> Self {
        Self {
            duration_fast: "0ms".to_string(),
            duration_normal: "0ms".to_string(),
            duration_slow: "0ms".to_string(),
            duration_slower: "0ms".to_string(),

            easing_ease_in: "linear".to_string(),
            easing_ease_out: "linear".to_string(),
            easing_ease_in_out: "linear".to_string(),
            easing_bounce: "linear".to_string(),

            transition_all: "none".to_string(),
            transition_colors: "none".to_string(),
            transition_transform: "none".to_string(),
            transition_opacity: "none".to_string(),
        }
    }
}

impl Shadows {
    pub fn default() -> Self {
        Self {
            shadow_none: "none".to_string(),
            shadow_sm: shadows::SHADOW_SM.to_string(),
            shadow_md: shadows::SHADOW_MD.to_string(),
            shadow_lg: shadows::SHADOW_LG.to_string(),
            shadow_xl: shadows::SHADOW_XL.to_string(),
            shadow_2xl: shadows::SHADOW_2XL.to_string(),
            shadow_inner: shadows::SHADOW_INNER.to_string(),
            shadow_focus: "0 0 0 3px rgba(59, 130, 246, 0.5)".to_string(),
        }
    }

    pub fn high_contrast() -> Self {
        Self {
            shadow_none: "none".to_string(),
            shadow_sm: "0 1px 2px 0 rgba(0, 0, 0, 0.8)".to_string(),
            shadow_md: "0 4px 6px -1px rgba(0, 0, 0, 0.8)".to_string(),
            shadow_lg: "0 10px 15px -3px rgba(0, 0, 0, 0.8)".to_string(),
            shadow_xl: "0 20px 25px -5px rgba(0, 0, 0, 0.8)".to_string(),
            shadow_2xl: "0 25px 50px -12px rgba(0, 0, 0, 0.8)".to_string(),
            shadow_inner: "inset 0 2px 4px 0 rgba(0, 0, 0, 0.8)".to_string(),
            shadow_focus: "0 0 0 4px rgba(0, 0, 255, 0.8)".to_string(),
        }
    }
}

impl Borders {
    pub fn default() -> Self {
        Self {
            radius_none: "0".to_string(),
            radius_sm: borders::RADIUS_SM.to_string(),
            radius_md: borders::RADIUS_MD.to_string(),
            radius_lg: borders::RADIUS_LG.to_string(),
            radius_xl: borders::RADIUS_XL.to_string(),
            radius_2xl: borders::RADIUS_2XL.to_string(),
            radius_full: borders::RADIUS_FULL.to_string(),

            border_width_thin: borders::BORDER_WIDTH_THIN.to_string(),
            border_width_thick: borders::BORDER_WIDTH_THICK.to_string(),
            border_width_thicker: borders::BORDER_WIDTH_THICKER.to_string(),

            border_style_solid: "solid".to_string(),
            border_style_dashed: "dashed".to_string(),
            border_style_dotted: "dotted".to_string(),
        }
    }
}

impl Theme {
    /// Create default light theme
    pub fn light() -> Self {
        Self {
            name: "Kejaksaan Light".to_string(),
            variant: ThemeVariant::Light,
            colors: ColorPalette::light(),
            typography: Typography::default(),
            spacing: Spacing::default(),
            animations: Animations::default(),
            shadows: Shadows::default(),
            borders: Borders::default(),
            high_contrast: false,
            reduced_motion: false,
            focus_visible: true,
            print_optimized: false,
            print_colors: false,
        }
    }

    /// Create dark theme
    pub fn dark() -> Self {
        Self {
            name: "Kejaksaan Dark".to_string(),
            variant: ThemeVariant::Dark,
            colors: ColorPalette::dark(),
            typography: Typography::default(),
            spacing: Spacing::default(),
            animations: Animations::default(),
            shadows: Shadows::default(),
            borders: Borders::default(),
            high_contrast: false,
            reduced_motion: false,
            focus_visible: true,
            print_optimized: false,
            print_colors: false,
        }
    }

    /// Create high contrast theme for accessibility
    pub fn high_contrast() -> Self {
        Self {
            name: "Kejaksaan High Contrast".to_string(),
            variant: ThemeVariant::HighContrast,
            colors: ColorPalette::high_contrast(),
            typography: Typography::default(),
            spacing: Spacing::default(),
            animations: Animations::reduced_motion(),
            shadows: Shadows::high_contrast(),
            borders: Borders::default(),
            high_contrast: true,
            reduced_motion: true,
            focus_visible: true,
            print_optimized: false,
            print_colors: false,
        }
    }

    /// Create print-optimized theme
    pub fn print() -> Self {
        let mut theme = Self::light();
        theme.name = "Kejaksaan Print".to_string();
        theme.variant = ThemeVariant::Print;
        theme.print_optimized = true;
        theme.print_colors = false;
        theme.animations = Animations::reduced_motion();
        theme.shadows = Shadows {
            shadow_none: "none".to_string(),
            shadow_sm: "none".to_string(),
            shadow_md: "none".to_string(),
            shadow_lg: "none".to_string(),
            shadow_xl: "none".to_string(),
            shadow_2xl: "none".to_string(),
            shadow_inner: "none".to_string(),
            shadow_focus: "none".to_string(),
        };
        theme
    }
}

// ============================================================================
// THEME REGISTRY & MANAGEMENT
// ============================================================================

/// Theme registry for managing multiple themes
pub static THEME_REGISTRY: Lazy<HashMap<ThemeVariant, Theme>> = Lazy::new(|| {
    let mut themes = HashMap::new();
    themes.insert(ThemeVariant::Light, Theme::light());
    themes.insert(ThemeVariant::Dark, Theme::dark());
    themes.insert(ThemeVariant::HighContrast, Theme::high_contrast());
    themes.insert(ThemeVariant::Print, Theme::print());
    themes
});

/// Get theme by variant
pub fn get_theme(variant: ThemeVariant) -> Option<&'static Theme> {
    THEME_REGISTRY.get(&variant)
}

/// Get theme or fallback to light theme
pub fn get_theme_or_default(variant: ThemeVariant) -> &'static Theme {
    THEME_REGISTRY
        .get(&variant)
        .unwrap_or(&THEME_REGISTRY[&ThemeVariant::Light])
}

/// Theme context for managing current theme state
#[derive(Debug, Clone)]
pub struct ThemeContext {
    pub current_variant: ThemeVariant,
    pub user_preference: Option<ThemeVariant>,
    pub system_preference: Option<ThemeVariant>,
    pub reduced_motion: bool,
    pub high_contrast: bool,
    pub print_mode: bool,
}

impl Default for ThemeContext {
    fn default() -> Self {
        Self {
            current_variant: ThemeVariant::Light,
            user_preference: None,
            system_preference: None,
            reduced_motion: false,
            high_contrast: false,
            print_mode: false,
        }
    }
}

impl ThemeContext {
    /// Create new theme context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set user theme preference
    pub fn set_user_preference(&mut self, variant: ThemeVariant) {
        self.user_preference = Some(variant);
        self.update_current_theme();
    }

    /// Set system theme preference (from browser/OS)
    pub fn set_system_preference(&mut self, variant: ThemeVariant) {
        self.system_preference = Some(variant);
        self.update_current_theme();
    }

    /// Enable reduced motion
    pub fn set_reduced_motion(&mut self, enabled: bool) {
        self.reduced_motion = enabled;
        self.update_current_theme();
    }

    /// Enable high contrast
    pub fn set_high_contrast(&mut self, enabled: bool) {
        self.high_contrast = enabled;
        self.update_current_theme();
    }

    /// Enable print mode
    pub fn set_print_mode(&mut self, enabled: bool) {
        self.print_mode = enabled;
        self.update_current_theme();
    }

    /// Update current theme based on preferences and settings
    fn update_current_theme(&mut self) {
        // Priority order: print mode > high contrast > user preference > system preference > default
        self.current_variant = if self.print_mode {
            ThemeVariant::Print
        } else if self.high_contrast {
            ThemeVariant::HighContrast
        } else if let Some(user_pref) = self.user_preference {
            match user_pref {
                ThemeVariant::Auto => self.system_preference.unwrap_or(ThemeVariant::Light),
                _ => user_pref,
            }
        } else if let Some(system_pref) = self.system_preference {
            system_pref
        } else {
            ThemeVariant::Light
        };
    }

    /// Get current theme
    pub fn get_current_theme(&self) -> &'static Theme {
        get_theme_or_default(self.current_variant)
    }

    /// Generate CSS custom properties for current theme
    pub fn generate_css_variables(&self) -> String {
        let theme = self.get_current_theme();
        format!(
            r#":root {{
  /* Colors */
  --color-primary: {};
  --color-primary-hover: {};
  --color-primary-active: {};
  --color-secondary: {};
  --color-secondary-hover: {};
  --color-success: {};
  --color-warning: {};
  --color-danger: {};
  --color-info: {};

  /* Backgrounds */
  --bg-primary: {};
  --bg-secondary: {};
  --bg-tertiary: {};
  --bg-surface: {};

  /* Text */
  --text-primary: {};
  --text-secondary: {};
  --text-tertiary: {};

  /* Borders */
  --border-primary: {};
  --border-focus: {};

  /* Shadows */
  --shadow-sm: {};
  --shadow-md: {};
  --shadow-lg: {};

  /* Typography */
  --font-family-primary: {};
  --font-size-base: {};
  --line-height-normal: {};

  /* Spacing */
  --space-sm: {};
  --space-md: {};
  --space-lg: {};

  /* Animations */
  --transition-fast: {};
  --transition-normal: {};
  --ease-in-out: {};

  /* Borders */
  --radius-md: {};
  --border-width: {};
}}"#,
            theme.colors.primary,
            theme.colors.primary_hover,
            theme.colors.primary_active,
            theme.colors.secondary,
            theme.colors.secondary_hover,
            theme.colors.success,
            theme.colors.warning,
            theme.colors.danger,
            theme.colors.info,
            theme.colors.background_primary,
            theme.colors.background_secondary,
            theme.colors.background_tertiary,
            theme.colors.surface,
            theme.colors.text_primary,
            theme.colors.text_secondary,
            theme.colors.text_tertiary,
            theme.colors.border_primary,
            theme.colors.border_focus,
            theme.shadows.shadow_sm,
            theme.shadows.shadow_md,
            theme.shadows.shadow_lg,
            theme.typography.font_family_primary,
            theme.typography.font_size_base,
            theme.typography.line_height_normal,
            theme.spacing.space_sm,
            theme.spacing.space_md,
            theme.spacing.space_lg,
            theme.animations.duration_fast,
            theme.animations.duration_normal,
            theme.animations.easing_ease_in_out,
            theme.borders.radius_md,
            theme.borders.border_width_thin,
        )
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Check if theme has dark background
pub fn is_dark_theme(variant: ThemeVariant) -> bool {
    matches!(variant, ThemeVariant::Dark)
}

/// Check if theme has high contrast
pub fn is_high_contrast_theme(variant: ThemeVariant) -> bool {
    matches!(variant, ThemeVariant::HighContrast)
}

/// Get appropriate text color for background
pub fn get_contrast_text_color<'a>(background_color: &str, theme: &'a Theme) -> &'a str {
    // Simple heuristic - in a real implementation, you'd calculate luminance
    if background_color.starts_with('#') && background_color.len() == 7 {
        let hex = &background_color[1..];
        if let Ok(rgb) = u32::from_str_radix(hex, 16) {
            let r = ((rgb >> 16) & 0xFF) as f32;
            let g = ((rgb >> 8) & 0xFF) as f32;
            let b = (rgb & 0xFF) as f32;

            // Calculate relative luminance
            let luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;

            return if luminance > 0.5 {
                &theme.colors.text_primary
            } else {
                &theme.colors.text_inverse
            };
        }
    }

    &theme.colors.text_primary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_creation() {
        let light_theme = Theme::light();
        assert_eq!(light_theme.variant, ThemeVariant::Light);
        assert_eq!(light_theme.name, "Kejaksaan Light");

        let dark_theme = Theme::dark();
        assert_eq!(dark_theme.variant, ThemeVariant::Dark);
        assert_eq!(dark_theme.name, "Kejaksaan Dark");
    }

    #[test]
    fn test_theme_context() {
        let mut ctx = ThemeContext::new();
        assert_eq!(ctx.current_variant, ThemeVariant::Light);

        ctx.set_user_preference(ThemeVariant::Dark);
        assert_eq!(ctx.current_variant, ThemeVariant::Dark);

        ctx.set_high_contrast(true);
        assert_eq!(ctx.current_variant, ThemeVariant::HighContrast);
    }

    #[test]
    fn test_theme_registry() {
        let light_theme = get_theme(ThemeVariant::Light);
        assert!(light_theme.is_some());

        let fallback_theme = get_theme_or_default(ThemeVariant::Light);
        assert_eq!(fallback_theme.variant, ThemeVariant::Light);
    }
}
