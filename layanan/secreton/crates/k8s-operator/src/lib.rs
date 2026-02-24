//! Secreton Kubernetes Operator
//!
//! This crate provides a Kubernetes operator for synchronizing secrets from Secreton
//! to Kubernetes Secret objects.

pub mod controller;
pub mod crd;
pub mod error;
pub mod reconciler;

pub use crd::SecretSync;
pub use error::{Error, Result};
