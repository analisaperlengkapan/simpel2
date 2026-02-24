use crate::Database;
use authenc_types::Result;
use authenc_core::models::client_scope::*;
use chrono::Utc;
use uuid::Uuid;

/// Database operations for client scopes
pub mod client_scopes {
    use super::*;

    /// Create a new client scope
    pub async fn create_scope(
        db: &Database,
        realm_id: Uuid,
        request: CreateClientScopeRequest,
    ) -> Result<ClientScope> {
        let scope_id = Uuid::new_v4();
        let now = Utc::now();
        let protocol = request
            .protocol
            .unwrap_or_else(|| "openid-connect".to_string());
        let consent_required = request.consent_required.unwrap_or(true);
        let display_on_consent_screen = request.display_on_consent_screen.unwrap_or(true);
        let include_in_token_scope = request.include_in_token_scope.unwrap_or(true);
        let gui_order = request.gui_order.unwrap_or(0);
        let attributes = request.attributes.unwrap_or_else(|| serde_json::json!({}));

        let query = r#"
            INSERT INTO client_scopes (
                id, realm_id, name, display_name, description, protocol,
                consent_required, display_on_consent_screen, consent_screen_text,
                include_in_token_scope, gui_order, icon_uri, attributes,
                enabled, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, TRUE, $14, $15)
            RETURNING
                id, realm_id, name, display_name, description, protocol,
                consent_required, display_on_consent_screen, consent_screen_text,
                include_in_token_scope, gui_order, icon_uri, attributes,
                enabled, created_at, updated_at
        "#;

        let row: tokio_postgres::Row = db
            .query_one(
                query,
                &[
                    &scope_id,
                    &realm_id,
                    &request.name,
                    &request.display_name,
                    &request.description,
                    &protocol,
                    &consent_required,
                    &display_on_consent_screen,
                    &request.consent_screen_text,
                    &include_in_token_scope,
                    &gui_order,
                    &request.icon_uri,
                    &attributes,
                    &now,
                    &now,
                ],
            )
            .await?;

        row.try_into()
    }

    /// Get a client scope by ID
    pub async fn get_scope_by_id(db: &Database, scope_id: Uuid) -> Result<Option<ClientScope>> {
        let query = r#"
            SELECT
                id, realm_id, name, display_name, description, protocol,
                consent_required, display_on_consent_screen, consent_screen_text,
                include_in_token_scope, gui_order, icon_uri, attributes,
                enabled, created_at, updated_at
            FROM client_scopes
            WHERE id = $1
        "#;

        let row = db.query_opt(query, &[&scope_id]).await?;
        match row {
            Some(row) => Ok(Some(row.try_into()?)),
            None => Ok(None),
        }
    }

    /// Get a client scope by name in a realm
    pub async fn get_scope_by_name(
        db: &Database,
        realm_id: Uuid,
        name: &str,
    ) -> Result<Option<ClientScope>> {
        let query = r#"
            SELECT
                id, realm_id, name, display_name, description, protocol,
                consent_required, display_on_consent_screen, consent_screen_text,
                include_in_token_scope, gui_order, icon_uri, attributes,
                enabled, created_at, updated_at
            FROM client_scopes
            WHERE realm_id = $1 AND name = $2
        "#;

        let row = db.query_opt(query, &[&realm_id, &name]).await?;
        match row {
            Some(row) => Ok(Some(row.try_into()?)),
            None => Ok(None),
        }
    }

