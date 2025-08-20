//! # Shared Constants for SIMPelv2 - Centralized & Type-Safe
//!
//! Comprehensive constant definitions untuk aplikasi Kejaksaan RI dengan focus pada:
//! - **Consistency**: Unified design system values
//! - **Type Safety**: Compile-time validation
//! - **Performance**: Zero-allocation constant access
//! - **Government Standards**: Sesuai standar visual identity Kejaksaan RI

use once_cell::sync::Lazy;
use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ============================================================================
// VISUAL IDENTITY - Kejaksaan RI Official Colors
// ============================================================================

/// Warna resmi Kejaksaan RI sesuai pedoman corporate identity
#[derive(Debug, Clone, Copy)]
pub struct KejaksaanColors;

impl KejaksaanColors {
    // === PRIMARY COLORS - Biru Kejaksaan ===
    /// Biru Kejaksaan utama (primary)
    pub const PRIMARY: &'static str = "#1E40AF"; // blue-800
    /// Biru Kejaksaan terang
    pub const PRIMARY_LIGHT: &'static str = "#3B82F6"; // blue-600
    /// Biru Kejaksaan gelap
    pub const PRIMARY_DARK: &'static str = "#1E3A8A"; // blue-900
    /// Biru Kejaksaan sangat terang (untuk background)
    pub const PRIMARY_50: &'static str = "#EFF6FF"; // blue-50
    /// Biru Kejaksaan terang sekali (untuk hover)
    pub const PRIMARY_100: &'static str = "#DBEAFE"; // blue-100

    // === SECONDARY COLORS - Emas Kejaksaan ===
    /// Emas Kejaksaan (secondary/accent)
    pub const GOLD: &'static str = "#F59E0B"; // amber-500
    /// Emas terang
    pub const GOLD_LIGHT: &'static str = "#FCD34D"; // amber-300
    /// Emas gelap
    pub const GOLD_DARK: &'static str = "#D97706"; // amber-600
    /// Background emas
    pub const GOLD_50: &'static str = "#FFFBEB"; // amber-50

    // === NEUTRAL COLORS - Supporting ===
    /// Putih bersih
    pub const WHITE: &'static str = "#FFFFFF";
    /// Abu-abu sangat terang (backgrounds)
    pub const GRAY_50: &'static str = "#F9FAFB";
    /// Abu-abu terang (borders)
    pub const GRAY_100: &'static str = "#F3F4F6";
    /// Abu-abu sedang terang
    pub const GRAY_200: &'static str = "#E5E7EB";
    /// Abu-abu sedang
    pub const GRAY_300: &'static str = "#D1D5DB";
    /// Abu-abu normal (text secondary)
    pub const GRAY_500: &'static str = "#6B7280";
    /// Abu-abu gelap (text primary)
    pub const GRAY_700: &'static str = "#374151";
    /// Abu-abu sangat gelap
    pub const GRAY_900: &'static str = "#111827";
    /// Hitam
    pub const BLACK: &'static str = "#000000";

    // === STATUS COLORS - Semantic ===
    /// Hijau untuk success/berhasil
    pub const SUCCESS: &'static str = "#10B981"; // emerald-500
    /// Background success
    pub const SUCCESS_50: &'static str = "#ECFDF5"; // emerald-50
    /// Kuning untuk warning/peringatan
    pub const WARNING: &'static str = "#F59E0B"; // amber-500
    /// Background warning
    pub const WARNING_50: &'static str = "#FFFBEB"; // amber-50
    /// Merah untuk error/gagal
    pub const ERROR: &'static str = "#EF4444"; // red-500
    /// Background error
    pub const ERROR_50: &'static str = "#FEF2F2"; // red-50
    /// Biru untuk info/informasi
    pub const INFO: &'static str = "#3B82F6"; // blue-500
    /// Background info
    pub const INFO_50: &'static str = "#EFF6FF"; // blue-50
}

// ============================================================================
// GOVERNMENT UNITS - Struktur Organisasi Kejaksaan RI
// ============================================================================

