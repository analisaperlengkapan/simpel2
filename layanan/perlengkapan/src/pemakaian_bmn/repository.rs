//! # Pemakaian BMN Repository
//!
//! Data access layer for BMN usage permits.
//! Requirements: REQ-P001, REQ-P002, REQ-P003, REQ-P008, REQ-P011, REQ-P012, REQ-P013

use deadpool_postgres::Pool;
use uuid::Uuid;

use crate::shared::error::{AppError, AppResult};
use crate::shared::repo::PoolExt;

use super::models::*;

/// Repository for pemakaian BMN data access
#[derive(Clone)]
pub struct PemakaianBmnRepository {
    pool: Pool,
}

impl PemakaianBmnRepository {
    /// Create a new repository instance
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Create a new permit
    /// Requirements: REQ-P001
    pub async fn create(
        &self,
        request: CreateIzinPemakaianRequest,
        created_by: Uuid,
        created_by_nama: String,
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
                status, created_by, created_by_nama, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10, $11, $12,
                $13, $14, $15, $16, $17,
                $18, $19, $20,
                $21, $22,
                $23, $24, $25, $26,
                $27, $28, $29,
                'DRAFT', $30, $31, NOW(), NOW()
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

    // ========================================================================
    // V035 (Fase 1.5): Internal-satker 3-step approval transitions.
    //
    // Setiap transisi memakai optimistic lock (kolom `version`) — jika dua
    // user menekan tombol bersamaan, hanya satu UPDATE yg menang; yg lain
    // mendapat `Conflict` dan FE wajib refresh.
    // ========================================================================

    /// Validator Satker meneruskan ke Approver Satker.
    /// State: SUBMITTED → SUBMITTED_APPROVER_SATKER (kode 3010).
    pub async fn validator_satker_forward(
        &self,
        id: Uuid,
        validator_id: Uuid,
        validator_nama: &str,
        expected_version: i32,
        catatan: Option<&str>,
    ) -> AppResult<IzinPemakaianBmn> {
        let client = self.pool.client().await?;
        let row_opt = client
            .query_opt(
                r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET status                   = 'SUBMITTED_APPROVER_SATKER',
                    status_kode              = 3010,
                    validator_satker_id      = $1,
                    validator_satker_nama    = $2,
                    tanggal_validasi_satker  = NOW(),
                    catatan_validator_satker = $3,
                    updated_by               = $1,
                    updated_by_nama          = $2,
                    updated_at               = NOW(),
                    version                  = version + 1
                WHERE id = $4 AND version = $5 AND status_kode = 3001
                RETURNING *
                "#,
                &[
                    &validator_id,
                    &validator_nama,
                    &catatan,
                    &id,
                    &expected_version,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        match row_opt {
            Some(row) => Ok(self.row_to_permit(row)),
            None => Err(AppError::Conflict(
                "Versi data sudah berubah atau status bukan SUBMITTED — silakan refresh".into(),
            )),
        }
    }

    /// Validator Satker mengembalikan ke Operator utk revisi.
    /// State: SUBMITTED → REVISI_OPERATOR (kode 3011).
    pub async fn validator_satker_return(
        &self,
        id: Uuid,
        validator_id: Uuid,
        validator_nama: &str,
        expected_version: i32,
        catatan: &str,
    ) -> AppResult<IzinPemakaianBmn> {
        if catatan.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Catatan revisi wajib diisi saat mengembalikan ke Operator".into(),
            ));
        }
        let client = self.pool.client().await?;
        let row_opt = client
            .query_opt(
                r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET status                   = 'REVISI_OPERATOR',
                    status_kode              = 3011,
                    validator_satker_id      = $1,
                    validator_satker_nama    = $2,
                    tanggal_validasi_satker  = NOW(),
                    catatan_validator_satker = $3,
                    updated_by               = $1,
                    updated_by_nama          = $2,
                    updated_at               = NOW(),
                    version                  = version + 1
                WHERE id = $4 AND version = $5
                  AND status_kode IN (3001, 3010)
                RETURNING *
                "#,
                &[
                    &validator_id,
                    &validator_nama,
                    &catatan,
                    &id,
                    &expected_version,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        match row_opt {
            Some(row) => Ok(self.row_to_permit(row)),
            None => Err(AppError::Conflict(
                "Versi data sudah berubah atau status tidak valid utk revisi — silakan refresh"
                    .into(),
            )),
        }
    }

    /// Approver Satker (Pengguna Barang Satker) menyetujui — siap aktivasi.
    /// State: SUBMITTED_APPROVER_SATKER → APPROVED.
    pub async fn approver_satker_approve(
        &self,
        id: Uuid,
        approver_id: Uuid,
        approver_nama: &str,
        expected_version: i32,
        catatan: Option<&str>,
    ) -> AppResult<IzinPemakaianBmn> {
        let client = self.pool.client().await?;
        let row_opt = client
            .query_opt(
                r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET status                   = 'APPROVED',
                    status_kode              = 3002,
                    approver_satker_id       = $1,
                    approver_satker_nama     = $2,
                    tanggal_approval_satker  = NOW(),
                    catatan_approver_satker  = $3,
                    approved_by              = $1,
                    approved_by_nama         = $2,
                    approved_at              = NOW(),
                    catatan_approval         = $3,
                    updated_by               = $1,
                    updated_by_nama          = $2,
                    updated_at               = NOW(),
                    version                  = version + 1
                WHERE id = $4 AND version = $5 AND status_kode = 3010
                RETURNING *
                "#,
                &[
                    &approver_id,
                    &approver_nama,
                    &catatan,
                    &id,
                    &expected_version,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        match row_opt {
            Some(row) => Ok(self.row_to_permit(row)),
            None => Err(AppError::Conflict(
                "Versi data berubah atau status bukan SUBMITTED_APPROVER_SATKER — silakan refresh"
                    .into(),
            )),
        }
    }

    /// Approver Satker mengembalikan ke Operator utk revisi.
    /// State: SUBMITTED_APPROVER_SATKER → REVISI_OPERATOR.
    pub async fn approver_satker_return(
        &self,
        id: Uuid,
        approver_id: Uuid,
        approver_nama: &str,
        expected_version: i32,
        catatan: &str,
    ) -> AppResult<IzinPemakaianBmn> {
        // Re-use validator_satker_return — guard kolom yg dipakai sama,
        // tapi audit field-nya `approver_satker_*` agar jelas siapa yg
        // menolak. Tetap simpan ke `catatan_validator_satker` agar
        // Operator melihat catatan terbaru di field umum.
        if catatan.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Catatan revisi wajib diisi saat mengembalikan ke Operator".into(),
            ));
        }
        let client = self.pool.client().await?;
        let row_opt = client
            .query_opt(
                r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET status                  = 'REVISI_OPERATOR',
                    status_kode             = 3011,
                    approver_satker_id      = $1,
                    approver_satker_nama    = $2,
                    tanggal_approval_satker = NOW(),
                    catatan_approver_satker = $3,
                    updated_by              = $1,
                    updated_by_nama         = $2,
                    updated_at              = NOW(),
                    version                 = version + 1
                WHERE id = $4 AND version = $5 AND status_kode = 3010
                RETURNING *
                "#,
                &[
                    &approver_id,
                    &approver_nama,
                    &catatan,
                    &id,
                    &expected_version,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        match row_opt {
            Some(row) => Ok(self.row_to_permit(row)),
            None => Err(AppError::Conflict(
                "Versi data berubah atau status bukan SUBMITTED_APPROVER_SATKER — silakan refresh"
                    .into(),
            )),
        }
    }

    /// Operator re-submit setelah revisi.
    /// State: REVISI_OPERATOR → SUBMITTED (kembali ke Validator Satker).
    pub async fn operator_resubmit(
        &self,
        id: Uuid,
        operator_id: Uuid,
        operator_nama: &str,
        expected_version: i32,
    ) -> AppResult<IzinPemakaianBmn> {
        let client = self.pool.client().await?;
        let row_opt = client
            .query_opt(
                r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET status          = 'SUBMITTED',
                    status_kode     = 3001,
                    updated_by      = $1,
                    updated_by_nama = $2,
                    updated_at      = NOW(),
                    version         = version + 1
                WHERE id = $3 AND version = $4 AND status_kode = 3011
                RETURNING *
                "#,
                &[&operator_id, &operator_nama, &id, &expected_version],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        match row_opt {
            Some(row) => Ok(self.row_to_permit(row)),
            None => Err(AppError::Conflict(
                "Versi data berubah atau status bukan REVISI_OPERATOR — silakan refresh".into(),
            )),
        }
    }

    /// Generate permit number
    /// Requirements: REQ-P005
    ///
    /// Format: `IP/YYYY/MM/NNNN` — sequential within each YYYY/MM bucket.
    ///
    /// Tanpa serialisasi, dua aktivasi paralel di bulan yg sama dpt membaca
    /// `MAX(num)` yg sama (snapshot CTE) dan menghasilkan dua `nomor_izin`
    /// identik → UNIQUE conflict atau nomor lompat. Mitigasi:
    /// `pg_advisory_xact_lock` dgn kunci per-bulan agar generator berurutan.
    pub async fn generate_permit_number(&self, id: Uuid) -> AppResult<String> {
        let mut client = self.pool.client().await?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Kunci per-bulan: hashtext('izin_pemakaian_bmn:nomor:YYYY-MM').
        // xact_lock dilepas otomatis di akhir transaksi (commit/rollback).
        tx.execute(
            "SELECT pg_advisory_xact_lock(hashtext('izin_pemakaian_bmn:nomor:' || TO_CHAR(NOW(), 'YYYY-MM')))",
            &[],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let query = r#"
            WITH next_number AS (
                SELECT COALESCE(MAX(
                    CAST(SUBSTRING(nomor_izin FROM 'IP/\d{4}/\d{2}/(\d{4})') AS INTEGER)
                ), 0) + 1 AS num
                FROM perlengkapan.izin_pemakaian_bmn
                WHERE nomor_izin LIKE 'IP/' || TO_CHAR(NOW(), 'YYYY/MM') || '/%'
            )
            UPDATE perlengkapan.izin_pemakaian_bmn
            SET nomor_izin = 'IP/' || TO_CHAR(NOW(), 'YYYY/MM/') || LPAD((SELECT num FROM next_number)::TEXT, 4, '0'),
                updated_at = NOW()
            WHERE id = $1
            RETURNING nomor_izin
        "#;

        let row = tx
            .query_one(query, &[&id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let nomor: String = row.get("nomor_izin");

        tx.commit()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(nomor)
    }

    /// Update document fields after document generation
    /// Requirements: REQ-P006, REQ-D002
    pub async fn update_document_fields(
        &self,
        id: Uuid,
        document_id: Uuid,
        document_url: String,
    ) -> AppResult<IzinPemakaianBmn> {
        let client = self.pool.client().await?;

        let query = r#"
            UPDATE perlengkapan.izin_pemakaian_bmn
            SET document_id = $1,
                document_url = $2,
                updated_at = NOW()
            WHERE id = $3
            RETURNING *
        "#;

        let row = client
            .query_one(query, &[&document_id, &document_url, &id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(self.row_to_permit(row))
    }

    // ========================================================================
    // Fase 1.11: Cek pegawai + Cek BMN dgn period
    // ========================================================================

    /// Lookup pegawai dari cache MySIMKARI by NIP.
    pub async fn find_pegawai_by_nip(&self, nip: &str) -> AppResult<Option<PegawaiInfo>> {
        let client = self.pool.client().await?;
        let row = client
            .query_opt(
                r#"
                SELECT nip, nama, jabatan, golpang AS pangkat, satker_id, nama_satker, foto
                FROM integrasi.mysimkari_pegawai
                WHERE nip = $1
                "#,
                &[&nip],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(row.map(|r| PegawaiInfo {
            nip: r.get("nip"),
            nama: r.try_get("nama").ok().flatten(),
            jabatan: r.try_get("jabatan").ok().flatten(),
            pangkat: r.try_get("pangkat").ok().flatten(),
            satker_id: r.try_get("satker_id").ok().flatten(),
            nama_satker: r.try_get("nama_satker").ok().flatten(),
            foto: r.try_get("foto").ok().flatten(),
        }))
    }

    /// Pemakaian BMN yg saat ini aktif utk pegawai.
    pub async fn list_pemakaian_aktif_by_pegawai(
        &self,
        nip: &str,
    ) -> AppResult<Vec<PemakaianAktifEntry>> {
        let client = self.pool.client().await?;
        let rows = client
            .query(
                r#"
                SELECT id, nomor_izin, bmn_nup, bmn_nama_barang,
                       tanggal_mulai, tanggal_selesai
                FROM perlengkapan.izin_pemakaian_bmn
                WHERE pegawai_nip = $1 AND status = 'ACTIVE'
                ORDER BY tanggal_selesai DESC
                "#,
                &[&nip],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(rows
            .iter()
            .map(|r| PemakaianAktifEntry {
                permit_id: r.get("id"),
                nomor_izin: r.try_get("nomor_izin").ok().flatten(),
                bmn_nup: r.get("bmn_nup"),
                bmn_nama_barang: r.get("bmn_nama_barang"),
                tanggal_mulai: r.get("tanggal_mulai"),
                tanggal_selesai: r.get("tanggal_selesai"),
            })
            .collect())
    }

    /// Histori pemakaian BMN pegawai (status non-aktif: Expired, Revoked,
    /// Rejected, Cancelled). Limit utk avoid blow-up.
    pub async fn list_pemakaian_histori_by_pegawai(
        &self,
        nip: &str,
        limit: i64,
    ) -> AppResult<Vec<PemakaianHistoriEntry>> {
        let client = self.pool.client().await?;
        let rows = client
            .query(
                r#"
                SELECT id, nomor_izin, bmn_nup, bmn_nama_barang, status,
                       tanggal_mulai, tanggal_selesai
                FROM perlengkapan.izin_pemakaian_bmn
                WHERE pegawai_nip = $1
                  AND status IN ('EXPIRED', 'REVOKED', 'REJECTED', 'CANCELLED')
                ORDER BY tanggal_selesai DESC
                LIMIT $2
                "#,
                &[&nip, &limit],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(rows
            .iter()
            .map(|r| PemakaianHistoriEntry {
                permit_id: r.get("id"),
                nomor_izin: r.try_get("nomor_izin").ok().flatten(),
                bmn_nup: r.get("bmn_nup"),
                bmn_nama_barang: r.get("bmn_nama_barang"),
                status: r.get("status"),
                tanggal_mulai: r.get("tanggal_mulai"),
                tanggal_selesai: r.get("tanggal_selesai"),
            })
            .collect())
    }

    /// Cek ketersediaan BMN utk periode tertentu (Fase 1.11). Mengembalikan
    /// `Available`, `PemakaianBerurutan` (existing berakhir sebelum
    /// usulan mulai → boleh), atau `Overlap` (tolak).
    pub async fn check_bmn_availability_for_period(
        &self,
        bmn_nup: &str,
        tgl_mulai: chrono::NaiveDate,
        tgl_selesai: chrono::NaiveDate,
    ) -> AppResult<BmnCheckStatus> {
        let client = self.pool.client().await?;
        // Cari izin ACTIVE utk NUP ini, urutkan tanggal_selesai DESC agar
        // izin paling baru di atas. Kita evaluasi overlap thd usulan
        // periode operator.
        let rows = client
            .query(
                r#"
                SELECT pegawai_nama, tanggal_mulai, tanggal_selesai
                FROM perlengkapan.izin_pemakaian_bmn
                WHERE bmn_nup = $1 AND status = 'ACTIVE'
                ORDER BY tanggal_selesai DESC
                "#,
                &[&bmn_nup],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Iterasi: jika ada existing yg overlap dgn (tgl_mulai..tgl_selesai)
        // → Overlap. Jika semua existing berakhir sebelum tgl_mulai dan ada
        // ≥1 existing → PemakaianBerurutan. Jika kosong → Available.
        let mut latest_existing: Option<(String, chrono::NaiveDate)> = None;
        for r in &rows {
            let ex_holder: String = r.get("pegawai_nama");
            let ex_start: chrono::NaiveDate = r.get("tanggal_mulai");
            let ex_end: chrono::NaiveDate = r.get("tanggal_selesai");

            // Overlap = NOT (ex_end < tgl_mulai OR ex_start > tgl_selesai)
            let overlap = !(ex_end < tgl_mulai || ex_start > tgl_selesai);
            if overlap {
                return Ok(BmnCheckStatus::Overlap {
                    existing_holder: ex_holder,
                    existing_sampai_tgl: ex_end,
                });
            }
            if latest_existing
                .as_ref()
                .map(|(_, prev_end)| ex_end > *prev_end)
                .unwrap_or(true)
            {
                latest_existing = Some((ex_holder, ex_end));
            }
        }

        match latest_existing {
            Some((holder, end_tgl)) => Ok(BmnCheckStatus::PemakaianBerurutan {
                existing_holder: holder,
                existing_sampai_tgl: end_tgl,
            }),
            None => Ok(BmnCheckStatus::Available),
        }
    }

    /// Check if BMN is available (no active permit) — legacy, kept for
    /// existing callers. Fase 1.11 callers harus pakai
    /// `check_bmn_availability_for_period`.
    /// Requirements: REQ-P002, REQ-P003
    pub async fn check_bmn_availability(
        &self,
        bmn_nup: &str,
    ) -> AppResult<BmnAvailabilityResponse> {
        let client = self.pool.client().await?;

        let query = r#"
            SELECT id, nomor_izin, pegawai_nama, tanggal_selesai
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE bmn_nup = $1 AND status = 'ACTIVE'
            LIMIT 1
        "#;

        let row_opt = client
            .query_opt(query, &[&bmn_nup])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(row) = row_opt {
            Ok(BmnAvailabilityResponse {
                bmn_nup: bmn_nup.to_string(),
                is_available: false,
                active_permit_id: Some(row.get("id")),
                active_permit_holder: Some(row.get("pegawai_nama")),
                active_permit_expires: Some(row.get("tanggal_selesai")),
            })
        } else {
            Ok(BmnAvailabilityResponse {
                bmn_nup: bmn_nup.to_string(),
                is_available: true,
                active_permit_id: None,
                active_permit_holder: None,
                active_permit_expires: None,
            })
        }
    }

    /// List permits with pagination and filters
    /// Requirements: REQ-P001, REQ-P011
    pub async fn list(&self, query: ListPermitsQuery) -> AppResult<PaginatedPermitsResponse> {
        let client = self.pool.client().await?;

        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).max(1).min(100);
        let offset = (page - 1) * per_page;

        let mut where_clauses = vec![];
        let mut param_count = 1;
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = vec![];

        if let Some(ref status) = query.status {
            where_clauses.push(format!("status = ${}", param_count));
            param_count += 1;
            params.push(Box::new(status.clone()));
        }

        if let Some(ref jenis_bmn) = query.jenis_bmn {
            where_clauses.push(format!("jenis_bmn = ${}", param_count));
            param_count += 1;
            params.push(Box::new(jenis_bmn.clone()));
        }

        if let Some(ref pegawai_nip) = query.pegawai_nip {
            where_clauses.push(format!("pegawai_nip = ${}", param_count));
            param_count += 1;
            params.push(Box::new(pegawai_nip.clone()));
        }

        if let Some(ref satker_id) = query.satker_id {
            where_clauses.push(format!("pegawai_satker_id = ${}", param_count));
            param_count += 1;
            params.push(Box::new(*satker_id));
        }

        if let Some(ref search) = query.search {
            // Escape LIKE-special characters to prevent pattern injection
            let escaped = search
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_");
            let search_pattern = format!("%{}%", escaped);
            where_clauses.push(format!(
                "(bmn_nama_barang ILIKE ${} OR pegawai_nama ILIKE ${} OR nomor_izin ILIKE ${})",
                param_count, param_count, param_count
            ));
            param_count += 1;
            params.push(Box::new(search_pattern));
        }
        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        // Count total
        let count_query = format!(
            "SELECT COUNT(*) as total FROM perlengkapan.izin_pemakaian_bmn {}",
            where_clause
        );

        let total_row = client
            .query_one(&count_query, &param_refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        let total: i64 = total_row.get("total");

        // Get data — parameterize LIMIT/OFFSET for defense-in-depth
        let limit_param_idx = param_count;
        let offset_param_idx = param_count + 1;
        let data_query = format!(
            "SELECT * FROM perlengkapan.izin_pemakaian_bmn {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause, limit_param_idx, offset_param_idx
        );

        let mut data_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();
        data_params.push(&per_page);
        data_params.push(&offset);

        let rows = client
            .query(&data_query, &data_params)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let data = rows
            .into_iter()
            .map(|row| self.row_to_permit(row))
            .collect();

        let total_pages = (total as f64 / per_page as f64).ceil() as i64;

        Ok(PaginatedPermitsResponse {
            data,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    /// Get permit history for a BMN
    /// Requirements: REQ-P012
    pub async fn get_bmn_usage_history(&self, bmn_nup: &str) -> AppResult<BmnUsageStats> {
        let client = self.pool.client().await?;

        let query = r#"
            SELECT
                bmn_nup,
                bmn_nama_barang,
                COUNT(*) as total_permits,
                COUNT(*) FILTER (WHERE status = 'ACTIVE') as active_permits,
                SUM(tanggal_selesai - tanggal_mulai) as total_days_used,
                (SELECT pegawai_nama FROM perlengkapan.izin_pemakaian_bmn
                 WHERE bmn_nup = $1 AND status = 'ACTIVE' LIMIT 1) as current_holder
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE bmn_nup = $1
            GROUP BY bmn_nup, bmn_nama_barang
        "#;

        let row = client
            .query_opt(query, &[&bmn_nup])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("No usage history for BMN: {}", bmn_nup)))?;

        // Get history entries
        let history_query = r#"
            SELECT id, nomor_izin, tanggal_mulai, tanggal_selesai, status, created_at
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE bmn_nup = $1
            ORDER BY created_at DESC
            LIMIT 10
        "#;

        let history_rows = client
            .query(history_query, &[&bmn_nup])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let permit_history = history_rows
            .into_iter()
            .map(|row| PermitHistoryEntry {
                id: row.get("id"),
                nomor_izin: row.get("nomor_izin"),
                tanggal_mulai: row.get("tanggal_mulai"),
                tanggal_selesai: row.get("tanggal_selesai"),
                status: row.get("status"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(BmnUsageStats {
            bmn_nup: row.get("bmn_nup"),
            bmn_nama: row.get("bmn_nama_barang"),
            total_permits: row.get("total_permits"),
            active_permits: row.get("active_permits"),
            total_days_used: row.get::<_, Option<i32>>("total_days_used").unwrap_or(0) as i64,
            current_holder: row.get("current_holder"),
            permit_history,
        })
    }

    /// Get permit history for a pegawai
    /// Requirements: REQ-P012
    pub async fn get_pegawai_usage_history(
        &self,
        pegawai_nip: &str,
    ) -> AppResult<PegawaiUsageStats> {
        let client = self.pool.client().await?;

        let query = r#"
            SELECT
                pegawai_nip,
                pegawai_nama,
                COUNT(*) as total_permits,
                COUNT(*) FILTER (WHERE status = 'ACTIVE') as active_permits
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE pegawai_nip = $1
            GROUP BY pegawai_nip, pegawai_nama
        "#;

        let row = client
            .query_opt(query, &[&pegawai_nip])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| {
                AppError::NotFound(format!("No usage history for pegawai: {}", pegawai_nip))
            })?;

        // Get history entries
        let history_query = r#"
            SELECT id, nomor_izin, tanggal_mulai, tanggal_selesai, status, created_at
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE pegawai_nip = $1
            ORDER BY created_at DESC
            LIMIT 10
        "#;

        let history_rows = client
            .query(history_query, &[&pegawai_nip])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let permit_history = history_rows
            .into_iter()
            .map(|row| PermitHistoryEntry {
                id: row.get("id"),
                nomor_izin: row.get("nomor_izin"),
                tanggal_mulai: row.get("tanggal_mulai"),
                tanggal_selesai: row.get("tanggal_selesai"),
                status: row.get("status"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(PegawaiUsageStats {
            pegawai_nip: row.get("pegawai_nip"),
            pegawai_nama: row.get("pegawai_nama"),
            total_permits: row.get("total_permits"),
            active_permits: row.get("active_permits"),
            permit_history,
        })
    }

    /// Auto-expire permits that have passed their end date
    /// Requirements: REQ-P010
    pub async fn auto_expire_permits(&self) -> AppResult<usize> {
        let client = self.pool.client().await?;

        let query = r#"
            UPDATE perlengkapan.izin_pemakaian_bmn
            SET status = 'EXPIRED',
                updated_at = NOW()
            WHERE status = 'ACTIVE'
            AND tanggal_selesai < CURRENT_DATE
        "#;

        let result = client
            .execute(query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result as usize)
    }

    /// Get permits expiring soon (for notifications)
    /// Requirements: REQ-P007
    pub async fn get_expiring_permits(
        &self,
        days_threshold: i32,
    ) -> AppResult<Vec<IzinPemakaianBmn>> {
        let client = self.pool.client().await?;

        let query = r#"
            SELECT * FROM perlengkapan.izin_pemakaian_bmn
            WHERE status = 'ACTIVE'
            AND tanggal_selesai BETWEEN CURRENT_DATE AND CURRENT_DATE + $1
            ORDER BY tanggal_selesai ASC
        "#;

        let rows = client
            .query(query, &[&days_threshold])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| self.row_to_permit(row))
            .collect())
    }

    // ========================================================================
    // Monitoring Dashboard Methods
    // ========================================================================

    /// Get active usage monitoring dashboard data
    /// Requirements: REQ-P011
    pub async fn get_active_usage_dashboard(
        &self,
        query: super::models::MonitoringDashboardQuery,
    ) -> AppResult<super::models::ActiveUsageMonitoringDashboard> {
        let client = self.pool.client().await?;

        // Build WHERE clause for filters
        let mut where_clauses = vec!["status = 'ACTIVE'".to_string()];
        let mut param_idx = 1;
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = vec![];

        if let Some(ref satker_id) = query.satker_id {
            where_clauses.push(format!("pegawai_satker_id = ${}", param_idx));
            param_idx += 1;
            params.push(Box::new(*satker_id));
        }

        if let Some(ref jenis_bmn) = query.jenis_bmn {
            where_clauses.push(format!("jenis_bmn = ${}", param_idx));
            param_idx += 1;
            params.push(Box::new(jenis_bmn.clone()));
        }
        let _ = param_idx;

        let where_clause = where_clauses.join(" AND ");
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        // Total active permits
        let total_query = format!(
            "SELECT COUNT(*) as total FROM perlengkapan.izin_pemakaian_bmn WHERE {}",
            where_clause
        );

        let total_row = client
            .query_one(&total_query, &param_refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        let total_active_permits: i64 = total_row.get("total");

        // Permits by jenis BMN
        let jenis_query = format!(
            r#"
            SELECT jenis_bmn, COUNT(*) as count
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE {}
            GROUP BY jenis_bmn
            ORDER BY count DESC
            "#,
            where_clause
        );

        let jenis_rows = client
            .query(&jenis_query, &param_refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let permits_by_jenis_bmn = jenis_rows
            .into_iter()
            .map(|row| {
                let count: i64 = row.get("count");
                let percentage = if total_active_permits > 0 {
                    (count as f64 / total_active_permits as f64) * 100.0
                } else {
                    0.0
                };
                super::models::PermitsByJenisBmn {
                    jenis_bmn: row.get("jenis_bmn"),
                    count,
                    percentage,
                }
            })
            .collect();

        // Permits by satker
        let satker_query = format!(
            r#"
            SELECT pegawai_satker_id, pegawai_satker_nama, COUNT(*) as active_permits
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE {}
            GROUP BY pegawai_satker_id, pegawai_satker_nama
            ORDER BY active_permits DESC
            LIMIT 10
            "#,
            where_clause
        );

        let satker_rows = client
            .query(&satker_query, &param_refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let permits_by_satker = satker_rows
            .into_iter()
            .map(|row| super::models::PermitsBySatker {
                satker_id: row.get("pegawai_satker_id"),
                satker_nama: row.get("pegawai_satker_nama"),
                active_permits: row.get("active_permits"),
            })
            .collect();

        // Expiring soon (next 30 days)
        let expiring_query = format!(
            r#"
            SELECT id, nomor_izin, bmn_nama_barang, pegawai_nama, tanggal_selesai,
                   (tanggal_selesai - CURRENT_DATE) as days_until_expiry
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE {} AND tanggal_selesai BETWEEN CURRENT_DATE AND CURRENT_DATE + 30
            ORDER BY tanggal_selesai ASC
            LIMIT 10
            "#,
            where_clause
        );

        let expiring_rows = client
            .query(&expiring_query, &param_refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let expiring_soon = expiring_rows
            .into_iter()
            .map(|row| super::models::ExpiringPermitInfo {
                id: row.get("id"),
                nomor_izin: row.get("nomor_izin"),
                bmn_nama: row.get("bmn_nama_barang"),
                pegawai_nama: row.get("pegawai_nama"),
                tanggal_selesai: row.get("tanggal_selesai"),
                days_until_expiry: row.get("days_until_expiry"),
            })
            .collect();

        // Recent activations (last 7 days)
        let recent_query = format!(
            r#"
            SELECT id, nomor_izin, bmn_nama_barang, pegawai_nama, approved_at
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE {} AND approved_at >= CURRENT_DATE - 7
            ORDER BY approved_at DESC
            LIMIT 10
            "#,
            where_clause
        );

        let recent_rows = client
            .query(&recent_query, &param_refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let recent_activations = recent_rows
            .into_iter()
            .map(|row| super::models::RecentActivationInfo {
                id: row.get("id"),
                nomor_izin: row.get("nomor_izin"),
                bmn_nama: row.get("bmn_nama_barang"),
                pegawai_nama: row.get("pegawai_nama"),
                activated_at: row.get("approved_at"),
            })
            .collect();

        Ok(super::models::ActiveUsageMonitoringDashboard {
            total_active_permits,
            permits_by_jenis_bmn,
            permits_by_satker,
            expiring_soon,
            recent_activations,
        })
    }

    /// Get BMN utilization report
    /// Requirements: REQ-P013
    pub async fn get_bmn_utilization_report(
        &self,
        _query: super::models::MonitoringDashboardQuery,
    ) -> AppResult<super::models::BmnUtilizationReport> {
        let client = self.pool.client().await?;

        // For this report, we need to query SIMAN data (from integrasi schema)
        // to get total BMN count and compare with permits

        // Total BMN count (from SIMAN integration)
        let total_bmn_query = r#"
            SELECT COUNT(DISTINCT nup) as total
            FROM integrasi.siman_aset_tanah
            WHERE kondisi = 'BAIK'
        "#;

        let total_bmn_row = client
            .query_one(total_bmn_query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        let total_bmn: i64 = total_bmn_row.get("total");

        // BMN with active permits
        let utilized_query = r#"
            SELECT COUNT(DISTINCT bmn_nup) as count
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE status = 'ACTIVE'
        "#;

        let utilized_row = client
            .query_one(utilized_query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        let bmn_with_active_permits: i64 = utilized_row.get("count");

        let bmn_without_permits = total_bmn - bmn_with_active_permits;
        let utilization_rate = if total_bmn > 0 {
            (bmn_with_active_permits as f64 / total_bmn as f64) * 100.0
        } else {
            0.0
        };

        // BMN utilization by type
        let by_type_query = r#"
            WITH bmn_counts AS (
                SELECT
                    CASE
                        WHEN kode_barang LIKE '03.01%' THEN 'KENDARAAN_BERMOTOR'
                        WHEN kode_barang LIKE '03.02%' THEN 'RUMAH_NEGARA'
                        WHEN kode_barang LIKE '03.03%' THEN 'LAPTOP'
                        ELSE 'LAINNYA'
                    END as jenis_bmn,
                    COUNT(DISTINCT nup) as total_bmn
                FROM integrasi.siman_aset_tanah
                WHERE kondisi = 'BAIK'
                GROUP BY jenis_bmn
            ),
            utilized_counts AS (
                SELECT jenis_bmn, COUNT(DISTINCT bmn_nup) as utilized_bmn
                FROM perlengkapan.izin_pemakaian_bmn
                WHERE status = 'ACTIVE'
                GROUP BY jenis_bmn
            )
            SELECT
                bc.jenis_bmn,
                bc.total_bmn,
                COALESCE(uc.utilized_bmn, 0) as utilized_bmn
            FROM bmn_counts bc
            LEFT JOIN utilized_counts uc ON bc.jenis_bmn = uc.jenis_bmn
        "#;

        let by_type_rows = client
            .query(by_type_query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let bmn_by_type = by_type_rows
            .into_iter()
            .map(|row| {
                let total: i64 = row.get("total_bmn");
                let utilized: i64 = row.get("utilized_bmn");
                let rate = if total > 0 {
                    (utilized as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                super::models::BmnUtilizationByType {
                    jenis_bmn: row.get("jenis_bmn"),
                    total_bmn: total,
                    utilized_bmn: utilized,
                    utilization_rate: rate,
                }
            })
            .collect();

        // Top utilized BMN
        let top_utilized_query = r#"
            SELECT
                bmn_nup,
                bmn_nama_barang,
                jenis_bmn,
                COUNT(*) as total_permits,
                SUM(tanggal_selesai - tanggal_mulai) as total_days_used,
                (SELECT pegawai_nama FROM perlengkapan.izin_pemakaian_bmn
                 WHERE bmn_nup = i.bmn_nup AND status = 'ACTIVE' LIMIT 1) as current_holder
            FROM perlengkapan.izin_pemakaian_bmn i
            GROUP BY bmn_nup, bmn_nama_barang, jenis_bmn
            ORDER BY total_permits DESC, total_days_used DESC
            LIMIT 10
        "#;

        let top_rows = client
            .query(top_utilized_query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let top_utilized_bmn = top_rows
            .into_iter()
            .map(|row| super::models::TopUtilizedBmn {
                bmn_nup: row.get("bmn_nup"),
                bmn_nama: row.get("bmn_nama_barang"),
                jenis_bmn: row.get("jenis_bmn"),
                total_permits: row.get("total_permits"),
                total_days_used: row.get::<_, Option<i32>>("total_days_used").unwrap_or(0) as i64,
                current_holder: row.get("current_holder"),
            })
            .collect();

        // Underutilized BMN (no permits in last 180 days)
        let underutilized_query = r#"
            SELECT
                s.nup as bmn_nup,
                s.nama_barang as bmn_nama,
                CASE
                    WHEN s.kode_barang LIKE '03.01%' THEN 'KENDARAAN_BERMOTOR'
                    WHEN s.kode_barang LIKE '03.02%' THEN 'RUMAH_NEGARA'
                    WHEN s.kode_barang LIKE '03.03%' THEN 'LAPTOP'
                    ELSE 'LAINNYA'
                END as jenis_bmn,
                (SELECT MAX(tanggal_selesai) FROM perlengkapan.izin_pemakaian_bmn
                 WHERE bmn_nup = s.nup) as last_used_date,
                (CURRENT_DATE - (SELECT MAX(tanggal_selesai) FROM perlengkapan.izin_pemakaian_bmn
                 WHERE bmn_nup = s.nup)) as days_since_last_use
            FROM integrasi.siman_aset_tanah s
            WHERE s.kondisi = 'BAIK'
            AND s.nup NOT IN (
                SELECT DISTINCT bmn_nup FROM perlengkapan.izin_pemakaian_bmn
                WHERE tanggal_selesai >= CURRENT_DATE - 180
            )
            LIMIT 20
        "#;

        let underutilized_rows = client
            .query(underutilized_query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let underutilized_bmn = underutilized_rows
            .into_iter()
            .map(|row| super::models::UnderutilizedBmn {
                bmn_nup: row.get("bmn_nup"),
                bmn_nama: row.get("bmn_nama"),
                jenis_bmn: row.get("jenis_bmn"),
                last_used_date: row.get("last_used_date"),
                days_since_last_use: row.get("days_since_last_use"),
            })
            .collect();

        Ok(super::models::BmnUtilizationReport {
            total_bmn,
            bmn_with_active_permits,
            bmn_without_permits,
            utilization_rate,
            bmn_by_type,
            top_utilized_bmn,
            underutilized_bmn,
        })
    }

    /// Tiga kartu agregat headline dashboard monitoring (Fase 2.6):
    /// **sedang dipakai / tidak dipakai / akan expired**.
    ///
    /// Kartu turunan-izin (`sedang_dipakai`, `akan_expired_30d`) menghormati
    /// filter `satker_id`/`jenis_bmn`. `tidak_dipakai` hanya dihitung saat
    /// TANPA filter — angka SIMAN tidak ter-scope per-satker di sini, jadi
    /// menampilkannya saat ter-filter akan menyesatkan (→ `None`). SIMAN
    /// best-effort: jika query SIMAN gagal, `tidak_dipakai = None` dan kartu
    /// lain tetap tersaji (dashboard tidak ikut tumbang).
    pub async fn get_monitoring_summary(
        &self,
        query: super::models::MonitoringDashboardQuery,
    ) -> AppResult<super::models::MonitoringSummaryCards> {
        let client = self.pool.client().await?;

        // WHERE dinamis utk kartu turunan-izin (parameterized, anti-SQLi).
        let mut where_clauses = vec!["status = 'ACTIVE'".to_string()];
        let mut param_idx = 1;
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = vec![];

        if let Some(ref satker_id) = query.satker_id {
            where_clauses.push(format!("pegawai_satker_id = ${}", param_idx));
            param_idx += 1;
            params.push(Box::new(*satker_id));
        }
        if let Some(ref jenis_bmn) = query.jenis_bmn {
            where_clauses.push(format!("jenis_bmn = ${}", param_idx));
            param_idx += 1;
            params.push(Box::new(jenis_bmn.clone()));
        }
        let _ = param_idx;

        let where_clause = where_clauses.join(" AND ");
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        // Kartu 1: sedang dipakai (izin ACTIVE).
        let sedang_dipakai: i64 = client
            .query_one(
                &format!(
                    "SELECT COUNT(*) AS c FROM perlengkapan.izin_pemakaian_bmn WHERE {}",
                    where_clause
                ),
                &param_refs,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("c");

        // Kartu 2: akan expired dalam 30 hari.
        let akan_expired_30d: i64 = client
            .query_one(
                &format!(
                    "SELECT COUNT(*) AS c FROM perlengkapan.izin_pemakaian_bmn \
                     WHERE {} AND tanggal_selesai BETWEEN CURRENT_DATE AND CURRENT_DATE + 30",
                    where_clause
                ),
                &param_refs,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("c");

        // Kartu 3: tidak dipakai — hanya saat tanpa filter (SIMAN tidak
        // ter-scope per-satker di sini). Best-effort: error SIMAN → None.
        let tidak_dipakai = if query.satker_id.is_none() && query.jenis_bmn.is_none() {
            match Self::count_idle_bmn(&client).await {
                Ok(n) => Some(n),
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "SIMAN tidak tersedia utk kartu 'tidak dipakai' — disajikan null"
                    );
                    None
                }
            }
        } else {
            None
        };

        Ok(super::models::MonitoringSummaryCards {
            sedang_dipakai,
            akan_expired_30d,
            tidak_dipakai,
        })
    }

    /// Jumlah BMN (kondisi BAIK di SIMAN) yg tidak punya izin ACTIVE =
    /// total distinct NUP − distinct NUP terpakai. Dipisah agar kegagalan
    /// SIMAN dapat ditangani best-effort oleh pemanggil.
    async fn count_idle_bmn(
        client: &deadpool_postgres::Object,
    ) -> Result<i64, tokio_postgres::Error> {
        let total: i64 = client
            .query_one(
                "SELECT COUNT(DISTINCT nup) AS c FROM integrasi.siman_aset_tanah WHERE kondisi = 'BAIK'",
                &[],
            )
            .await?
            .get("c");
        let utilized: i64 = client
            .query_one(
                "SELECT COUNT(DISTINCT bmn_nup) AS c FROM perlengkapan.izin_pemakaian_bmn WHERE status = 'ACTIVE'",
                &[],
            )
            .await?
            .get("c");
        Ok((total - utilized).max(0))
    }

    /// Create a BMN item for multi-BMN permits
    pub async fn create_bmn_item(
        &self,
        izin_pemakaian_id: Uuid,
        item: CreateBmnItemRequest,
    ) -> AppResult<PemakaianBmnItem> {
        let client = self.pool.client().await?;

        let id = Uuid::new_v4();
        let detail_json = item
            .detail_bmn
            .as_ref()
            .map(|v| serde_json::to_value(v).unwrap_or_default());

        let query = r#"
            INSERT INTO perlengkapan.pemakaian_bmn_items (
                id, izin_pemakaian_id, bmn_nup, bmn_kode_barang,
                bmn_nama_barang, detail_bmn, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, NOW())
            RETURNING *
        "#;

        let row = client
            .query_one(
                query,
                &[
                    &id,
                    &izin_pemakaian_id,
                    &item.bmn_nup,
                    &item.bmn_kode_barang,
                    &item.bmn_nama_barang,
                    &detail_json,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(PemakaianBmnItem::from_row(&row))
    }

    /// Get BMN items for a permit
    pub async fn get_bmn_items(&self, izin_pemakaian_id: Uuid) -> AppResult<Vec<PemakaianBmnItem>> {
        let client = self.pool.client().await?;

        let rows = client
            .query(
                "SELECT * FROM perlengkapan.pemakaian_bmn_items WHERE izin_pemakaian_id = $1 ORDER BY created_at",
                &[&izin_pemakaian_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows.iter().map(PemakaianBmnItem::from_row).collect())
    }

    /// Update both DOCX and PDF konsep surat URLs and the matching on-disk
    /// paths in a single statement. This is the only konsep-surat write
    /// path — DOCX + PDF are always produced together.
    pub async fn update_konsep_surat(
        &self,
        id: Uuid,
        docx_url: &str,
        docx_path: &str,
        pdf_url: &str,
        pdf_path: &str,
    ) -> AppResult<()> {
        let client = self.pool.client().await?;

        client
            .execute(
                r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET konsep_surat_url = $1,
                    konsep_surat_docx_path = $2,
                    konsep_surat_generated_at = NOW(),
                    konsep_surat_pdf_url = $3,
                    konsep_surat_pdf_path = $4,
                    konsep_surat_pdf_generated_at = NOW(),
                    updated_at = NOW()
                WHERE id = $5
                "#,
                &[&docx_url, &docx_path, &pdf_url, &pdf_path, &id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Fetch the on-disk path the route handler should stream from.
    pub async fn konsep_surat_path(&self, id: Uuid, format: &str) -> AppResult<Option<String>> {
        let column = match format {
            "docx" => "konsep_surat_docx_path",
            "pdf" => "konsep_surat_pdf_path",
            _ => return Ok(None),
        };
        let client = self.pool.client().await?;
        let query = format!(
            "SELECT {} FROM perlengkapan.izin_pemakaian_bmn WHERE id = $1",
            column
        );
        let row = client
            .query_opt(&query, &[&id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(row.and_then(|r| r.try_get::<_, Option<String>>(0).ok().flatten()))
    }

    /// Update signed PDF URL and mark as completed
    pub async fn update_signed_pdf(&self, id: Uuid, signed_pdf_url: &str) -> AppResult<()> {
        let client = self.pool.client().await?;

        client
            .execute(
                r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET signed_pdf_url = $1,
                    signed_pdf_uploaded_at = NOW(),
                    is_completed = true,
                    updated_at = NOW()
                WHERE id = $2
                "#,
                &[&signed_pdf_url, &id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Helper function to convert database row to IzinPemakaianBmn
    fn row_to_permit(&self, row: tokio_postgres::Row) -> IzinPemakaianBmn {
        IzinPemakaianBmn {
            id: row.get("id"),
            nomor_izin: row.get("nomor_izin"),
            pegawai_nip: row.get("pegawai_nip"),
            pegawai_nama: row.get("pegawai_nama"),
            pegawai_satker_id: row.get("pegawai_satker_id"),
            pegawai_satker_nama: row.get("pegawai_satker_nama"),
            pegawai_jabatan: row.get("pegawai_jabatan"),
            pegawai_golongan: row.try_get("pegawai_golongan").ok().flatten(),
            pegawai_pangkat: row.try_get("pegawai_pangkat").ok().flatten(),
            pegawai_unit_kerja: row.try_get("pegawai_unit_kerja").ok().flatten(),
            foto_pegawai: row.try_get("foto_pegawai").ok().flatten(),
            jenis_bmn: row.get("jenis_bmn"),
            bmn_nup: row.get("bmn_nup"),
            bmn_kode_barang: row.get("bmn_kode_barang"),
            bmn_nama_barang: row.get("bmn_nama_barang"),
            bmn_merk: row.get("bmn_merk"),
            bmn_tahun_perolehan: row.get("bmn_tahun_perolehan"),
            no_polisi: row.get("no_polisi"),
            no_bpkb: row.get("no_bpkb"),
            no_stnk: row.get("no_stnk"),
            no_rangka: row.get("no_rangka"),
            no_mesin: row.get("no_mesin"),
            alamat: row.get("alamat"),
            luas_tanah: row.get("luas_tanah"),
            luas_bangunan: row.get("luas_bangunan"),
            serial_number: row.get("serial_number"),
            spesifikasi: row.get("spesifikasi"),
            tanggal_mulai: row.get("tanggal_mulai"),
            tanggal_selesai: row.get("tanggal_selesai"),
            keperluan: row.get("keperluan"),
            lokasi_pemakaian: row.get("lokasi_pemakaian"),
            is_renewal: row.get("is_renewal"),
            previous_permit_id: row.get("previous_permit_id"),
            file_pendukung: row.get("file_pendukung"),
            document_id: row.get("document_id"),
            document_url: row.get("document_url"),
            konsep_surat_url: row.try_get("konsep_surat_url").ok().flatten(),
            konsep_surat_generated_at: row.try_get("konsep_surat_generated_at").ok().flatten(),
            konsep_surat_pdf_url: row.try_get("konsep_surat_pdf_url").ok().flatten(),
            konsep_surat_pdf_generated_at: row
                .try_get("konsep_surat_pdf_generated_at")
                .ok()
                .flatten(),
            signed_pdf_url: row.try_get("signed_pdf_url").ok().flatten(),
            signed_pdf_uploaded_at: row.try_get("signed_pdf_uploaded_at").ok().flatten(),
            is_completed: row.try_get("is_completed").unwrap_or(false),
            status: row.get("status"),
            catatan_approval: row.get("catatan_approval"),
            catatan_revocation: row.get("catatan_revocation"),
            approved_by: row.get("approved_by"),
            approved_by_nama: row.get("approved_by_nama"),
            approved_at: row.get("approved_at"),
            // V035 (Fase 1.5): kolom baru — try_get + default agar tetap
            // kompatibel dgn schema legacy (sebelum V035 di-apply di env dev).
            validator_satker_id: row.try_get("validator_satker_id").ok().flatten(),
            validator_satker_nama: row.try_get("validator_satker_nama").ok().flatten(),
            tanggal_validasi_satker: row.try_get("tanggal_validasi_satker").ok().flatten(),
            catatan_validator_satker: row.try_get("catatan_validator_satker").ok().flatten(),
            approver_satker_id: row.try_get("approver_satker_id").ok().flatten(),
            approver_satker_nama: row.try_get("approver_satker_nama").ok().flatten(),
            tanggal_approval_satker: row.try_get("tanggal_approval_satker").ok().flatten(),
            catatan_approver_satker: row.try_get("catatan_approver_satker").ok().flatten(),
            version: row.try_get("version").unwrap_or(1),
            revoked_by: row.get("revoked_by"),
            revoked_by_nama: row.get("revoked_by_nama"),
            revoked_at: row.get("revoked_at"),
            created_by: row.get("created_by"),
            created_by_nama: row.get("created_by_nama"),
            updated_by: row.get("updated_by"),
            updated_by_nama: row.get("updated_by_nama"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            bmn_items: vec![],
        }
    }
}
