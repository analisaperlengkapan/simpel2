// ========================================
// UTILITY CLASSES & STYLES
// ========================================

/// Common CSS utility classes for consistent styling
pub struct TailwindClasses;

impl TailwindClasses {
    // Layout Classes
    pub const CONTAINER: &'static str = "container mx-auto px-4";
    pub const FLEX_CENTER: &'static str = "flex items-center justify-center";
    pub const FLEX_BETWEEN: &'static str = "flex items-center justify-between";
    pub const FLEX_START: &'static str = "flex items-center justify-start";

    // Spacing Classes
    pub const SPACE_X_2: &'static str = "space-x-2";
    pub const SPACE_X_3: &'static str = "space-x-3";
    pub const SPACE_X_4: &'static str = "space-x-4";
    pub const SPACE_Y_2: &'static str = "space-y-2";
    pub const SPACE_Y_3: &'static str = "space-y-3";
    pub const SPACE_Y_4: &'static str = "space-y-4";

    // Text Classes
    pub const TEXT_SM: &'static str = "text-sm";
    pub const TEXT_BASE: &'static str = "text-base";
    pub const TEXT_LG: &'static str = "text-lg";
    pub const TEXT_XL: &'static str = "text-xl";
    pub const TEXT_2XL: &'static str = "text-2xl";

    // Font Weight Classes
    pub const FONT_NORMAL: &'static str = "font-normal";
    pub const FONT_MEDIUM: &'static str = "font-medium";
    pub const FONT_SEMIBOLD: &'static str = "font-semibold";
    pub const FONT_BOLD: &'static str = "font-bold";

    // Color Classes
    pub const TEXT_GRAY_500: &'static str = "text-gray-500";
    pub const TEXT_GRAY_600: &'static str = "text-gray-600";
    pub const TEXT_GRAY_700: &'static str = "text-gray-700";
    pub const TEXT_GRAY_900: &'static str = "text-gray-900";
    pub const TEXT_BLUE_600: &'static str = "text-blue-600";
    pub const TEXT_BLUE_700: &'static str = "text-blue-700";

    // Background Classes
    pub const BG_WHITE: &'static str = "bg-white";
    pub const BG_GRAY_50: &'static str = "bg-gray-50";
    pub const BG_GRAY_100: &'static str = "bg-gray-100";
    pub const BG_BLUE_50: &'static str = "bg-blue-50";
    pub const BG_BLUE_600: &'static str = "bg-blue-600";

    // Border Classes
    pub const BORDER: &'static str = "border";
    pub const BORDER_GRAY_200: &'static str = "border-gray-200";
    pub const BORDER_GRAY_300: &'static str = "border-gray-300";
    pub const ROUNDED: &'static str = "rounded";
    pub const ROUNDED_LG: &'static str = "rounded-lg";
    pub const ROUNDED_FULL: &'static str = "rounded-full";

    // Shadow Classes
    pub const SHADOW: &'static str = "shadow";
    pub const SHADOW_SM: &'static str = "shadow-sm";
    pub const SHADOW_MD: &'static str = "shadow-md";
    pub const SHADOW_LG: &'static str = "shadow-lg";
    pub const SHADOW_XL: &'static str = "shadow-xl";

    // Interactive Classes
    pub const HOVER_BG_GRAY_50: &'static str = "hover:bg-gray-50";
    pub const HOVER_BG_BLUE_700: &'static str = "hover:bg-blue-700";
    pub const HOVER_TEXT_BLUE_600: &'static str = "hover:text-blue-600";
    pub const FOCUS_RING: &'static str =
        "focus:ring-2 focus:ring-blue-500 focus:border-transparent";
    pub const TRANSITION: &'static str = "transition-colors duration-200";

    // Cursor Classes
    pub const CURSOR_POINTER: &'static str = "cursor-pointer";
    pub const CURSOR_NOT_ALLOWED: &'static str = "cursor-not-allowed";
}

/// Common component styling patterns
pub struct ComponentStyles;

impl ComponentStyles {
    // Button Styles
    pub fn button_base() -> &'static str {
        "inline-flex items-center px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2"
    }

    pub fn button_primary() -> &'static str {
        "bg-blue-600 text-white hover:bg-blue-700 focus:ring-blue-500"
    }

    pub fn button_secondary() -> &'static str {
        "bg-gray-600 text-white hover:bg-gray-700 focus:ring-gray-500"
    }

    pub fn button_outline() -> &'static str {
        "bg-transparent border border-gray-300 text-gray-700 hover:bg-gray-50 focus:ring-gray-500"
    }

    // Input Styles
    pub fn input_base() -> &'static str {
        "block w-full px-3 py-2 border border-gray-300 rounded-lg shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent"
    }

    pub fn input_disabled() -> &'static str {
        "disabled:bg-gray-50 disabled:text-gray-500 disabled:cursor-not-allowed"
    }

    // Card Styles
    pub fn card_base() -> &'static str {
        "bg-white rounded-lg shadow-sm border border-gray-200"
    }

    pub fn card_header() -> &'static str {
        "px-6 py-4 border-b border-gray-200"
    }

    pub fn card_body() -> &'static str {
        "px-6 py-4"
    }

    // Table Styles
    pub fn table_base() -> &'static str {
        "min-w-full divide-y divide-gray-200"
    }

    pub fn table_header() -> &'static str {
        "bg-gray-50"
    }

    pub fn table_header_cell() -> &'static str {
        "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
    }

    pub fn table_body() -> &'static str {
        "bg-white divide-y divide-gray-200"
    }

    pub fn table_row() -> &'static str {
        "hover:bg-gray-50"
    }

    pub fn table_cell() -> &'static str {
        "px-6 py-4 whitespace-nowrap text-sm text-gray-900"
    }

    // Modal Styles
    pub fn modal_overlay() -> &'static str {
        "fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity"
    }

    pub fn modal_container() -> &'static str {
        "fixed inset-0 z-50 overflow-y-auto"
    }

    pub fn modal_content() -> &'static str {
        "inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:w-full"
    }
}

