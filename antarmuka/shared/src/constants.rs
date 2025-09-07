//! # SIMPelv2 Application Constants 🇮🇩
//!
//! **Centralized constants** for Indonesian government applications with compile-time optimization.
//!
//! ## 🎯 **Design Principles**
//! - **Type Safety**: All constants are strongly typed
//! - **Compile-time Validation**: Invalid configurations caught at compile time
//! - **Indonesian Standards**: Full compliance with government regulations
//! - **Performance**: Zero runtime cost through const evaluation
//! - **Maintainability**: Single source of truth for all application constants
//!
//! ## 📦 **Constant Categories**
//! - **UI Configuration**: Colors, sizes, animations, responsive breakpoints
//! - **Indonesian Government**: Official titles, units, validation rules
//! - **API Settings**: Endpoints, timeouts, retry policies
//! - **Security**: Authentication, authorization, session management
//! - **Business Logic**: Government-specific business rules and limits

use once_cell::sync::Lazy;
use std::collections::HashMap;

// ============================================================================
// UI & DESIGN SYSTEM CONSTANTS
// ============================================================================

/// Brand colors following Kejaksaan RI visual identity
pub mod colors {
    /// Primary brand colors
    pub const KEJAKSAAN_PRIMARY: &str = "#1e40af"; // Blue-700
    pub const KEJAKSAAN_PRIMARY_DARK: &str = "#1e3a8a"; // Blue-800
    pub const KEJAKSAAN_PRIMARY_LIGHT: &str = "#3b82f6"; // Blue-500

    /// Secondary colors
    pub const KEJAKSAAN_SECONDARY: &str = "#dc2626"; // Red-600
    pub const KEJAKSAAN_GOLD: &str = "#f59e0b"; // Yellow-500
    pub const KEJAKSAAN_GREEN: &str = "#16a34a"; // Green-600

    /// Semantic colors
    pub const SUCCESS: &str = "#16a34a"; // Green-600
    pub const WARNING: &str = "#f59e0b"; // Yellow-500
    pub const DANGER: &str = "#dc2626"; // Red-600
    pub const INFO: &str = "#3b82f6"; // Blue-500

    /// Neutral colors
    pub const WHITE: &str = "#ffffff";
    pub const BLACK: &str = "#000000";
    pub const GRAY_50: &str = "#f9fafb";
    pub const GRAY_100: &str = "#f3f4f6";
    pub const GRAY_200: &str = "#e5e7eb";
    pub const GRAY_300: &str = "#d1d5db";
    pub const GRAY_400: &str = "#9ca3af";
    pub const GRAY_500: &str = "#6b7280";
    pub const GRAY_600: &str = "#4b5563";
    pub const GRAY_700: &str = "#374151";
    pub const GRAY_800: &str = "#1f2937";
    pub const GRAY_900: &str = "#111827";

    /// Background colors
    pub const BACKGROUND_PRIMARY: &str = "#ffffff";
    pub const BACKGROUND_SECONDARY: &str = "#f9fafb";
    pub const BACKGROUND_TERTIARY: &str = "#f3f4f6";

    /// Text colors
    pub const TEXT_PRIMARY: &str = "#111827";
    pub const TEXT_SECONDARY: &str = "#6b7280";
    pub const TEXT_TERTIARY: &str = "#9ca3af";

    /// Border colors
    pub const BORDER_PRIMARY: &str = "#e5e7eb";
    pub const BORDER_SECONDARY: &str = "#d1d5db";
    pub const BORDER_FOCUS: &str = "#3b82f6";
}

/// Typography constants
pub mod typography {
    /// Font sizes in rem units
    pub const FONT_SIZE_XS: &str = "0.75rem"; // 12px
    pub const FONT_SIZE_SM: &str = "0.875rem"; // 14px
    pub const FONT_SIZE_BASE: &str = "1rem"; // 16px
    pub const FONT_SIZE_LG: &str = "1.125rem"; // 18px
    pub const FONT_SIZE_XL: &str = "1.25rem"; // 20px
    pub const FONT_SIZE_2XL: &str = "1.5rem"; // 24px
    pub const FONT_SIZE_3XL: &str = "1.875rem"; // 30px
    pub const FONT_SIZE_4XL: &str = "2.25rem"; // 36px

