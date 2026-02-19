//! Automated Backup and Restore for Secreton
//!
//! This crate provides automated backup and restore functionality for Secreton,
//! including:
//! - Scheduled backups with cron expressions
//! - Backup of Raft state and PostgreSQL data
//! - Encrypted backup storage
//! - Pluggable storage backends (S3, local filesystem, etc.)
//! - Backup verification and integrity checks
//! - Point-in-time recovery
//! - Automatic retention policy enforcement
//! - Backup failure alerting

pub mod alerting;
pub mod error;
pub mod manager;
pub mod metadata;
pub mod scheduler;
pub mod storage;
pub mod types;

pub use alerting::{Alert, AlertHandler, AlertManager, AlertSeverity, AlertType};
pub use error::{BackupError, Result};
pub use manager::BackupManager;
pub use metadata::{Backup, BackupMetadata};
pub use storage::{BackupStorage, LocalStorage, S3Storage};
pub use types::{BackupConfig, BackupStatus, RestoreOptions, S3StorageConfig};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::alerting::{Alert, AlertHandler, AlertManager, AlertSeverity, AlertType};
    pub use crate::error::{BackupError, Result};
    pub use crate::manager::BackupManager;
    pub use crate::metadata::{Backup, BackupMetadata};
    pub use crate::storage::BackupStorage;
    pub use crate::types::{BackupConfig, BackupStatus};
}