/// Color scheme definitions for consistent theming
pub struct ColorScheme;

impl ColorScheme {
    // Primary Colors (Kejaksaan Blue)
    pub const PRIMARY_50: &'static str = "bg-blue-50";
    pub const PRIMARY_100: &'static str = "bg-blue-100";
    pub const PRIMARY_500: &'static str = "bg-blue-500";
    pub const PRIMARY_600: &'static str = "bg-blue-600";
    pub const PRIMARY_700: &'static str = "bg-blue-700";
    pub const PRIMARY_900: &'static str = "bg-blue-900";

    pub const PRIMARY_TEXT_500: &'static str = "text-blue-500";
    pub const PRIMARY_TEXT_600: &'static str = "text-blue-600";
    pub const PRIMARY_TEXT_700: &'static str = "text-blue-700";
    pub const PRIMARY_TEXT_900: &'static str = "text-blue-900";

    // Success Colors (Green)
    pub const SUCCESS_50: &'static str = "bg-green-50";
    pub const SUCCESS_100: &'static str = "bg-green-100";
    pub const SUCCESS_500: &'static str = "bg-green-500";
    pub const SUCCESS_600: &'static str = "bg-green-600";
    pub const SUCCESS_700: &'static str = "bg-green-700";

    pub const SUCCESS_TEXT_600: &'static str = "text-green-600";
    pub const SUCCESS_TEXT_700: &'static str = "text-green-700";
    pub const SUCCESS_TEXT_800: &'static str = "text-green-800";

    // Warning Colors (Yellow/Amber)
    pub const WARNING_50: &'static str = "bg-yellow-50";
    pub const WARNING_100: &'static str = "bg-yellow-100";
    pub const WARNING_500: &'static str = "bg-yellow-500";
    pub const WARNING_600: &'static str = "bg-yellow-600";

    pub const WARNING_TEXT_600: &'static str = "text-yellow-600";
    pub const WARNING_TEXT_800: &'static str = "text-yellow-800";

    // Error Colors (Red)
    pub const ERROR_50: &'static str = "bg-red-50";
    pub const ERROR_100: &'static str = "bg-red-100";
    pub const ERROR_500: &'static str = "bg-red-500";
    pub const ERROR_600: &'static str = "bg-red-600";

    pub const ERROR_TEXT_600: &'static str = "text-red-600";
    pub const ERROR_TEXT_800: &'static str = "text-red-800";

    // Neutral Colors (Gray)
    pub const NEUTRAL_50: &'static str = "bg-gray-50";
    pub const NEUTRAL_100: &'static str = "bg-gray-100";
    pub const NEUTRAL_200: &'static str = "bg-gray-200";
    pub const NEUTRAL_300: &'static str = "bg-gray-300";
    pub const NEUTRAL_500: &'static str = "bg-gray-500";
    pub const NEUTRAL_600: &'static str = "bg-gray-600";
    pub const NEUTRAL_700: &'static str = "bg-gray-700";
    pub const NEUTRAL_900: &'static str = "bg-gray-900";

    pub const NEUTRAL_TEXT_500: &'static str = "text-gray-500";
    pub const NEUTRAL_TEXT_600: &'static str = "text-gray-600";
    pub const NEUTRAL_TEXT_700: &'static str = "text-gray-700";
    pub const NEUTRAL_TEXT_900: &'static str = "text-gray-900";
}

/// Responsive breakpoint utilities
pub struct Breakpoints;

impl Breakpoints {
    pub const SM: &'static str = "sm:"; // 640px
    pub const MD: &'static str = "md:"; // 768px
    pub const LG: &'static str = "lg:"; // 1024px
    pub const XL: &'static str = "xl:"; // 1280px
    pub const XXL: &'static str = "2xl:"; // 1536px
}

/// Animation and transition utilities
pub struct Animations;

impl Animations {
    pub const FADE_IN: &'static str = "animate-fade-in";
    pub const FADE_OUT: &'static str = "animate-fade-out";
    pub const SLIDE_IN_RIGHT: &'static str = "animate-slide-in-right";
    pub const SLIDE_IN_LEFT: &'static str = "animate-slide-in-left";
    pub const SPIN: &'static str = "animate-spin";
    pub const PULSE: &'static str = "animate-pulse";
    pub const BOUNCE: &'static str = "animate-bounce";

    pub const TRANSITION_ALL: &'static str = "transition-all duration-300";
    pub const TRANSITION_COLORS: &'static str = "transition-colors duration-200";
    pub const TRANSITION_TRANSFORM: &'static str = "transition-transform duration-200";
}

/// Focus and accessibility utilities
pub struct Accessibility;

impl Accessibility {
    pub const FOCUS_VISIBLE: &'static str =
        "focus:outline-none focus-visible:ring-2 focus-visible:ring-blue-500";
    pub const SR_ONLY: &'static str = "sr-only";
    pub const NOT_SR_ONLY: &'static str = "not-sr-only";
}
