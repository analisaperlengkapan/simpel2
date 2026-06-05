use super::KebutuhanBmnService;
use crate::kebutuhan_bmn::models::*;
use crate::kebutuhan_bmn::repository::UserInfo;
use crate::shared::error::{AppError, AppResult};
use tracing::{info, warn};
use uuid::Uuid;

impl KebutuhanBmnService {
    /// Batch approve multiple kebutuhan
    ///
    /// Processes each item independently - failures don't affect other items
    /// Requirements: REQ-K004
    pub async fn batch_approve_kebutuhan(
        &self,
        request: crate::kebutuhan_bmn::models::BatchApproveRequest,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
        client_ip: String,
    ) -> AppResult<crate::kebutuhan_bmn::models::BatchOperationResponse> {
        use crate::kebutuhan_bmn::models::{BatchOperationItemResult, BatchOperationResponse};
        use chrono::Utc;

        // Validate batch size
        if request.kebutuhan_ids.is_empty() {
            return Err(AppError::BadRequest("Batch cannot be empty".to_string()));
        }
        if request.kebutuhan_ids.len() > 500 {
            return Err(AppError::BadRequest(
                "Batch size cannot exceed 500 items".to_string(),
            ));
        }

        let batch_id = Uuid::new_v4();
        let operation_type = "batch_approve".to_string();
        let executed_at = Utc::now();

        info!(
            "Starting batch approve operation {} for {} items by user {:?}",
            batch_id,
            request.kebutuhan_ids.len(),
            user_id
        );

        // Process each item independently
        let mut results = Vec::new();
        let mut successful_count = 0;
        let mut failed_count = 0;

        for kebutuhan_id in &request.kebutuhan_ids {
            let result = match self
                .process_single_approval(
                    *kebutuhan_id,
                    &request.komentar,
                    user_id,
                    user_info.clone(),
                    client_ip.clone(),
                )
                .await
            {
                Ok(_) => {
                    successful_count += 1;
                    BatchOperationItemResult {
                        kebutuhan_id: *kebutuhan_id,
                        success: true,
                        error_message: None,
                    }
                }
                Err(e) => {
                    failed_count += 1;
                    warn!("Failed to approve kebutuhan {}: {}", kebutuhan_id, e);
                    BatchOperationItemResult {
                        kebutuhan_id: *kebutuhan_id,
                        success: false,
                        error_message: Some(e.to_string()),
                    }
                }
            };
            results.push(result);
        }

        // Audit log the batch operation
        self.log_batch_operation(
            batch_id,
            &operation_type,
            request.kebutuhan_ids.len(),
            successful_count,
            failed_count,
            user_id,
        )
        .await?;

        info!(
            "Batch approve operation {} completed: {}/{} successful",
            batch_id,
            successful_count,
            request.kebutuhan_ids.len()
        );

        Ok(BatchOperationResponse {
            batch_id,
            total_items: request.kebutuhan_ids.len(),
            successful_items: successful_count,
            failed_items: failed_count,
            results,
            operation_type,
            executed_at,
            executed_by: user_id,
        })
    }

    /// Batch reject multiple kebutuhan
    ///
    /// Processes each item independently - failures don't affect other items
    /// Requirements: REQ-K004
    pub async fn batch_reject_kebutuhan(
        &self,
        request: crate::kebutuhan_bmn::models::BatchRejectRequest,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
        client_ip: String,
    ) -> AppResult<crate::kebutuhan_bmn::models::BatchOperationResponse> {
        use crate::kebutuhan_bmn::models::{BatchOperationItemResult, BatchOperationResponse};
        use chrono::Utc;

        // Validate batch size
        if request.kebutuhan_ids.is_empty() {
            return Err(AppError::BadRequest("Batch cannot be empty".to_string()));
        }
        if request.kebutuhan_ids.len() > 500 {
            return Err(AppError::BadRequest(
                "Batch size cannot exceed 500 items".to_string(),
            ));
        }

        let batch_id = Uuid::new_v4();
        let operation_type = "batch_reject".to_string();
        let executed_at = Utc::now();

        info!(
            "Starting batch reject operation {} for {} items by user {:?}",
            batch_id,
            request.kebutuhan_ids.len(),
            user_id
        );

        // Process each item independently
        let mut results = Vec::new();
        let mut successful_count = 0;
        let mut failed_count = 0;

        for kebutuhan_id in &request.kebutuhan_ids {
            let result = match self
                .process_single_rejection(
                    *kebutuhan_id,
                    &request.komentar,
                    user_id,
                    user_info.clone(),
                    client_ip.clone(),
                )
                .await
            {
                Ok(_) => {
                    successful_count += 1;
                    BatchOperationItemResult {
                        kebutuhan_id: *kebutuhan_id,
                        success: true,
                        error_message: None,
                    }
                }
                Err(e) => {
                    failed_count += 1;
                    warn!("Failed to reject kebutuhan {}: {}", kebutuhan_id, e);
                    BatchOperationItemResult {
                        kebutuhan_id: *kebutuhan_id,
                        success: false,
                        error_message: Some(e.to_string()),
                    }
                }
            };
            results.push(result);
        }

        // Audit log the batch operation
        self.log_batch_operation(
            batch_id,
            &operation_type,
            request.kebutuhan_ids.len(),
            successful_count,
            failed_count,
            user_id,
        )
        .await?;

        info!(
            "Batch reject operation {} completed: {}/{} successful",
            batch_id,
            successful_count,
            request.kebutuhan_ids.len()
        );

        Ok(BatchOperationResponse {
            batch_id,
            total_items: request.kebutuhan_ids.len(),
            successful_items: successful_count,
            failed_items: failed_count,
            results,
            operation_type,
            executed_at,
            executed_by: user_id,
        })
    }

