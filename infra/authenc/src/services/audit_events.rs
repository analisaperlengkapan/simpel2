//! Comprehensive Audit Event Helpers
//!
//! This module provides convenient functions for creating comprehensive audit events
//! with all required context (IP address, user agent, timestamps, etc.)

use crate::models::events::{
    AdminEvent, AuthDetails, Event, EventType, OperationType, ResourceType,
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Create a comprehensive UserLogin event with full context
///
/// # Arguments
/// * `realm_id` - The realm ID where login occurred
/// * `user_id` - The user ID who logged in
/// * `username` - The username (optional)
/// * `session_id` - The session ID created (optional)
/// * `client_id` - The client ID used for login (optional)
/// * `ip_address` - The IP address of the client
/// * `user_agent` - The user agent string of the client
/// * `auth_method` - The authentication method used (e.g., "password", "mfa", "oauth2")
/// * `success` - Whether the login was successful
///
/// # Returns
/// A fully populated Event with Login or LoginError type
pub fn create_user_login_event(
    realm_id: String,
    user_id: Option<String>,
    username: Option<String>,
    session_id: Option<String>,
    client_id: Option<String>,
    ip_address: String,
    user_agent: String,
    auth_method: String,
    success: bool,
) -> Event {
    let event_type = if success {
        EventType::Login
    } else {
        EventType::LoginError
    };

    let mut event = Event::new(event_type, realm_id);

    if let Some(uid) = user_id {
        event = event.user_id(uid);
    }

    if let Some(sid) = session_id {
        event = event.session_id(sid);
    }

    if let Some(cid) = client_id {
        event = event.client_id(cid);
    }

    event = event.ip_address(ip_address);

    // Add comprehensive details
    let mut details = HashMap::new();
    details.insert("user_agent".to_string(), user_agent);
    details.insert("auth_method".to_string(), auth_method);
    details.insert("timestamp".to_string(), Utc::now().to_rfc3339());

    if let Some(uname) = username {
        details.insert("username".to_string(), uname);
    }

    event.details(details)
}

/// Create a comprehensive UserLogout event with session duration
///
/// # Arguments
/// * `realm_id` - The realm ID where logout occurred
/// * `user_id` - The user ID who logged out
/// * `username` - The username (optional)
/// * `session_id` - The session ID being terminated
/// * `client_id` - The client ID (optional)
/// * `ip_address` - The IP address of the client
/// * `user_agent` - The user agent string of the client
/// * `session_start` - When the session started
/// * `logout_type` - The type of logout (e.g., "user_initiated", "timeout", "admin_forced")
///
/// # Returns
/// A fully populated Event with Logout type
pub fn create_user_logout_event(
    realm_id: String,
    user_id: String,
    username: Option<String>,
    session_id: String,
    client_id: Option<String>,
    ip_address: String,
    user_agent: String,
    session_start: DateTime<Utc>,
    logout_type: String,
) -> Event {
    let mut event = Event::new(EventType::Logout, realm_id);

    event = event
        .user_id(user_id)
        .session_id(session_id)
        .ip_address(ip_address);

    if let Some(cid) = client_id {
        event = event.client_id(cid);
    }

    // Calculate session duration
    let session_end = Utc::now();
    let duration = session_end.signed_duration_since(session_start);
    let duration_seconds = duration.num_seconds();

    // Add comprehensive details
    let mut details = HashMap::new();
    details.insert("user_agent".to_string(), user_agent);
    details.insert("logout_type".to_string(), logout_type);
    details.insert("session_start".to_string(), session_start.to_rfc3339());
    details.insert("session_end".to_string(), session_end.to_rfc3339());
    details.insert(
        "session_duration_seconds".to_string(),
        duration_seconds.to_string(),
    );
    details.insert("timestamp".to_string(), session_end.to_rfc3339());

    if let Some(uname) = username {
        details.insert("username".to_string(), uname);
    }

    event.details(details)
}

/// Create an MFAEnabled admin event
///
/// # Arguments
/// * `realm_id` - The realm ID where MFA was enabled
/// * `target_user_id` - The user ID for whom MFA was enabled
/// * `target_username` - The username (optional)
/// * `admin_user_id` - The admin user ID who enabled MFA
/// * `admin_username` - The admin username (optional)
/// * `ip_address` - The IP address of the admin
/// * `user_agent` - The user agent string of the admin
/// * `mfa_method` - The MFA method enabled (e.g., "totp", "webauthn")
///
/// # Returns
/// A fully populated AdminEvent for MFA enablement
pub fn create_mfa_enabled_admin_event(
    realm_id: String,
    target_user_id: String,
    target_username: Option<String>,
    admin_user_id: String,
    admin_username: Option<String>,
    ip_address: String,
    user_agent: String,
    mfa_method: String,
) -> AdminEvent {
    let auth_details = AuthDetails {
        user_id: admin_user_id.clone(),
        username: admin_username.clone(),
        ip_address: Some(ip_address),
        user_agent: Some(user_agent.clone()),
    };

    let resource_path = format!("users/{}/mfa", target_user_id);

    let mut event = AdminEvent::new(
        realm_id,
        auth_details,
        ResourceType::User,
        OperationType::Action,
        resource_path,
    );

    // Create representation with MFA details
    let representation = serde_json::json!({
        "action": "enable_mfa",
        "target_user_id": target_user_id,
        "target_username": target_username,
        "mfa_method": mfa_method,
        "enabled_by": admin_user_id,
        "enabled_by_username": admin_username,
        "timestamp": Utc::now().to_rfc3339(),
    });

    event.representation(representation.to_string())
}

/// Create an MFADisabled admin event
///
/// # Arguments
/// * `realm_id` - The realm ID where MFA was disabled
/// * `target_user_id` - The user ID for whom MFA was disabled
/// * `target_username` - The username (optional)
/// * `admin_user_id` - The admin user ID who disabled MFA
/// * `admin_username` - The admin username (optional)
/// * `ip_address` - The IP address of the admin
/// * `user_agent` - The user agent string of the admin
/// * `reason` - The reason for disabling MFA (optional)
///
/// # Returns
/// A fully populated AdminEvent for MFA disablement
pub fn create_mfa_disabled_admin_event(
    realm_id: String,
    target_user_id: String,
    target_username: Option<String>,
    admin_user_id: String,
    admin_username: Option<String>,
    ip_address: String,
    user_agent: String,
    reason: Option<String>,
) -> AdminEvent {
    let auth_details = AuthDetails {
        user_id: admin_user_id.clone(),
        username: admin_username.clone(),
        ip_address: Some(ip_address),
        user_agent: Some(user_agent.clone()),
    };

    let resource_path = format!("users/{}/mfa", target_user_id);

    let mut event = AdminEvent::new(
        realm_id,
        auth_details,
        ResourceType::User,
        OperationType::Action,
        resource_path,
    );

    // Create representation with MFA details
    let mut representation_data = serde_json::json!({
        "action": "disable_mfa",
        "target_user_id": target_user_id,
        "target_username": target_username,
        "disabled_by": admin_user_id,
        "disabled_by_username": admin_username,
        "timestamp": Utc::now().to_rfc3339(),
    });

    if let Some(r) = reason {
        representation_data["reason"] = serde_json::Value::String(r);
    }

    event.representation(representation_data.to_string())
}

/// Create a PermissionGranted admin event
///
/// # Arguments
/// * `realm_id` - The realm ID where permission was granted
/// * `target_user_id` - The user ID who received the permission
/// * `target_username` - The username (optional)
/// * `admin_user_id` - The admin user ID who granted the permission
/// * `admin_username` - The admin username (optional)
/// * `ip_address` - The IP address of the admin
/// * `user_agent` - The user agent string of the admin
/// * `resource` - The resource for which permission was granted
/// * `action` - The action permitted (e.g., "read", "write", "delete")
/// * `scope` - The scope of the permission (optional)
///
/// # Returns
/// A fully populated AdminEvent for permission grant
pub fn create_permission_granted_admin_event(
    realm_id: String,
    target_user_id: String,
    target_username: Option<String>,
    admin_user_id: String,
    admin_username: Option<String>,
    ip_address: String,
    user_agent: String,
    resource: String,
    action: String,
    scope: Option<String>,
) -> AdminEvent {
    let auth_details = AuthDetails {
        user_id: admin_user_id.clone(),
        username: admin_username.clone(),
        ip_address: Some(ip_address),
        user_agent: Some(user_agent.clone()),
    };

    let resource_path = format!("users/{}/permissions", target_user_id);

    let mut event = AdminEvent::new(
        realm_id,
        auth_details,
        ResourceType::Permission,
        OperationType::Create,
        resource_path,
    );

    // Create representation with permission details
    let mut representation_data = serde_json::json!({
        "target_user_id": target_user_id,
        "target_username": target_username,
        "resource": resource,
        "action": action,
        "granted_by": admin_user_id,
        "granted_by_username": admin_username,
        "timestamp": Utc::now().to_rfc3339(),
    });

    if let Some(s) = scope {
        representation_data["scope"] = serde_json::Value::String(s);
    }

    event.representation(representation_data.to_string())
}

/// Create a PermissionRevoked admin event
///
/// # Arguments
/// * `realm_id` - The realm ID where permission was revoked
/// * `target_user_id` - The user ID whose permission was revoked
/// * `target_username` - The username (optional)
/// * `admin_user_id` - The admin user ID who revoked the permission
/// * `admin_username` - The admin username (optional)
/// * `ip_address` - The IP address of the admin
/// * `user_agent` - The user agent string of the admin
/// * `resource` - The resource for which permission was revoked
/// * `action` - The action that was revoked (e.g., "read", "write", "delete")
/// * `reason` - The reason for revoking (optional)
///
/// # Returns
/// A fully populated AdminEvent for permission revocation
pub fn create_permission_revoked_admin_event(
    realm_id: String,
    target_user_id: String,
    target_username: Option<String>,
    admin_user_id: String,
    admin_username: Option<String>,
    ip_address: String,
    user_agent: String,
    resource: String,
    action: String,
    reason: Option<String>,
) -> AdminEvent {
    let auth_details = AuthDetails {
        user_id: admin_user_id.clone(),
        username: admin_username.clone(),
        ip_address: Some(ip_address),
        user_agent: Some(user_agent.clone()),
    };

    let resource_path = format!("users/{}/permissions", target_user_id);

    let mut event = AdminEvent::new(
        realm_id,
        auth_details,
        ResourceType::Permission,
        OperationType::Delete,
        resource_path,
    );

    // Create representation with permission details
    let mut representation_data = serde_json::json!({
        "target_user_id": target_user_id,
        "target_username": target_username,
        "resource": resource,
        "action": action,
        "revoked_by": admin_user_id,
        "revoked_by_username": admin_username,
        "timestamp": Utc::now().to_rfc3339(),
    });

    if let Some(r) = reason {
        representation_data["reason"] = serde_json::Value::String(r);
    }

    event.representation(representation_data.to_string())
}

/// Create a PasswordChanged event
///
/// # Arguments
/// * `realm_id` - The realm ID where password was changed
/// * `user_id` - The user ID whose password was changed
/// * `username` - The username (optional)
/// * `ip_address` - The IP address of the client
/// * `user_agent` - The user agent string of the client
/// * `changed_by_admin` - Whether the password was changed by an admin (vs self-service)
/// * `admin_user_id` - The admin user ID if changed by admin (optional)
/// * `reset_token_used` - Whether a password reset token was used
///
/// # Returns
/// A fully populated Event with UpdatePassword type
pub fn create_password_changed_event(
    realm_id: String,
    user_id: String,
    username: Option<String>,
    ip_address: String,
    user_agent: String,
    changed_by_admin: bool,
    admin_user_id: Option<String>,
    reset_token_used: bool,
) -> Event {
    let mut event = Event::new(EventType::UpdatePassword, realm_id);

    event = event.user_id(user_id.clone()).ip_address(ip_address);

    // Add comprehensive details
    let mut details = HashMap::new();
    details.insert("user_agent".to_string(), user_agent);
    details.insert("changed_by_admin".to_string(), changed_by_admin.to_string());
    details.insert("reset_token_used".to_string(), reset_token_used.to_string());
    details.insert("timestamp".to_string(), Utc::now().to_rfc3339());

    if let Some(uname) = username {
        details.insert("username".to_string(), uname);
    }

    if let Some(admin_id) = admin_user_id {
        details.insert("admin_user_id".to_string(), admin_id);
    }

    event.details(details)
}

/// Create an MFAEnabled user event (for user-initiated MFA setup)
///
/// # Arguments
/// * `realm_id` - The realm ID where MFA was enabled
/// * `user_id` - The user ID who enabled MFA
/// * `username` - The username (optional)
/// * `ip_address` - The IP address of the client
/// * `user_agent` - The user agent string of the client
/// * `mfa_method` - The MFA method enabled (e.g., "totp", "webauthn")
///
/// # Returns
/// A fully populated Event with MfaEnabled type
pub fn create_mfa_enabled_event(
    realm_id: String,
    user_id: String,
    username: Option<String>,
    ip_address: String,
    user_agent: String,
    mfa_method: String,
) -> Event {
    let mut event = Event::new(EventType::MfaEnabled, realm_id);

    event = event.user_id(user_id).ip_address(ip_address);

    // Add comprehensive details
    let mut details = HashMap::new();
    details.insert("user_agent".to_string(), user_agent);
    details.insert("mfa_method".to_string(), mfa_method);
    details.insert("timestamp".to_string(), Utc::now().to_rfc3339());

    if let Some(uname) = username {
        details.insert("username".to_string(), uname);
    }

    event.details(details)
}

/// Create an MFADisabled user event (for user-initiated MFA disable)
///
/// # Arguments
/// * `realm_id` - The realm ID where MFA was disabled
/// * `user_id` - The user ID who disabled MFA
/// * `username` - The username (optional)
/// * `ip_address` - The IP address of the client
/// * `user_agent` - The user agent string of the client
///
/// # Returns
/// A fully populated Event with MfaDisabled type
pub fn create_mfa_disabled_event(
    realm_id: String,
    user_id: String,
    username: Option<String>,
    ip_address: String,
    user_agent: String,
) -> Event {
    let mut event = Event::new(EventType::MfaDisabled, realm_id);

    event = event.user_id(user_id).ip_address(ip_address);

    // Add comprehensive details
    let mut details = HashMap::new();
    details.insert("user_agent".to_string(), user_agent);
    details.insert("timestamp".to_string(), Utc::now().to_rfc3339());

    if let Some(uname) = username {
        details.insert("username".to_string(), uname);
    }

    event.details(details)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user_login_event() {
        let event = create_user_login_event(
            "test-realm".to_string(),
            Some("user-123".to_string()),
            Some("testuser".to_string()),
            Some("session-456".to_string()),
            Some("client-789".to_string()),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            "password".to_string(),
            true,
        );

        assert_eq!(event.event_type, EventType::Login);
        assert_eq!(event.realm_id, "test-realm");
        assert_eq!(event.user_id, Some("user-123".to_string()));
        assert_eq!(event.session_id, Some("session-456".to_string()));
        assert_eq!(event.ip_address, Some("192.168.1.1".to_string()));
        assert!(event.details.contains_key("user_agent"));
        assert!(event.details.contains_key("auth_method"));
        assert!(event.details.contains_key("username"));
    }

    #[test]
    fn test_create_user_logout_event() {
        let session_start = Utc::now() - chrono::Duration::hours(2);

        let event = create_user_logout_event(
            "test-realm".to_string(),
            "user-123".to_string(),
            Some("testuser".to_string()),
            "session-456".to_string(),
            Some("client-789".to_string()),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            session_start,
            "user_initiated".to_string(),
        );

        assert_eq!(event.event_type, EventType::Logout);
        assert_eq!(event.user_id, Some("user-123".to_string()));
        assert!(event.details.contains_key("session_duration_seconds"));
        assert!(event.details.contains_key("logout_type"));
    }

    #[test]
    fn test_create_mfa_enabled_admin_event() {
        let event = create_mfa_enabled_admin_event(
            "test-realm".to_string(),
            "user-123".to_string(),
            Some("testuser".to_string()),
            "admin-456".to_string(),
            Some("admin".to_string()),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            "totp".to_string(),
        );

        assert_eq!(event.resource_type, ResourceType::User);
        assert_eq!(event.operation_type, OperationType::Action);
        assert_eq!(event.auth_details.user_id, "admin-456");
        assert!(event.representation.is_some());
    }

    #[test]
    fn test_create_permission_granted_admin_event() {
        let event = create_permission_granted_admin_event(
            "test-realm".to_string(),
            "user-123".to_string(),
            Some("testuser".to_string()),
            "admin-456".to_string(),
            Some("admin".to_string()),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            "documents".to_string(),
            "write".to_string(),
            Some("satker:12345".to_string()),
        );

        assert_eq!(event.resource_type, ResourceType::Permission);
        assert_eq!(event.operation_type, OperationType::Create);
        assert!(event.representation.is_some());

        let repr: serde_json::Value = serde_json::from_str(&event.representation.unwrap()).unwrap();
        assert_eq!(repr["resource"], "documents");
        assert_eq!(repr["action"], "write");
        assert_eq!(repr["scope"], "satker:12345");
    }

    #[test]
    fn test_create_password_changed_event() {
        let event = create_password_changed_event(
            "test-realm".to_string(),
            "user-123".to_string(),
            Some("testuser".to_string()),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            false,
            None,
            false,
        );

        assert_eq!(event.event_type, EventType::UpdatePassword);
        assert_eq!(event.user_id, Some("user-123".to_string()));
        assert!(event.details.contains_key("changed_by_admin"));
        assert_eq!(event.details.get("changed_by_admin").unwrap(), "false");
    }
}
