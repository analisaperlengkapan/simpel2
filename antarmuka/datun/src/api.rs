use gloo_net::http::Request;
use crate::types::DatunCase;

/// Fetch all cases from the backend
pub async fn fetch_cases() -> Result<Vec<DatunCase>, String> {
    Request::get("/api/v1/datun/cases")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}
