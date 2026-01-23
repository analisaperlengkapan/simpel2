use crate::types::MilitaryCase;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

// This URL should be configurable, but for now we point to the backend service
pub const API_URL: &str = "http://localhost:8086/api/v1/pidmil";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateCasePayload {
    pub title: String,
    pub case_type: String,
    pub description: String,
    pub priority: String,
    pub assigned_investigator: String,
    pub unit_involved: String,
    pub location: String,
}

pub async fn fetch_cases(page: Option<u32>, limit: Option<u32>) -> Result<Vec<MilitaryCase>, String> {
    let mut url = format!("{}/cases", API_URL);
    let mut params = Vec::new();

    if let Some(p) = page {
        params.push(format!("page={}", p));
    }
    if let Some(l) = limit {
        params.push(format!("limit={}", l));
    }

    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

pub async fn fetch_case_by_id(id: &str) -> Result<MilitaryCase, String> {
    Request::get(&format!("{}/cases/{}", API_URL, id))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

pub async fn create_case(payload: CreateCasePayload) -> Result<MilitaryCase, String> {
    Request::post(&format!("{}/cases", API_URL))
        .json(&payload)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}