    /// Batch update status for multiple kebutuhan
    ///
    /// Processes each item independently - failures don't affect other items
    /// Requirements: REQ-K004
    pub async fn batch_update_status(
        &self,
        request: crate::kebutuhan_bmn::models::BatchUpdateStatusRequest,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
        client_ip: String,
    ) -> AppResult<crate::kebutuhan_bmn::models::BatchOperationResponse> {
        use crate::kebutuhan_bmn::models::{BatchOperationItemResult, BatchOperationResponse};
        use chrono::Utc;

        // Validate batch size
        if request.kebutuhan_ids.is_empty() {
            return Err(AppError::BadRequest("Batch cannot be empty".to_string()));
        }
        if request.kebutuhan_ids.len() > 500 {
            return Err(AppError::BadRequest(
                "Batch size cannot exceed 500 items".to_string(),
            ));
        }

        // Validate target status
        let target_status = KebutuhanBmnStatus::from_code(request.target_status)
            .ok_or_else(|| AppError::BadRequest("Invalid target status code".to_string()))?;

        let batch_id = Uuid::new_v4();
        let operation_type = format!("batch_update_status_{}", target_status.label());
        let executed_at = Utc::now();

        info!(
            "Starting batch update status operation {} for {} items to {} by user {:?}",
            batch_id,
            request.kebutuhan_ids.len(),
            target_status.label(),
            user_id
        );

        // Process each item independently
        let mut results = Vec::new();
        let mut successful_count = 0;
        let mut failed_count = 0;

        for kebutuhan_id in &request.kebutuhan_ids {
            let result = match self
                .process_single_status_update(
                    *kebutuhan_id,
                    request.target_status,
                    &request.komentar,
                    user_id,
                    user_info.clone(),
                    client_ip.clone(),
                )
                .await
            {
                Ok(_) => {
                    successful_count += 1;
                    BatchOperationItemResult {
                        kebutuhan_id: *kebutuhan_id,
                        success: true,
                        error_message: None,
                    }
                }
                Err(e) => {
                    failed_count += 1;
                    warn!(
                        "Failed to update status for kebutuhan {}: {}",
                        kebutuhan_id, e
                    );
                    BatchOperationItemResult {
                        kebutuhan_id: *kebutuhan_id,
                        success: false,
                        error_message: Some(e.to_string()),
                    }
                }
            };
            results.push(result);
        }

        // Audit log the batch operation
        self.log_batch_operation(
            batch_id,
            &operation_type,
            request.kebutuhan_ids.len(),
            successful_count,
            failed_count,
            user_id,
        )
        .await?;

        info!(
            "Batch update status operation {} completed: {}/{} successful",
            batch_id,
            successful_count,
            request.kebutuhan_ids.len()
        );

        Ok(BatchOperationResponse {
            batch_id,
            total_items: request.kebutuhan_ids.len(),
            successful_items: successful_count,
            failed_items: failed_count,
            results,
            operation_type,
            executed_at,
            executed_by: user_id,
        })
    }

    // ========================================================================
    // Batch Operation Helpers
    // ========================================================================

    /// Process approval for a single kebutuhan
    async fn process_single_approval(
        &self,
        kebutuhan_id: Uuid,
        komentar: &Option<String>,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
        client_ip: String,
    ) -> AppResult<()> {
        let transition_request = WorkflowTransitionRequest {
            target_status: KebutuhanBmnStatus::Approved.to_code(),
            komentar: komentar.clone(),
        };

        self.transition_pengajuan_status(
            kebutuhan_id,
            transition_request,
            user_id,
            user_info,
            client_ip,
        )
        .await?;

        Ok(())
    }

    /// Process rejection for a single kebutuhan
    async fn process_single_rejection(
        &self,
        kebutuhan_id: Uuid,
        komentar: &str,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
        client_ip: String,
    ) -> AppResult<()> {
        let transition_request = WorkflowTransitionRequest {
            target_status: KebutuhanBmnStatus::Rejected.to_code(),
            komentar: Some(komentar.to_string()),
        };

        self.transition_pengajuan_status(
            kebutuhan_id,
            transition_request,
            user_id,
            user_info,
            client_ip,
        )
        .await?;

        Ok(())
    }

    /// Process status update for a single kebutuhan
    async fn process_single_status_update(
        &self,
        kebutuhan_id: Uuid,
        target_status: i32,
        komentar: &Option<String>,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
        client_ip: String,
    ) -> AppResult<()> {
        let transition_request = WorkflowTransitionRequest {
            target_status,
            komentar: komentar.clone(),
        };

        self.transition_pengajuan_status(
            kebutuhan_id,
            transition_request,
            user_id,
            user_info,
            client_ip,
        )
        .await?;

        Ok(())
    }

    /// Log batch operation to audit trail
    async fn log_batch_operation(
        &self,
        batch_id: Uuid,
        operation_type: &str,
        total_items: usize,
        successful_items: usize,
        failed_items: usize,
        user_id: Option<Uuid>,
    ) -> AppResult<()> {
        let query = r#"
            INSERT INTO perlengkapan.batch_operation_log (
                batch_id,
                operation_type,
                total_items,
                successful_items,
                failed_items,
                user_id,
                created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, NOW())
        "#;

        self.repository
            .pool()
            .get()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .execute(
                query,
                &[
                    &batch_id,
                    &operation_type,
                    &(total_items as i32),
                    &(successful_items as i32),
                    &(failed_items as i32),
                    &user_id,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}
