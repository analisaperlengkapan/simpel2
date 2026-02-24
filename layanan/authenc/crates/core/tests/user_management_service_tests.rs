//! Comprehensive unit tests for UserManagementServiceImpl
//!
//! Tests cover:
//! - User creation with validation
//! - User updates (email, password, enabled status)
//! - User deletion (soft delete)
//! - User search and filtering
//! - Email verification
//! - MFA enable/disable
//! - Input validation (email format, password strength, username format)
//! - Duplicate username/email handling
//!
//! Target: >80% code coverage

use async_trait::async_trait;
use authenc_core::services::user_management_service::UserManagementServiceImpl;
use authenc_types::{domain::*, domain_types::*, error::AuthencError, result::Result, traits::*};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// ============================================================================
// Mock Implementations
// ============================================================================

struct MockUserStore {
    users: Arc<Mutex<HashMap<UserId, User>>>,
    users_by_username: Arc<Mutex<HashMap<(String, RealmId), UserId>>>,
    users_by_email: Arc<Mutex<HashMap<(String, RealmId), UserId>>>,
}

impl MockUserStore {
    fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            users_by_username: Arc::new(Mutex::new(HashMap::new())),
            users_by_email: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
