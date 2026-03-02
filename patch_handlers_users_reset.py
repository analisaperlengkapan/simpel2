import re

filepath = "layanan/authenc/crates/iam-api/src/handlers/users.rs"
with open(filepath, "r") as f:
    content = f.read()

old_code = """pub async fn reset_user_password(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
    Json(_req): Json<ResetPasswordRequest>,
) -> ApiResult<StatusCode> {
    // We need credential management service or similar to do this
    Err(crate::error::ApiError(AuthencError::NotImplemented(
        "reset_password not yet implemented via user_service".to_string(),
    )))
}"""

new_code = """pub async fn reset_user_password(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ResetPasswordRequest>,
) -> ApiResult<StatusCode> {
    // Using UpdateUserRequest as a proxy to reset the password through user_service
    let core_req = CoreUpdateUserRequest {
        username: None,
        email: None,
        first_name: None,
        last_name: None,
        phone_number: None,
        nip: None,
        nama: None,
        jabatan: None,
        satker_code: None,
        attributes: None,
        enabled: None,
        email_verified: None,
        phone_verified: None,
        require_password_change: None,
        password: Some(req.new_password),
        mfa_enabled: None,
    };

    state.user_service.update_user(UserId::from_uuid(id), core_req).await.map_err(crate::error::ApiError)?;

    Ok(StatusCode::NO_CONTENT)
}"""

content = content.replace(old_code, new_code)

with open(filepath, "w") as f:
    f.write(content)
