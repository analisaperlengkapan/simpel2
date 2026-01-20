use crate::types::{MilitaryCase, MilitarySuspect};
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

pub async fn fetch_cases() -> Result<Vec<MilitaryCase>, String> {
    Request::get(&format!("{}/cases", API_URL))
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
