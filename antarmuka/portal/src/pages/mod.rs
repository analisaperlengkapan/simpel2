//! Pages module — all page components
//!
//! Removes password_reset (admin-only in government context).

pub mod admin;
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
pub mod not_found;
pub mod notifications;
pub mod passkeys;
pub mod password_change;
pub mod profile;
pub mod sessions;
pub mod settings;

pub use admin::*;
pub use apps::*;
pub use callback::*;
pub use dashboard::*;
pub use home::*;
pub use logged_out::*;
pub use login::*;
pub use mfa_backup_codes::*;
pub use mfa_backup_verification::MfaBackupVerificationPage;
pub use mfa_setup::MfaSetupPage;
pub use mfa_verification::MfaVerificationPage;
pub use not_found::*;
pub use notifications::*;
pub use passkeys::*;
pub use password_change::*;
pub use profile::*;
pub use sessions::*;
pub use settings::*;
