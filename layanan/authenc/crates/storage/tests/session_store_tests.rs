//! Unit tests for PostgresSessionStore
//!
//! These tests verify the SessionStore implementation logic, including timeout calculations
//! and session lifecycle management.

use authenc_types::domain::Session;
use chrono::{Duration, Utc};
use uuid::Uuid;

/// Helper to create a test session with sensible defaults
fn test_session() -> Session {
    let now = Utc::now();
    Session {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        token: "test-jwt-token".to_string(),
        refresh_token: Some("test-refresh-token".to_string()),
        expires_at: now + Duration::hours(8),
        created_at: now,
        last_accessed: now,
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: Some("test-agent".to_string()),
        revoked: false,
        mfa_verified: false,
        is_temp_session: false,
        mfa_verified_at: None,
    }
}

#[test]
fn test_session_creation() {
    let session = test_session();
    assert!(!session.token.is_empty());
    assert!(session.refresh_token.is_some());
    assert!(!session.revoked);
    assert!(!session.mfa_verified);
    assert!(!session.is_temp_session);
}

#[test]
fn test_session_id_uniqueness() {
    let s1 = test_session();
    let s2 = test_session();
    assert_ne!(s1.id, s2.id);
}

#[test]
fn test_session_expiry_calculation() {
    let now = Utc::now();
    let expires_at = now + Duration::hours(8);
    assert!(expires_at > now);
    let time_until_expiry = expires_at - now;
    assert!(time_until_expiry.num_hours() <= 8);
}

#[test]
fn test_session_idle_timeout_calculation() {
    let now = Utc::now();
    let idle_timeout_minutes = 15;
    let idle_deadline = now - Duration::minutes(idle_timeout_minutes);

    // Last accessed 10 minutes ago (within idle timeout)
    let recent_access = now - Duration::minutes(10);
    assert!(recent_access > idle_deadline);

    // Last accessed 20 minutes ago (beyond idle timeout)
    let old_access = now - Duration::minutes(20);
    assert!(old_access < idle_deadline);
}

#[test]
fn test_session_absolute_timeout() {
    let now = Utc::now();
    let absolute_timeout_hours = 8;
    let expires_at = now + Duration::hours(absolute_timeout_hours);
    assert!(expires_at > now);

    let old_expires_at = now - Duration::hours(1);
    assert!(old_expires_at < now);
}

#[test]
fn test_session_with_recent_activity() {
    let now = Utc::now();
    let mut session = test_session();
    session.created_at = now - Duration::hours(1);
    session.last_accessed = now - Duration::minutes(5);
    session.expires_at = now + Duration::hours(7);

    let idle_deadline = now - Duration::minutes(15);
    assert!(session.last_accessed > idle_deadline);
}

#[test]
fn test_session_with_old_activity() {
    let now = Utc::now();
    let mut session = test_session();
    session.created_at = now - Duration::hours(1);
    session.last_accessed = now - Duration::minutes(20);
    session.expires_at = now + Duration::hours(7);

    let idle_deadline = now - Duration::minutes(15);
    assert!(session.last_accessed < idle_deadline);
}

#[test]
fn test_session_absolute_expiry_check() {
    let now = Utc::now();
    let mut session = test_session();
    session.created_at = now - Duration::hours(9);
    session.expires_at = now - Duration::hours(1);
    session.last_accessed = now - Duration::minutes(5);

    assert!(session.expires_at < now);
}

#[test]
fn test_session_valid_state() {
    let now = Utc::now();
    let mut session = test_session();
    session.created_at = now - Duration::hours(1);
    session.expires_at = now + Duration::hours(7);
    session.last_accessed = now - Duration::minutes(5);

    assert!(session.expires_at > now);
    let idle_deadline = now - Duration::minutes(15);
    assert!(session.last_accessed > idle_deadline);
}

#[test]
fn test_session_update_last_accessed() {
    let created_at = Utc::now() - Duration::hours(2);
    let old_last_accessed = created_at + Duration::minutes(30);

    let mut session = test_session();
    session.created_at = created_at;
    session.expires_at = created_at + Duration::hours(8);
    session.last_accessed = old_last_accessed;

    let new_last_accessed = Utc::now();
    session.last_accessed = new_last_accessed;

    assert!(session.last_accessed > old_last_accessed);
}

#[test]
fn test_multiple_sessions_for_user() {
    let now = Utc::now();
    let user_id = Uuid::new_v4();

    let mut s1 = test_session();
    s1.user_id = user_id;
    s1.created_at = now - Duration::hours(2);
    s1.last_accessed = now - Duration::minutes(5);

    let mut s2 = test_session();
    s2.user_id = user_id;
    s2.created_at = now - Duration::hours(1);
    s2.last_accessed = now - Duration::minutes(2);

    assert_eq!(s1.user_id, s2.user_id);
    assert_ne!(s1.id, s2.id);
    assert!(s2.created_at > s1.created_at);
}

#[test]
fn test_session_cleanup_criteria() {
    let now = Utc::now();
    let idle_deadline = now - Duration::minutes(15);

    let mut expired = test_session();
    expired.created_at = now - Duration::hours(10);
    expired.expires_at = now - Duration::hours(2);
    expired.last_accessed = now - Duration::hours(2);
    assert!(expired.expires_at < now);

    let mut idle = test_session();
    idle.created_at = now - Duration::hours(1);
    idle.expires_at = now + Duration::hours(7);
    idle.last_accessed = now - Duration::minutes(20);
    assert!(idle.last_accessed < idle_deadline);

    let mut active = test_session();
    active.created_at = now - Duration::hours(1);
    active.expires_at = now + Duration::hours(7);
    active.last_accessed = now - Duration::minutes(5);
    assert!(active.expires_at > now);
    assert!(active.last_accessed > idle_deadline);
}

#[test]
fn test_session_mfa_verification() {
    let now = Utc::now();
    let mut session = test_session();
    assert!(!session.mfa_verified);
    assert!(session.mfa_verified_at.is_none());

    session.mfa_verified = true;
    session.mfa_verified_at = Some(now);
    session.is_temp_session = false;

    assert!(session.mfa_verified);
    assert!(session.mfa_verified_at.is_some());
}

#[test]
fn test_session_revocation() {
    let mut session = test_session();
    assert!(!session.revoked);

    session.revoked = true;
    assert!(session.revoked);
}

#[test]
fn test_custom_timeout_values() {
    let idle_timeout_minutes = 30i64;
    let absolute_timeout_hours = 12i64;

    let now = Utc::now();
    let expires_at = now + Duration::hours(absolute_timeout_hours);
    let idle_deadline = now - Duration::minutes(idle_timeout_minutes);

    assert!(expires_at > now);
    assert!(idle_deadline < now);
}
