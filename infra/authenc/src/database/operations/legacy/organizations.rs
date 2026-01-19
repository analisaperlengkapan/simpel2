use crate::{
    database::Database,
    error::{AuthencError, Result},
    models::{Organization, OrganizationInvitation, OrganizationMember},
    services::organization::OrganizationRole,
};
use chrono::Utc;
use log::error;
use std::str::FromStr;
use uuid::Uuid;

pub async fn create_organization(db: &Database, org: &Organization) -> Result<Organization> {
    let org_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO organizations (
            id, name, display_name, description, domain,
            logo_url, website_url, owner_id, realm_id,
            enabled, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING
            id, name, display_name, description, domain,
            logo_url, website_url, owner_id, realm_id,
            enabled, created_at, updated_at, deleted_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &org_id,
                &org.name,
                &org.display_name,
                &org.description,
                &org.domain,
                &org.logo_url,
                &org.website_url,
                &org.owner_id,
                &org.realm_id,
                &org.enabled,
                &now,
                &now,
            ],
        )
        .await?;

    // Convert row to Organization
    row.try_into()
}

pub async fn get_organization_by_id(db: &Database, org_id: Uuid) -> Result<Option<Organization>> {
    let query = r#"
        SELECT
            id, name, display_name, description, domain,
            logo_url, website_url, owner_id, realm_id,
            enabled, created_at, updated_at, deleted_at
        FROM organizations
        WHERE id = $1 AND deleted_at IS NULL
    "#;

    let row: tokio_postgres::Row = db.query_one(query, &[&org_id]).await?;
    // Convert row to Organization
    Ok(Some(row.try_into()?))
}

