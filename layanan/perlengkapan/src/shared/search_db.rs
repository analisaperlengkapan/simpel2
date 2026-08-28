//! Database-backed full-text search engine.
//!
//! Moved out of `lib-perlengkapan` (F0-C): the SQL/`tokio-postgres` search
//! implementation belongs in the service's infra layer, keeping
//! `lib-perlengkapan` a pure WASM-safe DTO/domain crate. The WASM-safe query
//! and result DTOs (`SearchQuery`, `SearchResults`, `SearchResult`) still come
//! from [`lib_perlengkapan::search`].

use deadpool_postgres::Pool;
use tokio_postgres::Row;

use lib_perlengkapan::search::{SearchQuery, SearchResult, SearchResults};

/// Search engine with database operations (PostgreSQL Indonesian full-text).
pub struct SearchEngineDb {
    db_pool: Pool,
}

impl SearchEngineDb {
    /// Create a new search engine with database pool
    pub fn new(db_pool: Pool) -> Self {
        Self { db_pool }
    }

    /// Search kebutuhan BMN with full-text search and filters
    ///
    /// Uses PostgreSQL's Indonesian text search configuration.
    ///
    /// Rows come from `vw_kebutuhan_bmn_summary`, joined to the base table for
    /// the columns the full-text expression reads. The previous form selected
    /// `k.*` from the base table alone and handed the result to a mapper that
    /// reads the view's derived columns, so the FIRST matching row panicked on
    /// `invalid column status_nama` — and with `panic = "abort"` in the release
    /// profile that ends the process, not the request. A probe sees 200 while
    /// the search happens to match nothing, which is exactly what staging
    /// showed.
    pub async fn search_kebutuhan<T>(
        &self,
        query: SearchQuery,
    ) -> Result<SearchResults<T>, Box<dyn std::error::Error + Send + Sync>>
    where
        T: for<'a> TryFrom<&'a Row, Error = Box<dyn std::error::Error + Send + Sync>>,
    {
        let client = self.db_pool.get().await?;

        let from_and_where = String::from(
            r#"
            FROM perlengkapan.vw_kebutuhan_bmn_summary v
            JOIN perlengkapan.pengajuan_kebutuhan_bmn k ON k.id = v.id
            WHERE to_tsvector('indonesian', k.nama || ' ' || COALESCE(k.deskripsi, ''))
                  @@ plainto_tsquery('indonesian', $1)
            "#,
        );

        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> =
            vec![Box::new(query.query.clone())];
        let mut filters = String::new();
        let mut param_idx = 2;

        // A campaign has no satker column of its own — it is nationwide and the
        // satkers hang off it. `k.satker_id` named nothing, and the filter was
        // typed `Uuid` besides; satker keys have been MySIMKARI `kode_satker`
        // (TEXT) since V006/#94.
        if let Some(ref satker_id) = query.filters.satker_id {
            filters.push_str(&format!(
                " AND EXISTS (SELECT 1 FROM perlengkapan.pengajuan_kebutuhan_bmn_satker s \
                  WHERE s.pengajuan_id = k.id AND s.satker_id = ${})",
                param_idx
            ));
            params.push(Box::new(satker_id.clone()));
            param_idx += 1;
        }

        if let Some(tahun) = query.filters.tahun_anggaran {
            filters.push_str(&format!(" AND k.tahun = ${}", param_idx));
            params.push(Box::new(tahun));
            param_idx += 1;
        }

        // `status_kode` is an integer and the filter arrives as text, so the
        // old `k.status_kode = ANY($n)` could not bind at all. Match the human
        // name the view resolves, or the code written as text, so both a
        // "SubmitWilayah" and a "2002" from the query string work.
        if let Some(ref status_list) = query.filters.status
            && !status_list.is_empty()
        {
            filters.push_str(&format!(
                " AND (v.status_nama = ANY(${idx}) OR k.status_kode::text = ANY(${idx}))",
                idx = param_idx
            ));
            params.push(Box::new(status_list.clone()));
            param_idx += 1;
        }

        // Likewise `kode_barang`: it lives on the barang rows two levels down,
        // never on the campaign.
        if let Some(ref kode) = query.filters.kode_barang {
            filters.push_str(&format!(
                " AND EXISTS (SELECT 1 FROM perlengkapan.pengajuan_kebutuhan_bmn_satker s \
                  JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker_barang b \
                    ON b.pengajuan_satker_id = s.id \
                  WHERE s.pengajuan_id = k.id AND b.kode_barang ILIKE ${})",
                param_idx
            ));
            params.push(Box::new(format!("%{}%", kode)));
            param_idx += 1;
        }

        // The dates arrive as strings. `$n::timestamptz` alone is not enough:
        // Postgres resolves an otherwise-unconstrained parameter to the cast's
        // TARGET type, so the placeholder still comes out timestamptz while the
        // client sends text, and the bind fails before any row is read.
        // `$n::text::timestamptz` pins the parameter to text first.
        if let Some(ref date_from) = query.filters.date_from {
            filters.push_str(&format!(
                " AND k.created_at >= ${}::text::timestamptz",
                param_idx
            ));
            params.push(Box::new(date_from.clone()));
            param_idx += 1;
        }

        if let Some(ref date_to) = query.filters.date_to {
            filters.push_str(&format!(
                " AND k.created_at <= ${}::text::timestamptz",
                param_idx
            ));
            params.push(Box::new(date_to.clone()));
            param_idx += 1;
        }
        let _ = param_idx;

        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        // Counted from the same FROM/WHERE rather than by splitting the data
        // query's text on the word LIMIT.
        let count_sql = format!("SELECT COUNT(*){from_and_where}{filters}");
        let total: i64 = client.query_one(&count_sql, &param_refs).await?.get(0);

        let sql = format!(
            "SELECT v.*, ts_rank(\
               to_tsvector('indonesian', k.nama || ' ' || COALESCE(k.deskripsi, '')), \
               plainto_tsquery('indonesian', $1)) AS relevance_score\
             {from_and_where}{filters} ORDER BY {order} LIMIT {limit} OFFSET {offset}",
            order = Self::campaign_order_by(&query.sort),
            limit = query.pagination.limit(),
            offset = query.pagination.offset(),
        );

        let rows = client.query(&sql, &param_refs).await?;

        let mut results = Vec::new();
        for row in rows {
            let relevance_score: f32 = row.try_get("relevance_score").unwrap_or(0.0);
            let item = T::try_from(&row)?;
            results.push(SearchResult::new(item, relevance_score as f64));
        }

        Ok(SearchResults::new(
            results,
            total,
            query.pagination.page,
            query.pagination.per_page,
            query.query,
        ))
    }