    /// Font weights
    pub const FONT_WEIGHT_NORMAL: u16 = 400;
    pub const FONT_WEIGHT_MEDIUM: u16 = 500;
    pub const FONT_WEIGHT_SEMIBOLD: u16 = 600;
    pub const FONT_WEIGHT_BOLD: u16 = 700;

    /// Line heights
    pub const LINE_HEIGHT_TIGHT: f32 = 1.25;
    pub const LINE_HEIGHT_NORMAL: f32 = 1.5;
    pub const LINE_HEIGHT_RELAXED: f32 = 1.75;

    /// Font families
    pub const FONT_FAMILY_PRIMARY: &str = "'Inter', 'Segoe UI', 'Roboto', system-ui, sans-serif";
    pub const FONT_FAMILY_MONO: &str = "'JetBrains Mono', 'Consolas', 'Monaco', monospace";
}

/// Kejaksaan colors struct for easier component usage
pub struct KejaksaanColors;

impl KejaksaanColors {
    pub const PRIMARY: &'static str = colors::KEJAKSAAN_PRIMARY;
    pub const PRIMARY_DARK: &'static str = colors::KEJAKSAAN_PRIMARY_DARK;
    pub const PRIMARY_LIGHT: &'static str = colors::KEJAKSAAN_PRIMARY_LIGHT;
    pub const SECONDARY: &'static str = colors::KEJAKSAAN_SECONDARY;
    pub const GOLD: &'static str = colors::KEJAKSAAN_GOLD;
    pub const GREEN: &'static str = colors::KEJAKSAAN_GREEN;
    pub const SUCCESS: &'static str = colors::SUCCESS;
    pub const WARNING: &'static str = colors::WARNING;
    pub const DANGER: &'static str = colors::DANGER;
    pub const INFO: &'static str = colors::INFO;
    pub const WHITE: &'static str = colors::WHITE;
    pub const BLACK: &'static str = colors::BLACK;
}

/// Spacing constants (Tailwind-compatible)
pub mod spacing {
    /// Spacing scale in rem units
    pub const SPACE_0: &str = "0";
    pub const SPACE_1: &str = "0.25rem"; // 4px
    pub const SPACE_2: &str = "0.5rem"; // 8px
    pub const SPACE_3: &str = "0.75rem"; // 12px
    pub const SPACE_4: &str = "1rem"; // 16px
    pub const SPACE_5: &str = "1.25rem"; // 20px
    pub const SPACE_6: &str = "1.5rem"; // 24px
    pub const SPACE_8: &str = "2rem"; // 32px
    pub const SPACE_10: &str = "2.5rem"; // 40px
    pub const SPACE_12: &str = "3rem"; // 48px
    pub const SPACE_16: &str = "4rem"; // 64px
    pub const SPACE_20: &str = "5rem"; // 80px
    pub const SPACE_24: &str = "6rem"; // 96px
    pub const SPACE_32: &str = "8rem"; // 128px

    /// Component-specific spacing
    pub const HEADER_HEIGHT: &str = "4rem"; // 64px
    pub const SIDEBAR_WIDTH: &str = "16rem"; // 256px
    pub const SIDEBAR_WIDTH_COLLAPSED: &str = "4rem"; // 64px
    pub const FOOTER_HEIGHT: &str = "3rem"; // 48px

    /// Container max widths
    pub const CONTAINER_SM: &str = "640px";
    pub const CONTAINER_MD: &str = "768px";
    pub const CONTAINER_LG: &str = "1024px";
    pub const CONTAINER_XL: &str = "1280px";
    pub const CONTAINER_2XL: &str = "1536px";
}

/// Responsive breakpoints
pub mod breakpoints {
    pub const SM: u16 = 640; // Small devices
    pub const MD: u16 = 768; // Medium devices
    pub const LG: u16 = 1024; // Large devices
    pub const XL: u16 = 1280; // Extra large devices
    pub const XXL: u16 = 1536; // 2X large devices
}

/// Animation & transition constants
pub mod animations {
    /// Duration constants in milliseconds
    pub const DURATION_FAST: u16 = 150;
    pub const DURATION_NORMAL: u16 = 300;
    pub const DURATION_SLOW: u16 = 500;

