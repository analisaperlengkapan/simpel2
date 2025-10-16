//! # Components Module
//!
//! Re-exports untuk semua komponen di microfrontend perlengkapan

// Core components
pub mod dashboard;
pub mod login;
pub mod sidebar;
pub mod sidebar_section;
pub mod user_menu;

// Re-exports untuk komponen yang digunakan
pub use login::LoginPage;

// Alias for compatibility
pub use dashboard::DashboardLayout as DashboardPage;
