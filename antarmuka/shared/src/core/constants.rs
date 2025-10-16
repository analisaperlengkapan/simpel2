//! Government-specific constants untuk aplikasi Kejaksaan RI
//!
//! Constants ini digunakan across semua microfrontends.

// ============================================================================
// BRANDING
// ============================================================================

pub const APP_NAME: &str = "SIMPelv2";
pub const APP_FULL_NAME: &str = "Sistem Informasi Manajemen Perlengkapan v2";
pub const ORG_NAME: &str = "Kejaksaan Agung Republik Indonesia";
pub const ORG_SHORT: &str = "Kejaksaan RI";

// ============================================================================
// COLORS (Design System)
// ============================================================================

/// Primary brand color (Kejaksaan RI green)
pub const COLOR_PRIMARY: &str = "#047857";
pub const COLOR_PRIMARY_HOVER: &str = "#065f46";
pub const COLOR_PRIMARY_LIGHT: &str = "#10b981";

/// Secondary colors
pub const COLOR_SECONDARY: &str = "#6b7280";
pub const COLOR_SECONDARY_HOVER: &str = "#4b5563";

/// Status colors
pub const COLOR_SUCCESS: &str = "#10b981";
pub const COLOR_ERROR: &str = "#ef4444";
pub const COLOR_WARNING: &str = "#f59e0b";
pub const COLOR_INFO: &str = "#3b82f6";

/// Neutral colors
pub const COLOR_TEXT: &str = "#1f2937";
pub const COLOR_TEXT_MUTED: &str = "#6b7280";
pub const COLOR_BORDER: &str = "#e5e7eb";
pub const COLOR_BACKGROUND: &str = "#ffffff";
pub const COLOR_BACKGROUND_ALT: &str = "#f9fafb";

// ============================================================================
// VALIDATION CONSTANTS
// ============================================================================

/// Indonesian NIK (Nomor Induk Kependudukan) - 16 digits
pub const NIK_LENGTH: usize = 16;

/// NIP (Nomor Induk Pegawai) - 18 digits
pub const NIP_LENGTH: usize = 18;

/// Phone number min/max length
pub const PHONE_MIN_LENGTH: usize = 10;
pub const PHONE_MAX_LENGTH: usize = 15;

/// Email max length
pub const EMAIL_MAX_LENGTH: usize = 255;

/// Name min/max length
pub const NAME_MIN_LENGTH: usize = 3;
pub const NAME_MAX_LENGTH: usize = 100;

// ============================================================================
// PAGINATION DEFAULTS
// ============================================================================

pub const DEFAULT_PAGE_SIZE: u32 = 20;
pub const DEFAULT_PAGE: u32 = 1;
pub const MAX_PAGE_SIZE: u32 = 100;

// ============================================================================
// UI CONSTANTS
// ============================================================================

/// Toast notification duration (ms)
pub const TOAST_DURATION: u32 = 3000;
pub const TOAST_DURATION_LONG: u32 = 5000;

/// Debounce delay for search inputs (ms)
pub const DEBOUNCE_DELAY: u32 = 300;

/// Animation durations (ms)
pub const ANIMATION_FAST: u32 = 150;
pub const ANIMATION_NORMAL: u32 = 300;
pub const ANIMATION_SLOW: u32 = 500;

// ============================================================================
// STORAGE KEYS (LocalStorage)
// ============================================================================

pub const STORAGE_KEY_THEME: &str = "simpelv2_theme";
pub const STORAGE_KEY_USER: &str = "simpelv2_user";
pub const STORAGE_KEY_TOKEN: &str = "simpelv2_token";
pub const STORAGE_KEY_SIDEBAR: &str = "simpelv2_sidebar_collapsed";

// ============================================================================
// API CONFIGURATION
// ============================================================================

/// Default API timeout (seconds)
pub const API_TIMEOUT: u64 = 30;

/// Request headers
pub const HEADER_CONTENT_TYPE: &str = "Content-Type";
pub const HEADER_AUTHORIZATION: &str = "Authorization";
pub const HEADER_ACCEPT: &str = "Accept";

/// Content types
pub const CONTENT_TYPE_JSON: &str = "application/json";
pub const CONTENT_TYPE_FORM: &str = "application/x-www-form-urlencoded";
pub const CONTENT_TYPE_MULTIPART: &str = "multipart/form-data";

// ============================================================================
// FILE UPLOAD CONSTRAINTS
// ============================================================================

/// Max file size (10 MB)
pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Allowed image formats
pub const ALLOWED_IMAGE_FORMATS: &[&str] = &["image/jpeg", "image/png", "image/webp"];

/// Allowed document formats
pub const ALLOWED_DOC_FORMATS: &[&str] = &[
    "application/pdf",
    "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.ms-excel",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
];

// ============================================================================
// GOVERNMENT SPECIFIC
// ============================================================================

/// Jabatan levels di Kejaksaan
pub const JABATAN_LEVELS: &[&str] = &[
    "Jaksa Agung",
    "Wakil Jaksa Agung",
    "Jaksa Agung Muda",
    "Kepala Kejaksaan Tinggi",
    "Kepala Kejaksaan Negeri",
    "Jaksa",
    "Staff",
];

/// Unit kerja utama
pub const UNIT_KERJA: &[&str] = &[
    "Jaksa Agung Muda Pidana Umum",
    "Jaksa Agung Muda Pidana Khusus",
    "Jaksa Agung Muda Perdata dan Tata Usaha Negara",
    "Jaksa Agung Muda Pembinaan",
    "Jaksa Agung Muda Pengawasan",
    "Jaksa Agung Muda Intelijen",
];

// ============================================================================
// REGEX PATTERNS (Validation)
// ============================================================================

/// Indonesian phone number pattern
pub const REGEX_PHONE: &str = r"^(\+62|62|0)[0-9]{9,13}$";

/// Email pattern (RFC 5322 simplified)
pub const REGEX_EMAIL: &str = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";

/// NIK pattern (16 digits)
pub const REGEX_NIK: &str = r"^[0-9]{16}$";

/// NIP pattern (18 digits)
pub const REGEX_NIP: &str = r"^[0-9]{18}$";

/// Indonesian postal code (5 digits)
pub const REGEX_POSTAL_CODE: &str = r"^[0-9]{5}$";

// ============================================================================
// DATE/TIME FORMATS
// ============================================================================

/// Standard date format (Indonesian)
pub const DATE_FORMAT: &str = "%d/%m/%Y";

/// DateTime format
pub const DATETIME_FORMAT: &str = "%d/%m/%Y %H:%M:%S";

/// Time format
pub const TIME_FORMAT: &str = "%H:%M:%S";

/// Indonesian month names
pub const MONTH_NAMES_ID: &[&str] = &[
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

/// Indonesian day names
pub const DAY_NAMES_ID: &[&str] = &[
    "Minggu", "Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu",
];
