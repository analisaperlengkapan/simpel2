//! Authentication module for Secreton
//!
//! This module provides authentication and authorization functionality
//! for the Secreton secret management system, including integration
//! with external identity providers like Authenc.

pub mod authenc_provider;

// Re-export commonly used types
pub use authenc_provider::{
    AuthProvider, AuthResult, AuthencAuthProvider, Credentials, PostQuantumValidator, PqSignature,
    TokenCache, TokenValidation, User, ValidationCache,
};
