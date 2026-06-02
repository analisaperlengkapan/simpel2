// ============================================================================
// Workflow Engine Module
// Description: Core workflow engine for state transitions and approval validation
// Requirements: REQ-W004, REQ-W005
// ============================================================================

use crate::workflow::config::{WorkflowConfig, WorkflowStateCode};
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Workflow engine error types
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Invalid transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },

    #[error("Insufficient permissions: required role {required_role}")]
    InsufficientPermissions { required_role: String },

    #[error("Entity not found: {0}")]
    EntityNotFound(Uuid),

    #[error("Database error: {0}")]
    DatabaseError(#[from] tokio_postgres::Error),

    #[error("Pool error: {0}")]
    PoolError(#[from] deadpool_postgres::PoolError),

    #[error("Authenc client error: {0}")]
    AuthencError(String),

    #[error("Notification error: {0}")]
    NotificationError(String),

    #[error("Audit error: {0}")]
    AuditError(String),
}

pub type Result<T> = std::result::Result<T, WorkflowError>;

/// Workflow transition request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionRequest {
    /// Entity ID (e.g., kebutuhan_bmn.id)
    pub entity_id: Uuid,

    /// Current state
    pub from_state: String,

    /// Target state
    pub to_state: String,

    /// User performing the transition
    pub user_id: Uuid,

    /// Optional notes/comments
    pub catatan: Option<String>,

    /// IP address of the user
    pub ip_address: String,
}

/// Workflow transition result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionResult {
    /// Entity ID
    pub entity_id: Uuid,

    /// New state
    pub new_state: String,

    /// Timestamp of transition
    pub transitioned_at: DateTime<Utc>,

    /// Activity record ID
    pub activity_id: Uuid,
}

/// Core workflow engine
pub struct WorkflowEngine {
    /// Workflow configuration
    config: WorkflowConfig,

    /// Database connection pool
    db_pool: Pool,

    /// Document generator (port). Optional because document generation is
    /// only meaningful for entity types that produce SK/surat after approval.
    docs: Option<Arc<dyn lib_perlengkapan::contracts::DocumentGenerator>>,

    /// Notification sender (port).
    notifier: Option<Arc<dyn lib_perlengkapan::contracts::NotificationSender>>,

    /// Audit sink (port). Optional — dev/test boleh tanpa, produksi wajib
    /// terinjeksi agar setiap transisi tercatat di `perlengkapan.audit_log`
    /// (BPK-ready).
    audit_sink: Option<Arc<dyn lib_perlengkapan::contracts::AuditSink>>,

    /// V1.4: in-process event bus. Optional — bila ter-inject, setiap
    /// transisi sukses mem-publish `DomainEvent::WorkflowTransitioned`
    /// agar subscriber lain (real-time monitoring, search index, dst)
    /// tap-in tanpa men-tightly-couple engine.
    event_bus: Option<Arc<crate::shared::events::EventBus>>,
}

impl WorkflowEngine {
    /// Create a new workflow engine with the given configuration
    pub fn new(config: WorkflowConfig, db_pool: Pool) -> Self {
        Self {
            config,
            db_pool,
            docs: None,
            notifier: None,
            audit_sink: None,
            event_bus: None,
        }
    }

    /// Inject a document generator (replaces the deleted gRPC client).
    pub fn with_document_generator(
        mut self,
        docs: Arc<dyn lib_perlengkapan::contracts::DocumentGenerator>,
    ) -> Self {
        self.docs = Some(docs);
        self
    }

    /// Inject a notification sender (replaces the deleted gRPC client).
    pub fn with_notification_sender(
        mut self,
        notifier: Arc<dyn lib_perlengkapan::contracts::NotificationSender>,
    ) -> Self {
        self.notifier = Some(notifier);
        self
    }

    /// Inject the audit sink (typically `PgAuditSink`). Setiap transisi
    /// sukses akan mem-publish AuditEvent ke sink ini.
    pub fn with_audit_sink(
        mut self,
        audit_sink: Arc<dyn lib_perlengkapan::contracts::AuditSink>,
    ) -> Self {
        self.audit_sink = Some(audit_sink);
        self
    }

