//! Software Statement Validator stub - RFC 7591 software_statement validation
//! TODO: Implement full JWT-based software statement validation

use authenc_storage::Database;
use authenc_types::{AuthencError, Result};
use std::sync::Arc;

#[async_trait::async_trait]
pub trait SoftwareStatementValidator: Send + Sync {
    async fn validate(&self, software_statement: &str) -> Result<serde_json::Value>;
}

pub struct ProductionSoftwareStatementValidator {
    _db: Arc<Database>,
}

impl ProductionSoftwareStatementValidator {
    pub fn new(db: Arc<Database>) -> Self {
        Self { _db: db }
    }
}

#[async_trait::async_trait]
impl SoftwareStatementValidator for ProductionSoftwareStatementValidator {
    async fn validate(&self, _software_statement: &str) -> Result<serde_json::Value> {
        Err(AuthencError::internal(
            "Software statement validation not yet implemented",
        ))
    }
}
