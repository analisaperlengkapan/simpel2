use crate::models::DocumentPermission;
use crate::error::AppError;
use uuid::Uuid;
use sqlx::PgPool;
use async_trait::async_trait;
use axum::{extract::{State, Path, RequestPartsExt}, http::Request, middleware::Next, response::Response};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Owner,
    Editor,
    Viewer,
}

impl Role {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "owner" => Some(Role::Owner),
            "editor" => Some(Role::Editor),
            "viewer" => Some(Role::Viewer),
            _ => None,
        }
    }
}

pub async fn get_user_role(pool: &PgPool, user_id: Uuid, document_id: Uuid) -> Result<Option<Role>, AppError> {
    let perm = sqlx::query_as!(DocumentPermission,
        r#"SELECT * FROM dokumen.document_permissions WHERE user_id = $1 AND document_id = $2 LIMIT 1"#,
        user_id, document_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(perm.and_then(|p| Role::from_str(&p.role)))
}

pub async fn is_owner(pool: &PgPool, user_id: Uuid, document_id: Uuid) -> Result<bool, AppError> {
    Ok(get_user_role(pool, user_id, document_id).await? == Some(Role::Owner))
}

pub async fn is_editor(pool: &PgPool, user_id: Uuid, document_id: Uuid) -> Result<bool, AppError> {
    match get_user_role(pool, user_id, document_id).await? {
        Some(Role::Owner) | Some(Role::Editor) => Ok(true),
        _ => Ok(false),
    }
}

pub async fn is_viewer(pool: &PgPool, user_id: Uuid, document_id: Uuid) -> Result<bool, AppError> {
    match get_user_role(pool, user_id, document_id).await? {
        Some(Role::Owner) | Some(Role::Editor) | Some(Role::Viewer) => Ok(true),
        _ => Ok(false),
    }
}

// Middleware Axum untuk validasi permission
pub async fn permission_middleware<B>(
    State(pool): State<PgPool>,
    mut req: Request<B>,
    next: Next<B>,
    required_role: Role,
) -> Result<Response, AppError> {
    // Ambil user_id dan document_id dari header/query/path
    let user_id = req.headers().get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Unauthorized)?;
    let document_id = req.headers().get("x-document-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::BadRequest("document_id wajib".to_string()))?;
    let allowed = match required_role {
        Role::Owner => is_owner(&pool, user_id, document_id).await?,
        Role::Editor => is_editor(&pool, user_id, document_id).await?,
        Role::Viewer => is_viewer(&pool, user_id, document_id).await?,
    };
    if !allowed {
        return Err(AppError::Forbidden);
    }
    Ok(next.run(req).await)
} 