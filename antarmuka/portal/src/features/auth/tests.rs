#[cfg(test)]
mod tests {
    // NB: this file is itself included as `auth::tests`, and it opens a second
    // `mod tests` — so `super::*` here resolves to the (empty) outer module, not
    // to `auth`. Import the items under test by absolute path instead.
    use crate::features::auth::{AuthService, UserSession};
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

        assert!(claims.mfa_enabled);
        assert!(!claims.mfa_setup_required);
        assert_eq!(claims.exp, Some(1700000000));
    }

    #[test]
    fn test_claims_deserialization_defaults() {
        // Test missing fields default to false/None
        let json_data = json!({
            "sub": "user123"
        });

        let claims: Claims = serde_json::from_value(json_data).expect("Failed to deserialize");

        assert!(!claims.mfa_enabled);
        assert!(!claims.mfa_setup_required);
        assert_eq!(claims.exp, None);
    }

    #[test]
    fn test_claims_deserialization_mfa_setup_required() {
        let json_data = json!({
            "sub": "user123",
            "mfa_setup_required": true
        });

        let claims: Claims = serde_json::from_value(json_data).expect("Failed to deserialize");

        assert!(claims.mfa_setup_required);
        // mfa_enabled should default to false
        assert!(!claims.mfa_enabled);
    }

    #[test]
    fn test_jwt_claims_parsing() {
        use crate::features::auth::AuthService;
        use base64::{Engine as _, engine::general_purpose};

        // Create a dummy JWT
        // Header: {"alg":"HS256","typ":"JWT"} -> eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9
        let header = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";

        // Payload: {"sub":"123","preferred_username":"testuser","name":"Test User","email":"test@example.com","realm_access":{"roles":["user"]},"exp":1704067200,"iat":1704060000,"iss":"https://auth.example.com"}
        let payload_json = r#"{"sub":"123","preferred_username":"testuser","name":"Test User","email":"test@example.com","realm_access":{"roles":["user"]},"exp":1704067200,"iat":1704060000,"iss":"https://auth.example.com"}"#;
        let payload = general_purpose::URL_SAFE_NO_PAD.encode(payload_json);

        // Signature (dummy)
        let signature = "signature";

        let token = format!("{}.{}.{}", header, payload, signature);

        // Test parsing with decode_jwt_claims
        let session = AuthService::decode_jwt_claims(&token).expect("Failed to decode JWT claims");
        assert_eq!(session.username, "testuser");
        assert_eq!(session.expires_at, Some(1704067200));
    }

    /// Build a `UserSession` with only the fields the policy predicates read.
    fn session_with(
        expires_at: Option<i64>,
        password_change: bool,
        mfa_setup: bool,
    ) -> UserSession {
        UserSession {
            id: "u1".into(),
            username: "u1".into(),
            role: crate::features::auth::UserRole::User,
            name: "User One".into(),
            email: "u1@kejaksaan.go.id".into(),
            avatar: None,
            nip: Some("1".into()),
            jabatan: None,
            satker_code: None,
            satuan_kerja: String::new(),
            captcha_validated: true,
            mfa_enabled: false,
            mfa_setup_required: mfa_setup,
            require_password_change: password_change,
            created_at: None,
            access_token: Some("t".into()),
            refresh_token: None,
            expires_at,
            roles: Vec::new(),
            assigned_roles: Vec::new(),
            active_role: None,
        }
    }

    /// A missing expiry must NOT read as "valid forever".
    ///
    /// `expires_at` is `Option` and `#[serde(default)]`, so `None` is reachable
    /// from `localStorage` alone — an older build's blob, or a hand-edited one.
    /// The old implementation returned `true` for that case, which turned an
    /// absent field into an unbounded session. The negative assertion is the
    /// one that matters: `assert!(is_session_valid(None))` is exactly the bug.
    #[test]
    fn session_without_expiry_is_not_valid() {
        let no_expiry = session_with(None, false, false);
        assert!(
            !AuthService::is_session_valid(&no_expiry),
            "a session with no expiry must fail closed, not be trusted forever"
        );
        // And an unknown expiry must trigger a refresh attempt rather than be
        // silently accepted.
        assert!(AuthService::should_refresh_token(&no_expiry));

        let future = session_with(Some(chrono::Utc::now().timestamp() + 3600), false, false);
        assert!(AuthService::is_session_valid(&future));
        assert!(!AuthService::should_refresh_token(&future));

        let past = session_with(Some(chrono::Utc::now().timestamp() - 1), false, false);
        assert!(!AuthService::is_session_valid(&past));
    }

    /// Both account-policy flags must block the v1 handoff, and the step must
    /// name the route that clears it.
    #[test]
    fn policy_step_blocks_v1_handoff() {
        use crate::features::auth::legacy_handoff::{PolicyStep, blocking_policy_step};

        assert_eq!(
            blocking_policy_step(&session_with(None, false, false)),
            None
        );

        assert_eq!(
            blocking_policy_step(&session_with(None, true, false)),
            Some(PolicyStep::PasswordChange)
        );
        assert_eq!(PolicyStep::PasswordChange.route(), "/password");

        // MFA setup also blocks; it is reachable only once authenc starts
        // emitting the claim, so the branch is asserted rather than assumed.
        assert_eq!(
            blocking_policy_step(&session_with(None, false, true)),
            Some(PolicyStep::MfaSetup)
        );
        assert_eq!(PolicyStep::MfaSetup.route(), "/mfa/setup");

        // Password change wins when both are set — it is checked first and
        // either route would clear its own flag, so the order must be stable.
        assert_eq!(
            blocking_policy_step(&session_with(None, true, true)),
            Some(PolicyStep::PasswordChange)
        );
    }

    /// The token goes to v1's callback and nowhere else on the origin: the
    /// prefix must end on a path-segment boundary. Both directions, so a check
    /// that refused everything would fail too.
    #[test]
    fn v1_handoff_target_must_be_v1_on_a_segment_boundary() {
        use crate::features::auth::legacy_handoff::is_v1_path;

        for ok in [
            "/perlengkapan/simpel/v1",
            "/perlengkapan/simpel/v1/",
            "/perlengkapan/simpel/v1/auth/oauth-callback",
        ] {
            assert!(is_v1_path(ok), "{ok} is v1");
        }
        for not_v1 in [
            "/perlengkapan/simpel/v1x/auth/oauth-callback",
            "/perlengkapan/simpel/v10/auth/oauth-callback",
            "/perlengkapan/simpel/v1.evil",
            "/perlengkapan/simpel/v2/auth/oauth-callback",
            "/portal/login",
            "/",
        ] {
            assert!(!is_v1_path(not_v1), "{not_v1} is not v1");
        }
    }
}
