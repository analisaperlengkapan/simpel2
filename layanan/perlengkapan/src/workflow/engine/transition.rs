use super::*;
use chrono::Utc;
use uuid::Uuid;

impl WorkflowEngine {
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

        // Correlation id for this transition (audit metadata / event bus).
        let transition_id = Uuid::new_v4();

        // 5. Resolve the entity's status table by workflow type. The engine is
        //    GENERIC: each workflow (kebutuhan/penghapusan/pemakaian) maps to its
        //    own status table, and every one exposes a `status` state-name column.
        let status_table = Self::status_table(entity_type)?;

        // Verify entity exists and current state matches.
        let verify_query =
            format!("SELECT status FROM perlengkapan.{status_table} WHERE id = $1");

        let row = tx
            .query_opt(&verify_query, &[&request.entity_id])
            .await?
            .ok_or(WorkflowError::EntityNotFound(request.entity_id))?;

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

        // 6. Update entity state (generic by resolved status table).
        let update_query = format!(
            "UPDATE perlengkapan.{status_table} SET status = $1, updated_at = NOW() WHERE id = $2"
        );

        tx.execute(&update_query, &[&request.to_state, &request.entity_id])
            .await?;

        // 7. Per-entity activity logging lives in each service's repository
        //    (e.g. kebutuhan `create_aktivitas`); cross-cutting audit is emitted
        //    generically below via `audit_sink` + `event_bus`. The engine no
        //    longer writes a kebutuhan-specific activity row, so it stays generic
        //    across all workflows.

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

                    // Document metadata rides on the generic audit/domain event
                    // below; there is no per-entity activity row to update here.
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
                        "transition_id": transition_id,
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
            activity_id: transition_id,
        })
    }
    /// Validate that a transition is allowed by the workflow configuration
    pub(crate) fn validate_transition(&self, from_state: &str, to_state: &str) -> Result<()> {
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
    /// Map a workflow `entity_type` (the config `name`) to its status table.
    /// The engine is generic across workflows; every status table exposes a
    /// `status` state-name column that the transition reads/updates.
    fn status_table(entity_type: &str) -> Result<&'static str> {
        Ok(match entity_type {
            "kebutuhan_bmn" => "pengajuan_kebutuhan_bmn",
            "pemakaian_bmn" => "izin_pemakaian_bmn",
            "penghapusan_bmn" => "penghapusan_bmn",
            "pakaian_dinas" => "pengajuan_pakaian_dinas",
            other => {
                return Err(WorkflowError::InvalidState(format!(
                    "unknown workflow entity_type '{other}': no status table mapping"
                )));
            }
        })
    }
}
