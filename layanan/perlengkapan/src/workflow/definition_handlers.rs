//! Workflow Definition API Handlers — READ ONLY, deliberately.
//!
//! The four workflows are defined in Rust (`WorkflowConfig::default_*`) and the
//! engine is built from them at boot (`main.rs`). These endpoints expose that
//! shape so the admin page can render it; they do not edit it.
//!
//! There used to be create/update/delete handlers here. Every one of them
//! returned `BadRequest` without touching state, because the write path was
//! never built: there is no `workflow_steps` table, and the
//! `perlengkapan.workflow_definitions` table that does exist is read by
//! nothing.
//!
//! Making these editable is not simply unfinished work — it would be wrong
//! until the engine is the sole authority on transitions. Today the real
//! enforcement lives in the per-domain Rust status enums (`can_transition_to`),
//! so a DB-editable graph could promise a transition the code still refuses.
//! Two sources of truth, silently disagreeing. If custom workflows are ever
//! needed, make the engine authoritative FIRST, then add the write path.

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

// Shared envelope, not a local copy. A second declaration of the wire envelope
// is how `penghapusan_bmn` drifted out of sync with the paginated shape the
// frontend DTO requires — see the note in `workflow::handlers`.
pub use lib_perlengkapan::response::ApiResponse;

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

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(definitions, "Success")),
    ))
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

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(detail, "Success")),
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
