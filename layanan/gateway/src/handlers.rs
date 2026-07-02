//! REST handlers + the pure response-mapping helpers they delegate to.
//!
//! The mapping helpers (`map_validate`, `map_get_secret`, …) are deliberately
//! free of any I/O so they can be unit-tested without a live gRPC server; the
//! real round-trip is covered by the `e2e-simpelv1-integration` CI job.

use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};

use crate::AppState;
use crate::proto::authenc::v1::{GetUserRequest, ValidateTokenRequest, ValidateTokenResponse};
use crate::proto::integrasi::v1::{
    GetMysimkariPegawaiRequest, GetSimanAssetsRequest, Pagination, SimanAssetCategory,
};
use crate::proto::secreton::v1::{
    DeleteSecretRequest, GenerateDatabaseCredentialsRequest, GetSecretRequest, GetSecretResponse,
    StoreSecretRequest,
};

// ── pure mapping helpers (unit-tested) ────────────────────────────────────

/// Map gRPC status codes onto the HTTP semantics the PHP clients expect:
/// definitive auth failures → 4xx (fail-closed); transport/unavailable → 5xx
/// (indeterminate → the PHP `isTokenActive` fails *open* on 5xx).
pub fn upstream_status(status: &tonic::Status) -> StatusCode {
    match status.code() {
        tonic::Code::Unauthenticated | tonic::Code::PermissionDenied => StatusCode::UNAUTHORIZED,
        tonic::Code::NotFound => StatusCode::NOT_FOUND,
        tonic::Code::InvalidArgument => StatusCode::BAD_REQUEST,
        _ => StatusCode::BAD_GATEWAY,
    }
}

fn upstream_error(service: &str, status: tonic::Status) -> Response {
    let code = upstream_status(&status);
    tracing::warn!(%service, grpc_code = ?status.code(), msg = status.message(), "upstream gRPC error");
    (
        code,
        Json(json!({ "error": "upstream_error", "service": service, "detail": status.message() })),
    )
        .into_response()
}

/// `ValidateToken` → REST. Valid ⇒ 200 `{claims:{…}}` (sub from user_id); the
/// claims shape matches what the PHP `verifyToken`/revocation middleware read.
/// Invalid ⇒ 401 (definitive reject → middleware fails closed).
pub fn map_validate(resp: ValidateTokenResponse) -> (StatusCode, Value) {
    if resp.valid {
        let claims = json!({
            "sub": resp.user_id,
            "username": resp.username,
            "name": resp.name,
            "nip": resp.nip,
            "jabatan": resp.jabatan,
            "satker_code": resp.satker_code,
            "roles": resp.realm_roles,
            "scopes": resp.scopes,
            "exp": resp.expires_at,
        });
        (StatusCode::OK, json!({ "claims": claims }))
    } else {
        (
            StatusCode::UNAUTHORIZED,
            json!({ "error": resp.error.unwrap_or_else(|| "invalid_token".to_string()) }),
        )
    }
}

/// `GetSecret` → REST `{value, data}`. The PHP `getSecret` reads `value`; the
/// full `data` map is included for callers that store structured secrets.
pub fn map_get_secret(resp: GetSecretResponse) -> Value {
    let value = resp.data.get("value").cloned();
    json!({ "value": value, "data": resp.data })
}

/// `GetSecret` (api-keys path) → REST `{api_key}` (PHP reads `api_key`).
pub fn map_api_key(resp: GetSecretResponse) -> Value {
    let api_key = resp
        .data
        .get("api_key")
        .or_else(|| resp.data.get("value"))
        .cloned();
    json!({ "api_key": api_key })
}

/// True if a serialized SIMAN asset matches `id` on any of its identifier-ish
/// fields (the proto field name varies; match defensively).
fn value_id_matches(v: &Value, id: &str) -> bool {
    const KEYS: [&str; 6] = ["id", "nup", "kode_barang", "kode", "asset_id", "kode_aset"];
    KEYS.iter().any(|k| {
        v.get(k)
            .map(|x| x.as_str() == Some(id) || x.to_string().trim_matches('"') == id)
            .unwrap_or(false)
    })
}

// ── handlers ───────────────────────────────────────────────────────────────

pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

#[derive(serde::Deserialize)]
pub struct VerifyBody {
    pub token: String,
}

pub async fn verify_token(State(st): State<AppState>, Json(body): Json<VerifyBody>) -> Response {
    let mut client = st.authenc.clone();
    let req = ValidateTokenRequest {
        token: body.token,
        required_scopes: vec![],
    };
    match client.validate_token(tonic::Request::new(req)).await {
        Ok(resp) => {
            let (code, body) = map_validate(resp.into_inner());
            (code, Json(body)).into_response()
        }
        Err(s) => upstream_error("authenc", s),
    }
}

