//! Kubernetes controller for SecretSync resources

use crate::{Error, Result, SecretSync};
use futures::StreamExt;
use k8s_openapi::api::core::v1::Secret;
use kube::{
    Client, ResourceExt,
    api::{Api, Patch, PatchParams, PostParams},
    runtime::{
        controller::{Action, Controller},
        watcher::Config,
    },
};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

/// Context shared across reconciliation loops
#[derive(Clone)]
pub struct Context {
    /// Kubernetes client
    pub client: Client,

    /// Default Secreton URL
    pub secreton_url: String,

    /// Default authentication token (optional)
    pub default_token: Option<String>,
}

/// Run the SecretSync controller
///
/// This function starts the controller loop that watches for SecretSync resources
/// and reconciles them by synchronizing secrets from Secreton to Kubernetes.
pub async fn run(context: Context) -> Result<()> {
    let client = context.client.clone();
    let secret_syncs = Api::<SecretSync>::all(client.clone());

    info!("Starting SecretSync controller");

    Controller::new(secret_syncs, Config::default())
        .run(reconcile, error_policy, Arc::new(context))
        .for_each(|res| async move {
            match res {
                Ok(o) => info!("Reconciled: {:?}", o),
                Err(e) => error!("Reconciliation error: {:?}", e),
            }
        })
        .await;

    Ok(())
}

/// Reconcile a SecretSync resource
///
/// This function is called whenever a SecretSync resource is created, updated, or deleted.
/// It fetches the secret from Secreton and creates/updates the corresponding Kubernetes Secret.
async fn reconcile(secret_sync: Arc<SecretSync>, ctx: Arc<Context>) -> Result<Action> {
    let name = secret_sync.name_any();
    let namespace = secret_sync
        .namespace()
        .ok_or_else(|| Error::ReconciliationFailed("SecretSync must be namespaced".to_string()))?;

    info!(
        "Reconciling SecretSync {}/{} for path {}",
        namespace, name, secret_sync.spec.secreton_path
    );

    // Get the Kubernetes Secret API for this namespace
    let secrets: Api<Secret> = Api::namespaced(ctx.client.clone(), &namespace);

    // Fetch secret from Secreton
    let secret_data = match fetch_secret_from_secreton(&secret_sync, &ctx).await {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to fetch secret from Secreton: {}", e);
            update_status(&secret_sync, &ctx, "Failed", Some(e.to_string())).await?;
            return Ok(Action::requeue(Duration::from_secs(60)));
        }
    };

    // Create or update Kubernetes Secret
    match create_or_update_secret(&secrets, &secret_sync, secret_data).await {
        Ok(_) => {
            info!("Successfully synced secret {}/{}", namespace, name);
            update_status(&secret_sync, &ctx, "Synced", None).await?;
        }
        Err(e) => {
            error!("Failed to create/update Kubernetes Secret: {}", e);
            update_status(&secret_sync, &ctx, "Failed", Some(e.to_string())).await?;
            return Ok(Action::requeue(Duration::from_secs(60)));
        }
    }

    // Requeue based on refresh interval
    let refresh_interval = secret_sync.spec.refresh_interval.unwrap_or(300);
    Ok(Action::requeue(Duration::from_secs(refresh_interval)))
}

/// Fetch secret data from Secreton
async fn fetch_secret_from_secreton(
    secret_sync: &SecretSync,
    ctx: &Context,
) -> Result<std::collections::HashMap<String, String>> {
    let secreton_url = secret_sync
        .spec
        .secreton_url
        .as_ref()
        .unwrap_or(&ctx.secreton_url);

    let path = &secret_sync.spec.secreton_path;

    // Build the full URL
    let url = format!("{}/v1{}", secreton_url, path);

    // Get authentication token
    let token = get_auth_token(secret_sync, ctx).await?;

    // Make HTTP request to Secreton
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("X-Engine-Token", token)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(Error::SecretNotFound(format!(
            "Secreton returned status: {}",
            response.status()
        )));
    }

    let json: serde_json::Value = response.json().await?;

    // Extract data from Secreton response
    let data = json
        .get("data")
        .and_then(|d| d.get("data"))
        .ok_or_else(|| Error::SecretNotFound("Invalid response format".to_string()))?;

    // Convert to HashMap<String, String>
    let mut result = std::collections::HashMap::new();
    if let Some(obj) = data.as_object() {
        for (key, value) in obj {
            if let Some(str_value) = value.as_str() {
                result.insert(key.clone(), str_value.to_string());
            } else {
                // Convert non-string values to JSON strings
                result.insert(key.clone(), value.to_string());
            }
        }
    }

    Ok(result)
}