    /// List all scopes in a realm
    pub async fn list_scopes(
        db: &Database,
        realm_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<ClientScope>> {
        let query = if enabled_only {
            r#"
                SELECT
                    id, realm_id, name, display_name, description, protocol,
                    consent_required, display_on_consent_screen, consent_screen_text,
                    include_in_token_scope, gui_order, icon_uri, attributes,
                    enabled, created_at, updated_at
                FROM client_scopes
                WHERE realm_id = $1 AND enabled = TRUE
                ORDER BY gui_order ASC, name ASC
            "#
        } else {
            r#"
                SELECT
                    id, realm_id, name, display_name, description, protocol,
                    consent_required, display_on_consent_screen, consent_screen_text,
                    include_in_token_scope, gui_order, icon_uri, attributes,
                    enabled, created_at, updated_at
                FROM client_scopes
                WHERE realm_id = $1
                ORDER BY gui_order ASC, name ASC
            "#
        };

        let rows: Vec<tokio_postgres::Row> = db.query(query, &[&realm_id]).await?;
        let mut scopes = Vec::new();

        for row in rows {
            scopes.push(row.try_into()?);
        }

        Ok(scopes)
    }

    /// Update a client scope
    pub async fn update_scope(
        db: &Database,
        scope_id: Uuid,
        request: UpdateClientScopeRequest,
    ) -> Result<ClientScope> {
        let now = Utc::now();

        let query = r#"
            UPDATE client_scopes
            SET
                display_name = COALESCE($2, display_name),
                description = COALESCE($3, description),
                consent_required = COALESCE($4, consent_required),
                display_on_consent_screen = COALESCE($5, display_on_consent_screen),
                consent_screen_text = COALESCE($6, consent_screen_text),
                include_in_token_scope = COALESCE($7, include_in_token_scope),
                gui_order = COALESCE($8, gui_order),
                icon_uri = COALESCE($9, icon_uri),
                attributes = COALESCE($10, attributes),
                enabled = COALESCE($11, enabled),
                updated_at = $12
            WHERE id = $1
            RETURNING
                id, realm_id, name, display_name, description, protocol,
                consent_required, display_on_consent_screen, consent_screen_text,
                include_in_token_scope, gui_order, icon_uri, attributes,
                enabled, created_at, updated_at
        "#;

        let row: tokio_postgres::Row = db
            .query_one(
                query,
                &[
                    &scope_id,
                    &request.display_name,
                    &request.description,
                    &request.consent_required,
                    &request.display_on_consent_screen,
                    &request.consent_screen_text,
                    &request.include_in_token_scope,
                    &request.gui_order,
                    &request.icon_uri,
                    &request.attributes,
                    &request.enabled,
                    &now,
                ],
            )
            .await?;

        row.try_into()
    }

    /// Delete a client scope
    pub async fn delete_scope(db: &Database, scope_id: Uuid) -> Result<bool> {
        let query = "DELETE FROM client_scopes WHERE id = $1";
        let rows_affected = db.execute(query, &[&scope_id]).await?;
        Ok(rows_affected > 0)
    }

    /// Get scopes by names (batch lookup)
    pub async fn get_scopes_by_names(
        db: &Database,
        realm_id: Uuid,
        names: &[String],
    ) -> Result<Vec<ClientScope>> {
        let query = r#"
            SELECT
                id, realm_id, name, display_name, description, protocol,
                consent_required, display_on_consent_screen, consent_screen_text,
                include_in_token_scope, gui_order, icon_uri, attributes,
                enabled, created_at, updated_at
            FROM client_scopes
            WHERE realm_id = $1 AND name = ANY($2) AND enabled = TRUE
            ORDER BY gui_order ASC, name ASC
        "#;

        let rows: Vec<tokio_postgres::Row> = db.query(query, &[&realm_id, &names]).await?;
        let mut scopes = Vec::new();

        for row in rows {
            scopes.push(row.try_into()?);
        }

        Ok(scopes)
    }
}

/// Database operations for client scope assignments
pub mod client_scope_assignments {
    use super::*;

    /// Assign default scopes to a client
    pub async fn assign_default_scopes(
        db: &Database,
        client_id: Uuid,
        scope_ids: &[Uuid],
    ) -> Result<()> {
        // Clear existing default scopes
        let clear_query = "DELETE FROM client_default_scopes WHERE client_id = $1";
        db.execute(clear_query, &[&client_id]).await?;

        // Insert new default scopes
        if !scope_ids.is_empty() {
            let insert_query = r#"
                INSERT INTO client_default_scopes (id, client_id, scope_id, created_at)
                SELECT gen_random_uuid(), $1, unnest($2::uuid[]), NOW()
            "#;
            db.execute(insert_query, &[&client_id, &scope_ids]).await?;
        }

        Ok(())
    }

    /// Assign optional scopes to a client
    pub async fn assign_optional_scopes(
        db: &Database,
        client_id: Uuid,
        scope_ids: &[Uuid],
    ) -> Result<()> {
        // Clear existing optional scopes
        let clear_query = "DELETE FROM client_optional_scopes WHERE client_id = $1";
        db.execute(clear_query, &[&client_id]).await?;

        // Insert new optional scopes
        if !scope_ids.is_empty() {
            let insert_query = r#"
                INSERT INTO client_optional_scopes (id, client_id, scope_id, created_at)
                SELECT gen_random_uuid(), $1, unnest($2::uuid[]), NOW()
            "#;
            db.execute(insert_query, &[&client_id, &scope_ids]).await?;
        }

        Ok(())
    }

