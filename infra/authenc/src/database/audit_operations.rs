//! Enhanced audit operations with tamper-proof signatures
//!
//! Provides database operations for storing and retrieving audit events
//! with HMAC-SHA256 signatures for integrity verification.

use crate::database::Database;
use crate::error::{AuthencError, Result};
use crate::models::events::{AdminEvent, Event};
use crate::services::audit_signature::AuditSignatureService;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

/// Store a user event with signature in the database
pub async fn store_event_with_signature(
    db: &Database,
    event: &Event,
    signature_service: &Arc<AuditSignatureService>,
) -> Result<()> {
    // Generate signature
    let signature = signature_service.sign_event(event).map_err(|e| {
        error!("Failed to generate event signature: {}", e);
        AuthencError::internal("Failed to generate event signature")
    })?;

    let details_json = serde_json::to_string(&event.details).map_err(|e| {
        error!("Failed to serialize event details: {}", e);
        AuthencError::validation("Failed to serialize event details")
    })?;

    let query = r#"
        INSERT INTO events (
            id, time, event_type, realm_id, realm_name, client_id,
            user_id, session_id, ip_address, error, details, signature
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
    "#;

    db.execute_prepared(
        query,
        &[
            &Uuid::parse_str(&event.id)
                .map_err(|_| AuthencError::validation("Invalid event ID"))?,
            &event.time,
            &event.event_type.as_str(),
            &event.realm_id,
            &event.realm_name,
            &event.client_id,
            &event.user_id,
            &event.session_id,
            &event.ip_address,
            &event.error,
            &details_json,
            &signature,
        ],
    )
    .await
    .map_err(|e| {
        error!("Failed to store event: {}", e);
        AuthencError::database("Failed to store event")
    })?;

    Ok(())
}

/// Store an admin event with signature in the database
pub async fn store_admin_event_with_signature(
    db: &Database,
    event: &AdminEvent,
    signature_service: &Arc<AuditSignatureService>,
) -> Result<()> {
    // Generate signature
    let signature = signature_service.sign_admin_event(event).map_err(|e| {
        error!("Failed to generate admin event signature: {}", e);
        AuthencError::internal("Failed to generate admin event signature")
    })?;

    let query = r#"
        INSERT INTO admin_events (
            id, time, realm_id, realm_name, auth_user_id, auth_username,
            auth_ip_address, auth_user_agent, resource_type, operation_type,
            resource_path, representation, error, signature
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
    "#;

    db.execute_prepared(
        query,
        &[
            &Uuid::parse_str(&event.id)
                .map_err(|_| AuthencError::validation("Invalid admin event ID"))?,
            &event.time,
            &event.realm_id,
            &event.realm_name,
            &Uuid::parse_str(&event.auth_details.user_id)
                .map_err(|_| AuthencError::validation("Invalid auth user ID"))?,
            &event.auth_details.username,
            &event.auth_details.ip_address,
            &event.auth_details.user_agent,
            &event.resource_type.as_str(),
            &event.operation_type.as_str(),
            &event.resource_path,
            &event.representation,
            &event.error,
            &signature,
        ],
    )
    .await
    .map_err(|e| {
        error!("Failed to store admin event: {}", e);
        AuthencError::database("Failed to store admin event")
    })?;

    Ok(())
}

/// Verify an event signature from the database
pub async fn verify_event_signature(
    db: &Database,
    event_id: Uuid,
    signature_service: &Arc<AuditSignatureService>,
) -> Result<bool> {
    let query = r#"
        SELECT id, time, event_type, realm_id, realm_name, client_id,
               user_id, session_id, ip_address, error, details, signature
        FROM events
        WHERE id = $1
    "#;

    let row = db.query_opt(query, &[&event_id]).await.map_err(|e| {
        error!("Failed to fetch event: {}", e);
        AuthencError::database("Failed to fetch event")
    })?;

    let row = match row {
        Some(r) => r,
        None => return Err(AuthencError::not_found("Event not found")),
    };

    let signature: Option<String> = row.get(11);

    if let Some(sig) = signature {
        // Reconstruct event from row
        use crate::models::events::EventType;
        use std::collections::HashMap;

        let event_type_str: String = row.get(2);
        let event_type = EventType::from_str(&event_type_str).unwrap_or(EventType::Login);

        let details_json: Option<String> = row.get(10);
        let details: HashMap<String, String> = details_json
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();

        let event = Event {
            id: row.get::<_, Uuid>(0).to_string(),
            time: row.get(1),
            event_type,
            realm_id: row.get(3),
            realm_name: row.get(4),
            client_id: row.get(5),
            user_id: row.get(6),
            session_id: row.get(7),
            ip_address: row.get(8),
            error: row.get(9),
            details,
        };

        // Verify signature
        match signature_service.verify_event(&event, &sig) {
            Ok(()) => Ok(true),
            Err(e) => {
                error!("Event signature verification failed: {}", e);
                Ok(false)
            }
        }
    } else {
        // No signature present
        Ok(false)
    }
}

/// Verify an admin event signature from the database
pub async fn verify_admin_event_signature(
    db: &Database,
    event_id: Uuid,
    signature_service: &Arc<AuditSignatureService>,
) -> Result<bool> {
    let query = r#"
        SELECT id, time, realm_id, realm_name, auth_user_id, auth_username,
               auth_ip_address, auth_user_agent, resource_type, operation_type,
               resource_path, representation, error, signature
        FROM admin_events
        WHERE id = $1
    "#;

    let row = db.query_opt(query, &[&event_id]).await.map_err(|e| {
        error!("Failed to fetch admin event: {}", e);
        AuthencError::database("Failed to fetch admin event")
    })?;

    let row = match row {
        Some(r) => r,
        None => return Err(AuthencError::not_found("Admin event not found")),
    };

    let signature: Option<String> = row.get(13);

    if let Some(sig) = signature {
        // Reconstruct admin event from row
        use crate::models::events::{AuthDetails, OperationType, ResourceType};

        let resource_type_str: String = row.get(8);
        let resource_type =
            ResourceType::from_str(&resource_type_str).unwrap_or(ResourceType::Custom);

        let operation_type_str: String = row.get(9);
        let operation_type =
            OperationType::from_str(&operation_type_str).unwrap_or(OperationType::Action);

        let auth_user_id: Option<Uuid> = row.get(4);

        let event = AdminEvent {
            id: row.get::<_, Uuid>(0).to_string(),
            time: row.get(1),
            realm_id: row.get(2),
            realm_name: row.get(3),
            auth_details: AuthDetails {
                user_id: auth_user_id.map(|id| id.to_string()).unwrap_or_default(),
                username: row.get(5),
                ip_address: row.get(6),
                user_agent: row.get(7),
            },
            resource_type,
            operation_type,
            resource_path: row.get(10),
            representation: row.get(11),
            error: row.get(12),
        };

        // Verify signature
        match signature_service.verify_admin_event(&event, &sig) {
            Ok(()) => Ok(true),
            Err(e) => {
                error!("Admin event signature verification failed: {}", e);
                Ok(false)
            }
        }
    } else {
        // No signature present
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::events::{AuthDetails, EventType, OperationType, ResourceType};
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_signature_service() -> Arc<AuditSignatureService> {
        let secret_key = b"test_secret_key_32_bytes_long!!!".to_vec();
        Arc::new(AuditSignatureService::new(secret_key))
    }

    #[test]
    fn test_event_signature_generation() {
        let signature_service = create_test_signature_service();

        let event = Event {
            id: "test-id".to_string(),
            time: Utc::now(),
            event_type: EventType::Login,
            realm_id: "test-realm".to_string(),
            realm_name: Some("Test Realm".to_string()),
            client_id: Some("test-client".to_string()),
            user_id: Some("test-user".to_string()),
            session_id: Some("test-session".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            error: None,
            details: HashMap::new(),
        };

        let signature = signature_service.sign_event(&event).unwrap();
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_admin_event_signature_generation() {
        let signature_service = create_test_signature_service();

        let event = AdminEvent {
            id: "test-id".to_string(),
            time: Utc::now(),
            realm_id: "test-realm".to_string(),
            realm_name: Some("Test Realm".to_string()),
            auth_details: AuthDetails {
                user_id: "admin-user".to_string(),
                username: Some("admin".to_string()),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: Some("test-agent".to_string()),
            },
            resource_type: ResourceType::User,
            operation_type: OperationType::Create,
            resource_path: "/users/test-user".to_string(),
            representation: Some("{}".to_string()),
            error: None,
        };

        let signature = signature_service.sign_admin_event(&event).unwrap();
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64);
    }
}