/// Informasi unit kerja dalam Kejaksaan RI
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UnitInfo {
    /// Nama singkat unit
    pub name: &'static str,
    /// Nama lengkap unit
    pub full_name: &'static str,
    /// Icon FontAwesome
    pub icon: &'static str,
    /// Warna tema unit
    pub color: &'static str,
    /// Kode unit (untuk sistem internal)
    pub code: &'static str,
    /// Deskripsi singkat
    pub description: &'static str,
    /// Level hierarki (1=tertinggi)
    pub hierarchy_level: u8,
}

/// Mapping kode unit ke informasi lengkap
pub static UNIT_CODES: Lazy<HashMap<&'static str, UnitInfo>> = Lazy::new(|| {
    let mut units = HashMap::new();

    // === BIDANG OPERASIONAL ===
    units.insert(
        "PIDUM",
        UnitInfo {
            name: "Pidana Umum",
            full_name: "Tindak Pidana Umum",
            icon: "fa-gavel",
            color: "red",
            code: "PIDUM",
            description: "Penanganan perkara pidana umum dan konvensional",
            hierarchy_level: 2,
        },
    );

    units.insert(
        "PIDSUS",
        UnitInfo {
            name: "Pidana Khusus",
            full_name: "Tindak Pidana Khusus",
            icon: "fa-user-secret",
            color: "purple",
            code: "PIDSUS",
            description: "Penanganan tipikor, terorisme, dan kejahatan khusus",
            hierarchy_level: 2,
        },
    );

    units.insert(
        "PIDMIL",
        UnitInfo {
            name: "Pidana Militer",
            full_name: "Tindak Pidana Militer",
            icon: "fa-star",
            color: "green",
            code: "PIDMIL",
            description: "Penanganan perkara pidana yang melibatkan militer",
            hierarchy_level: 2,
        },
    );

    units.insert(
        "DATUN",
        UnitInfo {
            name: "Perdata dan TUN",
            full_name: "Perdata dan Tata Usaha Negara",
            icon: "fa-balance-scale",
            color: "blue",
            code: "DATUN",
            description: "Penanganan perkara perdata dan sengketa TUN",
            hierarchy_level: 2,
        },
    );

    // === BIDANG PENGAWASAN & INTELIJEN ===
    units.insert(
        "INTEL",
        UnitInfo {
            name: "Intelijen",
            full_name: "Bidang Intelijen",
            icon: "fa-search",
            color: "indigo",
            code: "INTEL",
            description: "Pengumpulan dan analisis informasi strategis",
            hierarchy_level: 2,
        },
    );

    units.insert(
        "PENGAWASAN",
        UnitInfo {
            name: "Pengawasan",
            full_name: "Bidang Pengawasan",
            icon: "fa-eye",
            color: "purple",
            code: "WASKAT",
            description: "Pengawasan melekat dan evaluasi kinerja",
            hierarchy_level: 2,
        },
    );

    // === BIDANG KHUSUS ===
    units.insert(
        "PEMULIHAN_ASET",
        UnitInfo {
            name: "Pemulihan Aset",
            full_name: "Pemulihan Aset Negara",
            icon: "fa-coins",
            color: "green",
            code: "ASET",
            description: "Pemulihan aset hasil tindak pidana korupsi",
            hierarchy_level: 2,
        },
    );

    units.insert(
        "BADIKLAT",
        UnitInfo {
            name: "Pendidikan & Pelatihan",
            full_name: "Badan Pendidikan dan Pelatihan",
            icon: "fa-graduation-cap",
            color: "green",
            code: "DIKLAT",
            description: "Pembinaan dan pengembangan SDM Kejaksaan",
            hierarchy_level: 2,
        },
    );

    // === BIDANG PEMBINAAN ===
    units.insert(
        "PEMBINAAN",
        UnitInfo {
            name: "Pembinaan",
            full_name: "Bidang Pembinaan",
            icon: "fa-users",
            color: "teal",
            code: "BINAAN",
            description: "Pembinaan dan pengembangan organisasi",
            hierarchy_level: 2,
        },
    );

    units
});

/// Helper function untuk mendapatkan informasi unit
pub fn get_unit_info(code: &str) -> Option<&UnitInfo> {
    UNIT_CODES.get(code)
}