/// Get authentication token for Secreton
async fn get_auth_token(secret_sync: &SecretSync, ctx: &Context) -> Result<String> {
    // For now, use default token from context
    // TODO: Implement Kubernetes auth, AppRole, etc.
    if let Some(auth) = &secret_sync.spec.auth {
        match auth.method.as_str() {
            "token" => {
                if let Some(token_ref) = &auth.token_secret {
                    // Fetch token from Kubernetes Secret
                    let namespace = secret_sync
                        .namespace()
                        .ok_or_else(|| Error::AuthenticationFailed("No namespace".to_string()))?;
                    let secrets: Api<Secret> = Api::namespaced(ctx.client.clone(), &namespace);
                    let secret = secrets.get(&token_ref.name).await?;

                    let data = secret.data.ok_or_else(|| {
                        Error::AuthenticationFailed("Secret has no data".to_string())
                    })?;

                    let token_bytes = data.get(&token_ref.key).ok_or_else(|| {
                        Error::AuthenticationFailed("Token key not found".to_string())
                    })?;

                    let token = String::from_utf8(token_bytes.0.clone()).map_err(|_| {
                        Error::AuthenticationFailed("Invalid UTF-8 in token".to_string())
                    })?;

                    return Ok(token);
                }
            }
            _ => {
                warn!("Authentication method {} not yet implemented", auth.method);
            }
        }
    }

    // Fall back to default token
    ctx.default_token
        .clone()
        .ok_or_else(|| Error::AuthenticationFailed("No authentication configured".to_string()))
}

/// Create or update a Kubernetes Secret
async fn create_or_update_secret(
    secrets: &Api<Secret>,
    secret_sync: &SecretSync,
    data: std::collections::HashMap<String, String>,
) -> Result<()> {
    use k8s_openapi::ByteString;
    use std::collections::BTreeMap;

    let target_name = &secret_sync.spec.target_secret;

    // Convert data to BTreeMap<String, ByteString>
    let mut secret_data = BTreeMap::new();
    for (key, value) in data {
        secret_data.insert(key, ByteString(value.into_bytes()));
    }

    // Create Secret object
    let secret = Secret {
        metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
            name: Some(target_name.clone()),
            labels: Some({
                let mut labels = BTreeMap::new();
                labels.insert("managed-by".to_string(), "secreton-operator".to_string());
                labels.insert("secreton-sync".to_string(), secret_sync.name_any());
                labels
            }),
            ..Default::default()
        },
        data: Some(secret_data),
        ..Default::default()
    };

    // Try to create or update
    match secrets.get(target_name).await {
        Ok(_) => {
            // Secret exists, update it
            let patch = Patch::Merge(&secret);
            secrets
                .patch(target_name, &PatchParams::default(), &patch)
                .await?;
            info!("Updated Kubernetes Secret {}", target_name);
        }
        Err(_) => {
            // Secret doesn't exist, create it
            secrets.create(&PostParams::default(), &secret).await?;
            info!("Created Kubernetes Secret {}", target_name);
        }
    }

    Ok(())
}

/// Update the status of a SecretSync resource
async fn update_status(
    secret_sync: &SecretSync,
    ctx: &Context,
    phase: &str,
    message: Option<String>,
) -> Result<()> {
    let name = secret_sync.name_any();
    let namespace = secret_sync
        .namespace()
        .ok_or_else(|| Error::ReconciliationFailed("No namespace".to_string()))?;

    let api: Api<SecretSync> = Api::namespaced(ctx.client.clone(), &namespace);

    let mut status = secret_sync.status.clone().unwrap_or_default();
    status.phase = phase.to_string();
    status.last_sync_time = Some(chrono::Utc::now().to_rfc3339());

    if phase == "Synced" {
        status.last_successful_sync = Some(chrono::Utc::now().to_rfc3339());
    }

    if let Some(msg) = message {
        status.message = Some(msg);
    }

    status.sync_attempts = Some(status.sync_attempts.unwrap_or(0) + 1);

    let patch = serde_json::json!({
        "status": status
    });

    api.patch_status(&name, &PatchParams::default(), &Patch::Merge(&patch))
        .await?;

    Ok(())
}

/// Error policy for the controller
fn error_policy(_object: Arc<SecretSync>, error: &Error, _ctx: Arc<Context>) -> Action {
    error!("Reconciliation error: {:?}", error);
    Action::requeue(Duration::from_secs(60))
}
