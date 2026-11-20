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
            .map_err(|e| {
                // A jenis that a campaign still points at is held by a foreign key
                // (the spesifikasi under it cascade, the campaigns do not). Its
                // message is the database's own wording; say the reason instead.
                let in_use = e.as_db_error().is_some_and(|d| {
                    d.code() == &tokio_postgres::error::SqlState::FOREIGN_KEY_VIOLATION
                });
                if in_use {
                    bad_request(
                        "Jenis pakaian dinas tidak dapat dihapus: masih dipakai oleh pengajuan pakaian dinas",
                    )
                } else {
                    bad_request(&e.to_string())
                }
            })?;

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

impl PakaianDinasRepository {
    /// Sudah ada spesifikasi bernama sama di bawah jenis ini?
    ///
    /// Dicocokkan tanpa memedulikan kapitalisasi dan spasi tepi, sama seperti
    /// penjaga di migrasi seed — kalau tidak, "Celana" dan " celana " lolos
    /// sebagai dua baris berbeda.
    pub async fn spesifikasi_nama_terpakai(
        &self,
        jenis_id: uuid::Uuid,
        nama: &str,
    ) -> AppResult<bool> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let row = client
            .query_one(
                "SELECT EXISTS(
                    SELECT 1 FROM perlengkapan.ms_spesifikasi_pakaian_dinas
                    WHERE jenis_pakaian_dinas_id = $1 AND upper(btrim(nama)) = upper(btrim($2))
                 ) AS ada",
                &[&jenis_id, &nama],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        Ok(row.get("ada"))
    }

    /// Sudah ada subspesifikasi bernama sama di bawah spesifikasi ini?
    pub async fn subspesifikasi_nama_terpakai(
        &self,
        spesifikasi_id: uuid::Uuid,
        nama: &str,
    ) -> AppResult<bool> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let row = client
            .query_one(
                "SELECT EXISTS(
                    SELECT 1 FROM perlengkapan.ms_subspesifikasi_pakaian_dinas
                    WHERE spesifikasi_id = $1 AND upper(btrim(nama)) = upper(btrim($2))
                 ) AS ada",
                &[&spesifikasi_id, &nama],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        Ok(row.get("ada"))
    }
}

// ============ Master: Grup Ukuran dan daftar ukuran (V019) ============
//
// Grup ukuran adalah data referensi: menambah jenis isian baru (topi, nama dada, ...)
// berarti menambah baris di sini, bukan migrasi. Semua penjaga di bawah menolak yang
// membuat isian tersimpan menjadi tak sesuai lagi dengan aturan grupnya.
impl PakaianDinasRepository {
    pub async fn get_all_ukuran_grup(&self, hanya_aktif: bool) -> AppResult<Vec<UkuranGrup>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let rows = client
            .query(
                "SELECT kode, nama, tipe, urutan, is_active
                 FROM perlengkapan.ms_ukuran_group
                 WHERE ($1::boolean = false OR is_active)
                 ORDER BY urutan, kode",
                &[&hanya_aktif],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        Ok(rows.iter().map(UkuranGrup::from_row).collect())
    }

    pub async fn get_ukuran_grup(&self, kode: &str) -> AppResult<Option<UkuranGrup>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let row = client
            .query_opt(
                "SELECT kode, nama, tipe, urutan, is_active
                 FROM perlengkapan.ms_ukuran_group WHERE kode = $1",
                &[&kode],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        Ok(row.as_ref().map(UkuranGrup::from_row))
    }

    pub async fn create_ukuran_grup(
        &self,
        kode: &str,
        nama: &str,
        tipe: &str,
        urutan: Option<i32>,
    ) -> AppResult<UkuranGrup> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        // `ON CONFLICT DO NOTHING` + pemeriksaan baris: dua permintaan serentak dengan kode
        // yang sama menghasilkan satu yang menang dan satu pesan yang jelas, bukan galat
        // constraint mentah.
        let row = client
            .query_opt(
                "INSERT INTO perlengkapan.ms_ukuran_group (kode, nama, tipe, urutan)
                 VALUES ($1, $2, $3,
                         COALESCE($4, (SELECT COALESCE(MAX(urutan), 0) + 1 FROM perlengkapan.ms_ukuran_group)))
                 ON CONFLICT (kode) DO NOTHING
                 RETURNING kode, nama, tipe, urutan, is_active",
                &[&kode, &nama, &tipe, &urutan],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        row.as_ref()
            .map(UkuranGrup::from_row)
            .ok_or_else(|| bad_request(&format!("Grup ukuran \"{kode}\" sudah ada")))
    }

