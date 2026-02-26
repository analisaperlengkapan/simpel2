//! IAM admin HTTP request handlers

// Core admin handlers
pub mod admin;
pub mod audit;
pub mod clients;
pub mod federation;
pub mod realms;
pub mod roles;
pub mod users;

// Group and organization management
pub mod groups;
pub mod organizations;
pub mod satker;

// JIT provisioning
pub mod jit_admin;

// Client management
pub mod client_policy;
pub mod client_registration;
pub mod dcr_admin;

// Federation and SSO
pub mod federation_admin;

// SPI (Service Provider Interface)
pub mod spi_federation;
pub mod spi_management;

// UMA 2.0 (User-Managed Access)
pub mod uma;

// Zero Trust
pub mod zero_trust;

// OID4VC (OpenID for Verifiable Credentials)
pub mod oid4vc;
