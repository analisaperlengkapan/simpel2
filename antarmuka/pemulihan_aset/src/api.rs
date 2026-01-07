use gloo::net::http::Request;
use crate::types::*;
use uuid::Uuid;

const API_BASE_URL: &str = "/api/v1/pemulihan_aset";

pub async fn fetch_cases() -> Result<Vec<Case>, String> {
    let resp = Request::get(&format!("{}/cases", API_BASE_URL))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("Failed to fetch cases: {}", resp.status()));
    }

    resp.json().await.map_err(|e| e.to_string())
}

pub async fn create_case(req: CreateCaseRequest) -> Result<Case, String> {
    let resp = Request::post(&format!("{}/cases", API_BASE_URL))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("Failed to create case: {}", resp.status()));
    }

    resp.json().await.map_err(|e| e.to_string())
}

pub async fn fetch_assets(case_id: Uuid) -> Result<Vec<Asset>, String> {
    let resp = Request::get(&format!("{}/cases/{}/assets", API_BASE_URL, case_id))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("Failed to fetch assets: {}", resp.status()));
    }

    resp.json().await.map_err(|e| e.to_string())
}

pub async fn create_asset(req: CreateAssetRequest) -> Result<Asset, String> {
    let resp = Request::post(&format!("{}/assets", API_BASE_URL))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("Failed to create asset: {}", resp.status()));
    }

    resp.json().await.map_err(|e| e.to_string())
}
