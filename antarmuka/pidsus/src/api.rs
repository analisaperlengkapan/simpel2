use gloo_net::http::Request;
use leptos::prelude::*;
use crate::types::{SpecialCase, PidsusStatistics};

const API_BASE_URL: &str = "/pidsus/api/v1/pidsus"; // Via Nginx proxy

pub async fn fetch_dashboard_stats() -> Result<PidsusStatistics, String> {
    let url = format!("{}/dashboard/stats", API_BASE_URL);

    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<PidsusStatistics>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn fetch_recent_cases() -> Result<Vec<SpecialCase>, String> {
    let url = format!("{}/cases", API_BASE_URL);

    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<SpecialCase>>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn fetch_case_detail(id: String) -> Result<SpecialCase, String> {
    let url = format!("{}/cases/{}", API_BASE_URL, id);

    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<SpecialCase>()
        .await
        .map_err(|e| e.to_string())
}
