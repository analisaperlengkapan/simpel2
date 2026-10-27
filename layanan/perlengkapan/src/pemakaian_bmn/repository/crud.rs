use super::PemakaianBmnRepository;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::repo::PoolExt;
use crate::shared::satker_scope::{BoxedParam, SatkerScope, as_refs};
use uuid::Uuid;

impl PemakaianBmnRepository {
    /// Create a new permit
    /// Requirements: REQ-P001
    pub async fn create(
        &self,
        request: CreateIzinPemakaianRequest,
        created_by: Uuid,
        created_by_nama: String,
        // Authoritative MySIMKARI satker_code of the creating operator (from JWT
        // claims, #66) — persisted for RBAC scoping; NOT the client-supplied
        // pegawai_satker_id UUID.
        satker_code: Option<String>,
    ) -> AppResult<IzinPemakaianBmn> {
        let mut client = self.pool.client().await?;
        // Header and items in ONE transaction. A permit whose asset list is
        // half-written is not a smaller permit, it is a wrong one — the SK
        // would print the assets that made it and the conflict check would
        // free the ones that did not.
        let tx = client
            .transaction()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let id = Uuid::new_v4();
        let is_renewal = request.is_renewal.unwrap_or(false);

        let query = r#"
            INSERT INTO perlengkapan.izin_pemakaian_bmn (
                id, pegawai_nip, pegawai_nama, pegawai_satker_id, pegawai_satker_nama, pegawai_jabatan,
                pegawai_golongan, pegawai_pangkat, pegawai_unit_kerja, foto_pegawai,
                tanggal_mulai, tanggal_selesai, keperluan,
                is_renewal, previous_permit_id, file_pendukung,
                status, created_by, created_by_nama, satker_code, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10,
                $11, $12, $13,
                $14, $15, $16,
                'DRAFT', $17, $18, $19, NOW(), NOW()
            )
            RETURNING *
        "#;

        let row = tx
            .query_one(
                query,
                &[
                    &id,
                    &request.pegawai_nip,
                    &request.pegawai_nama,
                    &request.pegawai_satker_id,
                    &request.pegawai_satker_nama,
                    &request.pegawai_jabatan,
                    &request.pegawai_golongan,
                    &request.pegawai_pangkat,
                    &request.pegawai_unit_kerja,
                    &request.foto_pegawai,
                    &request.tanggal_mulai,
                    &request.tanggal_selesai,
                    &request.keperluan,
                    &is_renewal,
                    &request.previous_permit_id,
                    &request.file_pendukung,
                    &created_by,
                    &created_by_nama,
                    &satker_code,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let items = Self::insert_items(
            &tx,
            id,
            &request.items,
            request.tanggal_mulai,
            request.tanggal_selesai,
        )
        .await?;
        let row = Self::sync_permit_window(&tx, id).await?.unwrap_or(row);

        tx.commit()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut permit = self.row_to_permit(row);
        permit.bmn_items = items;
        Ok(permit)
    }

    /// Get permit by ID, restricted to `scope`.
    ///
    /// A permit is a named person holding a named asset for a stated period —
    /// the very thing the monitoring surfaces were scoped for. The LIST has
    /// been scoped since #66; this, the detail behind it, was not, and neither
    /// were the eleven other places that reach a permit by id.
    ///
    /// `scope` is in the signature rather than left to the callers because
    /// that is the third time the same shape has bitten this repository: #93
    /// scoped one kebutuhan endpoint and left five siblings, and the same
    /// happened in pakaian dinas. A rule the compiler enforces cannot be
    /// applied once and forgotten at the next call site.
    ///
    /// Requirements: REQ-P001
    pub async fn get_by_id(&self, id: Uuid, scope: &SatkerScope) -> AppResult<IzinPemakaianBmn> {
        let client = self.pool.client().await?;

        let mut params: Vec<BoxedParam> = vec![Box::new(id)];
        let scope_sql = crate::shared::satker_scope::scope_and(scope, "satker_code", &mut params);
        let query =
            format!("SELECT * FROM perlengkapan.izin_pemakaian_bmn WHERE id = $1{scope_sql}");

        let row = client
            .query_opt(&query, &as_refs(&params))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            // NotFound, not Forbidden: 403 confirms the permit exists under
            // another satker, which is the cross-tenant existence oracle #93
            // closed for satker detail.
            .ok_or_else(|| AppError::NotFound(format!("Izin pemakaian not found: {}", id)))?;

        // The asset list comes with the permit, always. Since V014 a permit
        // with an empty `bmn_items` is not a permit with no assets, it is a
        // permit that was not fully read — and the caller cannot tell those
        // apart. Loading it here is why `renew_permit` and the SK renderer
        // can trust what they are handed.
        let mut permit = self.row_to_permit(row);
        permit.bmn_items = self.get_bmn_items(id).await?;
        Ok(permit)
    }

    /// Update permit (only in DRAFT status)
    /// Requirements: REQ-P001
    pub async fn update(
        &self,
        id: Uuid,
        request: UpdateIzinPemakaianRequest,
        updated_by: Uuid,
        updated_by_nama: String,
    ) -> AppResult<IzinPemakaianBmn> {
        let mut client = self.pool.client().await?;

        // First check if permit is in DRAFT status. `All`: the caller's scope
        // was already applied by the service that reached this update, and a
        // second narrowing here would only make the status message wrong.
        let current = self.get_by_id(id, &SatkerScope::All).await?;
        if current.status != "DRAFT" {
            return Err(AppError::BadRequest(
                "Hanya izin dengan status DRAFT yang dapat diubah".to_string(),
            ));
        }

        let tx = client
            .transaction()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let query = r#"
            UPDATE perlengkapan.izin_pemakaian_bmn
            SET tanggal_mulai = COALESCE($1, tanggal_mulai),
                tanggal_selesai = COALESCE($2, tanggal_selesai),
                keperluan = COALESCE($3, keperluan),
                file_pendukung = COALESCE($4, file_pendukung),
                updated_by = $5,
                updated_by_nama = $6,
                updated_at = NOW()
            WHERE id = $7
            RETURNING *
        "#;

        let row = tx
            .query_one(
                query,
                &[
                    &request.tanggal_mulai,
                    &request.tanggal_selesai,
                    &request.keperluan,
                    &request.file_pendukung,
                    &updated_by,
                    &updated_by_nama,
                    &id,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // The permit's period after this update is what the items default to,
        // so it is read back from the row rather than from the request — the
        // request may well have left it alone.
        let mulai: chrono::NaiveDate = row.get("tanggal_mulai");
        let selesai: chrono::NaiveDate = row.get("tanggal_selesai");

        let (row, items) = match request.items.as_deref() {
            Some(items) => {
                let ditulis = Self::replace_items(&tx, id, items, mulai, selesai).await?;
                let row = Self::sync_permit_window(&tx, id).await?.unwrap_or(row);
                (row, ditulis)
            }
            // List untouched: read it back so the caller gets a whole permit
            // either way. A response whose `bmn_items` is empty only because
            // the client did not mention them would read as "assets removed".
            None => {
                let rows = tx
                    .query(
                        "SELECT * FROM perlengkapan.pemakaian_bmn_items \
                         WHERE izin_pemakaian_id = $1 ORDER BY urutan",
                        &[&id],
                    )
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;
                (row, rows.iter().map(PemakaianBmnItem::from_row).collect())
            }
        };

        tx.commit()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut permit = self.row_to_permit(row);
        permit.bmn_items = items;
        Ok(permit)
    }

    /// Update permit status
    /// Requirements: REQ-P004
    pub async fn update_status(
        &self,
        id: Uuid,
        status: &str,
        user_id: Uuid,
        user_nama: String,
        catatan: Option<String>,
    ) -> AppResult<IzinPemakaianBmn> {
        let client = self.pool.client().await?;

        let query = match status {
            "APPROVED" => {
                r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET status = $1,
                    approved_by = $2,
                    approved_by_nama = $3,
                    approved_at = NOW(),
                    catatan_approval = $4,
                    updated_by = $2,
                    updated_by_nama = $3,
                    updated_at = NOW()
                WHERE id = $5
                RETURNING *
            "#
            }
            "REVOKED" => {
                r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET status = $1,
                    revoked_by = $2,
                    revoked_by_nama = $3,
                    revoked_at = NOW(),
                    catatan_revocation = $4,
                    updated_by = $2,
                    updated_by_nama = $3,
                    updated_at = NOW()
                WHERE id = $5
                RETURNING *
            "#
            }
            _ => {
                r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET status = $1,
                    catatan_approval = $4,
                    updated_by = $2,
                    updated_by_nama = $3,
                    updated_at = NOW()
                WHERE id = $5
                RETURNING *
            "#
            }
        };

        let row = client
            .query_one(query, &[&status, &user_id, &user_nama, &catatan, &id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(self.row_to_permit(row))
    }

    /// Keep the denormalized `status_kode` aligned with `status`.
    ///
    /// The generic [`WorkflowEngine`] only writes the canonical `status`
    /// (string) column. pemakaian additionally denormalizes `status_kode`
    /// (int), and the internal-satker workflow methods gate on it
    /// (`WHERE status_kode = 3001 / 3010`). Engine-driven transitions
    /// (submit DRAFT→SUBMITTED, activate APPROVED→ACTIVE) therefore must
    /// re-sync `status_kode`, or the next role action sees a stale code and
    /// no-ops. Called right after the engine transition in
    /// `transition_permit_status`.
    pub async fn sync_status_kode(&self, id: Uuid, status_kode: i32) -> AppResult<()> {
        let client = self.pool.client().await?;
        client
            .execute(
                "UPDATE perlengkapan.izin_pemakaian_bmn SET status_kode = $1 WHERE id = $2",
                &[&status_kode, &id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // V035 (Fase 1.5): Internal-satker 3-step approval transitions.
    //
    // Setiap transisi memakai optimistic lock (kolom `version`) — jika dua
    // user menekan tombol bersamaan, hanya satu UPDATE yg menang; yg lain
    // mendapat `Conflict` dan FE wajib refresh.
    // ========================================================================
}
