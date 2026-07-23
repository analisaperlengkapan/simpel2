//! Decomposed components for workflow configuration admin.
//!
//! These are **viewers only**. The editor components (config editor panel, step
//! editor modal, SLA editor, delete modal) were removed along with the backend
//! write endpoints they called, which had never done anything but return
//! `BadRequest` — see `layanan/perlengkapan/src/workflow/definition_handlers.rs`
//! for why editing is not merely unimplemented but currently unsafe to add.
//!
//! The orchestrator [`crate::pages::workflow::config_management::WorkflowConfigManagement`]
//! composes these into the full page.

pub mod config_json_preview;
pub mod config_list_panel;
pub mod role_matrix;
pub mod transition_matrix;

pub use config_json_preview::ConfigJsonPreview;
pub use config_list_panel::ConfigListPanel;
pub use role_matrix::RoleMatrix;
pub use transition_matrix::TransitionMatrix;
