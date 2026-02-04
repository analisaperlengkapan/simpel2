use crate::error::AppError;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use deadpool_postgres::Pool;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Role {
    User,
    Agent,
    Admin,
}

impl Role {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "user" => Some(Role::User),
            "agent" => Some(Role::Agent),
            "admin" => Some(Role::Admin),
            _ => None,
        }
    }
}

#[allow(dead_code)]
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
#[allow(dead_code)]
pub async fn rbac_middleware(
    State(pool): State<deadpool_postgres::Pool>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // TODO: Extract role from JWT or authenticated session
    // For now, default to "user" but DO NOT trust x-user-role header directly in production
    // This placeholder mocks role extraction until auth middleware injects user identity
    let role = "user";

    // TODO: Derive required resource/action from the route path or extensions
    // For now, we allow access to proceed if no specific rule is violated
    // In a real implementation, these would be matched against the request path/method
    let required_resource = "default";
    let required_action = "read";

    let allowed = has_permission(&pool, role, required_resource, required_action).await?;
    if !allowed {
        return Err(AppError::Forbidden);
    }
    Ok(next.run(req).await)
}
