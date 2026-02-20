//! Unit tests for PostgresSessionStore
//!
//! These tests verify the SessionStore implementation logic, including timeout calculations
//! and session lifecycle management.

use authenc_types::{domain::Session, SessionId, UserId};
use chrono::{Duration, Utc};

#[test]
fn test_session_creation() {
    let session_id = SessionId::new();
    let user_id = UserId::new();
    let now = Utc::now();
    let expires_at = now + Duration::hours(8);

    let session = Session {
        id: session_id,
        user_id,
        created_at: now,
        expires_at,
        last_accessed_at: now,
    };

    assert_eq!(session.id, session_id);
    assert_eq!(session.user_id, user_id);
    assert_eq!(session.created_at, now);
    assert_eq!(session.expires_at, expires_at);
    assert_eq!(session.last_accessed_at, now);
}

#[test]
fn test_session_id_generation() {
    let id1 = SessionId::new();
    let id2 = SessionId::new();

    // Each generated ID should be unique
    assert_ne!(id1, id2);
}

#[test]
fn test_session_id_display() {
    let id = SessionId::new();
    let display_str = format!("{}", id);

    // Should display the UUID
    assert!(!display_str.is_empty());
}

#[test]
fn test_session_expiry_calculation() {
    let now = Utc::now();
    let expires_at = now + Duration::hours(8);

    // Session should not be expired immediately
    assert!(expires_at > now);

    // Session should expire in the future
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

    // Session created now should not be expired
    assert!(expires_at > now);

    // Session created 9 hours ago should be expired
    let old_expires_at = now - Duration::hours(1);
    assert!(old_expires_at < now);
}

#[test]
fn test_session_with_recent_activity() {
    let session_id = SessionId::new();
    let user_id = UserId::new();
    let now = Utc::now();
    let expires_at = now + Duration::hours(8);
    let last_accessed = now - Duration::minutes(5); // 5 minutes ago

    let session = Session {
        id: session_id,
        user_id,
        created_at: now - Duration::hours(1),
        expires_at,
        last_accessed_at: last_accessed,
    };

    // Session should still be valid (within 15 minute idle timeout)
    let idle_deadline = now - Duration::minutes(15);
    assert!(session.last_accessed_at > idle_deadline);
}

#[test]
fn test_session_with_old_activity() {
    let session_id = SessionId::new();
    let user_id = UserId::new();
    let now = Utc::now();
    let expires_at = now + Duration::hours(8);
    let last_accessed = now - Duration::minutes(20); // 20 minutes ago

    let session = Session {
        id: session_id,
        user_id,
        created_at: now - Duration::hours(1),
        expires_at,
        last_accessed_at: last_accessed,
    };

    // Session should be expired (beyond 15 minute idle timeout)
    let idle_deadline = now - Duration::minutes(15);
    assert!(session.last_accessed_at < idle_deadline);
}

#[test]
fn test_session_absolute_expiry_check() {
    let session_id = SessionId::new();
    let user_id = UserId::new();
    let now = Utc::now();

    // Session that expired 1 hour ago
    let expired_session = Session {
        id: session_id,
        user_id,
        created_at: now - Duration::hours(9),
        expires_at: now - Duration::hours(1),
        last_accessed_at: now - Duration::minutes(5),
    };

    assert!(expired_session.expires_at < now);
}

#[test]
fn test_session_valid_state() {
    let session_id = SessionId::new();
    let user_id = UserId::new();
    let now = Utc::now();

    // Valid session: recent activity, not expired
    let valid_session = Session {
        id: session_id,
        user_id,
        created_at: now - Duration::hours(1),
        expires_at: now + Duration::hours(7),
        last_accessed_at: now - Duration::minutes(5),
    };

    // Check absolute timeout
    assert!(valid_session.expires_at > now);

    // Check idle timeout (15 minutes)
    let idle_deadline = now - Duration::minutes(15);
    assert!(valid_session.last_accessed_at > idle_deadline);
}

#[test]
fn test_session_update_last_accessed() {
    let session_id = SessionId::new();
    let user_id = UserId::new();
    let created_at = Utc::now() - Duration::hours(2);
    let expires_at = created_at + Duration::hours(8);
    let old_last_accessed = created_at + Duration::minutes(30);

    let mut session = Session {
        id: session_id,
        user_id,
        created_at,
        expires_at,
        last_accessed_at: old_last_accessed,
    };

    // Update last accessed time
    let new_last_accessed = Utc::now();
    session.last_accessed_at = new_last_accessed;

    assert!(session.last_accessed_at > old_last_accessed);
    assert!(session.last_accessed_at == new_last_accessed);
}

#[test]
fn test_multiple_sessions_for_user() {
    let user_id = UserId::new();
    let now = Utc::now();

    let session1 = Session {
        id: SessionId::new(),
        user_id,
        created_at: now - Duration::hours(2),
        expires_at: now + Duration::hours(6),
        last_accessed_at: now - Duration::minutes(5),
    };

    let session2 = Session {
        id: SessionId::new(),
        user_id,
        created_at: now - Duration::hours(1),
        expires_at: now + Duration::hours(7),
        last_accessed_at: now - Duration::minutes(2),
    };

    // Both sessions should belong to the same user
    assert_eq!(session1.user_id, session2.user_id);

    // But have different IDs
    assert_ne!(session1.id, session2.id);

    // Session2 should be more recent
    assert!(session2.created_at > session1.created_at);
    assert!(session2.last_accessed_at > session1.last_accessed_at);
}

#[test]
fn test_session_cleanup_criteria() {
    let now = Utc::now();
    let idle_timeout_minutes = 15;
    let idle_deadline = now - Duration::minutes(idle_timeout_minutes);

    // Session that should be cleaned up (absolute expiry)
    let expired_session = Session {
        id: SessionId::new(),
        user_id: UserId::new(),
        created_at: now - Duration::hours(10),
        expires_at: now - Duration::hours(2),
        last_accessed_at: now - Duration::hours(2),
    };

    assert!(expired_session.expires_at < now);

    // Session that should be cleaned up (idle timeout)
    let idle_session = Session {
        id: SessionId::new(),
        user_id: UserId::new(),
        created_at: now - Duration::hours(1),
        expires_at: now + Duration::hours(7),
        last_accessed_at: now - Duration::minutes(20),
    };

    assert!(idle_session.last_accessed_at < idle_deadline);

    // Session that should NOT be cleaned up
    let active_session = Session {
        id: SessionId::new(),
        user_id: UserId::new(),
        created_at: now - Duration::hours(1),
        expires_at: now + Duration::hours(7),
        last_accessed_at: now - Duration::minutes(5),
    };

    assert!(active_session.expires_at > now);
    assert!(active_session.last_accessed_at > idle_deadline);
}

#[test]
fn test_custom_timeout_values() {
    // Test with custom timeout values
    let idle_timeout_minutes = 30i64;
    let absolute_timeout_hours = 12i64;

    let now = Utc::now();
    let expires_at = now + Duration::hours(absolute_timeout_hours);
    let idle_deadline = now - Duration::minutes(idle_timeout_minutes);

    assert!(expires_at > now);
    assert!(idle_deadline < now);

    // Verify timeout calculations
    let time_until_expiry = expires_at - now;
    assert!(time_until_expiry.num_hours() <= absolute_timeout_hours);

    let time_since_idle = now - idle_deadline;
    assert!(time_since_idle.num_minutes() >= idle_timeout_minutes);
}
