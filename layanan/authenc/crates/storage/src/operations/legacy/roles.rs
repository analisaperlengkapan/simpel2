/// Database operations for role management
use crate::{database::Database, error::Result, models::Role};
use chrono::Utc;
use uuid::Uuid;

pub async fn create_role(
    db: &Database,
    name: &str,
    description: Option<&str>,
    realm_id: &Uuid,
) -> Result<Role> {
    let client = db.get_connection().await?;
    let role_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO roles (id, name, description, realm_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING
            id, name, description, realm_id, composite, client_role,
            client_id, attributes, created_at, updated_at
    "#;

    let row = client
        .query_one(
            query,
            &[&role_id, &name, &description, &realm_id, &now, &now],
        )
        .await?;

    // Convert row to Role
    Ok(Role {
        id: row.get(0),
        name: row.get(1),
        description: row.get(2),
        realm_id: row.get(3),
        composite: row.get(4),
        client_role: row.get(5),
        client_id: row.get(6),
        attributes: row
            .get::<_, Option<String>>(7)
            .and_then(|s: String| serde_json::from_str(&s).ok()),
        created_at: row.get(8),
        updated_at: row.get(9),
        deleted_at: None, // Not selected in query
        permissions: Vec::new(),
        managed_by: None,
        scope: None,
        priority: 0,
        active: true,
    })
}

pub async fn get_role_by_id(db: &Database, role_id: &Uuid) -> Result<Option<Role>> {
    let client = db.get_connection().await?;
    let query = r#"
        SELECT
            id, name, description, realm_id, composite, client_role,
            client_id, attributes, created_at, updated_at
        FROM roles
        WHERE id = $1 AND deleted_at IS NULL
    "#;

    let row = client.query_opt(query, &[&role_id]).await?;
    Ok(row.map(|r| Role {
        id: r.get(0),
        name: r.get(1),
        description: r.get(2),
        realm_id: r.get(3),
        composite: r.get(4),
        client_role: r.get(5),
        client_id: r.get(6),
        attributes: r
            .get::<_, Option<String>>(7)
            .and_then(|s: String| serde_json::from_str(&s).ok()),
        created_at: r.get(8),
        updated_at: r.get(9),
        deleted_at: None, // Not selected in query
        permissions: Vec::new(),
        managed_by: None,
        scope: None,
        priority: 0,
        active: true,
    }))
}

pub async fn list_roles_by_realm(db: &Database, realm_id: &Uuid) -> Result<Vec<Role>> {
    let client = db.get_connection().await?;
    let query = r#"
        SELECT
            id, name, description, realm_id, composite, client_role,
            client_id, attributes, created_at, updated_at
        FROM roles
        WHERE realm_id = $1 AND deleted_at IS NULL
        ORDER BY created_at DESC
    "#;

    let rows = client.query(query, &[&realm_id]).await?;
    let mut roles = Vec::new();

    for row in rows {
        roles.push(Role {
            id: row.get(0),
            name: row.get(1),
            description: row.get(2),
            realm_id: row.get(3),
            composite: row.get(4),
            client_role: row.get(5),
            client_id: row.get(6),
            attributes: row
                .get::<_, Option<String>>(7)
                .and_then(|s: String| serde_json::from_str(&s).ok()),
            created_at: row.get(8),
            updated_at: row.get(9),
            deleted_at: None, // Not selected in query
            permissions: Vec::new(),
            managed_by: None,
            scope: None,
            priority: 0,
            active: true,
        });
    }

    Ok(roles)
}

pub async fn assign_role_to_user(db: &Database, user_id: &Uuid, role_id: &Uuid) -> Result<()> {
    let client = db.get_connection().await?;
    let now = Utc::now();

    let query = r#"
        INSERT INTO user_roles (user_id, role_id, assigned_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, role_id) DO NOTHING
    "#;

    client.execute(query, &[&user_id, &role_id, &now]).await?;
    Ok(())
}

pub async fn remove_role_from_user(db: &Database, user_id: &Uuid, role_id: &Uuid) -> Result<()> {
    let client = db.get_connection().await?;
    let query = r#"
        DELETE FROM user_roles
        WHERE user_id = $1 AND role_id = $2
    "#;

    client.execute(query, &[&user_id, &role_id]).await?;
    Ok(())
}

pub async fn get_user_roles(db: &Database, user_id: &Uuid) -> Result<Vec<Role>> {
    let client = db.get_connection().await?;
    let query = r#"
        SELECT r.id, r.name, r.description, r.realm_id, r.composite, r.client_role,
               r.client_id, r.attributes, r.created_at, r.updated_at
        FROM roles r
        JOIN user_roles ur ON r.id = ur.role_id
        WHERE ur.user_id = $1 AND r.deleted_at IS NULL
    "#;

    let rows = client.query(query, &[&user_id]).await?;
    let mut roles = Vec::new();

    for row in rows {
        roles.push(Role {
            id: row.get(0),
            name: row.get(1),
            description: row.get(2),
            realm_id: row.get(3),
            composite: row.get(4),
            client_role: row.get(5),
            client_id: row.get(6),
            attributes: row
                .get::<_, Option<String>>(7)
                .and_then(|s: String| serde_json::from_str(&s).ok()),
            created_at: row.get(8),
            updated_at: row.get(9),
            deleted_at: None, // Not selected in query
            permissions: Vec::new(),
            managed_by: None,
            scope: None,
            priority: 0,
            active: true,
        });
    }

    Ok(roles)
}

pub async fn user_has_permission(db: &Database, user_id: &Uuid, permission: &str) -> Result<bool> {
    let query = r#"
        SELECT COUNT(*) > 0
        FROM user_roles ur
        JOIN role_permissions rp ON ur.role_id = rp.role_id
        JOIN permissions p ON rp.permission_id = p.id
        WHERE ur.user_id = $1 AND p.name = $2 AND p.deleted_at IS NULL
    "#;

    let row = db.query_one(query, &[&user_id, &permission]).await?;
    let has_permission: bool = row.get(0);

    Ok(has_permission)
}

pub async fn get_user_permissions(db: &Database, user_id: &Uuid) -> Result<Vec<String>> {
    let query = r#"
        SELECT DISTINCT p.name
        FROM user_roles ur
        JOIN role_permissions rp ON ur.role_id = rp.role_id
        JOIN permissions p ON rp.permission_id = p.id
        WHERE ur.user_id = $1 AND p.deleted_at IS NULL
        ORDER BY p.name
    "#;

    let rows = db.query(query, &[&user_id]).await?;
    let permissions = rows
        .into_iter()
        .map(|row| row.get::<_, String>(0))
        .collect();

    Ok(permissions)
}

pub async fn assign_permission_to_role(
    db: &Database,
    role_id: &Uuid,
    permission_id: &Uuid,
) -> Result<()> {
    let now = Utc::now();

    let query = r#"
        INSERT INTO role_permissions (role_id, permission_id, assigned_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (role_id, permission_id) DO NOTHING
    "#;

    db.execute(query, &[&role_id, &permission_id, &now]).await?;

    Ok(())
}

pub async fn remove_permission_from_role(
    db: &Database,
    role_id: &Uuid,
    permission_id: &Uuid,
) -> Result<()> {
    let query = r#"
        DELETE FROM role_permissions
        WHERE role_id = $1 AND permission_id = $2
    "#;

    db.execute(query, &[&role_id, &permission_id]).await?;

    Ok(())
}
