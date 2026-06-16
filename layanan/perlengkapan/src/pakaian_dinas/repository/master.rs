use super::PakaianDinasRepository;
use crate::pakaian_dinas::models::*;
use crate::shared::error::{AppError, AppResult, bad_request};
use uuid::Uuid;

impl PakaianDinasRepository {
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
            .query_one(
                "SELECT COUNT(*) as total FROM perlengkapan.ms_jenis_pakaian_dinas",
                &[],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let total: i64 = count_row.get("total");

        let rows = client
            .query(
                r#"
                SELECT id, nama, deskripsi, is_active, created_at, updated_at
                FROM perlengkapan.ms_jenis_pakaian_dinas
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
                "SELECT id, nama, deskripsi, is_active, created_at, updated_at FROM perlengkapan.ms_jenis_pakaian_dinas WHERE id = $1",
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
                INSERT INTO perlengkapan.ms_jenis_pakaian_dinas (id, nama, deskripsi, is_active, created_at, updated_at)
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
                UPDATE perlengkapan.ms_jenis_pakaian_dinas
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
            .execute(
                "DELETE FROM perlengkapan.ms_jenis_pakaian_dinas WHERE id = $1",
                &[&id],
            )
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
                    "SELECT COUNT(*) as total FROM perlengkapan.ms_spesifikasi_pakaian_dinas WHERE jenis_pakaian_dinas_id = $1".to_string(),
                    r#"
                    SELECT s.id, s.jenis_pakaian_dinas_id, s.nama, s.gender, s.ukuran_group,
                           s.deskripsi, s.is_active, s.created_at, s.updated_at,
                           j.nama as jenis_pakaian_nama
                    FROM perlengkapan.ms_spesifikasi_pakaian_dinas s
                    LEFT JOIN perlengkapan.ms_jenis_pakaian_dinas j ON s.jenis_pakaian_dinas_id = j.id
                    WHERE s.jenis_pakaian_dinas_id = $1
                    ORDER BY s.nama ASC
                    LIMIT $2 OFFSET $3
                    "#.to_string(),
                    vec![jid, &per_page_i64, &offset_i64],
                )
        } else {
            (
                "SELECT COUNT(*) as total FROM perlengkapan.ms_spesifikasi_pakaian_dinas".to_string(),
                r#"
                    SELECT s.id, s.jenis_pakaian_dinas_id, s.nama, s.gender, s.ukuran_group,
                           s.deskripsi, s.is_active, s.created_at, s.updated_at,
                           j.nama as jenis_pakaian_nama
                    FROM perlengkapan.ms_spesifikasi_pakaian_dinas s
                    LEFT JOIN perlengkapan.ms_jenis_pakaian_dinas j ON s.jenis_pakaian_dinas_id = j.id
                    ORDER BY j.nama ASC, s.nama ASC
                    LIMIT $1 OFFSET $2
                    "#
                .to_string(),
                vec![&per_page_i64, &offset_i64],
            )
        };

        let count_row = if let Some(jid) = jenis_id.as_ref() {
            client.query_one(&count_query, &[jid]).await
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
                FROM perlengkapan.ms_spesifikasi_pakaian_dinas s
                LEFT JOIN perlengkapan.ms_jenis_pakaian_dinas j ON s.jenis_pakaian_dinas_id = j.id
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
                INSERT INTO perlengkapan.ms_spesifikasi_pakaian_dinas
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
                UPDATE perlengkapan.ms_spesifikasi_pakaian_dinas
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
                "DELETE FROM perlengkapan.ms_spesifikasi_pakaian_dinas WHERE id = $1",
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
                    "SELECT COUNT(*) as total FROM perlengkapan.ms_subspesifikasi_pakaian_dinas WHERE spesifikasi_id = $1",
                    &[&sid],
                )
                .await
                .map_err(|e| bad_request(&e.to_string()))?;
            ("filtered".to_string(), row.get("total"))
        } else {
            let row = client
                .query_one(
                    "SELECT COUNT(*) as total FROM perlengkapan.ms_subspesifikasi_pakaian_dinas",
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
                    FROM perlengkapan.ms_subspesifikasi_pakaian_dinas ss
                    LEFT JOIN perlengkapan.ms_spesifikasi_pakaian_dinas s ON ss.spesifikasi_id = s.id
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
                    FROM perlengkapan.ms_subspesifikasi_pakaian_dinas ss
                    LEFT JOIN perlengkapan.ms_spesifikasi_pakaian_dinas s ON ss.spesifikasi_id = s.id
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
                FROM perlengkapan.ms_subspesifikasi_pakaian_dinas ss
                LEFT JOIN perlengkapan.ms_spesifikasi_pakaian_dinas s ON ss.spesifikasi_id = s.id
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
                INSERT INTO perlengkapan.ms_subspesifikasi_pakaian_dinas
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
                "DELETE FROM perlengkapan.ms_subspesifikasi_pakaian_dinas WHERE id = $1",
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
                    r#"SELECT ukuran, "group", urutan FROM perlengkapan.ms_ukuran WHERE "group" = $1 ORDER BY urutan ASC"#,
                    &[&g],
                )
                .await
        } else {
            client
                .query(
                    r#"SELECT ukuran, "group", urutan FROM perlengkapan.ms_ukuran ORDER BY "group", urutan ASC"#,
                    &[],
                )
                .await
        }
        .map_err(|e| bad_request(&e.to_string()))?;

        Ok(rows.iter().map(Ukuran::from_row).collect())
    }

    // ============ Pengajuan ============
}
