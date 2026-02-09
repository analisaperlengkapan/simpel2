//! Local components module for Pembinaan Keuangan

//! # Components Module
//!
//! Re-exports untuk semua komponen di microfrontend keuangan

// Core components
pub mod dashboard;
pub mod login;
pub mod sidebar;
pub mod sidebar_section;
pub mod user_menu;

// Re-exports untuk komponen yang digunakan
pub use dashboard::DashboardLayout;
pub use login::LoginPage;
pub use sidebar::Sidebar;
pub use user_menu::UserMenu;
