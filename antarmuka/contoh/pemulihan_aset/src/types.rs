use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Case {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String, // ISO string for frontend
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreateCaseRequest {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub case_id: Uuid,
    pub name: String,
    pub asset_type: String,
    pub estimated_value: Option<f64>,
    pub status: String,
    pub location: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreateAssetRequest {
    pub case_id: Uuid,
    pub name: String,
    pub asset_type: String,
    pub estimated_value: Option<f64>,
    pub location: Option<String>,
}
