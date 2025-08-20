//! UI-specific types untuk shared components

use serde::{Deserialize, Serialize};

/// Navigation item untuk header
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NavItem {
    pub label: String,
    pub path: String,
    pub icon: String,
    pub active: Option<bool>,
    pub badge: Option<String>, // For notification badges
}

impl NavItem {
    pub fn new(label: impl Into<String>, path: impl Into<String>, icon: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            path: path.into(),
            icon: icon.into(),
            active: None,
            badge: None,
        }
    }

    pub fn with_badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    pub fn with_active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }
}

/// Trend data untuk stat cards
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trend {
    pub percentage: f64,
    pub direction: TrendDirection,
    pub period: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrendDirection {
    Up,
    Down,
    Stable,
}

/// Table header definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableHeader {
    pub key: String,
    pub label: String,
    pub sortable: bool,
    pub width: Option<String>,
    pub align: Option<String>, // left, center, right
}

impl TableHeader {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            sortable: false,
            width: None,
            align: None,
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

    pub fn align_center(mut self) -> Self {
        self.align = Some("center".to_string());
        self
    }

    pub fn align_right(mut self) -> Self {
        self.align = Some("right".to_string());
        self
    }
}

/// Table row data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableRow {
    pub id: String,
    pub cells: Vec<String>,
    pub actions: Option<Vec<TableAction>>,
}

/// Table action button
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableAction {
    pub label: String,
    pub icon: String,
    pub action: String,
    pub variant: ActionVariant,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionVariant {
    Primary,
    Secondary,
    Success,
    Warning,
    Danger,
}

/// Sort direction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Toast notification types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

/// Breadcrumb item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// Form field types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormField {
    pub name: String,
    pub label: String,
    pub field_type: FieldType,
    pub required: bool,
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
    pub validation: Option<ValidationRule>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldType {
    Text,
    Email,
    Password,
    Number,
    Date,
    DateTime,
    Select(Vec<SelectOption>),
    Textarea,
    Checkbox,
    Radio(Vec<SelectOption>),
    File,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub disabled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationRule {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub custom_validator: Option<String>,
}

/// Menu item untuk sidebar/navigation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub path: Option<String>,
    pub icon: String,
    pub children: Option<Vec<MenuItem>>,
    pub badge: Option<MenuBadge>,
    pub active: bool,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MenuBadge {
    pub text: String,
    pub variant: String, // primary, secondary, success, warning, danger
}

/// Dashboard widget types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub id: String,
    pub title: String,
    pub widget_type: WidgetType,
    pub data: serde_json::Value,
    pub config: WidgetConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WidgetType {
    StatCard,
    Chart,
    Table,
    List,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WidgetConfig {
    pub width: String,
    pub height: String,
    pub refresh_interval: Option<u32>,
    pub auto_refresh: bool,
}

/// Notification types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: String,
    pub read: bool,
    pub action_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
    System,
}

/// Common constants
pub struct Constants {
    pub date_format: &'static str,
    pub datetime_format: &'static str,
    pub time_format: &'static str,
    pub currency_format: &'static str,
    pub decimal_places: u8,
}

impl Default for Constants {
    fn default() -> Self {
        Self {
            date_format: "%d/%m/%Y",
            datetime_format: "%d/%m/%Y %H:%M",
            time_format: "%H:%M:%S",
            currency_format: "Rp. {:,.0}",
            decimal_places: 2,
        }
    }
}
