use crate::types::{CreateRencanaRequest, RencanaPengadaan, UpdateRencanaRequest};
use gloo_net::http::Request;
use leptos::*;
use uuid::Uuid;

const API_BASE_URL: &str = "http://localhost:8080/api/v1/perencanaan";

pub async fn get_rencana_list() -> Result<Vec<RencanaPengadaan>, String> {
    Request::get(&format!("{}/pengadaan", API_BASE_URL))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<RencanaPengadaan>>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn create_rencana(req: CreateRencanaRequest) -> Result<RencanaPengadaan, String> {
    Request::post(&format!("{}/pengadaan", API_BASE_URL))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<RencanaPengadaan>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn update_rencana(id: Uuid, req: UpdateRencanaRequest) -> Result<RencanaPengadaan, String> {
    Request::put(&format!("{}/pengadaan/{}", API_BASE_URL, id))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<RencanaPengadaan>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn delete_rencana(id: Uuid) -> Result<(), String> {
    let resp = Request::delete(&format!("{}/pengadaan/{}", API_BASE_URL, id))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status() == 204 {
        Ok(())
    } else {
        Err(format!("Failed to delete: {}", resp.status()))
    }
}