pub async fn add_member(
    db: &Database,
    org_id: Uuid,
    user_id: Uuid,
    role: &str,
    invited_by: Option<Uuid>,
) -> Result<()> {
    let member_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO organization_members (
            id, organization_id, user_id, role, invited_by,
            invited_at, joined_at, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    "#;

    db.execute(
        query,
        &[
            &member_id,
            &org_id,
            &user_id,
            &role,
            &invited_by,
            &now,
            &now,
            &now,
            &now,
        ],
    )
    .await?;

    Ok(())
}

pub async fn create_invitation(db: &Database, invitation: &OrganizationInvitation) -> Result<()> {
    let invitation_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO organization_invitations (
            id, organization_id, email, role, invited_by,
            token_hash, expires_at, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    "#;

    db.execute(
        query,
        &[
            &invitation_id,
            &invitation.organization_id,
            &invitation.email,
            &invitation.role,
            &invitation.invited_by,
            &invitation.token_hash,
            &invitation.expires_at,
            &now,
        ],
    )
    .await?;

    Ok(())
}

pub async fn get_invitation_by_token(
    db: &Database,
    token_hash: &str,
) -> Result<Option<OrganizationInvitation>> {
    let query = r#"
        SELECT
            id, organization_id, email, role, invited_by,
            token_hash, expires_at, accepted_at, accepted_by, created_at
        FROM organization_invitations
        WHERE token_hash = $1 AND expires_at > NOW() AND accepted_at IS NULL
    "#;

    let row: tokio_postgres::Row = db.query_one(query, &[&token_hash]).await?;
    // Convert row to OrganizationInvitation
    Ok(Some(row.try_into()?))
}

pub async fn accept_invitation(db: &Database, token_hash: &str, user_id: Uuid) -> Result<()> {
    let now = Utc::now();

    // First get the invitation
    let invitation = get_invitation_by_token(db, token_hash)
        .await?
        .ok_or_else(|| AuthencError::resource_not_found("Invitation not found or expired"))?;

    // Mark invitation as accepted
    let update_query = r#"
        UPDATE organization_invitations
        SET accepted_at = $2, accepted_by = $3
        WHERE token_hash = $1
    "#;
    db.execute(update_query, &[&token_hash, &now, &user_id])
        .await?;

    // Add user as organization member
    add_member(
        db,
        invitation.organization_id,
        user_id,
        &invitation.role,
        Some(invitation.invited_by),
    )
    .await?;

    Ok(())
}

pub async fn get_organization_by_domain(
    db: &Database,
    domain: &str,
) -> Result<Option<Organization>> {
    let query = r#"
        SELECT
            id, name, display_name, description, domain, logo_url, website,
            enabled, created_at, updated_at, attributes
        FROM organizations
        WHERE domain = $1
    "#;

    let row = db.query_opt(query, &[&domain]).await.map_err(|e| {
        error!("Failed to get organization by domain: {}", e);
        AuthencError::database("Failed to get organization by domain")
    })?;

    if let Some(row) = row {
        // Convert row to Organization
        Ok(Some(Organization {
            id: row.get(0),
            name: row.get(1),
            display_name: row.get(2),
            description: row.get(3),
            domain: row.get(4),
            logo_url: row.get(5),
            website_url: row.get(6),
            enabled: row.get(7),
            created_at: row.get(8),
            updated_at: row.get(9),
            owner_id: row.get(10),
            realm_id: row.get(11),
            deleted_at: row.get(12),
        }))
    } else {
        Ok(None)
    }
}

pub async fn update_organization(db: &Database, org: &Organization) -> Result<()> {
    let query = r#"
        UPDATE organizations
        SET name = $2, display_name = $3, description = $4, domain = $5,
            logo_url = $6, website_url = $7, enabled = $8, updated_at = $9
        WHERE id = $1
    "#;

    db.execute(
        query,
        &[
            &org.id,
            &org.name,
            &org.display_name,
            &org.description,
            &org.domain,
            &org.logo_url,
            &org.website_url,
            &org.enabled,
            &org.updated_at,
        ],
    )
    .await
    .map_err(|e| {
        error!("Failed to update organization: {}", e);
        AuthencError::database("Failed to update organization")
    })?;

    Ok(())
}

pub async fn delete_organization(db: &Database, organization_id: &Uuid) -> Result<()> {
    let query = "DELETE FROM organizations WHERE id = $1";

    db.execute(query, &[organization_id]).await.map_err(|e| {
        error!("Failed to delete organization: {}", e);
        AuthencError::database("Failed to delete organization")
    })?;

    Ok(())
}

pub async fn get_organization_members(
    db: &Database,
    organization_id: &Uuid,
) -> Result<Vec<OrganizationMember>> {
    let query = r#"
        SELECT om.user_id, om.organization_id, om.role, om.joined_at, om.invited_by
        FROM organization_members om
        WHERE om.organization_id = $1
        ORDER BY om.joined_at
    "#;

    let rows: Vec<tokio_postgres::Row> =
        db.query(query, &[organization_id]).await.map_err(|e| {
            error!("Failed to get organization members: {}", e);
            AuthencError::database("Failed to get organization members")
        })?;

    let mut members = Vec::new();
    for row in rows {
        members.push(OrganizationMember {
            id: row.get(0),
            organization_id: row.get(1),
            user_id: row.get(2),
            role: OrganizationRole::from_str(&row.get::<_, String>(3))
                .unwrap_or(OrganizationRole::Member)
                .as_str()
                .to_string(),
            invited_by: row.get(4),
            invited_at: row.get(5),
            joined_at: row.get(6),
            created_at: row.get(7),
            updated_at: row.get(8),
        });
    }

    Ok(members)
}

pub async fn get_user_organizations(db: &Database, user_id: &Uuid) -> Result<Vec<Organization>> {
    let query = r#"
        SELECT
            o.id, o.name, o.display_name, o.description, o.domain, o.logo_url, o.website_url,
            o.enabled, o.created_at, o.updated_at, o.owner_id, o.realm_id, o.deleted_at
        FROM organizations o
        JOIN organization_members om ON o.id = om.organization_id
        WHERE om.user_id = $1
        ORDER BY o.created_at
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[user_id]).await.map_err(|e| {
        error!("Failed to get user organizations: {}", e);
        AuthencError::database("Failed to get user organizations")
    })?;

    let mut organizations = Vec::new();
    for row in rows {
        organizations.push(Organization {
            id: row.get(0),
            name: row.get(1),
            display_name: row.get(2),
            description: row.get(3),
            domain: row.get(4),
            logo_url: row.get(5),
            website_url: row.get(6),
            enabled: row.get(7),
            created_at: row.get(8),
            updated_at: row.get(9),
            owner_id: row.get(10),
            realm_id: row.get(11),
            deleted_at: row.get(12),
        });
    }

    Ok(organizations)
}

pub async fn remove_organization_member(
    db: &Database,
    organization_id: &Uuid,
    user_id: &Uuid,
) -> Result<()> {
    let query = "DELETE FROM organization_members WHERE organization_id = $1 AND user_id = $2";

    db.execute(query, &[organization_id, user_id])
        .await
        .map_err(|e| {
            error!("Failed to remove organization member: {}", e);
            AuthencError::database("Failed to remove organization member")
        })?;

    Ok(())
}

pub async fn update_member_role(
    db: &Database,
    organization_id: &Uuid,
    user_id: &Uuid,
    role: OrganizationRole,
) -> Result<()> {
    let query = r#"
        UPDATE organization_members
        SET role = $3
        WHERE organization_id = $1 AND user_id = $2
    "#;

    db.execute(query, &[organization_id, user_id, &role.as_str()])
        .await
        .map_err(|e| {
            error!("Failed to update member role: {}", e);
            AuthencError::database("Failed to update member role")
        })?;

    Ok(())
}

/// Mewakili struktur data `OrganizationDomain`.
#[derive(Debug, Clone)]
/// Database operations for organizations
pub struct OrganizationDomain {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub domain: String,
    pub verified: bool,
    pub verification_token: Option<String>,
    pub verification_method: String,
    pub verified_at: Option<chrono::DateTime<Utc>>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

pub async fn add_domain(
    db: &Database,
    organization_id: Uuid,
    domain: &str,
    verification_method: &str,
) -> Result<OrganizationDomain> {
    let domain_id = Uuid::new_v4();
    let verification_token = Uuid::new_v4().to_string();
    let now = Utc::now();

    let query = r#"
        INSERT INTO organization_domains (
            id, organization_id, domain, verified, verification_token,
            verification_method, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, organization_id, domain, verified, verification_token,
                  verification_method, verified_at, created_at, updated_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &domain_id,
                &organization_id,
                &domain,
                &false,
                &verification_token,
                &verification_method,
                &now,
                &now,
            ],
        )
        .await
        .map_err(|e| {
            error!("Failed to add organization domain: {}", e);
            AuthencError::database("Failed to add organization domain")
        })?;

    Ok(OrganizationDomain {
        id: row.get(0),
        organization_id: row.get(1),
        domain: row.get(2),
        verified: row.get(3),
        verification_token: row.get(4),
        verification_method: row.get(5),
        verified_at: row.get(6),
        created_at: row.get(7),
        updated_at: row.get(8),
    })
}

pub async fn verify_domain(db: &Database, domain_id: Uuid) -> Result<()> {
    let now = Utc::now();

    let query = r#"
        UPDATE organization_domains
        SET verified = true, verified_at = $2, updated_at = $3
        WHERE id = $1
    "#;

    db.execute(query, &[&domain_id, &now, &now])
        .await
        .map_err(|e| {
            error!("Failed to verify domain: {}", e);
            AuthencError::database("Failed to verify domain")
        })?;

    Ok(())
}

pub async fn get_domains(db: &Database, organization_id: Uuid) -> Result<Vec<OrganizationDomain>> {
    let query = r#"
        SELECT id, organization_id, domain, verified, verification_token,
               verification_method, verified_at, created_at, updated_at
        FROM organization_domains
        WHERE organization_id = $1
        ORDER BY created_at DESC
    "#;

    let rows: Vec<tokio_postgres::Row> =
        db.query(query, &[&organization_id]).await.map_err(|e| {
            error!("Failed to get organization domains: {}", e);
            AuthencError::database("Failed to get organization domains")
        })?;

    let mut domains = Vec::new();
    for row in rows {
        domains.push(OrganizationDomain {
            id: row.get(0),
            organization_id: row.get(1),
            domain: row.get(2),
            verified: row.get(3),
            verification_token: row.get(4),
            verification_method: row.get(5),
            verified_at: row.get(6),
            created_at: row.get(7),
            updated_at: row.get(8),
        });
    }

    Ok(domains)
}

pub async fn link_identity_provider(
    db: &Database,
    organization_id: Uuid,
    identity_provider_id: Uuid,
    priority: i32,
) -> Result<()> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO organization_identity_providers (
            id, organization_id, identity_provider_id, priority, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (organization_id, identity_provider_id)
        DO UPDATE SET priority = $4, updated_at = $6
    "#;

    db.execute(
        query,
        &[
            &id,
            &organization_id,
            &identity_provider_id,
            &priority,
            &now,
            &now,
        ],
    )
    .await
    .map_err(|e| {
        error!("Failed to link identity provider: {}", e);
        AuthencError::database("Failed to link identity provider")
    })?;

    Ok(())
}

pub async fn unlink_identity_provider(
    db: &Database,
    organization_id: Uuid,
    identity_provider_id: Uuid,
) -> Result<()> {
    let query = r#"
        DELETE FROM organization_identity_providers
        WHERE organization_id = $1 AND identity_provider_id = $2
    "#;

    db.execute(query, &[&organization_id, &identity_provider_id])
        .await
        .map_err(|e| {
            error!("Failed to unlink identity provider: {}", e);
            AuthencError::database("Failed to unlink identity provider")
        })?;

    Ok(())
}
