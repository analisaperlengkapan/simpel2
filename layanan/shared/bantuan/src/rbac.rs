use crate::models::RbacPermission;
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use axum::{extract::{State, RequestPartsExt}, http::Request, middleware::Next, response::Response};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    User,
    Agent,
    Admin,
}

impl Role {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "user" => Some(Role::User),
            "agent" => Some(Role::Agent),
            "admin" => Some(Role::Admin),
            _ => None,
        }
    }
}

pub async fn has_permission(pool: &PgPool, role: &str, resource: &str, action: &str) -> Result<bool, AppError> {
    let perm = sqlx::query_as!(RbacPermission,
        r#"SELECT * FROM bantuan.rbac_permissions WHERE role = $1 AND resource = $2 AND action = $3 LIMIT 1"#,
        role, resource, action
    )
    .fetch_optional(pool)
    .await?;
    Ok(perm.is_some())
}

// Middleware Axum untuk validasi permission
pub async fn rbac_middleware<B>(
    State(pool): State<PgPool>,
    req: Request<B>,
    next: Next<B>,
    required_resource: &'static str,
    required_action: &'static str,
) -> Result<Response, AppError> {
    // Ambil role user dari header (atau session/auth)
    let role = req.headers().get("x-user-role")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("user");
    let allowed = has_permission(&pool, role, required_resource, required_action).await?;
    if !allowed {
        return Err(AppError::Forbidden);
    }
    Ok(next.run(req).await)
} 