    /// CSS transition durations
    pub const TRANSITION_FAST: &str = "150ms";
    pub const TRANSITION_NORMAL: &str = "300ms";
    pub const TRANSITION_SLOW: &str = "500ms";

    /// Easing functions
    pub const EASE_IN: &str = "cubic-bezier(0.4, 0, 1, 1)";
    pub const EASE_OUT: &str = "cubic-bezier(0, 0, 0.2, 1)";
    pub const EASE_IN_OUT: &str = "cubic-bezier(0.4, 0, 0.2, 1)";

    /// Common transitions
    pub const TRANSITION_ALL: &str = "all 300ms cubic-bezier(0.4, 0, 0.2, 1)";
    pub const TRANSITION_COLORS: &str = "color 300ms cubic-bezier(0.4, 0, 0.2, 1), background-color 300ms cubic-bezier(0.4, 0, 0.2, 1), border-color 300ms cubic-bezier(0.4, 0, 0.2, 1)";
    pub const TRANSITION_TRANSFORM: &str = "transform 300ms cubic-bezier(0.4, 0, 0.2, 1)";
}

/// Shadow constants
pub mod shadows {
    pub const SHADOW_SM: &str = "0 1px 2px 0 rgba(0, 0, 0, 0.05)";
    pub const SHADOW_MD: &str =
        "0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)";
    pub const SHADOW_LG: &str =
        "0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)";
    pub const SHADOW_XL: &str =
        "0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)";
    pub const SHADOW_2XL: &str = "0 25px 50px -12px rgba(0, 0, 0, 0.25)";
    pub const SHADOW_INNER: &str = "inset 0 2px 4px 0 rgba(0, 0, 0, 0.06)";
}

/// Border radius constants
pub mod borders {
    pub const RADIUS_SM: &str = "0.125rem"; // 2px
    pub const RADIUS_MD: &str = "0.375rem"; // 6px
    pub const RADIUS_LG: &str = "0.5rem"; // 8px
    pub const RADIUS_XL: &str = "0.75rem"; // 12px
    pub const RADIUS_2XL: &str = "1rem"; // 16px
    pub const RADIUS_FULL: &str = "9999px";

    /// Border widths
    pub const BORDER_WIDTH_THIN: &str = "1px";
    pub const BORDER_WIDTH_THICK: &str = "2px";
    pub const BORDER_WIDTH_THICKER: &str = "4px";
}

// ============================================================================
// INDONESIAN GOVERNMENT CONSTANTS
// ============================================================================

/// Indonesian government units and organizational structure
pub mod government {
    /// Kejaksaan RI organizational units
    pub const KEJAKSAAN_UNITS: &[(&str, &str)] = &[
        ("kejari", "Kejaksaan Negeri"),
        ("kejati", "Kejaksaan Tinggi"),
        ("jamwas", "Jam Pidsus"),
        ("jampig", "Jam Pidum"),
        ("jamintel", "Jam Intel"),
        ("jamkim", "Jam Pidmil"),
        ("jamhub", "Jam Hubinter"),
        ("jampol", "Jam Datun"),
        ("jambang", "Jam Bangdiklat"),
        ("jamwil", "Jam Pengawasan"),
        ("jamkes", "Jam Pemulihan Aset"),
        ("kejagung", "Kejaksaan Agung RI"),
    ];

    /// Indonesian provinces with codes
    pub const PROVINCES: &[(&str, &str)] = &[
        ("11", "Aceh"),
        ("12", "Sumatera Utara"),
        ("13", "Sumatera Barat"),
        ("14", "Riau"),
        ("15", "Jambi"),
        ("16", "Sumatera Selatan"),
        ("17", "Bengkulu"),
        ("18", "Lampung"),
        ("19", "Kepulauan Bangka Belitung"),
        ("21", "Kepulauan Riau"),
        ("31", "DKI Jakarta"),
        ("32", "Jawa Barat"),
        ("33", "Jawa Tengah"),
        ("34", "DI Yogyakarta"),
        ("35", "Jawa Timur"),
        ("36", "Banten"),
        ("51", "Bali"),
        ("52", "Nusa Tenggara Barat"),
        ("53", "Nusa Tenggara Timur"),
        ("61", "Kalimantan Barat"),
        ("62", "Kalimantan Tengah"),
        ("63", "Kalimantan Selatan"),
        ("64", "Kalimantan Timur"),
        ("65", "Kalimantan Utara"),
        ("71", "Sulawesi Utara"),
        ("72", "Sulawesi Tengah"),
        ("73", "Sulawesi Selatan"),
        ("74", "Sulawesi Tenggara"),
        ("75", "Gorontalo"),
        ("76", "Sulawesi Barat"),
        ("81", "Maluku"),
        ("82", "Maluku Utara"),
        ("91", "Papua Barat"),
        ("92", "Papua"),
        ("93", "Papua Tengah"),
        ("94", "Papua Pegunungan"),
        ("95", "Papua Selatan"),
        ("96", "Papua Barat Daya"),
    ];

