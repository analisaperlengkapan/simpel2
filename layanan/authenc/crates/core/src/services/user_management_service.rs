//! User management service implementation
//!
//! This module implements user management operations, including:
//! - User creation with validation
//! - User profile updates
//! - User deletion (soft delete)
//! - User search and filtering
//! - Email verification workflow

use std::sync::Arc;
use tracing::{debug, info, warn};

use authenc_types::{
    domain::*,
    domain_types::{RealmId, UserId},
    error::AuthencError,
    result::Result,
    traits::*,
};
use uuid::Uuid;

/// Implementation of the user management service
///
/// This service provides CRUD operations for users with:
/// - Input validation (email format, password strength, username uniqueness)
/// - Password hashing on creation/update
/// - Soft delete (sets enabled=false instead of removing from database)
/// - Email verification workflow
pub struct UserManagementServiceImpl {
    /// User storage for CRUD operations
    user_store: Arc<dyn UserStore>,
    /// Password hasher for hashing passwords
    password_hasher: Arc<dyn PasswordHasher>,
}

impl UserManagementServiceImpl {
    /// Create a new user management service
    ///
    /// # Arguments
    ///
    /// * `user_store` - User storage implementation
    /// * `password_hasher` - Password hasher implementation
    pub fn new(user_store: Arc<dyn UserStore>, password_hasher: Arc<dyn PasswordHasher>) -> Self {
        Self {
            user_store,
            password_hasher,
        }
    }

