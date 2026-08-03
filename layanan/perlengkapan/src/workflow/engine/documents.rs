use super::*;
use chrono::Utc;
use uuid::Uuid;

impl WorkflowEngine {
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
    pub(crate) async fn generate_document_for_entity(
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
                // Neither `perlengkapan.kebutuhan_bmn` nor `perlengkapan.ms_satker`
                // has ever existed. The real entity is the campaign
                // `pengajuan_kebutuhan_bmn`, whose columns are `tahun` (not
                // `tahun_anggaran`) and `status_kode` (an integer, not a string
                // `status`). Because the caller swallows errors (see
                // `transition.rs`), approving a kebutuhan campaign silently
                // produced no SK at all.
                //
                // `satker_nama` stays in the payload so the template contract is
                // unchanged, but a campaign is national by construction and fans
                // out to N rows in `pengajuan_kebutuhan_bmn_satker` — there is no
                // single satker to name, so it is empty exactly as the old
                // `LEFT JOIN` miss would have rendered it.
                let query = r#"
                    SELECT id, nama, tahun, status_kode, created_at
                    FROM perlengkapan.pengajuan_kebutuhan_bmn
                    WHERE id = $1
                "#;

                let row = client.query_one(query, &[entity_id]).await?;

                let template_id = Self::resolve_template_id(&client, "sk_kebutuhan_bmn").await?;

                let entity_data = serde_json::json!({
                    "id": entity_id.to_string(),
                    "satker_nama": "",
                    "nama_kegiatan": row.get::<_, String>("nama"),
                    "tahun_anggaran": row.get::<_, i32>("tahun"),
                    "status": row.get::<_, i32>("status_kode").to_string(),
                    "created_at": row
                        .get::<_, Option<chrono::DateTime<Utc>>>("created_at")
                        .map(|t| t.to_rfc3339())
                        .unwrap_or_default(),
                    "document_type": "SK Kebutuhan BMN",
                    "approval_date": chrono::Utc::now().format("%d %B %Y").to_string(),
                });

                (template_id, entity_data)
            }
            "penghapusan_bmn" => {
                // Same phantom `perlengkapan.ms_satker` join as the arm above, so
                // auto-SK on approval was silently broken here too. The table
                // itself is real; only the join was not.
                //
                // It cannot simply be repointed at the satker SoT
                // (`integrasi.mysimkari_satker`): that table keys on
                // `kode_satker TEXT` while `penghapusan_bmn.satker_id` is a
                // `uuid`, the same key-type mismatch as #94. `satker_nama` is
                // therefore left empty — which is what the old `LEFT JOIN` miss
                // already rendered — rather than inventing a join that cannot
                // match. Resolving it properly needs the satker-identity work.
                let query = r#"
                    SELECT id, alasan, status, created_at
                    FROM perlengkapan.penghapusan_bmn
                    WHERE id = $1
                "#;

                let row = client.query_one(query, &[entity_id]).await?;

                let template_id = Self::resolve_template_id(&client, "sk_penghapusan").await?;

                let entity_data = serde_json::json!({
                    "id": entity_id.to_string(),
                    "satker_nama": "",
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

        let request = crate::contracts::DocumentRequest {
            template_id: template_id.to_string(),
            format: crate::contracts::DocumentFormat::Pdf,
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
}
