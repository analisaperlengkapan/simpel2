use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use authenc_storage::Database;
use authenc_types::{AuthencError, CreateUserRequest, UpdateUserRequest, User};

/// User store for managing users in the database
#[derive(Debug, Clone)]
pub struct UserStore {
    /// Database instance
    database: Arc<Database>,
}

impl UserStore {
    /// Create a new user store
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Get a reference to the database
    pub fn database(&self) -> &Arc<Database> {
        &self.database
    }
}

/// Trait for user store operations
#[async_trait]
pub trait UserStoreTrait: Send + Sync {
    /// Get user by ID
    async fn get_user(&self, user_id: Uuid) -> Result<Option<User>, AuthencError>;

    /// Get user by username
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, AuthencError>;

    /// Get user by email
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AuthencError>;

    /// Create a new user
    async fn add_user(&self, request: CreateUserRequest) -> Result<User, AuthencError>;

    /// Update user
    async fn update_user(
        &self,
        user_id: Uuid,
        request: UpdateUserRequest,
    ) -> Result<User, AuthencError>;

    /// Delete user (soft delete)
    async fn delete_user(&self, user_id: Uuid) -> Result<(), AuthencError>;

    /// Get all users
    async fn get_all(&self) -> Result<Vec<User>, AuthencError>;

    /// Get users by realm
    async fn get_users_by_realm(&self, realm_id: Uuid) -> Result<Vec<User>, AuthencError>;
}

/// Implementation of UserStoreTrait for UserStore
#[async_trait]
impl UserStoreTrait for UserStore {
    async fn get_user(&self, _user_id: Uuid) -> Result<Option<User>, AuthencError> {
        // TODO: Implement when operations::users is available in authenc-storage
        Err(AuthencError::database(
            "User operations not yet available in storage crate",
        ))
    }

    async fn get_user_by_username(&self, _username: &str) -> Result<Option<User>, AuthencError> {
        // TODO: Implement when operations::users is available in authenc-storage
        Err(AuthencError::database(
            "User operations not yet available in storage crate",
        ))
    }

    async fn get_user_by_email(&self, _email: &str) -> Result<Option<User>, AuthencError> {
        // TODO: Implement when operations::users is available in authenc-storage
        Err(AuthencError::database(
            "User operations not yet available in storage crate",
        ))
    }

    async fn add_user(&self, _request: CreateUserRequest) -> Result<User, AuthencError> {
        // TODO: Implement when operations::users is available in authenc-storage
        Err(AuthencError::database(
            "User operations not yet available in storage crate",
        ))
    }

    async fn update_user(
        &self,
        _user_id: Uuid,
        _request: UpdateUserRequest,
    ) -> Result<User, AuthencError> {
        // TODO: Implement when operations::users is available in authenc-storage
        Err(AuthencError::database(
            "User operations not yet available in storage crate",
        ))
    }

    async fn delete_user(&self, _user_id: Uuid) -> Result<(), AuthencError> {
        // TODO: Implement when operations::users is available in authenc-storage
        Err(AuthencError::database(
            "User operations not yet available in storage crate",
        ))
    }

    async fn get_all(&self) -> Result<Vec<User>, AuthencError> {
        // TODO: Implement when operations::users is available in authenc-storage
        Err(AuthencError::database(
            "User operations not yet available in storage crate",
        ))
    }

    async fn get_users_by_realm(&self, _realm_id: Uuid) -> Result<Vec<User>, AuthencError> {
        // TODO: Implement when operations::users is available in authenc-storage
        Err(AuthencError::database(
            "User operations not yet available in storage crate",
        ))
    }
}

/// Implementation of UserStoreTrait for `Arc<UserStore>` to enable direct trait method calls on Arc-wrapped instances
#[async_trait]
impl UserStoreTrait for Arc<UserStore> {
    async fn get_user(&self, user_id: Uuid) -> Result<Option<User>, AuthencError> {
        self.as_ref().get_user(user_id).await
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, AuthencError> {
        self.as_ref().get_user_by_username(username).await
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AuthencError> {
        self.as_ref().get_user_by_email(email).await
    }

    async fn add_user(&self, request: CreateUserRequest) -> Result<User, AuthencError> {
        self.as_ref().add_user(request).await
    }

    async fn update_user(
        &self,
        user_id: Uuid,
        request: UpdateUserRequest,
    ) -> Result<User, AuthencError> {
        self.as_ref().update_user(user_id, request).await
    }

    async fn delete_user(&self, user_id: Uuid) -> Result<(), AuthencError> {
        self.as_ref().delete_user(user_id).await
    }

    async fn get_all(&self) -> Result<Vec<User>, AuthencError> {
        self.as_ref().get_all().await
    }

    async fn get_users_by_realm(&self, realm_id: Uuid) -> Result<Vec<User>, AuthencError> {
        self.as_ref().get_users_by_realm(realm_id).await
    }
}
