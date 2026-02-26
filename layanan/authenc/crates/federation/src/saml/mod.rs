//! SAML Federation Module
//!
//! This module provides SAML 2.0 authentication and federation capabilities.

pub mod service;
pub mod signature;

pub use service::*;
pub use signature::*;
