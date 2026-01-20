//! Webhook Notification Handler
//!
//! This module provides endpoints for managing webhook subscriptions and
//! sending notifications on secret changes with retry policy.

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::error::ApiError;
use crate::response::ApiResponse;

/// Webhook subscription request
#[derive(Debug, Deserialize, Serialize)]
pub struct WebhookSubscription {
    /// Webhook URL to call
    pub url: String,

    /// Secret paths to watch (supports wildcards)
    pub paths: Vec<String>,

    /// Events to trigger on
    pub events: Vec<WebhookEvent>,

    /// HTTP method (default: POST)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,

    /// Custom headers to include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    /// Secret for HMAC signature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,

    /// Retry configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,

    /// Timeout in seconds (default: 30)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,

    /// Active status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

/// Webhook events
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEvent {
    SecretCreated,
    SecretUpdated,
    SecretDeleted,
    SecretRotated,
    SecretRevoked,
    LeaseExpired,
    PolicyChanged,
    AuthFailed,
}

/// Retry configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (default: 5)
    pub max_attempts: u32,

    /// Initial retry delay in seconds (default: 1)
    pub initial_delay: u64,

    /// Maximum retry delay in seconds (default: 60)
    pub max_delay: u64,

    /// Backoff multiplier (default: 2.0 for exponential backoff)
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: 1,
            max_delay: 60,
            backoff_multiplier: 2.0,
        }
    }
}

/// Webhook subscription response
#[derive(Debug, Serialize)]
pub struct WebhookSubscriptionResponse {
    pub id: String,
    pub url: String,
    pub paths: Vec<String>,
    pub events: Vec<WebhookEvent>,
    pub active: bool,
    pub created_at: String,
}

/// Webhook notification payload
#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookPayload {
    /// Event type
    pub event: WebhookEvent,

    /// Timestamp
    pub timestamp: String,

    /// Secret path
    pub path: String,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Webhook subscription ID
    pub subscription_id: String,
}

/// Webhook delivery status
#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookDelivery {
    pub id: String,
    pub subscription_id: String,
    pub payload: WebhookPayload,
    pub status: DeliveryStatus,
    pub attempts: u32,
    pub last_attempt_at: Option<String>,
    pub next_retry_at: Option<String>,
    pub response_code: Option<u16>,
    pub error_message: Option<String>,
}

/// Delivery status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryStatus {
    Pending,
    Delivered,
    Failed,
    Retrying,
}

