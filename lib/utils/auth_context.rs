//! Authentication context utilities
//!
//! This module provides a generic authentication context structure that can be
//! used with different backend service implementations. The actual service
//! implementations are provided by the consuming crate (e.g., authenc).

use std::sync::Arc;

/// Generic trait for user store operations
pub trait UserStoreOps: Send + Sync + 'static {}

/// Generic trait for TOTP store operations
pub trait TotpStoreOps: Send + Sync + 'static {}

/// Generic trait for session store operations
pub trait SessionStoreOps: Send + Sync + 'static {}

/// Generic trait for brute force protection
pub trait BruteForceProtectorOps: Send + Sync + 'static {}

/// Generic trait for anomaly detection
pub trait AnomalyDetectorOps: Send + Sync + 'static {}

/// Generic trait for federation registry
pub trait FederationRegistryOps: Send + Sync + 'static {}

/// Generic authentication context that aggregates all dependencies for authentication handlers.
/// This struct is injected as a single dependency to simplify handler signatures and improve maintainability.
///
/// # Type Parameters
/// * `U` - User store implementation
/// * `T` - TOTP store implementation
/// * `S` - Session store implementation
/// * `B` - Brute force protector implementation
/// * `A` - Anomaly detector implementation
/// * `F` - Federation registry implementation
#[derive(Clone)]
pub struct AuthContext<U, T, S, B, A, F>
where
    U: UserStoreOps,
    T: TotpStoreOps,
    S: SessionStoreOps,
    B: BruteForceProtectorOps,
    A: AnomalyDetectorOps,
    F: FederationRegistryOps,
{
    /// Store for user data and authentication
    pub user_store: Arc<U>,
    /// Store for TOTP secrets
    pub totp_store: Arc<T>,
    /// Store for session tokens
    pub session_store: Arc<S>,
    /// Brute-force protection logic
    pub brute_force: Arc<B>,
    /// Anomaly detection (e.g., new IPs)
    pub anomaly_detector: Arc<A>,
    /// Federation registry for external user sources
    pub federation_registry: Arc<F>,
}

impl<U, T, S, B, A, F> AuthContext<U, T, S, B, A, F>
where
    U: UserStoreOps,
    T: TotpStoreOps,
    S: SessionStoreOps,
    B: BruteForceProtectorOps,
    A: AnomalyDetectorOps,
    F: FederationRegistryOps,
{
    /// Create a new authentication context with all dependencies
    pub fn new(
        user_store: Arc<U>,
        totp_store: Arc<T>,
        session_store: Arc<S>,
        brute_force: Arc<B>,
        anomaly_detector: Arc<A>,
        federation_registry: Arc<F>,
    ) -> Self {
        Self {
            user_store,
            totp_store,
            session_store,
            brute_force,
            anomaly_detector,
            federation_registry,
        }
    }
}
