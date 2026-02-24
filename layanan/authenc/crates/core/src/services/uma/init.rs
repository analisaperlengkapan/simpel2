//! UMA 2.0 Database Initialization
//!
//! Initializes UMA 2.0 tables and indexes programmatically.

use authenc_storage::Database;
use authenc_types::{AuthencError, Result};
use std::sync::Arc;

/// Initialize UMA 2.0 tables
pub async fn init_uma_tables(database: &Arc<Database>) -> Result<()> {
    tracing::info!("Initializing UMA 2.0 database tables...");

    // Create uma_policies table
    database
        .execute(
            r#"
            CREATE TABLE IF NOT EXISTS uma_policies (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                name VARCHAR(255) NOT NULL,
                description TEXT,
                policy_type VARCHAR(50) NOT NULL CHECK (policy_type IN ('role', 'user', 'group', 'time', 'attribute', 'javascript', 'aggregate', 'client')),
                logic VARCHAR(20) NOT NULL DEFAULT 'POSITIVE' CHECK (logic IN ('POSITIVE', 'NEGATIVE')),
                decision_strategy VARCHAR(20) NOT NULL DEFAULT 'UNANIMOUS' CHECK (decision_strategy IN ('UNANIMOUS', 'AFFIRMATIVE', 'CONSENSUS')),
                config JSONB NOT NULL DEFAULT '{}',
                enabled BOOLEAN NOT NULL DEFAULT true,
                realm_id UUID NOT NULL,
                resource_server_id UUID NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
            &[],
        )
        .await
        .map_err(|e| AuthencError::database(format!("Failed to create uma_policies table: {}", e)))?;

    // Create indexes for uma_policies
    let policy_indexes = vec![
        "CREATE INDEX IF NOT EXISTS idx_uma_policies_realm_id ON uma_policies(realm_id)",
        "CREATE INDEX IF NOT EXISTS idx_uma_policies_resource_server_id ON uma_policies(resource_server_id)",
        "CREATE INDEX IF NOT EXISTS idx_uma_policies_policy_type ON uma_policies(policy_type)",
        "CREATE INDEX IF NOT EXISTS idx_uma_policies_enabled ON uma_policies(enabled)",
        "CREATE INDEX IF NOT EXISTS idx_uma_policies_config ON uma_policies USING GIN(config)",
    ];

    for index_query in policy_indexes {
        database
            .execute(index_query, &[])
            .await
            .map_err(|e| AuthencError::database(format!("Failed to create index: {}", e)))?;
    }

    // Create uma_policy_resources table
    database
        .execute(
            r#"
            CREATE TABLE IF NOT EXISTS uma_policy_resources (
                policy_id UUID NOT NULL,
                resource_id UUID NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (policy_id, resource_id)
            )
            "#,
            &[],
        )
        .await
        .map_err(|e| {
            AuthencError::database(format!(
                "Failed to create uma_policy_resources table: {}",
                e
            ))
        })?;

    database
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_uma_policy_resources_resource_id ON uma_policy_resources(resource_id)",
            &[],
        )
        .await
        .map_err(|e| AuthencError::database(format!("Failed to create index: {}", e)))?;

    // Create uma_policy_scopes table
    database
        .execute(
            r#"
            CREATE TABLE IF NOT EXISTS uma_policy_scopes (
                policy_id UUID NOT NULL,
                scope_id UUID NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (policy_id, scope_id)
            )
            "#,
            &[],
        )
        .await
        .map_err(|e| {
            AuthencError::database(format!("Failed to create uma_policy_scopes table: {}", e))
        })?;

    database
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_uma_policy_scopes_scope_id ON uma_policy_scopes(scope_id)",
            &[],
        )
        .await
        .map_err(|e| AuthencError::database(format!("Failed to create index: {}", e)))?;

    // Create uma_policy_clients table
    database
        .execute(
            r#"
            CREATE TABLE IF NOT EXISTS uma_policy_clients (
                policy_id UUID NOT NULL,
                client_id VARCHAR(255) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (policy_id, client_id)
            )
            "#,
            &[],
        )
        .await
        .map_err(|e| {
            AuthencError::database(format!("Failed to create uma_policy_clients table: {}", e))
        })?;

    database
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_uma_policy_clients_client_id ON uma_policy_clients(client_id)",
            &[],
        )
        .await
        .map_err(|e| AuthencError::database(format!("Failed to create index: {}", e)))?;

    // Create uma_delegation_policies table
    database
        .execute(
            r#"
            CREATE TABLE IF NOT EXISTS uma_delegation_policies (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                owner_id VARCHAR(255) NOT NULL,
                resource_id UUID,
                scopes TEXT[] NOT NULL,
                delegates TEXT[] NOT NULL,
                conditions JSONB NOT NULL DEFAULT '{}',
                enabled BOOLEAN NOT NULL DEFAULT true,
                valid_from TIMESTAMPTZ,
                valid_until TIMESTAMPTZ,
                realm_id UUID NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
            &[],
        )
        .await
        .map_err(|e| {
            AuthencError::database(format!(
                "Failed to create uma_delegation_policies table: {}",
                e
            ))
        })?;

    // Create indexes for uma_delegation_policies
    let delegation_indexes = vec![
        "CREATE INDEX IF NOT EXISTS idx_uma_delegation_policies_owner_id ON uma_delegation_policies(owner_id)",
        "CREATE INDEX IF NOT EXISTS idx_uma_delegation_policies_resource_id ON uma_delegation_policies(resource_id)",
        "CREATE INDEX IF NOT EXISTS idx_uma_delegation_policies_realm_id ON uma_delegation_policies(realm_id)",
        "CREATE INDEX IF NOT EXISTS idx_uma_delegation_policies_enabled ON uma_delegation_policies(enabled)",
        "CREATE INDEX IF NOT EXISTS idx_uma_delegation_policies_valid_until ON uma_delegation_policies(valid_until)",
        "CREATE INDEX IF NOT EXISTS idx_uma_delegation_policies_delegates ON uma_delegation_policies USING GIN(delegates)",
        "CREATE INDEX IF NOT EXISTS idx_uma_delegation_policies_scopes ON uma_delegation_policies USING GIN(scopes)",
    ];

    for index_query in delegation_indexes {
        database
            .execute(index_query, &[])
            .await
            .map_err(|e| AuthencError::database(format!("Failed to create index: {}", e)))?;
    }

    // Create uma_claims_gathering_state table
    database
        .execute(
            r#"
            CREATE TABLE IF NOT EXISTS uma_claims_gathering_state (
                state_id VARCHAR(255) PRIMARY KEY,
                ticket VARCHAR(255) NOT NULL,
                required_claims JSONB NOT NULL,
                collected_claims JSONB NOT NULL DEFAULT '{}',
                subject_id VARCHAR(255) NOT NULL,
                client_id VARCHAR(255) NOT NULL,
                realm_id UUID NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
            &[],
        )
        .await
        .map_err(|e| {
            AuthencError::database(format!(
                "Failed to create uma_claims_gathering_state table: {}",
                e
            ))
        })?;

    // Create indexes for uma_claims_gathering_state
    let claims_indexes = vec![
        "CREATE INDEX IF NOT EXISTS idx_uma_claims_gathering_state_realm_id ON uma_claims_gathering_state(realm_id)",
        "CREATE INDEX IF NOT EXISTS idx_uma_claims_gathering_state_subject_id ON uma_claims_gathering_state(subject_id)",
        "CREATE INDEX IF NOT EXISTS idx_uma_claims_gathering_state_client_id ON uma_claims_gathering_state(client_id)",
        "CREATE INDEX IF NOT EXISTS idx_uma_claims_gathering_state_expires_at ON uma_claims_gathering_state(expires_at)",
        "CREATE INDEX IF NOT EXISTS idx_uma_claims_gathering_state_ticket ON uma_claims_gathering_state(ticket)",
    ];

    for index_query in claims_indexes {
        database
            .execute(index_query, &[])
            .await
            .map_err(|e| AuthencError::database(format!("Failed to create index: {}", e)))?;
    }

    // Create uma_rpt_tokens table
    database
        .execute(
            r#"
            CREATE TABLE IF NOT EXISTS uma_rpt_tokens (
                jti VARCHAR(255) PRIMARY KEY,
                subject_id VARCHAR(255) NOT NULL,
                client_id VARCHAR(255) NOT NULL,
                permissions JSONB NOT NULL,
                realm_id UUID NOT NULL,
                issued_at TIMESTAMPTZ NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                revoked BOOLEAN NOT NULL DEFAULT false,
                revoked_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
            &[],
        )
        .await
        .map_err(|e| {
            AuthencError::database(format!("Failed to create uma_rpt_tokens table: {}", e))
        })?;

    // Create indexes for uma_rpt_tokens
    let rpt_indexes = vec![
        "CREATE INDEX IF NOT EXISTS idx_uma_rpt_tokens_subject_id ON uma_rpt_tokens(subject_id)",
        "CREATE INDEX IF NOT EXISTS idx_uma_rpt_tokens_client_id ON uma_rpt_tokens(client_id)",
        "CREATE INDEX IF NOT EXISTS idx_uma_rpt_tokens_realm_id ON uma_rpt_tokens(realm_id)",
        "CREATE INDEX IF NOT EXISTS idx_uma_rpt_tokens_expires_at ON uma_rpt_tokens(expires_at)",
        "CREATE INDEX IF NOT EXISTS idx_uma_rpt_tokens_revoked ON uma_rpt_tokens(revoked)",
    ];

    for index_query in rpt_indexes {
        database
            .execute(index_query, &[])
            .await
            .map_err(|e| AuthencError::database(format!("Failed to create index: {}", e)))?;
    }

    // Create cleanup functions
    database
        .execute(
            r#"
            CREATE OR REPLACE FUNCTION cleanup_expired_claims_gathering_state()
            RETURNS INTEGER AS $$
            DECLARE
                deleted_count INTEGER;
            BEGIN
                DELETE FROM uma_claims_gathering_state
                WHERE expires_at < NOW();

                GET DIAGNOSTICS deleted_count = ROW_COUNT;
                RETURN deleted_count;
            END;
            $$ LANGUAGE plpgsql
            "#,
            &[],
        )
        .await
        .map_err(|e| AuthencError::database(format!("Failed to create cleanup function: {}", e)))?;

    database
        .execute(
            r#"
            CREATE OR REPLACE FUNCTION cleanup_expired_rpt_tokens()
            RETURNS INTEGER AS $$
            DECLARE
                deleted_count INTEGER;
            BEGIN
                DELETE FROM uma_rpt_tokens
                WHERE expires_at < NOW()
                AND revoked = false;

                GET DIAGNOSTICS deleted_count = ROW_COUNT;
                RETURN deleted_count;
            END;
            $$ LANGUAGE plpgsql
            "#,
            &[],
        )
        .await
        .map_err(|e| AuthencError::database(format!("Failed to create cleanup function: {}", e)))?;

    tracing::info!("✅ UMA 2.0 database tables initialized successfully");
    Ok(())
}
