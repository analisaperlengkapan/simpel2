use crate::error::AppError;
use crate::models::{Permission, Role, UserRole};
use sqlx::PgPool;
use tracing::{error, info};
use uuid::Uuid;

pub struct RbacService {
    pool: PgPool,
}

impl RbacService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn assign_role_to_user(&self, user_id: Uuid, role_id: Uuid) -> Result<(), AppError> {
        let user_role_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        sqlx::query!(
            "INSERT INTO keamanan.user_roles (id, user_id, role_id, created_at) 
             VALUES ($1, $2, $3, $4) 
             ON CONFLICT (user_id, role_id) DO NOTHING",
            user_role_id,
            user_id,
            role_id,
            now
        )
        .execute(&self.pool)
        .await?;

        info!("Role assigned to user: user_id={}, role_id={}", user_id, role_id);
        Ok(())
    }

    pub async fn remove_role_from_user(&self, user_id: Uuid, role_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM keamanan.user_roles WHERE user_id = $1 AND role_id = $2",
            user_id,
            role_id
        )
        .execute(&self.pool)
        .await?;

        info!("Role removed from user: user_id={}, role_id={}", user_id, role_id);
        Ok(())
    }

    pub async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<Role>, AppError> {
        let roles = sqlx::query_as!(
            Role,
            "SELECT r.* FROM keamanan.roles r 
             JOIN keamanan.user_roles ur ON r.id = ur.role_id 
             WHERE ur.user_id = $1",
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(roles)
    }

    pub async fn get_user_permissions(&self, user_id: Uuid) -> Result<Vec<String>, AppError> {
        let permissions = sqlx::query!(
            "SELECT DISTINCT p.name FROM keamanan.permissions p 
             JOIN keamanan.role_permissions rp ON p.id = rp.permission_id 
             JOIN keamanan.user_roles ur ON rp.role_id = ur.role_id 
             WHERE ur.user_id = $1",
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(permissions.into_iter().map(|p| p.name).collect())
    }

    pub async fn check_permission(&self, user_id: Uuid, permission: &str) -> Result<bool, AppError> {
        let permissions = self.get_user_permissions(user_id).await?;
        
        Ok(permissions.contains(&permission.to_string()) || permissions.contains(&"*".to_string()))
    }

    pub async fn add_permission_to_role(&self, role_id: Uuid, permission_id: Uuid) -> Result<(), AppError> {
        let role_permission_id = Uuid::new_v4();

        sqlx::query!(
            "INSERT INTO keamanan.role_permissions (id, role_id, permission_id) 
             VALUES ($1, $2, $3) 
             ON CONFLICT (role_id, permission_id) DO NOTHING",
            role_permission_id,
            role_id,
            permission_id
        )
        .execute(&self.pool)
        .await?;

        info!("Permission added to role: role_id={}, permission_id={}", role_id, permission_id);
        Ok(())
    }

    pub async fn remove_permission_from_role(&self, role_id: Uuid, permission_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM keamanan.role_permissions WHERE role_id = $1 AND permission_id = $2",
            role_id,
            permission_id
        )
        .execute(&self.pool)
        .await?;

        info!("Permission removed from role: role_id={}, permission_id={}", role_id, permission_id);
        Ok(())
    }

    pub async fn create_permission(&self, name: &str, resource: &str, action: &str) -> Result<Permission, AppError> {
        let permission_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let permission = sqlx::query_as!(
            Permission,
            "INSERT INTO keamanan.permissions (id, name, resource, action, created_at) 
             VALUES ($1, $2, $3, $4, $5) 
             RETURNING *",
            permission_id,
            name,
            resource,
            action,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        info!("Permission created: {}", name);
        Ok(permission)
    }

    pub async fn get_all_permissions(&self) -> Result<Vec<Permission>, AppError> {
        let permissions = sqlx::query_as!(
            Permission,
            "SELECT * FROM keamanan.permissions ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(permissions)
    }

    pub async fn get_role_permissions(&self, role_id: Uuid) -> Result<Vec<Permission>, AppError> {
        let permissions = sqlx::query_as!(
            Permission,
            "SELECT p.* FROM keamanan.permissions p 
             JOIN keamanan.role_permissions rp ON p.id = rp.permission_id 
             WHERE rp.role_id = $1",
            role_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(permissions)
    }

    pub async fn validate_access(&self, user_id: Uuid, resource: &str, action: &str) -> Result<bool, AppError> {
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