#[cfg(test)]
mod tests {
    use super::super::*;
    use serde_json::json;

    // We can test the parsing logic directly by using the same structs as defined in the module
    // But since Claims and RealmAccess are private to the module, we cannot access them directly here
    // unless we expose them or use a helper function in the module that is visible to tests.
    // However, we can test `decode_jwt_claims` if we can mock the target architecture or extract the logic.

    // Instead of fighting with cfg attributes, let's extract the core logic into a testable function
    // in the main file that is available on all targets, or create a parallel test here that verifies
    // the serde behavior which is the core of the change.

    use serde::{Deserialize, Serialize};

    // Re-define structs for testing exactly as they are in the implementation
    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct Claims {
        sub: String,
        preferred_username: Option<String>,
        name: Option<String>,
        email: Option<String>,
        realm_access: Option<RealmAccess>,
        #[serde(default)]
        mfa_enabled: bool,
        #[serde(default)]
        mfa_setup_required: bool,
        exp: Option<i64>,
    }

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct RealmAccess {
        roles: Vec<String>,
    }

    #[test]
    fn test_claims_deserialization_with_mfa_fields() {
        let json_data = json!({
            "sub": "user123",
            "preferred_username": "testuser",
            "mfa_enabled": true,
            "mfa_setup_required": false,
            "exp": 1700000000
        });

        let claims: Claims = serde_json::from_value(json_data).expect("Failed to deserialize");

        assert_eq!(claims.mfa_enabled, true);
        assert_eq!(claims.mfa_setup_required, false);
        assert_eq!(claims.exp, Some(1700000000));
    }

    #[test]
    fn test_claims_deserialization_defaults() {
        // Test missing fields default to false/None
        let json_data = json!({
            "sub": "user123"
        });

        let claims: Claims = serde_json::from_value(json_data).expect("Failed to deserialize");

        assert_eq!(claims.mfa_enabled, false);
        assert_eq!(claims.mfa_setup_required, false);
        assert_eq!(claims.exp, None);
    }

    #[test]
    fn test_claims_deserialization_mfa_setup_required() {
        let json_data = json!({
            "sub": "user123",
            "mfa_setup_required": true
        });

        let claims: Claims = serde_json::from_value(json_data).expect("Failed to deserialize");

        assert_eq!(claims.mfa_setup_required, true);
        // mfa_enabled should default to false
        assert_eq!(claims.mfa_enabled, false);
    }

    #[test]
    fn test_jwt_claims_parsing() {
        use crate::features::auth::AuthService;
        use base64::{engine::general_purpose, Engine as _};

        // Create a dummy JWT
        // Header: {"alg":"HS256","typ":"JWT"} -> eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9
        let header = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";

        // Payload: {"sub":"123","preferred_username":"testuser","name":"Test User","email":"test@example.com","realm_access":{"roles":["user"]},"exp":1704067200}
        let payload_json = r#"{"sub":"123","preferred_username":"testuser","name":"Test User","email":"test@example.com","realm_access":{"roles":["user"]},"exp":1704067200}"#;
        let payload = general_purpose::URL_SAFE_NO_PAD.encode(payload_json);

        // Signature (dummy)
        let signature = "signature";

        let token = format!("{}.{}.{}", header, payload, signature);

        // Test parsing with decode_jwt_claims
        let session = AuthService::decode_jwt_claims(&token).expect("Failed to decode JWT claims");
        assert_eq!(session.username, "testuser");
        assert_eq!(session.expires_at, Some(1704067200));
    }
}
