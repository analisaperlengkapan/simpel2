//! Workflow API types and functions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Workflow definition response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub description: String,
    pub version: String,
    pub status: String,
    pub supports_parallel_approval: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Workflow step information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub state_name: String,
    pub state_code: Option<i32>,
    pub required_role: Option<String>,
    pub sla_minutes: Option<u32>,
    pub next_states: Vec<String>,
    pub is_terminal: bool,
    pub escalation_enabled: bool,
}

/// Workflow definition detail with steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinitionDetail {
    #[serde(flatten)]
    pub definition: WorkflowDefinition,
    pub steps: Vec<WorkflowStep>,
    pub transitions: HashMap<String, Vec<String>>,
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: String,
}

/// Fetch all workflow definitions
#[cfg(target_arch = "wasm32")]
pub async fn fetch_workflow_definitions() -> Result<ApiResponse<Vec<WorkflowDefinition>>, String> {
    use crate::api::client::{API_BASE, get_auth_token};
    use gloo_net::http::Request;

    let url = format!("{}/workflow/definitions", API_BASE);
    let token = get_auth_token().unwrap_or_default();

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.ok() {
        return Err(format!("API error: HTTP {}", resp.status()));
    }

    resp.json::<ApiResponse<Vec<WorkflowDefinition>>>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_workflow_definitions() -> Result<ApiResponse<Vec<WorkflowDefinition>>, String> {
    Ok(ApiResponse {
        success: true,
        data: vec![],
        message: "Server-side stub".to_string(),
    })
}

/// Fetch workflow definition detail by name
#[cfg(target_arch = "wasm32")]
pub async fn fetch_workflow_definition_detail(
    name: &str,
) -> Result<ApiResponse<WorkflowDefinitionDetail>, String> {
    use crate::api::client::{API_BASE, get_auth_token};
    use gloo_net::http::Request;

    let url = format!("{}/workflow/definitions/{}", API_BASE, name);
    let token = get_auth_token().unwrap_or_default();

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.ok() {
        return Err(format!("API error: HTTP {}", resp.status()));
    }

    resp.json::<ApiResponse<WorkflowDefinitionDetail>>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_workflow_definition_detail(
    _name: &str,
) -> Result<ApiResponse<WorkflowDefinitionDetail>, String> {
    Err("Server-side stub".to_string())
}

/// Format SLA minutes to human-readable string
pub fn format_sla(minutes: u32) -> String {
    if minutes < 60 {
        format!("{} menit", minutes)
    } else if minutes < 1440 {
        let hours = minutes / 60;
        format!("{} jam", hours)
    } else {
        let days = minutes / 1440;
        format!("{} hari", days)
    }
}

// ============================================================================
// CRUD Operations
// ============================================================================

/// Request body for creating a workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: String,
    pub version: String,
    pub status: String,
    pub supports_parallel_approval: bool,
}

/// Request body for updating a workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkflowRequest {
    pub description: Option<String>,
    pub version: Option<String>,
    pub status: Option<String>,
    pub supports_parallel_approval: Option<bool>,
}

/// Request body for creating/updating a workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertStepRequest {
    pub state_name: String,
    pub state_code: Option<i32>,
    pub required_role: Option<String>,
    pub sla_minutes: Option<u32>,
    pub next_states: Vec<String>,
    pub escalation_enabled: bool,
}

/// Create a new workflow definition
#[cfg(target_arch = "wasm32")]
pub async fn create_workflow_definition(
    request: CreateWorkflowRequest,
) -> Result<ApiResponse<WorkflowDefinition>, String> {
    use crate::api::client::{API_BASE, get_auth_token};
    use gloo_net::http::Request;

    let url = format!("{}/workflow/definitions", API_BASE);
    let token = get_auth_token().unwrap_or_default();

    let resp = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)
        .map_err(|e| format!("Serialize error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.ok() {
        return Err(format!("API error: HTTP {}", resp.status()));
    }

    resp.json::<ApiResponse<WorkflowDefinition>>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_workflow_definition(
    _request: CreateWorkflowRequest,
) -> Result<ApiResponse<WorkflowDefinition>, String> {
    Err("Server-side stub".to_string())
}

/// Update an existing workflow definition
#[cfg(target_arch = "wasm32")]
pub async fn update_workflow_definition(
    name: &str,
    request: UpdateWorkflowRequest,
) -> Result<ApiResponse<WorkflowDefinition>, String> {
    use crate::api::client::{API_BASE, get_auth_token};
    use gloo_net::http::Request;

    let url = format!("{}/workflow/definitions/{}", API_BASE, name);
    let token = get_auth_token().unwrap_or_default();

    let resp = Request::put(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)
        .map_err(|e| format!("Serialize error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.ok() {
        return Err(format!("API error: HTTP {}", resp.status()));
    }

    resp.json::<ApiResponse<WorkflowDefinition>>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn update_workflow_definition(
    _name: &str,
    _request: UpdateWorkflowRequest,
) -> Result<ApiResponse<WorkflowDefinition>, String> {
    Err("Server-side stub".to_string())
}

/// Delete a workflow definition
#[cfg(target_arch = "wasm32")]
pub async fn delete_workflow_definition(name: &str) -> Result<ApiResponse<()>, String> {
    use crate::api::client::{API_BASE, get_auth_token};
    use gloo_net::http::Request;

    let url = format!("{}/workflow/definitions/{}", API_BASE, name);
    let token = get_auth_token().unwrap_or_default();

    let resp = Request::delete(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.ok() {
        return Err(format!("API error: HTTP {}", resp.status()));
    }

    resp.json::<ApiResponse<()>>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_workflow_definition(_name: &str) -> Result<ApiResponse<()>, String> {
    Err("Server-side stub".to_string())
}

/// Add or update a workflow step
#[cfg(target_arch = "wasm32")]
pub async fn upsert_workflow_step(
    workflow_name: &str,
    request: UpsertStepRequest,
) -> Result<ApiResponse<WorkflowStep>, String> {
    use crate::api::client::{API_BASE, get_auth_token};
    use gloo_net::http::Request;

    let url = format!("{}/workflow/definitions/{}/steps", API_BASE, workflow_name);
    let token = get_auth_token().unwrap_or_default();

    let resp = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)
        .map_err(|e| format!("Serialize error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.ok() {
        return Err(format!("API error: HTTP {}", resp.status()));
    }

    resp.json::<ApiResponse<WorkflowStep>>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn upsert_workflow_step(
    _workflow_name: &str,
    _request: UpsertStepRequest,
) -> Result<ApiResponse<WorkflowStep>, String> {
    Err("Server-side stub".to_string())
}

/// Delete a workflow step
#[cfg(target_arch = "wasm32")]
pub async fn delete_workflow_step(
    workflow_name: &str,
    state_name: &str,
) -> Result<ApiResponse<()>, String> {
    use crate::api::client::{API_BASE, get_auth_token};
    use gloo_net::http::Request;

    let url = format!(
        "{}/workflow/definitions/{}/steps/{}",
        API_BASE, workflow_name, state_name
    );
    let token = get_auth_token().unwrap_or_default();

    let resp = Request::delete(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.ok() {
        return Err(format!("API error: HTTP {}", resp.status()));
    }

    resp.json::<ApiResponse<()>>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_workflow_step(
    _workflow_name: &str,
    _state_name: &str,
) -> Result<ApiResponse<()>, String> {
    Err("Server-side stub".to_string())
}