/// Get semua unit codes sebagai Vec
pub fn get_all_unit_codes() -> Vec<&'static str> {
    UNIT_CODES.keys().copied().collect()
}

// ============================================================================
// HIERARCHY & PERMISSIONS - Struktur Jabatan & Wewenang
// ============================================================================

/// Level jabatan dalam hierarki Kejaksaan RI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum JabatanLevel {
    /// Jaksa Agung (Level 1 - Tertinggi)
    JaksaAgung = 1,
    /// Jaksa Agung Muda (Level 2)
    JaksaAgungMuda = 2,
    /// Jamwas (Level 3)
    Jamwas = 3,
    /// Jampidsus (Level 4)
    Jampidsus = 4,
    /// Jampidum (Level 5)
    Jampidum = 5,
    /// Kepala Unit Kerja (Level 6)
    KepalaUnit = 6,
    /// Jaksa Tinggi (Level 7)
    JaksaTinggi = 7,
    /// Jaksa (Level 8)
    Jaksa = 8,
    /// Jaksa Muda (Level 9)
    JaksaMuda = 9,
    /// Pegawai Negeri (Level 10)
    PegawaiNegeri = 10,
    /// Tenaga Kontrak (Level 11)
    TenagaKontrak = 11,
}

impl JabatanLevel {
    /// Get level number untuk comparison
    pub fn level(&self) -> u8 {
        *self as u8
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            JabatanLevel::JaksaAgung => "Jaksa Agung",
            JabatanLevel::JaksaAgungMuda => "Jaksa Agung Muda",
            JabatanLevel::Jamwas => "Jamwas",
            JabatanLevel::Jampidsus => "Jampidsus",
            JabatanLevel::Jampidum => "Jampidum",
            JabatanLevel::KepalaUnit => "Kepala Unit",
            JabatanLevel::JaksaTinggi => "Jaksa Tinggi",
            JabatanLevel::Jaksa => "Jaksa",
            JabatanLevel::JaksaMuda => "Jaksa Muda",
            JabatanLevel::PegawaiNegeri => "Pegawai Negeri",
            JabatanLevel::TenagaKontrak => "Tenaga Kontrak",
        }
    }

    /// Check if this level can supervise another level
    pub fn can_supervise(&self, other: &JabatanLevel) -> bool {
        self.level() < other.level()
    }
}

/// Level wewenang dalam sistem aplikasi
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum WewenangLevel {
    /// Super Administrator (Full Access)
    SuperAdmin = 1,
    /// Administrator Unit (Unit-wide Access)
    Admin = 2,
    /// Pimpinan (Leadership Access)
    Pimpinan = 3,
    /// Pelaksana (Operational Access)
    Pelaksana = 4,
    /// Read Only (View Access)
    ReadOnly = 5,
    /// Guest (Limited Access)
    Guest = 6,
}

impl WewenangLevel {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            WewenangLevel::SuperAdmin => "Super Administrator",
            WewenangLevel::Admin => "Administrator",
            WewenangLevel::Pimpinan => "Pimpinan",
            WewenangLevel::Pelaksana => "Pelaksana",
            WewenangLevel::ReadOnly => "Hanya Baca",
            WewenangLevel::Guest => "Tamu",
        }
    }

    /// Check if this level has permission for action
    pub fn can_perform(&self, action: &str) -> bool {
        match self {
            WewenangLevel::SuperAdmin => true,
            WewenangLevel::Admin => {
                matches!(action, "read" | "create" | "update" | "delete" | "manage")
            }
            WewenangLevel::Pimpinan => matches!(action, "read" | "create" | "update" | "approve"),
            WewenangLevel::Pelaksana => matches!(action, "read" | "create" | "update"),
            WewenangLevel::ReadOnly => matches!(action, "read"),
            WewenangLevel::Guest => matches!(action, "read") && action.contains("public"),
        }
    }
}

// ============================================================================
// API CONFIGURATION - Endpoint Management
// ============================================================================

/// API endpoint constants dengan versioning
#[derive(Debug, Clone)]
pub struct ApiEndpoints;