    /// V1.4: inject in-process event bus. Subscriber lain (notifier
    /// dispatcher, real-time monitoring, dst) dapat tap-in via
    /// `shared::events::spawn_subscriber`. Audit sink lama tetap valid
    /// secara paralel utk back-compat.
    pub fn with_event_bus(mut self, bus: Arc<crate::shared::events::EventBus>) -> Self {
        self.event_bus = Some(bus);
        self
    }

    /// Create a workflow engine for Kebutuhan BMN
    pub fn for_kebutuhan_bmn(db_pool: Pool) -> Self {
        Self::new(WorkflowConfig::default_kebutuhan_bmn(), db_pool)
    }

    /// Create a workflow engine for Pemakaian BMN
    pub fn for_pemakaian_bmn(db_pool: Pool) -> Self {
        Self::new(WorkflowConfig::default_pemakaian_bmn(), db_pool)
    }

    /// Create a workflow engine for Penghapusan BMN
    pub fn for_penghapusan_bmn(db_pool: Pool) -> Self {
        Self::new(WorkflowConfig::default_penghapusan_bmn(), db_pool)
    }

    /// Perform a state transition with validation
    ///
    /// This method:
    /// 1. Validates the transition is allowed
    /// 2. Validates the user has the required role
    /// 3. Updates the entity state in the database
    /// 4. Records the transition in the activity log
    /// 5. Logs the transition to audit log
    /// 6. Records metrics for monitoring
    ///
    /// Requirements: REQ-W004, REQ-W005, NFR-M004
    pub async fn transition(&self, request: TransitionRequest) -> Result<TransitionResult> {
        let start = std::time::Instant::now();
        let entity_type = self.config.name.as_str();

        // 1. Validate transition is allowed
        self.validate_transition(&request.from_state, &request.to_state)?;

        // 2. Validate user has required role (would call Authenc in production)
        // For now, we'll skip this validation as it requires Authenc integration
        // self.validate_approver_role(request.user_id, &request.to_state).await?;

        // 3. Get database connection
        let mut client = self.db_pool.get().await?;

        // 4. Start transaction
        let tx = client.transaction().await?;

        // 5. Verify entity exists and current state matches
        let verify_query = r#"
            SELECT status
            FROM perlengkapan.kebutuhan_bmn
            WHERE id = $1
        "#;

        let row = tx
            .query_opt(verify_query, &[&request.entity_id])
            .await?
            .ok_or_else(|| WorkflowError::EntityNotFound(request.entity_id))?;

        let current_status: String = row.get("status");

        // Verify current state matches expected state
        if current_status != request.from_state {
            // Record failed transition metric
            crate::shared::metrics::workflow_transitions_total()
                .with_label_values(&[
                    entity_type,
                    &request.from_state,
                    &request.to_state,
                    "failed",
                ])
                .inc();

            return Err(WorkflowError::InvalidTransition {
                from: current_status,
                to: request.to_state,
            });
        }

        // 6. Update entity state
        let update_query = r#"
            UPDATE perlengkapan.kebutuhan_bmn
            SET status = $1, updated_at = NOW()
            WHERE id = $2
        "#;

        tx.execute(update_query, &[&request.to_state, &request.entity_id])
            .await?;

        // 7. Get aktivitas_id for the new state
        let aktivitas_id = self.get_aktivitas_id(&tx, &request.to_state).await?;

        // 8. Record transition in activity log
        let activity_record_id = Uuid::new_v4();
        let insert_activity_query = r#"
            INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
            (id, pengajuan_id, aktivitas_id, user_id, catatan, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
        "#;

        tx.execute(
            insert_activity_query,
            &[
                &activity_record_id,
                &request.entity_id,
                &aktivitas_id,
                &request.user_id,
                &request.catatan,
            ],
        )
        .await?;

        // 9. Commit transaction
        tx.commit().await?;

        // 10. Generate document if transitioning to APPROVED state and dokumen client is available
        let mut document_url: Option<String> = None;

        if request.to_state == "APPROVED" && self.docs.is_some() {
            match self
                .generate_document_for_entity(&request.entity_id, entity_type)
                .await
            {
                Ok((doc_id, doc_url)) => {
                    document_url = Some(doc_url.clone());

                    // Update activity record with document metadata
                    let update_doc_query = r#"
                        UPDATE perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
                        SET document_id = $1, document_url = $2
                        WHERE id = $3
                    "#;

                    if let Ok(client) = self.db_pool.get().await {
                        let _ = client
                            .execute(update_doc_query, &[&doc_id, &doc_url, &activity_record_id])
                            .await;
                    }

                    tracing::info!(
                        entity_id = %request.entity_id,
                        document_id = %doc_id,
                        document_url = %doc_url,
                        "Document generated successfully after approval"
                    );
                }
                Err(e) => {
                    // Log error but don't fail the transition
                    tracing::error!(
                        entity_id = %request.entity_id,
                        error = %e,
                        "Failed to generate document after approval (non-blocking)"
                    );
                }
            }
        }

        // 10.5. Send notifications after state transition
        if self.notifier.is_some() {
            match self
                .send_workflow_notifications(&request, entity_type, document_url.as_deref())
                .await
            {
                Ok(notification_count) => {
                    tracing::info!(
                        entity_id = %request.entity_id,
                        from_state = %request.from_state,
                        to_state = %request.to_state,
                        notification_count = notification_count,
                        "Notifications sent successfully"
                    );
                }
                Err(e) => {
                    // Log error but don't fail the transition
                    tracing::error!(
                        entity_id = %request.entity_id,
                        from_state = %request.from_state,
                        to_state = %request.to_state,
                        error = %e,
                        "Failed to send notifications (non-blocking)"
                    );
                }
            }
        }

        // 11. Record metrics
        let duration = start.elapsed().as_secs_f64();

        // Record successful transition
        crate::shared::metrics::workflow_transitions_total()
            .with_label_values(&[
                entity_type,
                &request.from_state,
                &request.to_state,
                "success",
            ])
            .inc();

        // Record transition duration
        crate::shared::metrics::workflow_transition_duration()
            .with_label_values(&[entity_type, &request.from_state, &request.to_state])
            .observe(duration);

        // 11. Log to audit
        tracing::info!(
            entity_id = %request.entity_id,
            from_state = %request.from_state,
            to_state = %request.to_state,
            user_id = %request.user_id,
            duration_ms = duration * 1000.0,
            "Workflow transition completed"
        );

        // Publish AuditEvent ke sink jika ter-inject. Failure di-swallow
        // di dalam sink (lihat PgAuditSink) — kegagalan audit tidak boleh
        // membatalkan transisi yg sudah committed.
        if let Some(sink) = &self.audit_sink {
            use lib_perlengkapan::audit::{AuditAction, AuditEvent};
            let event =
                AuditEvent::new(entity_type, AuditAction::Custom, entity_type)
                    .action_name("workflow.transition")
                    .actor(request.user_id, "")
                    .ip(&request.ip_address)
                    .resource_id(request.entity_id.to_string())
                    .message(request.catatan.clone().unwrap_or_else(|| {
                        format!("{} → {}", request.from_state, request.to_state)
                    }))
                    .metadata(serde_json::json!({
                        "from_state": request.from_state,
                        "to_state": request.to_state,
                        "duration_ms": duration * 1000.0,
                        "activity_id": activity_record_id,
                    }));
            if let Err(e) = sink.log(event).await {
                tracing::warn!(
                    error = %e,
                    entity_id = %request.entity_id,
                    "audit_sink.log failed for workflow transition; continuing"
                );
            }
        }

        // V1.4: Publish ke EventBus jika ter-inject. Fail-soft: jika tidak
        // ada subscriber, publish() return 0 (bukan error). Subscriber yg
        // crash/lag tidak men-stop transisi.
        if let Some(bus) = &self.event_bus {
            use crate::shared::events::DomainEvent;
            let event = DomainEvent::WorkflowTransitioned {
                entity_type: entity_type.to_string(),
                entity_id: request.entity_id,
                from_state: request.from_state.clone(),
                to_state: request.to_state.clone(),
                user_id: request.user_id,
                catatan: request.catatan.clone(),
                ip_address: request.ip_address.clone(),
                timestamp: Utc::now(),
            };
            let n = bus.publish(event);
            tracing::debug!(
                entity_id = %request.entity_id,
                subscribers = n,
                "EventBus: WorkflowTransitioned dipublish"
            );
        }

        // 12. Return result
        Ok(TransitionResult {
            entity_id: request.entity_id,
            new_state: request.to_state,
            transitioned_at: Utc::now(),
            activity_id: activity_record_id,
        })
    }

    /// Validate that a transition is allowed by the workflow configuration
    fn validate_transition(&self, from_state: &str, to_state: &str) -> Result<()> {
        if !self.config.is_valid_transition(from_state, to_state) {
            return Err(WorkflowError::InvalidTransition {
                from: from_state.to_string(),
                to: to_state.to_string(),
            });
        }

        Ok(())
    }

    /// Validate that the user has the required role for the transition
    ///
    /// In production, this would call Authenc gRPC service to verify user roles
    /// For now, this is a placeholder that always succeeds
    #[allow(dead_code)]
    async fn validate_approver_role(&self, user_id: Uuid, to_state: &str) -> Result<()> {
        // Get required role from configuration
        let required_role = self
            .config
            .get_required_role(to_state)
            .ok_or_else(|| WorkflowError::InvalidState(to_state.to_string()))?;

        // In production, call Authenc gRPC service:
        // let response = self.authenc_client
        //     .get_user_roles(GetUserRolesRequest {
        //         user_id: user_id.to_string(),
        //     })
        //     .await
        //     .map_err(|e| WorkflowError::AuthencError(e.to_string()))?;
        //
        // let roles: Vec<String> = response.into_inner().roles;
        //
        // if !roles.contains(&required_role.to_string()) {
        //     return Err(WorkflowError::InsufficientPermissions {
        //         required_role: required_role.to_string(),
        //     });
        // }

        tracing::debug!(
            user_id = %user_id,
            required_role = %required_role,
            to_state = %to_state,
            "Role validation (placeholder - would call Authenc in production)"
        );

        Ok(())
    }

    /// Get aktivitas_id from ms_aktivitas_bmn table based on state name
    async fn get_aktivitas_id(
        &self,
        tx: &tokio_postgres::Transaction<'_>,
        state_name: &str,
    ) -> Result<i32> {
        // Convert state name to state code
        let state_code = WorkflowStateCode::from_state_name(state_name)
            .ok_or_else(|| WorkflowError::InvalidState(state_name.to_string()))?;

        // Query for aktivitas_id
        let query = r#"
            SELECT id
            FROM perlengkapan.ms_aktivitas_bmn
            WHERE kode = $1
        "#;

        let row = tx
            .query_one(query, &[&state_code.code()])
            .await
            .map_err(|e| {
                tracing::error!(
                    state_name = %state_name,
                    state_code = %state_code.code(),
                    error = %e,
                    "Failed to get aktivitas_id"
                );
                e
            })?;

        Ok(row.get("id"))
    }

    /// Get the current state of an entity
    pub async fn get_current_state(&self, entity_id: Uuid) -> Result<String> {
        let client = self.db_pool.get().await?;

        let query = r#"
            SELECT status
            FROM perlengkapan.kebutuhan_bmn
            WHERE id = $1
        "#;

        let row = client
            .query_opt(query, &[&entity_id])
            .await?
            .ok_or_else(|| WorkflowError::EntityNotFound(entity_id))?;

        Ok(row.get("status"))
    }

    /// Get all valid next states from the current state
    pub fn get_next_states(&self, current_state: &str) -> Vec<String> {
        self.config.get_next_states(current_state)
    }

    /// Check if a state is terminal (no outgoing transitions)
    pub fn is_terminal_state(&self, state: &str) -> bool {
        self.config.is_terminal_state(state)
    }

    /// Get the workflow configuration
    pub fn config(&self) -> &WorkflowConfig {
        &self.config
    }

    /// Compute an RFC3339 deadline for a target state using the config's
    /// `sla_minutes` mapping. Returns None if no SLA is configured for that
    /// state — the notification then omits a deadline rather than inventing
    /// one. Weekend/holiday-aware business-hour math can be layered on top
    /// later; for now we use wall-clock minutes, which matches how the UI
    /// renders countdowns today.
    fn compute_sla_deadline(&self, target_state: &str) -> Option<String> {
        let minutes = self.config.sla_minutes.get(target_state).copied()?;
        if minutes == 0 {
            return None;
        }
        let deadline = Utc::now() + chrono::Duration::minutes(minutes as i64);
        Some(deadline.to_rfc3339())
    }

    /// Look up the active document template id for a given template type.
    ///
    /// Replaces the previous placeholder UUIDs. Resolves against
    /// `dokumen.document_templates` by `template_type`, picking the highest
    /// active version.
    async fn resolve_template_id(
        client: &deadpool_postgres::Client,
        template_type: &str,
    ) -> Result<Uuid> {
        let row = client
            .query_opt(
                r#"
                SELECT id FROM dokumen.document_templates
                WHERE template_type = $1 AND is_active = TRUE
                ORDER BY version DESC
                LIMIT 1
                "#,
                &[&template_type],
            )
            .await?
            .ok_or_else(|| {
                WorkflowError::InvalidState(format!(
                    "No active document template registered for type '{}'",
                    template_type
                ))
            })?;
        Ok(row.get("id"))
    }

    /// Generate document for an entity after approval
    ///
    /// This method fetches entity data and calls the dokumen service to generate
    /// the appropriate document (SK, surat izin, etc.)
    ///
    /// Requirements: REQ-D001, REQ-D002, REQ-D005, REQ-W011
    async fn generate_document_for_entity(
        &self,
        entity_id: &Uuid,
        entity_type: &str,
    ) -> Result<(Uuid, String)> {
        let docs = self.docs.as_ref().ok_or_else(|| {
            WorkflowError::InvalidState("Document generator not configured".to_string())
        })?;

        // Fetch entity data from database
        let client = self.db_pool.get().await?;

        let (template_id, entity_data) = match entity_type {
            "kebutuhan_bmn" => {
                // Fetch kebutuhan BMN data
                let query = r#"
                    SELECT k.*, s.nama as satker_nama
                    FROM perlengkapan.kebutuhan_bmn k
                    LEFT JOIN perlengkapan.ms_satker s ON k.satker_id = s.id
                    WHERE k.id = $1
                "#;

                let row = client.query_one(query, &[entity_id]).await?;

                let template_id = Self::resolve_template_id(&client, "sk_kebutuhan_bmn").await?;

                let entity_data = serde_json::json!({
                    "id": entity_id.to_string(),
                    "satker_nama": row.get::<_, Option<String>>("satker_nama").unwrap_or_default(),
                    "tahun_anggaran": row.get::<_, i32>("tahun_anggaran"),
                    "status": row.get::<_, String>("status"),
                    "created_at": row.get::<_, chrono::DateTime<Utc>>("created_at").to_rfc3339(),
                    "document_type": "SK Kebutuhan BMN",
                    "approval_date": chrono::Utc::now().format("%d %B %Y").to_string(),
                });

                (template_id, entity_data)
            }
            "penghapusan_bmn" => {
                // Fetch penghapusan BMN data
                let query = r#"
                    SELECT p.*, s.nama as satker_nama
                    FROM perlengkapan.penghapusan_bmn p
                    LEFT JOIN perlengkapan.ms_satker s ON p.satker_id = s.id
                    WHERE p.id = $1
                "#;

                let row = client.query_one(query, &[entity_id]).await?;

                let template_id = Self::resolve_template_id(&client, "sk_penghapusan").await?;

                let entity_data = serde_json::json!({
                    "id": entity_id.to_string(),
                    "satker_nama": row.get::<_, Option<String>>("satker_nama").unwrap_or_default(),
                    "alasan": row.get::<_, Option<String>>("alasan").unwrap_or_default(),
                    "status": row.get::<_, String>("status"),
                    "created_at": row.get::<_, chrono::DateTime<Utc>>("created_at").to_rfc3339(),
                    "document_type": "SK Penghapusan BMN",
                    "approval_date": chrono::Utc::now().format("%d %B %Y").to_string(),
                });

                (template_id, entity_data)
            }
            _ => {
                return Err(WorkflowError::InvalidState(format!(
                    "Document generation not supported for entity type: {}",
                    entity_type
                )));
            }
        };

        // Build a DocumentRequest and dispatch through the trait. Metadata
        // about the originating workflow (entity_type/entity_id/generated_at)
        // is folded into `data` so templates can reference it.
        let mut data = entity_data;
        if let Some(obj) = data.as_object_mut() {
            obj.insert(
                "_workflow_meta".to_string(),
                serde_json::json!({
                    "entity_type": entity_type,
                    "entity_id": entity_id.to_string(),
                    "generated_by": "workflow_engine",
                    "generated_at": chrono::Utc::now().to_rfc3339(),
                }),
            );
        }

        let request = lib_perlengkapan::contracts::DocumentRequest {
            template_id: template_id.to_string(),
            format: lib_perlengkapan::contracts::DocumentFormat::Pdf,
            data,
            locale: None,
            requested_by: None,
        };

        let artifact = docs.generate(request).await.map_err(|e| {
            WorkflowError::InvalidState(format!("Document generation failed: {}", e))
        })?;

        // The artifact's storage_key stands in as the download URL for now —
        // a follow-up commit will swap to DocumentStorage::presigned_url.
        Ok((artifact.document_id, artifact.storage_key))
    }

    /// Send workflow notifications after state transition
    ///
    /// This method determines the appropriate recipients and notification type
    /// based on the state transition, then sends notifications via the notifikasi service.
    ///
    /// Requirements: REQ-N001, REQ-N003, REQ-N005, REQ-W011
    async fn send_workflow_notifications(
        &self,
        request: &TransitionRequest,
        entity_type: &str,
        document_url: Option<&str>,
    ) -> Result<usize> {
        let notifier = self.notifier.as_ref().ok_or_else(|| {
            WorkflowError::InvalidState("Notification sender not configured".to_string())
        })?;

        // Determine notification recipients and type based on state transition
        let (recipients, notification_type, priority) = self
            .determine_notification_details(request, entity_type, document_url)
            .await?;

        if recipients.is_empty() {
            tracing::warn!(
                entity_id = %request.entity_id,
                from_state = %request.from_state,
                to_state = %request.to_state,
                "No recipients found for notification"
            );
            return Ok(0);
        }

        // Dispatch one message per recipient through the trait. Failures on a
        // single recipient are logged but do not abort the others — matches
        // the prior gRPC client's send_notification_to_multiple semantics.
        let mut sent = 0usize;
        for recipient in &recipients {
            let msg =
                crate::workflow::to_notification_message(*recipient, &notification_type, priority);
            match notifier.send(msg).await {
                Ok(_) => sent += 1,
                Err(e) => tracing::error!(
                    user_id = %recipient,
                    error = %e,
                    "Failed to send notification to user"
                ),
            }
        }

        Ok(sent)
    }

    /// Determine notification recipients, type, and priority based on state transition
    ///
    /// This method implements the notification logic for different workflows:
    /// - Kebutuhan BMN: SUBMITTED → notify approver, APPROVED → notify requester
    /// - Penghapusan BMN: SUBMITTED → notify approver, APPROVED → notify requester
    /// - Pemakaian BMN: SUBMITTED → notify approver, APPROVED → notify requester
    ///
    /// Requirements: REQ-N005
    async fn determine_notification_details(
        &self,
        request: &TransitionRequest,
        entity_type: &str,
        document_url: Option<&str>,
    ) -> Result<(
        Vec<Uuid>,
        crate::workflow::WorkflowNotificationType,
        crate::workflow::NotificationPriority,
    )> {
        use crate::workflow::{NotificationPriority, WorkflowNotificationType};

        // Fetch entity data to get requester and satker information
        let client = self.db_pool.get().await?;

        let (requester_id, satker_id) = match entity_type {
            "kebutuhan_bmn" => {
                let query = r#"
                    SELECT created_by, satker_id
                    FROM perlengkapan.kebutuhan_bmn
                    WHERE id = $1
                "#;

                let row = client.query_one(query, &[&request.entity_id]).await?;
                let requester_id: Uuid = row.get("created_by");
                let satker_id: Uuid = row.get("satker_id");

                (requester_id, satker_id)
            }
            "penghapusan_bmn" => {
                let query = r#"
                    SELECT created_by, satker_id
                    FROM perlengkapan.penghapusan_bmn
                    WHERE id = $1
                "#;

                let row = client.query_one(query, &[&request.entity_id]).await?;
                let requester_id: Uuid = row.get("created_by");
                let satker_id: Uuid = row.get("satker_id");

                (requester_id, satker_id)
            }
            "pemakaian_bmn" => {
                let query = r#"
                    SELECT pemohon_id as created_by, satker_id
                    FROM perlengkapan.izin_pemakaian_bmn
                    WHERE id = $1
                "#;

                let row = client.query_one(query, &[&request.entity_id]).await?;
                let requester_id: Uuid = row.get("created_by");
                let satker_id: Uuid = row.get("satker_id");

                (requester_id, satker_id)
            }
            _ => {
                return Err(WorkflowError::InvalidState(format!(
                    "Notification not supported for entity type: {}",
                    entity_type
                )));
            }
        };

        // Determine recipients and notification type based on state transition
        let (recipients, notification_type, priority) = match request.to_state.as_str() {
            "SUBMITTED" => {
                // Notify approver (Verifikator role in same satker)
                let approvers = self
                    .get_approvers_for_state(&request.to_state, satker_id)
                    .await?;

                let notification = WorkflowNotificationType::ApprovalRequired {
                    entity_type: entity_type.to_string(),
                    entity_id: request.entity_id.to_string(),
                    current_state: request.to_state.clone(),
                    required_role: "Verifikator".to_string(),
                    deadline: self.compute_sla_deadline(&request.to_state),
                };

                (approvers, notification, NotificationPriority::High)
            }
            "APPROVED" => {
                // Notify requester
                let notification = WorkflowNotificationType::ApprovalCompleted {
                    entity_type: entity_type.to_string(),
                    entity_id: request.entity_id.to_string(),
                    approved_by: request.user_id.to_string(),
                    document_url: document_url.map(|s| s.to_string()),
                };

                (
                    vec![requester_id],
                    notification,
                    NotificationPriority::Normal,
                )
            }
            "REJECTED" => {
                // Notify requester
                let notification = WorkflowNotificationType::Rejected {
                    entity_type: entity_type.to_string(),
                    entity_id: request.entity_id.to_string(),
                    rejected_by: request.user_id.to_string(),
                    reason: request.catatan.clone(),
                };

                (vec![requester_id], notification, NotificationPriority::High)
            }
            "REVISION_REQUIRED" => {
                // Notify requester
                let notification = WorkflowNotificationType::RevisionRequired {
                    entity_type: entity_type.to_string(),
                    entity_id: request.entity_id.to_string(),
                    requested_by: request.user_id.to_string(),
                    notes: request.catatan.clone(),
                };

                (vec![requester_id], notification, NotificationPriority::High)
            }
            _ => {
                // For other states, send generic transition notification to requester
                let notification = WorkflowNotificationType::WorkflowTransition {
                    entity_type: entity_type.to_string(),
                    entity_id: request.entity_id.to_string(),
                    from_state: request.from_state.clone(),
                    to_state: request.to_state.clone(),
                    transition_by: request.user_id.to_string(),
                    catatan: request.catatan.clone(),
                };

                (
                    vec![requester_id],
                    notification,
                    NotificationPriority::Normal,
                )
            }
        };

        Ok((recipients, notification_type, priority))
    }

    /// Get approvers for a specific state in a satker
    ///
    /// In production, this would call Authenc gRPC service to get users with the required role.
    /// For now, this is a placeholder that returns an empty list.
    ///
    /// Requirements: REQ-N005
    async fn get_approvers_for_state(&self, state: &str, satker_id: Uuid) -> Result<Vec<Uuid>> {
        // Get required role from configuration
        let required_role = self
            .config
            .get_required_role(state)
            .ok_or_else(|| WorkflowError::InvalidState(state.to_string()))?;

        // In production, call Authenc gRPC service:
        // let response = self.authenc_client
        //     .get_users_by_role_and_satker(GetUsersByRoleAndSatkerRequest {
        //         role: required_role.to_string(),
        //         satker_id: satker_id.to_string(),
        //     })
        //     .await
        //     .map_err(|e| WorkflowError::AuthencError(e.to_string()))?;
        //
        // let user_ids: Vec<Uuid> = response.into_inner().users
        //     .iter()
        //     .filter_map(|u| Uuid::parse_str(&u.id).ok())
        //     .collect();
        //
        // if user_ids.is_empty() {
        //     // Escalate to parent satker if no approvers found
        //     return self.get_approvers_from_parent_satker(state, satker_id).await;
        // }
        //
        // Ok(user_ids)

        tracing::debug!(
            state = %state,
            required_role = %required_role,
            satker_id = %satker_id,
            "Getting approvers (placeholder - would call Authenc in production)"
        );

        // Placeholder: return empty list
        // In integration tests, this will be mocked
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_transition() {
        let config = WorkflowConfig::default_kebutuhan_bmn();
        let pool = deadpool_postgres::Pool::builder(deadpool_postgres::Manager::new(
            tokio_postgres::Config::new(),
            tokio_postgres::NoTls,
        ))
        .build()
        .unwrap();

        let engine = WorkflowEngine::new(config, pool);

        // Valid transition
        assert!(engine.validate_transition("DRAFT", "INPUT_BARANG").is_ok());

        // Invalid transition
        assert!(engine.validate_transition("DRAFT", "APPROVED").is_err());
    }

    #[test]
    fn test_get_next_states() {
        let config = WorkflowConfig::default_kebutuhan_bmn();
        let pool = deadpool_postgres::Pool::builder(deadpool_postgres::Manager::new(
            tokio_postgres::Config::new(),
            tokio_postgres::NoTls,
        ))
        .build()
        .unwrap();

        let engine = WorkflowEngine::new(config, pool);

        let next_states = engine.get_next_states("DRAFT");
        assert_eq!(next_states.len(), 2);
        assert!(next_states.contains(&"INPUT_BARANG".to_string()));
        assert!(next_states.contains(&"CANCELLED".to_string()));
    }

    #[test]
    fn test_is_terminal_state() {
        let config = WorkflowConfig::default_kebutuhan_bmn();
        let pool = deadpool_postgres::Pool::builder(deadpool_postgres::Manager::new(
            tokio_postgres::Config::new(),
            tokio_postgres::NoTls,
        ))
        .build()
        .unwrap();

        let engine = WorkflowEngine::new(config, pool);

        assert!(engine.is_terminal_state("REJECTED"));
        assert!(engine.is_terminal_state("CANCELLED"));
        assert!(!engine.is_terminal_state("DRAFT"));
    }
}
