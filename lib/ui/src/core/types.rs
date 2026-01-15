//! Unified type definitions untuk seluruh aplikasi
//!
//! Menggabungkan types.rs dan types_ui.rs dengan fokus pada yang essential.

use serde::{Deserialize, Serialize};

// ============================================================================
// NAVIGATION TYPES
// ============================================================================

/// Menu item untuk navigasi aplikasi
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NavItem {
    pub id: String,
    pub label: String,
    pub path: String,
    pub icon: Option<String>,
    pub badge: Option<String>,
    pub active: bool,
    pub children: Vec<NavItem>,
    pub permissions: Vec<String>,
}

impl NavItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            path: path.into(),
            icon: None,
            badge: None,
            active: false,
            children: Vec::new(),
            permissions: Vec::new(),
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn with_badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    pub fn active(mut self) -> Self {
        self.active = true;
        self
    }

    pub fn with_children(mut self, children: Vec<NavItem>) -> Self {
        self.children = children;
        self
    }
}

/// Breadcrumb item untuk navigasi halaman
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BreadcrumbItem {
    pub label: String,
    pub path: Option<String>,
    pub icon: Option<String>,
}

impl BreadcrumbItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            path: None,
            icon: None,
        }
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

// ============================================================================
// TABLE TYPES
// ============================================================================

/// Column definition untuk DataTable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub key: String,
    pub label: String,
    pub sortable: bool,
    pub width: Option<String>,
    pub align: TableAlign,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum TableAlign {
    #[default]
    Left,
    Center,
    Right,
}

impl TableColumn {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            sortable: false,
            width: None,
            align: TableAlign::Left,
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

    pub fn align(mut self, align: TableAlign) -> Self {
        self.align = align;
        self
    }
}

/// Sort direction untuk table
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    pub fn toggle(&self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }
}

/// Pagination info
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Pagination {
    pub current_page: u32,
    pub page_size: u32,
    pub total_items: u64,
    pub total_pages: u32,
}

impl Pagination {
    pub fn new(current_page: u32, page_size: u32, total_items: u64) -> Self {
        let total_pages = ((total_items as f64) / (page_size as f64)).ceil() as u32;
        Self {
            current_page,
            page_size,
            total_items,
            total_pages,
        }
    }

    pub fn has_prev(&self) -> bool {
        self.current_page > 1
    }

    pub fn has_next(&self) -> bool {
        self.current_page < self.total_pages
    }
}

// ============================================================================
// API RESPONSE TYPES
// ============================================================================

/// Standard API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum ApiResponse<T> {
    #[serde(rename = "success")]
    Success {
        data: T,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
    },
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self::Success {
            data,
            message: None,
        }
    }

    pub fn success_with_message(data: T, message: impl Into<String>) -> Self {
        Self::Success {
            data,
            message: Some(message.into()),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
            code: None,
        }
    }

    pub fn error_with_code(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
            code: Some(code.into()),
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }
}

/// Paginated API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub pagination: Pagination,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, current_page: u32, page_size: u32, total_items: u64) -> Self {
        Self {
            items,
            pagination: Pagination::new(current_page, page_size, total_items),
        }
    }
}

// ============================================================================
// VALIDATION TYPES
// ============================================================================

/// Validation result untuk form inputs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
        }
    }

    pub fn invalid(errors: Vec<ValidationError>) -> Self {
        Self {
            valid: false,
            errors,
        }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.valid = false;
        self.errors.push(error);
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }
}

/// Single validation error
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
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
// UI STATE TYPES
// ============================================================================

/// Button variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
    Success,
    Ghost,
}

impl ButtonVariant {
    pub fn to_class(&self) -> &'static str {
        match self {
            Self::Primary => "btn-primary",
            Self::Secondary => "btn-secondary",
            Self::Danger => "btn-danger",
            Self::Success => "btn-success",
            Self::Ghost => "btn-ghost",
        }
    }
}

/// Button sizes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

impl ButtonSize {
    pub fn to_class(&self) -> &'static str {
        match self {
            Self::Small => "btn-sm",
            Self::Medium => "btn-md",
            Self::Large => "btn-lg",
        }
    }
}

/// Toast notification types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

impl ToastType {
    pub fn to_class(&self) -> &'static str {
        match self {
            Self::Success => "toast-success",
            Self::Error => "toast-error",
            Self::Warning => "toast-warning",
            Self::Info => "toast-info",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Success => "✓",
            Self::Error => "✕",
            Self::Warning => "⚠",
            Self::Info => "ℹ",
        }
    }
}

/// Alert variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertVariant {
    Info,
    Success,
    Warning,
    Error,
}

impl AlertVariant {
    pub fn to_class(&self) -> &'static str {
        match self {
            Self::Info => "alert-info",
            Self::Success => "alert-success",
            Self::Warning => "alert-warning",
            Self::Error => "alert-error",
        }
    }
}