    /// ORDER BY for the campaign search, mapped to columns this query actually
    /// selects.
    ///
    /// `SortOptions::to_sql` is shared across surfaces and names
    /// `priority_score`, `gap` and `tahun_anggaran`, none of which exist on a
    /// campaign row — each was a "column does not exist" waiting for a caller
    /// to pick that sort. Ordering is not user input (it comes from a parsed
    /// enum), so the risk was a 500, not an injection; the fix is to let each
    /// surface say which columns it has.
    fn campaign_order_by(sort: &lib_perlengkapan::search::SortOptions) -> String {
        use lib_perlengkapan::search::{SortDirection, SortField};
        let column = match sort.field {
            SortField::Relevance => "relevance_score",
            SortField::CreatedAt => "created_at",
            SortField::UpdatedAt => "updated_at",
            SortField::TahunAnggaran => "tahun",
            // A campaign has neither; fall back to relevance rather than
            // emitting a column name the query cannot resolve.
            SortField::Priority | SortField::Gap => "relevance_score",
        };
        let direction = match sort.direction {
            SortDirection::Ascending => "ASC",
            SortDirection::Descending => "DESC",
        };
        // `id` breaks ties so paging is stable across requests.
        format!("{column} {direction}, v.id ASC")
    }

    /// Search kebutuhan BMN barang (items within kebutuhan)
    pub async fn search_kebutuhan_barang<T>(
        &self,
        query: SearchQuery,
    ) -> Result<SearchResults<T>, Box<dyn std::error::Error + Send + Sync>>
    where
        T: for<'a> TryFrom<&'a Row, Error = Box<dyn std::error::Error + Send + Sync>>,
    {
        let client = self.db_pool.get().await?;

        let mut sql = String::from(
            r#"
            SELECT
                b.*,
                ts_rank(
                    to_tsvector('indonesian', b.nama || ' ' || COALESCE(b.keterangan, '')),
                    plainto_tsquery('indonesian', $1)
                ) as relevance_score
            FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_barang b
            WHERE to_tsvector('indonesian', b.nama || ' ' || COALESCE(b.keterangan, ''))
                  @@ plainto_tsquery('indonesian', $1)
            "#,
        );

        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> =
            vec![Box::new(query.query.clone())];
        let mut param_idx = 2;

        // Add filters
        if let Some(ref kode) = query.filters.kode_barang {
            sql.push_str(&format!(" AND b.kode_barang ILIKE ${}", param_idx));
            params.push(Box::new(format!("%{}%", kode)));
            param_idx += 1;
        }

        if let Some(ref priority_levels) = query.filters.priority_level
            && !priority_levels.is_empty()
        {
            sql.push_str(&format!(" AND b.prioritas = ANY(${})", param_idx));
            // Convert priority levels to integers
            let priority_ints: Vec<i32> = priority_levels
                .iter()
                .filter_map(|p| p.parse().ok())
                .collect();
            params.push(Box::new(priority_ints));
            param_idx += 1;
        }
        let _ = param_idx;

        // Add sorting
        sql.push_str(&format!(" ORDER BY {}", query.sort.to_sql()));

        // Add pagination
        sql.push_str(&format!(
            " LIMIT {} OFFSET {}",
            query.pagination.limit(),
            query.pagination.offset()
        ));

        // Execute query
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = client.query(&sql, &param_refs).await?;

        // Convert rows to search results
        let mut results = Vec::new();
        for row in rows {
            let relevance_score: f32 = row.try_get("relevance_score").unwrap_or(0.0);
            let item = T::try_from(&row)?;
            results.push(SearchResult::new(item, relevance_score as f64));
        }

        // Get total count
        let count_sql = format!(
            "SELECT COUNT(*) FROM ({}) as subquery",
            sql.split("LIMIT").next().unwrap()
        );

        let count_row = client.query_one(&count_sql, &param_refs).await?;
        let total: i64 = count_row.get(0);

        Ok(SearchResults::new(
            results,
            total,
            query.pagination.page,
            query.pagination.per_page,
            query.query,
        ))
    }

