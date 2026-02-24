use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use authenc_storage::Database;
use authenc_types::{AuthencError, CreateSocialAccountRequest, SocialAccount};

use authenc_types::domain::social_account::SocialProvider;

/// Social account store for managing social account links in the database
#[derive(Debug, Clone)]
pub struct SocialAccountStore {
    /// Database instance
    database: Arc<Database>,
}

impl SocialAccountStore {
    /// Create a new social account store
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Get a reference to the database
    pub fn database(&self) -> &Arc<Database> {
        &self.database
    }
}

/// Trait for social account store operations
#[async_trait]
pub trait SocialAccountStoreTrait: Send + Sync {
    /// Get social account by ID
    async fn get_social_account(
        &self,
        account_id: Uuid,
    ) -> Result<Option<SocialAccount>, AuthencError>;

    /// Get social accounts for a user
    async fn get_user_social_accounts(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<SocialAccount>, AuthencError>;

    /// Get social account by provider and provider user ID
    async fn get_social_account_by_provider(
        &self,
        provider: &SocialProvider,
        provider_user_id: &str,
    ) -> Result<Option<SocialAccount>, AuthencError>;

    /// Check if user has social account linked for provider
    async fn has_social_account(
        &self,
        user_id: Uuid,
        provider: &SocialProvider,
    ) -> Result<bool, AuthencError>;

    /// Create a new social account link
    async fn add_social_account(
        &self,
        user_id: Uuid,
        request: CreateSocialAccountRequest,
    ) -> Result<SocialAccount, AuthencError>;

    /// Update social account information
    async fn update_social_account(
        &self,
        account_id: Uuid,
        request: CreateSocialAccountRequest,
    ) -> Result<SocialAccount, AuthencError>;

    /// Remove social account link
    async fn remove_social_account(&self, account_id: Uuid) -> Result<(), AuthencError>;

    /// Remove social account link by user and provider
    async fn remove_social_account_by_provider(
        &self,
        user_id: Uuid,
        provider: &SocialProvider,
    ) -> Result<(), AuthencError>;
}

#[async_trait]
impl SocialAccountStoreTrait for SocialAccountStore {
    async fn get_social_account(
        &self,
        _account_id: Uuid,
    ) -> Result<Option<SocialAccount>, AuthencError> {
        // TODO: Implement when operations::social_accounts is available in authenc-storage
        Err(AuthencError::database(
            "Social account operations not yet available in storage crate",
        ))
    }

    async fn get_user_social_accounts(
        &self,
        _user_id: Uuid,
    ) -> Result<Vec<SocialAccount>, AuthencError> {
        // TODO: Implement when operations::social_accounts is available in authenc-storage
        Err(AuthencError::database(
            "Social account operations not yet available in storage crate",
        ))
    }

    async fn get_social_account_by_provider(
        &self,
        _provider: &SocialProvider,
        _provider_user_id: &str,
    ) -> Result<Option<SocialAccount>, AuthencError> {
        // TODO: Implement when operations::social_accounts is available in authenc-storage
        Err(AuthencError::database(
            "Social account operations not yet available in storage crate",
        ))
    }

    async fn has_social_account(
        &self,
        _user_id: Uuid,
        _provider: &SocialProvider,
    ) -> Result<bool, AuthencError> {
        // TODO: Implement when operations::social_accounts is available in authenc-storage
        Err(AuthencError::database(
            "Social account operations not yet available in storage crate",
        ))
    }

    async fn add_social_account(
        &self,
        _user_id: Uuid,
        _request: CreateSocialAccountRequest,
    ) -> Result<SocialAccount, AuthencError> {
        // TODO: Implement when operations::social_accounts is available in authenc-storage
        Err(AuthencError::database(
            "Social account operations not yet available in storage crate",
        ))
    }

    async fn update_social_account(
        &self,
        _account_id: Uuid,
        _request: CreateSocialAccountRequest,
    ) -> Result<SocialAccount, AuthencError> {
        // TODO: Implement when operations::social_accounts is available in authenc-storage
        Err(AuthencError::database(
            "Social account operations not yet available in storage crate",
        ))
    }

    async fn remove_social_account(&self, _account_id: Uuid) -> Result<(), AuthencError> {
        // TODO: Implement when operations::social_accounts is available in authenc-storage
        Err(AuthencError::database(
            "Social account operations not yet available in storage crate",
        ))
    }

    async fn remove_social_account_by_provider(
        &self,
        _user_id: Uuid,
        _provider: &SocialProvider,
    ) -> Result<(), AuthencError> {
        // TODO: Implement when operations::social_accounts is available in authenc-storage
        Err(AuthencError::database(
            "Social account operations not yet available in storage crate",
        ))
    }
}
