//! # Kebutuhan BMN Services
//!
//! Business logic layer for BMN needs analysis system.
//! Handles workflow management, validation, and integration with external services.
//!
//! ## SIMAN
//! The feasibility analysis compares a satker's request against the assets it
//! already holds, read from `integrasi.siman_aset` — the same source of truth
//! `bank_aset` reads. It used to go through a remote-API client that was never
//! constructed anywhere, so the comparison never happened.

use std::sync::Arc;

use crate::shared::grpc::clients::AuthencClient;
use crate::shared::grpc::clients::IntegrasiClient;
use crate::workflow::engine::WorkflowEngine;

use super::repository::PgKebutuhanBmnRepository;

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
            workflow_engine: Arc::new(workflow_engine),
        }
    }

    /// Set layanan-integrasi gRPC client after construction
    pub fn with_integrasi_client(mut self, integrasi_client: IntegrasiClient) -> Self {
        self.integrasi_client = Some(integrasi_client);
        self
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