impl ApiEndpoints {
    // === BASE CONFIGURATION ===
    pub const API_VERSION: &'static str = "v1";
    pub const BASE_URL: &'static str = "/api/v1";

    // === AUTHENTICATION & AUTHORIZATION ===
    pub const AUTH_LOGIN: &'static str = "/api/v1/auth/login";
    pub const AUTH_LOGOUT: &'static str = "/api/v1/auth/logout";
    pub const AUTH_REFRESH: &'static str = "/api/v1/auth/refresh";
    pub const AUTH_PROFILE: &'static str = "/api/v1/auth/profile";

    // === USER MANAGEMENT ===
    pub const USERS: &'static str = "/api/v1/users";
    pub const USER_PROFILE: &'static str = "/api/v1/users/profile";
    pub const USER_PERMISSIONS: &'static str = "/api/v1/users/permissions";

    // === AUDIT & MONITORING ===
    pub const AUDIT_LOGS: &'static str = "/api/v1/audit";
    pub const SYSTEM_HEALTH: &'static str = "/api/v1/system/health";
    pub const SYSTEM_METRICS: &'static str = "/api/v1/system/metrics";

    // === DOCUMENTS & FILES ===
    pub const DOCUMENTS: &'static str = "/api/v1/documents";
    pub const DOCUMENT_UPLOAD: &'static str = "/api/v1/documents/upload";
    pub const DOCUMENT_DOWNLOAD: &'static str = "/api/v1/documents/download";

    // === NOTIFICATIONS ===
    pub const NOTIFICATIONS: &'static str = "/api/v1/notifications";
    pub const NOTIFICATIONS_UNREAD: &'static str = "/api/v1/notifications/unread";

    // === REPORTS ===
    pub const REPORTS: &'static str = "/api/v1/reports";
    pub const REPORTS_GENERATE: &'static str = "/api/v1/reports/generate";
    pub const REPORTS_DOWNLOAD: &'static str = "/api/v1/reports/download";
}

// ============================================================================
// UI DESIGN SYSTEM - Responsive & Consistent
// ============================================================================

/// Responsive breakpoints sesuai Tailwind CSS
#[derive(Debug, Clone)]
pub struct Breakpoints;

impl Breakpoints {
    /// Small devices (phones, 640px and up)
    pub const SM: &'static str = "640px";
    /// Medium devices (tablets, 768px and up)
    pub const MD: &'static str = "768px";
    /// Large devices (desktops, 1024px and up)
    pub const LG: &'static str = "1024px";
    /// Extra large devices (large desktops, 1280px and up)
    pub const XL: &'static str = "1280px";
    /// 2X large devices (larger desktops, 1536px and up)
    pub const XXL: &'static str = "1536px";

    /// Get all breakpoints as array
    pub const ALL: &'static [(&'static str, &'static str)] = &[
        ("sm", Self::SM),
        ("md", Self::MD),
        ("lg", Self::LG),
        ("xl", Self::XL),
        ("2xl", Self::XXL),
    ];
}

/// Spacing system berdasarkan 4px grid
#[derive(Debug, Clone)]
pub struct Spacing;

impl Spacing {
    /// 4px
    pub const XS: &'static str = "0.25rem";
    /// 8px
    pub const SM: &'static str = "0.5rem";
    /// 12px
    pub const MD: &'static str = "0.75rem";
    /// 16px (base)
    pub const BASE: &'static str = "1rem";
    /// 20px
    pub const LG: &'static str = "1.25rem";
    /// 24px
    pub const XL: &'static str = "1.5rem";
    /// 32px
    pub const XXL: &'static str = "2rem";
    /// 48px
    pub const XXXL: &'static str = "3rem";
    /// 64px
    pub const HUGE: &'static str = "4rem";
}

/// Typography system dengan Indonesian-friendly fonts
#[derive(Debug, Clone)]
pub struct Typography;

