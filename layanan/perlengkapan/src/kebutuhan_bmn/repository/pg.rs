use super::{KebutuhanBmnRepository, PgKebutuhanBmnRepository, UserInfo};
use crate::kebutuhan_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::satker_scope::{BoxedParam, SatkerScope, as_refs};
use async_trait::async_trait;
use serde_json::{Value, json};
use tracing::{error, info};
use uuid::Uuid;

#[async_trait]
impl KebutuhanBmnRepository for PgKebutuhanBmnRepository {
    async fn create_pengajuan(
        &self,
        request: CreatePengajuanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmn> {
        let client = self.get_client().await?;

        let pilihan_satker = request
            .pilihan_satker
            .clone()
            .unwrap_or_else(|| "semua".to_string());
        // V029: scope_satker = canonical kolom baru; pilihan_satker
        // (legacy) ikut diisi agar backward-compatible. Wilayah_id wajib
        // jika scope=wilayah (CHECK constraint di DB juga menegakkan).
        let scope_satker = pilihan_satker.clone();
        if scope_satker == "wilayah" && request.wilayah_id.as_deref().unwrap_or("").is_empty() {
            return Err(AppError::BadRequest(
                "wilayah_id wajib diisi ketika pilihan_satker = 'wilayah'".into(),
            ));
        }
        let asset_ids: Value = json!(
            request
                .asset_types
                .iter()
                .filter_map(|a| a.ms_jenis_asset_id)
                .collect::<Vec<_>>()
        );

        let row = client
            .query_one(
                r#"
                INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn
                    (nama, deskripsi, tahun, tgl_mulai, tgl_selesai, pilihan_satker,
                     scope_satker, wilayah_id, id_jenis_asset, created_by, updated_by)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10)
                RETURNING *
                "#,
                &[
                    &request.nama,
                    &request.deskripsi,
                    &request.tahun,
                    &request.tgl_mulai,
                    &request.tgl_selesai,
                    &pilihan_satker,
                    &scope_satker,
                    &request.wilayah_id,
                    &asset_ids,
                    &user_id,
                ],
            )
            .await
            .map_err(|e| {
                if let Some(db_err) = e.as_db_error() {
                    error!(
                        "Failed to create pengajuan: {} - {}",
                        db_err.message(),
                        db_err.detail().unwrap_or("")
                    );
                } else {
                    error!("Failed to create pengajuan: {}", e);
                }
                AppError::Internal(format!("Database error: {}", e))
            })?;

        let pengajuan = PengajuanKebutuhanBmn::from_row(&row);
        info!("Created pengajuan kebutuhan BMN: {}", pengajuan.id);

        // Create asset types
        for asset in &request.asset_types {
            self.create_pengajuan_asset(pengajuan.id, asset.clone())
                .await?;
        }

        // V029: scope=wilayah → auto-resolve satker dari
        // integrasi.v_satker_wilayah (Kejati penaung). Fallback ke satker_ids
        // eksplisit untuk scope lain.
        let satker_codes: Vec<String> = if scope_satker == "wilayah" {
            if let Some(ref wid) = request.wilayah_id {
                self.list_satker_codes_by_wilayah(wid).await?
            } else {
                Vec::new()
            }
        } else {
            request.satker_ids.clone()
        };

        for satker_id in &satker_codes {
            self.create_pengajuan_satker(pengajuan.id, satker_id, None, user_id)
                .await?;
        }

        // V029 (Fase 1.6): Insert allowed-list BMN. Empty = no whitelist
        // (legacy mode); operator dapat input bebas. Validator Pusat yang
        // memilih.
        for ref_req in &request.bmn_referensi_diizinkan {
            self.insert_bmn_referensi(pengajuan.id, ref_req.clone())
                .await?;
        }

        Ok(pengajuan)
    }

