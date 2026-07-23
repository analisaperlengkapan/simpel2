//! UI Components untuk shared library
//!
//! Organized by category for better maintainability.

pub mod accessibility;
pub mod accessibility_controls;
pub mod advanced;
pub mod app_shell;
pub mod captcha;
pub mod charts;
pub mod custom_branding;
pub mod dashboard;
pub mod data_table_v2;
pub mod display;
pub mod error_boundary;
pub mod feedback;
pub mod floating;
pub mod forms;
pub mod guards;
pub mod icon;
pub mod layout;
pub mod logo;
pub mod monitoring_dashboard;
pub mod navigation;
pub mod notifications;
pub mod optimized_image;
pub mod route_suspense;
pub mod search;
pub mod security_meta;
pub mod theme_editor;

// Re-exports - using specific imports to avoid conflicts
pub use accessibility::Heading;
pub use accessibility_controls::*;
pub use advanced::*;
pub use captcha::*;
pub use custom_branding::*;
pub use dashboard::*;
pub use display::{Badge, Table};
pub use error_boundary::*;
pub use feedback::*;
pub use forms::*;
pub use icon::*;
pub use layout::*;
pub use logo::*;
pub use monitoring_dashboard::*;
pub use navigation::{AppHeader, Breadcrumb};
pub use notifications::*;
// Image optimization components (includes Avatar, OptimizedImage)
pub use optimized_image::*;
pub use search::*;
pub use security_meta::*;
pub use theme_editor::*;
