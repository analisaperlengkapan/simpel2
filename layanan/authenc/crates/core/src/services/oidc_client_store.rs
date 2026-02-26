//! OIDC Client Store stub - placeholder for OIDC client management
//! TODO: Implement full OIDC client CRUD backed by authenc_storage

use authenc_storage::Database;
use authenc_types::domain::oidc_client::OidcClient;
use authenc_types::{AuthencError, Result};
use std::sync::Arc;

pub struct OidcClientStore {
    _db: Arc<Database>,
}

impl OidcClientStore {
    pub fn new(db: Arc<Database>) -> Self {
        Self { _db: db }
    }

    pub async fn get(&self, _client_id: &str) -> Result<Option<OidcClient>> {
        Ok(None) // TODO: implement
    }

    pub async fn all(&self) -> Result<Vec<OidcClient>> {
        Ok(vec![]) // TODO: implement
    }

    pub async fn add(&self, _client: OidcClient) -> Result<()> {
        Err(AuthencError::internal(
            "OidcClientStore.add not yet implemented",
        ))
    }

    pub async fn delete(&self, _client_id: &str) -> Result<bool> {
        Ok(false) // TODO: implement
    }
}