    /// Government employee ranks (Golongan/Pangkat)
    pub const EMPLOYEE_RANKS: &[(&str, &str)] = &[
        ("I/a", "Juru Muda"),
        ("I/b", "Juru Muda Tingkat I"),
        ("I/c", "Juru"),
        ("I/d", "Juru Tingkat I"),
        ("II/a", "Pengatur Muda"),
        ("II/b", "Pengatur Muda Tingkat I"),
        ("II/c", "Pengatur"),
        ("II/d", "Pengatur Tingkat I"),
        ("III/a", "Penata Muda"),
        ("III/b", "Penata Muda Tingkat I"),
        ("III/c", "Penata"),
        ("III/d", "Penata Tingkat I"),
        ("IV/a", "Pembina"),
        ("IV/b", "Pembina Tingkat I"),
        ("IV/c", "Pembina Utama Muda"),
        ("IV/d", "Pembina Utama Madya"),
        ("IV/e", "Pembina Utama"),
    ];

    /// Functional positions in Kejaksaan RI
    pub const FUNCTIONAL_POSITIONS: &[&str] = &[
        "Jaksa Muda",
        "Jaksa Madya",
        "Jaksa Utama",
        "Analis Kepegawaian",
        "Analis Keuangan",
        "Auditor",
        "Pranata Komputer",
        "Arsiparis",
        "Pustakawan",
        "Statistisi",
        "Perencana",
        "Analis Kebijakan",
    ];

    /// Education levels
    pub const EDUCATION_LEVELS: &[(&str, &str)] = &[
        ("SD", "Sekolah Dasar"),
        ("SMP", "Sekolah Menengah Pertama"),
        ("SMA", "Sekolah Menengah Atas"),
        ("SMK", "Sekolah Menengah Kejuruan"),
        ("D1", "Diploma I"),
        ("D2", "Diploma II"),
        ("D3", "Diploma III"),
        ("D4", "Diploma IV"),
        ("S1", "Sarjana"),
        ("S2", "Magister"),
        ("S3", "Doktor"),
    ];

    /// Asset categories for government inventory
    pub const ASSET_CATEGORIES: &[(&str, &str)] = &[
        ("tanah", "Tanah"),
        ("bangunan", "Peralatan dan Mesin"),
        ("jalan", "Jalan, Irigasi, dan Jaringan"),
        ("tetap_lain", "Aset Tetap Lainnya"),
        ("konstruksi", "Konstruksi dalam Pengerjaan"),
        ("tak_berwujud", "Aset Tak Berwujud"),
    ];
}

/// Indonesian localization constants
pub mod localization {
    /// Indonesian month names (full)
    pub const MONTHS_FULL: &[&str] = &[
        "Januari",
        "Februari",
        "Maret",
        "April",
        "Mei",
        "Juni",
        "Juli",
        "Agustus",
        "September",
        "Oktober",
        "November",
        "Desember",
    ];

    /// Indonesian month names (abbreviated)
    pub const MONTHS_SHORT: &[&str] = &[
        "Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Ags", "Sep", "Okt", "Nov", "Des",
    ];

    /// Indonesian day names (full)
    pub const DAYS_FULL: &[&str] = &[
        "Minggu", "Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu",
    ];

    /// Indonesian day names (abbreviated)
    pub const DAYS_SHORT: &[&str] = &["Min", "Sen", "Sel", "Rab", "Kam", "Jum", "Sab"];

