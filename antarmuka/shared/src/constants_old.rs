//! # Konstanta Sistem Kejaksaan
//!
//! File ini berisi semua konstanta yang digunakan dalam sistem
//! Kejaksaan Agung Republik Indonesia.

use std::collections::HashMap;

/// Warna resmi Kejaksaan sesuai panduan visual identity
pub struct KejaksaanColors;

impl KejaksaanColors {
    /// Biru Kejaksaan (Primary)
    pub const PRIMARY_BLUE: &'static str = "#1E40AF"; // blue-800
    pub const PRIMARY_BLUE_LIGHT: &'static str = "#3B82F6"; // blue-600
    pub const PRIMARY_BLUE_DARK: &'static str = "#1E3A8A"; // blue-900
    
    /// Emas Kejaksaan (Secondary)
    pub const GOLD: &'static str = "#F59E0B"; // amber-500
    pub const GOLD_LIGHT: &'static str = "#FCD34D"; // amber-300
    pub const GOLD_DARK: &'static str = "#D97706"; // amber-600
    
    /// Netral (Supporting)
    pub const GRAY_50: &'static str = "#F9FAFB";
    pub const GRAY_100: &'static str = "#F3F4F6";
    pub const GRAY_500: &'static str = "#6B7280";
    pub const GRAY_900: &'static str = "#111827";
    
    /// Status Colors
    pub const SUCCESS: &'static str = "#10B981"; // emerald-500
    pub const WARNING: &'static str = "#F59E0B"; // amber-500
    pub const ERROR: &'static str = "#EF4444"; // red-500
    pub const INFO: &'static str = "#3B82F6"; // blue-500
}

/// Kode unit-unit dalam Kejaksaan RI
pub fn get_unit_codes() -> HashMap<&'static str, UnitInfo> {
    let mut units = HashMap::new();
    
    units.insert("PIDUM", UnitInfo {
        name: "Pidana Umum",
        full_name: "Tindak Pidana Umum",
        icon: "fa-gavel",
        color: "red",
        description: "Unit penanganan perkara pidana umum",
    });
    
    units.insert("PIDSUS", UnitInfo {
        name: "Pidana Khusus", 
        full_name: "Tindak Pidana Khusus",
        icon: "fa-user-secret",
        color: "purple",
        description: "Unit penanganan perkara pidana khusus",
    });
    
    units.insert("PIDMIL", UnitInfo {
        name: "Pidana Militer",
        full_name: "Tindak Pidana Militer", 
        icon: "fa-star",
        color: "yellow",
        description: "Unit penanganan perkara pidana militer",
    });
    
    units.insert("DATUN", UnitInfo {
        name: "Perdata dan TUN",
        full_name: "Perdata dan Tata Usaha Negara",
        icon: "fa-balance-scale",
        color: "blue",
        description: "Unit penanganan perkara perdata dan TUN",
    });
    
    units.insert("INTEL", UnitInfo {
        name: "Intelijen",
        full_name: "Bidang Intelijen",
        icon: "fa-search",
        color: "indigo", 
        description: "Unit intelijen dan keamanan",
    });
    
    units.insert("PENGAWASAN", UnitInfo {
        name: "Pengawasan",
        full_name: "Bidang Pengawasan",
        icon: "fa-eye",
        color: "purple",
        description: "Unit pengawasan internal",
    });
    
    units.insert("PEMULIHAN_ASET", UnitInfo {
        name: "Pemulihan Aset",
        full_name: "Pemulihan Aset Negara",
        icon: "fa-coins",
        color: "green",
        description: "Unit pemulihan aset hasil tindak pidana",
    });
    
    units.insert("BADIKLAT", UnitInfo {
        name: "Pendidikan & Pelatihan",
        full_name: "Badan Pendidikan dan Pelatihan",
        icon: "fa-graduation-cap",
        color: "green",
        description: "Unit pendidikan dan pelatihan",
    });
    
    units
}

#[derive(Debug, Clone)]
pub struct UnitInfo {
    pub name: &'static str,
    pub full_name: &'static str, 
    pub icon: &'static str,
    pub color: &'static str,
    pub description: &'static str,
}

/// Level jabatan dalam struktur Kejaksaan
pub enum JabatanLevel {
    JaksaAgung,
    JaksaAgungMuda,
    Jamwas,
    Jampidsus,
    Jampidum,
    JaksaTinggi,
    Jaksa,
    JaksaMuda,
    PegawaiNegeri,
}

/// Level wewenang dalam sistem
pub enum WewenangLevel {
    SuperAdmin,
    Admin,
    Pimpinan,
    Pelaksana,
    ReadOnly,
}

/// API Endpoints standar
pub struct ApiEndpoints;

impl ApiEndpoints {
    pub const BASE_URL: &'static str = "/api/v1";
    pub const AUTH: &'static str = "/auth";
    pub const USERS: &'static str = "/users";
    pub const AUDIT: &'static str = "/audit";
    pub const REPORTS: &'static str = "/reports";
    pub const DOCUMENTS: &'static str = "/documents";
    pub const NOTIFICATIONS: &'static str = "/notifications";
}

/// Breakpoints responsive design
pub struct Breakpoints;

impl Breakpoints {
    pub const SM: &'static str = "640px";   // Small devices
    pub const MD: &'static str = "768px";   // Medium devices
    pub const LG: &'static str = "1024px";  // Large devices
    pub const XL: &'static str = "1280px";  // Extra large devices
    pub const XXL: &'static str = "1536px"; // 2X large devices
}

/// Spacing system sesuai design system
pub struct Spacing;

impl Spacing {
    pub const XS: &'static str = "0.25rem";  // 4px
    pub const SM: &'static str = "0.5rem";   // 8px
    pub const BASE: &'static str = "1rem";   // 16px
    pub const LG: &'static str = "1.5rem";   // 24px
    pub const XL: &'static str = "2rem";     // 32px
    pub const XXL: &'static str = "3rem";    // 48px
}

/// Typography system
pub struct Typography;

impl Typography {
    // Font Families
    pub const FONT_SANS: &'static str = "'Poppins', 'Inter', system-ui, sans-serif";
    pub const FONT_MONO: &'static str = "'JetBrains Mono', 'Fira Code', monospace";
    
    // Font Sizes
    pub const TEXT_XS: &'static str = "0.75rem";    // 12px
    pub const TEXT_SM: &'static str = "0.875rem";   // 14px  
    pub const TEXT_BASE: &'static str = "1rem";     // 16px
    pub const TEXT_LG: &'static str = "1.125rem";   // 18px
    pub const TEXT_XL: &'static str = "1.25rem";    // 20px
    pub const TEXT_2XL: &'static str = "1.5rem";    // 24px
    pub const TEXT_3XL: &'static str = "1.875rem";  // 30px
    
    // Font Weights
    pub const WEIGHT_NORMAL: &'static str = "400";
    pub const WEIGHT_MEDIUM: &'static str = "500";
    pub const WEIGHT_SEMIBOLD: &'static str = "600";
    pub const WEIGHT_BOLD: &'static str = "700";
}

/// Durasi animasi standar
pub struct AnimationDuration;

impl AnimationDuration {
    pub const FAST: &'static str = "150ms";
    pub const NORMAL: &'static str = "300ms";
    pub const SLOW: &'static str = "500ms";
}

/// Z-index layers
pub struct ZIndex;

impl ZIndex {
    pub const DROPDOWN: i32 = 10;
    pub const MODAL: i32 = 50;
    pub const TOAST: i32 = 100;
    pub const TOOLTIP: i32 = 200;
}
