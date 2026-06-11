/// Database operations for group management
use crate::Database;
use authenc_types::domain::group::Group;
use authenc_types::{AuthencError, Result};
use chrono::Utc;
use tracing::{error, warn};
use uuid::Uuid;

pub async fn create_group(
    db: &Database,
    realm_id: Uuid,
    name: &str,
    parent_id: Option<Uuid>,
    description: Option<&str>,
    attributes: &serde_json::Value,
) -> Result<Group> {
    let group_id = Uuid::new_v4();
    let now = Utc::now();

    // Calculate path based on parent
    let path = if let Some(pid) = parent_id {
        // Get parent path
        let parent_row: tokio_postgres::Row = db
            .query_one("SELECT path, realm_id FROM groups WHERE id = $1", &[&pid])
            .await
            .map_err(|e| {
                error!("Failed to get parent group: {}", e);
                AuthencError::not_found(format!("Parent group {} not found", pid))
            })?;

        let parent_path: String = parent_row.get(0);
        let parent_realm: Uuid = parent_row.get(1);

        // Verify parent is in same realm
        if parent_realm != realm_id {
            return Err(AuthencError::validation(
                "Parent group must belong to the same realm",
            ));
        }

        format!("{}/{}", parent_path, name)
    } else {
        format!("/{}", name)
    };

    let query = r#"
        INSERT INTO groups (
            id, realm_id, parent_id, name, path,
            description, attributes, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING
            id, realm_id, parent_id, name, path,
            description, attributes, created_at, updated_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &group_id,
                &realm_id,
                &parent_id,
                &name,
                &path,
                &description,
                &attributes,
                &now,
                &now,
            ],
        )
        .await
        .map_err(|e| {
            error!("Group creation failed: {}", e);
            if e.to_string().contains("uq_groups_name_parent") {
                AuthencError::conflict("Group name already exists in this parent")
            } else {
                AuthencError::database(format!("Failed to create group: {}", e))
            }
        })?;

    Ok(Group {
        id: row.get(0),
        realm_id: row.get(1),
        parent_id: row.get(2),
        name: row.get(3),
        path: row.get(4),
        description: row.get(5),
        attributes: row.get(6),
        created_at: row.get(7),
        updated_at: row.get(8),
    })
}

pub async fn get_group_by_id(db: &Database, group_id: Uuid) -> Result<Option<Group>> {
    let query = r#"
        SELECT
            id, realm_id, parent_id, name, path,
            description, attributes, created_at, updated_at
        FROM groups
        WHERE id = $1
    "#;

    match db.query_opt(query, &[&group_id]).await {
        Ok(Some(row)) => Ok(Some(Group {
            id: row.get(0),
            realm_id: row.get(1),
            parent_id: row.get(2),
            name: row.get(3),
            path: row.get(4),
            description: row.get(5),
            attributes: row.get(6),
            created_at: row.get(7),
            updated_at: row.get(8),
        })),
        Ok(None) => Ok(None),
        Err(e) => {
            error!("Failed to get group: {}", e);
            Err(AuthencError::database(format!(
                "Failed to get group: {}",
                e
            )))
        }
    }
}

pub async fn get_groups_by_realm(
    db: &Database,
    realm_id: Uuid,
    first: Option<i64>,
    max: Option<i64>,
) -> Result<Vec<Group>> {
    let query = r#"
        SELECT
            id, realm_id, parent_id, name, path,
            description, attributes, created_at, updated_at
        FROM groups
        WHERE realm_id = $1
        ORDER BY path
        LIMIT $2 OFFSET $3
    "#;

    let limit = max.unwrap_or(100).min(1000);
    let offset = first.unwrap_or(0);

    let rows = db
        .query(query, &[&realm_id, &limit, &offset])
        .await
        .map_err(|e| {
            error!("Failed to get groups by realm: {}", e);
            AuthencError::database(format!("Failed to get groups: {}", e))
        })?;

    Ok(rows
        .iter()
        .map(|row: &tokio_postgres::Row| Group {
            id: row.get::<_, Uuid>(0),
            realm_id: row.get::<_, Uuid>(1),
            parent_id: row.get::<_, Option<Uuid>>(2),
            name: row.get::<_, String>(3),
            path: row.get::<_, String>(4),
            description: row.get::<_, Option<String>>(5),
            attributes: row.get::<_, serde_json::Value>(6),
            created_at: row.get::<_, chrono::DateTime<chrono::Utc>>(7),
            updated_at: row.get::<_, chrono::DateTime<chrono::Utc>>(8),
        })
        .collect())
}

