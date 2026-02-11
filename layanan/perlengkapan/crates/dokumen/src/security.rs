use crate::error::AppError;
use crate::models::DocumentPermission;
use uuid::Uuid;

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

pub async fn get_user_role(
    pool: &deadpool_postgres::Pool,
    user_id: Uuid,
    document_id: Uuid,
) -> Result<Option<Role>, AppError> {
    let client = pool.get().await?;
    let row = client
        .query_opt(
            "SELECT * FROM dokumen.document_permissions WHERE user_id = $1 AND document_id = $2 LIMIT 1",
            &[&user_id, &document_id],
        )
        .await?;

    Ok(row.map(|r| DocumentPermission::from(&r)).and_then(|p| Role::from_str(&p.role)))
}

pub async fn is_owner(pool: &deadpool_postgres::Pool, user_id: Uuid, document_id: Uuid) -> Result<bool, AppError> {
    Ok(get_user_role(pool, user_id, document_id).await? == Some(Role::Owner))
}

pub async fn is_editor(pool: &deadpool_postgres::Pool, user_id: Uuid, document_id: Uuid) -> Result<bool, AppError> {
    match get_user_role(pool, user_id, document_id).await? {
        Some(Role::Owner) | Some(Role::Editor) => Ok(true),
        _ => Ok(false),
    }
}

pub async fn is_viewer(pool: &deadpool_postgres::Pool, user_id: Uuid, document_id: Uuid) -> Result<bool, AppError> {
    match get_user_role(pool, user_id, document_id).await? {
        Some(Role::Owner) | Some(Role::Editor) | Some(Role::Viewer) => Ok(true),
        _ => Ok(false),
    }
}
