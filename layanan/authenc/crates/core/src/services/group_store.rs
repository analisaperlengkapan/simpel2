//! Group Store — full CRUD backed by authenc_storage

use authenc_storage::Database;
use authenc_storage::operations::groups;
use authenc_types::domain::group::Group;
use authenc_types::Result;
use std::sync::Arc;
use uuid::Uuid;

pub struct GroupStore {
    db: Arc<Database>,
}

impl GroupStore {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Get a group by ID
    pub async fn get(&self, group_id: &Uuid) -> Result<Option<Group>> {
        groups::get_group_by_id(&self.db, *group_id).await
    }

    /// List all groups in a realm with pagination
    pub async fn list_by_realm(
        &self,
        realm_id: Uuid,
        first: Option<i64>,
        max: Option<i64>,
    ) -> Result<Vec<Group>> {
        groups::get_groups_by_realm(&self.db, realm_id, first, max).await
    }

    /// Get direct subgroups of a parent group
    pub async fn subgroups(&self, parent_id: Uuid) -> Result<Vec<Group>> {
        groups::get_subgroups(&self.db, parent_id, true).await
    }

    /// Create a new group
    pub async fn create(
        &self,
        realm_id: Uuid,
        name: &str,
        parent_id: Option<Uuid>,
        description: Option<&str>,
        attributes: &serde_json::Value,
    ) -> Result<Group> {
        groups::create_group(&self.db, realm_id, name, parent_id, description, attributes).await
    }

    /// Update a group
    pub async fn update(
        &self,
        group_id: Uuid,
        name: Option<String>,
        parent_id: Option<Option<Uuid>>,
        description: Option<Option<String>>,
        attributes: Option<serde_json::Value>,
    ) -> Result<Group> {
        groups::update_group(&self.db, group_id, name, parent_id, description, attributes).await
    }

    /// Delete a group
    pub async fn delete(&self, group_id: &Uuid) -> Result<()> {
        groups::delete_group(&self.db, *group_id).await
    }

    /// Add a user to a group
    pub async fn add_member(&self, group_id: Uuid, user_id: Uuid) -> Result<()> {
        groups::add_user_to_group(&self.db, user_id, group_id, None, None).await
    }

    /// Remove a user from a group
    pub async fn remove_member(&self, group_id: Uuid, user_id: Uuid) -> Result<()> {
        groups::remove_user_from_group(&self.db, user_id, group_id).await
    }

    /// Get all groups a user belongs to
    pub async fn user_groups(&self, user_id: Uuid) -> Result<Vec<Group>> {
        groups::get_user_groups(&self.db, user_id).await
    }

    /// Get members of a group (user IDs)
    pub async fn members(&self, group_id: Uuid, first: Option<i64>, max: Option<i64>) -> Result<Vec<Uuid>> {
        groups::get_group_members(&self.db, group_id, first, max).await
    }

    /// Count members in a group
    pub async fn member_count(&self, group_id: Uuid) -> Result<i64> {
        groups::count_group_members(&self.db, group_id).await
    }

    /// Count subgroups of a group
    pub async fn subgroup_count(&self, group_id: Uuid) -> Result<i64> {
        groups::count_subgroups(&self.db, group_id).await
    }
}
