//! Perlengkapan's role reference catalog.
//!
//! Presentation metadata (colour, icon) for the roles this service understands,
//! plus the per-role capability narrative rendered by `AdminRolesPage`.
//!
//! # Why this is not the old `role_switcher`
//!
//! `PerlengkapanRole` used to live in `components/role_switcher.rs`, next to a
//! dropdown that let the user *become* any of these roles client-side. The
//! catalog and the switcher were separate concerns sharing a file; when the
//! switcher was removed (it let the UI assert a role the JWT did not grant),
//! the catalog was worth keeping and worth relocating.
//!
//! Labels come from [`lib_core::authz::role_label`] so perlengkapan and the
//! portal cannot drift apart on what a role is called.

use lib_core::authz::role_label;

/// One role, described for display.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PerlengkapanRole {
    /// Exact realm role string, as it appears in the token.
    pub key: String,
    /// Display label, sourced from the shared catalog.
    pub label: String,
    /// One-line scope description.
    pub description: String,
    /// Accent colour name, mapped to a Tailwind gradient by the caller.
    pub color: String,
    /// Font Awesome class.
    pub icon: String,
}

impl PerlengkapanRole {
    fn new(key: &str, description: &str, color: &str, icon: &str) -> Self {
        Self {
            key: key.to_string(),
            label: role_label(key),
            description: description.to_string(),
            color: color.to_string(),
            icon: icon.to_string(),
        }
    }

    /// The roles this page documents, ordered from the narrowest scope to the
    /// widest — the same order the backend's `SatkerScope` tiers follow.
    ///
    /// The administrative entries are **derived** from
    /// [`lib_core::authz::ADMIN_ROLES`] rather than written out, because a
    /// hand-kept list is how this page came to document only `admin` while
    /// `admin_pusat` and `superadmin` were both accepted by every server-side
    /// admin guard. A reference page that omits two live administrator roles
    /// is not merely incomplete: it is the page an operator reads to decide
    /// what to request, and it described a role model the API does not have.
    pub fn all_roles() -> Vec<Self> {
        let mut roles = vec![
            Self::new(
                "operator_satker",
                "Pengelola perlengkapan di tingkat Satuan Kerja",
                "blue",
                "fas fa-keyboard",
            ),
            Self::new(
                "validator_wilayah",
                "Verifikator di tingkat Kejaksaan Tinggi",
                "amber",
                "fas fa-check-double",
            ),
            Self::new(
                "validator_pusat",
                "Verifikator akhir di Kejaksaan Agung",
                "emerald",
                "fas fa-stamp",
            ),
        ];

        // Administrative roles, in the shared allowlist's order. The colour is
        // uniform because they carry identical authority; the label and scope
        // text come from the shared catalog via `role_label`, so this page
        // cannot call a role something the portal calls something else.
        for key in lib_core::authz::ADMIN_ROLES {
            let description = match *key {
                "admin" => "Administrator sistem perlengkapan",
                "admin_pusat" => "Administrator tingkat Kejaksaan Agung",
                "superadmin" => "Administrator dengan akses tertinggi",
                // A role added to the allowlist without a scope line here still
                // renders; `role_label` gives it a readable name.
                _ => "Administrator sistem",
            };
            roles.push(Self::new(key, description, "red", "fas fa-user-shield"));
        }

        roles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every administrator the backend accepts must appear on the reference
    /// page. The regression this pins: the page listed only `admin`, so
    /// `admin_pusat` — the role the portal actually assigns to Kejaksaan Agung
    /// administrators — was documented nowhere, and `superadmin` (accepted by
    /// `is_admin_role`) appeared as no role at all.
    #[test]
    fn catalog_documents_every_admin_role() {
        let keys: Vec<String> = PerlengkapanRole::all_roles()
            .into_iter()
            .map(|r| r.key)
            .collect();

        for admin in lib_core::authz::ADMIN_ROLES {
            assert!(
                keys.iter().any(|k| k == admin),
                "administrative role '{admin}' is missing from the role reference page"
            );
        }
    }

    /// The page must not document a role that does not exist. The old catalog
    /// was one source of the `super_admin` (underscore) spelling, which no
    /// issuer mints and which `is_admin_role` rejects.
    #[test]
    fn catalog_has_no_phantom_roles() {
        for role in PerlengkapanRole::all_roles() {
            assert!(
                lib_core::authz::role_info(&role.key).is_some(),
                "'{}' is documented but is not a known role",
                role.key
            );
        }
    }

    /// Labels come from the shared catalog, so perlengkapan and the portal
    /// cannot disagree about what a role is called.
    #[test]
    fn labels_match_the_shared_catalog() {
        for role in PerlengkapanRole::all_roles() {
            assert_eq!(
                role.label,
                lib_core::authz::role_label(&role.key),
                "role '{}' is labelled differently from the shared catalog",
                role.key
            );
        }
    }

    /// The page covers the satker-bound roles too, not just administrators.
    #[test]
    fn catalog_covers_satker_roles() {
        let keys: Vec<String> = PerlengkapanRole::all_roles()
            .into_iter()
            .map(|r| r.key)
            .collect();
        for role in ["operator_satker", "validator_wilayah", "validator_pusat"] {
            assert!(keys.iter().any(|k| k == role), "'{role}' missing");
        }
    }
}
