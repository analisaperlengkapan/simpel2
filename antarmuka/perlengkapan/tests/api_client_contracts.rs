//! Source-contract smoke tests: API modules added after the Pilar 1.1 error
//! migration must route every call through `client::api_*` helpers so the
//! auth token, base URL, and `AppError` mapping stay consistent. Direct use
//! of `gloo_net::http::Request` in these files would bypass those guarantees.

fn source(path_from_crate_root: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/{}",
        env!("CARGO_MANIFEST_DIR"),
        path_from_crate_root
    ))
    .expect("source file should be readable")
}

fn assert_uses_api_client_helpers(path: &str) {
    let content = source(path);
    assert!(
        content.contains("api_get")
            || content.contains("api_post")
            || content.contains("api_put")
            || content.contains("api_delete")
            || content.contains("api_patch"),
        "{path} should call client::api_* helpers instead of raw gloo_net"
    );
    assert!(
        !content.contains("gloo_net::http::Request"),
        "{path} should not invoke gloo_net::http::Request directly; go through client::api_*"
    );
}

fn assert_returns_app_error(path: &str) {
    let content = source(path);
    assert!(
        content.contains("AppError") || content.contains("AppResult"),
        "{path} should surface errors as AppError/AppResult, not ad-hoc String"
    );
}

#[test]
fn admin_api_module_uses_shared_client() {
    assert_uses_api_client_helpers("src/api/admin.rs");
    assert_returns_app_error("src/api/admin.rs");
}

#[test]
fn bank_aset_api_module_uses_shared_client() {
    assert_uses_api_client_helpers("src/api/bank_aset.rs");
    assert_returns_app_error("src/api/bank_aset.rs");
}

#[test]
fn api_client_module_owns_the_single_token_key() {
    let client = source("src/api/client.rs");
    assert!(
        client.contains("AUTH_TOKEN_KEY") && client.contains("\"auth_token\""),
        "client.rs should own a single AUTH_TOKEN_KEY constant bound to 'auth_token'"
    );
    assert!(
        client.contains("fn require_auth_token"),
        "client.rs should expose require_auth_token() for guard usage"
    );
}

#[test]
fn api_error_module_exposes_user_message() {
    let err = source("src/api/error.rs");
    assert!(
        err.contains("pub fn user_message"),
        "error.rs should expose AppError::user_message() for UI consumption"
    );
    assert!(
        err.contains("pub enum AppError"),
        "error.rs should define AppError as a typed enum"
    );
}

#[test]
fn api_mod_registers_admin_and_bank_aset_modules() {
    let api_mod = source("src/api/mod.rs");
    assert!(
        api_mod.contains("pub mod admin"),
        "api/mod.rs should re-export the admin API module"
    );
    assert!(
        api_mod.contains("pub mod bank_aset"),
        "api/mod.rs should re-export the bank_aset API module"
    );
    assert!(
        api_mod.contains("pub mod client"),
        "api/mod.rs should re-export the client module"
    );
    assert!(
        api_mod.contains("pub mod error"),
        "api/mod.rs should re-export the error module"
    );
}