pub async fn get_subgroups(
    db: &Database,
    parent_id: Uuid,
    direct_only: bool,
) -> Result<Vec<Group>> {
    let query = if direct_only {
        r#"
            SELECT
                id, realm_id, parent_id, name, path,
                description, attributes, created_at, updated_at
            FROM groups
            WHERE parent_id = $1
            ORDER BY name
        "#
    } else {
        // Get all descendants by path prefix matching
        r#"
            SELECT
                g.id, g.realm_id, g.parent_id, g.name, g.path,
                g.description, g.attributes, g.created_at, g.updated_at
            FROM groups g
            INNER JOIN groups parent ON parent.id = $1
            WHERE g.path LIKE parent.path || '/%'
            ORDER BY g.path
        "#
    };

    let rows = db.query(query, &[&parent_id]).await.map_err(|e| {
        error!("Failed to get subgroups: {}", e);
        AuthencError::database(format!("Failed to get subgroups: {}", e))
    })?;

    Ok(rows
        .iter()
        .map(|row: &tokio_postgres::Row| Group {
            id: row.get::<_, Uuid>(0),
            realm_id: row.get::<_, Uuid>(1),
            parent_id: row.get::<_, Option<Uuid>>(2),
            name: row.get::<_, String>(3),
            path: row.get::<_, String>(4),
            description: row.get::<_, Option<String>>(5),
            attributes: row.get::<_, serde_json::Value>(6),
            created_at: row.get::<_, chrono::DateTime<chrono::Utc>>(7),
            updated_at: row.get::<_, chrono::DateTime<chrono::Utc>>(8),
        })
        .collect())
}

pub async fn update_group(
    db: &Database,
    group_id: Uuid,
    name: Option<String>,
    parent_id: Option<Option<Uuid>>, // None = no change, Some(None) = set to null, Some(Some(id)) = set to id
    description: Option<Option<String>>,
    attributes: Option<serde_json::Value>,
) -> Result<Group> {
    let now = Utc::now();

    // Build query based on what's being updated
    let has_name = name.is_some();
    let has_parent = parent_id.is_some();
    let has_desc = description.is_some();
    let has_attrs = attributes.is_some();

    if !has_name && !has_parent && !has_desc && !has_attrs {
        // No updates, just return current group
        return get_group_by_id(db, group_id)
            .await?
            .ok_or_else(|| AuthencError::not_found("Group not found"));
    }

    // Use COALESCE for conditional updates
    let query = r#"
        UPDATE groups
        SET
            name = COALESCE($2, name),
            parent_id = CASE WHEN $3::boolean THEN $4 ELSE parent_id END,
            description = CASE WHEN $5::boolean THEN $6 ELSE description END,
            attributes = COALESCE($7, attributes),
            updated_at = $8
        WHERE id = $1
        RETURNING
            id, realm_id, parent_id, name, path,
            description, attributes, created_at, updated_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &group_id,
                &name,
                &has_parent,
                &parent_id.flatten(),
                &has_desc,
                &description.flatten(),
                &attributes,
                &now,
            ],
        )
        .await
        .map_err(|e| {
            error!("Group update failed: {}", e);
            AuthencError::database(format!("Failed to update group: {}", e))
        })?;

    Ok(Group {
        id: row.get(0),
        realm_id: row.get(1),
        parent_id: row.get(2),
        name: row.get(3),
        path: row.get(4),
        description: row.get(5),
        attributes: row.get(6),
        created_at: row.get(7),
        updated_at: row.get(8),
    })
}

pub async fn delete_group(db: &Database, group_id: Uuid) -> Result<()> {
    let query = "DELETE FROM groups WHERE id = $1";

    let rows_affected = db.execute(query, &[&group_id]).await.map_err(|e| {
        error!("Group deletion failed: {}", e);
        AuthencError::database(format!("Failed to delete group: {}", e))
    })?;

    if rows_affected == 0 {
        return Err(AuthencError::not_found("Group not found"));
    }

    Ok(())
}

