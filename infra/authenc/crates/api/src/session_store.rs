//! Temporary session storage for WebAuthn registration and authentication flows
//!
//! NOTE: This is an in-memory implementation for development.
//! For production, use Redis or a distributed cache.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use authenc_webauthn::{AuthenticationSession, RegistrationSession};

/// Session store for WebAuthn flows
#[derive(Clone)]
pub struct SessionStore {
    /// Registration sessions (session_id -> RegistrationSession)
    registration_sessions: Arc<RwLock<HashMap<String, RegistrationSession>>>,
    /// Authentication sessions (session_id -> AuthenticationSession)
    authentication_sessions: Arc<RwLock<HashMap<String, AuthenticationSession>>>,
}

impl SessionStore {
    /// Create a new session store
    pub fn new() -> Self {
        Self {
            registration_sessions: Arc::new(RwLock::new(HashMap::new())),
            authentication_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store a registration session
    pub async fn store_registration_session(
        &self,
        session_id: String,
        session: RegistrationSession,
    ) {
        let mut sessions = self.registration_sessions.write().await;
        sessions.insert(session_id, session);
    }

    /// Get a registration session
    pub async fn get_registration_session(&self, session_id: &str) -> Option<RegistrationSession> {
        let sessions = self.registration_sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// Remove a registration session
    pub async fn remove_registration_session(&self, session_id: &str) -> Option<RegistrationSession> {
        let mut sessions = self.registration_sessions.write().await;
        sessions.remove(session_id)
    }

    /// Store an authentication session
    pub async fn store_authentication_session(
        &self,
        session_id: String,
        session: AuthenticationSession,
    ) {
        let mut sessions = self.authentication_sessions.write().await;
        sessions.insert(session_id, session);
    }

    /// Get an authentication session
    pub async fn get_authentication_session(&self, session_id: &str) -> Option<AuthenticationSession> {
        let sessions = self.authentication_sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// Remove an authentication session
    pub async fn remove_authentication_session(&self, session_id: &str) -> Option<AuthenticationSession> {
        let mut sessions = self.authentication_sessions.write().await;
        sessions.remove(session_id)
    }

    /// Clean up expired sessions (older than 10 minutes)
    pub async fn cleanup_expired_sessions(&self) {
        let now = chrono::Utc::now();
        let expiry_duration = chrono::Duration::minutes(10);

        // Clean registration sessions
        {
            let mut sessions = self.registration_sessions.write().await;
            sessions.retain(|_, session| {
                now.signed_duration_since(session.created_at) < expiry_duration
            });
        }

        // Clean authentication sessions
        {
            let mut sessions = self.authentication_sessions.write().await;
            sessions.retain(|_, session| {
                now.signed_duration_since(session.created_at) < expiry_duration
            });
        }
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authenc_types::UserId;
    use webauthn_rs::prelude::*;

    #[tokio::test]
    async fn test_registration_session_storage() {
        let store = SessionStore::new();
        let session_id = Uuid::new_v4().to_string();

        // Create a mock registration session
        let user_id = UserId(Uuid::new_v4());
        let session = RegistrationSession {
            user_id,
            state: PasskeyRegistration::default(),
            created_at: chrono::Utc::now(),
        };

        // Store session
        store.store_registration_session(session_id.clone(), session.clone()).await;

        // Retrieve session
        let retrieved = store.get_registration_session(&session_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user_id, user_id);

        // Remove session
        let removed = store.remove_registration_session(&session_id).await;
        assert!(removed.is_some());

        // Verify removed
        let retrieved = store.get_registration_session(&session_id).await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_authentication_session_storage() {
        let store = SessionStore::new();
        let session_id = Uuid::new_v4().to_string();

        // Create a mock authentication session
        let user_id = Some(UserId(Uuid::new_v4()));
        let session = AuthenticationSession {
            user_id,
            state: PasskeyAuthentication::default(),
            created_at: chrono::Utc::now(),
        };

        // Store session
        store.store_authentication_session(session_id.clone(), session.clone()).await;

        // Retrieve session
        let retrieved = store.get_authentication_session(&session_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user_id, user_id);

        // Remove session
        let removed = store.remove_authentication_session(&session_id).await;
        assert!(removed.is_some());

        // Verify removed
        let retrieved = store.get_authentication_session(&session_id).await;
        assert!(retrieved.is_none());
    }
}
