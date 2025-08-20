use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    auth::AuthService,
    error::AppError,
    models::{AppState, JwtClaims},
};

pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract token from Authorization header
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Unauthorized);
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix

    // Verify JWT token
    let auth_service = AuthService::new(state.pool.clone(), state.config.jwt_secret.clone());
    let claims = auth_service.verify_token(token)?;

    // Add claims to request extensions
    let mut request = request;
    request.extensions_mut().insert(claims);

    if let Some(claims) = request.extensions().get::<JwtClaims>() {
        info!("User authenticated: {}", claims.username);
    } else {
        warn!("User authenticated: JwtClaims not found in request extensions");
    }

    Ok(next.run(request).await)
}

pub async fn rbac_middleware(
    required_permission: &str,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let claims = request
        .extensions()
        .get::<JwtClaims>()
        .ok_or(AppError::Unauthorized)?;

    // Check if user has required permission
    if !claims.permissions.contains(&required_permission.to_string())
        && !claims.permissions.contains(&"*".to_string())
    {
        error!("User {} lacks permission: {}", claims.username, required_permission);
        return Err(AppError::Forbidden);
    }

    info!("User {} authorized for: {}", claims.username, required_permission);

    Ok(next.run(request).await)
}

pub async fn audit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let start_time = std::time::Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();

    // Extract user info if available
    let user_id = request
        .extensions()
        .get::<JwtClaims>()
        .map(|claims| claims.sub);

    let response = next.run(request).await;

    // Log audit entry
    let duration = start_time.elapsed();
    let status = response.status();
    
    let audit_log = crate::models::AuditLog {
        id: Uuid::new_v4(),
        user_id,
        action: method.to_string(),
        resource: uri.to_string(),
        resource_id: None,
        details: serde_json::json!({
            "method": method.to_string(),
            "uri": uri.to_string(),
            "status": status.as_u16(),
            "duration_ms": duration.as_millis(),
            "user_agent": headers
                .get("user-agent")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("unknown"),
        }),
        ip_address: headers
            .get("x-forwarded-for")
            .or(headers.get("x-real-ip"))
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string(),
        user_agent: headers
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string(),
        timestamp: chrono::Utc::now(),
        hash: "".to_string(), // Will be calculated in the database trigger
    };

    // Store audit log asynchronously
    tokio::spawn(async move {
        if let Err(e) = store_audit_log(&state.pool, audit_log).await {
            error!("Failed to store audit log: {}", e);
        }
    });

    Ok(response)
}

async fn store_audit_log(pool: &PgPool, audit_log: crate::models::AuditLog) -> Result<(), AppError> {
    sqlx::query!(
        "INSERT INTO keamanan.audit_logs (id, user_id, action, resource, resource_id, details, ip_address, user_agent, timestamp, hash) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        audit_log.id,
        audit_log.user_id,
        audit_log.action,
        audit_log.resource,
        audit_log.resource_id,
        audit_log.details,
        audit_log.ip_address,
        audit_log.user_agent,
        audit_log.timestamp,
        audit_log.hash
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract client IP
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .or(request.headers().get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    // Simple in-memory rate limiting (in production, use Redis)
    // This is a simplified implementation
    let rate_limit_key = format!("rate_limit:{}", client_ip);
    
    // Check rate limit (simplified)
    // In production, implement proper rate limiting with Redis
    
    Ok(next.run(request).await)
}

pub async fn cors_middleware(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let response = next.run(request).await;
    
    // Add CORS headers
    let mut response = response;
    let headers = response.headers_mut();
    
    if let Ok(val) = "*".parse() {
        headers.insert("Access-Control-Allow-Origin", val);
    }
    if let Ok(val) = "GET, POST, PUT, DELETE, OPTIONS".parse() {
        headers.insert("Access-Control-Allow-Methods", val);
    }
    if let Ok(val) = "Content-Type, Authorization".parse() {
        headers.insert("Access-Control-Allow-Headers", val);
    }
    
    Ok(response)
} 