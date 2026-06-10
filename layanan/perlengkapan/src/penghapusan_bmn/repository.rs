// ============================================================================
// Penghapusan BMN Repository
// Description: Database operations for SK Penghapusan BMN workflow
// Requirements: REQ-W001
// ============================================================================

use super::models::*;
use crate::shared::error::{AppError, AppResult};
use deadpool_postgres::Pool;
use uuid::Uuid;

pub struct PenghapusanBmnRepository {
    pool: Pool,
}

impl PenghapusanBmnRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Create a new penghapusan BMN record
    pub async fn create(
        &self,
        request: CreatePenghapusanBmnRequest,
        created_by: Uuid,
    ) -> AppResult<PenghapusanBmn> {
        let client = self.pool.get().await?;

        // V029 (Fase 0.5): kolom kanonik `nilai_perolehan`; legacy `nilai_residu`
        // di-skip untuk INSERT baru (NULL); migrasi mem-backfill row lama.
        // V029 (Fase 1.9): validasi + persist `kewenangan_penetap_sk`.
        let kewenangan = request.kewenangan_penetap_sk.to_uppercase();
        if !["PUSAT", "WILAYAH"].contains(&kewenangan.as_str()) {
            return Err(AppError::BadRequest(
                "kewenangan_penetap_sk harus 'PUSAT' atau 'WILAYAH'".into(),
            ));
        }

        let query = r#"
            INSERT INTO perlengkapan.penghapusan_bmn (
                id, satker_id, asset_id, kode_barang, nama_barang, nup,
                tanggal_penghapusan, alasan, metode_penghapusan, nilai_perolehan,
                nilai_perolehan_dari_backfill,
                status, status_kode, lampiran_persyaratan, lampiran_pendukung,
                catatan_operator, is_completed,
                kewenangan_penetap_sk, penetap_sk_jabatan,
                created_by, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                false,
                $11, $12, $13, $14, $15, false,
                $16, $17,
                $18, NOW(), NOW()
            )
            RETURNING *
        "#;

        let id = Uuid::new_v4();
        let status = PenghapusanBmnStatus::Draft;
        let lampiran_pendukung: Option<serde_json::Value> = None;

        let row = client
            .query_one(
                query,
                &[
                    &id,
                    &request.satker_id,
                    &request.asset_id,
                    &request.kode_barang,
                    &request.nama_barang,
                    &request.nup,
                    &request.tanggal_penghapusan,
                    &request.alasan,
                    &request.metode_penghapusan,
                    &request.nilai_perolehan,
                    &status.to_state_name(),
                    &status.to_code(),
                    &request.lampiran_persyaratan,
                    &lampiran_pendukung,
                    &request.catatan_operator,
                    &kewenangan,
                    &request.penetap_sk_jabatan,
                    &created_by,
                ],
            )
            .await?;

        // V036 (Fase 2.8): persist item-item BMN. Bila request multi-item
        // kosong, fallback ke satu item dari kolom tunggal (backward compat).
        // Tabel induk tetap menyimpan item pertama (kolom tunggal di atas).
        let items: Vec<CreatePenghapusanBmnItemRequest> = if request.items.is_empty() {
            vec![CreatePenghapusanBmnItemRequest {
                asset_id: Some(request.asset_id),
                kode_barang: request.kode_barang.clone(),
                nama_barang: request.nama_barang.clone(),
                nup: request.nup.clone(),
                nilai_perolehan: request.nilai_perolehan,
                kondisi: None,
            }]
        } else {
            request.items.clone()
        };
        for (idx, item) in items.iter().enumerate() {
            client
                .execute(
                    r#"
                    INSERT INTO perlengkapan.penghapusan_bmn_item (
                        penghapusan_id, asset_id, kode_barang, nama_barang, nup,
                        nilai_perolehan, kondisi, urutan
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    "#,
                    &[
                        &id,
                        &item.asset_id,
                        &item.kode_barang,
                        &item.nama_barang,
                        &item.nup,
                        &item.nilai_perolehan,
                        &item.kondisi,
                        &((idx as i32) + 1),
                    ],
                )
                .await?;
        }

        Ok(PenghapusanBmn::from_row(&row))
    }

    /// Daftar item BMN dalam satu usulan (Fase 2.8), terurut `urutan`.
    pub async fn list_items(&self, penghapusan_id: Uuid) -> AppResult<Vec<PenghapusanBmnItem>> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT * FROM perlengkapan.penghapusan_bmn_item
                 WHERE penghapusan_id = $1 ORDER BY urutan ASC, created_at ASC",
                &[&penghapusan_id],
            )
            .await?;
        Ok(rows.iter().map(PenghapusanBmnItem::from_row).collect())
    }

    // ========================================================================
    // V029 (Fase 1.9): SK Wilayah update methods (paralel dgn jalur PUSAT)
    // ========================================================================

    /// Update konsep SK URLs jalur WILAYAH + transition state ke
    /// KonsepSKWilayahGenerated.
    pub async fn update_konsep_sk_wilayah(
        &self,
        id: Uuid,
        konsep_sk_docx_url: &str,
        konsep_sk_pdf_url: Option<&str>,
    ) -> AppResult<()> {
        let client = self.pool.get().await?;
        let new_status = PenghapusanBmnStatus::KonsepSKWilayahGenerated;
        let query = r#"
            UPDATE perlengkapan.penghapusan_bmn
            SET konsep_sk_wilayah_url = $1,
                konsep_sk_wilayah_pdf_url = $2,
                konsep_sk_wilayah_generated_at = NOW(),
                status = $3,
                status_kode = $4,
                updated_at = NOW()
            WHERE id = $5
        "#;
        client
            .execute(
                query,
                &[
                    &konsep_sk_docx_url,
                    &konsep_sk_pdf_url,
                    &new_status.to_state_name(),
                    &new_status.to_code(),
                    &id,
                ],
            )
            .await?;
        Ok(())
    }

    /// Upload signed SK PDF jalur WILAYAH + transition state ke
    /// SKSignedWilayah (lalu service auto-transition ke Completed via flag
    /// is_completed=true).
    pub async fn update_signed_sk_wilayah(
        &self,
        id: Uuid,
        signed_sk_pdf_url: &str,
    ) -> AppResult<()> {
        let client = self.pool.get().await?;
        let new_status = PenghapusanBmnStatus::SKSignedWilayah;
        let query = r#"
            UPDATE perlengkapan.penghapusan_bmn
            SET signed_sk_wilayah_pdf_url = $1,
                signed_sk_wilayah_pdf_uploaded_at = NOW(),
                status = $2,
                status_kode = $3,
                is_completed = true,
                updated_at = NOW()
            WHERE id = $4
        "#;
        client
            .execute(
                query,
                &[
                    &signed_sk_pdf_url,
                    &new_status.to_state_name(),
                    &new_status.to_code(),
                    &id,
                ],
            )
            .await?;
        Ok(())
    }

    /// Get penghapusan BMN by ID
    pub async fn get_by_id(&self, id: Uuid) -> AppResult<PenghapusanBmn> {
        let client = self.pool.get().await?;

        let query = r#"
            SELECT * FROM perlengkapan.penghapusan_bmn
            WHERE id = $1
        "#;

        let row = client
            .query_opt(query, &[&id])
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Penghapusan BMN not found: {}", id)))?;

        Ok(PenghapusanBmn::from_row(&row))
    }

    /// List penghapusan BMN with filters and pagination
    pub async fn list(
        &self,
        filters: PenghapusanBmnFilters,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<PenghapusanBmn>, i64)> {
        let client = self.pool.get().await?;

        let mut where_clauses = vec!["1=1".to_string()];
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = vec![];
        let mut param_count = 1;

        if let Some(ref satker_id) = filters.satker_id {
            where_clauses.push(format!("satker_id = ${}", param_count));
            params.push(Box::new(*satker_id));
            param_count += 1;
        }

        if let Some(ref status) = filters.status {
            // Normalize a comma-separated list + legacy/UI aliases to the
            // canonical state names stored in the `status` column, then match
            // with ANY(...). Without this the FE reviewer/SK queues compared a
            // raw alias/comma string against a single canonical value → 0 rows.
            let has_tokens = status.split(',').any(|s| !s.trim().is_empty());
            let normalized = PenghapusanBmnStatus::normalize_status_filter(status);
            if !normalized.is_empty() {
                where_clauses.push(format!("status = ANY(${})", param_count));
                params.push(Box::new(normalized));
                param_count += 1;
            } else if has_tokens {
                // A status filter was requested but no token resolved to a real
                // state → match nothing, rather than silently widening to all
                // rows. (A blank `?status=` falls through as a no-op = all.)
                where_clauses.push("FALSE".to_string());
            }
        }

        if let Some(ref metode) = filters.metode_penghapusan {
            where_clauses.push(format!("metode_penghapusan = ${}", param_count));
            params.push(Box::new(metode.clone()));
            param_count += 1;
        }

        if let Some(ref tahun) = filters.tahun {
            where_clauses.push(format!(
                "EXTRACT(YEAR FROM tanggal_penghapusan) = ${}",
                param_count
            ));
            params.push(Box::new(*tahun));
            param_count += 1;
        }

        let where_clause = where_clauses.join(" AND ");

        // Count total
        let count_query = format!(
            "SELECT COUNT(*) FROM perlengkapan.penghapusan_bmn WHERE {}",
            where_clause
        );

        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let count_row = client.query_one(&count_query, &param_refs).await?;
        let total: i64 = count_row.get(0);

        // Get paginated data
        let offset = (page - 1) * per_page;
        let data_query = format!(
            "SELECT * FROM perlengkapan.penghapusan_bmn WHERE {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause,
            param_count,
            param_count + 1
        );

        params.push(Box::new(per_page));
        params.push(Box::new(offset));

        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = client.query(&data_query, &param_refs).await?;
        let penghapusan: Vec<PenghapusanBmn> = rows.iter().map(PenghapusanBmn::from_row).collect();

        Ok((penghapusan, total))
    }

    /// Update penghapusan BMN (Draft/ReturnedToOperator only)
    pub async fn update(
        &self,
        id: Uuid,
        request: UpdatePenghapusanBmnRequest,
    ) -> AppResult<PenghapusanBmn> {
        let client = self.pool.get().await?;

        // V029: Saat user menyimpan nilai_perolehan baru, anggap data sudah
        // diverifikasi → reset flag backfill ke FALSE.
        let query = r#"
            UPDATE perlengkapan.penghapusan_bmn
            SET tanggal_penghapusan = COALESCE($1, tanggal_penghapusan),
                alasan = COALESCE($2, alasan),
                metode_penghapusan = COALESCE($3, metode_penghapusan),
                nilai_perolehan = COALESCE($4, nilai_perolehan),
                nilai_perolehan_dari_backfill = CASE
                    WHEN $4::numeric IS NOT NULL THEN FALSE
                    ELSE nilai_perolehan_dari_backfill
                END,
                lampiran_persyaratan = COALESCE($5, lampiran_persyaratan),
                catatan_operator = COALESCE($6, catatan_operator),
                updated_at = NOW()
            WHERE id = $7
            RETURNING *
        "#;

        let row = client
            .query_opt(
                query,
                &[
                    &request.tanggal_penghapusan,
                    &request.alasan,
                    &request.metode_penghapusan,
                    &request.nilai_perolehan,
                    &request.lampiran_persyaratan,
                    &request.catatan_operator,
                    &id,
                ],
            )
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Penghapusan BMN not found: {}", id)))?;

        Ok(PenghapusanBmn::from_row(&row))
    }

    /// Delete penghapusan BMN (soft delete by setting status to CANCELLED)
    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        let client = self.pool.get().await?;

        let query = r#"
            UPDATE perlengkapan.penghapusan_bmn
            SET status = 'CANCELLED', updated_at = NOW()
            WHERE id = $1
        "#;

        let rows_affected = client.execute(query, &[&id]).await?;

        if rows_affected == 0 {
            return Err(AppError::NotFound(format!(
                "Penghapusan BMN not found: {}",
                id
            )));
        }

        Ok(())
    }

    /// Update status (used by workflow engine)
    pub async fn update_status(&self, id: Uuid, status: &str) -> AppResult<()> {
        let client = self.pool.get().await?;

        let status_kode = PenghapusanBmnStatus::from_state_name(status)
            .map(|s| s.to_code())
            .unwrap_or(0);

        let query = r#"
            UPDATE perlengkapan.penghapusan_bmn
            SET status = $1, status_kode = $2, updated_at = NOW()
            WHERE id = $3
        "#;

        let rows_affected = client.execute(query, &[&status, &status_kode, &id]).await?;

        if rows_affected == 0 {
            return Err(AppError::NotFound(format!(
                "Penghapusan BMN not found: {}",
                id
            )));
        }

        Ok(())
    }

    /// Update validator wilayah info
    pub async fn update_validator_wilayah(
        &self,
        id: Uuid,
        validator_id: Uuid,
        catatan: Option<String>,
    ) -> AppResult<()> {
        let client = self.pool.get().await?;

        let query = r#"
            UPDATE perlengkapan.penghapusan_bmn
            SET validator_wilayah_id = $1,
                catatan_validator_wilayah = COALESCE($2, catatan_validator_wilayah),
                tanggal_submit_wilayah = NOW(),
                updated_at = NOW()
            WHERE id = $3
        "#;

        client
            .execute(query, &[&validator_id, &catatan, &id])
            .await?;
        Ok(())
    }

    /// Update validator pusat info
    pub async fn update_validator_pusat(
        &self,
        id: Uuid,
        validator_id: Uuid,
        catatan: Option<String>,
    ) -> AppResult<()> {
        let client = self.pool.get().await?;

        let query = r#"
            UPDATE perlengkapan.penghapusan_bmn
            SET validator_pusat_id = $1,
                catatan_validator_pusat = COALESCE($2, catatan_validator_pusat),
                tanggal_submit_pusat = NOW(),
                updated_at = NOW()
            WHERE id = $3
        "#;

        client
            .execute(query, &[&validator_id, &catatan, &id])
            .await?;
        Ok(())
    }

    /// Update both DOCX and PDF konsep SK URLs and their on-disk paths in
    /// one statement. This is the only konsep-SK write path — DOCX + PDF
    /// are always produced together.
    pub async fn update_konsep_sk(
        &self,
        id: Uuid,
        docx_url: &str,
        docx_path: &str,
        pdf_url: &str,
        pdf_path: &str,
    ) -> AppResult<()> {
        let client = self.pool.get().await?;
        let query = r#"
            UPDATE perlengkapan.penghapusan_bmn
            SET konsep_sk_url = $1,
                konsep_sk_docx_path = $2,
                konsep_sk_generated_at = NOW(),
                konsep_sk_pdf_url = $3,
                konsep_sk_pdf_path = $4,
                konsep_sk_pdf_generated_at = NOW(),
                updated_at = NOW()
            WHERE id = $5
        "#;
        client
            .execute(query, &[&docx_url, &docx_path, &pdf_url, &pdf_path, &id])
            .await?;
        Ok(())
    }

    /// Fetch the on-disk path the route handler should stream from.
    pub async fn konsep_sk_path(&self, id: Uuid, format: &str) -> AppResult<Option<String>> {
        let column = match format {
            "docx" => "konsep_sk_docx_path",
            "pdf" => "konsep_sk_pdf_path",
            _ => return Ok(None),
        };
        let client = self.pool.get().await?;
        let query = format!(
            "SELECT {} FROM perlengkapan.penghapusan_bmn WHERE id = $1",
            column
        );
        let row = client.query_opt(&query, &[&id]).await?;
        Ok(row.and_then(|r| r.try_get::<_, Option<String>>(0).ok().flatten()))
    }

    /// Update signed SK PDF URL
    pub async fn update_signed_sk(&self, id: Uuid, signed_sk_pdf_url: &str) -> AppResult<()> {
        let client = self.pool.get().await?;

        let query = r#"
            UPDATE perlengkapan.penghapusan_bmn
            SET signed_sk_pdf_url = $1,
                signed_sk_uploaded_at = NOW(),
                is_completed = true,
                updated_at = NOW()
            WHERE id = $2
        "#;

        client.execute(query, &[&signed_sk_pdf_url, &id]).await?;
        Ok(())
    }

    /// Update document metadata (legacy, kept for backward compat)
    pub async fn update_document_metadata(
        &self,
        id: Uuid,
        document_id: Uuid,
        document_url: String,
    ) -> AppResult<()> {
        let client = self.pool.get().await?;

        let query = r#"
            UPDATE perlengkapan.penghapusan_bmn
            SET document_id = $1, document_url = $2, updated_at = NOW()
            WHERE id = $3
        "#;

        let rows_affected = client
            .execute(query, &[&document_id, &document_url, &id])
            .await?;

        if rows_affected == 0 {
            return Err(AppError::NotFound(format!(
                "Penghapusan BMN not found: {}",
                id
            )));
        }

        Ok(())
    }

    // ========================================================================
    // V030: File upload — Surat Usulan + Lampiran[]
    // ========================================================================

    /// Set Surat Usulan file URL (1 file per penghapusan; overwrite jika
    /// sudah ada).
    pub async fn set_surat_usulan_url(&self, id: Uuid, file_url: &str) -> AppResult<()> {
        let client = self.pool.get().await?;
        let rows = client
            .execute(
                r#"UPDATE perlengkapan.penghapusan_bmn
                   SET surat_usulan_file_url = $1,
                       surat_usulan_uploaded_at = NOW(),
                       updated_at = NOW()
                   WHERE id = $2"#,
                &[&file_url, &id],
            )
            .await?;
        if rows == 0 {
            return Err(AppError::NotFound(format!(
                "Penghapusan BMN not found: {}",
                id
            )));
        }
        Ok(())
    }

    /// Insert satu entry lampiran pendukung.
    pub async fn insert_lampiran(
        &self,
        penghapusan_id: Uuid,
        nama: &str,
        file_url: &str,
        content_type: Option<&str>,
        size_bytes: Option<i64>,
        uploaded_by: Option<Uuid>,
    ) -> AppResult<PenghapusanBmnLampiran> {
        let client = self.pool.get().await?;
        let row = client
            .query_one(
                r#"INSERT INTO perlengkapan.penghapusan_bmn_lampiran
                   (penghapusan_id, nama, file_url, content_type, size_bytes, uploaded_by)
                   VALUES ($1, $2, $3, $4, $5, $6)
                   RETURNING id, penghapusan_id, nama, file_url, content_type, size_bytes, uploaded_by, uploaded_at"#,
                &[
                    &penghapusan_id,
                    &nama,
                    &file_url,
                    &content_type,
                    &size_bytes,
                    &uploaded_by,
                ],
            )
            .await?;
        Ok(PenghapusanBmnLampiran::from_row(&row))
    }

    /// List semua lampiran utk satu penghapusan.
    pub async fn list_lampiran(
        &self,
        penghapusan_id: Uuid,
    ) -> AppResult<Vec<PenghapusanBmnLampiran>> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                r#"SELECT id, penghapusan_id, nama, file_url, content_type, size_bytes, uploaded_by, uploaded_at
                   FROM perlengkapan.penghapusan_bmn_lampiran
                   WHERE penghapusan_id = $1
                   ORDER BY uploaded_at ASC"#,
                &[&penghapusan_id],
            )
            .await?;
        Ok(rows.iter().map(PenghapusanBmnLampiran::from_row).collect())
    }
}