    async fn get_pengajuan_by_id(&self, id: Uuid) -> AppResult<PengajuanKebutuhanBmn> {
        let client = self.get_client().await?;

        let row = client
            .query_opt(
                "SELECT * FROM perlengkapan.pengajuan_kebutuhan_bmn WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Pengajuan {} not found", id)))?;

        Ok(PengajuanKebutuhanBmn::from_row(&row))
    }

    async fn get_all_pengajuan(
        &self,
        page: i32,
        per_page: i32,
        filter: Option<PengajuanFilter>,
        scope: &crate::shared::satker_scope::SatkerScope,
    ) -> AppResult<(Vec<KebutuhanBmnSummary>, i64)> {
        let client = self.get_client().await?;
        let offset = (page - 1) * per_page;

        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
        // Campaign-visibility predicate first (#66); its `$n` is resolved by
        // pushing onto `params`, so `param_idx` is seeded from the count below.
        if let Some(cond) =
            crate::kebutuhan_bmn::scope::campaign_visibility_condition(scope, &mut params)
        {
            conditions.push(cond);
        }
        let mut param_idx = params.len() + 1;

        if let Some(ref f) = filter {
            if let Some(tahun) = f.tahun {
                conditions.push(format!("tahun = ${}", param_idx));
                params.push(Box::new(tahun));
                param_idx += 1;
            }
            if let Some(status) = f.status_kode {
                conditions.push(format!("status_kode = ${}", param_idx));
                params.push(Box::new(status));
                param_idx += 1;
            }
            if let Some(ref search) = f.search {
                conditions.push(format!("nama ILIKE ${}", param_idx));
                params.push(Box::new(format!("%{}%", search)));
                param_idx += 1;
            }
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // Get total count
        let count_query = format!(
            "SELECT COUNT(*) FROM perlengkapan.vw_kebutuhan_bmn_summary {}",
            where_clause
        );

        let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let count_row = client
            .query_one(&count_query, &params_refs)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
        let total: i64 = count_row.get(0);

        // Get paginated data
        // Postgres infers LIMIT/OFFSET params as int8 (bigint); bind i64 so
        // tokio_postgres serialization matches the inferred type. (#33)
        params.push(Box::new(per_page as i64));
        params.push(Box::new(offset as i64));

        let data_query = format!(
            r#"
            SELECT * FROM perlengkapan.vw_kebutuhan_bmn_summary
            {}
            ORDER BY created_at DESC
            LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            param_idx,
            param_idx + 1
        );

        let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = client
            .query(&data_query, &params_refs)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        let summaries: Vec<KebutuhanBmnSummary> =
            rows.iter().map(KebutuhanBmnSummary::from_row).collect();

        Ok((summaries, total))
    }

    async fn get_rekap_laporan(
        &self,
        filter: RekapLaporanFilter,
        scope: &crate::shared::satker_scope::SatkerScope,
    ) -> AppResult<Vec<RekapLaporanRow>> {
        let client = self.get_client().await?;

        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();

        // Campaign visibility (#66/#71). The helper emits a bare `id IN (…)`
        // predicate meant for a single-table query; this one joins three
        // tables, so wrap it in a subquery against the campaign table — the
        // inner `id` then resolves unambiguously and the tested helper is
        // reused verbatim (including its fail-closed `FALSE` for Denied).
        if let Some(cond) =
            crate::kebutuhan_bmn::scope::campaign_visibility_condition(scope, &mut params)
        {
            conditions.push(format!(
                "p.id IN (SELECT id FROM perlengkapan.pengajuan_kebutuhan_bmn WHERE {})",
                cond
            ));
        }
        let mut param_idx = params.len() + 1;

        // Row-level satker scoping — NOT redundant with the campaign predicate
        // above. Campaign visibility answers "may I see this campaign exists?",
        // which for a nationwide (`scope_satker = 'semua'`) campaign is true for
        // everyone. This recap lists one row per *satker's* requested item, so
        // without the extra clause an operator would read every other satker's
        // line items off a nationwide campaign. Verified against a real DB: the
        // seeded 'semua' campaign returned all 3 satkers' barang to a single
        // satker caller until this was added.
        match scope {
            SatkerScope::All | SatkerScope::Denied => {}
            SatkerScope::Satker(code) => {
                conditions.push(format!("ps.satker_id = ${}", param_idx));
                params.push(Box::new(code.clone()));
                param_idx += 1;
            }
            SatkerScope::Wilayah(code) => {
                conditions.push(format!(
                    "ps.satker_id IN (SELECT s.kode_satker FROM integrasi.v_satker_wilayah s \
                     WHERE s.wilayah_code = (SELECT s2.wilayah_code FROM integrasi.v_satker_wilayah s2 \
                     WHERE s2.kode_satker = ${}))",
                    param_idx
                ));
                params.push(Box::new(code.clone()));
                param_idx += 1;
            }
            // A cross-satker reader drilled into one region: the region is
            // named directly instead of being resolved from a member satker.
            SatkerScope::WilayahKode(wilayah) => {
                conditions.push(format!(
                    "ps.satker_id IN (SELECT s.kode_satker FROM integrasi.v_satker_wilayah s \
                     WHERE s.wilayah_code = ${})",
                    param_idx
                ));
                params.push(Box::new(wilayah.clone()));
                param_idx += 1;
            }
        }

        if let Some(tahun) = filter.tahun {
            conditions.push(format!("p.tahun = ${}", param_idx));
            params.push(Box::new(tahun));
            param_idx += 1;
        }
        if let Some(status) = filter.status_kode {
            conditions.push(format!("ps.status_kode = ${}", param_idx));
            params.push(Box::new(status));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // `ms_aktivitas_bmn` is the same status-label lookup that
        // `vw_kebutuhan_bmn_summary` joins; LEFT so an unmapped code still
        // yields a row (with a NULL label) rather than dropping the item.
        let query = format!(
            r#"
            SELECT
                p.id            AS pengajuan_id,
                p.nama          AS pengajuan_nama,
                p.tahun         AS tahun,
                ps.satker_id    AS satker_id,
                ps.satker_nama  AS satker_nama,
                psb.kode_barang AS kode_barang,
                psb.nama        AS nama_barang,
                psb.satuan      AS satuan,
                psb.jumlah      AS jumlah,
                COALESCE(psb.jml_setuju, 0) AS jml_setuju,
                ps.status_kode  AS status_kode,
                m.nama          AS status_nama
            FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_barang psb
            JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker ps
              ON ps.id = psb.pengajuan_satker_id
            JOIN perlengkapan.pengajuan_kebutuhan_bmn p
              ON p.id = ps.pengajuan_id
            LEFT JOIN perlengkapan.ms_aktivitas_bmn m
              ON m.kode = ps.status_kode
            {}
            ORDER BY p.tahun DESC, ps.satker_nama ASC, psb.nama ASC
            "#,
            where_clause
        );

        let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = client
            .query(&query, &params_refs)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(rows
            .iter()
            .map(|r| RekapLaporanRow {
                pengajuan_id: r.get("pengajuan_id"),
                pengajuan_nama: r.get("pengajuan_nama"),
                tahun: r.get("tahun"),
                satker_id: r.get("satker_id"),
                satker_nama: r.get("satker_nama"),
                kode_barang: r.get("kode_barang"),
                nama_barang: r.get("nama_barang"),
                satuan: r.get("satuan"),
                jumlah: r.get("jumlah"),
                jml_setuju: r.get("jml_setuju"),
                status_kode: r.get("status_kode"),
                status_nama: r.get("status_nama"),
            })
            .collect())
    }

    async fn update_pengajuan(
        &self,
        id: Uuid,
        request: UpdatePengajuanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmn> {
        let client = self.get_client().await?;

        // First check version for optimistic locking
        let current = self.get_pengajuan_by_id(id).await?;
        if current.version != request.version {
            return Err(AppError::Conflict(
                "Data has been modified by another user. Please refresh and try again.".to_string(),
            ));
        }

        let mut updates = Vec::new();
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
        let mut param_idx = 1;

        if let Some(ref nama) = request.nama {
            updates.push(format!("nama = ${}", param_idx));
            params.push(Box::new(nama.clone()));
            param_idx += 1;
        }
        if let Some(ref deskripsi) = request.deskripsi {
            updates.push(format!("deskripsi = ${}", param_idx));
            params.push(Box::new(deskripsi.clone()));
            param_idx += 1;
        }
        if let Some(tgl_mulai) = request.tgl_mulai {
            updates.push(format!("tgl_mulai = ${}", param_idx));
            params.push(Box::new(tgl_mulai));
            param_idx += 1;
        }
        if let Some(tgl_selesai) = request.tgl_selesai {
            updates.push(format!("tgl_selesai = ${}", param_idx));
            params.push(Box::new(tgl_selesai));
            param_idx += 1;
        }
        if let Some(ref pilihan) = request.pilihan_satker {
            // V029: tulis ke kedua kolom (legacy + canonical) selama
            // backward compat masih dijaga.
            updates.push(format!("pilihan_satker = ${}", param_idx));
            params.push(Box::new(pilihan.clone()));
            param_idx += 1;
            updates.push(format!("scope_satker = ${}", param_idx));
            params.push(Box::new(pilihan.clone()));
            param_idx += 1;
        }
        if let Some(ref wid) = request.wilayah_id {
            updates.push(format!("wilayah_id = ${}", param_idx));
            params.push(Box::new(wid.clone()));
            param_idx += 1;
        }

        // Always update version and updated_by
        updates.push("version = version + 1".to_string());
        updates.push(format!("updated_by = ${}", param_idx));
        params.push(Box::new(user_id));
        param_idx += 1;

        // Add id and version for WHERE clause
        params.push(Box::new(id));
        params.push(Box::new(request.version));

        let query = format!(
            r#"
            UPDATE perlengkapan.pengajuan_kebutuhan_bmn
            SET {}
            WHERE id = ${} AND version = ${}
            RETURNING *
            "#,
            updates.join(", "),
            param_idx,
            param_idx + 1
        );

        let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let row = client
            .query_opt(&query, &params_refs)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::Conflict("Concurrent modification detected".to_string()))?;

        Ok(PengajuanKebutuhanBmn::from_row(&row))
    }

    async fn delete_pengajuan(&self, id: Uuid) -> AppResult<()> {
        let client = self.get_client().await?;

        // Check if pengajuan exists and is in draft status
        let pengajuan = self.get_pengajuan_by_id(id).await?;
        if pengajuan.status != KebutuhanBmnStatus::Draft {
            return Err(AppError::BadRequest(
                "Only draft pengajuan can be deleted".to_string(),
            ));
        }

        client
            .execute(
                "DELETE FROM perlengkapan.pengajuan_kebutuhan_bmn WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        info!("Deleted pengajuan kebutuhan BMN: {}", id);
        Ok(())
    }

    async fn get_pengajuan_assets(
        &self,
        pengajuan_id: Uuid,
    ) -> AppResult<Vec<PengajuanKebutuhanBmnAsset>> {
        let client = self.get_client().await?;

        let rows = client
            .query(
                "SELECT * FROM perlengkapan.pengajuan_kebutuhan_bmn_asset WHERE pengajuan_id = $1",
                &[&pengajuan_id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(rows
            .iter()
            .map(PengajuanKebutuhanBmnAsset::from_row)
            .collect())
    }

    async fn create_pengajuan_asset(
        &self,
        pengajuan_id: Uuid,
        request: CreateAssetTypeRequest,
    ) -> AppResult<PengajuanKebutuhanBmnAsset> {
        let client = self.get_client().await?;

        let row = client
            .query_one(
                r#"
                INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_asset
                    (pengajuan_id, kode_barang, nm_barang, ms_jenis_asset_id, keterangan)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING *
                "#,
                &[
                    &pengajuan_id,
                    &request.kode_barang,
                    &request.nm_barang,
                    &request.ms_jenis_asset_id,
                    &request.keterangan,
                ],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(PengajuanKebutuhanBmnAsset::from_row(&row))
    }

    async fn get_pengajuan_satkers(
        &self,
        pengajuan_id: Uuid,
        scope: &SatkerScope,
    ) -> AppResult<Vec<PengajuanKebutuhanBmnSatker>> {
        let client = self.get_client().await?;

        let mut params: Vec<BoxedParam> = vec![Box::new(pengajuan_id)];
        let scope_sql = crate::shared::satker_scope::scope_and(scope, "satker_id", &mut params);
        let rows = client
            .query(
                &format!(
                    r#"
                SELECT * FROM perlengkapan.pengajuan_kebutuhan_bmn_satker
                WHERE pengajuan_id = $1{scope_sql}
                ORDER BY prioritas ASC, satker_nama ASC
                "#
                ),
                &as_refs(&params),
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(rows
            .iter()
            .map(PengajuanKebutuhanBmnSatker::from_row)
            .collect())
    }

    async fn get_satker_by_id(
        &self,
        id: Uuid,
        scope: &SatkerScope,
    ) -> AppResult<PengajuanKebutuhanBmnSatker> {
        let client = self.get_client().await?;

        // The predicate rides in the WHERE rather than in a check the caller
        // makes afterwards: a check that happens in a later statement is a
        // check the next refactor can leave behind — and this row's readers
        // include four workflow actions that write.
        let mut params: Vec<BoxedParam> = vec![Box::new(id)];
        let scope_sql = crate::shared::satker_scope::scope_and(scope, "satker_id", &mut params);
        let row = client
            .query_opt(
                &format!(
                    "SELECT * FROM perlengkapan.pengajuan_kebutuhan_bmn_satker \
                     WHERE id = $1{scope_sql}"
                ),
                &as_refs(&params),
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
            // NotFound, not Forbidden: 403 confirms the id exists under some
            // other satker, which is the cross-tenant existence oracle #93
            // closed for satker detail — and then left open on its siblings.
            .ok_or_else(|| AppError::NotFound(format!("Satker {} not found", id)))?;

        Ok(PengajuanKebutuhanBmnSatker::from_row(&row))
    }

    async fn create_pengajuan_satker(
        &self,
        pengajuan_id: Uuid,
        satker_id: &str,
        satker_name: Option<String>,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnSatker> {
        let client = self.get_client().await?;

        let row = client
            .query_one(
                r#"
                INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker
                    (pengajuan_id, satker_id, satker_nama, created_by, updated_by)
                VALUES ($1, $2, $3, $4, $4)
                ON CONFLICT (pengajuan_id, satker_id) DO UPDATE
                SET updated_at = NOW()
                RETURNING *
                "#,
                &[&pengajuan_id, &satker_id, &satker_name, &user_id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(PengajuanKebutuhanBmnSatker::from_row(&row))
    }

    async fn update_satker_status(
        &self,
        satker_id: Uuid,
        new_status: i32,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnSatker> {
        let client = self.get_client().await?;

        let row = client
            .query_one(
                r#"
                UPDATE perlengkapan.pengajuan_kebutuhan_bmn_satker
                SET status_kode = $1, updated_by = $2
                WHERE id = $3
                RETURNING *
                "#,
                &[&new_status, &user_id, &satker_id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(PengajuanKebutuhanBmnSatker::from_row(&row))
    }

    async fn get_satker_barang(
        &self,
        satker_id: Uuid,
        page: i32,
        per_page: i32,
        filter: Option<BarangFilter>,
    ) -> AppResult<(Vec<PengajuanKebutuhanBmnBarang>, i64)> {
        let client = self.get_client().await?;
        let offset = (page - 1) * per_page;

        let mut conditions = vec!["pengajuan_satker_id = $1".to_string()];
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> =
            vec![Box::new(satker_id)];
        let mut param_idx = 2;

        if let Some(ref f) = filter {
            if let Some(ref kode) = f.kode_barang {
                conditions.push(format!("kode_barang = ${}", param_idx));
                params.push(Box::new(kode.clone()));
                param_idx += 1;
            }
            if let Some(min_prioritas) = f.prioritas_min {
                conditions.push(format!("prioritas >= ${}", param_idx));
                params.push(Box::new(min_prioritas));
                param_idx += 1;
            }
            if let Some(ref search) = f.search {
                conditions.push(format!("nama ILIKE ${}", param_idx));
                params.push(Box::new(format!("%{}%", search)));
                param_idx += 1;
            }
        }

        let where_clause = conditions.join(" AND ");

        // Get total count
        let count_query = format!(
            "SELECT COUNT(*) FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_barang WHERE {}",
            where_clause
        );

        let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let count_row = client
            .query_one(&count_query, &params_refs)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
        let total: i64 = count_row.get(0);

        // Get paginated data
        // Postgres infers LIMIT/OFFSET params as int8 (bigint); bind i64 so
        // tokio_postgres serialization matches the inferred type. (#33)
        params.push(Box::new(per_page as i64));
        params.push(Box::new(offset as i64));

        let data_query = format!(
            r#"
            SELECT * FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_barang
            WHERE {}
            ORDER BY prioritas ASC, created_at ASC
            LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            param_idx,
            param_idx + 1
        );

        let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = client
            .query(&data_query, &params_refs)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        let items: Vec<PengajuanKebutuhanBmnBarang> = rows
            .iter()
            .map(PengajuanKebutuhanBmnBarang::from_row)
            .collect();

        Ok((items, total))
    }

    async fn get_barang_by_id(&self, id: Uuid) -> AppResult<PengajuanKebutuhanBmnBarang> {
        let client = self.get_client().await?;

        let row = client
            .query_opt(
                "SELECT * FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_barang WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Barang {} not found", id)))?;

        Ok(PengajuanKebutuhanBmnBarang::from_row(&row))
    }

    async fn create_barang(
        &self,
        satker_id: Uuid,
        request: CreateBarangRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnBarang> {
        let client = self.get_client().await?;

        let file_pendukung: Value = json!(request.file_pendukung);

        let row = client
            .query_one(
                r#"
                INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker_barang
                    (pengajuan_satker_id, nama, kode_barang, jumlah, satuan,
                     alasan, keterangan, file_pendukung, created_by, updated_by)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
                RETURNING *
                "#,
                &[
                    &satker_id,
                    &request.nama,
                    &request.kode_barang,
                    &request.jumlah,
                    &request.satuan,
                    &request.alasan,
                    &request.keterangan,
                    &file_pendukung,
                    &user_id,
                ],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        info!("Created barang for satker {}: {}", satker_id, request.nama);
        Ok(PengajuanKebutuhanBmnBarang::from_row(&row))
    }

    async fn update_barang_approval(
        &self,
        barang_id: Uuid,
        request: UpdateBarangApprovalRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnBarang> {
        let client = self.get_client().await?;

        let row = client
            .query_one(
                r#"
                UPDATE perlengkapan.pengajuan_kebutuhan_bmn_satker_barang
                SET jml_setuju = $1, keterangan = COALESCE($2, keterangan), updated_by = $3
                WHERE id = $4
                RETURNING *
                "#,
                &[
                    &request.jml_setuju,
                    &request.keterangan,
                    &user_id,
                    &barang_id,
                ],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(PengajuanKebutuhanBmnBarang::from_row(&row))
    }

    async fn delete_barang(&self, id: Uuid) -> AppResult<()> {
        let client = self.get_client().await?;

        client
            .execute(
                "DELETE FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_barang WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        info!("Deleted barang: {}", id);
        Ok(())
    }

    async fn set_barang_prioritas(&self, items: Vec<PrioritasItem>) -> AppResult<()> {
        let client = self.get_client().await?;

        for item in items {
            client
                .execute(
                    r#"
                    UPDATE perlengkapan.pengajuan_kebutuhan_bmn_satker_barang
                    SET prioritas = $1, skor = COALESCE($2, skor)
                    WHERE id = $3
                    "#,
                    &[&item.prioritas, &item.skor, &item.barang_id],
                )
                .await
                .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
        }

        Ok(())
    }

    async fn count_siman_assets_for(
        &self,
        satker_code: &str,
        kode_barang: &str,
        sample_limit: i64,
    ) -> AppResult<(i32, Vec<ExistingAssetInfo>)> {
        use crate::shared::siman_columns::{
            ASSET_KONDISI_SQL, ASSET_NAMA_BARANG_SQL, ASSET_NUP_SQL, kode_barang_norm_sql,
            normalize_kode_barang,
        };

        // A barang with no code cannot be matched against SIMAN at all; say so
        // by returning nothing rather than matching every asset in the satker.
        if kode_barang.trim().is_empty() {
            return Ok((0, Vec::new()));
        }

        let client = self.get_client().await?;
        let norm = normalize_kode_barang(kode_barang);

        // MySIMKARI `kode_satker` -> SIMAN `kdsatker_keu` through the
        // integrasi-owned map. The two coding schemes differ, and matching on
        // `nama_satker` is the string-equality trap that map exists to replace.
        let where_sql = format!(
            "WHERE {kode} = $1 \
             AND kdsatker_keu IN (SELECT kdsatker_keu FROM integrasi.v_satker_code_map \
                                  WHERE kode_satker = $2 AND kdsatker_keu IS NOT NULL)",
            kode = kode_barang_norm_sql("")
        );

        let count_row = client
            .query_one(
                &format!("SELECT COUNT(*)::int8 AS total FROM integrasi.siman_aset {where_sql}"),
                &[&norm, &satker_code],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
        let total: i64 = count_row.get("total");

        let rows = client
            .query(
                &format!(
                    "SELECT {ASSET_NUP_SQL} AS no_aset, \
                            {ASSET_NAMA_BARANG_SQL} AS nama_aset, \
                            {ASSET_KONDISI_SQL} AS kondisi, \
                            NULLIF(alamat, '') AS lokasi \
                     FROM integrasi.siman_aset {where_sql} \
                     ORDER BY id \
                     LIMIT $3"
                ),
                &[&norm, &satker_code, &sample_limit],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        let sample = rows
            .iter()
            .map(|r| ExistingAssetInfo {
                no_aset: r
                    .try_get::<_, Option<String>>("no_aset")
                    .ok()
                    .flatten()
                    .unwrap_or_default(),
                nama_aset: r
                    .try_get::<_, Option<String>>("nama_aset")
                    .ok()
                    .flatten()
                    .unwrap_or_default(),
                kondisi: r
                    .try_get::<_, Option<String>>("kondisi")
                    .ok()
                    .flatten()
                    .unwrap_or_default(),
                lokasi: r.try_get::<_, Option<String>>("lokasi").ok().flatten(),
            })
            .collect();

        // The count is what the gap is computed from, so it must not be capped
        // by the sample limit. i32 to match the column the analysis compares
        // against; a satker holding >2 billion of one barang code is not a
        // case this system has.
        Ok((total.min(i64::from(i32::MAX)) as i32, sample))
    }

    async fn get_satker_aktivitas(
        &self,
        satker_id: Uuid,
        scope: &SatkerScope,
    ) -> AppResult<Vec<PengajuanKebutuhanBmnAktivitas>> {
        let client = self.get_client().await?;

        // The activity table has no satker column of its own, so the scope is
        // applied to its parent: you must be able to see the participation row
        // before you can read who acted on it. Measured on staging, this
        // returned another satker's trail — and another WILAYAH's — complete
        // with each validator's user id and NIP.
        let mut params: Vec<BoxedParam> = vec![Box::new(satker_id)];
        let scope_sql = crate::shared::satker_scope::scope_and(scope, "s.satker_id", &mut params);
        let rows = client
            .query(
                &format!(
                    r#"
                SELECT a.* FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas a
                JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker s
                  ON s.id = a.pengajuan_satker_id
                WHERE a.pengajuan_satker_id = $1{scope_sql}
                ORDER BY a.created_at DESC
                "#
                ),
                &as_refs(&params),
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(rows
            .iter()
            .map(PengajuanKebutuhanBmnAktivitas::from_row)
            .collect())
    }

    async fn create_aktivitas(
        &self,
        satker_id: Uuid,
        from_status: Option<i32>,
        to_status: i32,
        aksi: &str,
        komentar: Option<String>,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
    ) -> AppResult<PengajuanKebutuhanBmnAktivitas> {
        let client = self.get_client().await?;

        let (nip, nama, pangkat, jabatan, role) = user_info
            .map(|u| (u.nip, u.nama, u.pangkat, u.jabatan, u.role))
            .unwrap_or_default();

        let row = client
            .query_one(
                r#"
                INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
                    (pengajuan_satker_id, from_status_kode, to_status_kode, user_id,
                     nip, nama, pangkat, jabatan, role, aksi, komentar)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                RETURNING *
                "#,
                &[
                    &satker_id,
                    &from_status,
                    &to_status,
                    &user_id,
                    &nip,
                    &nama,
                    &pangkat,
                    &jabatan,
                    &role,
                    &aksi,
                    &komentar,
                ],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(PengajuanKebutuhanBmnAktivitas::from_row(&row))
    }

    async fn update_pengajuan_status(
        &self,
        id: Uuid,
        new_status: i32,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmn> {
        let client = self.get_client().await?;

        let row = client
            .query_one(
                r#"
                UPDATE perlengkapan.pengajuan_kebutuhan_bmn
                SET status_kode = $1, updated_by = $2, version = version + 1
                WHERE id = $3
                RETURNING *
                "#,
                &[&new_status, &user_id, &id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(PengajuanKebutuhanBmn::from_row(&row))
    }

    async fn update_satker_submit_data(
        &self,
        satker_id: Uuid,
        catatan_satker: Option<String>,
        lampiran_surat_permohonan: Option<String>,
        lampiran_pendukung: Option<Vec<LampiranItem>>,
    ) -> AppResult<()> {
        let client = self.get_client().await?;

        let lampiran_json = lampiran_pendukung.map(|v| serde_json::to_value(v).unwrap_or_default());

        client
            .execute(
                r#"
                UPDATE perlengkapan.pengajuan_kebutuhan_bmn_satker
                SET catatan_satker = COALESCE($1, catatan_satker),
                    lampiran_surat_permohonan = COALESCE($2, lampiran_surat_permohonan),
                    lampiran_pendukung = COALESCE($3, lampiran_pendukung),
                    updated_at = NOW()
                WHERE id = $4
                "#,
                &[
                    &catatan_satker,
                    &lampiran_surat_permohonan,
                    &lampiran_json,
                    &satker_id,
                ],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn save_analisis_snapshot(&self, satker_id: Uuid, snapshot: &Value) -> AppResult<()> {
        let client = self.get_client().await?;

        client
            .execute(
                r#"
                UPDATE perlengkapan.pengajuan_kebutuhan_bmn_satker
                SET analisis_snapshot_at_submit = $1,
                    updated_at = NOW()
                WHERE id = $2
                "#,
                &[snapshot, &satker_id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn get_analisis_snapshot(&self, satker_id: Uuid) -> AppResult<Option<Value>> {
        let client = self.get_client().await?;

        let row = client
            .query_opt(
                "SELECT analisis_snapshot_at_submit FROM perlengkapan.pengajuan_kebutuhan_bmn_satker WHERE id = $1",
                &[&satker_id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(row.and_then(|r| r.try_get::<_, Option<Value>>(0).ok().flatten()))
    }

    async fn update_satker_validator_wilayah(
        &self,
        satker_id: Uuid,
        validator_id: Option<Uuid>,
        catatan: Option<String>,
    ) -> AppResult<()> {
        let client = self.get_client().await?;

        client
            .execute(
                r#"
                UPDATE perlengkapan.pengajuan_kebutuhan_bmn_satker
                SET validator_wilayah_id = COALESCE($1, validator_wilayah_id),
                    catatan_validator_wilayah = COALESCE($2, catatan_validator_wilayah),
                    tanggal_submit_wilayah = NOW(),
                    updated_at = NOW()
                WHERE id = $3
                "#,
                &[&validator_id, &catatan, &satker_id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn update_satker_validator_pusat(
        &self,
        satker_id: Uuid,
        validator_id: Option<Uuid>,
        catatan: Option<String>,
        is_approved: bool,
    ) -> AppResult<()> {
        let client = self.get_client().await?;

        client
            .execute(
                r#"
                UPDATE perlengkapan.pengajuan_kebutuhan_bmn_satker
                SET validator_pusat_id = COALESCE($1, validator_pusat_id),
                    catatan_validator_pusat = COALESCE($2, catatan_validator_pusat),
                    tanggal_submit_pusat = NOW(),
                    is_approved = $3,
                    alasan_keputusan = $2,
                    updated_at = NOW()
                WHERE id = $4
                "#,
                &[&validator_id, &catatan, &is_approved, &satker_id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn update_pengajuan_laporan(
        &self,
        id: Uuid,
        laporan_url: &str,
        laporan_format: &str,
    ) -> AppResult<()> {
        let client = self.get_client().await?;

        client
            .execute(
                r#"
                UPDATE perlengkapan.pengajuan_kebutuhan_bmn
                SET laporan_url = $1,
                    laporan_format = $2,
                    laporan_generated_at = NOW(),
                    updated_at = NOW()
                WHERE id = $3
                "#,
                &[&laporan_url, &laporan_format, &id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn list_satker_codes_by_wilayah(&self, wilayah: &str) -> AppResult<Vec<String>> {
        let client = self.get_client().await?;
        let rows = client
            .query(
                r#"
                SELECT kode_satker
                FROM integrasi.v_satker_wilayah
                WHERE wilayah_code = $1
                ORDER BY kode_satker
                "#,
                &[&wilayah],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
        Ok(rows
            .iter()
            .map(|r| r.get::<_, String>("kode_satker"))
            .collect())
    }

    async fn list_wilayah(&self) -> AppResult<Vec<crate::kebutuhan_bmn::models::WilayahKejati>> {
        let client = self.get_client().await?;
        let rows = client
            .query(
                r#"
                SELECT DISTINCT wilayah_code, wilayah_nama
                FROM integrasi.v_satker_wilayah
                ORDER BY wilayah_nama
                "#,
                &[],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
        Ok(rows
            .iter()
            .map(|r| crate::kebutuhan_bmn::models::WilayahKejati {
                kode: r.get("wilayah_code"),
                nama: r
                    .get::<_, Option<String>>("wilayah_nama")
                    .unwrap_or_default(),
            })
            .collect())
    }

    async fn satker_code_in_scope(&self, scope: &SatkerScope, code: &str) -> AppResult<bool> {
        // Tier murni (All/Denied/Satker) diputuskan tanpa menyentuh DB, dan
        // aturannya hidup di SatkerScope agar tak ada salinan kedua di sini.
        // `unwrap_or(false)` = fail-closed; secara praktis tak terjangkau
        // karena Wilayah adalah satu-satunya varian yang mengembalikan None.
        let SatkerScope::Wilayah(caller) = scope else {
            return Ok(scope.contains_code_local(code).unwrap_or(false));
        };

        // Kueri-nya = `SatkerScope::WILAYAH_MEMBERSHIP_SQL`, satu-satunya
        // salinan: sengaja kembar dgn cabang Wilayah di
        // `SatkerScope::push_condition` ("satker target berada di wilayah yang
        // sama dengan satker pemanggil", di-resolve dari
        // `integrasi.v_satker_wilayah`), hanya berbentuk EXISTS karena di sini
        // kita menguji SATU kode, bukan memfilter sekumpulan baris.
        let client = self.get_client().await?;
        let row = client
            .query_one(SatkerScope::WILAYAH_MEMBERSHIP_SQL, &[&code, &caller])
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
        Ok(row.get::<_, bool>("in_scope"))
    }

    async fn insert_bmn_referensi(
        &self,
        pengajuan_id: Uuid,
        request: CreateBmnReferensiRequest,
    ) -> AppResult<PengajuanBmnReferensi> {
        let client = self.get_client().await?;
        let row = client
            .query_one(
                r#"
                INSERT INTO perlengkapan.pengajuan_bmn_referensi_diizinkan
                    (pengajuan_id, kode_barang, nama_barang, keterangan)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (pengajuan_id, kode_barang) DO UPDATE
                SET nama_barang = EXCLUDED.nama_barang,
                    keterangan = EXCLUDED.keterangan
                RETURNING id, pengajuan_id, kode_barang, nama_barang, keterangan, created_at
                "#,
                &[
                    &pengajuan_id,
                    &request.kode_barang,
                    &request.nama_barang,
                    &request.keterangan,
                ],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
        Ok(PengajuanBmnReferensi::from_row(&row))
    }

    async fn list_bmn_referensi(
        &self,
        pengajuan_id: Uuid,
    ) -> AppResult<Vec<PengajuanBmnReferensi>> {
        let client = self.get_client().await?;
        let rows = client
            .query(
                r#"
                SELECT id, pengajuan_id, kode_barang, nama_barang, keterangan, created_at
                FROM perlengkapan.pengajuan_bmn_referensi_diizinkan
                WHERE pengajuan_id = $1
                ORDER BY kode_barang
                "#,
                &[&pengajuan_id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
        Ok(rows.iter().map(PengajuanBmnReferensi::from_row).collect())
    }

    async fn is_bmn_allowed_for_pengajuan(
        &self,
        pengajuan_id: Uuid,
        kode_barang: &str,
    ) -> AppResult<bool> {
        let client = self.get_client().await?;
        let total: i64 = client
            .query_one(
                "SELECT COUNT(*) FROM perlengkapan.pengajuan_bmn_referensi_diizinkan WHERE pengajuan_id = $1",
                &[&pengajuan_id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
            .get(0);
        if total == 0 {
            return Ok(true);
        }
        let row = client
            .query_opt(
                r#"
                SELECT 1
                FROM perlengkapan.pengajuan_bmn_referensi_diizinkan
                WHERE pengajuan_id = $1 AND kode_barang = $2
                "#,
                &[&pengajuan_id, &kode_barang],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
        Ok(row.is_some())
    }
}