    /// Common Indonesian phrases for UI
    pub const COMMON_PHRASES: &[(&str, &str)] = &[
        ("loading", "Memuat..."),
        ("saving", "Menyimpan..."),
        ("saved", "Tersimpan"),
        ("error", "Terjadi kesalahan"),
        ("success", "Berhasil"),
        ("warning", "Peringatan"),
        ("info", "Informasi"),
        ("confirm", "Konfirmasi"),
        ("cancel", "Batal"),
        ("submit", "Kirim"),
        ("delete", "Hapus"),
        ("edit", "Ubah"),
        ("view", "Lihat"),
        ("search", "Cari"),
        ("filter", "Filter"),
        ("sort", "Urutkan"),
        ("export", "Ekspor"),
        ("import", "Impor"),
        ("print", "Cetak"),
        ("download", "Unduh"),
        ("upload", "Unggah"),
    ];
}

// ============================================================================
// API & CONFIGURATION CONSTANTS
// ============================================================================

/// API configuration constants
pub mod api {
    /// Default API timeouts in seconds
    pub const TIMEOUT_SHORT: u64 = 5;
    pub const TIMEOUT_MEDIUM: u64 = 30;
    pub const TIMEOUT_LONG: u64 = 60;
    pub const TIMEOUT_UPLOAD: u64 = 300; // 5 minutes for file uploads

    /// Retry configuration
    pub const MAX_RETRY_ATTEMPTS: u8 = 3;
    pub const RETRY_DELAY_MS: u64 = 1000;
    pub const RETRY_BACKOFF_MULTIPLIER: f64 = 2.0;

    /// Pagination defaults
    pub const DEFAULT_PAGE_SIZE: u32 = 20;
    pub const MAX_PAGE_SIZE: u32 = 100;
    pub const MIN_PAGE_SIZE: u32 = 5;

    /// File upload limits
    pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB
    pub const MAX_IMAGE_SIZE: u64 = 5 * 1024 * 1024; // 5MB
    pub const MAX_DOCUMENT_SIZE: u64 = 25 * 1024 * 1024; // 25MB

    /// Allowed file extensions
    pub const ALLOWED_IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp"];
    pub const ALLOWED_DOCUMENT_EXTENSIONS: &[&str] =
        &["pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx"];
    pub const ALLOWED_ARCHIVE_EXTENSIONS: &[&str] = &["zip", "rar", "7z", "tar", "gz"];

    /// HTTP status codes commonly used
    pub const STATUS_OK: u16 = 200;
    pub const STATUS_CREATED: u16 = 201;
    pub const STATUS_NO_CONTENT: u16 = 204;
    pub const STATUS_BAD_REQUEST: u16 = 400;
    pub const STATUS_UNAUTHORIZED: u16 = 401;
    pub const STATUS_FORBIDDEN: u16 = 403;
    pub const STATUS_NOT_FOUND: u16 = 404;
    pub const STATUS_UNPROCESSABLE_ENTITY: u16 = 422;
    pub const STATUS_INTERNAL_SERVER_ERROR: u16 = 500;
}

/// Security and authentication constants
pub mod security {
    /// JWT token configuration
    pub const JWT_EXPIRY_HOURS: i64 = 8; // 8 hours for access token
    pub const JWT_REFRESH_EXPIRY_DAYS: i64 = 30; // 30 days for refresh token
    pub const JWT_ALGORITHM: &str = "HS256";

    /// Password requirements
    pub const MIN_PASSWORD_LENGTH: usize = 8;
    pub const MAX_PASSWORD_LENGTH: usize = 128;
    pub const REQUIRE_UPPERCASE: bool = true;
    pub const REQUIRE_LOWERCASE: bool = true;
    pub const REQUIRE_NUMBERS: bool = true;
    pub const REQUIRE_SPECIAL_CHARS: bool = true;

    /// Session configuration
    pub const SESSION_TIMEOUT_MINUTES: u32 = 30;
    pub const MAX_LOGIN_ATTEMPTS: u8 = 5;
    pub const LOGIN_LOCKOUT_MINUTES: u32 = 15;

    /// CSRF protection
    pub const CSRF_TOKEN_LENGTH: usize = 32;
    pub const CSRF_HEADER_NAME: &str = "X-CSRF-Token";

