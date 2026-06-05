//! # Kebutuhan BMN Services
//!
//! Business logic layer for BMN needs analysis system.
//! Handles workflow management, validation, and integration with external services.
//!
//! ## SIMAN Integration
//! This service integrates with SIMAN (Sistem Informasi Manajemen Aset Negara)
//! to fetch existing BMN inventory for feasibility analysis.

use std::sync::Arc;

use crate::shared::grpc::clients::AuthencClient;
use crate::shared::grpc::clients::IntegrasiClient;
use crate::workflow::engine::WorkflowEngine;

use super::repository::PgKebutuhanBmnRepository;
use super::siman_integration::SimanIntegration;

// ============================================================================
// Service Interface
// ============================================================================

/// Service for BMN needs analysis business logic
#[derive(Clone)]
pub struct KebutuhanBmnService {
    repository: Arc<PgKebutuhanBmnRepository>,
    #[allow(dead_code)]
    authenc_client: AuthencClient,
    integrasi_client: Option<IntegrasiClient>,
    siman: Option<Arc<SimanIntegration>>,
    workflow_engine: Arc<WorkflowEngine>,
}

impl KebutuhanBmnService {
    /// Create a new service instance
    pub fn new(
        repository: PgKebutuhanBmnRepository,
        authenc_client: AuthencClient,
        workflow_engine: WorkflowEngine,
    ) -> Self {
        Self {
            repository: Arc::new(repository),
            authenc_client,
            integrasi_client: None,
            siman: None,
            workflow_engine: Arc::new(workflow_engine),
        }
    }

    /// Create a new service instance with SIMAN integration
    pub fn with_siman(
        repository: PgKebutuhanBmnRepository,
        authenc_client: AuthencClient,
        siman: SimanIntegration,
        workflow_engine: WorkflowEngine,
    ) -> Self {
        Self {
            repository: Arc::new(repository),
            authenc_client,
            integrasi_client: None,
            siman: Some(Arc::new(siman)),
            workflow_engine: Arc::new(workflow_engine),
        }
    }

    /// Set layanan-integrasi gRPC client after construction
    pub fn with_integrasi_client(mut self, integrasi_client: IntegrasiClient) -> Self {
        self.integrasi_client = Some(integrasi_client);
        self
    }

    /// Set SIMAN integration after construction
    pub fn set_siman(&mut self, siman: SimanIntegration) {
        self.siman = Some(Arc::new(siman));
    }

    // ========================================================================
    // Pengajuan Operations
    // ========================================================================
}

mod analisis;
mod batch;
mod pengajuan;
mod query;
mod satker_barang;
mod workflow;

#[cfg(test)]
mod tests;
