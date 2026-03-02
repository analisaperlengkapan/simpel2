import re

filepath = "layanan/authenc/crates/iam-api/src/handlers/users.rs"
with open(filepath, "r") as f:
    content = f.read()

# Update enable_user_mfa to return better mock data or generate a real TOTP secret if possible
# Since we don't have mfa_service configured in state.rs yet, we can generate a simple one.

old_code = """    let _user = state.user_service.enable_mfa(UserId::from_uuid(id)).await.map_err(crate::error::ApiError)?;
    // the user object returned has mfa_enabled = true now.
    // MOCK details:
    Ok(Json(EnableMfaResponse {
        secret: "mock_secret".to_string(),
        qr_code: "mock_qr".to_string(),
    }))"""

new_code = """    let user = state.user_service.enable_mfa(UserId::from_uuid(id)).await.map_err(crate::error::ApiError)?;

    // Generate an actual TOTP secret using totp-rs
    #[cfg(feature = "totp-rs")]
    {
        use totp_rs::{Algorithm, Secret, TOTP};
        let secret = Secret::generate_secret().to_bytes().unwrap();
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret,
            Some("SIMPEL Kejaksaan RI".to_string()),
            user.username.clone(),
        ).unwrap();

        let qr_code = totp.get_qr_base64().unwrap_or_default();
        let secret_string = totp.get_secret_base32();

        Ok(Json(EnableMfaResponse {
            secret: secret_string,
            qr_code,
        }))
    }
    #[cfg(not(feature = "totp-rs"))]
    {
        Ok(Json(EnableMfaResponse {
            secret: "totp-rs-feature-disabled".to_string(),
            qr_code: "".to_string(),
        }))
    }"""

content = content.replace(old_code, new_code)

with open(filepath, "w") as f:
    f.write(content)