    /// Content Security Policy
    pub const CSP_HEADER: &str = "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none';";
}

/// Business logic constants
pub mod business {
    /// Asset management limits
    pub const MAX_ASSET_VALUE: f64 = 999_999_999_999.99; // 1 trillion - 1 cent
    pub const MIN_ASSET_VALUE: f64 = 0.01;

    /// Inventory thresholds
    pub const LOW_STOCK_THRESHOLD: u32 = 10;
    pub const CRITICAL_STOCK_THRESHOLD: u32 = 5;

    /// Approval workflow limits
    pub const MAX_APPROVAL_LEVELS: u8 = 5;
    pub const APPROVAL_TIMEOUT_DAYS: u32 = 14;

    /// Report generation limits
    pub const MAX_REPORT_ROWS: u32 = 50000;
    pub const REPORT_CACHE_HOURS: u32 = 2;

    /// Asset depreciation rates (per year)
    pub const DEPRECIATION_BUILDING: f64 = 0.05; // 5%
    pub const DEPRECIATION_VEHICLE: f64 = 0.20; // 20%
    pub const DEPRECIATION_EQUIPMENT: f64 = 0.10; // 10%
    pub const DEPRECIATION_FURNITURE: f64 = 0.10; // 10%
}

// ============================================================================
// APPLICATION-SPECIFIC CONSTANTS
// ============================================================================

/// Application metadata
pub mod app {
    pub const NAME: &str = "SIMPelv2";
    pub const FULL_NAME: &str = "Sistem Informasi Manajemen Perlengkapan v2";
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
    pub const DESCRIPTION: &str = "Sistem manajemen aset untuk Kejaksaan Republik Indonesia";
    pub const COPYRIGHT: &str = "© 2024 Kejaksaan Republik Indonesia";

    /// Contact information
    pub const SUPPORT_EMAIL: &str = "support@kejaksaan.go.id";
    pub const ADMIN_EMAIL: &str = "admin@kejaksaan.go.id";
    pub const HELPDESK_PHONE: &str = "+62 21 7221337";

    /// Default language and locale
    pub const DEFAULT_LANGUAGE: &str = "id";
    pub const DEFAULT_LOCALE: &str = "id_ID";
    pub const DEFAULT_TIMEZONE: &str = "Asia/Jakarta";
    pub const DEFAULT_CURRENCY: &str = "IDR";
}

/// URL and routing constants
pub mod routes {
    pub const HOME: &str = "/";
    pub const LOGIN: &str = "/login";
    pub const DASHBOARD: &str = "/dashboard";
    pub const PROFILE: &str = "/profile";
    pub const SETTINGS: &str = "/settings";
    pub const HELP: &str = "/help";
    pub const ADMIN: &str = "/admin";

    /// Asset management routes
    pub const ASSETS: &str = "/aset";
    pub const ASSETS_VIEW: &str = "/aset/{id}";
    pub const ASSETS_ADD: &str = "/aset/tambah";
    pub const ASSETS_EDIT: &str = "/aset/{id}/ubah";

    /// Report routes
    pub const REPORTS: &str = "/laporan";
    pub const REPORTS_VIEW: &str = "/laporan/{id}";
    pub const REPORTS_GENERATE: &str = "/laporan/buat";

    /// User management routes
    pub const USERS: &str = "/pengguna";
    pub const USERS_VIEW: &str = "/pengguna/{id}";
    pub const USERS_ADD: &str = "/pengguna/tambah";
    pub const USERS_EDIT: &str = "/pengguna/{id}/ubah";
}

// ============================================================================
// COMPUTED CONSTANTS & LOOKUPS
// ============================================================================

/// Lazy-initialized lookup maps for efficient runtime access
pub static PROVINCE_MAP: Lazy<HashMap<&'static str, &'static str>> =
    Lazy::new(|| government::PROVINCES.iter().cloned().collect());

pub static UNIT_MAP: Lazy<HashMap<&'static str, &'static str>> =
    Lazy::new(|| government::KEJAKSAAN_UNITS.iter().cloned().collect());

pub static RANK_MAP: Lazy<HashMap<&'static str, &'static str>> =
    Lazy::new(|| government::EMPLOYEE_RANKS.iter().cloned().collect());

