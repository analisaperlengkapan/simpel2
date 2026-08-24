use super::PakaianDinasRepository;
use crate::pakaian_dinas::models::*;
use crate::pakaian_dinas::scope::campaign_visibility_condition;
use crate::shared::error::{AppError, AppResult, bad_request};
use crate::shared::satker_scope::SatkerScope;
use chrono::Datelike;
use tokio_postgres::types::ToSql;
use uuid::Uuid;

type BoxedParam = Box<dyn ToSql + Sync + Send>;

impl PakaianDinasRepository {
    /// List pakaian-dinas campaigns, tiered-RBAC scoped to what the caller may
    /// see (#72). The same visibility predicate is applied to BOTH the COUNT and
    /// the data query so pagination totals match the rows returned.
    pub async fn get_all_pengajuan(
        &self,
        page: i32,
        per_page: i32,
        tahun: Option<i32>,
        scope: &SatkerScope,
    ) -> AppResult<(Vec<PengajuanPakaianDinas>, i64)> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let offset = (page - 1) * per_page;

        // Shared predicate: optional year filter + campaign-visibility scope.
        // Built with positional binds so COUNT and data stay in lockstep.
        let mut params: Vec<BoxedParam> = Vec::new();
        let mut conds: Vec<String> = Vec::new();
        if let Some(t) = tahun {
            params.push(Box::new(t));
            conds.push(format!("p.tahun = ${}", params.len()));
        }
        if let Some(cond) = campaign_visibility_condition(scope, &mut params) {
            conds.push(cond);
        }
        let where_clause = if conds.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conds.join(" AND "))
        };

        // COUNT with the same predicate. Scoped in a block so the immutable
        // borrow of `params` ends before we extend it with LIMIT/OFFSET below.
        let count_sql = format!(
            "SELECT COUNT(*) as total FROM perlengkapan.pengajuan_pakaian_dinas p {where_clause}"
        );
        let total: i64 = {
            let count_refs: Vec<&(dyn ToSql + Sync)> = params
                .iter()
                .map(|b| b.as_ref() as &(dyn ToSql + Sync))
                .collect();
            client
                .query_one(&count_sql, &count_refs)
                .await
                .map_err(|e| bad_request(&e.to_string()))?
                .get("total")
        };

        // Data query: same predicate + LIMIT/OFFSET appended.
        params.push(Box::new(per_page as i64));
        let limit_idx = params.len();
        params.push(Box::new(offset as i64));
        let offset_idx = params.len();
        let data_sql = format!(
            r#"
            SELECT p.*, j.nama as jenis_pakaian_nama,
                   COALESCE(a.deskripsi, a.nama) as aktivitas_label,
                   (SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker_terpilih WHERE pengajuan_id = p.id) as total_satker,
                   (SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker ps
                    WHERE ps.pengajuan_id = p.id AND ps.aktivitas_id = 1008) as satker_selesai
            FROM perlengkapan.pengajuan_pakaian_dinas p
            LEFT JOIN perlengkapan.ms_jenis_pakaian_dinas j ON p.jenis_pakaian_dinas_id = j.id
            LEFT JOIN perlengkapan.ms_aktivitas_bmn a ON p.aktivitas_id = a.kode
            {where_clause}
            ORDER BY p.created_at DESC
            LIMIT ${limit_idx} OFFSET ${offset_idx}
            "#
        );
        let data_refs: Vec<&(dyn ToSql + Sync)> = params
            .iter()
            .map(|b| b.as_ref() as &(dyn ToSql + Sync))
            .collect();
        let rows = client
            .query(&data_sql, &data_refs)
            .await
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
                       COALESCE(a.deskripsi, a.nama) as aktivitas_label,
                       (SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker_terpilih WHERE pengajuan_id = p.id) as total_satker,
                       (SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker ps
                        WHERE ps.pengajuan_id = p.id AND ps.aktivitas_id = 1008) as satker_selesai
                FROM perlengkapan.pengajuan_pakaian_dinas p
                LEFT JOIN perlengkapan.ms_jenis_pakaian_dinas j ON p.jenis_pakaian_dinas_id = j.id
                LEFT JOIN perlengkapan.ms_aktivitas_bmn a ON p.aktivitas_id = a.kode
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
                INSERT INTO perlengkapan.pengajuan_pakaian_dinas
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
                    INSERT INTO perlengkapan.pengajuan_pakaian_dinas_pakaian
                        (id, pengajuan_id, jenis_pakaian_id, jenis_pakaian_nama,
                         spesifikasi_id, spesifikasi_nama, spesifikasi_ukuran_group)
                    SELECT $1, $2, s.jenis_pakaian_dinas_id, j.nama, s.id, s.nama, s.ukuran_group
                    FROM perlengkapan.ms_spesifikasi_pakaian_dinas s
                    LEFT JOIN perlengkapan.ms_jenis_pakaian_dinas j ON s.jenis_pakaian_dinas_id = j.id
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
        let resolved_satker_codes: Vec<String> = if request.pilihan_satker == "wilayah" {
            match request
                .wilayah_id
                .as_deref()
                .filter(|w| !w.trim().is_empty())
            {
                Some(wid) => self.list_satker_codes_by_wilayah(wid).await?,
                None => Vec::new(),
            }
        } else {
            request.satker_ids.clone().unwrap_or_default()
        };

        for satker_id in &resolved_satker_codes {
            client
                .execute(
                    r#"
                    INSERT INTO perlengkapan.pengajuan_pakaian_dinas_satker_terpilih
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

    /// Resolve `kode_satker` untuk satu wilayah Kejaksaan Tinggi (#19), sumber
    /// `integrasi.mysimkari_satker`. Kembar dgn
    /// `KebutuhanBmnRepository::list_satker_codes_by_wilayah` — nama dan bentuk
    /// sengaja disamakan.
    ///
    /// Sebelum V006/#94 fungsi ini `SELECT id` lalu membacanya sebagai `Uuid`,
    /// padahal kolomnya `BIGSERIAL` → tokio-postgres menolak konversi dan
    /// pembuatan campaign ber-scope "wilayah" selalu gagal.
    pub async fn list_satker_codes_by_wilayah(&self, wilayah: &str) -> AppResult<Vec<String>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let rows = client
            .query(
                "SELECT kode_satker FROM integrasi.mysimkari_satker \
                 WHERE wilayah = $1 ORDER BY kode_satker",
                &[&wilayah],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        Ok(rows
            .iter()
            .map(|r| r.get::<_, String>("kode_satker"))
            .collect())
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
        client.execute("DELETE FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran WHERE pengajuan_satker_id IN (SELECT id FROM perlengkapan.pengajuan_pakaian_dinas_satker WHERE pengajuan_id = $1)", &[&id]).await.ok();
        client.execute("DELETE FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai WHERE pengajuan_satker_id IN (SELECT id FROM perlengkapan.pengajuan_pakaian_dinas_satker WHERE pengajuan_id = $1)", &[&id]).await.ok();
        client.execute("DELETE FROM perlengkapan.pengajuan_pakaian_dinas_satker_aktivitas WHERE pengajuan_satker_id IN (SELECT id FROM perlengkapan.pengajuan_pakaian_dinas_satker WHERE pengajuan_id = $1)", &[&id]).await.ok();
        client
            .execute(
                "DELETE FROM perlengkapan.pengajuan_pakaian_dinas_satker WHERE pengajuan_id = $1",
                &[&id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM perlengkapan.pengajuan_pakaian_dinas_pakaian WHERE pengajuan_id = $1",
                &[&id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM perlengkapan.pengajuan_pakaian_dinas_satker_terpilih WHERE pengajuan_id = $1",
                &[&id],
            )
            .await
            .ok();

        let result = client
            .execute(
                "DELETE FROM perlengkapan.pengajuan_pakaian_dinas WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        if result == 0 {
            return Err(AppError::NotFound("Pengajuan tidak ditemukan".to_string()));
        }

        Ok(())
    }

    // ============ Pengajuan Satker ============
}
