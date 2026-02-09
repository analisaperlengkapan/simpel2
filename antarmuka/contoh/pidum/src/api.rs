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

pub async fn fetch_timeline(id: Uuid) -> Result<Vec<PerkaraTimeline>, String> {
    Request::get(&format!("{}/perkara/{}/timeline", API_BASE_URL, id))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

pub async fn create_comment(id: Uuid, req: CreateCommentRequest) -> Result<PerkaraComment, String> {
    let resp = Request::post(&format!("{}/perkara/{}/comments", API_BASE_URL, id))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Error {}: {}", status, text));
    }

    resp.json().await.map_err(|e| e.to_string())
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