    /// Get default scopes for a client
    pub async fn get_default_scopes(db: &Database, client_id: Uuid) -> Result<Vec<ClientScope>> {
        let query = r#"
            SELECT
                cs.id, cs.realm_id, cs.name, cs.display_name, cs.description, cs.protocol,
                cs.consent_required, cs.display_on_consent_screen, cs.consent_screen_text,
                cs.include_in_token_scope, cs.gui_order, cs.icon_uri, cs.attributes,
                cs.enabled, cs.created_at, cs.updated_at
            FROM client_scopes cs
            INNER JOIN client_default_scopes cds ON cs.id = cds.scope_id
            WHERE cds.client_id = $1 AND cs.enabled = TRUE
            ORDER BY cs.gui_order ASC, cs.name ASC
        "#;

        let rows: Vec<tokio_postgres::Row> = db.query(query, &[&client_id]).await?;
        let mut scopes = Vec::new();

        for row in rows {
            scopes.push(row.try_into()?);
        }

        Ok(scopes)
    }

    /// Get optional scopes for a client
    pub async fn get_optional_scopes(db: &Database, client_id: Uuid) -> Result<Vec<ClientScope>> {
        let query = r#"
            SELECT
                cs.id, cs.realm_id, cs.name, cs.display_name, cs.description, cs.protocol,
                cs.consent_required, cs.display_on_consent_screen, cs.consent_screen_text,
                cs.include_in_token_scope, cs.gui_order, cs.icon_uri, cs.attributes,
                cs.enabled, cs.created_at, cs.updated_at
            FROM client_scopes cs
            INNER JOIN client_optional_scopes cos ON cs.id = cos.scope_id
            WHERE cos.client_id = $1 AND cs.enabled = TRUE
            ORDER BY cs.gui_order ASC, cs.name ASC
        "#;

        let rows: Vec<tokio_postgres::Row> = db.query(query, &[&client_id]).await?;
        let mut scopes = Vec::new();

        for row in rows {
            scopes.push(row.try_into()?);
        }

        Ok(scopes)
    }

    /// Get all scopes for a client (default + optional)
    pub async fn get_all_client_scopes(
        db: &Database,
        client_id: Uuid,
    ) -> Result<Vec<ClientScopeAssignment>> {
        let query = r#"
            SELECT
                cs.id, cs.realm_id, cs.name, cs.display_name, cs.description, cs.protocol,
                cs.consent_required, cs.display_on_consent_screen, cs.consent_screen_text,
                cs.include_in_token_scope, cs.gui_order, cs.icon_uri, cs.attributes,
                cs.enabled, cs.created_at, cs.updated_at,
                'default' AS assignment_type
            FROM client_scopes cs
            INNER JOIN client_default_scopes cds ON cs.id = cds.scope_id
            WHERE cds.client_id = $1 AND cs.enabled = TRUE

            UNION ALL

            SELECT
                cs.id, cs.realm_id, cs.name, cs.display_name, cs.description, cs.protocol,
                cs.consent_required, cs.display_on_consent_screen, cs.consent_screen_text,
                cs.include_in_token_scope, cs.gui_order, cs.icon_uri, cs.attributes,
                cs.enabled, cs.created_at, cs.updated_at,
                'optional' AS assignment_type
            FROM client_scopes cs
            INNER JOIN client_optional_scopes cos ON cs.id = cos.scope_id
            WHERE cos.client_id = $1 AND cs.enabled = TRUE

            ORDER BY gui_order ASC, name ASC
        "#;

        let rows: Vec<tokio_postgres::Row> = db.query(query, &[&client_id]).await?;
        let mut assignments = Vec::new();

        for row in rows {
            let scope: ClientScope = ClientScope {
                id: row.get("id"),
                realm_id: row.get("realm_id"),
                name: row.get("name"),
                display_name: row.get("display_name"),
                description: row.get("description"),
                protocol: row.get("protocol"),
                consent_required: row.get("consent_required"),
                display_on_consent_screen: row.get("display_on_consent_screen"),
                consent_screen_text: row.get("consent_screen_text"),
                include_in_token_scope: row.get("include_in_token_scope"),
                gui_order: row.get("gui_order"),
                icon_uri: row.get("icon_uri"),
                attributes: row.get("attributes"),
                enabled: row.get("enabled"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };

            let assignment_type_str: String = row.get("assignment_type");
            let assignment_type = if assignment_type_str == "default" {
                ScopeAssignmentType::Default
            } else {
                ScopeAssignmentType::Optional
            };

            assignments.push(ClientScopeAssignment {
                scope: scope.into(),
                assignment_type,
            });
        }

        Ok(assignments)
    }

    /// Check if client has a specific scope (default or optional)
    pub async fn client_has_scope(
        db: &Database,
        client_id: Uuid,
        scope_name: &str,
    ) -> Result<bool> {
        let query = r#"
            SELECT EXISTS (
                SELECT 1 FROM client_scopes cs
                INNER JOIN client_default_scopes cds ON cs.id = cds.scope_id
                WHERE cds.client_id = $1 AND cs.name = $2 AND cs.enabled = TRUE

                UNION

                SELECT 1 FROM client_scopes cs
                INNER JOIN client_optional_scopes cos ON cs.id = cos.scope_id
                WHERE cos.client_id = $1 AND cs.name = $2 AND cs.enabled = TRUE
            ) AS has_scope
        "#;

        let row: tokio_postgres::Row = db.query_one(query, &[&client_id, &scope_name]).await?;
        Ok(row.get::<_, bool>("has_scope"))
    }
}

/// Database operations for user consent scopes
pub mod user_consent_scopes {
    use super::*;

