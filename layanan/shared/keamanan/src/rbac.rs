use crate::error::AppError;
use crate::models::{Permission, Role, UserRole};
use deadpool_postgres::Pool;
use tracing::{error, info};
use uuid::Uuid;

pub struct RbacService {
    pool: Pool,
}

impl RbacService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn assign_role_to_user(&self, user_id: Uuid, role_id: Uuid) -> Result<(), AppError> {
        let user_role_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let client = self.pool.get().await?;
        client
            .execute(
                "INSERT INTO keamanan.user_roles (id, user_id, role_id, created_at)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (user_id, role_id) DO NOTHING",
                &[&user_role_id, &user_id, &role_id, &now],
            )
            .await?;

        info!(
            "Role assigned to user: user_id={}, role_id={}",
            user_id, role_id
        );
        Ok(())
    }

    pub async fn remove_role_from_user(
        &self,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), AppError> {
        let client = self.pool.get().await?;
        client
            .execute(
                "DELETE FROM keamanan.user_roles WHERE user_id = $1 AND role_id = $2",
                &[&user_id, &role_id],
            )
            .await?;

        info!(
            "Role removed from user: user_id={}, role_id={}",
            user_id, role_id
        );
        Ok(())
    }

    pub async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<Role>, AppError> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT r.id, r.name, r.description, r.created_at, r.updated_at FROM keamanan.roles r
                 JOIN keamanan.user_roles ur ON r.id = ur.role_id
                 WHERE ur.user_id = $1",
                &[&user_id]
            )
            .await?;

        let roles = rows.into_iter().map(|row| Role::from(&row)).collect();
        Ok(roles)
    }

    pub async fn get_user_permissions(&self, user_id: Uuid) -> Result<Vec<String>, AppError> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT DISTINCT p.name FROM keamanan.permissions p
                 JOIN keamanan.role_permissions rp ON p.id = rp.permission_id
                 JOIN keamanan.user_roles ur ON rp.role_id = ur.role_id
                 WHERE ur.user_id = $1",
                &[&user_id],
            )
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| row.get::<_, String>("name"))
            .collect())
    }

    pub async fn check_permission(
        &self,
        user_id: Uuid,
        permission: &str,
    ) -> Result<bool, AppError> {
        let permissions = self.get_user_permissions(user_id).await?;

        Ok(permissions.contains(&permission.to_string()) || permissions.contains(&"*".to_string()))
    }

    pub async fn add_permission_to_role(
        &self,
        role_id: Uuid,
        permission_id: Uuid,
    ) -> Result<(), AppError> {
        let role_permission_id = Uuid::new_v4();

        let client = self.pool.get().await?;
        client
            .execute(
                "INSERT INTO keamanan.role_permissions (id, role_id, permission_id)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (role_id, permission_id) DO NOTHING",
                &[&role_permission_id, &role_id, &permission_id],
            )
            .await?;

        info!(
            "Permission added to role: role_id={}, permission_id={}",
            role_id, permission_id
        );
        Ok(())
    }

    pub async fn remove_permission_from_role(
        &self,
        role_id: Uuid,
        permission_id: Uuid,
    ) -> Result<(), AppError> {
        let client = self.pool.get().await?;
        client
            .execute(
                "DELETE FROM keamanan.role_permissions WHERE role_id = $1 AND permission_id = $2",
                &[&role_id, &permission_id],
            )
            .await?;

        info!(
            "Permission removed from role: role_id={}, permission_id={}",
            role_id, permission_id
        );
        Ok(())
    }

    pub async fn create_permission(
        &self,
        name: &str,
        resource: &str,
        action: &str,
    ) -> Result<Permission, AppError> {
        let permission_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let client = self.pool.get().await?;
        let rows = client
            .query(
                "INSERT INTO keamanan.permissions (id, name, resource, action, created_at)
                 VALUES ($1, $2, $3, $4, $5)
                 RETURNING id, name, resource, action, created_at, updated_at",
                &[&permission_id, &name, &resource, &action, &now],
            )
            .await?;

        let permission = Permission::from(rows.first().unwrap());
        info!("Permission created: {}", name);
        Ok(permission)
    }

    pub async fn get_all_permissions(&self) -> Result<Vec<Permission>, AppError> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT id, name, resource, action, created_at, updated_at FROM keamanan.permissions ORDER BY name",
                &[]
            )
            .await?;

        let permissions = rows.into_iter().map(|row| Permission::from(&row)).collect();
        Ok(permissions)
    }

    pub async fn get_role_permissions(&self, role_id: Uuid) -> Result<Vec<Permission>, AppError> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT p.id, p.name, p.resource, p.action, p.created_at, p.updated_at FROM keamanan.permissions p
                 JOIN keamanan.role_permissions rp ON p.id = rp.permission_id
                 WHERE rp.role_id = $1",
                &[&role_id]
            )
            .await?;

        let permissions = rows.into_iter().map(|row| Permission::from(&row)).collect();
        Ok(permissions)
    }

    pub async fn validate_access(
        &self,
        user_id: Uuid,
        resource: &str,
        action: &str,
    ) -> Result<bool, AppError> {
        let permissions = self.get_user_permissions(user_id).await?;

        // Check for wildcard permission
        if permissions.contains(&"*".to_string()) {
            return Ok(true);
        }

        // Check for specific resource:action permission
        let specific_permission = format!("{}:{}", resource, action);
        if permissions.contains(&specific_permission) {
            return Ok(true);
        }

        // Check for resource:* permission
        let resource_wildcard = format!("{}:*", resource);
        if permissions.contains(&resource_wildcard) {
            return Ok(true);
        }

        Ok(false)
    }
}
