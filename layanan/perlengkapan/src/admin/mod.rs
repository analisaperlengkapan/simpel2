//! # Admin module
//!
//! Audit log viewer and master data hub exposed at `/admin/*`. Callers must
//! hold a cross-satker role (admin/pusat); see `handlers::require_admin`.

pub mod handlers;
pub mod models;
pub mod repository;
pub mod templates;
// NOTE: no `users` module. `/admin/users` read `v_user_role_summary`,
// `perlengkapan_users` and `perlengkapan_user_roles` — none of which any
// migration creates, so the page returned nothing but errors in every
// environment since it was written (the frontend swallowed them into an empty
// list, which is why it looked merely empty rather than broken).
//
// It is not being repaired here because it should not exist: users and roles
// belong to authenc, which is the IAM source of truth per the SSoT rules in
// `layanan/AGENTS.md`. A second user/role master in perlengkapan is exactly the
// over-reach the satker refactor removed. Portal already administers users and
// roles against authenc for real.

pub use handlers::{
    create_master_record, delete_master_record, list_audit_logs, list_master_records,
    list_master_sources, update_master_record,
};
pub use templates::{get_template, list_templates, preview_template};
