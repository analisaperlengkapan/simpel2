//! Microfrontend registry and management
//!
//! Centralized registry of all available microfrontends in SIMPEL

use serde::{Deserialize, Serialize};

/// Sub-application variant (e.g., v1, v2)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubApp {
    /// Version identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// URL path
    pub url: String,
}

/// Microfrontend application definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MicrofrontendApp {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Short description
    pub description: String,
    /// Icon (emoji or path)
    pub icon: String,
    /// Launch URL
    pub url: String,
    /// Color theme (for card styling)
    pub color: AppColor,
    /// Category
    pub category: AppCategory,
    /// Required role to access
    pub required_role: Option<String>,
    /// Status
    pub status: AppStatus,
    /// Optional submenu for app variants (e.g., v1/v2 choices)
    pub submenu: Option<Vec<SubApp>>,
}

/// Application color theme
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AppColor {
    /// Red color theme
    Red,
    /// Blue color theme
    Blue,
    /// Green color theme
    Green,
    /// Yellow color theme
    Yellow,
    /// Purple color theme
    Purple,
    /// Pink color theme
    Pink,
    /// Indigo color theme
    Indigo,
    /// Orange color theme
    Orange,
}

impl AppColor {
    /// Get Tailwind CSS classes for this color
    pub fn to_classes(&self) -> &'static str {
        match self {
            Self::Red => {
                "from-navy-700 to-navy-800 hover:from-navy-800 hover:to-navy-900 border border-navy-600"
            }
            Self::Blue => "from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700",
            Self::Green => "from-green-500 to-green-600 hover:from-green-600 hover:to-green-700",
            Self::Yellow => {
                "from-yellow-500 to-yellow-600 hover:from-yellow-600 hover:to-yellow-700"
            }
            Self::Purple => {
                "from-purple-500 to-purple-600 hover:from-purple-600 hover:to-purple-700"
            }
            Self::Pink => "from-pink-500 to-pink-600 hover:from-pink-600 hover:to-pink-700",
            Self::Indigo => {
                "from-indigo-500 to-indigo-600 hover:from-indigo-600 hover:to-indigo-700"
            }
            Self::Orange => {
                "from-orange-500 to-orange-600 hover:from-orange-600 hover:to-orange-700"
            }
        }
    }
}

/// Application category
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AppCategory {
    /// Core prosecution systems
    Prosecution,
    /// Training and education
    Training,
    /// Legal and compliance
    Legal,
    /// Asset management
    Asset,
    /// Intelligence and investigation
    Intelligence,
    /// Supervision and monitoring
    Supervision,
}

impl AppCategory {
    /// Get category display name in Indonesian
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Prosecution => "Penyidikan & Penuntutan",
            Self::Training => "Pendidikan & Pelatihan",
            Self::Legal => "Perdata & Tata Usaha Negara",
            Self::Asset => "Pengelolaan Aset",
            Self::Intelligence => "Intelijen",
            Self::Supervision => "Pengawasan",
        }
    }
}

/// Application status
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AppStatus {
    /// Application is active and available
    Active,
    /// Application is under maintenance
    Maintenance,
    /// Application is in beta testing
    Beta,
    /// Application is disabled
    Disabled,
}

impl AppStatus {
    /// Check if app is available
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Active | Self::Beta)
    }

    /// Get status badge text
    pub fn badge_text(&self) -> &'static str {
        match self {
            Self::Active => "Aktif",
            Self::Maintenance => "Maintenance",
            Self::Beta => "Beta",
            Self::Disabled => "Nonaktif",
        }
    }
}

/// Microfrontend registry
pub struct MicrofrontendRegistry;

impl MicrofrontendRegistry {
    /// Get all registered applications
    pub fn get_all_apps() -> Vec<MicrofrontendApp> {
        vec![
            // Asset management — Perlengkapan with v1/v2 submenu
            MicrofrontendApp {
                id: "perlengkapan".to_string(),
                name: "Perlengkapan".to_string(),
                description: "Manajemen Aset dan Logistik".to_string(),
                icon: "📦".to_string(),
                url: "/perlengkapan".to_string(),
                color: AppColor::Orange,
                category: AppCategory::Asset,
                required_role: None,
                status: AppStatus::Active,
                submenu: Some(vec![
                    SubApp {
                        id: "perlengkapan-v2".to_string(),
                        name: "SIMPEL v2 (Modern)".to_string(),
                        description: "Versi terbaru berbasis Rust & WebAssembly".to_string(),
                        url: "/perlengkapan/simpel/v2".to_string(),
                    },
                    SubApp {
                        id: "perlengkapan-v1".to_string(),
                        name: "SIMPEL v1 (Legacy)".to_string(),
                        description: "Versi sebelumnya berbasis Laravel".to_string(),
                        url: "/perlengkapan/simpel/v1".to_string(),
                    },
                ]),
            },
        ]
    }

    /// Get apps by category
    pub fn get_apps_by_category(category: AppCategory) -> Vec<MicrofrontendApp> {
        Self::get_all_apps()
            .into_iter()
            .filter(|app| app.category == category)
            .collect()
    }

    /// Get app by ID
    pub fn get_app_by_id(id: &str) -> Option<MicrofrontendApp> {
        Self::get_all_apps().into_iter().find(|app| app.id == id)
    }

    /// Get all categories with at least one app
    pub fn get_all_categories() -> Vec<AppCategory> {
        let apps = Self::get_all_apps();
        let mut categories = Vec::new();
        for cat in [
            AppCategory::Prosecution,
            AppCategory::Training,
            AppCategory::Legal,
            AppCategory::Asset,
            AppCategory::Intelligence,
            AppCategory::Supervision,
        ] {
            if apps.iter().any(|a| a.category == cat) {
                categories.push(cat);
            }
        }
        categories
    }
}