    /// Validate email format
    ///
    /// # Arguments
    ///
    /// * `email` - Email address to validate
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err` if invalid
    fn validate_email(&self, email: &str) -> Result<()> {
        // Basic email validation: contains @ and has characters before and after
        if !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
            return Err(AuthencError::ValidationError(
                "Invalid email format".to_string(),
            ));
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(AuthencError::ValidationError(
                "Invalid email format".to_string(),
            ));
        }

        // Check domain has at least one dot
        if !parts[1].contains('.') {
            return Err(AuthencError::ValidationError(
                "Invalid email domain".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate password strength
    ///
    /// Requirements:
    /// - Minimum 8 characters
    /// - At least one uppercase letter
    /// - At least one lowercase letter
    /// - At least one digit
    ///
    /// # Arguments
    ///
    /// * `password` - Password to validate
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err` if invalid
    fn validate_password(&self, password: &str) -> Result<()> {
        if password.len() < 8 {
            return Err(AuthencError::ValidationError(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        if !password.chars().any(|c| c.is_uppercase()) {
            return Err(AuthencError::ValidationError(
                "Password must contain at least one uppercase letter".to_string(),
            ));
        }

        if !password.chars().any(|c| c.is_lowercase()) {
            return Err(AuthencError::ValidationError(
                "Password must contain at least one lowercase letter".to_string(),
            ));
        }

        if !password.chars().any(|c| c.is_numeric()) {
            return Err(AuthencError::ValidationError(
                "Password must contain at least one digit".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate username format
    ///
    /// Requirements:
    /// - Minimum 3 characters
    /// - Maximum 32 characters
    /// - Only alphanumeric, underscore, and hyphen
    ///
    /// # Arguments
    ///
    /// * `username` - Username to validate
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err` if invalid
    fn validate_username(&self, username: &str) -> Result<()> {
        if username.len() < 3 {
            return Err(AuthencError::ValidationError(
                "Username must be at least 3 characters".to_string(),
            ));
        }

        if username.len() > 32 {
            return Err(AuthencError::ValidationError(
                "Username must be at most 32 characters".to_string(),
            ));
        }

        if !username
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(AuthencError::ValidationError(
                "Username can only contain alphanumeric characters, underscore, and hyphen"
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Create a new user
    ///
    /// This method:
    /// 1. Validates username format
    /// 2. Validates email format
    /// 3. Validates password strength
    /// 4. Checks username uniqueness
    /// 5. Checks email uniqueness
    /// 6. Hashes password
    /// 7. Creates user in database
    ///
    /// # Arguments
    ///
    /// * `request` - User creation request
    ///
    /// # Returns
    ///
    /// The created user
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails or username/email already exists
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User> {
        debug!(
            username = %request.username,
            email = %request.email,
            realm_id = ?request.realm_id,
            "Creating user"
        );

        // Step 1: Validate username
        self.validate_username(&request.username)?;

        // Step 2: Validate email
        self.validate_email(&request.email)?;

        // Step 3: Validate password (if provided)
        if let Some(ref password) = request.password {
            self.validate_password(password)?;
        }

        // Resolve realm_id to RealmId (default to new if None)
        let realm_id = request
            .realm_id
            .map(RealmId::from_uuid)
            .unwrap_or_else(RealmId::new);

        // Step 4: Check username uniqueness
        if self
            .user_store
            .username_exists(&request.username, realm_id)
            .await?
        {
            warn!(
                username = %request.username,
                realm_id = ?request.realm_id,
                "Username already exists"
            );
            return Err(AuthencError::UsernameAlreadyExists(request.username));
        }

        // Step 5: Check email uniqueness
        if self
            .user_store
            .email_exists(&request.email, realm_id)
            .await?
        {
            warn!(
                email = %request.email,
                realm_id = ?request.realm_id,
                "Email already exists"
            );
            return Err(AuthencError::EmailAlreadyExists(request.email));
        }

        // Step 6: Hash password if provided
        let password_hash = if let Some(ref password) = request.password {
            Some(self.password_hasher.hash(password)?)
        } else {
            None
        };

        // Step 7: Create user in database
        let create_request = CreateUserRequest {
            username: request.username.clone(),
            email: request.email.clone(),
            password: password_hash,
            realm_id: request.realm_id,
            satker_code: request.satker_code.clone(),
            first_name: request.first_name.clone(),
            last_name: request.last_name.clone(),
            nip: request.nip.clone(),
            nama: request.nama.clone(),
            jabatan: request.jabatan.clone(),
            phone_number: request.phone_number.clone(),
            organization_id: request.organization_id,
            roles: request.roles.clone(),
            attributes: request.attributes.clone(),
        };

        let user = self.user_store.create_user(create_request).await?;

        info!(
            user_id = %user.id,
            username = %user.username,
            email = %user.email,
            "User created successfully"
        );

        Ok(user)
    }

    /// Get a user by ID
    ///
    /// # Arguments
    ///
    /// * `user_id` - User ID
    ///
    /// # Returns
    ///
    /// The user
    pub async fn get_user(&self, user_id: UserId) -> Result<User> {
        debug!(user_id = %user_id, "Getting user");
        self.user_store.get_user(user_id).await
    }

    /// Get a user by username
    ///
    /// # Arguments
    ///
    /// * `username` - Username
    /// * `realm_id` - Realm ID
    ///
    /// # Returns
    ///
    /// The user
    pub async fn get_user_by_username(&self, username: &str, realm_id: RealmId) -> Result<User> {
        debug!(username = %username, realm_id = %realm_id, "Getting user by username");
        self.user_store
            .get_user_by_username(username, realm_id)
            .await
    }

    /// Get a user by email
    ///
    /// # Arguments
    ///
    /// * `email` - Email address
    /// * `realm_id` - Realm ID
    ///
    /// # Returns
    ///
    /// The user
    pub async fn get_user_by_email(&self, email: &str, realm_id: RealmId) -> Result<User> {
        debug!(email = %email, realm_id = %realm_id, "Getting user by email");
        self.user_store.get_user_by_email(email, realm_id).await
    }

    /// Update a user
    ///
    /// This method:
    /// 1. Validates new email format (if provided)
    /// 2. Validates new password strength (if provided)
    /// 3. Checks new email uniqueness (if provided)
    /// 4. Hashes new password (if provided)
    /// 5. Updates user in database
    ///
    /// # Arguments
    ///
    /// * `user_id` - User ID
    /// * `request` - User update request
    ///
    /// # Returns
    ///
    /// The updated user
    pub async fn update_user(
        &self,
        user_id: UserId,
        mut request: UpdateUserRequest,
    ) -> Result<User> {
        debug!(user_id = %user_id, "Updating user");

        // Get existing user to check realm
        let existing_user = self.user_store.get_user(user_id).await?;

        // Resolve realm_id for uniqueness checks
        let existing_realm_id = existing_user
            .realm_id
            .map(RealmId::from_uuid)
            .unwrap_or_else(RealmId::new);

        // Step 1: Validate new email if provided
        if let Some(ref email) = request.email {
            self.validate_email(email)?;

            // Check email uniqueness (excluding current user)
            if self
                .user_store
                .email_exists(email, existing_realm_id)
                .await?
            {
                let existing_email_user = self
                    .user_store
                    .get_user_by_email(email, existing_realm_id)
                    .await?;
                if existing_email_user.id != *user_id.as_uuid() {
                    warn!(
                        email = %email,
                        realm_id = ?existing_user.realm_id,
                        "Email already exists"
                    );
                    return Err(AuthencError::EmailAlreadyExists(email.clone()));
                }
            }
        }

        // Step 2: Validate and hash new password if provided
        if let Some(ref password) = request.password {
            self.validate_password(password)?;
            let password_hash = self.password_hasher.hash(password)?;
            request.password = Some(password_hash);
        }

        // Step 3: Update user in database
        let user = self.user_store.update_user(user_id, request).await?;

        info!(
            user_id = %user.id,
            username = %user.username,
            "User updated successfully"
        );

        Ok(user)
    }

    /// Delete a user (soft delete)
    ///
    /// This method performs a soft delete by setting enabled=false
    /// instead of removing the user from the database.
    ///
    /// # Arguments
    ///
    /// * `user_id` - User ID
    pub async fn delete_user(&self, user_id: UserId) -> Result<()> {
        debug!(user_id = %user_id, "Deleting user (soft delete)");

        // Soft delete: set enabled=false
        let update_request = UpdateUserRequest {
            username: None,
            email: None,
            satker_code: None,
            first_name: None,
            last_name: None,
            nip: None,
            nama: None,
            jabatan: None,
            phone_number: None,
            phone_verified: None,
            require_password_change: None,
            password: None,
            enabled: Some(false),
            email_verified: None,
            mfa_enabled: None,
            attributes: None,
        };

        self.user_store.update_user(user_id, update_request).await?;

        info!(user_id = %user_id, "User deleted successfully (soft delete)");

        Ok(())
    }

    /// List users in a realm with pagination
    ///
    /// # Arguments
    ///
    /// * `realm_id` - Realm ID
    /// * `offset` - Offset for pagination (0-based)
    /// * `limit` - Maximum number of users to return
    ///
    /// # Returns
    ///
    /// List of users
    pub async fn list_users(
        &self,
        realm_id: RealmId,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<User>> {
        debug!(
            realm_id = %realm_id,
            offset = offset,
            limit = limit,
            "Listing users"
        );

        self.user_store.list_users(realm_id, offset, limit).await
    }

    /// Search users by username or email
    ///
    /// This method searches for users whose username or email contains the query string.
    ///
    /// # Arguments
    ///
    /// * `realm_id` - Realm ID
    /// * `query` - Search query
    /// * `limit` - Maximum number of results
    ///
    /// # Returns
    ///
    /// List of matching users
    pub async fn search_users(
        &self,
        realm_id: RealmId,
        query: &str,
        limit: usize,
    ) -> Result<Vec<User>> {
        debug!(
            realm_id = %realm_id,
            query = %query,
            limit = limit,
            "Searching users"
        );

        self.user_store
            .search_users(realm_id, query, 0, limit)
            .await
    }

    /// Count total users in a realm (for pagination)
    pub async fn count_users(&self, realm_id: RealmId) -> Result<i64> {
        self.user_store.count_users(realm_id).await
    }

    /// Count enabled (active) users in a realm
    pub async fn count_enabled_users(&self, realm_id: RealmId) -> Result<i64> {
        self.user_store.count_enabled_users(realm_id).await
    }

    /// List users with optional enabled filter pushed to DB
    pub async fn list_users_filtered(
        &self,
        realm_id: RealmId,
        enabled: Option<bool>,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<User>> {
        self.user_store
            .list_users_filtered(realm_id, enabled, offset, limit)
            .await
    }

    /// Count users with optional enabled filter pushed to DB
    pub async fn count_users_filtered(
        &self,
        realm_id: RealmId,
        enabled: Option<bool>,
    ) -> Result<i64> {
        self.user_store
            .count_users_filtered(realm_id, enabled)
            .await
    }

    /// Search users with pagination and return count
    pub async fn search_users_paginated(
        &self,
        realm_id: RealmId,
        query: &str,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<User>, i64)> {
        let total = self.user_store.count_search_users(realm_id, query).await?;
        let users = self
            .user_store
            .search_users(realm_id, query, offset, limit)
            .await?;
        Ok((users, total))
    }

    /// Verify user email
    ///
    /// This method marks a user's email as verified.
    ///
    /// # Arguments
    ///
    /// * `user_id` - User ID
    pub async fn verify_email(&self, user_id: UserId) -> Result<User> {
        debug!(user_id = %user_id, "Verifying user email");

        let update_request = UpdateUserRequest {
            username: None,
            email: None,
            satker_code: None,
            first_name: None,
            last_name: None,
            nip: None,
            nama: None,
            jabatan: None,
            phone_number: None,
            phone_verified: None,
            require_password_change: None,
            password: None,
            enabled: None,
            email_verified: Some(true),
            mfa_enabled: None,
            attributes: None,
        };

        let user = self.user_store.update_user(user_id, update_request).await?;

        info!(
            user_id = %user.id,
            username = %user.username,
            "Email verified successfully"
        );

        Ok(user)
    }

    /// Enable MFA for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - User ID
    pub async fn enable_mfa(&self, user_id: UserId) -> Result<User> {
        debug!(user_id = %user_id, "Enabling MFA");

        let update_request = UpdateUserRequest {
            username: None,
            email: None,
            satker_code: None,
            first_name: None,
            last_name: None,
            nip: None,
            nama: None,
            jabatan: None,
            phone_number: None,
            phone_verified: None,
            require_password_change: None,
            password: None,
            enabled: None,
            email_verified: None,
            mfa_enabled: Some(true),
            attributes: None,
        };

        let user = self.user_store.update_user(user_id, update_request).await?;

        info!(
            user_id = %user.id,
            username = %user.username,
            "MFA enabled successfully"
        );

        Ok(user)
    }

    /// Disable MFA for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - User ID
    pub async fn disable_mfa(&self, user_id: UserId) -> Result<User> {
        debug!(user_id = %user_id, "Disabling MFA");

        let update_request = UpdateUserRequest {
            username: None,
            email: None,
            satker_code: None,
            first_name: None,
            last_name: None,
            nip: None,
            nama: None,
            jabatan: None,
            phone_number: None,
            phone_verified: None,
            require_password_change: None,
            password: None,
            enabled: None,
            email_verified: None,
            mfa_enabled: Some(false),
            attributes: None,
        };

        let user = self.user_store.update_user(user_id, update_request).await?;

        info!(
            user_id = %user.id,
            username = %user.username,
            "MFA disabled successfully"
        );

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use tokio::sync::Mutex;

    // Mock implementations for testing

    struct MockUserStore {
        users: Arc<Mutex<HashMap<Uuid, User>>>,
        users_by_username: Arc<Mutex<HashMap<(String, RealmId), Uuid>>>,
        users_by_email: Arc<Mutex<HashMap<(String, RealmId), Uuid>>>,
    }

    impl MockUserStore {
        fn new() -> Self {
            Self {
                users: Arc::new(Mutex::new(HashMap::new())),
                users_by_username: Arc::new(Mutex::new(HashMap::new())),
                users_by_email: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl UserStore for MockUserStore {
        async fn get_user(&self, id: UserId) -> Result<User> {
            let users = self.users.lock().await;
            users
                .get(id.as_uuid())
                .cloned()
                .ok_or_else(|| AuthencError::UserNotFound(format!("User {} not found", id)))
        }

        async fn get_user_by_username(&self, username: &str, realm_id: RealmId) -> Result<User> {
            let users_by_username = self.users_by_username.lock().await;
            let user_uuid = users_by_username
                .get(&(username.to_string(), realm_id))
                .ok_or_else(|| AuthencError::UserNotFound(format!("User {} not found", username)))?
                .clone();

            let users = self.users.lock().await;
            users
                .get(&user_uuid)
                .cloned()
                .ok_or_else(|| AuthencError::UserNotFound(format!("User {} not found", username)))
        }

        async fn get_user_by_email(&self, email: &str, realm_id: RealmId) -> Result<User> {
            let users_by_email = self.users_by_email.lock().await;
            let user_uuid = users_by_email
                .get(&(email.to_string(), realm_id))
                .ok_or_else(|| {
                    AuthencError::UserNotFound(format!("User with email {} not found", email))
                })?
                .clone();

            let users = self.users.lock().await;
            users.get(&user_uuid).cloned().ok_or_else(|| {
                AuthencError::UserNotFound(format!("User with email {} not found", email))
            })
        }

        async fn create_user(&self, req: CreateUserRequest) -> Result<User> {
            let user = User::new(
                req.username.clone(),
                req.email.clone(),
                req.satker_code.clone(),
                req.password, // Already hashed by service
                req.realm_id,
            );

            let realm_id = user
                .realm_id
                .map(RealmId::from_uuid)
                .unwrap_or_else(RealmId::new);

            let mut users = self.users.lock().await;
            users.insert(user.id, user.clone());

            let mut users_by_username = self.users_by_username.lock().await;
            users_by_username.insert((user.username.clone(), realm_id), user.id);

            let mut users_by_email = self.users_by_email.lock().await;
            users_by_email.insert((user.email.clone(), realm_id), user.id);

            Ok(user)
        }

        async fn update_user(&self, id: UserId, req: UpdateUserRequest) -> Result<User> {
            let mut users = self.users.lock().await;
            let user = users
                .get_mut(id.as_uuid())
                .ok_or_else(|| AuthencError::UserNotFound(format!("User {} not found", id)))?;

            if let Some(email) = req.email {
                user.email = email;
            }
            if let Some(password) = req.password {
                user.password_hash = Some(password);
            }
            if let Some(enabled) = req.enabled {
                user.enabled = enabled;
            }
            if let Some(email_verified) = req.email_verified {
                user.email_verified = email_verified;
            }
            if let Some(mfa_enabled) = req.mfa_enabled {
                user.mfa_enabled = mfa_enabled;
            }
            if let Some(require_password_change) = req.require_password_change {
                user.require_password_change = require_password_change;
            }
            if let Some(first_name) = req.first_name {
                user.first_name = Some(first_name);
            }
            if let Some(last_name) = req.last_name {
                user.last_name = Some(last_name);
            }
            if let Some(phone_number) = req.phone_number {
                user.phone_number = Some(phone_number);
            }
            if let Some(username) = req.username {
                user.username = username;
            }
            if let Some(satker_code) = req.satker_code {
                user.satker_code = satker_code;
            }
            if let Some(nip) = req.nip {
                user.nip = Some(nip);
            }
            if let Some(nama) = req.nama {
                user.nama = Some(nama);
            }
            if let Some(jabatan) = req.jabatan {
                user.jabatan = Some(jabatan);
            }
            if let Some(phone_verified) = req.phone_verified {
                user.phone_verified = phone_verified;
            }
            if let Some(attributes) = req.attributes {
                user.attributes = Some(attributes);
            }

            user.updated_at = chrono::Utc::now();

            Ok(user.clone())
        }

        async fn delete_user(&self, id: UserId) -> Result<()> {
            let mut users = self.users.lock().await;
            users.remove(id.as_uuid());
            Ok(())
        }

        async fn list_users(
            &self,
            realm_id: RealmId,
            offset: usize,
            limit: usize,
        ) -> Result<Vec<User>> {
            let users = self.users.lock().await;
            let mut realm_users: Vec<User> = users
                .values()
                .filter(|u| u.realm_id == Some(*realm_id.as_uuid()))
                .cloned()
                .collect();

            realm_users.sort_by(|a, b| a.created_at.cmp(&b.created_at));

            Ok(realm_users.into_iter().skip(offset).take(limit).collect())
        }

        async fn username_exists(&self, username: &str, realm_id: RealmId) -> Result<bool> {
            let users_by_username = self.users_by_username.lock().await;
            Ok(users_by_username.contains_key(&(username.to_string(), realm_id)))
        }

        async fn email_exists(&self, email: &str, realm_id: RealmId) -> Result<bool> {
            let users_by_email = self.users_by_email.lock().await;
            Ok(users_by_email.contains_key(&(email.to_string(), realm_id)))
        }

        async fn count_users(&self, realm_id: RealmId) -> Result<i64> {
            let users = self.users.lock().await;
            Ok(users.values().filter(|u| u.realm_id == Some(*realm_id.as_uuid())).count() as i64)
        }

        async fn count_enabled_users(&self, realm_id: RealmId) -> Result<i64> {
            let users = self.users.lock().await;
            Ok(users.values().filter(|u| u.realm_id == Some(*realm_id.as_uuid()) && u.enabled).count() as i64)
        }

        async fn list_users_filtered(&self, realm_id: RealmId, enabled: Option<bool>, offset: usize, limit: usize) -> Result<Vec<User>> {
            let users = self.users.lock().await;
            let mut realm_users: Vec<User> = users
                .values()
                .filter(|u| u.realm_id == Some(*realm_id.as_uuid()) && enabled.map_or(true, |e| u.enabled == e))
                .cloned()
                .collect();
            realm_users.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            Ok(realm_users.into_iter().skip(offset).take(limit).collect())
        }

        async fn count_users_filtered(&self, realm_id: RealmId, enabled: Option<bool>) -> Result<i64> {
            let users = self.users.lock().await;
            Ok(users.values().filter(|u| u.realm_id == Some(*realm_id.as_uuid()) && enabled.map_or(true, |e| u.enabled == e)).count() as i64)
        }

        async fn search_users(
            &self,
            realm_id: RealmId,
            query: &str,
            offset: usize,
            limit: usize,
        ) -> Result<Vec<User>> {
            let users = self.users.lock().await;
            let query_lower = query.to_lowercase();
            let mut matched_users: Vec<User> = users
                .values()
                .filter(|u| u.realm_id == Some(*realm_id.as_uuid()))
                .filter(|u| {
                    u.username.to_lowercase().contains(&query_lower) ||
                    u.email.to_lowercase().contains(&query_lower) ||
                    u.nip.as_ref().map(|n| n.to_lowercase().contains(&query_lower)).unwrap_or(false) ||
                    u.nama.as_ref().map(|n| n.to_lowercase().contains(&query_lower)).unwrap_or(false)
                })
                .cloned()
                .collect();

            matched_users.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            Ok(matched_users.into_iter().skip(offset).take(limit).collect())
        }

        async fn count_search_users(&self, realm_id: RealmId, query: &str) -> Result<i64> {
            let users = self.users.lock().await;
            let query_lower = query.to_lowercase();
            Ok(users
                .values()
                .filter(|u| u.realm_id == Some(*realm_id.as_uuid()))
                .filter(|u| {
                    u.username.to_lowercase().contains(&query_lower) ||
                    u.email.to_lowercase().contains(&query_lower) ||
                    u.nip.as_ref().map(|n| n.to_lowercase().contains(&query_lower)).unwrap_or(false) ||
                    u.nama.as_ref().map(|n| n.to_lowercase().contains(&query_lower)).unwrap_or(false)
                })
                .count() as i64)
        }
    }

    struct MockPasswordHasher;

    impl PasswordHasher for MockPasswordHasher {
        fn hash(&self, password: &str) -> Result<String> {
            Ok(format!("hashed_{}", password))
        }

        fn verify(&self, password: &str, hash: &str) -> Result<bool> {
            Ok(hash == format!("hashed_{}", password))
        }
    }

    fn make_create_request(
        username: &str,
        email: &str,
        password: &str,
        realm_uuid: Uuid,
    ) -> CreateUserRequest {
        CreateUserRequest {
            username: username.to_string(),
            email: email.to_string(),
            password: Some(password.to_string()),
            realm_id: Some(realm_uuid),
            satker_code: String::new(),
            first_name: None,
            last_name: None,
            nip: None,
            nama: None,
            jabatan: None,
            phone_number: None,
            organization_id: None,
            roles: None,
            attributes: None,
        }
    }

    fn make_update_request_email(email: &str) -> UpdateUserRequest {
        UpdateUserRequest {
            username: None,
            email: Some(email.to_string()),
            satker_code: None,
            first_name: None,
            last_name: None,
            nip: None,
            nama: None,
            jabatan: None,
            phone_number: None,
            phone_verified: None,
            require_password_change: None,
            password: None,
            enabled: None,
            email_verified: None,
            mfa_enabled: None,
            attributes: None,
        }
    }

    fn make_update_request_password(password: &str) -> UpdateUserRequest {
        UpdateUserRequest {
            username: None,
            email: None,
            satker_code: None,
            first_name: None,
            last_name: None,
            nip: None,
            nama: None,
            jabatan: None,
            phone_number: None,
            phone_verified: None,
            require_password_change: None,
            password: Some(password.to_string()),
            enabled: None,
            email_verified: None,
            mfa_enabled: None,
            attributes: None,
        }
    }

    #[tokio::test]
    async fn test_create_user_success() {
        let user_store = Arc::new(MockUserStore::new());
        let password_hasher = Arc::new(MockPasswordHasher);
        let service = UserManagementServiceImpl::new(user_store, password_hasher);

        let realm_id = RealmId::new();
        let request = make_create_request(
            "testuser",
            "test@example.com",
            "Password123",
            *realm_id.as_uuid(),
        );

        let user = service.create_user(request).await.unwrap();

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.password_hash, Some("hashed_Password123".to_string()));
        assert!(user.enabled);
        assert!(!user.email_verified);
        assert!(!user.mfa_enabled);
    }

    #[tokio::test]
    async fn test_create_user_invalid_email() {
        let user_store = Arc::new(MockUserStore::new());
        let password_hasher = Arc::new(MockPasswordHasher);
        let service = UserManagementServiceImpl::new(user_store, password_hasher);

        let realm_id = RealmId::new();
        let request = make_create_request(
            "testuser",
            "invalid-email",
            "Password123",
            *realm_id.as_uuid(),
        );

        let result = service.create_user(request).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthencError::ValidationError(_)
        ));
    }

    #[tokio::test]
    async fn test_create_user_weak_password() {
        let user_store = Arc::new(MockUserStore::new());
        let password_hasher = Arc::new(MockPasswordHasher);
        let service = UserManagementServiceImpl::new(user_store, password_hasher);

        let realm_id = RealmId::new();
        let request =
            make_create_request("testuser", "test@example.com", "weak", *realm_id.as_uuid());

        let result = service.create_user(request).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthencError::ValidationError(_)
        ));
    }

    #[tokio::test]
    async fn test_create_user_duplicate_username() {
        let user_store = Arc::new(MockUserStore::new());
        let password_hasher = Arc::new(MockPasswordHasher);
        let service = UserManagementServiceImpl::new(user_store.clone(), password_hasher);

        let realm_uuid = *RealmId::new().as_uuid();

        // Create first user
        service
            .create_user(make_create_request(
                "testuser",
                "test1@example.com",
                "Password123",
                realm_uuid,
            ))
            .await
            .unwrap();

        // Try to create second user with same username
        let result = service
            .create_user(make_create_request(
                "testuser",
                "test2@example.com",
                "Password123",
                realm_uuid,
            ))
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthencError::UsernameAlreadyExists(_)
        ));
    }

    #[tokio::test]
    async fn test_create_user_duplicate_email() {
        let user_store = Arc::new(MockUserStore::new());
        let password_hasher = Arc::new(MockPasswordHasher);
        let service = UserManagementServiceImpl::new(user_store.clone(), password_hasher);

        let realm_uuid = *RealmId::new().as_uuid();

        // Create first user
        service
            .create_user(make_create_request(
                "testuser1",
                "test@example.com",
                "Password123",
                realm_uuid,
            ))
            .await
            .unwrap();

        // Try to create second user with same email
        let result = service
            .create_user(make_create_request(
                "testuser2",
                "test@example.com",
                "Password123",
                realm_uuid,
            ))
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthencError::EmailAlreadyExists(_)
        ));
    }

    #[tokio::test]
    async fn test_update_user_email() {
        let user_store = Arc::new(MockUserStore::new());
        let password_hasher = Arc::new(MockPasswordHasher);
        let service = UserManagementServiceImpl::new(user_store.clone(), password_hasher);

        let realm_uuid = *RealmId::new().as_uuid();

        // Create user
        let user = service
            .create_user(make_create_request(
                "testuser",
                "old@example.com",
                "Password123",
                realm_uuid,
            ))
            .await
            .unwrap();

        // Update email
        let updated_user = service
            .update_user(
                UserId::from_uuid(user.id),
                make_update_request_email("new@example.com"),
            )
            .await
            .unwrap();
        assert_eq!(updated_user.email, "new@example.com");
    }

    #[tokio::test]
    async fn test_update_user_password() {
        let user_store = Arc::new(MockUserStore::new());
        let password_hasher = Arc::new(MockPasswordHasher);
        let service = UserManagementServiceImpl::new(user_store.clone(), password_hasher);

        let realm_uuid = *RealmId::new().as_uuid();

        // Create user
        let user = service
            .create_user(make_create_request(
                "testuser",
                "test@example.com",
                "OldPassword123",
                realm_uuid,
            ))
            .await
            .unwrap();

        // Update password
        let updated_user = service
            .update_user(
                UserId::from_uuid(user.id),
                make_update_request_password("NewPassword456"),
            )
            .await
            .unwrap();
        assert_eq!(
            updated_user.password_hash,
            Some("hashed_NewPassword456".to_string())
        );
    }

    #[tokio::test]
    async fn test_delete_user() {
        let user_store = Arc::new(MockUserStore::new());
        let password_hasher = Arc::new(MockPasswordHasher);
        let service = UserManagementServiceImpl::new(user_store.clone(), password_hasher);

        let realm_uuid = *RealmId::new().as_uuid();

        // Create user
        let user = service
            .create_user(make_create_request(
                "testuser",
                "test@example.com",
                "Password123",
                realm_uuid,
            ))
            .await
            .unwrap();

        // Delete user (soft delete)
        service
            .delete_user(UserId::from_uuid(user.id))
            .await
            .unwrap();

        // Verify user is disabled
        let deleted_user = service.get_user(UserId::from_uuid(user.id)).await.unwrap();
        assert!(!deleted_user.enabled);
    }

    #[tokio::test]
    async fn test_list_users() {
        let user_store = Arc::new(MockUserStore::new());
        let password_hasher = Arc::new(MockPasswordHasher);
        let service = UserManagementServiceImpl::new(user_store.clone(), password_hasher);

        let realm_id = RealmId::new();
        let realm_uuid = *realm_id.as_uuid();

        // Create multiple users
        for i in 1..=5 {
            let request = make_create_request(
                &format!("user{}", i),
                &format!("user{}@example.com", i),
                "Password123",
                realm_uuid,
            );
            service.create_user(request).await.unwrap();
        }

        // List users with pagination
        let users = service.list_users(realm_id, 0, 3).await.unwrap();
        assert_eq!(users.len(), 3);

        let users_page2 = service.list_users(realm_id, 3, 3).await.unwrap();
        assert_eq!(users_page2.len(), 2);
    }

    #[tokio::test]
    async fn test_search_users() {
        let user_store = Arc::new(MockUserStore::new());
        let password_hasher = Arc::new(MockPasswordHasher);
        let service = UserManagementServiceImpl::new(user_store.clone(), password_hasher);

        let realm_id = RealmId::new();
        let realm_uuid = *realm_id.as_uuid();

        // Create users
        service
            .create_user(make_create_request(
                "alice",
                "alice@example.com",
                "Password123",
                realm_uuid,
            ))
            .await
            .unwrap();
        service
            .create_user(make_create_request(
                "bob",
                "bob@example.com",
                "Password123",
                realm_uuid,
            ))
            .await
            .unwrap();

        // Search by username
        let results = service.search_users(realm_id, "ali", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].username, "alice");

        // Search by email
        let results = service.search_users(realm_id, "bob@", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].username, "bob");
    }

    #[tokio::test]
    async fn test_verify_email() {
        let user_store = Arc::new(MockUserStore::new());
        let password_hasher = Arc::new(MockPasswordHasher);
        let service = UserManagementServiceImpl::new(user_store.clone(), password_hasher);

        let realm_uuid = *RealmId::new().as_uuid();

        // Create user
        let user = service
            .create_user(make_create_request(
                "testuser",
                "test@example.com",
                "Password123",
                realm_uuid,
            ))
            .await
            .unwrap();
        assert!(!user.email_verified);

        // Verify email
        let verified_user = service
            .verify_email(UserId::from_uuid(user.id))
            .await
            .unwrap();
        assert!(verified_user.email_verified);
    }

    #[tokio::test]
    async fn test_enable_mfa() {
        let user_store = Arc::new(MockUserStore::new());
        let password_hasher = Arc::new(MockPasswordHasher);
        let service = UserManagementServiceImpl::new(user_store.clone(), password_hasher);

        let realm_uuid = *RealmId::new().as_uuid();

        // Create user
        let user = service
            .create_user(make_create_request(
                "testuser",
                "test@example.com",
                "Password123",
                realm_uuid,
            ))
            .await
            .unwrap();
        assert!(!user.mfa_enabled);

        // Enable MFA
        let mfa_user = service
            .enable_mfa(UserId::from_uuid(user.id))
            .await
            .unwrap();
        assert!(mfa_user.mfa_enabled);
    }

    #[tokio::test]
    async fn test_disable_mfa() {
        let user_store = Arc::new(MockUserStore::new());
        let password_hasher = Arc::new(MockPasswordHasher);
        let service = UserManagementServiceImpl::new(user_store.clone(), password_hasher);

        let realm_uuid = *RealmId::new().as_uuid();

        // Create user with MFA enabled
        let user = service
            .create_user(make_create_request(
                "testuser",
                "test@example.com",
                "Password123",
                realm_uuid,
            ))
            .await
            .unwrap();
        let mfa_user = service
            .enable_mfa(UserId::from_uuid(user.id))
            .await
            .unwrap();
        assert!(mfa_user.mfa_enabled);

        // Disable MFA
        let no_mfa_user = service
            .disable_mfa(UserId::from_uuid(user.id))
            .await
            .unwrap();
        assert!(!no_mfa_user.mfa_enabled);
    }
}
