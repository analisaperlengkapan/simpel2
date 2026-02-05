/// Database operations for device management
use crate::{
    database::Database,
    error::{AuthencError, Result},
    models::{Device, DeviceInfo},
};
use chrono::Utc;
use log::error;
use uuid::Uuid;

pub async fn register_device(
    db: &Database,
    user_id: Uuid,
    device_info: &DeviceInfo,
) -> Result<Device> {
    let device_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO devices (
            id, user_id, device_name, device_fingerprint, trust_score,
            os, os_version, browser, browser_version, ip_address,
            user_agent, security_features, first_seen_at, last_seen_at, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        RETURNING
            id, user_id, device_name, device_fingerprint, trust_score,
            risk_level, os, os_version, browser, browser_version,
            ip_address, user_agent, security_features, last_seen_at,
            first_seen_at, created_at, updated_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &device_id,
                &user_id,
                &device_info.device_name,
                &device_info.fingerprint,
                &0.5f64, // Initial trust score
                &device_info.os,
                &device_info.os_version,
                &device_info.browser,
                &device_info.browser_version,
                &device_info.ip_address,
                &device_info.user_agent,
                &device_info.security_features,
                &now,
                &now,
                &now,
                &now,
            ],
        )
        .await
        .map_err(|e| {
            error!("Device registration query failed: {}", e);
            AuthencError::database(format!("Database query failed: {}", e))
        })?;

    // Convert row to Device
    let security_features: Option<serde_json::Value> = row.try_get("security_features").ok();
    let mut device: Device = row.try_into()?;
    device.security_features = security_features;
    Ok(device)
}

pub async fn get_device_by_id(db: &Database, device_id: Uuid) -> Result<Option<Device>> {
    let query = r#"
        SELECT
            id, user_id, device_name, device_fingerprint, trust_score,
            risk_level, os, os_version, browser, browser_version,
            ip_address, user_agent, security_features, last_seen_at,
            first_seen_at, created_at, updated_at
        FROM devices
        WHERE id = $1
    "#;

    match db.query_opt(query, &[&device_id]).await? {
        Some(row) => {
            let security_features: Option<serde_json::Value> = row.try_get("security_features").ok();
            let mut device: Device = row.try_into()?;
            device.security_features = security_features;
            Ok(Some(device))
        }
        None => Ok(None),
    }
}

pub async fn update_device_info(
    db: &Database,
    device_id: Uuid,
    device_name: Option<String>,
    security_features: Option<serde_json::Value>,
) -> Result<()> {
    let now = Utc::now();
    let query = r#"
        UPDATE devices
        SET
            device_name = COALESCE($2, device_name),
            security_features = COALESCE($3, security_features),
            updated_at = $4
        WHERE id = $1
    "#;
    db.execute(query, &[&device_id, &device_name, &security_features, &now])
        .await?;
    Ok(())
}

pub async fn update_trust_score(
    db: &Database,
    device_id: Uuid,
    new_score: f64,
    factors: serde_json::Value,
) -> Result<()> {
    let now = Utc::now();

    // First, get the current score for history
    let current_query = "SELECT trust_score FROM devices WHERE id = $1";
    let current_row: tokio_postgres::Row = db.query_one(current_query, &[&device_id]).await?;
    let current_score: f64 = current_row.get(0);

    // Update the device trust score
    let update_query = r#"
        UPDATE devices
        SET trust_score = $2, updated_at = $3
        WHERE id = $1
    "#;
    db.execute(update_query, &[&device_id, &new_score, &now])
        .await?;

    // Insert trust score history
    let history_query = r#"
        INSERT INTO device_trust_history (
            device_id, previous_score, new_score, factors, changed_at
        )
        VALUES ($1, $2, $3, $4, $5)
    "#;
    db.execute(
        history_query,
        &[
            &device_id,
            &current_score,
            &new_score,
            &serde_json::to_string(&factors).unwrap_or_default(),
            &now,
        ],
    )
    .await?;

    Ok(())
}

pub async fn update_last_seen(db: &Database, device_id: Uuid) -> Result<()> {
    let now = Utc::now();
    let query = r#"
        UPDATE devices
        SET last_seen_at = $2, updated_at = $2
        WHERE id = $1
    "#;
    db.execute(query, &[&device_id, &now]).await?;
    Ok(())
}

pub async fn list_user_devices(db: &Database, user_id: Uuid) -> Result<Vec<Device>> {
    let query = r#"
        SELECT
            id, user_id, device_name, device_fingerprint, trust_score,
            risk_level, os, os_version, browser, browser_version,
            ip_address, user_agent, location_data, security_features, last_seen_at,
            first_seen_at, created_at, updated_at
        FROM devices
        WHERE user_id = $1
        ORDER BY last_seen_at DESC
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&user_id]).await?;
    // Convert rows to Vec<Device>
    rows.into_iter()
        .map(|row| {
            let security_features: Option<serde_json::Value> = row.try_get("security_features").ok();
            let mut device: Device = row.try_into()?;
            device.security_features = security_features;
            Ok(device)
        })
        .collect::<Result<Vec<Device>>>()
}

pub async fn delete_device(db: &Database, device_id: Uuid) -> Result<()> {
    let query = "DELETE FROM devices WHERE id = $1";
    db.execute(query, &[&device_id]).await?;
    Ok(())
}
