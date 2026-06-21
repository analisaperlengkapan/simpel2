use super::PemakaianBmnRepository;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::repo::PoolExt;

impl PemakaianBmnRepository {
    /// List permits with pagination and filters.
    ///
    /// `scope` enforces tiered RBAC data visibility on the authoritative
    /// `satker_code` column (#66): operator/validator_satker see their own
    /// satker, validator_wilayah their wilayah, pusat/admin everything, and an
    /// unidentified caller sees nothing (fail-closed).
    /// Requirements: REQ-P001, REQ-P011
    pub async fn list(
        &self,
        query: ListPermitsQuery,
        scope: &crate::shared::satker_scope::SatkerScope,
    ) -> AppResult<PaginatedPermitsResponse> {
        let client = self.pool.client().await?;

        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * per_page;

        let mut where_clauses = vec![];
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = vec![];
        // RBAC data-visibility predicate on the authoritative satker_code (#66).
        if let Some(cond) = scope.push_condition("satker_code", &mut params) {
            where_clauses.push(cond);
        }
        let mut param_count = params.len() + 1;

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
}
