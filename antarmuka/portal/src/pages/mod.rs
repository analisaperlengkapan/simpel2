//! Pages module

pub mod apps;
pub mod callback;
pub mod dashboard;
pub mod home;
pub mod logged_out;
pub mod login;
pub mod mfa_backup_codes;
pub mod mfa_backup_verification;
pub mod mfa_setup;
pub mod mfa_verification;
pub mod monitoring;
pub mod not_found;
pub mod notifications;
pub mod password_reset;
pub mod pembinaan;
pub mod secrets;
pub mod settings;

// Re-export pages for easier access
pub use apps::*;
pub use callback::*;
pub use dashboard::*;
pub use home::*;
pub use logged_out::*;
pub use login::*;
pub use mfa_backup_codes::*;
pub use mfa_backup_verification::*;
pub use mfa_setup::*;
pub use mfa_verification::*;
pub use monitoring::*;
pub use not_found::*;
pub use notifications::*;
pub use password_reset::*;
pub use pembinaan::*;
pub use secrets::*;
pub use settings::*;
