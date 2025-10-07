use crate::error::AppError;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use deadpool_postgres::Pool;

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

pub async fn has_permission(
    pool: &Pool,
    role: &str,
    resource: &str,
    action: &str,
) -> Result<bool, AppError> {
    let client = pool.get().await?;
    let row = client.query_opt(
        r#"SELECT * FROM bantuan.rbac_permissions WHERE role = $1 AND resource = $2 AND action = $3 LIMIT 1"#,
        &[&role, &resource, &action]
    ).await?;
    Ok(row.is_some())
}

// Middleware Axum untuk validasi permission
pub async fn rbac_middleware(
    State(pool): State<deadpool_postgres::Pool>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Ambil role user dari header (atau session/auth)
    let role = req
        .headers()
        .get("x-user-role")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("user");

    // Ambil required resource dan action dari header atau extension
    let required_resource = req
        .headers()
        .get("x-required-resource")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default");
    let required_action = req
        .headers()
        .get("x-required-action")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("read");

    let allowed = has_permission(&pool, role, required_resource, required_action).await?;
    if !allowed {
        return Err(AppError::Forbidden);
    }
    Ok(next.run(req).await)
}
