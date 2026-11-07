//! # Admin module
//!
//! Master data hub and document templates exposed at `/admin/*`, both needing
//! `Capability::Administer` (`ADMIN_ROLES`); see `handlers::require_admin`.
//!
//! There is deliberately no `/admin/audit` any more: it read
//! `workflow_transitions` JOIN `workflow_instances`, two tables no code path ever
//! INSERTs into, so the page it fed was empty in every environment. The audit
//! trail lives in `perlengkapan.audit_log` and is served by `crate::audit`
//! (`GET /audit`), which is what the admin page now calls.
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
    create_master_record, delete_master_record, list_master_records, list_master_sources,
    update_master_record,
};
pub use templates::{get_template, list_templates, preview_template};