    /// Highlight search terms in text
    ///
    /// Wraps matching terms with <mark> tags for frontend display
    pub fn highlight_terms(&self, text: &str, search_query: &str) -> String {
        let terms: Vec<&str> = search_query.split_whitespace().collect();
        let mut highlighted = text.to_string();

        // Combine terms into a single regex to avoid loop compilation
        if !terms.is_empty() {
            let escaped_terms: Vec<String> = terms.iter().map(|t| regex::escape(t)).collect();
            let pattern_str = format!("(?i)({})", escaped_terms.join("|"));
            let pattern =
                regex::Regex::new(&pattern_str).unwrap_or_else(|_| regex::Regex::new("").unwrap());

            highlighted = pattern
                .replace_all(&highlighted, "<mark>$1</mark>")
                .to_string();
        }

        highlighted
    }

    /// Get search suggestions based on partial query
    ///
    /// Returns the most relevant distinct campaign names, best match first.
    ///
    /// The previous form was `SELECT DISTINCT nama … ORDER BY similarity(nama,
    /// $2)`, which Postgres rejects outright: "for SELECT DISTINCT, ORDER BY
    /// expressions must appear in select list". It is a parse-time error, so
    /// the endpoint answered 500 to every caller in every environment from the
    /// day it was written — no input reached it, and no row was needed to
    /// trigger it. `GROUP BY nama` gives the same distinct set while making the
    /// ordering expression legal, since it is functionally dependent on the
    /// group key. `nama ASC` breaks ties so repeated calls agree.
    pub async fn get_suggestions(
        &self,
        partial_query: &str,
        limit: i32,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let client = self.db_pool.get().await?;

        let sql = r#"
            SELECT nama
            FROM perlengkapan.pengajuan_kebutuhan_bmn
            WHERE nama ILIKE $1
            GROUP BY nama
            ORDER BY similarity(nama, $2) DESC, nama ASC
            LIMIT $3
        "#;

        let pattern = format!("%{}%", partial_query);
        // LIMIT is int8. Binding the i32 straight through is the second fault
        // in this one query, and it could never surface while the first one
        // stopped the statement at parse time.
        let limit = i64::from(limit);
        let rows = client
            .query(sql, &[&pattern, &partial_query, &limit])
            .await?;

        let suggestions: Vec<String> = rows.iter().map(|row| row.get("nama")).collect();

        Ok(suggestions)
    }
}
