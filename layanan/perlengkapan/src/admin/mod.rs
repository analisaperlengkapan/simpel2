//! # Admin module
//!
//! Audit log viewer and master data hub exposed at `/admin/*`. Master data and
//! templates need `Capability::Administer`, the workflow audit trail needs
//! `Capability::ViewAudit` (both `ADMIN_ROLES`); see `handlers::require_admin`.
//! `break_glass` is the audited emergency override — it replaces the implicit
//! "admin bypasses every workflow check" the API used to have.

pub mod break_glass;
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

pub use break_glass::{break_glass_transition, list_break_glass};
pub use handlers::{
    create_master_record, delete_master_record, list_audit_logs, list_master_records,
    list_master_sources, update_master_record,
};
pub use templates::{get_template, list_templates, preview_template};