    /// Grant consent for scopes
    pub async fn grant_consent(
        db: &Database,
        user_id: Uuid,
        client_id: Uuid,
        request: GrantScopeConsentRequest,
    ) -> Result<Vec<UserConsentScope>> {
        let now = Utc::now();
        let expires_at = request
            .expires_in
            .map(|secs| now + chrono::Duration::seconds(secs));
        let consent_source = request
            .consent_source
            .unwrap_or_else(|| "explicit".to_string());

        let query = r#"
            INSERT INTO user_consent_scopes (
                id, user_id, client_id, scope_id, granted_at, expires_at, consent_source
            )
            SELECT
                gen_random_uuid(), $1, $2, unnest($3::uuid[]), $4, $5, $6
            ON CONFLICT (user_id, client_id, scope_id)
            DO UPDATE SET
                granted_at = EXCLUDED.granted_at,
                expires_at = EXCLUDED.expires_at,
                consent_source = EXCLUDED.consent_source
            RETURNING id, user_id, client_id, scope_id, granted_at, expires_at, consent_source
        "#;

        let rows: Vec<tokio_postgres::Row> = db
            .query(
                query,
                &[
                    &user_id,
                    &client_id,
                    &request.scope_ids,
                    &now,
                    &expires_at,
                    &consent_source,
                ],
            )
            .await?;

        let mut consents = Vec::new();
        for row in rows {
            consents.push(UserConsentScope {
                id: row.get("id"),
                user_id: row.get("user_id"),
                client_id: row.get("client_id"),
                scope_id: row.get("scope_id"),
                granted_at: row.get("granted_at"),
                expires_at: row.get("expires_at"),
                consent_source: row.get("consent_source"),
            });
        }

        Ok(consents)
    }

