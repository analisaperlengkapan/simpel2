//! # SIMPelv2 Comprehensive Type System 🎯
//!
//! **Type-safe, performance-optimized data structures** for Kejaksaan RI applications.
//!
//! ## 🛡️ **Design Principles**
//! - **Zero Runtime Errors**: Compile-time type safety for critical government applications
//! - **Performance**: Efficient memory layout and zero-cost abstractions
//! - **Extensibility**: Easy to extend without breaking existing code
//! - **Serialization**: Full serde support for API communication
//! - **Validation**: Built-in validation for government data standards
//! - **Internationalization**: Support for Indonesian government standards
//!
//! ## 📦 **Type Categories**
//! - **UI Types**: Component props, states, and styling enums
//! - **Data Types**: Core business entities and data structures
//! - **API Types**: Request/response types with validation
//! - **Auth Types**: User, roles, and permission structures
//! - **Government Types**: Indonesia-specific data types (NIP, NIK, etc.)
//! - **Utility Types**: Helper types for common patterns

use chrono::{DateTime, Utc};
use leptos::prelude::AnyView;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

// ============================================================================
// UI COMPONENT TYPES - Modern Rust Patterns
// ============================================================================

/// Button styling variants with consistent theming
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ButtonVariant {
    /// Primary action (Kejaksaan blue)
    Primary,
    /// Secondary action (neutral styling)
    Secondary,
    /// Success state (green)
    Success,
    /// Warning state (amber)
    Warning,
    /// Error/danger state (red)
    Danger,
    /// Subtle/ghost styling (transparent background)
    Ghost,
    /// Outline only (border with transparent background)
    Outline,
}

impl Default for ButtonVariant {
    fn default() -> Self {
        Self::Primary
    }
}

impl Display for ButtonVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Danger => "danger",
            Self::Ghost => "ghost",
            Self::Outline => "outline",
        };
        write!(f, "{}", name)
    }
}

/// Button size variants for consistent scaling
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ButtonSize {
    /// Small button (28px height)
    Small,
    /// Medium button (36px height) - default
    Medium,
    /// Large button (44px height)
    Large,
}

impl Default for ButtonSize {
    fn default() -> Self {
        Self::Medium
    }
}

/// Icon position in buttons and other components
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum IconPosition {
    Left,
    Right,
    Top,
    Bottom,
}

impl Default for IconPosition {
    fn default() -> Self {
        Self::Left
    }
}

/// Input field types with validation rules
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
    Time,
    /// Indonesian NIP (Nomor Induk Pegawai)
    Nip,
    /// Indonesian NIK (Nomor Induk Kependudukan)
    Nik,
    /// Currency input (Indonesian Rupiah)
    Currency,
}

impl Display for InputType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let type_str = match self {
            Self::Text => "text",
            Self::Email => "email",
            Self::Password => "password",
            Self::Number => "number",
            Self::Tel => "tel",
            Self::Url => "url",
            Self::Search => "search",
            Self::Date => "date",
            Self::DateTime => "datetime-local",
            Self::Time => "time",
            Self::Nip | Self::Nik => "text",
            Self::Currency => "text",
        };
        write!(f, "{}", type_str)
    }
}

/// Modal size variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ModalSize {
    /// Small modal (max-width: 384px)
    Small,
    /// Medium modal (max-width: 512px) - Default
    Medium,
    /// Large modal (max-width: 768px)
    Large,
    /// Extra large modal (max-width: 1024px)
    ExtraLarge,
    /// Full screen modal
    FullScreen,
}

impl Default for ModalSize {
    fn default() -> Self {
        ModalSize::Medium
    }
}

/// Toast notification types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ToastType {
    /// Success notification (green)
    Success,
    /// Warning notification (yellow)
    Warning,
    /// Error notification (red)
    Error,
    /// Info notification (blue) - Default
    Info,
}

impl Default for ToastType {
    fn default() -> Self {
        ToastType::Info
    }
}

/// Toast position on screen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ToastPosition {
    /// Top left corner
    TopLeft,
    /// Top right corner - Default
    TopRight,
    /// Top center
    TopCenter,
    /// Bottom left corner
    BottomLeft,
    /// Bottom right corner
    BottomRight,
    /// Bottom center
    BottomCenter,
}

impl Default for ToastPosition {
    fn default() -> Self {
        ToastPosition::TopRight
    }
}

