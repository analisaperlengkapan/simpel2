use super::params::*;
use axum::{
    Json,
    extract::{Query, State},
};

use crate::shared::error::{AppError, bad_request};
use crate::shared::middleware::Claims;
use lib_perlengkapan::response::{ApiResponse, PaginatedResponse};

use crate::kebutuhan_bmn::models::*;
use crate::kebutuhan_bmn::services::KebutuhanBmnService;

/// GET /kebutuhan-bmn/search
/// Advanced full-text search with filters and pagination
pub async fn search_kebutuhan(
    State(service): State<KebutuhanBmnService>,
    Query(params): Query<SearchQueryParams>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<KebutuhanBmnSummary>>, AppError> {
    use lib_perlengkapan::search::{
        Pagination, SearchFilters, SearchQuery, SortDirection, SortField, SortOptions,
    };

    // Validate search query
    if params.q.trim().is_empty() {
        return Err(bad_request("Search query cannot be empty"));
    }
    if params.q.len() < 2 {
        return Err(bad_request("Search query must be at least 2 characters"));
    }

    // Validate pagination
    let pagination_check = PaginationQuery {
        page: params.page,
        per_page: params.per_page,
    };
    pagination_check.validate()?;

    // Parse status list
    let status_list = params.status.as_ref().map(|s| {
        s.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>()
    });

    // Build search query
    let search_query = SearchQuery {
        query: params.q.clone(),
        filters: SearchFilters {
            satker_id: params.satker_id,
            tahun_anggaran: params.tahun_anggaran,
            status: status_list,
            kode_barang: params.kode_barang.clone(),
            priority_level: None,
            date_from: params.date_from.clone(),
            date_to: params.date_to.clone(),
        },
        pagination: Pagination {
            page: params.page,
            per_page: params.per_page,
        },
        sort: SortOptions {
            field: SortField::parse_str(&params.sort_by).unwrap_or(SortField::Relevance),
            direction: SortDirection::parse_str(&params.sort_dir)
                .unwrap_or(SortDirection::Descending),
        },
    };

    // Execute search
    let results = service.search_kebutuhan(search_query).await?;

    Ok(Json(PaginatedResponse::new(
        results.results.into_iter().map(|r| r.item).collect(),
        results.total,
        results.page,
        results.per_page,
        format!("Found {} results for '{}'", results.total, params.q),
    )))
}
pub async fn get_search_suggestions(
    State(service): State<KebutuhanBmnService>,
    Query(params): Query<SuggestionsQuery>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<String>>>, AppError> {
    if params.q.len() < 2 {
        return Err(bad_request("Query must be at least 2 characters"));
    }

    let suggestions = service
        .get_search_suggestions(&params.q, params.limit)
        .await?;

    Ok(Json(ApiResponse::success(
        suggestions,
        "Suggestions retrieved successfully".to_string(),
    )))
}
