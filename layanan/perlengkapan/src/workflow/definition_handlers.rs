//! Workflow Definition API Handlers
//!
//! REST API endpoints for workflow definition management (CRUD operations)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::shared::error::AppError;
use crate::shared::middleware::Claims;
use crate::{AppState, workflow::config::WorkflowConfig};

// ═══════════════════════════════════════════════════════════════════════════
// Request/Response Types
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: String,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data,
            message: "Success".to_string(),
        }
    }

    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            data,
            message,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowDefinitionResponse {
    pub name: String,
    pub description: String,
    pub version: String,
    pub status: String,
    pub supports_parallel_approval: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStepResponse {
    pub state_name: String,
    pub state_code: Option<i32>,
    pub required_role: Option<String>,
    pub sla_minutes: Option<u32>,
    pub next_states: Vec<String>,
    pub is_terminal: bool,
    pub escalation_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowDefinitionDetailResponse {
    #[serde(flatten)]
    pub definition: WorkflowDefinitionResponse,
    pub steps: Vec<WorkflowStepResponse>,
    pub transitions: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: String,
    pub version: String,
    pub status: String,
    pub supports_parallel_approval: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkflowRequest {
    pub description: Option<String>,
    pub version: Option<String>,
    pub status: Option<String>,
    pub supports_parallel_approval: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertStepRequest {
    pub state_name: String,
    pub state_code: Option<i32>,
    pub required_role: Option<String>,
    pub sla_minutes: Option<u32>,
    pub next_states: Vec<String>,
    pub escalation_enabled: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Handler Functions
// ═══════════════════════════════════════════════════════════════════════════

/// GET /api/v1/workflow/definitions
///
/// Get all workflow definitions
pub async fn get_workflow_definitions(
    State(_state): State<AppState>,
    _claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    // For now, return the hardcoded workflow definitions
    // In the future, this could be stored in the database
    let definitions = vec![
        workflow_config_to_response(&WorkflowConfig::default_kebutuhan_bmn()),
        workflow_config_to_response(&WorkflowConfig::default_pemakaian_bmn()),
        workflow_config_to_response(&WorkflowConfig::default_penghapusan_bmn()),
        workflow_config_to_response(&WorkflowConfig::default_pakaian_dinas()),
    ];

    Ok((StatusCode::OK, Json(ApiResponse::success(definitions))))
}

/// GET /api/v1/workflow/definitions/{name}
///
/// Get workflow definition detail by name
pub async fn get_workflow_definition_by_name(
    State(_state): State<AppState>,
    Path(name): Path<String>,
    _claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    let config = match name.as_str() {
        "kebutuhan_bmn" => WorkflowConfig::default_kebutuhan_bmn(),
        "pemakaian_bmn" => WorkflowConfig::default_pemakaian_bmn(),
        "penghapusan_bmn" => WorkflowConfig::default_penghapusan_bmn(),
        "pakaian_dinas" => WorkflowConfig::default_pakaian_dinas(),
        _ => {
            return Err(AppError::NotFound(format!(
                "Workflow definition '{}' not found",
                name
            )));
        }
    };

    let detail = workflow_config_to_detail_response(&config);

    Ok((StatusCode::OK, Json(ApiResponse::success(detail))))
}

/// POST /api/v1/workflow/definitions
///
/// Create a new workflow definition
pub async fn create_workflow_definition(
    State(_state): State<AppState>,
    Json(_request): Json<CreateWorkflowRequest>,
) -> Result<impl IntoResponse, AppError> {
    // For now, workflow definitions are hardcoded
    // Return error indicating this operation is not yet supported
    Err::<(StatusCode, Json<ApiResponse<()>>), AppError>(AppError::BadRequest(
        "Creating custom workflow definitions is not yet supported. Use predefined workflows: kebutuhan_bmn, pemakaian_bmn, penghapusan_bmn, pakaian_dinas".to_string(),
    ))
}

/// PUT /api/v1/workflow/definitions/{name}
///
/// Update an existing workflow definition
pub async fn update_workflow_definition(
    State(_state): State<AppState>,
    Path(_name): Path<String>,
    Json(_request): Json<UpdateWorkflowRequest>,
) -> Result<impl IntoResponse, AppError> {
    // For now, workflow definitions are hardcoded
    // Return error indicating this operation is not yet supported
    Err::<(StatusCode, Json<ApiResponse<()>>), AppError>(AppError::BadRequest(
        "Updating workflow definitions is not yet supported. Workflow configurations are managed in code.".to_string(),
    ))
}

/// DELETE /api/v1/workflow/definitions/{name}
///
/// Delete a workflow definition
pub async fn delete_workflow_definition(
    State(_state): State<AppState>,
    Path(_name): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // For now, workflow definitions are hardcoded
    // Return error indicating this operation is not yet supported
    Err::<(StatusCode, Json<ApiResponse<()>>), AppError>(AppError::BadRequest(
        "Deleting workflow definitions is not supported. Workflow configurations are managed in code.".to_string(),
    ))
}

/// POST /api/v1/workflow/definitions/{name}/steps
///
/// Add or update a workflow step
pub async fn upsert_workflow_step(
    State(_state): State<AppState>,
    Path(_workflow_name): Path<String>,
    Json(_request): Json<UpsertStepRequest>,
) -> Result<impl IntoResponse, AppError> {
    // For now, workflow definitions are hardcoded
    // Return error indicating this operation is not yet supported
    Err::<(StatusCode, Json<ApiResponse<()>>), AppError>(AppError::BadRequest(
        "Modifying workflow steps is not yet supported. Workflow configurations are managed in code.".to_string(),
    ))
}

/// DELETE /api/v1/workflow/definitions/{name}/steps/{state}
///
/// Delete a workflow step
pub async fn delete_workflow_step(
    State(_state): State<AppState>,
    Path((_workflow_name, _state_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    // For now, workflow definitions are hardcoded
    // Return error indicating this operation is not yet supported
    Err::<(StatusCode, Json<ApiResponse<()>>), AppError>(AppError::BadRequest(
        "Deleting workflow steps is not supported. Workflow configurations are managed in code."
            .to_string(),
    ))
}

// ═══════════════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════════════

fn workflow_config_to_response(config: &WorkflowConfig) -> WorkflowDefinitionResponse {
    use chrono::Utc;
    let now = Utc::now().to_rfc3339();

    WorkflowDefinitionResponse {
        name: config.name.clone(),
        description: config.description.clone(),
        version: "1.0.0".to_string(),
        status: "active".to_string(),
        supports_parallel_approval: config.supports_parallel_approval,
        created_at: now.clone(),
        updated_at: now,
    }
}

fn workflow_config_to_detail_response(config: &WorkflowConfig) -> WorkflowDefinitionDetailResponse {
    let definition = workflow_config_to_response(config);

    // Build steps from transitions
    let mut steps = Vec::new();
    for (state_name, next_states) in &config.transitions {
        let step = WorkflowStepResponse {
            state_name: state_name.clone(),
            state_code: None, // Could map from WorkflowStateCode if needed
            required_role: config.required_roles.get(state_name).cloned(),
            sla_minutes: config.sla_minutes.get(state_name).copied(),
            next_states: next_states.clone(),
            is_terminal: next_states.is_empty(),
            escalation_enabled: config.sla_minutes.contains_key(state_name),
        };
        steps.push(step);
    }

    WorkflowDefinitionDetailResponse {
        definition,
        steps,
        transitions: config.transitions.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_config_to_response() {
        let config = WorkflowConfig::default_kebutuhan_bmn();
        let response = workflow_config_to_response(&config);

        assert_eq!(response.name, "kebutuhan_bmn");
        assert_eq!(response.status, "active");
        assert!(response.supports_parallel_approval);
    }

    #[test]
    fn test_workflow_config_to_detail_response() {
        let config = WorkflowConfig::default_kebutuhan_bmn();
        let detail = workflow_config_to_detail_response(&config);

        assert_eq!(detail.definition.name, "kebutuhan_bmn");
        assert!(!detail.steps.is_empty());
        assert!(!detail.transitions.is_empty());

        // Check that terminal states are marked correctly
        let rejected_step = detail.steps.iter().find(|s| s.state_name == "REJECTED");
        assert!(rejected_step.is_some());
        assert!(rejected_step.unwrap().is_terminal);
    }
}