/// Table column configuration
pub struct TableColumn<T> {
    /// Column unique key
    pub key: String,
    /// Column header text
    pub header: String,
    /// Whether column is sortable
    pub sortable: bool,
    /// Column width (CSS value)
    pub width: Option<String>,
    /// Cell renderer function
    pub render: Box<dyn Fn(T) -> AnyView>,
}

impl<T> TableColumn<T> {
    /// Create a new table column
    pub fn new<F>(key: &str, header: &str, render: F) -> Self
    where
        F: Fn(T) -> AnyView + 'static,
    {
        Self {
            key: key.to_string(),
            header: header.to_string(),
            sortable: false,
            width: None,
            render: Box::new(render),
        }
    }

    /// Make column sortable
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    /// Set column width
    pub fn width(mut self, width: &str) -> Self {
        self.width = Some(width.to_string());
        self
    }
}

/// Table sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SortDirection {
    /// Ascending sort (A-Z, 0-9)
    Asc,
    /// Descending sort (Z-A, 9-0)
    Desc,
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Asc
    }
}

/// Pagination information
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PaginationInfo {
    /// Current page number (1-based)
    pub current_page: u32,
    /// Total number of pages
    pub total_pages: u32,
    /// Items per page
    pub page_size: u32,
    /// Total number of items
    pub total_items: u64,
    /// Whether there are more pages
    pub has_next: bool,
    /// Whether there are previous pages
    pub has_previous: bool,
}

impl PaginationInfo {
    /// Create new pagination info
    pub fn new(current_page: u32, page_size: u32, total_items: u64) -> Self {
        let total_pages = ((total_items as f64) / (page_size as f64)).ceil() as u32;

        Self {
            current_page,
            total_pages,
            page_size,
            total_items,
            has_next: current_page < total_pages,
            has_previous: current_page > 1,
        }
    }
}

/// Form field validation state
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FieldValidation {
    /// Field name/identifier
    pub field: String,
    /// Whether field is valid
    pub is_valid: bool,
    /// Validation error message (if invalid)
    pub error_message: Option<String>,
    /// Field value that was validated
    pub value: String,
}

impl FieldValidation {
    /// Create valid field validation
    pub fn valid(field: &str, value: &str) -> Self {
        Self {
            field: field.to_string(),
            is_valid: true,
            error_message: None,
            value: value.to_string(),
        }
    }

    /// Create invalid field validation
    pub fn invalid(field: &str, value: &str, error: &str) -> Self {
        Self {
            field: field.to_string(),
            is_valid: false,
            error_message: Some(error.to_string()),
            value: value.to_string(),
        }
    }
}

/// Form submission state
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FormState {
    /// Form is idle, ready for input
    Idle,
    /// Form is being validated
    Validating,
    /// Form is being submitted
    Submitting,
    /// Form submission succeeded
    Success,
    /// Form submission failed
    Error(String),
}

impl Default for FormState {
    fn default() -> Self {
        FormState::Idle
    }
}

/// Loading state for UI components
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LoadingState {
    /// Not loading
    Idle,
    /// Loading data
    Loading,
    /// Loading completed successfully
    Success,
    /// Loading failed
    Error,
}

impl Default for LoadingState {
    fn default() -> Self {
        LoadingState::Idle
    }
}

/// Alert/Message types for notifications
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AlertType {
    /// Success message (green)
    Success,
    /// Information message (blue)
    Info,
    /// Warning message (yellow)
    Warning,
    /// Error message (red)
    Error,
}

impl Default for AlertType {
    fn default() -> Self {
        AlertType::Info
    }
}

/// Card padding variants
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CardPadding {
    /// No padding
    None,
    /// Small padding (12px)
    Small,
    /// Medium padding (16px) - default
    Medium,
    /// Large padding (24px)
    Large,
    /// Extra large padding (32px)
    ExtraLarge,
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
    /// Small spinner (16px)
    Small,
    /// Medium spinner (24px) - default
    Medium,
    /// Large spinner (32px)
    Large,
    /// Extra large spinner (48px)
    ExtraLarge,
}

impl Default for SpinnerSize {
    fn default() -> Self {
        Self::Medium
    }
}

/// Message/notification types for user feedback
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MessageType {
    /// Success message (green)
    Success,
    /// Error message (red)
    Error,
    /// Warning message (amber)
    Warning,
    /// Info message (blue)
    Info,
}

