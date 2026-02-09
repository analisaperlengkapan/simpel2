use crate::types::{
    ApiErrorResponse, CreateRencanaRequest, RencanaPengadaan, UpdateRencanaRequest,
};
use gloo_net::http::Request;
use uuid::Uuid;

const API_BASE_URL: &str = "http://localhost:8080/api/v1/perencanaan";

async fn handle_response<T>(resp: gloo_net::http::Response) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    if resp.ok() {
        resp.json::<T>().await.map_err(|e| e.to_string())
    } else {
        match resp.json::<ApiErrorResponse>().await {
            Ok(err_resp) => Err(err_resp.message),
            Err(_) => Err(format!("Request failed with status: {}", resp.status())),
        }
    }
}

pub async fn get_rencana_list() -> Result<Vec<RencanaPengadaan>, String> {
    let resp = Request::get(&format!("{}/pengadaan", API_BASE_URL))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    handle_response(resp).await
}

pub async fn create_rencana(req: CreateRencanaRequest) -> Result<RencanaPengadaan, String> {
    let resp = Request::post(&format!("{}/pengadaan", API_BASE_URL))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    handle_response(resp).await
}

pub async fn update_rencana(
    id: Uuid,
    req: UpdateRencanaRequest,
) -> Result<RencanaPengadaan, String> {
    let resp = Request::put(&format!("{}/pengadaan/{}", API_BASE_URL, id))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    handle_response(resp).await
}

pub async fn delete_rencana(id: Uuid) -> Result<(), String> {
    let resp = Request::delete(&format!("{}/pengadaan/{}", API_BASE_URL, id))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        match resp.json::<ApiErrorResponse>().await {
            Ok(err_resp) => Err(err_resp.message),
            Err(_) => Err(format!("Failed to delete: {}", resp.status())),
        }
    }
}
