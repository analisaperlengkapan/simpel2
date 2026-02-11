// ============================================================================
// Workflow Module
// Description: Centralized workflow engine for approval processes
// Requirements: REQ-W001 through REQ-W012
// ============================================================================

pub mod config;
pub mod delegation;
pub mod dokumen_client;
pub mod engine;
pub mod monitoring;
pub mod notifikasi_client;
pub mod parallel;
pub mod sla;

// Include generated dokumen proto
pub mod dokumen_proto {
    tonic::include_proto!("dokumen");
}

// Include generated notifikasi proto
pub mod notifikasi_proto {
    tonic::include_proto!("notifikasi.v1");
}

pub use config::{WorkflowConfig, WorkflowStateCode};
pub use delegation::{
    DelegationManager, Delegation, DelegationStatus, CreateDelegationRequest, DelegationError,
};
pub use dokumen_client::{DokumenClient, DocumentGenerationResult};
pub use engine::{WorkflowEngine, WorkflowError, TransitionRequest, TransitionResult};
pub use monitoring::{
    WorkflowMonitor, WorkflowMetrics, BottleneckInfo, WorkflowHistory, StateDuration,
    WorkflowSummary, MonitoringError,
};
pub use notifikasi_client::{
    NotifikasiClient, NotificationPriority, WorkflowNotificationType, NotificationSendResult,
};
pub use parallel::{
    ParallelApprovalEngine, ParallelApproval, ParallelApprovalStatus, ApprovalVote,
    CreateParallelApprovalRequest, RecordApprovalRequest, ParallelApprovalError,
};
pub use sla::{SlaMonitor, SlaBreachInfo, SlaStatus, SlaError};