    /// Get consented scopes for user and client
    pub async fn get_consented_scopes(
        db: &Database,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<Vec<ClientScope>> {
        let query = r#"
            SELECT
                cs.id, cs.realm_id, cs.name, cs.display_name, cs.description, cs.protocol,
                cs.consent_required, cs.display_on_consent_screen, cs.consent_screen_text,
                cs.include_in_token_scope, cs.gui_order, cs.icon_uri, cs.attributes,
                cs.enabled, cs.created_at, cs.updated_at
            FROM client_scopes cs
            INNER JOIN user_consent_scopes ucs ON cs.id = ucs.scope_id
            WHERE ucs.user_id = $1
              AND ucs.client_id = $2
              AND (ucs.expires_at IS NULL OR ucs.expires_at > NOW())
            ORDER BY cs.gui_order ASC, cs.name ASC
        "#;

        let rows: Vec<tokio_postgres::Row> = db.query(query, &[&user_id, &client_id]).await?;
        let mut scopes = Vec::new();

        for row in rows {
            scopes.push(row.try_into()?);
        }

        Ok(scopes)
    }

    /// Revoke consent for specific scopes
    pub async fn revoke_consent(
        db: &Database,
        user_id: Uuid,
        client_id: Uuid,
        scope_ids: Option<&[Uuid]>,
    ) -> Result<u64> {
        let rows_affected = if let Some(scope_ids) = scope_ids {
            let query = "DELETE FROM user_consent_scopes WHERE user_id = $1 AND client_id = $2 AND scope_id = ANY($3)";
            db.execute(query, &[&user_id, &client_id, &scope_ids])
                .await?
        } else {
            let query = "DELETE FROM user_consent_scopes WHERE user_id = $1 AND client_id = $2";
            db.execute(query, &[&user_id, &client_id]).await?
        };

        Ok(rows_affected)
    }

    /// Check if user has consented to all requested scopes
    pub async fn check_consent(
        db: &Database,
        user_id: Uuid,
        client_id: Uuid,
        requested_scope_names: &[String],
    ) -> Result<ConsentCheckResult> {
        // Get client scopes that require consent
        let scopes_query = r#"
            SELECT
                cs.id, cs.realm_id, cs.name, cs.display_name, cs.description, cs.protocol,
                cs.consent_required, cs.display_on_consent_screen, cs.consent_screen_text,
                cs.include_in_token_scope, cs.gui_order, cs.icon_uri, cs.attributes,
                cs.enabled, cs.created_at, cs.updated_at
            FROM client_scopes cs
            WHERE cs.name = ANY($1)
              AND cs.consent_required = TRUE
              AND cs.enabled = TRUE
        "#;

        let scope_rows: Vec<tokio_postgres::Row> =
            db.query(scopes_query, &[&requested_scope_names]).await?;

        let mut scopes_requiring_consent = Vec::new();
        let mut scope_names_requiring_consent = Vec::new();

        for row in scope_rows {
            let scope: ClientScope = row.try_into()?;
            scope_names_requiring_consent.push(scope.name.clone());
            scopes_requiring_consent.push(scope.into());
        }

        if scopes_requiring_consent.is_empty() {
            // No scopes require consent
            return Ok(ConsentCheckResult {
                consent_needed: false,
                scopes_requiring_consent: Vec::new(),
                consented_scopes: Vec::new(),
                missing_consent_scopes: Vec::new(),
            });
        }

        // Get consented scopes
        let consent_query = r#"
            SELECT cs.name
            FROM client_scopes cs
            INNER JOIN user_consent_scopes ucs ON cs.id = ucs.scope_id
            WHERE ucs.user_id = $1
              AND ucs.client_id = $2
              AND cs.name = ANY($3)
              AND (ucs.expires_at IS NULL OR ucs.expires_at > NOW())
        "#;

        let consent_rows: Vec<tokio_postgres::Row> = db
            .query(
                consent_query,
                &[&user_id, &client_id, &scope_names_requiring_consent],
            )
            .await?;

        let consented_scopes: Vec<String> = consent_rows
            .iter()
            .map(|row| row.get::<_, String>("name"))
            .collect();

        // Find missing consents
        let missing_consent_scopes: Vec<String> = scope_names_requiring_consent
            .into_iter()
            .filter(|scope| !consented_scopes.contains(scope))
            .collect();

        let consent_needed = !missing_consent_scopes.is_empty();

        Ok(ConsentCheckResult {
            consent_needed,
            scopes_requiring_consent: if consent_needed {
                scopes_requiring_consent
            } else {
                Vec::new()
            },
            consented_scopes,
            missing_consent_scopes,
        })
    }
}

/// Scope validation operations
pub mod scope_validation {
    use super::*;

    /// Validate requested scopes against client's allowed scopes
    pub async fn validate_scopes(
        db: &Database,
        client_id: Uuid,
        realm_id: Uuid,
        requested_scope_names: &[String],
    ) -> Result<ScopeValidationResult> {
        // Get all allowed scopes for client
        let allowed_scopes = client_scope_assignments::get_all_client_scopes(db, client_id).await?;
        let allowed_scope_names: Vec<String> = allowed_scopes
            .iter()
            .map(|a| a.scope.name.clone())
            .collect();

        // Separate valid and invalid scopes
        let mut valid_scope_names = Vec::new();
        let mut invalid_scopes = Vec::new();

        for scope_name in requested_scope_names {
            if allowed_scope_names.contains(scope_name) {
                valid_scope_names.push(scope_name.clone());
            } else {
                invalid_scopes.push(scope_name.clone());
            }
        }

        // Fetch full scope objects for valid scopes
        let valid_scopes = if !valid_scope_names.is_empty() {
            client_scopes::get_scopes_by_names(db, realm_id, &valid_scope_names).await?
        } else {
            Vec::new()
        };

        Ok(ScopeValidationResult {
            valid: invalid_scopes.is_empty(),
            invalid_scopes,
            valid_scopes,
        })
    }
}
