use super::PemakaianBmnRepository;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::repo::PoolExt;
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
        let client = self.pool.client().await?;

        let id = Uuid::new_v4();
        let is_renewal = request.is_renewal.unwrap_or(false);

        let query = r#"
            INSERT INTO perlengkapan.izin_pemakaian_bmn (
                id, pegawai_nip, pegawai_nama, pegawai_satker_id, pegawai_satker_nama, pegawai_jabatan,
                jenis_bmn, bmn_nup, bmn_kode_barang, bmn_nama_barang, bmn_merk, bmn_tahun_perolehan,
                no_polisi, no_bpkb, no_stnk, no_rangka, no_mesin,
                alamat, luas_tanah, luas_bangunan,
                serial_number, spesifikasi,
                tanggal_mulai, tanggal_selesai, keperluan, lokasi_pemakaian,
                is_renewal, previous_permit_id, file_pendukung,
                status, created_by, created_by_nama, satker_code, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10, $11, $12,
                $13, $14, $15, $16, $17,
                $18, $19, $20,
                $21, $22,
                $23, $24, $25, $26,
                $27, $28, $29,
                'DRAFT', $30, $31, $32, NOW(), NOW()
            )
            RETURNING *
        "#;

        let row = client
            .query_one(
                query,
                &[
                    &id,
                    &request.pegawai_nip,
                    &request.pegawai_nama,
                    &request.pegawai_satker_id,
                    &request.pegawai_satker_nama,
                    &request.pegawai_jabatan,
                    &request.jenis_bmn,
                    &request.bmn_nup,
                    &request.bmn_kode_barang,
                    &request.bmn_nama_barang,
                    &request.bmn_merk,
                    &request.bmn_tahun_perolehan,
                    &request.no_polisi,
                    &request.no_bpkb,
                    &request.no_stnk,
                    &request.no_rangka,
                    &request.no_mesin,
                    &request.alamat,
                    &request.luas_tanah,
                    &request.luas_bangunan,
                    &request.serial_number,
                    &request.spesifikasi,
                    &request.tanggal_mulai,
                    &request.tanggal_selesai,
                    &request.keperluan,
                    &request.lokasi_pemakaian,
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

        Ok(self.row_to_permit(row))
    }

    /// Get permit by ID
    /// Requirements: REQ-P001
    pub async fn get_by_id(&self, id: Uuid) -> AppResult<IzinPemakaianBmn> {
        let client = self.pool.client().await?;

        let query = r#"
            SELECT * FROM perlengkapan.izin_pemakaian_bmn
            WHERE id = $1
        "#;

        let row = client
            .query_opt(query, &[&id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("Izin pemakaian not found: {}", id)))?;

        Ok(self.row_to_permit(row))
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
        let client = self.pool.client().await?;

        // First check if permit is in DRAFT status
        let current = self.get_by_id(id).await?;
        if current.status != "DRAFT" {
            return Err(AppError::BadRequest(
                "Hanya izin dengan status DRAFT yang dapat diubah".to_string(),
            ));
        }

        let query = r#"
            UPDATE perlengkapan.izin_pemakaian_bmn
            SET tanggal_mulai = COALESCE($1, tanggal_mulai),
                tanggal_selesai = COALESCE($2, tanggal_selesai),
                keperluan = COALESCE($3, keperluan),
                lokasi_pemakaian = COALESCE($4, lokasi_pemakaian),
                file_pendukung = COALESCE($5, file_pendukung),
                updated_by = $6,
                updated_by_nama = $7,
                updated_at = NOW()
            WHERE id = $8
            RETURNING *
        "#;

        let row = client
            .query_one(
                query,
                &[
                    &request.tanggal_mulai,
                    &request.tanggal_selesai,
                    &request.keperluan,
                    &request.lokasi_pemakaian,
                    &request.file_pendukung,
                    &updated_by,
                    &updated_by_nama,
                    &id,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(self.row_to_permit(row))
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