pub async fn add_user_to_group(
    db: &Database,
    user_id: Uuid,
    group_id: Uuid,
    expires_at: Option<chrono::DateTime<Utc>>,
    attributes: Option<serde_json::Value>,
) -> Result<()> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let attrs = attributes.unwrap_or_else(|| serde_json::json!({}));

    let query = r#"
        INSERT INTO user_groups (id, user_id, group_id, joined_at, expires_at, attributes)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (user_id, group_id) DO UPDATE
        SET expires_at = EXCLUDED.expires_at, attributes = EXCLUDED.attributes
    "#;

    db.execute(
        query,
        &[&id, &user_id, &group_id, &now, &expires_at, &attrs],
    )
    .await
    .map_err(|e| {
        error!("Failed to add user to group: {}", e);
        AuthencError::database(format!("Failed to add user to group: {}", e))
    })?;

    Ok(())
}

pub async fn remove_user_from_group(db: &Database, user_id: Uuid, group_id: Uuid) -> Result<()> {
    let query = "DELETE FROM user_groups WHERE user_id = $1 AND group_id = $2";

    let rows_affected = db
        .execute(query, &[&user_id, &group_id])
        .await
        .map_err(|e| {
            error!("Failed to remove user from group: {}", e);
            AuthencError::database(format!("Failed to remove user from group: {}", e))
        })?;

    if rows_affected == 0 {
        warn!(
            "Attempted to remove user {} from group {} but membership didn't exist",
            user_id, group_id
        );
    }

    Ok(())
}

pub async fn get_user_groups(db: &Database, user_id: Uuid) -> Result<Vec<Group>> {
    let query = r#"
        SELECT
            g.id, g.realm_id, g.parent_id, g.name, g.path,
            g.description, g.attributes, g.created_at, g.updated_at
        FROM groups g
        INNER JOIN user_groups ug ON g.id = ug.group_id
        WHERE ug.user_id = $1
          AND (ug.expires_at IS NULL OR ug.expires_at > NOW())
        ORDER BY g.path
    "#;

    let rows = db.query(query, &[&user_id]).await.map_err(|e| {
        error!("Failed to get user groups: {}", e);
        AuthencError::database(format!("Failed to get user groups: {}", e))
    })?;

    Ok(rows
        .iter()
        .map(|row: &tokio_postgres::Row| Group {
            id: row.get::<_, Uuid>(0),
            realm_id: row.get::<_, Uuid>(1),
            parent_id: row.get::<_, Option<Uuid>>(2),
            name: row.get::<_, String>(3),
            path: row.get::<_, String>(4),
            description: row.get::<_, Option<String>>(5),
            attributes: row.get::<_, serde_json::Value>(6),
            created_at: row.get::<_, chrono::DateTime<chrono::Utc>>(7),
            updated_at: row.get::<_, chrono::DateTime<chrono::Utc>>(8),
        })
        .collect())
}

pub async fn get_group_members(
    db: &Database,
    group_id: Uuid,
    first: Option<i64>,
    max: Option<i64>,
) -> Result<Vec<Uuid>> {
    let query = r#"
        SELECT user_id
        FROM user_groups
        WHERE group_id = $1
          AND (expires_at IS NULL OR expires_at > NOW())
        ORDER BY joined_at
        LIMIT $2 OFFSET $3
    "#;

    let limit = max.unwrap_or(100).min(1000);
    let offset = first.unwrap_or(0);

    let rows = db
        .query(query, &[&group_id, &limit, &offset])
        .await
        .map_err(|e| {
            error!("Failed to get group members: {}", e);
            AuthencError::database(format!("Failed to get group members: {}", e))
        })?;

    Ok(rows
        .iter()
        .map(|row: &tokio_postgres::Row| row.get::<_, Uuid>(0))
        .collect())
}

pub async fn count_group_members(db: &Database, group_id: Uuid) -> Result<i64> {
    let query = r#"
        SELECT COUNT(*)::bigint
        FROM user_groups
        WHERE group_id = $1
          AND (expires_at IS NULL OR expires_at > NOW())
    "#;

    let row: tokio_postgres::Row = db.query_one(query, &[&group_id]).await.map_err(|e| {
        error!("Failed to count group members: {}", e);
        AuthencError::database(format!("Failed to count group members: {}", e))
    })?;

    Ok(row.get(0))
}

pub async fn count_subgroups(db: &Database, group_id: Uuid) -> Result<i64> {
    let query = r#"
        SELECT COUNT(*)::bigint
        FROM groups
        WHERE parent_id = $1
    "#;

    let row: tokio_postgres::Row = db.query_one(query, &[&group_id]).await.map_err(|e| {
        error!("Failed to count subgroups: {}", e);
        AuthencError::database(format!("Failed to count subgroups: {}", e))
    })?;

    Ok(row.get(0))
}
