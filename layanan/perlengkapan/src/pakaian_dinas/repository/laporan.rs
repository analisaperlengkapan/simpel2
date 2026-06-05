use super::PakaianDinasRepository;
use crate::pakaian_dinas::models::*;
use crate::shared::error::{AppResult, bad_request};
use tokio_postgres::types::ToSql;
use uuid::Uuid;

impl PakaianDinasRepository {
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

        let mut params: Vec<Box<dyn ToSql + Sync + Send>> = vec![Box::new(pengajuan_id)];
        let mut idx: usize = 2;
        if let Some(ref jk) = filter.jenis_kelamin {
            query.push_str(&format!(" AND psp.jenis_kelamin = ${}", idx));
            params.push(Box::new(jk.clone()));
            idx += 1;
        }
        if let Some(ref eselon) = filter.eselon {
            query.push_str(&format!(" AND psp.eselon = ${}", idx));
            params.push(Box::new(eselon.clone()));
            idx += 1;
        }
        if let Some(ref jenis) = filter.jenis {
            query.push_str(&format!(" AND psp.jenis = ${}", idx));
            params.push(Box::new(jenis.clone()));
            idx += 1;
        }
        let _ = idx;

        query.push_str(" GROUP BY pp.spesifikasi_nama, pp.spesifikasi_ukuran_group, pu.ukuran ORDER BY pp.spesifikasi_nama, pu.ukuran");

        let param_refs: Vec<&(dyn ToSql + Sync)> =
            params.iter().map(|b| &**b as &(dyn ToSql + Sync)).collect();
        let rows = client
            .query(&query, &param_refs[..])
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

        // Build dynamic filter with parameter binding
        let mut where_clause = "WHERE ps.pengajuan_id = $1 AND ps.aktivitas_id = 1008".to_string();
        let mut params: Vec<Box<dyn ToSql + Sync + Send>> = vec![Box::new(pengajuan_id)];
        let mut idx: usize = 2;
        if let Some(ref jk) = filter.jenis_kelamin {
            where_clause.push_str(&format!(" AND psp.jenis_kelamin = ${}", idx));
            params.push(Box::new(jk.clone()));
            idx += 1;
        }
        if let Some(ref satker_id) = filter.satker_id {
            where_clause.push_str(&format!(" AND ps.satker_id = ${}", idx));
            params.push(Box::new(*satker_id));
            idx += 1;
        }
        if let Some(ref eselon) = filter.eselon {
            where_clause.push_str(&format!(" AND psp.eselon = ${}", idx));
            params.push(Box::new(eselon.clone()));
            idx += 1;
        }
        if let Some(ref jenis) = filter.jenis {
            where_clause.push_str(&format!(" AND psp.jenis = ${}", idx));
            params.push(Box::new(jenis.clone()));
            idx += 1;
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

        let count_refs: Vec<&(dyn ToSql + Sync)> =
            params.iter().map(|b| &**b as &(dyn ToSql + Sync)).collect();
        let count_row = client
            .query_one(&count_query, &count_refs[..])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let total: i64 = count_row.get("total");

        // LIMIT/OFFSET placeholders take the next two indices after the filter params.
        let limit_idx = idx;
        let offset_idx = idx + 1;
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
            LIMIT ${} OFFSET ${}
            "#,
            where_clause, limit_idx, offset_idx
        );

        params.push(Box::new(per_page as i64));
        params.push(Box::new(offset as i64));
        let data_refs: Vec<&(dyn ToSql + Sync)> =
            params.iter().map(|b| &**b as &(dyn ToSql + Sync)).collect();
        let rows = client
            .query(&data_query, &data_refs[..])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let items: Vec<LaporanDaftarPegawai> =
            rows.iter().map(LaporanDaftarPegawai::from_row).collect();
        Ok((items, total))
    }

    // ============ Workflow Methods ============
}