pub async fn get_user(State(st): State<AppState>, Path(id): Path<String>) -> Response {
    let mut client = st.authenc.clone();
    let req = GetUserRequest { user_id: id };
    match client.get_user(tonic::Request::new(req)).await {
        Ok(resp) => match resp.into_inner().user {
            Some(user) => (StatusCode::OK, Json(user)).into_response(),
            None => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "user_not_found" })),
            )
                .into_response(),
        },
        Err(s) => upstream_error("authenc", s),
    }
}

pub async fn get_secret(State(st): State<AppState>, Path(path): Path<String>) -> Response {
    let mut client = st.secreton.clone();
    let req = GetSecretRequest {
        path,
        version: None,
    };
    match client.get_secret(tonic::Request::new(req)).await {
        Ok(resp) => (StatusCode::OK, Json(map_get_secret(resp.into_inner()))).into_response(),
        Err(s) => upstream_error("secreton", s),
    }
}

#[derive(serde::Deserialize)]
pub struct StoreBody {
    pub value: Option<String>,
    pub data: Option<HashMap<String, String>>,
}

pub async fn put_secret(
    State(st): State<AppState>,
    Path(path): Path<String>,
    Json(body): Json<StoreBody>,
) -> Response {
    let mut client = st.secreton.clone();
    let mut data = body.data.unwrap_or_default();
    if let Some(v) = body.value {
        data.insert("value".to_string(), v);
    }
    let req = StoreSecretRequest {
        path,
        data,
        security_level: 0,
        tags: vec![],
        ttl_seconds: None,
    };
    match client.store_secret(tonic::Request::new(req)).await {
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(s) => upstream_error("secreton", s),
    }
}

pub async fn delete_secret(State(st): State<AppState>, Path(path): Path<String>) -> Response {
    let mut client = st.secreton.clone();
    let req = DeleteSecretRequest { path };
    match client.delete_secret(tonic::Request::new(req)).await {
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(s) => upstream_error("secreton", s),
    }
}

pub async fn get_database_credentials(
    State(st): State<AppState>,
    Path(db): Path<String>,
) -> Response {
    let mut client = st.secreton.clone();
    let req = GenerateDatabaseCredentialsRequest {
        role_name: db,
        ttl_seconds: None,
    };
    match client
        .generate_database_credentials(tonic::Request::new(req))
        .await
    {
        Ok(resp) => {
            let inner = resp.into_inner();
            (
                StatusCode::OK,
                Json(json!({
                    "lease_id": inner.lease_id,
                    "lease_duration": inner.lease_duration,
                    "renewable": inner.renewable,
                    "credentials": inner.credentials,
                })),
            )
                .into_response()
        }
        Err(s) => upstream_error("secreton", s),
    }
}

pub async fn get_api_key(State(st): State<AppState>, Path(name): Path<String>) -> Response {
    let mut client = st.secreton.clone();
    let req = GetSecretRequest {
        path: format!("api-keys/{name}"),
        version: None,
    };
    match client.get_secret(tonic::Request::new(req)).await {
        Ok(resp) => (StatusCode::OK, Json(map_api_key(resp.into_inner()))).into_response(),
        Err(s) => upstream_error("secreton", s),
    }
}

fn one_page(per_page: i32) -> Option<Pagination> {
    Some(Pagination {
        page: 1,
        per_page,
        sort_by: String::new(),
        ascending: true,
    })
}

pub async fn get_mysimkari_employee(
    State(st): State<AppState>,
    Path(nip): Path<String>,
) -> Response {
    let mut client = st.integrasi.clone();
    let req = GetMysimkariPegawaiRequest {
        kode_satker: String::new(),
        nama_filter: String::new(),
        nip_filter: nip,
        pagination: one_page(1),
    };
    match client.get_mysimkari_pegawai(tonic::Request::new(req)).await {
        Ok(resp) => match resp.into_inner().items.into_iter().next() {
            Some(emp) => (StatusCode::OK, Json(emp)).into_response(),
            None => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "employee_not_found" })),
            )
                .into_response(),
        },
        Err(s) => upstream_error("integrasi", s),
    }
}

