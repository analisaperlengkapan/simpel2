//! # Components Module
//!
//! Re-exports untuk semua komponen di microfrontend perlengkapan

// Core components
pub mod dashboard;
pub mod login;

// Re-exports untuk komponen yang digunakan
pub use dashboard::DashboardLayout;
pub use login::LoginPage;
