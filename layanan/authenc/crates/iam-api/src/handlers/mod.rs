//! IAM admin HTTP request handlers
//!
//! Trimmed to the surfaces the platform actually uses (task #45 audit):
//! system stats, user management, role listing/assignment, OAuth2 client
//! inspection, and audit-log access. The Keycloak-parity modules that were
//! routed but 100% `NotImplemented` stubs (organizations, groups, realms,
//! federation, SPI, JIT, DCR, client policies, UMA, zero-trust, OID4VC,
//! satker admin) were deleted — none had a real consumer or a backing
//! service. They return together with their FE pages + e2e when a real
//! need appears.

pub mod admin;
pub mod audit;
pub mod clients;
pub mod roles;
pub mod users;