/// Subscribe to webhook notifications
///
/// POST /v1/webhooks/subscribe
pub async fn subscribe_webhook(
    State(_state): State<Arc<crate::services::ServiceContainer>>,
    Json(request): Json<WebhookSubscription>,
) -> Result<Json<ApiResponse<WebhookSubscriptionResponse>>, ApiError> {
    info!("Creating webhook subscription for URL: {}", request.url);

    // Validate URL
    if !request.url.starts_with("http://") && !request.url.starts_with("https://") {
        return Err(ApiError::BadRequest {
            message: "Webhook URL must start with http:// or https://".to_string(),
        });
    }

    // Validate paths
    if request.paths.is_empty() {
        return Err(ApiError::BadRequest {
            message: "At least one path must be specified".to_string(),
        });
    }

    // Validate events
    if request.events.is_empty() {
        return Err(ApiError::BadRequest {
            message: "At least one event must be specified".to_string(),
        });
    }

    // Generate subscription ID
    let subscription_id = Uuid::new_v4().to_string();

    // Store subscription
    let subscription = WebhookSubscriptionData {
        id: subscription_id.clone(),
        url: request.url.clone(),
        paths: request.paths.clone(),
        events: request.events.clone(),
        method: request.method.unwrap_or_else(|| "POST".to_string()),
        headers: request.headers.unwrap_or_default(),
        secret: request.secret,
        retry: request.retry.unwrap_or_default(),
        timeout: request.timeout.unwrap_or(30),
        active: request.active.unwrap_or(true),
        created_at: chrono::Utc::now(),
    };

    store_subscription(&subscription).await?;

    let response = WebhookSubscriptionResponse {
        id: subscription_id,
        url: request.url,
        paths: request.paths,
        events: request.events,
        active: subscription.active,
        created_at: subscription.created_at.to_rfc3339(),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Unsubscribe from webhook notifications
///
/// DELETE /v1/webhooks/subscribe/{subscription_id}
pub async fn unsubscribe_webhook(
    State(_state): State<Arc<crate::services::ServiceContainer>>,
    Path(subscription_id): Path<String>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    info!("Deleting webhook subscription: {}", subscription_id);

    delete_subscription(&subscription_id).await?;

    Ok(Json(ApiResponse::success(())))
}

/// List webhook subscriptions
///
/// GET /v1/webhooks/subscriptions
pub async fn list_subscriptions(
    State(_state): State<Arc<crate::services::ServiceContainer>>,
) -> Result<Json<ApiResponse<Vec<WebhookSubscriptionResponse>>>, ApiError> {
    let subscriptions = get_all_subscriptions().await?;

    let response: Vec<WebhookSubscriptionResponse> = subscriptions
        .into_iter()
        .map(|sub| WebhookSubscriptionResponse {
            id: sub.id,
            url: sub.url,
            paths: sub.paths,
            events: sub.events,
            active: sub.active,
            created_at: sub.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(ApiResponse::success(response)))
}

/// Get webhook subscription details
///
/// GET /v1/webhooks/subscriptions/{subscription_id}
pub async fn get_subscription(
    State(_state): State<Arc<crate::services::ServiceContainer>>,
    Path(subscription_id): Path<String>,
) -> Result<Json<ApiResponse<WebhookSubscriptionResponse>>, ApiError> {
    let subscription = get_subscription_by_id(&subscription_id).await?;

    let response = WebhookSubscriptionResponse {
        id: subscription.id,
        url: subscription.url,
        paths: subscription.paths,
        events: subscription.events,
        active: subscription.active,
        created_at: subscription.created_at.to_rfc3339(),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Update webhook subscription
///
/// PUT /v1/webhooks/subscriptions/{subscription_id}
pub async fn update_subscription(
    State(_state): State<Arc<crate::services::ServiceContainer>>,
    Path(subscription_id): Path<String>,
    Json(request): Json<WebhookSubscription>,
) -> Result<Json<ApiResponse<WebhookSubscriptionResponse>>, ApiError> {
    info!("Updating webhook subscription: {}", subscription_id);

    let mut subscription = get_subscription_by_id(&subscription_id).await?;

    // Update fields
    subscription.url = request.url;
    subscription.paths = request.paths;
    subscription.events = request.events;
    subscription.method = request.method.unwrap_or_else(|| "POST".to_string());
    subscription.headers = request.headers.unwrap_or_default();
    subscription.secret = request.secret;
    subscription.retry = request.retry.unwrap_or_default();
    subscription.timeout = request.timeout.unwrap_or(30);
    subscription.active = request.active.unwrap_or(true);

    update_subscription_data(&subscription).await?;

    let response = WebhookSubscriptionResponse {
        id: subscription.id,
        url: subscription.url,
        paths: subscription.paths,
        events: subscription.events,
        active: subscription.active,
        created_at: subscription.created_at.to_rfc3339(),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// List webhook deliveries
///
/// GET /v1/webhooks/deliveries
pub async fn list_deliveries(
    State(_state): State<Arc<crate::services::ServiceContainer>>,
) -> Result<Json<ApiResponse<Vec<WebhookDelivery>>>, ApiError> {
    let deliveries = get_all_deliveries().await?;

    Ok(Json(ApiResponse::success(deliveries)))
}

/// Get webhook delivery details
///
/// GET /v1/webhooks/deliveries/{delivery_id}
pub async fn get_delivery(
    State(_state): State<Arc<crate::services::ServiceContainer>>,
    Path(delivery_id): Path<String>,
) -> Result<Json<ApiResponse<WebhookDelivery>>, ApiError> {
    let delivery = get_delivery_by_id(&delivery_id).await?;

    Ok(Json(ApiResponse::success(delivery)))
}

/// Retry a failed webhook delivery
///
/// POST /v1/webhooks/deliveries/{delivery_id}/retry
pub async fn retry_delivery(
    State(_state): State<Arc<crate::services::ServiceContainer>>,
    Path(delivery_id): Path<String>,
) -> Result<Json<ApiResponse<WebhookDelivery>>, ApiError> {
    info!("Retrying webhook delivery: {}", delivery_id);

    let delivery = get_delivery_by_id(&delivery_id).await?;

    if delivery.status == DeliveryStatus::Delivered {
        return Err(ApiError::BadRequest {
            message: "Cannot retry a delivered webhook".to_string(),
        });
    }

    // Trigger retry
    trigger_webhook_delivery(&delivery).await?;

    let updated_delivery = get_delivery_by_id(&delivery_id).await?;

    Ok(Json(ApiResponse::success(updated_delivery)))
}

// Internal data structures and helper functions

#[derive(Debug, Clone)]
struct WebhookSubscriptionData {
    id: String,
    url: String,
    paths: Vec<String>,
    events: Vec<WebhookEvent>,
    method: String,
    headers: HashMap<String, String>,
    secret: Option<String>,
    retry: RetryConfig,
    timeout: u64,
    active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

async fn store_subscription(subscription: &WebhookSubscriptionData) -> Result<(), ApiError> {
    // TODO: Store in database
    info!("Storing webhook subscription: {}", subscription.id);
    Ok(())
}

async fn delete_subscription(subscription_id: &str) -> Result<(), ApiError> {
    // TODO: Delete from database
    info!("Deleting webhook subscription: {}", subscription_id);
    Ok(())
}

async fn get_all_subscriptions() -> Result<Vec<WebhookSubscriptionData>, ApiError> {
    // TODO: Query from database
    Ok(vec![])
}

async fn get_subscription_by_id(subscription_id: &str) -> Result<WebhookSubscriptionData, ApiError> {
    // TODO: Query from database
    info!("Getting webhook subscription: {}", subscription_id);

    // Mock data for now
    Ok(WebhookSubscriptionData {
        id: subscription_id.to_string(),
        url: "https://example.com/webhook".to_string(),
        paths: vec!["/secret/data/*".to_string()],
        events: vec![WebhookEvent::SecretUpdated],
        method: "POST".to_string(),
        headers: HashMap::new(),
        secret: None,
        retry: RetryConfig::default(),
        timeout: 30,
        active: true,
        created_at: chrono::Utc::now(),
    })
}

async fn update_subscription_data(subscription: &WebhookSubscriptionData) -> Result<(), ApiError> {
    // TODO: Update in database
    info!("Updating webhook subscription: {}", subscription.id);
    Ok(())
}

async fn get_all_deliveries() -> Result<Vec<WebhookDelivery>, ApiError> {
    // TODO: Query from database
    Ok(vec![])
}

async fn get_delivery_by_id(delivery_id: &str) -> Result<WebhookDelivery, ApiError> {
    // TODO: Query from database
    info!("Getting webhook delivery: {}", delivery_id);

    // Mock data for now
    Ok(WebhookDelivery {
        id: delivery_id.to_string(),
        subscription_id: "sub-123".to_string(),
        payload: WebhookPayload {
            event: WebhookEvent::SecretUpdated,
            timestamp: chrono::Utc::now().to_rfc3339(),
            path: "/secret/data/test".to_string(),
            metadata: HashMap::new(),
            subscription_id: "sub-123".to_string(),
        },
        status: DeliveryStatus::Failed,
        attempts: 1,
        last_attempt_at: Some(chrono::Utc::now().to_rfc3339()),
        next_retry_at: Some((chrono::Utc::now() + chrono::Duration::seconds(2)).to_rfc3339()),
        response_code: Some(500),
        error_message: Some("Internal Server Error".to_string()),
    })
}

async fn trigger_webhook_delivery(delivery: &WebhookDelivery) -> Result<(), ApiError> {
    info!("Triggering webhook delivery: {}", delivery.id);

    // Get subscription
    let subscription = get_subscription_by_id(&delivery.subscription_id).await?;

    // Send webhook with retry logic
    send_webhook_with_retry(&subscription, &delivery.payload).await?;

    Ok(())
}

/// Send webhook notification with exponential backoff retry
pub async fn send_webhook_with_retry(
    subscription: &WebhookSubscriptionData,
    payload: &WebhookPayload,
) -> Result<(), ApiError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(subscription.timeout))
        .build()
        .map_err(|e| ApiError::internal(e.to_string()))?;

    let mut attempt = 0;
    let mut delay = subscription.retry.initial_delay;

    loop {
        attempt += 1;

        info!(
            "Sending webhook to {} (attempt {}/{})",
            subscription.url, attempt, subscription.retry.max_attempts
        );

        // Build request
        let mut request = client
            .request(
                subscription.method.parse().unwrap_or(reqwest::Method::POST),
                &subscription.url,
            )
            .json(payload);

        // Add custom headers
        for (key, value) in &subscription.headers {
            request = request.header(key, value);
        }

        // Add HMAC signature if secret is configured
        if let Some(secret) = &subscription.secret {
            let signature = compute_hmac_signature(payload, secret)?;
            request = request.header("X-Webhook-Signature", signature);
        }

        // Send request
        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    info!("Webhook delivered successfully to {}", subscription.url);
                    return Ok(());
                } else {
                    warn!(
                        "Webhook delivery failed with status {}: {}",
                        response.status(),
                        subscription.url
                    );

                    if attempt >= subscription.retry.max_attempts {
                        return Err(ApiError::internal(format!(
                            "Webhook delivery failed after {} attempts",
                            attempt
                        )));
                    }
                }
            }
            Err(e) => {
                error!("Webhook delivery error: {}", e);

                if attempt >= subscription.retry.max_attempts {
                    return Err(ApiError::internal(format!(
                        "Webhook delivery failed after {} attempts: {}",
                        attempt, e
                    )));
                }
            }
        }

        // Calculate next delay with exponential backoff
        tokio::time::sleep(Duration::from_secs(delay)).await;

        delay = ((delay as f64) * subscription.retry.backoff_multiplier) as u64;
        delay = delay.min(subscription.retry.max_delay);
    }
}

fn compute_hmac_signature(payload: &WebhookPayload, secret: &str) -> Result<String, ApiError> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    let payload_json = serde_json::to_string(payload)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| ApiError::internal(e.to_string()))?;

    mac.update(payload_json.as_bytes());

    let result = mac.finalize();
    let signature = hex::encode(result.into_bytes());

    Ok(format!("sha256={}", signature))
}

/// Create routes for webhook endpoints
pub fn create_routes() -> axum::Router<std::sync::Arc<crate::services::ServiceContainer>> {
    use axum::routing::{delete, get, post, put};

    axum::Router::new()
        .route("/subscribe", post(subscribe_webhook))
        .route("/subscribe/:subscription_id", delete(unsubscribe_webhook))
        .route("/subscriptions", get(list_subscriptions))
        .route("/subscriptions/:subscription_id", get(get_subscription))
        .route("/subscriptions/:subscription_id", put(update_subscription))
        .route("/deliveries", get(list_deliveries))
        .route("/deliveries/:delivery_id", get(get_delivery))
        .route("/deliveries/:delivery_id/retry", post(retry_delivery))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_delay, 1);
        assert_eq!(config.max_delay, 60);
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_webhook_event_serialization() {
        let event = WebhookEvent::SecretUpdated;
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, "\"secret_updated\"");
    }

    #[test]
    fn test_webhook_subscription_deserialization() {
        let json = r#"{
            "url": "https://example.com/webhook",
            "paths": ["/secret/data/*"],
            "events": ["secret_updated", "secret_deleted"]
        }"#;

        let subscription: WebhookSubscription = serde_json::from_str(json).unwrap();
        assert_eq!(subscription.url, "https://example.com/webhook");
        assert_eq!(subscription.paths.len(), 1);
        assert_eq!(subscription.events.len(), 2);
    }

    #[test]
    fn test_compute_hmac_signature() {
        let payload = WebhookPayload {
            event: WebhookEvent::SecretUpdated,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            path: "/secret/data/test".to_string(),
            metadata: HashMap::new(),
            subscription_id: "sub-123".to_string(),
        };

        let signature = compute_hmac_signature(&payload, "secret").unwrap();
        assert!(signature.starts_with("sha256="));
    }

    #[test]
    fn test_exponential_backoff_calculation() {
        let config = RetryConfig::default();
        let mut delay = config.initial_delay;

        // First retry: 1s
        assert_eq!(delay, 1);

        // Second retry: 2s
        delay = ((delay as f64) * config.backoff_multiplier) as u64;
        assert_eq!(delay, 2);

        // Third retry: 4s
        delay = ((delay as f64) * config.backoff_multiplier) as u64;
        assert_eq!(delay, 4);

        // Fourth retry: 8s
        delay = ((delay as f64) * config.backoff_multiplier) as u64;
        assert_eq!(delay, 8);

        // Fifth retry: 16s
        delay = ((delay as f64) * config.backoff_multiplier) as u64;
        assert_eq!(delay, 16);

        // Sixth retry: 32s
        delay = ((delay as f64) * config.backoff_multiplier) as u64;
        assert_eq!(delay, 32);

        // Seventh retry: 60s (capped at max_delay)
        delay = ((delay as f64) * config.backoff_multiplier) as u64;
        delay = delay.min(config.max_delay);
        assert_eq!(delay, 60);
    }
}
