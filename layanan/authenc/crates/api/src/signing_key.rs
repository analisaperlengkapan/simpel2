//! Start-up resolution of the JWT signing key.
//!
//! Every authenc replica must sign and verify with the SAME key. With no
//! `JWT_SECRET` each process draws its own random key, which is fine for one
//! developer process and wrong everywhere else:
//!
//! * with two replicas (the staging HPA runs up to two, production runs more) a
//!   token minted by replica A is rejected by replica B, so a request is accepted
//!   or refused depending on which pod happened to answer;
//! * every restart or rollout invalidates every session at once.
//!
//! rc40 on staging showed exactly that: the HPA scaled authenc to two and the e2e
//! run failed with `Token validation failed` on tokens minted seconds earlier. The
//! chart never supplied the variable, and authenc only logged "not for production"
//! and carried on. So staging and production now refuse to start without it.

use authenc_crypto::JwtService;

/// Environments that must configure a persistent key. Anything else (unset,
/// `development`, `test`, …) may fall back to an ephemeral one.
fn requires_persistent_key(environment: &str) -> bool {
    let environment = environment.trim();
    environment.eq_ignore_ascii_case("production") || environment.eq_ignore_ascii_case("staging")
}

/// Decode `secret_hex` (64 hex chars = 32 bytes), or — only outside staging and
/// production — draw an ephemeral key.
pub fn resolve(environment: &str, secret_hex: &str) -> Result<[u8; 32], String> {
    let secret_hex = secret_hex.trim();

    if secret_hex.is_empty() {
        if requires_persistent_key(environment) {
            return Err(format!(
                "JWT_SECRET is required when APP_ENVIRONMENT={environment}: without it every \
                 authenc replica signs with its own random key, so a token minted by one \
                 replica is rejected by the others and every restart logs everyone out. \
                 Set it to 64 hex characters (`openssl rand -hex 32`); the Helm chart supplies \
                 it from Secret `authenc-jwt`."
            ));
        }
        tracing::warn!(
            "JWT_SECRET not set - generating an ephemeral signing key (development only; \
             tokens do not survive a restart and are not valid on another replica)"
        );
        return Ok(JwtService::generate_signing_key());
    }

    let bytes = hex::decode(secret_hex).map_err(|e| format!("Invalid JWT_SECRET hex: {e}"))?;
    <[u8; 32]>::try_from(bytes.as_slice())
        .map_err(|_| "JWT_SECRET must be 32 bytes (64 hex chars)".to_string())
}

#[cfg(test)]
mod tests {
    use super::resolve;

    const KEY: &str = "1111111111111111111111111111111111111111111111111111111111111111";

    #[test]
    fn a_configured_key_is_used_verbatim() {
        assert_eq!(resolve("production", KEY).unwrap(), [0x11; 32]);
        assert_eq!(resolve("staging", KEY).unwrap(), [0x11; 32]);
        assert_eq!(resolve("", KEY).unwrap(), [0x11; 32]);
    }

    #[test]
    fn two_replicas_with_the_same_secret_hold_the_same_key() {
        // The property the cluster depends on: same Secret in, same key out.
        assert_eq!(
            resolve("production", KEY).unwrap(),
            resolve("production", KEY).unwrap()
        );
    }

    #[test]
    fn surrounding_whitespace_from_a_mounted_secret_is_ignored() {
        assert_eq!(
            resolve("production", &format!("{KEY}\n")).unwrap(),
            [0x11; 32]
        );
    }

    #[test]
    fn production_and_staging_refuse_to_start_without_a_key() {
        for env in ["production", "PRODUCTION", "staging", " Staging "] {
            let err = resolve(env, "").unwrap_err();
            assert!(err.contains("JWT_SECRET is required"), "{env}: {err}");
            // A blank/whitespace-only value is "not set" too.
            assert!(resolve(env, "  \n").is_err(), "{env}: blank accepted");
        }
    }

    #[test]
    fn development_may_fall_back_to_an_ephemeral_key_that_differs_per_process() {
        for env in ["", "development", "test"] {
            let a = resolve(env, "").unwrap();
            let b = resolve(env, "").unwrap();
            assert_ne!(a, b, "{env}: ephemeral keys must not repeat");
        }
    }

    #[test]
    fn a_malformed_key_is_rejected_in_every_environment() {
        for env in ["production", "development"] {
            assert!(resolve(env, "not-hex").unwrap_err().contains("hex"));
            assert!(resolve(env, "abcd").unwrap_err().contains("32 bytes"));
            // 31 and 33 bytes.
            assert!(resolve(env, &"11".repeat(31)).is_err());
            assert!(resolve(env, &"11".repeat(33)).is_err());
        }
    }
}
