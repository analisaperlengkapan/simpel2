use super::*;
use uuid::Uuid;

impl WorkflowEngine {
    /// Send workflow notifications after state transition
    ///
    /// This method determines the appropriate recipients and notification type
    /// based on the state transition, then sends notifications via the notifikasi service.
    ///
    /// Requirements: REQ-N001, REQ-N003, REQ-N005, REQ-W011
    pub(crate) async fn send_workflow_notifications(
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

                (Some(requester_id), satker_id)
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

                (Some(requester_id), satker_id)
            }
            "pemakaian_bmn" => {
                // `izin_pemakaian_bmn` has neither `pemohon_id` nor `satker_id`
                // (the requester is `created_by`, the unit is
                // `pegawai_satker_id`) — the old column names made this query
                // fail with "column does not exist" on every pemakaian
                // notification. `created_by` is nullable, so fall back to the
                // approver/validator-facing satker rather than panicking.
                let query = r#"
                    SELECT created_by, pegawai_satker_id
                    FROM perlengkapan.izin_pemakaian_bmn
                    WHERE id = $1
                "#;

                let row = client.query_one(query, &[&request.entity_id]).await?;
                let requester_id: Option<Uuid> = row.get("created_by");
                let satker_id: Uuid = row.get("pegawai_satker_id");

                if requester_id.is_none() {
                    tracing::warn!(
                        entity_id = %request.entity_id,
                        "pemakaian_bmn permit has no created_by; requester will not be notified"
                    );
                }

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
                    requester_id.into_iter().collect(),
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

                (
                    requester_id.into_iter().collect(),
                    notification,
                    NotificationPriority::High,
                )
            }
            "REVISION_REQUIRED" => {
                // Notify requester
                let notification = WorkflowNotificationType::RevisionRequired {
                    entity_type: entity_type.to_string(),
                    entity_id: request.entity_id.to_string(),
                    requested_by: request.user_id.to_string(),
                    notes: request.catatan.clone(),
                };

                (
                    requester_id.into_iter().collect(),
                    notification,
                    NotificationPriority::High,
                )
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
                    requester_id.into_iter().collect(),
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
