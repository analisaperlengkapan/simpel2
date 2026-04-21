// ============================================================================
// Workflow Module
// Description: Centralized workflow engine for approval processes
// Requirements: REQ-W001 through REQ-W012
// ============================================================================

pub mod config;
pub mod definition_handlers;
pub mod delegation;
pub mod dokumen_client;
pub mod engine;
pub mod handlers;
pub mod monitoring;
pub mod notifikasi_client;
pub mod parallel;
pub mod sla;
pub mod sla_scheduler;

// Include generated dokumen proto
pub mod dokumen_proto {
    tonic::include_proto!("dokumen");
}

// Include generated notifikasi proto
pub mod notifikasi_proto {
    tonic::include_proto!("notifikasi.v1");
}

pub use dokumen_client::{DocumentGenerationResult, DokumenClient};
pub use notifikasi_client::{NotificationPriority, NotifikasiClient, WorkflowNotificationType};
pub use sla_scheduler::{SlaEscalationScheduler, SlaSchedulerConfig};
