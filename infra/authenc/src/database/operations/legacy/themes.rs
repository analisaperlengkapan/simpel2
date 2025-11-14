/// Theme customization database operations
use super::*;

pub async fn create_theme(
    db: &Database,
    realm_id: Uuid,
    name: &str,
    theme_type: &str,
    parent_theme: Option<&str>,
    css_content: Option<&str>,
    css_variables: Option<serde_json::Value>,
    description: Option<&str>,
) -> Result<serde_json::Value> {
    let query = r#"
        INSERT INTO custom_themes (
            realm_id, name, theme_type, parent_theme,
            css_content, css_variables, description
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, realm_id, name, theme_type, parent_theme,
                  is_active, is_default, created_at, updated_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &realm_id,
                &name,
                &theme_type,
                &parent_theme,
                &css_content,
                &css_variables,
                &description,
            ],
        )
        .await?;

    Ok(serde_json::json!({
        "id": row.get::<_, Uuid>("id"),
        "realm_id": row.get::<_, Uuid>("realm_id"),
        "name": row.get::<_, String>("name"),
        "theme_type": row.get::<_, String>("theme_type"),
        "parent_theme": row.get::<_, Option<String>>("parent_theme"),
        "is_active": row.get::<_, bool>("is_active"),
        "is_default": row.get::<_, bool>("is_default"),
        "created_at": row.get::<_, chrono::DateTime<chrono::Utc>>("created_at"),
        "updated_at": row.get::<_, chrono::DateTime<chrono::Utc>>("updated_at")
    }))
}

pub async fn get_theme(db: &Database, theme_id: Uuid) -> Result<Option<serde_json::Value>> {
    let query = r#"
        SELECT id, realm_id, name, theme_type, parent_theme,
               css_content, css_variables, templates, resources, messages,
               description, version, author, is_active, is_default,
               created_at, updated_at
        FROM custom_themes
        WHERE id = $1
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&theme_id]).await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let row = &rows[0];
    Ok(Some(serde_json::json!({
        "id": row.get::<_, Uuid>("id"),
        "realm_id": row.get::<_, Uuid>("realm_id"),
        "name": row.get::<_, String>("name"),
        "theme_type": row.get::<_, String>("theme_type"),
        "parent_theme": row.get::<_, Option<String>>("parent_theme"),
        "css_content": row.get::<_, Option<String>>("css_content"),
        "css_variables": row.get::<_, Option<serde_json::Value>>("css_variables"),
        "templates": row.get::<_, Option<serde_json::Value>>("templates"),
        "resources": row.get::<_, Option<serde_json::Value>>("resources"),
        "messages": row.get::<_, Option<serde_json::Value>>("messages"),
        "description": row.get::<_, Option<String>>("description"),
        "version": row.get::<_, Option<String>>("version"),
        "author": row.get::<_, Option<String>>("author"),
        "is_active": row.get::<_, bool>("is_active"),
        "is_default": row.get::<_, bool>("is_default"),
        "created_at": row.get::<_, chrono::DateTime<chrono::Utc>>("created_at"),
        "updated_at": row.get::<_, chrono::DateTime<chrono::Utc>>("updated_at")
    })))
}

pub async fn get_realm_themes(
    db: &Database,
    realm_id: Uuid,
    theme_type: Option<&str>,
) -> Result<Vec<serde_json::Value>> {
    let query = if theme_type.is_some() {
        r#"
            SELECT id, realm_id, name, theme_type, parent_theme,
                   description, is_active, is_default,
                   created_at, updated_at
            FROM custom_themes
            WHERE realm_id = $1 AND theme_type = $2
            ORDER BY name ASC
        "#
    } else {
        r#"
            SELECT id, realm_id, name, theme_type, parent_theme,
                   description, is_active, is_default,
                   created_at, updated_at
            FROM custom_themes
            WHERE realm_id = $1
            ORDER BY theme_type ASC, name ASC
        "#
    };

    let rows: Vec<tokio_postgres::Row> = if let Some(ttype) = theme_type {
        db.query(query, &[&realm_id, &ttype]).await?
    } else {
        db.query(query, &[&realm_id]).await?
    };

    let mut themes = Vec::new();
    for row in rows {
        themes.push(serde_json::json!({
            "id": row.get::<_, Uuid>("id"),
            "realm_id": row.get::<_, Uuid>("realm_id"),
            "name": row.get::<_, String>("name"),
            "theme_type": row.get::<_, String>("theme_type"),
            "parent_theme": row.get::<_, Option<String>>("parent_theme"),
            "description": row.get::<_, Option<String>>("description"),
            "is_active": row.get::<_, bool>("is_active"),
            "is_default": row.get::<_, bool>("is_default"),
            "created_at": row.get::<_, chrono::DateTime<chrono::Utc>>("created_at"),
            "updated_at": row.get::<_, chrono::DateTime<chrono::Utc>>("updated_at")
        }));
    }

    Ok(themes)
}

