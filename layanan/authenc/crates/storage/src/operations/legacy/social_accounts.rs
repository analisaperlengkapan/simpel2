/// Database operations for social accounts
use crate::Database;
use authenc_types::Result;
use authenc_types::domain::social_account::{
    CreateSocialAccountRequest, SocialAccount, SocialProvider,
};
use chrono::Utc;
use uuid::Uuid;

/// Parse a SocialProvider from a database string value
fn parse_provider(s: &str) -> SocialProvider {
    match s {
        "Google" => SocialProvider::Google,
        "GitHub" => SocialProvider::GitHub,
        "Microsoft" => SocialProvider::Microsoft,
        "Facebook" => SocialProvider::Facebook,
        "Twitter" => SocialProvider::Twitter,
        "LinkedIn" => SocialProvider::LinkedIn,
        other => SocialProvider::Custom(other.to_string()),
    }
}

/// Convert a SocialProvider to a database string value
fn provider_to_str(p: &SocialProvider) -> String {
    match p {
        SocialProvider::Google => "Google".to_string(),
        SocialProvider::GitHub => "GitHub".to_string(),
        SocialProvider::Microsoft => "Microsoft".to_string(),
        SocialProvider::Facebook => "Facebook".to_string(),
        SocialProvider::Twitter => "Twitter".to_string(),
        SocialProvider::LinkedIn => "LinkedIn".to_string(),
        SocialProvider::Custom(s) => s.clone(),
    }
}

pub async fn get_social_account(db: &Database, account_id: Uuid) -> Result<Option<SocialAccount>> {
    let query = r#"
        SELECT id, user_id, provider, provider_user_id, display_name, email,
               profile_picture_url, access_token, refresh_token, token_expires_at,
               linked_at, updated_at
        FROM user_social_accounts
        WHERE id = $1 AND deleted_at IS NULL
    "#;

    let row = db.query_opt(query, &[&account_id]).await?;
    match row {
        Some(row) => {
            let provider_str: String = row.get(2);
            let provider = parse_provider(&provider_str);

            Ok(Some(SocialAccount {
                id: row.get(0),
                user_id: row.get(1),
                provider,
                provider_user_id: row.get(3),
                display_name: row.get(4),
                email: row.get(5),
                profile_picture_url: row.get(6),
                access_token: row.get(7),
                refresh_token: row.get(8),
                token_expires_at: row.get(9),
                linked_at: row.get(10),
                updated_at: row.get(11),
            }))
        }
        None => Ok(None),
    }
}

pub async fn get_user_social_accounts(db: &Database, user_id: Uuid) -> Result<Vec<SocialAccount>> {
    let query = r#"
        SELECT id, user_id, provider, provider_user_id, display_name, email,
               profile_picture_url, access_token, refresh_token, token_expires_at,
               linked_at, updated_at
        FROM user_social_accounts
        WHERE user_id = $1 AND deleted_at IS NULL
        ORDER BY linked_at DESC
    "#;

    let rows = db.query(query, &[&user_id]).await?;
    let accounts = rows
        .into_iter()
        .map(|row: tokio_postgres::Row| {
            let provider_str: String = row.get(2);
            let provider = parse_provider(&provider_str);

            SocialAccount {
                id: row.get(0),
                user_id: row.get(1),
                provider,
                provider_user_id: row.get(3),
                display_name: row.get(4),
                email: row.get(5),
                profile_picture_url: row.get(6),
                access_token: row.get(7),
                refresh_token: row.get(8),
                token_expires_at: row.get(9),
                linked_at: row.get(10),
                updated_at: row.get(11),
            }
        })
        .collect();

    Ok(accounts)
}

pub async fn get_social_account_by_provider(
    db: &Database,
    provider: &str,
    provider_user_id: &str,
) -> Result<Option<SocialAccount>> {
    let query = r#"
        SELECT id, user_id, provider, provider_user_id, display_name, email,
               profile_picture_url, access_token, refresh_token, token_expires_at,
               linked_at, updated_at
        FROM user_social_accounts
        WHERE provider = $1 AND provider_user_id = $2 AND deleted_at IS NULL
    "#;

    let row = db.query_opt(query, &[&provider, &provider_user_id]).await?;
    match row {
        Some(row) => {
            let provider_str: String = row.get(2);
            let provider = parse_provider(&provider_str);

            Ok(Some(SocialAccount {
                id: row.get(0),
                user_id: row.get(1),
                provider,
                provider_user_id: row.get(3),
                display_name: row.get(4),
                email: row.get(5),
                profile_picture_url: row.get(6),
                access_token: row.get(7),
                refresh_token: row.get(8),
                token_expires_at: row.get(9),
                linked_at: row.get(10),
                updated_at: row.get(11),
            }))
        }
        None => Ok(None),
    }
}