    pub async fn update_ukuran_grup(
        &self,
        kode: &str,
        nama: &str,
        urutan: Option<i32>,
        is_active: bool,
    ) -> AppResult<UkuranGrup> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let row = client
            .query_opt(
                "UPDATE perlengkapan.ms_ukuran_group
                 SET nama = $2, urutan = COALESCE($3, urutan), is_active = $4, updated_at = now()
                 WHERE kode = $1
                 RETURNING kode, nama, tipe, urutan, is_active",
                &[&kode, &nama, &urutan, &is_active],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        row.as_ref()
            .map(UkuranGrup::from_row)
            .ok_or_else(|| AppError::NotFound("Grup ukuran tidak ditemukan".to_string()))
    }

    /// Hapus grup yang tak dipakai siapa pun. Yang masih dirujuk spesifikasi atau punya
    /// daftar ukuran ditolak dengan alasan yang menyebut jumlahnya, bukan dengan galat FK.
    pub async fn delete_ukuran_grup(&self, kode: &str) -> AppResult<()> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let pakai = client
            .query_one(
                "SELECT (SELECT count(*) FROM perlengkapan.ms_spesifikasi_pakaian_dinas WHERE ukuran_group = $1) AS spesifikasi,
                        (SELECT count(*) FROM perlengkapan.ms_ukuran WHERE \"group\" = $1) AS ukuran",
                &[&kode],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let spesifikasi: i64 = pakai.get("spesifikasi");
        let ukuran: i64 = pakai.get("ukuran");
        if spesifikasi > 0 {
            return Err(bad_request(&format!(
                "Grup \"{kode}\" masih dipakai {spesifikasi} spesifikasi; ganti grup spesifikasinya dulu atau nonaktifkan grup ini"
            )));
        }
        if ukuran > 0 {
            return Err(bad_request(&format!(
                "Grup \"{kode}\" masih punya {ukuran} ukuran; hapus ukurannya dulu atau nonaktifkan grup ini"
            )));
        }
        let n = client
            .execute(
                "DELETE FROM perlengkapan.ms_ukuran_group WHERE kode = $1",
                &[&kode],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        if n == 0 {
            return Err(AppError::NotFound(
                "Grup ukuran tidak ditemukan".to_string(),
            ));
        }
        Ok(())
    }

    /// Ukuran yang sah untuk sebuah grup, urut. Untuk grup `TEKS` kosong.
    pub async fn daftar_ukuran_grup(&self, kode: &str) -> AppResult<Vec<String>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let rows = client
            .query(
                "SELECT ukuran FROM perlengkapan.ms_ukuran WHERE \"group\" = $1 ORDER BY urutan, ukuran",
                &[&kode],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        Ok(rows.iter().map(|r| r.get::<_, String>("ukuran")).collect())
    }

    pub async fn create_ukuran(
        &self,
        group: &str,
        ukuran: &str,
        urutan: Option<i32>,
    ) -> AppResult<Ukuran> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let row = client
            .query_opt(
                // Casts eksplisit: `$2` dipakai dua kali, dan tanpa tipe Postgres menyimpulkannya
                // berbeda di VALUES dan di subkueri ("inconsistent types deduced for parameter $2").
                "INSERT INTO perlengkapan.ms_ukuran (ukuran, \"group\", urutan)
                 VALUES ($1::varchar, $2::varchar,
                         COALESCE($3::int4, (SELECT COALESCE(MAX(urutan), 0) + 1 FROM perlengkapan.ms_ukuran WHERE \"group\" = $2::varchar)))
                 ON CONFLICT (ukuran, \"group\") DO NOTHING
                 RETURNING ukuran, \"group\", urutan",
                &[&ukuran, &group, &urutan],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        row.as_ref()
            .map(Ukuran::from_row)
            .ok_or_else(|| bad_request(&format!("Ukuran \"{ukuran}\" sudah ada di grup {group}")))
    }

    pub async fn update_ukuran_urutan(
        &self,
        group: &str,
        ukuran: &str,
        urutan: i32,
    ) -> AppResult<Ukuran> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let row = client
            .query_opt(
                "UPDATE perlengkapan.ms_ukuran SET urutan = $3
                 WHERE \"group\" = $1 AND ukuran = $2
                 RETURNING ukuran, \"group\", urutan",
                &[&group, &ukuran, &urutan],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        row.as_ref()
            .map(Ukuran::from_row)
            .ok_or_else(|| AppError::NotFound("Ukuran tidak ditemukan".to_string()))
    }

    /// Susun ulang seluruh daftar ukuran sebuah grup: semua perubahan `urutan` dalam SATU transaksi
    /// di bawah kunci eksklusif grup itu, atau tak satu pun. `daftar` harus persis himpunan ukuran
    /// yang ada sekarang; selisihnya (ditambah, dihapus, atau ganda sejak layar memuatnya) ditolak
    /// dan tak ada yang berubah. Urutan ditulis sebagai kelipatan 10 supaya ada ruang sisipan.
    pub async fn urutkan_ukuran(&self, group: &str, daftar: &[String]) -> AppResult<Vec<Ukuran>> {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let tx = client
            .transaction()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        tx.execute(
            "SELECT pg_advisory_xact_lock(hashtext($1::text)::bigint)",
            &[&format!("ukuran_grup:{group}")],
        )
        .await
        .map_err(|e| bad_request(&e.to_string()))?;

        let ada: Vec<String> = tx
            .query(
                "SELECT ukuran FROM perlengkapan.ms_ukuran WHERE \"group\" = $1",
                &[&group],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?
            .iter()
            .map(|r| r.get::<_, String>("ukuran"))
            .collect();
        let mut minta: Vec<&String> = daftar.iter().collect();
        minta.sort();
        let mut sekarang: Vec<&String> = ada.iter().collect();
        sekarang.sort();
        let ganda = minta.windows(2).any(|w| w[0] == w[1]);
        if ganda || minta != sekarang {
            return Err(bad_request(&format!(
                "Daftar ukuran grup {group} berbeda dari yang ada di server (ada {} ukuran, dikirim {}). \
                 Muat ulang halaman lalu ulangi",
                ada.len(),
                daftar.len()
            )));
        }

        let urutan: Vec<i32> = (1..=daftar.len() as i32).map(|i| i * 10).collect();
        tx.execute(
            "UPDATE perlengkapan.ms_ukuran m SET urutan = v.urutan
             FROM unnest($2::text[], $3::int4[]) AS v(ukuran, urutan)
             WHERE m.\"group\" = $1 AND m.ukuran = v.ukuran",
            &[&group, &daftar, &urutan],
        )
        .await
        .map_err(|e| bad_request(&e.to_string()))?;
        let rows = tx
            .query(
                "SELECT ukuran, \"group\", urutan FROM perlengkapan.ms_ukuran
                 WHERE \"group\" = $1 ORDER BY urutan, ukuran",
                &[&group],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        tx.commit().await.map_err(|e| bad_request(&e.to_string()))?;
        Ok(rows.iter().map(Ukuran::from_row).collect())
    }

    /// Hapus satu ukuran dari daftar sebuah grup, kecuali masih dipakai isian yang
    /// tersimpan: ukuran yang hilang dari daftar membuat isian itu tak bisa lagi dipilih
    /// dan tak lolos validasi bila disimpan ulang.
    ///
    /// Penghitungan pemakaian dan penghapusan terjadi di SATU transaksi di bawah kunci advisory
    /// EKSKLUSIF grup itu. Penulis ukuran mengambil kunci bersama yang sama sebelum membaca
    /// daftar, jadi tak ada penyimpanan yang lolos validasi terhadap ukuran yang sedang
    /// dihapus; membungkus SELECT dan DELETE dalam transaksi saja tak cukup tanpa itu.
    pub async fn delete_ukuran(&self, group: &str, ukuran: &str) -> AppResult<()> {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let tx = client
            .transaction()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        tx.execute(
            "SELECT pg_advisory_xact_lock(hashtext($1::text)::bigint)",
            &[&format!("ukuran_grup:{group}")],
        )
        .await
        .map_err(|e| bad_request(&e.to_string()))?;
        let pakai = tx
            .query_one(
                "SELECT
                   (SELECT count(*) FROM perlengkapan.pegawai_pakaian_dinas_ukuran u
                      JOIN perlengkapan.ms_spesifikasi_pakaian_dinas s ON s.id = u.spesifikasi_id
                     WHERE s.ukuran_group = $1 AND u.ukuran = $2) AS profil,
                   (SELECT count(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran su
                      JOIN perlengkapan.pengajuan_pakaian_dinas_pakaian pk ON pk.id = su.pakaian_id
                     WHERE pk.spesifikasi_ukuran_group = $1 AND su.ukuran = $2) AS kampanye",
                &[&group, &ukuran],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let profil: i64 = pakai.get("profil");
        let kampanye: i64 = pakai.get("kampanye");
        if profil + kampanye > 0 {
            return Err(bad_request(&format!(
                "Ukuran \"{ukuran}\" di grup {group} masih dipakai {profil} profil pegawai dan {kampanye} isian kampanye"
            )));
        }
        let n = tx
            .execute(
                "DELETE FROM perlengkapan.ms_ukuran WHERE \"group\" = $1 AND ukuran = $2",
                &[&group, &ukuran],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        if n == 0 {
            return Err(AppError::NotFound("Ukuran tidak ditemukan".to_string()));
        }
        tx.commit().await.map_err(|e| bad_request(&e.to_string()))?;
        Ok(())
    }

    /// Berapa isian profil pegawai yang memakai spesifikasi ini. Dipakai untuk menolak
    /// pergantian grup spesifikasi yang membuat isian itu tak sah.
    pub async fn jumlah_isian_profil_spesifikasi(&self, spesifikasi_id: Uuid) -> AppResult<i64> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let row = client
            .query_one(
                "SELECT count(*) AS n FROM perlengkapan.pegawai_pakaian_dinas_ukuran WHERE spesifikasi_id = $1",
                &[&spesifikasi_id],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        Ok(row.get("n"))
    }
}
