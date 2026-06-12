//! Decomposed components for workflow configuration admin.
//!
//! The orchestrator [`crate::pages::workflow::config_management::WorkflowConfigManagement`]
//! composes these into the full page.

pub mod config_editor_panel;
pub mod config_json_preview;
pub mod config_list_panel;
pub mod delete_modal;
pub mod role_matrix;
pub mod sla_editor;
pub mod step_editor_modal;
pub mod time_unit;
pub mod transition_matrix;

pub use config_editor_panel::ConfigEditorPanel;
pub use config_json_preview::ConfigJsonPreview;
pub use config_list_panel::ConfigListPanel;
pub use delete_modal::DeleteConfigModal;
pub use role_matrix::RoleMatrix;
pub use step_editor_modal::StepEditorModal;
pub use transition_matrix::TransitionMatrix;
