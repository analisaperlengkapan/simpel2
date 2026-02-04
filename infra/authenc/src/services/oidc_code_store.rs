use crate::AuthencError;
use crate::database::Database;
use crate::error::Result;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

/// OIDC authorization code store for managing OAuth2 authorization codes
pub struct OidcCodeStore {
    /// Database connection
    db: Arc<Database>,
    /// Time-to-live for authorization codes in seconds
    ttl: u64,
}

impl OidcCodeStore {
    /// Create new OIDC code store
    ///
    /// # Note
    /// This method requires a database connection. Use `with_database()` instead.
    ///
    /// # Panics
    /// This method will panic if called. It exists only for backward compatibility.
    pub fn new(_ttl_secs: u64) -> Self {
        panic!(
            "OidcCodeStore requires database connection. Use OidcCodeStore::with_database() instead."
        );
    }

    /// Create OIDC code store with database connection and TTL (recommended for production)
    pub fn with_database(db: Arc<Database>, ttl_secs: u64) -> Self {
        Self { db, ttl: ttl_secs }
    }

    /// Insert new authorization code
    #[allow(clippy::too_many_arguments)]
    pub async fn insert(
        &self,
        code: String,
        client_id: String,
        user_id: String,
        redirect_uri: String,
        scopes: Vec<String>,
        code_challenge: Option<String>,
        code_challenge_method: Option<String>,
    ) -> Result<()> {
        use crate::database::operations::oauth2;
        use crate::models::OAuth2AuthorizationCode;

        // Lookup client UUID from string client_id
        let client = oauth2::get_client_by_client_id(&self.db, &client_id)
            .await?
            .ok_or_else(|| AuthencError::validation("Invalid client_id"))?;

        // Parse user_id as UUID
        let user_uuid = Uuid::parse_str(&user_id)
            .map_err(|_| AuthencError::validation("Invalid user_id format"))?;

        let auth_code = OAuth2AuthorizationCode {
            id: Uuid::new_v4(),
            code: code.clone(),
            client_id: client.id,
            user_id: user_uuid,
            redirect_uri,
            scopes,
            code_challenge,
            code_challenge_method,
            expires_at: Utc::now() + chrono::Duration::seconds(self.ttl as i64),
            used: false,
            created_at: Utc::now(),
        };

        oauth2::store_authorization_code(&self.db, &auth_code).await?;
        Ok(())
    }

    /// Take and consume authorization code, returning the full code record if valid
    pub async fn take(
        &self,
        code: &str,
        client_id: &str,
    ) -> Result<Option<crate::models::OAuth2AuthorizationCode>> {
        use crate::database::operations::oauth2;

        // Lookup client UUID from string client_id
        let client = oauth2::get_client_by_client_id(&self.db, client_id)
            .await?
            .ok_or_else(|| AuthencError::validation("Invalid client_id"))?;

        match oauth2::get_authorization_code(&self.db, code).await? {
            Some(auth_code) => {
                // Verify client matches
                if auth_code.client_id != client.id {
                    return Ok(None);
                }

                // Mark code as used
                oauth2::mark_code_used(&self.db, code).await?;

                Ok(Some(auth_code))
            }
            None => Ok(None),
        }
    }
}
