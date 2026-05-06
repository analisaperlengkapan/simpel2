//! # Pakaian Dinas Repository
//!
//! Database operations for the Pakaian Dinas module.
//! Uses tokio-postgres for async database access.

use deadpool_postgres::Pool;
use uuid::Uuid;

use super::models::*;
use crate::errors::{AppError, AppResult, bad_request};

/// Repository for Pakaian Dinas database operations
#[derive(Clone)]
pub struct PakaianDinasRepository {
    pool: Pool,
}

impl PakaianDinasRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    // ============ Master: Jenis Pakaian Dinas ============

    pub async fn get_all_jenis(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<JenisPakaianDinas>, i64)> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let offset = (page - 1) * per_page;

        let count_row = client
            .query_one("SELECT COUNT(*) as total FROM ms_jenis_pakaian_dinas", &[])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let total: i64 = count_row.get("total");

        let rows = client
            .query(
                r#"
                SELECT id, nama, deskripsi, is_active, created_at, updated_at
                FROM ms_jenis_pakaian_dinas
                ORDER BY nama ASC
                LIMIT $1 OFFSET $2
                "#,
                &[&(per_page as i64), &(offset as i64)],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let items: Vec<JenisPakaianDinas> = rows.iter().map(JenisPakaianDinas::from_row).collect();
        Ok((items, total))
    }

    pub async fn get_jenis_by_id(&self, id: Uuid) -> AppResult<JenisPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let row = client
            .query_opt(
                "SELECT id, nama, deskripsi, is_active, created_at, updated_at FROM ms_jenis_pakaian_dinas WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Jenis pakaian dinas tidak ditemukan".to_string()))?;

        Ok(JenisPakaianDinas::from_row(&row))
    }

    pub async fn create_jenis(
        &self,
        request: CreateJenisPakaianDinasRequest,
    ) -> AppResult<JenisPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let row = client
            .query_one(
                r#"
                INSERT INTO ms_jenis_pakaian_dinas (id, nama, deskripsi, is_active, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $5)
                RETURNING id, nama, deskripsi, is_active, created_at, updated_at
                "#,
                &[&id, &request.nama, &request.deskripsi, &request.is_active, &now],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        Ok(JenisPakaianDinas::from_row(&row))
    }

    pub async fn update_jenis(
        &self,
        id: Uuid,
        request: CreateJenisPakaianDinasRequest,
    ) -> AppResult<JenisPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let now = chrono::Utc::now();

        let row = client
            .query_opt(
                r#"
                UPDATE ms_jenis_pakaian_dinas
                SET nama = $2, deskripsi = $3, is_active = $4, updated_at = $5
                WHERE id = $1
                RETURNING id, nama, deskripsi, is_active, created_at, updated_at
                "#,
                &[
                    &id,
                    &request.nama,
                    &request.deskripsi,
                    &request.is_active,
                    &now,
                ],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Jenis pakaian dinas tidak ditemukan".to_string()))?;

        Ok(JenisPakaianDinas::from_row(&row))
    }

    pub async fn delete_jenis(&self, id: Uuid) -> AppResult<()> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let result = client
            .execute("DELETE FROM ms_jenis_pakaian_dinas WHERE id = $1", &[&id])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        if result == 0 {
            return Err(AppError::NotFound(
                "Jenis pakaian dinas tidak ditemukan".to_string(),
            ));
        }

        Ok(())
    }

    // ============ Master: Spesifikasi ============

    pub async fn get_all_spesifikasi(
        &self,
        page: i32,
        per_page: i32,
        jenis_id: Option<Uuid>,
    ) -> AppResult<(Vec<SpesifikasiPakaianDinas>, i64)> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let offset = (page - 1) * per_page;

        // Pre-compute these to avoid temporary value lifetime issues
        let per_page_i64 = per_page as i64;
        let offset_i64 = offset as i64;

        let (count_query, data_query, params): (
            String,
            String,
            Vec<&(dyn tokio_postgres::types::ToSql + Sync)>,
        ) = if let Some(jid) = jenis_id.as_ref() {
            (
                    "SELECT COUNT(*) as total FROM ms_spesifikasi_pakaian_dinas WHERE jenis_pakaian_dinas_id = $1".to_string(),
                    r#"
                    SELECT s.id, s.jenis_pakaian_dinas_id, s.nama, s.gender, s.ukuran_group,
                           s.deskripsi, s.is_active, s.created_at, s.updated_at,
                           j.nama as jenis_pakaian_nama
                    FROM ms_spesifikasi_pakaian_dinas s
                    LEFT JOIN ms_jenis_pakaian_dinas j ON s.jenis_pakaian_dinas_id = j.id
                    WHERE s.jenis_pakaian_dinas_id = $1
                    ORDER BY s.nama ASC
                    LIMIT $2 OFFSET $3
                    "#.to_string(),
                    vec![jid, &per_page_i64, &offset_i64],
                )
        } else {
            (
                "SELECT COUNT(*) as total FROM ms_spesifikasi_pakaian_dinas".to_string(),
                r#"
                    SELECT s.id, s.jenis_pakaian_dinas_id, s.nama, s.gender, s.ukuran_group,
                           s.deskripsi, s.is_active, s.created_at, s.updated_at,
                           j.nama as jenis_pakaian_nama
                    FROM ms_spesifikasi_pakaian_dinas s
                    LEFT JOIN ms_jenis_pakaian_dinas j ON s.jenis_pakaian_dinas_id = j.id
                    ORDER BY j.nama ASC, s.nama ASC
                    LIMIT $1 OFFSET $2
                    "#
                .to_string(),
                vec![&per_page_i64, &offset_i64],
            )
        };

        let count_row = if jenis_id.is_some() {
            client
                .query_one(&count_query, &[jenis_id.as_ref().unwrap()])
                .await
        } else {
            client.query_one(&count_query, &[]).await
        }
        .map_err(|e| bad_request(&e.to_string()))?;
        let total: i64 = count_row.get("total");

        let rows = client
            .query(&data_query, &params[..])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let items: Vec<SpesifikasiPakaianDinas> =
            rows.iter().map(SpesifikasiPakaianDinas::from_row).collect();
        Ok((items, total))
    }

    pub async fn get_spesifikasi_by_id(&self, id: Uuid) -> AppResult<SpesifikasiPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let row = client
            .query_opt(
                r#"
                SELECT s.id, s.jenis_pakaian_dinas_id, s.nama, s.gender, s.ukuran_group,
                       s.deskripsi, s.is_active, s.created_at, s.updated_at,
                       j.nama as jenis_pakaian_nama
                FROM ms_spesifikasi_pakaian_dinas s
                LEFT JOIN ms_jenis_pakaian_dinas j ON s.jenis_pakaian_dinas_id = j.id
                WHERE s.id = $1
                "#,
                &[&id],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Spesifikasi tidak ditemukan".to_string()))?;

        Ok(SpesifikasiPakaianDinas::from_row(&row))
    }

    pub async fn create_spesifikasi(
        &self,
        request: CreateSpesifikasiRequest,
    ) -> AppResult<SpesifikasiPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let _row = client
            .query_one(
                r#"
                INSERT INTO ms_spesifikasi_pakaian_dinas
                    (id, jenis_pakaian_dinas_id, nama, gender, ukuran_group, deskripsi, is_active, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
                RETURNING id, jenis_pakaian_dinas_id, nama, gender, ukuran_group, deskripsi, is_active, created_at, updated_at
                "#,
                &[
                    &id,
                    &request.jenis_pakaian_dinas_id,
                    &request.nama,
                    &request.gender,
                    &request.ukuran_group,
                    &request.deskripsi,
                    &request.is_active,
                    &now,
                ],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        // Fetch with join to get jenis_pakaian_nama
        self.get_spesifikasi_by_id(id).await
    }

    pub async fn update_spesifikasi(
        &self,
        id: Uuid,
        request: CreateSpesifikasiRequest,
    ) -> AppResult<SpesifikasiPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let now = chrono::Utc::now();

        let result = client
            .execute(
                r#"
                UPDATE ms_spesifikasi_pakaian_dinas
                SET jenis_pakaian_dinas_id = $2, nama = $3, gender = $4, ukuran_group = $5,
                    deskripsi = $6, is_active = $7, updated_at = $8
                WHERE id = $1
                "#,
                &[
                    &id,
                    &request.jenis_pakaian_dinas_id,
                    &request.nama,
                    &request.gender,
                    &request.ukuran_group,
                    &request.deskripsi,
                    &request.is_active,
                    &now,
                ],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        if result == 0 {
            return Err(AppError::NotFound(
                "Spesifikasi tidak ditemukan".to_string(),
            ));
        }

        self.get_spesifikasi_by_id(id).await
    }

    pub async fn delete_spesifikasi(&self, id: Uuid) -> AppResult<()> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let result = client
            .execute(
                "DELETE FROM ms_spesifikasi_pakaian_dinas WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        if result == 0 {
            return Err(AppError::NotFound(
                "Spesifikasi tidak ditemukan".to_string(),
            ));
        }

        Ok(())
    }

    // ============ Master: SubSpesifikasi ============

    pub async fn get_all_subspesifikasi(
        &self,
        page: i32,
        per_page: i32,
        spesifikasi_id: Option<Uuid>,
    ) -> AppResult<(Vec<SubSpesifikasiPakaianDinas>, i64)> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let offset = (page - 1) * per_page;

        let (_count_query, total): (String, i64) = if let Some(sid) = spesifikasi_id {
            let row = client
                .query_one(
                    "SELECT COUNT(*) as total FROM ms_subspesifikasi_pakaian_dinas WHERE spesifikasi_id = $1",
                    &[&sid],
                )
                .await
                .map_err(|e| bad_request(&e.to_string()))?;
            ("filtered".to_string(), row.get("total"))
        } else {
            let row = client
                .query_one(
                    "SELECT COUNT(*) as total FROM ms_subspesifikasi_pakaian_dinas",
                    &[],
                )
                .await
                .map_err(|e| bad_request(&e.to_string()))?;
            ("all".to_string(), row.get("total"))
        };

        let rows = if let Some(sid) = spesifikasi_id {
            client
                .query(
                    r#"
                    SELECT ss.id, ss.spesifikasi_id, ss.nama, ss.gender, ss.is_active,
                           ss.created_at, ss.updated_at, s.nama as spesifikasi_nama
                    FROM ms_subspesifikasi_pakaian_dinas ss
                    LEFT JOIN ms_spesifikasi_pakaian_dinas s ON ss.spesifikasi_id = s.id
                    WHERE ss.spesifikasi_id = $1
                    ORDER BY ss.nama ASC
                    LIMIT $2 OFFSET $3
                    "#,
                    &[&sid, &(per_page as i64), &(offset as i64)],
                )
                .await
        } else {
            client
                .query(
                    r#"
                    SELECT ss.id, ss.spesifikasi_id, ss.nama, ss.gender, ss.is_active,
                           ss.created_at, ss.updated_at, s.nama as spesifikasi_nama
                    FROM ms_subspesifikasi_pakaian_dinas ss
                    LEFT JOIN ms_spesifikasi_pakaian_dinas s ON ss.spesifikasi_id = s.id
                    ORDER BY s.nama ASC, ss.nama ASC
                    LIMIT $1 OFFSET $2
                    "#,
                    &[&(per_page as i64), &(offset as i64)],
                )
                .await
        }
        .map_err(|e| bad_request(&e.to_string()))?;

        let items: Vec<SubSpesifikasiPakaianDinas> = rows
            .iter()
            .map(SubSpesifikasiPakaianDinas::from_row)
            .collect();
        Ok((items, total))
    }

    pub async fn get_subspesifikasi_by_id(
        &self,
        id: Uuid,
    ) -> AppResult<SubSpesifikasiPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let row = client
            .query_opt(
                r#"
                SELECT ss.id, ss.spesifikasi_id, ss.nama, ss.gender, ss.is_active,
                       ss.created_at, ss.updated_at, s.nama as spesifikasi_nama
                FROM ms_subspesifikasi_pakaian_dinas ss
                LEFT JOIN ms_spesifikasi_pakaian_dinas s ON ss.spesifikasi_id = s.id
                WHERE ss.id = $1
                "#,
                &[&id],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?
            .ok_or_else(|| AppError::NotFound("SubSpesifikasi tidak ditemukan".to_string()))?;

        Ok(SubSpesifikasiPakaianDinas::from_row(&row))
    }

    pub async fn create_subspesifikasi(
        &self,
        request: CreateSubSpesifikasiRequest,
    ) -> AppResult<SubSpesifikasiPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        client
            .execute(
                r#"
                INSERT INTO ms_subspesifikasi_pakaian_dinas
                    (id, spesifikasi_id, nama, gender, is_active, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $6)
                "#,
                &[
                    &id,
                    &request.spesifikasi_id,
                    &request.nama,
                    &request.gender,
                    &request.is_active,
                    &now,
                ],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        self.get_subspesifikasi_by_id(id).await
    }

    pub async fn delete_subspesifikasi(&self, id: Uuid) -> AppResult<()> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let result = client
            .execute(
                "DELETE FROM ms_subspesifikasi_pakaian_dinas WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        if result == 0 {
            return Err(AppError::NotFound(
                "SubSpesifikasi tidak ditemukan".to_string(),
            ));
        }

        Ok(())
    }

    // ============ Master: Ukuran ============

    pub async fn get_all_ukuran(&self, group: Option<String>) -> AppResult<Vec<Ukuran>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let rows = if let Some(g) = group {
            client
                .query(
                    r#"SELECT ukuran, "group", urutan FROM ms_ukuran WHERE "group" = $1 ORDER BY urutan ASC"#,
                    &[&g],
                )
                .await
        } else {
            client
                .query(
                    r#"SELECT ukuran, "group", urutan FROM ms_ukuran ORDER BY "group", urutan ASC"#,
                    &[],
                )
                .await
        }
        .map_err(|e| bad_request(&e.to_string()))?;

        Ok(rows.iter().map(Ukuran::from_row).collect())
    }

    // ============ Pengajuan ============

    pub async fn get_all_pengajuan(
        &self,
        page: i32,
        per_page: i32,
        tahun: Option<i32>,
    ) -> AppResult<(Vec<PengajuanPakaianDinas>, i64)> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let offset = (page - 1) * per_page;

        let (_count_sql, data_sql, total): (String, String, i64) = if let Some(t) = tahun {
            let row = client
                .query_one(
                    "SELECT COUNT(*) as total FROM pengajuan_pakaian_dinas WHERE tahun = $1",
                    &[&t],
                )
                .await
                .map_err(|e| bad_request(&e.to_string()))?;
            (
                "filtered".to_string(),
                r#"
                    SELECT p.*, j.nama as jenis_pakaian_nama,
                           (SELECT COUNT(*) FROM pengajuan_pakaian_dinas_satker_terpilih WHERE pengajuan_id = p.id) as total_satker,
                           (SELECT COUNT(*) FROM pengajuan_pakaian_dinas_satker ps
                            WHERE ps.pengajuan_id = p.id AND ps.aktivitas_id = 1008) as satker_selesai
                    FROM pengajuan_pakaian_dinas p
                    LEFT JOIN ms_jenis_pakaian_dinas j ON p.jenis_pakaian_dinas_id = j.id
                    WHERE p.tahun = $1
                    ORDER BY p.created_at DESC
                    LIMIT $2 OFFSET $3
                    "#.to_string(),
                row.get("total"),
            )
        } else {
            let row = client
                .query_one("SELECT COUNT(*) as total FROM pengajuan_pakaian_dinas", &[])
                .await
                .map_err(|e| bad_request(&e.to_string()))?;
            (
                "all".to_string(),
                r#"
                SELECT p.*, j.nama as jenis_pakaian_nama,
                       (SELECT COUNT(*) FROM pengajuan_pakaian_dinas_satker_terpilih WHERE pengajuan_id = p.id) as total_satker,
                       (SELECT COUNT(*) FROM pengajuan_pakaian_dinas_satker ps
                        WHERE ps.pengajuan_id = p.id AND ps.aktivitas_id = 1008) as satker_selesai
                FROM pengajuan_pakaian_dinas p
                LEFT JOIN ms_jenis_pakaian_dinas j ON p.jenis_pakaian_dinas_id = j.id
                ORDER BY p.created_at DESC
                LIMIT $1 OFFSET $2
                "#.to_string(),
                row.get("total"),
            )
        };

        let rows = if tahun.is_some() {
            client
                .query(
                    &data_sql,
                    &[&tahun.unwrap(), &(per_page as i64), &(offset as i64)],
                )
                .await
        } else {
            client
                .query(&data_sql, &[&(per_page as i64), &(offset as i64)])
                .await
        }
        .map_err(|e| bad_request(&e.to_string()))?;

        let items: Vec<PengajuanPakaianDinas> =
            rows.iter().map(PengajuanPakaianDinas::from_row).collect();
        Ok((items, total))
    }

    pub async fn get_pengajuan_by_id(&self, id: Uuid) -> AppResult<PengajuanPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let row = client
            .query_opt(
                r#"
                SELECT p.*, j.nama as jenis_pakaian_nama,
                       (SELECT COUNT(*) FROM pengajuan_pakaian_dinas_satker_terpilih WHERE pengajuan_id = p.id) as total_satker,
                       (SELECT COUNT(*) FROM pengajuan_pakaian_dinas_satker ps
                        WHERE ps.pengajuan_id = p.id AND ps.aktivitas_id = 1008) as satker_selesai
                FROM pengajuan_pakaian_dinas p
                LEFT JOIN ms_jenis_pakaian_dinas j ON p.jenis_pakaian_dinas_id = j.id
                WHERE p.id = $1
                "#,
                &[&id],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Pengajuan tidak ditemukan".to_string()))?;

        Ok(PengajuanPakaianDinas::from_row(&row))
    }

    pub async fn create_pengajuan(
        &self,
        request: CreatePengajuanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        let tahun = request.tahun.unwrap_or_else(|| chrono::Local::now().year());

        // Insert main pengajuan
        client
            .execute(
                r#"
                INSERT INTO pengajuan_pakaian_dinas
                    (id, nama, deskripsi, tgl_mulai, tgl_selesai, is_reguler, tahun,
                     pilihan_satker, dengan_unit_kerja, jenis_pakaian_dinas_id, aktivitas_id,
                     created_by, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $13)
                "#,
                &[
                    &id,
                    &request.nama,
                    &request.deskripsi,
                    &request.tgl_mulai,
                    &request.tgl_selesai,
                    &request.is_reguler,
                    &tahun,
                    &request.pilihan_satker,
                    &request.dengan_unit_kerja,
                    &request.jenis_pakaian_dinas_id,
                    &1000i32, // Initial status: Input
                    &user_id,
                    &now,
                ],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        // Insert selected pakaian (specifications)
        for spec_id in &request.spesifikasi_ids {
            let pakaian_id = Uuid::new_v4();
            client
                .execute(
                    r#"
                    INSERT INTO pengajuan_pakaian_dinas_pakaian
                        (id, pengajuan_id, jenis_pakaian_id, jenis_pakaian_nama,
                         spesifikasi_id, spesifikasi_nama, spesifikasi_ukuran_group)
                    SELECT $1, $2, s.jenis_pakaian_dinas_id, j.nama, s.id, s.nama, s.ukuran_group
                    FROM ms_spesifikasi_pakaian_dinas s
                    LEFT JOIN ms_jenis_pakaian_dinas j ON s.jenis_pakaian_dinas_id = j.id
                    WHERE s.id = $3
                    "#,
                    &[&pakaian_id, &id, spec_id],
                )
                .await
                .map_err(|e| bad_request(&e.to_string()))?;
        }

        // Insert selected satkers (if pilihan_satker = "sebagian")
        if let Some(satker_ids) = &request.satker_ids {
            for satker_id in satker_ids {
                client
                    .execute(
                        r#"
                        INSERT INTO pengajuan_pakaian_dinas_satker_terpilih
                            (pengajuan_id, satker_id, is_show_in_form)
                        VALUES ($1, $2, true)
                        "#,
                        &[&id, satker_id],
                    )
                    .await
                    .map_err(|e| bad_request(&e.to_string()))?;
            }
        }

        self.get_pengajuan_by_id(id).await
    }

    pub async fn delete_pengajuan(&self, id: Uuid) -> AppResult<()> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        // Check if pengajuan exists and is still in initial status
        let pengajuan = self.get_pengajuan_by_id(id).await?;
        if pengajuan.aktivitas_id != 1000 {
            return Err(bad_request(
                "Hanya pengajuan dengan status 'Input' yang dapat dihapus",
            ));
        }

        // Delete in order due to FK constraints
        client.execute("DELETE FROM pengajuan_pakaian_dinas_satker_pegawai_ukuran WHERE pengajuan_satker_id IN (SELECT id FROM pengajuan_pakaian_dinas_satker WHERE pengajuan_id = $1)", &[&id]).await.ok();
        client.execute("DELETE FROM pengajuan_pakaian_dinas_satker_pegawai WHERE pengajuan_satker_id IN (SELECT id FROM pengajuan_pakaian_dinas_satker WHERE pengajuan_id = $1)", &[&id]).await.ok();
        client.execute("DELETE FROM pengajuan_pakaian_dinas_satker_aktivitas WHERE pengajuan_satker_id IN (SELECT id FROM pengajuan_pakaian_dinas_satker WHERE pengajuan_id = $1)", &[&id]).await.ok();
        client
            .execute(
                "DELETE FROM pengajuan_pakaian_dinas_satker WHERE pengajuan_id = $1",
                &[&id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM pengajuan_pakaian_dinas_pakaian WHERE pengajuan_id = $1",
                &[&id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM pengajuan_pakaian_dinas_satker_terpilih WHERE pengajuan_id = $1",
                &[&id],
            )
            .await
            .ok();

        let result = client
            .execute("DELETE FROM pengajuan_pakaian_dinas WHERE id = $1", &[&id])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        if result == 0 {
            return Err(AppError::NotFound("Pengajuan tidak ditemukan".to_string()));
        }

        Ok(())
    }

    // ============ Pengajuan Satker ============

    pub async fn get_pengajuan_satker_list(
        &self,
        pengajuan_id: Uuid,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<PengajuanSatker>, i64)> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let offset = (page - 1) * per_page;

        let count_row = client
            .query_one(
                "SELECT COUNT(*) as total FROM pengajuan_pakaian_dinas_satker WHERE pengajuan_id = $1",
                &[&pengajuan_id],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let total: i64 = count_row.get("total");

        let rows = client
            .query(
                r#"
                SELECT ps.*, s.nama as satker_nama, s.kode as satker_kode,
                       (SELECT COUNT(*) FROM pengajuan_pakaian_dinas_satker_pegawai WHERE pengajuan_satker_id = ps.id) as total_pegawai
                FROM pengajuan_pakaian_dinas_satker ps
                LEFT JOIN integrasi.mysimkari_satker s ON ps.satker_id = s.id
                WHERE ps.pengajuan_id = $1
                ORDER BY s.nama ASC
                LIMIT $2 OFFSET $3
                "#,
                &[&pengajuan_id, &(per_page as i64), &(offset as i64)],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let items: Vec<PengajuanSatker> = rows.iter().map(PengajuanSatker::from_row).collect();
        Ok((items, total))
    }

    pub async fn get_pengajuan_satker_by_id(&self, id: Uuid) -> AppResult<PengajuanSatker> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let row = client
            .query_opt(
                r#"
                SELECT ps.*, s.nama as satker_nama, s.kode as satker_kode,
                       (SELECT COUNT(*) FROM pengajuan_pakaian_dinas_satker_pegawai WHERE pengajuan_satker_id = ps.id) as total_pegawai
                FROM pengajuan_pakaian_dinas_satker ps
                LEFT JOIN integrasi.mysimkari_satker s ON ps.satker_id = s.id
                WHERE ps.id = $1
                "#,
                &[&id],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Pengajuan satker tidak ditemukan".to_string()))?;

        Ok(PengajuanSatker::from_row(&row))
    }

    // ============ Employee Sizes ============

    pub async fn get_pegawai_pakaian_dinas(
        &self,
        nip: &str,
    ) -> AppResult<Option<PegawaiPakaianDinas>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let row = client
            .query_opt(
                "SELECT * FROM pegawai_pakaian_dinas WHERE nip = $1",
                &[&nip],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        Ok(row.map(|r| PegawaiPakaianDinas::from_row(&r)))
    }

    pub async fn upsert_pegawai_pakaian_dinas(
        &self,
        request: &UpdatePersonalUkuranRequest,
        nip: &str,
        nama: Option<&str>,
    ) -> AppResult<PegawaiPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let now = chrono::Utc::now();

        client
            .execute(
                r#"
                INSERT INTO pegawai_pakaian_dinas
                    (nip, nama, ukuran_baju, ukuran_celana, ukuran_sepatu, with_hijab, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (nip) DO UPDATE SET
                    ukuran_baju = EXCLUDED.ukuran_baju,
                    ukuran_celana = EXCLUDED.ukuran_celana,
                    ukuran_sepatu = EXCLUDED.ukuran_sepatu,
                    with_hijab = EXCLUDED.with_hijab,
                    updated_at = EXCLUDED.updated_at
                "#,
                &[
                    &nip,
                    &nama,
                    &request.ukuran_baju,
                    &request.ukuran_celana,
                    &request.ukuran_sepatu,
                    &request.with_hijab,
                    &now,
                ],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        self.get_pegawai_pakaian_dinas(nip)
            .await?
            .ok_or_else(|| bad_request("Gagal menyimpan data ukuran"))
    }

    /// Full profile upsert — writes sizes + reporting fields in one shot.
    /// Used by the wizard and admin bulk-import.
    pub async fn upsert_pegawai_profile(
        &self,
        request: &super::models::UpsertPegawaiProfileRequest,
    ) -> AppResult<PegawaiPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let now = chrono::Utc::now();

        client
            .execute(
                r#"
                INSERT INTO pegawai_pakaian_dinas
                    (nip, nama, pangkat, jabatan, eselon, jenis_kelamin,
                     jenis_pegawai, with_hijab, mapped_unit_kerja, kode_satker,
                     ukuran_baju, ukuran_celana, ukuran_sepatu, updated_at)
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
                ON CONFLICT (nip) DO UPDATE SET
                    nama = COALESCE(EXCLUDED.nama, pegawai_pakaian_dinas.nama),
                    pangkat = COALESCE(EXCLUDED.pangkat, pegawai_pakaian_dinas.pangkat),
                    jabatan = COALESCE(EXCLUDED.jabatan, pegawai_pakaian_dinas.jabatan),
                    eselon = COALESCE(EXCLUDED.eselon, pegawai_pakaian_dinas.eselon),
                    jenis_kelamin = COALESCE(EXCLUDED.jenis_kelamin, pegawai_pakaian_dinas.jenis_kelamin),
                    jenis_pegawai = COALESCE(EXCLUDED.jenis_pegawai, pegawai_pakaian_dinas.jenis_pegawai),
                    with_hijab = EXCLUDED.with_hijab,
                    mapped_unit_kerja = COALESCE(EXCLUDED.mapped_unit_kerja, pegawai_pakaian_dinas.mapped_unit_kerja),
                    kode_satker = COALESCE(EXCLUDED.kode_satker, pegawai_pakaian_dinas.kode_satker),
                    ukuran_baju = COALESCE(EXCLUDED.ukuran_baju, pegawai_pakaian_dinas.ukuran_baju),
                    ukuran_celana = COALESCE(EXCLUDED.ukuran_celana, pegawai_pakaian_dinas.ukuran_celana),
                    ukuran_sepatu = COALESCE(EXCLUDED.ukuran_sepatu, pegawai_pakaian_dinas.ukuran_sepatu),
                    updated_at = EXCLUDED.updated_at
                "#,
                &[
                    &request.nip,
                    &request.nama,
                    &request.pangkat,
                    &request.jabatan,
                    &request.eselon,
                    &request.jenis_kelamin,
                    &request.jenis_pegawai,
                    &request.with_hijab,
                    &request.mapped_unit_kerja,
                    &request.kode_satker,
                    &request.ukuran_baju,
                    &request.ukuran_celana,
                    &request.ukuran_sepatu,
                    &now,
                ],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        self.get_pegawai_pakaian_dinas(&request.nip)
            .await?
            .ok_or_else(|| bad_request("Gagal menyimpan profil pegawai"))
    }

    /// Bulk upsert profiles — used by the pakaian dinas wizard when
    /// submitting a full satker roster at once.
    pub async fn bulk_upsert_pegawai_profiles(
        &self,
        requests: &[super::models::UpsertPegawaiProfileRequest],
    ) -> AppResult<usize> {
        if requests.is_empty() {
            return Ok(0);
        }
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let now = chrono::Utc::now();

        let mut count = 0usize;
        for req in requests {
            client
                .execute(
                    r#"
                    INSERT INTO pegawai_pakaian_dinas
                        (nip, nama, pangkat, jabatan, eselon, jenis_kelamin,
                         jenis_pegawai, with_hijab, mapped_unit_kerja, kode_satker,
                         ukuran_baju, ukuran_celana, ukuran_sepatu, updated_at)
                    VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
                    ON CONFLICT (nip) DO UPDATE SET
                        nama = COALESCE(EXCLUDED.nama, pegawai_pakaian_dinas.nama),
                        pangkat = COALESCE(EXCLUDED.pangkat, pegawai_pakaian_dinas.pangkat),
                        jabatan = COALESCE(EXCLUDED.jabatan, pegawai_pakaian_dinas.jabatan),
                        eselon = COALESCE(EXCLUDED.eselon, pegawai_pakaian_dinas.eselon),
                        jenis_kelamin = COALESCE(EXCLUDED.jenis_kelamin, pegawai_pakaian_dinas.jenis_kelamin),
                        jenis_pegawai = COALESCE(EXCLUDED.jenis_pegawai, pegawai_pakaian_dinas.jenis_pegawai),
                        with_hijab = EXCLUDED.with_hijab,
                        mapped_unit_kerja = COALESCE(EXCLUDED.mapped_unit_kerja, pegawai_pakaian_dinas.mapped_unit_kerja),
                        kode_satker = COALESCE(EXCLUDED.kode_satker, pegawai_pakaian_dinas.kode_satker),
                        ukuran_baju = COALESCE(EXCLUDED.ukuran_baju, pegawai_pakaian_dinas.ukuran_baju),
                        ukuran_celana = COALESCE(EXCLUDED.ukuran_celana, pegawai_pakaian_dinas.ukuran_celana),
                        ukuran_sepatu = COALESCE(EXCLUDED.ukuran_sepatu, pegawai_pakaian_dinas.ukuran_sepatu),
                        updated_at = EXCLUDED.updated_at
                    "#,
                    &[
                        &req.nip,
                        &req.nama,
                        &req.pangkat,
                        &req.jabatan,
                        &req.eselon,
                        &req.jenis_kelamin,
                        &req.jenis_pegawai,
                        &req.with_hijab,
                        &req.mapped_unit_kerja,
                        &req.kode_satker,
                        &req.ukuran_baju,
                        &req.ukuran_celana,
                        &req.ukuran_sepatu,
                        &now,
                    ],
                )
                .await
                .map_err(|e| bad_request(&e.to_string()))?;
            count += 1;
        }

        Ok(count)
    }

    // ============ MySIMKARI Integration ============

    pub async fn get_mysimkari_pegawai_by_satker(
        &self,
        satker_id: Uuid,
    ) -> AppResult<Vec<MysimkariPegawai>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let rows = client
            .query(
                r#"
                SELECT * FROM integrasi.mysimkari_pegawai
                WHERE satker_id = $1
                ORDER BY nama ASC
                "#,
                &[&satker_id],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        Ok(rows.iter().map(MysimkariPegawai::from_row).collect())
    }

    // ============ Reports ============

    pub async fn get_laporan_rekap_ukuran(
        &self,
        pengajuan_id: Uuid,
        filter: &LaporanFilter,
    ) -> AppResult<Vec<LaporanRekapUkuran>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let mut query = r#"
            SELECT
                pp.spesifikasi_nama as pakaian_nama,
                pp.spesifikasi_ukuran_group as ukuran_group,
                pu.ukuran,
                SUM(CASE WHEN psp.jenis_kelamin = 'L' THEN 1 ELSE 0 END) as jumlah_laki,
                SUM(CASE WHEN psp.jenis_kelamin = 'P' THEN 1 ELSE 0 END) as jumlah_perempuan,
                COUNT(*) as jumlah_total
            FROM pengajuan_pakaian_dinas_satker_pegawai_ukuran pu
            JOIN pengajuan_pakaian_dinas_satker_pegawai psp ON pu.pegawai_id = psp.id
            JOIN pengajuan_pakaian_dinas_satker ps ON psp.pengajuan_satker_id = ps.id
            JOIN pengajuan_pakaian_dinas_pakaian pp ON pu.pakaian_id = pp.id
            WHERE ps.pengajuan_id = $1
              AND ps.aktivitas_id = 1008
        "#
        .to_string();

        if let Some(ref jk) = filter.jenis_kelamin {
            query.push_str(&format!(" AND psp.jenis_kelamin = '{}'", jk));
        }
        if let Some(ref eselon) = filter.eselon {
            query.push_str(&format!(" AND psp.eselon = '{}'", eselon));
        }
        if let Some(ref jenis) = filter.jenis {
            query.push_str(&format!(" AND psp.jenis = '{}'", jenis));
        }

        query.push_str(" GROUP BY pp.spesifikasi_nama, pp.spesifikasi_ukuran_group, pu.ukuran ORDER BY pp.spesifikasi_nama, pu.ukuran");

        let rows = client
            .query(&query, &[&pengajuan_id])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        Ok(rows.iter().map(LaporanRekapUkuran::from_row).collect())
    }

    pub async fn get_laporan_daftar_pegawai(
        &self,
        pengajuan_id: Uuid,
        filter: &LaporanFilter,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<LaporanDaftarPegawai>, i64)> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let offset = (page - 1) * per_page;

        // Build dynamic filter
        let mut where_clause = "WHERE ps.pengajuan_id = $1 AND ps.aktivitas_id = 1008".to_string();
        if let Some(ref jk) = filter.jenis_kelamin {
            where_clause.push_str(&format!(" AND psp.jenis_kelamin = '{}'", jk));
        }
        if let Some(ref satker_id) = filter.satker_id {
            where_clause.push_str(&format!(" AND ps.satker_id = '{}'", satker_id));
        }
        if let Some(ref eselon) = filter.eselon {
            where_clause.push_str(&format!(" AND psp.eselon = '{}'", eselon));
        }
        if let Some(ref jenis) = filter.jenis {
            where_clause.push_str(&format!(" AND psp.jenis = '{}'", jenis));
        }

        let count_query = format!(
            r#"
            SELECT COUNT(*) as total
            FROM pengajuan_pakaian_dinas_satker_pegawai psp
            JOIN pengajuan_pakaian_dinas_satker ps ON psp.pengajuan_satker_id = ps.id
            {}
            "#,
            where_clause
        );

        let count_row = client
            .query_one(&count_query, &[&pengajuan_id])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let total: i64 = count_row.get("total");

        let data_query = format!(
            r#"
            SELECT
                psp.nip, psp.nama, s.nama as satker_nama, psp.jabatan, psp.pangkat,
                psp.jenis_kelamin, psp.gol_kd, psp.jenis, psp.eselon, psp.with_hijab,
                MAX(CASE WHEN pp.spesifikasi_ukuran_group = 'BAJU' THEN pu.ukuran END) as ukuran_baju,
                MAX(CASE WHEN pp.spesifikasi_ukuran_group = 'CELANA' THEN pu.ukuran END) as ukuran_celana,
                MAX(CASE WHEN pp.spesifikasi_ukuran_group = 'SEPATU' THEN pu.ukuran END) as ukuran_sepatu
            FROM pengajuan_pakaian_dinas_satker_pegawai psp
            JOIN pengajuan_pakaian_dinas_satker ps ON psp.pengajuan_satker_id = ps.id
            LEFT JOIN integrasi.mysimkari_satker s ON ps.satker_id = s.id
            LEFT JOIN pengajuan_pakaian_dinas_satker_pegawai_ukuran pu ON psp.id = pu.pegawai_id
            LEFT JOIN pengajuan_pakaian_dinas_pakaian pp ON pu.pakaian_id = pp.id
            {}
            GROUP BY psp.id, psp.nip, psp.nama, s.nama, psp.jabatan, psp.pangkat,
                     psp.jenis_kelamin, psp.gol_kd, psp.jenis, psp.eselon, psp.with_hijab
            ORDER BY s.nama, psp.nama
            LIMIT $2 OFFSET $3
            "#,
            where_clause
        );

        let rows = client
            .query(
                &data_query,
                &[&pengajuan_id, &(per_page as i64), &(offset as i64)],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let items: Vec<LaporanDaftarPegawai> =
            rows.iter().map(LaporanDaftarPegawai::from_row).collect();
        Ok((items, total))
    }

    // ============ Workflow Methods ============

    /// Update pengajuan status and log activity
    pub async fn update_pengajuan_status(
        &self,
        pengajuan_id: Uuid,
        new_aktivitas_id: i32,
        user_id: Uuid,
        catatan: Option<String>,
    ) -> AppResult<()> {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        // Update pengajuan status
        let update_query = r#"
            UPDATE perlengkapan.pengajuan_pakaian_dinas
            SET aktivitas_id = $1, updated_at = NOW()
            WHERE id = $2
        "#;

        tx.execute(update_query, &[&new_aktivitas_id, &pengajuan_id])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        // Log activity
        let activity_id = Uuid::new_v4();
        let insert_activity_query = r#"
            INSERT INTO perlengkapan.pengajuan_pakaian_dinas_aktivitas
            (id, pengajuan_id, aktivitas_id, user_id, catatan, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
        "#;

        tx.execute(
            insert_activity_query,
            &[
                &activity_id,
                &pengajuan_id,
                &new_aktivitas_id,
                &user_id,
                &catatan,
            ],
        )
        .await
        .map_err(|e| bad_request(&e.to_string()))?;

        tx.commit().await.map_err(|e| bad_request(&e.to_string()))?;

        Ok(())
    }

    /// Update pengajuan document metadata
    pub async fn update_pengajuan_document(
        &self,
        pengajuan_id: Uuid,
        document_id: Uuid,
        document_url: String,
    ) -> AppResult<()> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let update_query = r#"
            UPDATE perlengkapan.pengajuan_pakaian_dinas
            SET document_id = $1, document_url = $2, updated_at = NOW()
            WHERE id = $3
        "#;

        client
            .execute(update_query, &[&document_id, &document_url, &pengajuan_id])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        Ok(())
    }
}

// ============ Unit Tests ============

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    // Note: These tests require a database connection
    // Run with: cargo test --features test-db

    #[test]
    fn test_repository_new() {
        // This is a placeholder for integration tests
        // Actual tests would require a database connection
    }
}

use chrono::Datelike;