pub async fn update_theme(
    db: &Database,
    theme_id: Uuid,
    updates: serde_json::Value,
) -> Result<()> {
    let now = chrono::Utc::now();
    let mut set_clauses = Vec::new();
    let mut param_index = 2;
    let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> =
        vec![Box::new(theme_id)];

    if let Some(name) = updates.get("name").and_then(|v| v.as_str()) {
        set_clauses.push(format!("name = ${}", param_index));
        params.push(Box::new(name.to_string()));
        param_index += 1;
    }

    if let Some(css_content) = updates.get("css_content").and_then(|v| v.as_str()) {
        set_clauses.push(format!("css_content = ${}", param_index));
        params.push(Box::new(css_content.to_string()));
        param_index += 1;
    }

    if let Some(css_variables) = updates.get("css_variables") {
        set_clauses.push(format!("css_variables = ${}", param_index));
        params.push(Box::new(css_variables.clone()));
        param_index += 1;
    }

    if let Some(templates) = updates.get("templates") {
        set_clauses.push(format!("templates = ${}", param_index));
        params.push(Box::new(templates.clone()));
        param_index += 1;
    }

    if let Some(messages) = updates.get("messages") {
        set_clauses.push(format!("messages = ${}", param_index));
        params.push(Box::new(messages.clone()));
        param_index += 1;
    }

    if set_clauses.is_empty() {
        return Ok(());
    }

    set_clauses.push(format!("updated_at = ${}", param_index));
    params.push(Box::new(now));

    let query = format!(
        "UPDATE custom_themes SET {} WHERE id = $1",
        set_clauses.join(", ")
    );

    let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        params.iter().map(|p| p.as_ref()).collect();

    db.execute(&query, &params_refs).await?;
    Ok(())
}

pub async fn activate_theme(db: &Database, theme_id: Uuid) -> Result<()> {
    let now = chrono::Utc::now();

    // Deactivate other themes of same type in same realm
    let deactivate_query = r#"
        UPDATE custom_themes
        SET is_active = FALSE, updated_at = $1
        WHERE realm_id = (SELECT realm_id FROM custom_themes WHERE id = $2)
            AND theme_type = (SELECT theme_type FROM custom_themes WHERE id = $2)
            AND id != $2
    "#;

    db.execute(deactivate_query, &[&now, &theme_id]).await?;

    // Activate the selected theme
    let activate_query = r#"
        UPDATE custom_themes
        SET is_active = TRUE, updated_at = $1
        WHERE id = $2
    "#;

    db.execute(activate_query, &[&now, &theme_id]).await?;
    Ok(())
}

pub async fn delete_theme(db: &Database, theme_id: Uuid) -> Result<()> {
    let query = "DELETE FROM custom_themes WHERE id = $1";
    db.execute(query, &[&theme_id]).await?;
    Ok(())
}

pub async fn add_theme_resource(
    db: &Database,
    theme_id: Uuid,
    resource_name: &str,
    resource_type: &str,
    mime_type: Option<&str>,
    content_url: Option<&str>,
    content_data: Option<&[u8]>,
) -> Result<Uuid> {
    let query = r#"
        INSERT INTO theme_resources (
            theme_id, resource_name, resource_type,
            mime_type, content_url, content_data, content_size
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (theme_id, resource_name)
        DO UPDATE SET
            resource_type = EXCLUDED.resource_type,
            mime_type = EXCLUDED.mime_type,
            content_url = EXCLUDED.content_url,
            content_data = EXCLUDED.content_data,
            content_size = EXCLUDED.content_size,
            updated_at = NOW()
        RETURNING id
    "#;

    let content_size = content_data.map(|d| d.len() as i64);

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &theme_id,
                &resource_name,
                &resource_type,
                &mime_type,
                &content_url,
                &content_data,
                &content_size,
            ],
        )
        .await?;

    Ok(row.get(0))
}

