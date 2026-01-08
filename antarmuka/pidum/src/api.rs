use crate::types::*;
use gloo_net::http::Request;
use leptos::prelude::*;
use uuid::Uuid;

const API_BASE_URL: &str = "/api/v1/pidum";

pub async fn fetch_perkara_list() -> Result<Vec<Perkara>, String> {
    Request::get(&format!("{}/perkara", API_BASE_URL))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

pub async fn create_perkara(req: CreatePerkaraRequest) -> Result<Perkara, String> {
    Request::post(&format!("{}/perkara", API_BASE_URL))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

pub async fn fetch_perkara_detail(id: Uuid) -> Result<Perkara, String> {
    Request::get(&format!("{}/perkara/{}", API_BASE_URL, id))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}