impl Display for MessageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Success => "success",
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
        };
        write!(f, "{}", name)
    }
}

// ============================================================================
// NAVIGATION & LAYOUT TYPES
// ============================================================================

/// Navigation item for menus, breadcrumbs, etc.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NavItem {
    /// Unique identifier
    pub id: String,
    /// Display label
    pub label: String,
    /// URL path
    pub path: String,
    /// FontAwesome icon class
    pub icon: Option<String>,
    /// Whether this item is currently active
    pub active: bool,
    /// Child navigation items for nested menus
    pub children: Option<SmallVec<[Box<NavItem>; 4]>>,
    /// Additional metadata
    pub metadata: Option<HashMap<String, String>>,
    /// Required permissions to access this item
    pub required_permissions: Option<SmallVec<[String; 2]>>,
}

impl NavItem {
    /// Create a new navigation item
    pub fn new(id: impl Into<String>, label: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            path: path.into(),
            icon: None,
            active: false,
            children: None,
            metadata: None,
            required_permissions: None,
        }
    }

    /// Set the icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Mark as active
    pub fn active(mut self) -> Self {
        self.active = true;
        self
    }

    /// Add child items
    pub fn with_children(mut self, children: Vec<NavItem>) -> Self {
        self.children = Some(SmallVec::from_vec(
            children.into_iter().map(Box::new).collect(),
        ));
        self
    }

    /// Add required permissions
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.required_permissions = Some(SmallVec::from_vec(permissions));
        self
    }
}

/// Breadcrumb item
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BreadcrumbItem {
    /// Display label
    pub label: String,
    /// URL (None for current page)
    pub href: Option<String>,
    /// Optional icon
    pub icon: Option<String>,
}

impl BreadcrumbItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: None,
            icon: None,
        }
    }

    pub fn with_href(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

// ============================================================================
// TABLE & DATA DISPLAY TYPES
// ============================================================================

/// Table column alignment
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TableColumnAlign {
    Left,
    Center,
    Right,
}

/// Table column data types for formatting
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TableColumnType {
    Text,
    Number,
    Currency,
    Date,
    DateTime,
    Boolean,
    Email,
    Phone,
    Url,
    /// Indonesian NIP
    Nip,
    /// Indonesian NIK
    Nik,
}

/// Table sorting configuration
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TableSort {
    /// Column key to sort by
    pub column: String,
    /// Sort direction
    pub direction: SortDirection,
}

// ============================================================================
// USER & AUTHENTICATION TYPES
// ============================================================================

/// User information
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct User {
    /// User ID
    pub id: String,
    /// Full name
    pub name: String,
    /// Email address
    pub email: String,
    /// NIP (Nomor Induk Pegawai)
    pub nip: Option<String>,
    /// User role
    pub role: UserRole,
    /// Unit kerja
    pub unit: Option<String>,
    /// Profile avatar URL
    pub avatar_url: Option<String>,
    /// Last login timestamp
    pub last_login: Option<DateTime<Utc>>,
    /// User permissions
    pub permissions: SmallVec<[String; 8]>,
}

/// User roles in Kejaksaan RI system
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum UserRole {
    /// Super administrator
    SuperAdmin,
    /// Unit administrator
    Admin,
    /// Regular user/operator
    User,
    /// Read-only user
    Viewer,
    /// External stakeholder
    External,
}

impl Display for UserRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::SuperAdmin => "Super Admin",
            Self::Admin => "Administrator",
            Self::User => "User",
            Self::Viewer => "Viewer",
            Self::External => "External",
        };
        write!(f, "{}", name)
    }
}

// ============================================================================
// API & DATA TRANSFER TYPES
// ============================================================================

/// Standard API response wrapper
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ApiResponse<T> {
    /// Whether the request was successful
    pub success: bool,
    /// Response data (if successful)
    pub data: Option<T>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Additional metadata
    pub meta: Option<ApiMetadata>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
            meta: None,
        }
    }

    pub fn with_meta(mut self, meta: ApiMetadata) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// API response metadata
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ApiMetadata {
    /// Total number of items (for pagination)
    pub total: Option<u32>,
    /// Current page number
    pub page: Option<u32>,
    /// Items per page
    pub per_page: Option<u32>,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
    /// API version
    pub version: String,
}