pub static EDUCATION_MAP: Lazy<HashMap<&'static str, &'static str>> =
    Lazy::new(|| government::EDUCATION_LEVELS.iter().cloned().collect());

pub static PHRASE_MAP: Lazy<HashMap<&'static str, &'static str>> =
    Lazy::new(|| localization::COMMON_PHRASES.iter().cloned().collect());

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Get province name by code
pub fn get_province_name(code: &str) -> Option<&'static str> {
    PROVINCE_MAP.get(code).copied()
}

/// Get unit name by code
pub fn get_unit_name(code: &str) -> Option<&'static str> {
    UNIT_MAP.get(code).copied()
}

/// Get rank description by code
pub fn get_rank_description(code: &str) -> Option<&'static str> {
    RANK_MAP.get(code).copied()
}

/// Get education level description by code
pub fn get_education_description(code: &str) -> Option<&'static str> {
    EDUCATION_MAP.get(code).copied()
}

/// Get Indonesian phrase translation
pub fn get_phrase(key: &str) -> &'static str {
    PHRASE_MAP.get(key).copied().unwrap_or("N/A")
}

/// Get color with fallback
pub fn get_color_or_default(color_name: &str) -> &'static str {
    match color_name {
        "primary" => colors::KEJAKSAAN_PRIMARY,
        "primary-dark" => colors::KEJAKSAAN_PRIMARY_DARK,
        "primary-light" => colors::KEJAKSAAN_PRIMARY_LIGHT,
        "secondary" => colors::KEJAKSAAN_SECONDARY,
        "success" => colors::SUCCESS,
        "warning" => colors::WARNING,
        "danger" => colors::DANGER,
        "info" => colors::INFO,
        _ => colors::GRAY_500,
    }
}

/// Check if file extension is allowed for type
pub fn is_extension_allowed(extension: &str, file_type: &str) -> bool {
    let extension = extension.to_lowercase();
    match file_type {
        "image" => api::ALLOWED_IMAGE_EXTENSIONS.contains(&extension.as_str()),
        "document" => api::ALLOWED_DOCUMENT_EXTENSIONS.contains(&extension.as_str()),
        "archive" => api::ALLOWED_ARCHIVE_EXTENSIONS.contains(&extension.as_str()),
        _ => false,
    }
}

/// Validate file size for type
pub fn is_file_size_valid(size: u64, file_type: &str) -> bool {
    match file_type {
        "image" => size <= api::MAX_IMAGE_SIZE,
        "document" => size <= api::MAX_DOCUMENT_SIZE,
        _ => size <= api::MAX_FILE_SIZE,
    }
}

// ============================================================================
// FEATURE-GATED CONSTANTS
// ============================================================================

#[cfg(feature = "development")]
pub mod development {
    pub const DEBUG_MODE: bool = true;
    pub const API_BASE_URL: &str = "http://localhost:3000";
    pub const WEBSOCKET_URL: &str = "ws://localhost:3001";
    pub const LOG_LEVEL: &str = "debug";
}

#[cfg(feature = "production")]
pub mod production {
    pub const DEBUG_MODE: bool = false;
    pub const API_BASE_URL: &str = "https://api.kejaksaan.go.id";
    pub const WEBSOCKET_URL: &str = "wss://api.kejaksaan.go.id/ws";
    pub const LOG_LEVEL: &str = "info";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_province_lookup() {
        assert_eq!(get_province_name("31"), Some("DKI Jakarta"));
        assert_eq!(get_province_name("invalid"), None);
    }

    #[test]
    fn test_color_lookup() {
        assert_eq!(get_color_or_default("primary"), colors::KEJAKSAAN_PRIMARY);
        assert_eq!(get_color_or_default("invalid"), colors::GRAY_500);
    }

    #[test]
    fn test_file_validation() {
        assert!(is_extension_allowed("jpg", "image"));
        assert!(is_extension_allowed("PDF", "document"));
        assert!(!is_extension_allowed("exe", "image"));

        assert!(is_file_size_valid(1024, "image"));
        assert!(!is_file_size_valid(api::MAX_IMAGE_SIZE + 1, "image"));
    }
}