pub async fn has_social_account(db: &Database, user_id: Uuid, provider: &str) -> Result<bool> {
    let query = r#"
        SELECT COUNT(*) FROM user_social_accounts
        WHERE user_id = $1 AND provider = $2 AND deleted_at IS NULL
    "#;

    let row: tokio_postgres::Row = db.query_one(query, &[&user_id, &provider]).await?;
    let count: i64 = row.get(0);

    Ok(count > 0)
}

pub async fn add_social_account(
    db: &Database,
    user_id: Uuid,
    request: CreateSocialAccountRequest,
) -> Result<SocialAccount> {
    let account_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO user_social_accounts (
            id, user_id, provider, provider_user_id, display_name, email,
            profile_picture_url, access_token, refresh_token, token_expires_at,
            linked_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id
    "#;

    db.execute(
        query,
        &[
            &account_id,
            &user_id,
            &provider_to_str(&request.provider),
            &request.provider_user_id,
            &request.display_name,
            &request.email,
            &request.profile_picture_url,
            &request.access_token,
            &request.refresh_token,
            &request.token_expires_at,
            &now,
            &now,
        ],
    )
    .await?;

    Ok(SocialAccount {
        id: account_id,
        user_id,
        provider: request.provider,
        provider_user_id: request.provider_user_id,
        display_name: request.display_name,
        email: request.email,
        profile_picture_url: request.profile_picture_url,
        access_token: request.access_token,
        refresh_token: request.refresh_token,
        token_expires_at: request.token_expires_at,
        linked_at: now,
        updated_at: now,
    })
}

pub async fn update_social_account(
    db: &Database,
    account_id: Uuid,
    request: CreateSocialAccountRequest,
) -> Result<SocialAccount> {
    let now = Utc::now();

    let query = r#"
        UPDATE user_social_accounts SET
            provider = $2, provider_user_id = $3, display_name = $4, email = $5,
            profile_picture_url = $6, access_token = $7, refresh_token = $8,
            token_expires_at = $9, updated_at = $10
        WHERE id = $1
    "#;

    db.execute(
        query,
        &[
            &account_id,
            &provider_to_str(&request.provider),
            &request.provider_user_id,
            &request.display_name,
            &request.email,
            &request.profile_picture_url,
            &request.access_token,
            &request.refresh_token,
            &request.token_expires_at,
            &now,
        ],
    )
    .await?;

    // Query the updated row to get the correct user_id and other fields
    let select_query = r#"
        SELECT id, user_id, provider, provider_user_id, display_name, email,
               profile_picture_url, access_token, refresh_token, token_expires_at,
               linked_at, updated_at
        FROM user_social_accounts
        WHERE id = $1
    "#;
    let row: tokio_postgres::Row = db.query_one(select_query, &[&account_id]).await?;

    Ok(SocialAccount {
        id: row.get("id"),
        user_id: row.get("user_id"),
        provider: parse_provider(row.get::<_, &str>("provider")),
        provider_user_id: row.get("provider_user_id"),
        display_name: row.get("display_name"),
        email: row.get("email"),
        profile_picture_url: row.get("profile_picture_url"),
        access_token: row.get("access_token"),
        refresh_token: row.get("refresh_token"),
        token_expires_at: row.get("token_expires_at"),
        linked_at: row.get("linked_at"),
        updated_at: row.get("updated_at"),
    })
}

pub async fn remove_social_account(db: &Database, account_id: Uuid) -> Result<()> {
    let now = Utc::now();

    let query = "UPDATE user_social_accounts SET deleted_at = $2 WHERE id = $1";
    db.execute(query, &[&account_id, &now]).await?;

    Ok(())
}

pub async fn remove_social_account_by_provider(
    db: &Database,
    user_id: Uuid,
    provider: &str,
) -> Result<()> {
    let now = Utc::now();

    let query = r#"
        UPDATE user_social_accounts
        SET deleted_at = $3
        WHERE user_id = $1 AND provider = $2 AND deleted_at IS NULL
    "#;

    db.execute(query, &[&user_id, &provider, &now]).await?;

    Ok(())
}
