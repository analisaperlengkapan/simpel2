//! # Shared Types for SIMPelv2 - Optimized & Type-Safe
//!
//! Comprehensive type system untuk aplikasi Kejaksaan RI dengan focus pada:
//! - **Type Safety**: Zero runtime type errors
//! - **Performance**: Compile-time optimized structures
//! - **Government Standards**: Sesuai standar sistem pemerintahan
//! - **Extensibility**: Easy to extend dan maintain

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// UI COMPONENT TYPES - Modern Rust patterns
// ============================================================================

/// Button styling variants dengan consistent theming
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ButtonVariant {
    /// Primary action (blue)
    Primary,
    /// Secondary action (gray outline)
    Secondary,
    /// Success state (green)
    Success,
    /// Warning state (yellow)
    Warning,
    /// Error/danger state (red)
    Danger,
    /// Subtle/ghost styling
    Ghost,
    /// Outline only
    Outline,
}

impl Default for ButtonVariant {
    fn default() -> Self {
        Self::Primary
    }
}

/// Button size variants untuk consistent scaling
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

impl Default for ButtonSize {
    fn default() -> Self {
        Self::Medium
    }
}

/// Card padding options
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CardPadding {
    None,
    Small,
    Medium,
    Large,
}

impl Default for CardPadding {
    fn default() -> Self {
        Self::Medium
    }
}

/// Loading spinner size variants
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SpinnerSize {
    Small,
    Medium,
    Large,
}

impl Default for SpinnerSize {
    fn default() -> Self {
        Self::Medium
    }
}

/// Table column configuration
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TableColumn {
    pub key: String,
    pub title: String,
    pub sortable: bool,
    pub width: Option<String>,
}

impl TableColumn {
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            sortable: false,
            width: None,
        }
    }

    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    pub fn with_width(mut self, width: impl Into<String>) -> Self {
        self.width = Some(width.into());
        self
    }
}

/// Input field types dengan validation hints
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum InputType {
    Text,
    Email,
    Password,
    Number,
    Tel,
    Url,
    Search,
    Date,
    DateTime,
}

/// Message/alert types for consistent communication
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MessageType {
    Success,
    Warning,
    Error,
    Info,
}

/// Loading states for async operations
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LoadingState {
    Idle,
    Loading,
    Success,
    Error(String),
}

/// Modal/dialog size variants
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ModalSize {
    Small,      // max-w-md
    Medium,     // max-w-lg
    Large,      // max-w-2xl
    ExtraLarge, // max-w-4xl
    FullScreen,
}

// ============================================================================
// NAVIGATION & ROUTING TYPES
// ============================================================================

/// Navigation item dengan support untuk nested routes
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NavItem {
    /// Route path
    pub path: String,
    /// Display label
    pub label: String,
    /// Icon class (FontAwesome atau custom)
    pub icon: String,
    /// Whether currently active
    pub active: bool,
    /// Nested navigation items
    pub children: Option<Vec<NavItem>>,
    /// Required permission untuk akses
    pub permission: Option<String>,
}

impl NavItem {
    /// Create new navigation item
    pub fn new(path: impl Into<String>, label: impl Into<String>, icon: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            label: label.into(),
            icon: icon.into(),
            active: false,
            children: None,
            permission: None,
        }
    }

    /// Add nested children
    pub fn with_children(mut self, children: Vec<NavItem>) -> Self {
        self.children = Some(children);
        self
    }

    /// Add permission requirement
    pub fn with_permission(mut self, permission: impl Into<String>) -> Self {
        self.permission = Some(permission.into());
        self
    }
}

/// Breadcrumb item untuk navigation trails
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BreadcrumbItem {
    pub label: String,
    pub href: Option<String>,
}

// ============================================================================
// USER & AUTHENTICATION TYPES
// ============================================================================

/// User roles dalam sistem Kejaksaan
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum UserRole {
    /// Super Administrator
    SuperAdmin,
    /// Administrator unit
    Admin,
    /// User biasa dengan akses terbatas
    User,
    /// User read-only
    Viewer,
    /// Guest dengan akses minimal
    Guest,
}

/// User profile information
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct User {
    pub id: String,
    pub nip: String,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub unit: String,
    pub jabatan: String,
    pub avatar_url: Option<String>,
    pub permissions: Vec<String>,
    pub active: bool,
}

/// User session information
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UserSession {
    pub token: String,
    pub user: User,
    pub expires_at: i64, // Unix timestamp
    pub csrf_token: String,
}

/// Audit log entry untuk tracking
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AuditLog {
    pub id: String,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub timestamp: i64,
    pub ip_address: String,
    pub user_agent: String,
    pub details: HashMap<String, String>,
}

// ============================================================================
// API & RESPONSE TYPES
// ============================================================================

/// Standard API response wrapper
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
    pub errors: Vec<String>,
    pub timestamp: i64,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: "Success".to_string(),
            errors: Vec::new(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            message: message.into(),
            errors: Vec::new(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

/// Paginated response untuk large datasets
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Service response dari layanan backend
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ServiceResponse<T> {
    pub status: u16,
    pub body: ApiResponse<T>,
}

// ============================================================================
// KEJAKSAAN-SPECIFIC TYPES
// ============================================================================

/// Unit institusi dalam Kejaksaan
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum InstitusiUnit {
    /// Kejaksaan Agung
    KejaksaanAgung,
    /// Kejaksaan Tinggi
    KejaksaanTinggi(String),
    /// Kejaksaan Negeri
    KejaksaanNegeri(String),
    /// Cabang Kejaksaan Negeri
    CabangKejaksaanNegeri(String),
}

/// Level jabatan dalam hierarki
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum JabatanLevel {
    /// Jaksa Agung
    JaksaAgung,
    /// Jaksa Agung Muda
    JaksaAgungMuda,
    /// Kepala unit utama
    KepalaUnit,
    /// Jaksa utama
    JaksaUtama,
    /// Jaksa madya
    JaksaMadya,
    /// Jaksa muda
    JaksaMuda,
    /// Jaksa pertama
    JaksaPertama,
    /// Staff
    Staff,
}

/// Level wewenang akses
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum WewenangLevel {
    /// Akses penuh sistem
    Full,
    /// Akses unit dan bawahan
    Unit,
    /// Akses bagian/seksi
    Bagian,
    /// Akses personal/terbatas
    Personal,
    /// Read-only
    ReadOnly,
}

// ============================================================================
// FORM & VALIDATION TYPES
// ============================================================================

/// Generic form field dengan validation
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FormField<T> {
    pub value: T,
    pub errors: Vec<String>,
    pub touched: bool,
    pub required: bool,
    pub disabled: bool,
}

impl<T: Default> Default for FormField<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
            errors: Vec::new(),
            touched: false,
            required: false,
            disabled: false,
        }
    }
}

impl<T> FormField<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            errors: Vec::new(),
            touched: false,
            required: false,
            disabled: false,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn mark_touched(mut self) -> Self {
        self.touched = true;
        self
    }
}

/// Form validation result
#[derive(Clone, Debug, PartialEq)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: HashMap<String, Vec<String>>,
}

impl ValidationResult {
    pub fn success() -> Self {
        Self {
            valid: true,
            errors: HashMap::new(),
        }
    }

    pub fn with_errors(errors: HashMap<String, Vec<String>>) -> Self {
        Self {
            valid: errors.is_empty(),
            errors,
        }
    }
}
