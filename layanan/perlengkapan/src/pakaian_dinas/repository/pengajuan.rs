use super::PakaianDinasRepository;
use crate::pakaian_dinas::models::*;
use crate::shared::error::{AppError, AppResult, bad_request};
use chrono::Datelike;
use uuid::Uuid;

impl PakaianDinasRepository {
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

        let rows = if let Some(tahun) = tahun {
            client
                .query(
                    &data_sql,
                    &[&tahun, &(per_page as i64), &(offset as i64)],
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
                     created_by, created_at, updated_at, scope_satker, wilayah_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $13, $14, $15)
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
                    // V031: scope_satker mirror pilihan_satker; wilayah_id (#19)
                    &request.pilihan_satker,
                    &request.wilayah_id,
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

        // Resolve & insert selected satkers.
        // - "wilayah" (#19): auto-resolve dari integrasi.mysimkari_satker.wilayah.
        // - "sebagian": pakai satker_ids dari operator.
        // - "all"/"semua": kosong (artinya seluruh satker).
        let resolved_satker_ids: Vec<Uuid> = if request.pilihan_satker == "wilayah" {
            match request
                .wilayah_id
                .as_deref()
                .filter(|w| !w.trim().is_empty())
            {
                Some(wid) => self.list_satker_ids_by_wilayah(wid).await?,
                None => Vec::new(),
            }
        } else {
            request.satker_ids.clone().unwrap_or_default()
        };

        for satker_id in &resolved_satker_ids {
            client
                .execute(
                    r#"
                    INSERT INTO pengajuan_pakaian_dinas_satker_terpilih
                        (pengajuan_id, satker_id, is_show_in_form)
                    VALUES ($1, $2, true)
                    ON CONFLICT (pengajuan_id, satker_id) DO NOTHING
                    "#,
                    &[&id, satker_id],
                )
                .await
                .map_err(|e| bad_request(&e.to_string()))?;
        }

        self.get_pengajuan_by_id(id).await
    }

    /// Resolve satker UUID untuk satu wilayah Kejaksaan Tinggi (#19), sumber
    /// `integrasi.mysimkari_satker`. Sejalan dgn resolver Kebutuhan BMN namun
    /// mengembalikan `id` (UUID) karena `satker_terpilih.satker_id` bertipe UUID.
    pub async fn list_satker_ids_by_wilayah(&self, wilayah: &str) -> AppResult<Vec<Uuid>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let rows = client
            .query(
                "SELECT id FROM integrasi.mysimkari_satker WHERE wilayah = $1",
                &[&wilayah],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        Ok(rows.iter().map(|r| r.get::<_, Uuid>("id")).collect())
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
}
