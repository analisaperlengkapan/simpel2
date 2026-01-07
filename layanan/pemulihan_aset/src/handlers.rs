use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use garde::Validate;
use crate::db::DB;
use crate::model::{Case, Asset, CreateCaseRequest, CreateAssetRequest};
use serde_json::{json, Value};

pub async fn list_cases(State(db): State<DB>) -> Result<Json<Vec<Case>>, (StatusCode, Json<Value>)> {
    match db.list_cases().await {
        Ok(cases) => Ok(Json(cases)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn create_case(
    State(db): State<DB>,
    Json(req): Json<CreateCaseRequest>,
) -> Result<Json<Case>, (StatusCode, Json<Value>)> {
    if let Err(e) = req.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ));
    }

    match db.create_case(req).await {
        Ok(case) => Ok(Json(case)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn list_assets(
    State(db): State<DB>,
    Path(case_id): Path<Uuid>,
) -> Result<Json<Vec<Asset>>, (StatusCode, Json<Value>)> {
    match db.list_assets(case_id).await {
        Ok(assets) => Ok(Json(assets)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn create_asset(
    State(db): State<DB>,
    Json(req): Json<CreateAssetRequest>,
) -> Result<Json<Asset>, (StatusCode, Json<Value>)> {
    if let Err(e) = req.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ));
    }

    match db.create_asset(req).await {
        Ok(asset) => Ok(Json(asset)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}