pub async fn get_theme_resource(
    db: &Database,
    theme_id: Uuid,
    resource_name: &str,
) -> Result<Option<serde_json::Value>> {
    let query = r#"
        SELECT id, theme_id, resource_name, resource_type,
               mime_type, content_url, content_size,
               created_at, updated_at
        FROM theme_resources
        WHERE theme_id = $1 AND resource_name = $2
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&theme_id, &resource_name]).await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let row = &rows[0];
    Ok(Some(serde_json::json!({
        "id": row.get::<_, Uuid>("id"),
        "theme_id": row.get::<_, Uuid>("theme_id"),
        "resource_name": row.get::<_, String>("resource_name"),
        "resource_type": row.get::<_, String>("resource_type"),
        "mime_type": row.get::<_, Option<String>>("mime_type"),
        "content_url": row.get::<_, Option<String>>("content_url"),
        "content_size": row.get::<_, Option<i64>>("content_size"),
        "created_at": row.get::<_, chrono::DateTime<chrono::Utc>>("created_at"),
        "updated_at": row.get::<_, chrono::DateTime<chrono::Utc>>("updated_at")
    })))
}

pub async fn save_theme_template(
    db: &Database,
    theme_id: Uuid,
    template_name: &str,
    template_type: &str,
    content: &str,
) -> Result<Uuid> {
    let query = r#"
        INSERT INTO theme_templates (
            theme_id, template_name, template_type, content
        )
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (theme_id, template_name)
        DO UPDATE SET
            content = EXCLUDED.content,
            version = theme_templates.version + 1,
            updated_at = NOW()
        RETURNING id
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[&theme_id, &template_name, &template_type, &content],
        )
        .await?;

    Ok(row.get(0))
}

pub async fn get_theme_template(
    db: &Database,
    theme_id: Uuid,
    template_name: &str,
) -> Result<Option<String>> {
    let query = r#"
        SELECT content
        FROM theme_templates
        WHERE theme_id = $1 AND template_name = $2
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&theme_id, &template_name]).await?;

    if rows.is_empty() {
        return Ok(None);
    }

    Ok(Some(rows[0].get(0)))
}

pub async fn set_realm_theme(
    db: &Database,
    realm_id: Uuid,
    theme_type: &str,
    theme_id: Uuid,
) -> Result<()> {
    let column_name = match theme_type {
        "login" => "login_theme_id",
        "account" => "account_theme_id",
        "admin" => "admin_theme_id",
        "email" => "email_theme_id",
        _ => return Err(crate::error::AuthencError::validation("Invalid theme type")),
    };

    let query = format!(
        r#"
            INSERT INTO realm_theme_settings (realm_id, {})
            VALUES ($1, $2)
            ON CONFLICT (realm_id)
            DO UPDATE SET {} = EXCLUDED.{}, updated_at = NOW()
        "#,
        column_name, column_name, column_name
    );

    db.execute(&query, &[&realm_id, &theme_id]).await?;
    Ok(())
}

pub async fn get_realm_active_themes(
    db: &Database,
    realm_id: Uuid,
) -> Result<serde_json::Value> {
    let query = r#"
        SELECT login_theme_id, account_theme_id, admin_theme_id, email_theme_id
        FROM realm_theme_settings
        WHERE realm_id = $1
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&realm_id]).await?;

    if rows.is_empty() {
        return Ok(serde_json::json!({}));
    }

    let row = &rows[0];
    Ok(serde_json::json!({
        "login_theme_id": row.get::<_, Option<Uuid>>("login_theme_id"),
        "account_theme_id": row.get::<_, Option<Uuid>>("account_theme_id"),
        "admin_theme_id": row.get::<_, Option<Uuid>>("admin_theme_id"),
        "email_theme_id": row.get::<_, Option<Uuid>>("email_theme_id")
    }))
}
