use crate::types::DatunCase;
use gloo_net::http::Request;

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
