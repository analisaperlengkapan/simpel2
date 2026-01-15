#![recursion_limit = "1024"]

//! # SIMPelv2 Portal Utama - Gateway to Justice Technology
//!
//! Portal Utama sistem SIMPelv2 yang menyediakan:
//! - **Dashboard Terpadu**: Overview semua layanan kejaksaan
//! - **SSO Integration**: Single Sign-On dengan sistem pemerintahan
//! - **Microfrontend Router**: Gateway ke semua aplikasi SIMPelv2
//! - **Government Compliance**: Sesuai standar keamanan siber nasional
//! - **Responsive Design**: Optimized untuk semua device

#![warn(missing_docs)]
#![warn(clippy::all)]
#![forbid(unsafe_code)]

pub mod app;
pub mod components;
pub mod features;
pub mod pages;
pub mod utils;

// Re-export the main App component
pub use app::App;

/// Prelude for commonly used items in portal
pub mod prelude {
    // Re-export specific items to avoid ambiguity
    pub use crate::features::auth::{
        AuthService, MfaSetupData, MfaStatus, UserSession as PortalUserSession,
    };
    pub use crate::features::microfrontends::{
        AppCategory, AppColor, AppStatus, MicrofrontendApp, MicrofrontendRegistry,
    };
    pub use crate::pages::{
        AppsPage, CallbackPage, DashboardPage, HomePage, LoginPage, MfaBackupCodesPage,
        MfaBackupVerificationPage, MfaSetupPage, MfaVerificationPage, NotFoundPage,
        NotificationsPage, PembinaanPage,
    };
    pub use leptos::prelude::*;
    pub use leptos_router::*;
    // Re-export shared components without glob to avoid conflicts
    pub use shared_microfrontend::components;
    pub use shared_microfrontend::core;
    pub use shared_microfrontend::hooks;
    pub use shared_microfrontend::utils;
}