impl Typography {
    // === FONT FAMILIES ===
    /// Font sans-serif untuk UI (Poppins + fallbacks)
    pub const FONT_SANS: &'static str = "'Poppins', 'Inter', 'Segoe UI', system-ui, sans-serif";
    /// Font serif untuk documents
    pub const FONT_SERIF: &'static str = "'Crimson Text', 'Times New Roman', serif";
    /// Font monospace untuk code
    pub const FONT_MONO: &'static str = "'JetBrains Mono', 'Fira Code', 'Consolas', monospace";

    // === FONT SIZES ===
    /// 12px - Very small text
    pub const TEXT_XS: &'static str = "0.75rem";
    /// 14px - Small text
    pub const TEXT_SM: &'static str = "0.875rem";
    /// 16px - Base text (default)
    pub const TEXT_BASE: &'static str = "1rem";
    /// 18px - Large text
    pub const TEXT_LG: &'static str = "1.125rem";
    /// 20px - Extra large text
    pub const TEXT_XL: &'static str = "1.25rem";
    /// 24px - 2X large text
    pub const TEXT_2XL: &'static str = "1.5rem";
    /// 30px - 3X large text
    pub const TEXT_3XL: &'static str = "1.875rem";
    /// 36px - 4X large text
    pub const TEXT_4XL: &'static str = "2.25rem";
    /// 48px - 5X large text
    pub const TEXT_5XL: &'static str = "3rem";

    // === FONT WEIGHTS ===
    /// 300 - Light
    pub const WEIGHT_LIGHT: &'static str = "300";
    /// 400 - Normal/Regular
    pub const WEIGHT_NORMAL: &'static str = "400";
    /// 500 - Medium
    pub const WEIGHT_MEDIUM: &'static str = "500";
    /// 600 - Semi-bold
    pub const WEIGHT_SEMIBOLD: &'static str = "600";
    /// 700 - Bold
    pub const WEIGHT_BOLD: &'static str = "700";
    /// 800 - Extra bold
    pub const WEIGHT_EXTRABOLD: &'static str = "800";

    // === LINE HEIGHTS ===
    /// 1.25 - Tight line height for headings
    pub const LEADING_TIGHT: &'static str = "1.25";
    /// 1.5 - Normal line height for body text
    pub const LEADING_NORMAL: &'static str = "1.5";
    /// 1.75 - Relaxed line height for readability
    pub const LEADING_RELAXED: &'static str = "1.75";
}

/// Animation & transition constants
#[derive(Debug, Clone)]
pub struct Animation;

impl Animation {
    // === DURATIONS ===
    /// 150ms - Fast transitions
    pub const FAST: &'static str = "150ms";
    /// 300ms - Normal transitions
    pub const NORMAL: &'static str = "300ms";
    /// 500ms - Slow transitions
    pub const SLOW: &'static str = "500ms";
    /// 750ms - Very slow transitions
    pub const VERY_SLOW: &'static str = "750ms";

    // === TIMING FUNCTIONS ===
    /// Ease in out - Default smooth animation
    pub const EASE: &'static str = "ease-in-out";
    /// Ease out - Snappy start, slow end
    pub const EASE_OUT: &'static str = "ease-out";
    /// Ease in - Slow start, snappy end
    pub const EASE_IN: &'static str = "ease-in";
    /// Linear - Constant speed
    pub const LINEAR: &'static str = "linear";
}

/// Z-index layer management
#[derive(Debug, Clone)]
pub struct ZIndex;

impl ZIndex {
    /// Base layer
    pub const BASE: i32 = 0;
    /// Dropdown menus
    pub const DROPDOWN: i32 = 10;
    /// Sticky headers
    pub const STICKY: i32 = 20;
    /// Fixed headers/sidebars
    pub const FIXED: i32 = 30;
    /// Modal backdrop
    pub const MODAL_BACKDROP: i32 = 40;
    /// Modal content
    pub const MODAL: i32 = 50;
    /// Toast notifications
    pub const TOAST: i32 = 100;
    /// Tooltips
    pub const TOOLTIP: i32 = 200;
    /// Loading overlays
    pub const LOADING: i32 = 500;
    /// Debug overlays (highest)
    pub const DEBUG: i32 = 9999;
}

// ============================================================================
// APPLICATION CONFIGURATION
// ============================================================================

/// Application-wide configuration constants
#[derive(Debug, Clone)]
pub struct AppConfig;