/// Paginated list response
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PaginatedResponse<T> {
    /// List items
    pub items: Vec<T>,
    /// Total number of items
    pub total: u32,
    /// Current page (1-indexed)
    pub page: u32,
    /// Items per page
    pub per_page: u32,
    /// Total number of pages
    pub total_pages: u32,
    /// Whether there are more pages
    pub has_more: bool,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: u32, page: u32, per_page: u32) -> Self {
        let total_pages = (total + per_page - 1) / per_page;
        let has_more = page < total_pages;

        Self {
            items,
            total,
            page,
            per_page,
            total_pages,
            has_more,
        }
    }
}

/// API error types
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ApiError {
    /// Network/connection error
    Network(String),
    /// HTTP error with status code
    Http { status: u16, message: String },
    /// Validation error
    Validation { field: String, message: String },
    /// Authentication required
    Unauthorized,
    /// Permission denied
    Forbidden,
    /// Resource not found
    NotFound,
    /// Server error
    Server(String),
    /// Client error (4xx)
    Client(String),
    /// Unknown error
    Unknown(String),
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Http { status, message } => write!(f, "HTTP {} error: {}", status, message),
            Self::Validation { field, message } => {
                write!(f, "Validation error on {}: {}", field, message)
            }
            Self::Unauthorized => write!(f, "Authentication required"),
            Self::Forbidden => write!(f, "Permission denied"),
            Self::NotFound => write!(f, "Resource not found"),
            Self::Server(msg) => write!(f, "Server error: {}", msg),
            Self::Client(msg) => write!(f, "Client error: {}", msg),
            Self::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

// ============================================================================
// GOVERNMENT-SPECIFIC TYPES
// ============================================================================

/// Indonesian government unit codes
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum UnitCode {
    /// Pidana Umum
    Pidum,
    /// Pidana Khusus
    Pidsus,
    /// Pidana Militer
    Pidmil,
    /// Datun (Data dan Informasi)
    Datun,
    /// Intelijen
    Intel,
    /// Pengawasan
    Pengawasan,
    /// Pemulihan Aset
    PemulihanAset,
    /// Badiklat (Pendidikan dan Pelatihan)
    Badiklat,
    /// Pembinaan
    Pembinaan,
}

impl Display for UnitCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Pidum => "PIDUM",
            Self::Pidsus => "PIDSUS",
            Self::Pidmil => "PIDMIL",
            Self::Datun => "DATUN",
            Self::Intel => "INTEL",
            Self::Pengawasan => "PENGAWASAN",
            Self::PemulihanAset => "PEMULIHAN_ASET",
            Self::Badiklat => "BADIKLAT",
            Self::Pembinaan => "PEMBINAAN",
        };
        write!(f, "{}", name)
    }
}

/// Validation result for Indonesian government data
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ValidationResult {
    /// Whether the data is valid
    pub is_valid: bool,
    /// List of validation errors
    pub errors: SmallVec<[ValidationError; 4]>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: SmallVec::new(),
        }
    }

    pub fn invalid(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            errors: SmallVec::from_vec(errors),
        }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }
}

/// Validation error details
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ValidationError {
    /// Field name that failed validation
    pub field: String,
    /// Error message
    pub message: String,
    /// Error code for internationalization
    pub code: Option<String>,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code: None,
        }
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

// ============================================================================
// THEME & STYLING TYPES
// ============================================================================

/// Theme configuration for different units
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ThemeConfig {
    /// Primary color
    pub primary: String,
    /// Secondary color
    pub secondary: String,
    /// Accent color
    pub accent: String,
    /// Success color
    pub success: String,
    /// Warning color
    pub warning: String,
    /// Error color
    pub error: String,
    /// Background color
    pub background: String,
    /// Text color
    pub text: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            primary: "#1E40AF".to_string(),
            secondary: "#64748B".to_string(),
            accent: "#3B82F6".to_string(),
            success: "#10B981".to_string(),
            warning: "#F59E0B".to_string(),
            error: "#EF4444".to_string(),
            background: "#FFFFFF".to_string(),
            text: "#1F2937".to_string(),
        }
    }
}

/// Responsive breakpoint definitions
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Breakpoint {
    /// Mobile (< 640px)
    Mobile,
    /// Tablet (640px - 1024px)
    Tablet,
    /// Desktop (1024px - 1280px)
    Desktop,
    /// Large desktop (> 1280px)
    Large,
}
