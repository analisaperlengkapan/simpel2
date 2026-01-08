use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use gloo_net::http::Request;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SupervisionSchedule {
    pub id: Uuid,
    pub title: String,
    pub scheduled_at: DateTime<Utc>,
    pub location: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduleRequest {
    pub title: String,
    pub scheduled_at: DateTime<Utc>,
    pub location: String,
}

pub async fn fetch_schedules() -> Result<Vec<SupervisionSchedule>, String> {
    Request::get("http://localhost:3004/api/v1/pengawasan/schedules")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

pub async fn create_schedule(req: CreateScheduleRequest) -> Result<SupervisionSchedule, String> {
    Request::post("http://localhost:3004/api/v1/pengawasan/schedules")
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}