impl AppConfig {
    /// Application name
    pub const APP_NAME: &'static str = "SIMPel v2";
    /// Application full name
    pub const APP_FULL_NAME: &'static str = "Sistem Informasi Manajemen Perkara v2";
    /// Current version
    pub const VERSION: &'static str = "2.0.0";
    /// Copyright
    pub const COPYRIGHT: &'static str = "© 2024 Kejaksaan Agung Republik Indonesia";

    // === PAGINATION ===
    /// Default items per page
    pub const DEFAULT_PAGE_SIZE: usize = 20;
    /// Maximum items per page
    pub const MAX_PAGE_SIZE: usize = 100;
    /// Available page size options
    pub const PAGE_SIZE_OPTIONS: &'static [usize] = &[10, 20, 50, 100];

    // === FILE UPLOAD ===
    /// Maximum file size (10MB)
    pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
    /// Allowed file extensions
    pub const ALLOWED_EXTENSIONS: &'static [&'static str] = &[
        "pdf", "doc", "docx", "xls", "xlsx", "jpg", "jpeg", "png", "gif",
    ];

    // === SESSION ===
    /// Session timeout (8 hours)
    pub const SESSION_TIMEOUT: u64 = 8 * 60 * 60;
    /// Remember me duration (30 days)
    pub const REMEMBER_ME_DURATION: u64 = 30 * 24 * 60 * 60;

    // === VALIDATION ===
    /// Minimum password length
    pub const MIN_PASSWORD_LENGTH: usize = 8;
    /// Maximum login attempts
    pub const MAX_LOGIN_ATTEMPTS: u32 = 5;
    /// Lockout duration (15 minutes)
    pub const LOCKOUT_DURATION: u64 = 15 * 60;
}

/// Form field length constraints
#[derive(Debug, Clone)]
pub struct FieldLimits;

impl FieldLimits {
    /// Short text fields (names, titles)
    pub const SHORT_TEXT: usize = 100;
    /// Medium text fields (descriptions)
    pub const MEDIUM_TEXT: usize = 255;
    /// Long text fields (content, notes)
    pub const LONG_TEXT: usize = 1000;
    /// Very long text (detailed content)
    pub const VERY_LONG_TEXT: usize = 5000;

    /// NIP length (Indonesia)
    pub const NIP_LENGTH: usize = 18;
    /// Phone number max length
    pub const PHONE_MAX_LENGTH: usize = 15;
    /// Email max length
    pub const EMAIL_MAX_LENGTH: usize = 255;
}

// ============================================================================
// LOCALIZATION - Indonesian Language Support
// ============================================================================

/// Common Indonesian text constants
#[derive(Debug, Clone)]
pub struct Text;

impl Text {
    // === COMMON ACTIONS ===
    pub const SAVE: &'static str = "Simpan";
    pub const CANCEL: &'static str = "Batal";
    pub const DELETE: &'static str = "Hapus";
    pub const EDIT: &'static str = "Edit";
    pub const VIEW: &'static str = "Lihat";
    pub const ADD: &'static str = "Tambah";
    pub const SEARCH: &'static str = "Cari";
    pub const FILTER: &'static str = "Filter";
    pub const EXPORT: &'static str = "Ekspor";
    pub const IMPORT: &'static str = "Impor";
    pub const PRINT: &'static str = "Cetak";
    pub const DOWNLOAD: &'static str = "Unduh";
    pub const UPLOAD: &'static str = "Unggah";
    pub const SUBMIT: &'static str = "Kirim";
    pub const RESET: &'static str = "Reset";
    pub const CLEAR: &'static str = "Bersihkan";
    pub const CLOSE: &'static str = "Tutup";
    pub const CONFIRM: &'static str = "Konfirmasi";
    pub const BACK: &'static str = "Kembali";
    pub const NEXT: &'static str = "Selanjutnya";
    pub const PREVIOUS: &'static str = "Sebelumnya";

