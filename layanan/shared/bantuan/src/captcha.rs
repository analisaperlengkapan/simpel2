use crate::config::AppConfig;
use crate::error::AppError;
use axum::{extract::State, http::Request, middleware::Next, response::Response};
use chrono::Utc;
use deadpool_postgres::Pool;
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

pub async fn verify_captcha(secret: &str, token: &str) -> Result<bool, AppError> {
    let client = Client::new();
    let resp = client
        .post("https://www.google.com/recaptcha/api/siteverify")
        .form(&json!({"secret": secret, "response": token}))
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("CAPTCHA error: {}", e)))?;
    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::Validation(format!("CAPTCHA error: {}", e)))?;
    Ok(json["success"].as_bool().unwrap_or(false))
}

pub async fn captcha_middleware(
    State(config): State<AppConfig>,
    State(pool): State<Pool>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get("x-captcha-token")
        .and_then(|v| v.to_str().ok());
    let user_id = req
        .headers()
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok());
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok());
    let mut success = false;
    if let Some(token) = token {
        success = verify_captcha(&config.captcha_secret, token)
            .await
            .unwrap_or(false);
    }
    // Log ke DB
    let client = pool.get().await.ok();
    if let Some(client) = client {
        client.execute(
            r#"INSERT INTO bantuan.captcha_logs (id, user_id, ip_address, success, created_at) VALUES ($1, $2, $3, $4, $5)"#,
            &[&Uuid::new_v4(), &user_id, &ip, &success, &Utc::now()]
        ).await.ok();
    }
    if !success {
        return Err(AppError::Validation(
            "CAPTCHA gagal diverifikasi".to_string(),
        ));
    }
    Ok(next.run(req).await)
}
