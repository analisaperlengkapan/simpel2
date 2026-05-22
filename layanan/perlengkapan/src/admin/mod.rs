//! # Admin module
//!
//! Audit log viewer and master data hub exposed at `/admin/*`. Callers must
//! hold a cross-satker role (admin/pusat); see `handlers::require_admin`.

pub mod handlers;
pub mod models;
pub mod repository;
pub mod templates;

pub use handlers::{
    create_master_record, delete_master_record, list_audit_logs, list_master_records,
    list_master_sources, update_master_record,
};
pub use templates::{get_template, list_templates, preview_template};