    // === STATUS ===
    pub const ACTIVE: &'static str = "Aktif";
    pub const INACTIVE: &'static str = "Nonaktif";
    pub const PENDING: &'static str = "Menunggu";
    pub const APPROVED: &'static str = "Disetujui";
    pub const REJECTED: &'static str = "Ditolak";
    pub const DRAFT: &'static str = "Konsep";
    pub const PUBLISHED: &'static str = "Diterbitkan";
    pub const ARCHIVED: &'static str = "Diarsipkan";

    // === VALIDATION MESSAGES ===
    pub const REQUIRED_FIELD: &'static str = "Field ini wajib diisi";
    pub const INVALID_EMAIL: &'static str = "Format email tidak valid";
    pub const INVALID_PHONE: &'static str = "Format nomor telepon tidak valid";
    pub const PASSWORD_TOO_SHORT: &'static str = "Password minimal 8 karakter";
    pub const PASSWORDS_NOT_MATCH: &'static str = "Password tidak sama";

    // === LOADING & MESSAGES ===
    pub const LOADING: &'static str = "Memuat...";
    pub const NO_DATA: &'static str = "Tidak ada data";
    pub const SUCCESS_SAVE: &'static str = "Data berhasil disimpan";
    pub const SUCCESS_DELETE: &'static str = "Data berhasil dihapus";
    pub const ERROR_SAVE: &'static str = "Gagal menyimpan data";
    pub const ERROR_DELETE: &'static str = "Gagal menghapus data";
    pub const ERROR_LOAD: &'static str = "Gagal memuat data";
    pub const CONFIRM_DELETE: &'static str = "Anda yakin ingin menghapus data ini?";
}

// ============================================================================
// ACCESSIBILITY - WCAG 2.1 AA Compliance
// ============================================================================

/// ARIA labels dan accessibility constants
#[derive(Debug, Clone)]
pub struct Accessibility;

impl Accessibility {
    // === ARIA LABELS (Indonesian) ===
    pub const MENU_BUTTON: &'static str = "Buka menu navigasi";
    pub const CLOSE_MODAL: &'static str = "Tutup dialog";
    pub const LOADING_CONTENT: &'static str = "Sedang memuat konten";
    pub const SEARCH_INPUT: &'static str = "Kolom pencarian";
    pub const USER_MENU: &'static str = "Menu pengguna";
    pub const NOTIFICATIONS: &'static str = "Notifikasi";
    pub const LOGOUT_BUTTON: &'static str = "Keluar dari sistem";
    pub const HOME_LINK: &'static str = "Kembali ke beranda";
    pub const EXTERNAL_LINK: &'static str = "Tautan eksternal, buka di tab baru";

    // === KEYBOARD SHORTCUTS ===
    pub const SHORTCUT_SEARCH: &'static str = "Ctrl+K";
    pub const SHORTCUT_HELP: &'static str = "Ctrl+?";
    pub const SHORTCUT_LOGOUT: &'static str = "Ctrl+Shift+Q";

    // === SCREEN READER ===
    pub const SR_ONLY_CLASS: &'static str = "sr-only";
    pub const SKIP_TO_CONTENT: &'static str = "Langsung ke konten utama";
}

/// Error handling constants
#[derive(Debug, Clone)]
pub struct ErrorMessages;

impl ErrorMessages {
    // === HTTP Errors ===
    pub const NETWORK_ERROR: &'static str = "Terjadi masalah koneksi jaringan";
    pub const SERVER_ERROR: &'static str = "Terjadi kesalahan server";
    pub const UNAUTHORIZED: &'static str = "Anda tidak memiliki akses";
    pub const FORBIDDEN: &'static str = "Akses ditolak";
    pub const NOT_FOUND: &'static str = "Halaman tidak ditemukan";
    pub const TIMEOUT: &'static str = "Permintaan melebihi batas waktu";

    // === Validation Errors ===
    pub const INVALID_FORMAT: &'static str = "Format tidak valid";
    pub const FIELD_TOO_LONG: &'static str = "Teks terlalu panjang";
    pub const FIELD_TOO_SHORT: &'static str = "Teks terlalu pendek";
    pub const INVALID_NUMBER: &'static str = "Harus berupa angka";
    pub const INVALID_DATE: &'static str = "Format tanggal tidak valid";
}
