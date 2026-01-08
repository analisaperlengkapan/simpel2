use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use crate::app::{IntelOperation, CreateOperationRequest, IntelReport};

// Best practice: Use relative path for production compatibility behind a proxy,
// or use a configurable base URL. For this dev setup, we'll use the direct URL
// but structure it to be easily swappable.
#[cfg(debug_assertions)]
const API_BASE_URL: &str = "http://localhost:8080/api/v1/intel";

#[cfg(not(debug_assertions))]
const API_BASE_URL: &str = "/api/v1/intel";

pub async fn fetch_operations() -> Result<Vec<IntelOperation>, String> {
    Request::get(&format!("{}/operations", API_BASE_URL))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<IntelOperation>>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn fetch_operation_by_id(id: &str) -> Result<IntelOperation, String> {
    Request::get(&format!("{}/operations/{}", API_BASE_URL, id))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<IntelOperation>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn create_operation(req: &CreateOperationRequest) -> Result<IntelOperation, String> {
    Request::post(&format!("{}/operations", API_BASE_URL))
        .json(req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<IntelOperation>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn fetch_reports() -> Result<Vec<IntelReport>, String> {
    Request::get(&format!("{}/reports", API_BASE_URL))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<IntelReport>>()
        .await
        .map_err(|e| e.to_string())
}