pub async fn get_siman_inventory(State(st): State<AppState>, Path(id): Path<String>) -> Response {
    // SIMAN has no "by id" RPC; scan each category's page and match defensively.
    const CATS: [SimanAssetCategory; 4] = [
        SimanAssetCategory::Tanah,
        SimanAssetCategory::GedungBangunan,
        SimanAssetCategory::AlatBesar,
        SimanAssetCategory::AngkutanBermotor,
    ];
    for cat in CATS {
        let mut client = st.integrasi.clone();
        let req = GetSimanAssetsRequest {
            category: cat as i32,
            kode_satker: String::new(),
            pagination: one_page(500),
        };
        match client.get_siman_assets(tonic::Request::new(req)).await {
            Ok(resp) => {
                for asset in resp.into_inner().items {
                    let v = serde_json::to_value(&asset).unwrap_or(Value::Null);
                    if value_id_matches(&v, &id) {
                        return (StatusCode::OK, Json(v)).into_response();
                    }
                }
            }
            Err(s) => return upstream_error("integrasi", s),
        }
    }
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": "inventory_not_found" })),
    )
        .into_response()
}

/// MonSAKTI is dormant (Kejaksaan migrating to MyIntress; API re-submission in
/// progress — see F-SSOT). Return a clear 501 so the PHP client degrades
/// gracefully (its callers treat non-2xx as null/false) instead of hanging.
pub async fn monsakti_unavailable() -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({ "error": "monsakti_dormant", "detail": "MonSAKTI integration is dormant (migrating to MyIntress)" })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_resp() -> ValidateTokenResponse {
        ValidateTokenResponse {
            valid: true,
            user_id: Some("u1".to_string()),
            scopes: vec!["role:admin".to_string()],
            expires_at: Some(1_900_000_000),
            error: None,
            username: Some("alice".to_string()),
            name: Some("Alice".to_string()),
            nip: Some("123".to_string()),
            jabatan: Some("Operator".to_string()),
            satker_code: Some("0200010".to_string()),
            realm_roles: vec!["operator_satker".to_string()],
        }
    }

    #[test]
    fn valid_token_maps_to_200_with_claims_sub() {
        let (code, body) = map_validate(valid_resp());
        assert_eq!(code, StatusCode::OK);
        assert_eq!(body["claims"]["sub"], "u1");
        assert_eq!(body["claims"]["satker_code"], "0200010");
        assert_eq!(body["claims"]["roles"][0], "operator_satker");
    }

    #[test]
    fn invalid_token_maps_to_401_with_error() {
        let mut resp = valid_resp();
        resp.valid = false;
        resp.error = Some("revoked".to_string());
        let (code, body) = map_validate(resp);
        assert_eq!(code, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error"], "revoked");
    }

    #[test]
    fn invalid_token_without_error_has_default_message() {
        let mut resp = valid_resp();
        resp.valid = false;
        resp.error = None;
        let (_, body) = map_validate(resp);
        assert_eq!(body["error"], "invalid_token");
    }

    fn secret_with(data: &[(&str, &str)]) -> GetSecretResponse {
        GetSecretResponse {
            id: "id".to_string(),
            path: "p".to_string(),
            data: data
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            version: 1,
            security_level: 0,
            created_at: 0,
            updated_at: 0,
        }
    }

    #[test]
    fn get_secret_exposes_value_and_data() {
        let body = map_get_secret(secret_with(&[("value", "s3cr3t"), ("extra", "x")]));
        assert_eq!(body["value"], "s3cr3t");
        assert_eq!(body["data"]["extra"], "x");
    }

    #[test]
    fn get_secret_without_value_key_is_null_value() {
        let body = map_get_secret(secret_with(&[("only", "x")]));
        assert!(body["value"].is_null());
        assert_eq!(body["data"]["only"], "x");
    }

    #[test]
    fn api_key_falls_back_to_value() {
        assert_eq!(
            map_api_key(secret_with(&[("api_key", "AK")]))["api_key"],
            "AK"
        );
        assert_eq!(map_api_key(secret_with(&[("value", "V")]))["api_key"], "V");
    }

    #[test]
    fn grpc_status_maps_to_fail_open_and_fail_closed() {
        // transport/unknown → 5xx (indeterminate → PHP fails open)
        assert_eq!(
            upstream_status(&tonic::Status::unavailable("down")),
            StatusCode::BAD_GATEWAY
        );
        // definitive auth reject → 401 (PHP fails closed)
        assert_eq!(
            upstream_status(&tonic::Status::unauthenticated("nope")),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            upstream_status(&tonic::Status::not_found("x")),
            StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn siman_id_match_is_defensive_across_field_names() {
        assert!(value_id_matches(&json!({ "nup": "A-1" }), "A-1"));
        assert!(value_id_matches(&json!({ "kode_barang": "KB9" }), "KB9"));
        assert!(!value_id_matches(&json!({ "other": "z" }), "A-1"));
    }
}
